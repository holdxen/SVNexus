//! SSH identity discovery and loading — the counterpart of OpenSSH's
//! `ssh.c:2347 load_public_identity_files()`, `sshconnect2.c:1677 pubkey_prepare()`
//! and `authfile.c`.
//!
//! Which paths to try comes from the `ssh_identity` crate (OpenSSH config
//! semantics: defaults, `Host` scoping, `Include`, `%` expansion). This
//! module only decides **what is in those paths**, so that the tunnel can:
//!
//! 1. try candidates in OpenSSH's order, skipping missing/unusable ones with
//!    a log line instead of failing the connection —
//!    `sshconnect2.c:1529 do_log2("no such identity: %s")`;
//! 2. ask the frontend for a passphrase **only** when a candidate really is
//!    encrypted (`sshconnect2.c:1541`, first try with an empty passphrase,
//!    then prompt);
//! 3. treat an empty/cancelled passphrase as "skip this key", not as a
//!    connection error — `debug2("no passphrase given, try next key")`.
//!
//! Deliberately out of scope for now (same as the agreed v1):
//! ssh-agent, PKCS#11, `sk-*` hardware keys, certificates and the
//! `-cert.pub` auto-discovery.

use std::path::{Path, PathBuf};

use flatline::key::{Error as KeyError, Parser, Public};

/// One path suggested by `ssh_identity::discover()`, ready to be tried.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    /// `identity_file_userprovided[]`: `false` for the built-in defaults,
    /// which OpenSSH logs at a quieter level.
    pub user_provided: bool,
}

/// What a [`Candidate`] turned out to contain.
#[derive(Debug)]
pub enum Identity {
    /// Nothing at that path — keep going with the next candidate.
    Missing,
    /// Private key usable as-is; hand `bytes` straight to
    /// `Session::authenticate_public_key`.
    Plain {
        bytes: Vec<u8>,
        key_type: String,
        comment: String,
    },
    /// Encrypted private key: supply a passphrase to [`load`] to get
    /// [`Identity::Plain`] or [`Identity::WrongPassphrase`].
    Encrypted {
        /// From `<path>.pub` when one exists — the private blob itself is
        /// unreadable without the passphrase.
        key_type: Option<String>,
    },
    /// The supplied passphrase did not work — prompt again.
    WrongPassphrase { detail: String },
    /// Tier 1/2 of `authfile.c:253 sshkey_load_public()` found a public key,
    /// but there is no private key we could sign with.
    PublicOnly {
        key_type: String,
        comment: Option<String>,
        detail: String,
    },
    /// Unreadable, unsupported or corrupt.
    Unusable { detail: String },
}

/// Every identity path OpenSSH would consider for `host:port`, in order.
///
/// Warnings from `~/.ssh/config` parsing are logged, not returned: a broken
/// config line must never stop the connection.
pub fn discover(host: &str, port: u16, remote_user: Option<&str>) -> Vec<Candidate> {
    match std::env::home_dir() {
        Some(home) => discover_in(&home, host, port, remote_user),
        None => {
            tracing::warn!("No home directory, SSH identity discovery skipped");
            Vec::new()
        }
    }
}

/// [`discover`] with an explicit home directory (used by tests).
pub fn discover_in(
    home: &Path,
    host: &str,
    port: u16,
    remote_user: Option<&str>,
) -> Vec<Candidate> {
    let found = ssh_identity::discover(
        &ssh_identity::DiscoverOptions::new(home, host, port).remote_user(remote_user),
    );

    for warning in &found.warnings {
        tracing::warn!("ssh config: {warning}");
    }

    found
        .identities
        .into_iter()
        .map(|identity| Candidate {
            path: identity.path,
            user_provided: identity.user_provided,
        })
        .collect()
}

/// Read and classify one candidate.
///
/// `passphrase` is `None` on the first attempt, exactly like
/// `sshconnect2.c load_identity_file()` which starts with an empty
/// passphrase before prompting.
pub fn load(candidate: &Candidate, passphrase: Option<&[u8]>) -> Identity {
    let path = &candidate.path;

    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // load_identity_file(): info for user-configured keys, debug3
            // for the built-in defaults.
            if candidate.user_provided {
                tracing::info!("no such identity: {}: {}", path.display(), e);
            } else {
                tracing::debug!("no such identity: {}: {}", path.display(), e);
            }
            return Identity::Missing;
        }
        Err(e) => return Identity::Unusable {
            detail: format!("can't read: {e}"),
        },
    };

    if let Some(warning) = permission_warning(path) {
        tracing::warn!("{warning}");
    }

    // Never hand an encrypted PEM to OpenSSL without a passphrase: it would
    // prompt on stdin (\"Enter PEM pass phrase:\") and block the tunnel.
    if passphrase.is_none() && is_pem_encrypted(&bytes) {
        tracing::debug!("identity file {} is encrypted", path.display());
        let key_type = public_key_info(path, &bytes).map(|(r#type, _)| r#type);
        return Identity::Encrypted { key_type };
    }

    let parser = Parser::default();
    match parser.parse_private_key_file(&bytes, passphrase) {
        Ok(private) => Identity::Plain {
            bytes,
            key_type: private.r#type,
            comment: private.comment,
        },
        Err(error) => classify(path, &bytes, passphrase, error),
    }
}

/// Decide what a failed private-key parse means.
fn classify(
    path: &Path,
    bytes: &[u8],
    passphrase: Option<&[u8]>,
    error: flatline::error::Error,
) -> Identity {
    // Three ways a parse failure means "this key is encrypted":
    //  * flatline had no passphrase for an encrypted OpenSSH-format key;
    //  * the traditional PEM header says so;
    //  * a passphrase was tried but was wrong — OpenSSH reports that as a
    //    checksum mismatch, so re-probe without a passphrase to be sure.
    let encrypted = is_wrong_passphrase(&error)
        || is_pem_encrypted(bytes)
        || (passphrase.is_some() && probe_needs_passphrase(bytes));

    if encrypted {
        if passphrase.is_none() {
            tracing::debug!("identity file {} is encrypted", path.display());
            let key_type = public_key_info(path, bytes).map(|(r#type, _)| r#type);
            return Identity::Encrypted { key_type };
        }
        return Identity::WrongPassphrase {
            detail: error.to_string(),
        };
    }

    if passphrase.is_some() {
        // A passphrase was handed in for a key that does not need one (or
        // whose real problem is something else) — no point prompting again.
        return Identity::Unusable {
            detail: error.to_string(),
        };
    }

    // authfile.c:253 `sshkey_load_public()` — tier 1 and 2 here, tier 3 is
    // `sshkey_load_pubkey_from_private()`, which is exactly the parse that
    // just failed.
    if let Some((key_type, comment)) = parse_public(bytes) {
        return Identity::PublicOnly {
            key_type,
            comment,
            detail: "only a public key is stored here".to_string(),
        };
    }
    if let Some((key_type, comment)) = sidecar_public(path) {
        return Identity::PublicOnly {
            key_type,
            comment,
            detail: format!("private key unusable: {error}"),
        };
    }

    Identity::Unusable {
        detail: error.to_string(),
    }
}

fn is_wrong_passphrase(error: &flatline::error::Error) -> bool {
    matches!(
        error,
        flatline::error::Error::KeyError {
            source: KeyError::WrongPassphrase
        }
    )
}

/// Try to parse without a passphrase: only an encrypted OpenSSH-format key
/// fails that way.
fn probe_needs_passphrase(bytes: &[u8]) -> bool {
    matches!(
        Parser::default().parse_private_key_file(bytes, None),
        Err(error) if is_wrong_passphrase(&error)
    )
}

/// `authfile.c:82 sshkey_perm_ok()`.
///
/// OpenSSH refuses such a key outright; we only warn, because a GUI client
/// may legitimately share a key with other local accounts. Files not owned
/// by us are not checked at all — neither does OpenSSH.
#[cfg(unix)]
pub fn permission_warning(path: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;

    let meta = std::fs::metadata(path).ok()?;
    // No libc here: "owned by me" is decided by comparing against the owner
    // of the home directory.
    let home = std::env::home_dir()?;
    let home_meta = std::fs::metadata(&home).ok()?;
    if meta.uid() != home_meta.uid() {
        return None;
    }
    if meta.mode() & 0o077 == 0 {
        return None;
    }
    Some(format!(
        "Permissions 0{:03o} for '{}' are too open, private key files should not be accessible by others",
        meta.mode() & 0o777,
        path.display()
    ))
}

/// Unix mode bits don't exist on Windows — key files there are protected
/// by ACLs, which is a different check; skip it for now.
#[cfg(not(unix))]
pub fn permission_warning(_path: &Path) -> Option<String> {
    None
}

/// Encryption is announced in the header for both PEM flavours: the
/// traditional one carries `Proc-Type: 4,ENCRYPTED`, PKCS#8 uses
/// `-----BEGIN ENCRYPTED PRIVATE KEY-----`. The OpenSSH format instead
/// stores a cipher name in its base64 body (covered by flatline returning
/// `WrongPassphrase`).
fn is_pem_encrypted(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return false;
    };
    (text.contains("Proc-Type:") && text.contains("ENCRYPTED"))
        || text.contains("BEGIN ENCRYPTED PRIVATE KEY")
}

/// Tier 1 (the file itself) then tier 2 (`<path>.pub`) of
/// `authfile.c:253 sshkey_load_public()`.
fn public_key_info(path: &Path, bytes: &[u8]) -> Option<(String, Option<String>)> {
    parse_public(bytes).or_else(|| sidecar_public(path))
}

fn sidecar_public(path: &Path) -> Option<(String, Option<String>)> {
    let mut name = path.as_os_str().to_os_string();
    name.push(".pub");
    let bytes = std::fs::read(PathBuf::from(name)).ok()?;
    parse_public(&bytes)
}

fn parse_public(bytes: &[u8]) -> Option<(String, Option<String>)> {
    match Parser::default().parse_public_key_file(bytes) {
        Ok(Public::Normal { r#type, comment, .. })
        | Ok(Public::Certificate { r#type, comment, .. }) => {
            // `.pub` files end with a newline, flatline keeps it in the
            // trailing comment.
            Some((r#type, comment.map(|c| c.trim().to_string())))
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const PLAIN_ED25519: &str = "\
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAMwAAAAtzc2gtZW
QyNTUxOQAAACBsHE+vtToQGwZ+utTPLX2eQS8IF2d+TuolOI8QXDxcBAAAAJDyFTdW8hU3
VgAAAAtzc2gtZWQyNTUxOQAAACBsHE+vtToQGwZ+utTPLX2eQS8IF2d+TuolOI8QXDxcBA
AAAEAYxhd7hqYACAZcAdyGBLy3G7jo2NYiNz+2KdqZQzitqmwcT6+1OhAbBn661M8tfZ5B
LwgXZ35O6iU4jxBcPFwEAAAADXBsYWluQGV4YW1wbGU=
-----END OPENSSH PRIVATE KEY-----
";

    const PLAIN_ED25519_PUB: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGwcT6+1OhAbBn661M8tfZ5BLwgXZ35O6iU4jxBcPFwE plain@example\n";

    const ENCRYPTED_ED25519: &str = "\
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABCvmItPnO
eD2LBXuF+gvoFWAAAAGAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIJyGIa5a4U9fq4nE
T3DZ9+2Kq8f/Xt38Gp9z//sjWzdbAAAAkCTOLULhfUSOBoRVTcueoa3K7SFXyT0ZrBmfeb
FNk1Jk5VyrvQc0+KU0NUlV6Knti84U1e6gCn79fWDQURFf8hAD5pGX3jiT9vvAVRr2I2+3
VcgkFouBDUpjEcgWk+5Pg+pm4HmPBhRc9mvgMH+0TshCkJGFdtMl3r7PwNKX+v1//u49Io
rjQSA1UacTtHgtMA==
-----END OPENSSH PRIVATE KEY-----
";

    const ENCRYPTED_ED25519_PUB: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJyGIa5a4U9fq4nET3DZ9+2Kq8f/Xt38Gp9z//sjWzdb enc@example\n";

    /// Traditional PEM (PKCS#1) encrypted with `hunter2` — the other
    /// encryption style `is_pem_encrypted()` has to recognise.
    const ENCRYPTED_PEM_RSA: &str = "\
-----BEGIN RSA PRIVATE KEY-----
Proc-Type: 4,ENCRYPTED
DEK-Info: AES-128-CBC,2E9B15E25D2814D3D14C980EB361DF65

OOgasi41JDI8BWOcr4EZbfdqTa5jRU3DVT4PwnjmjZDw/aOm9ecMBZurFSimAUMQ
7dnHdgYoN7ejBwIzHc3343EChqbADe9evQTh3JTfEvp+iDGW37LTiZhXFA/PTAQB
fBS+ZIESrVvjtkzlk2uluoFJk8hnZs3OJM09A4WDFBK6fRhGqb01GYj0boTDKjox
7VK7CS7/eXL7vGWtrKeVhARsDuJOuop20e4R1RznyOY3AqP7xWSZpqz4pVYTMpYf
TJQ7U+xQjg3hgx0eEg2Ek43hAYPKdOLSU2E0BNDUp2PHAmjk/bMphLYT0RyzuSOp
d+rlXcgHPFb3fzlc0YL7+dLrMVWtXIJnkd4tnBzPutak6hqSyHz44PF75G29VEQg
F0z7/Xs0xY6az1nqNczn9sEJKhqsuiNKf8v67m9gnRlUtteAkIVV5Pvm4dcs12+E
YJmAjGwDDTpRihcKhIyHM5BAA2NaHeygdgBqBAM8Rz2lpHHwCMri7ml2gwfsazzX
appQT/noeIEJ6CUD6dYdCUeskbptmphGeBCKVvJLRljr2oa+gOCVL85elgQotPT9
g4HM+uUkr6jzhrYGOZ6JyrnoHR8CvERgD2/NGhSYdBfB+V0uRqpdZacQ+pVfJqAq
ojWB2aWNkNmiwH8bp1rhhPTr+yv3koy2mPTJdr8XIL/B1eKzfiXjL3JnArQlCyzV
2ru5UND/lUBawbpe0kS5cIZtmbxTueE2SYt+kDJ8NcVylrgLOibpOGSkw0yCUEMz
Puj01pQ5gmjtKGeBrj3tzfnSU8b9+GFkGWkfOIpmwg5a6fJvuUiQ8uFxDp1N3/Uh
23D25KPdiiRJF67bmY9XPXrCKB2X4O8AL8uYz5EU89DBmovRA3UKlMVbguyDJJqj
Ofgnc3sduMNcNenuDKjtdntno+Clef01twUCarkxpagFszDVpevemqENvoRBmDrb
k7VIWj4mDCrNag1JuqD7CZLlgVit3oEpoi7k6hsXmzaKy/JgPEE0nYMxQyG9/oAa
N8VrR+Ndi9l2QHx0NF/GFEhDP+Nwfq3Zfog14UfltE7tWb7vfk3cfZPLCwGQLvLG
jFSaSndb4S9quKhV9UruaYn9UaRdVRR8R1nyK0U8769RU5sUjg9A3EQil7oLAV29
EpGE1TByDrV1J3pxrNRK4Tt7+fPZWxJzS8lrLf4sHJDwYBmCQHuCEHTwgxu11iQS
XRjDAk++GeeUaXD+0wfTc5Hgy54kJ6CcNd5hjE8+g426MUxyT3OcDN2Ng0vTQSBz
/gsV1hAkLj4SM0xQn1/8sqMKfMIhzpirH2dPyZ3O+6e6G3W7VU9XR2Iaen/kGic8
NIE69zYyqLegLgzClOoPNciyJ8+J0lC88rgvl9CbGFFmo/wB6q359M0JyZzx17mM
wehPgRuA8tHYcaZYWv2UTZYLvrsELyk8qMYCPGR+BR20KwwXo23dgtgTKGE5awCM
JRNeEL8RfVdMpXBpxcsXKXr03BBFNa2IR6hrWbRX/TmSHR70rApHQeioJAqKWfMR
IiLXyyl8ljlPZ3T0/bZWjp7gOLO4Xd1Wl506+kpGOtQWyFoasqQrh0DaMELnoMCs
-----END RSA PRIVATE KEY-----
";

    fn candidate(path: PathBuf) -> Candidate {
        Candidate {
            path,
            user_provided: true,
        }
    }

    fn write(dir: &TempDir, name: &str, content: &str) -> Candidate {
        let path = dir.path().join(name);
        std::fs::write(&path, content).expect("write fixture");
        candidate(path)
    }

    #[test]
    fn plain_openssh_key_loads_without_passphrase() {
        let dir = TempDir::new().unwrap();
        let key = write(&dir, "id_ed25519", PLAIN_ED25519);

        match load(&key, None) {
            Identity::Plain {
                bytes,
                key_type,
                comment,
            } => {
                assert_eq!(key_type, "ssh-ed25519");
                assert_eq!(comment, "plain@example");
                assert_eq!(bytes, PLAIN_ED25519.as_bytes());
            }
            other => panic!("expected Plain, got {other:?}"),
        }
    }

    #[test]
    fn encrypted_key_reports_encrypted_then_decrypts() {
        let dir = TempDir::new().unwrap();
        let key = write(&dir, "id_enc", ENCRYPTED_ED25519);
        // tier 2 of sshkey_load_public(): the type comes from the sidecar,
        // the private blob itself is unreadable without the passphrase.
        std::fs::write(dir.path().join("id_enc.pub"), ENCRYPTED_ED25519_PUB).unwrap();

        match load(&key, None) {
            Identity::Encrypted { key_type, .. } => {
                assert_eq!(key_type.as_deref(), Some("ssh-ed25519"));
            }
            other => panic!("expected Encrypted, got {other:?}"),
        }

        assert!(
            matches!(
                load(&key, Some(b"wrong")),
                Identity::WrongPassphrase { .. }
            ),
            "a wrong passphrase must be reported as such so we can re-prompt"
        );

        assert!(
            matches!(load(&key, Some(b"hunter2")), Identity::Plain { .. }),
            "the fixture must actually decrypt — bcrypt-kdf + aes256-ctr"
        );
    }

    #[test]
    fn traditional_pem_encryption_is_detected() {
        let dir = TempDir::new().unwrap();
        let key = write(&dir, "id_rsa", ENCRYPTED_PEM_RSA);

        assert!(
            matches!(load(&key, None), Identity::Encrypted { .. }),
            "Proc-Type: 4,ENCRYPTED must be recognised"
        );
        assert!(matches!(
            load(&key, Some(b"wrong")),
            Identity::WrongPassphrase { .. }
        ));
        match load(&key, Some(b"hunter2")) {
            Identity::Plain { key_type, .. } => assert_eq!(key_type, "ssh-rsa"),
            other => panic!("expected Plain, got {other:?}"),
        }
    }

    #[test]
    fn public_only_file_is_reported_not_failed() {
        let dir = TempDir::new().unwrap();
        let key = write(&dir, "id_only_pub", PLAIN_ED25519_PUB);

        match load(&key, None) {
            Identity::PublicOnly {
                key_type,
                comment,
                detail,
            } => {
                assert_eq!(key_type, "ssh-ed25519");
                assert_eq!(comment.as_deref(), Some("plain@example"));
                assert!(detail.contains("public key"), "{detail}");
            }
            other => panic!("expected PublicOnly, got {other:?}"),
        }
    }

    #[test]
    fn pub_sidecar_is_the_second_tier() {
        let dir = TempDir::new().unwrap();
        // A private key file we cannot use at all…
        let key = write(&dir, "broken", "not really a key\n");
        // …but the .pub next to it is fine.
        std::fs::write(dir.path().join("broken.pub"), PLAIN_ED25519_PUB).unwrap();

        match load(&key, None) {
            Identity::PublicOnly {
                key_type, detail, ..
            } => {
                assert_eq!(key_type, "ssh-ed25519");
                assert!(detail.contains("private key unusable"), "{detail}");
            }
            other => panic!("expected PublicOnly, got {other:?}"),
        }
    }

    #[test]
    fn missing_and_unusable_candidates() {
        let dir = TempDir::new().unwrap();

        let missing = candidate(dir.path().join("does_not_exist"));
        assert!(matches!(load(&missing, None), Identity::Missing));
        // The built-in defaults stay quiet; user-configured paths are logged
        // at info level (sshconnect2.c:1529).
        let mut quiet = missing;
        quiet.user_provided = false;
        assert!(matches!(load(&quiet, None), Identity::Missing));

        // A passphrase handed to a key that does not need one is ignored,
        // exactly like sshkey_load_private_type() does.
        let plain = write(&dir, "id_ed25519", PLAIN_ED25519);
        assert!(matches!(load(&plain, Some(b"stray")), Identity::Plain { .. }));
    }

    #[test]
    #[cfg(unix)]
    fn open_permissions_are_warned_about() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("id_ed25519");
        std::fs::write(&path, PLAIN_ED25519).unwrap();

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(permission_warning(&path), None);

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let warning = permission_warning(&path).expect("0644 must warn");
        assert!(warning.contains("too open"), "{warning}");
        assert!(warning.contains("0644"), "{warning}");
    }

    #[test]
    fn discovery_follows_ssh_config() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".ssh")).unwrap();
        std::fs::write(
            dir.path().join(".ssh/config"),
            "Host example.com\nIdentityFile ~/.ssh/work\nHost *\nIdentityFile ~/.ssh/any\n",
        )
        .unwrap();

        let work = discover_in(dir.path(), "example.com", 22, None);
        assert_eq!(
            work,
            [
                Candidate {
                    path: dir.path().join(".ssh/work"),
                    user_provided: true
                },
                Candidate {
                    path: dir.path().join(".ssh/any"),
                    user_provided: true
                },
            ]
        );

        let other = discover_in(dir.path(), "other.test", 22, Some("bob"));
        assert_eq!(
            other,
            [Candidate {
                path: dir.path().join(".ssh/any"),
                user_provided: true
            }]
        );
    }

    #[test]
    fn defaults_are_discovered_when_no_config_exists() {
        let dir = TempDir::new().unwrap();
        let found = discover_in(dir.path(), "example.com", 22, None);
        let paths: Vec<&Path> = found.iter().map(|c| c.path.as_path()).collect();
        assert_eq!(
            paths,
            [
                dir.path().join(".ssh/id_rsa"),
                dir.path().join(".ssh/id_ecdsa"),
                dir.path().join(".ssh/id_ecdsa_sk"),
                dir.path().join(".ssh/id_ed25519"),
                dir.path().join(".ssh/id_ed25519_sk"),
            ]
        );
        assert!(found.iter().all(|c| !c.user_provided));
    }
}
