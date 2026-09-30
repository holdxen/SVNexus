//! # ssh_identity
//!
//! Discover the SSH identity files OpenSSH would try for a given target —
//! the Rust counterpart of what `openssh-portable` does across
//! `readconf.c fill_default_options()`, `readconf.c process_config_line_depth()`
//! and `ssh.c load_public_identity_files()`.
//!
//! The crate answers exactly one question: **which paths, in which order?**
//! It never opens a socket, never reads key material and never decides
//! whether a key is usable — that is the caller's job (see the flatline
//! based parsing in the application layer).
//!
//! ## Quick start
//!
//! ```no_run
//! use ssh_identity::{discover, DiscoverOptions};
//!
//! let home = std::env::home_dir().unwrap();
//! let found = discover(&DiscoverOptions::new(&home, "example.com", 22));
//!
//! for identity in &found.identities {
//!     println!("{:?} (user_provided={})", identity.path, identity.user_provided);
//! }
//! for warning in &found.warnings {
//!     eprintln!("warn: {warning}");
//! }
//! ```
//!
//! ## Fidelity notes
//!
//! * Defaults: `~/.ssh/id_rsa`, `id_ecdsa`, `id_ecdsa_sk`, `id_ed25519`,
//!   `id_ed25519_sk` — added only when the config names no `IdentityFile`
//!   at all (`readconf.c:2917`), and dropped again for `IdentityFile none`
//!   (`ssh.c:2402`).
//! * `Host` patterns use `match_pattern()` semantics: `*` and `?` only,
//!   case sensitive, no `[...]`, `!` negation vetoes the line.
//! * Only `Match all` is understood; every other `Match` condition is
//!   reported as a warning and its block ignored.
//! * `Include` resolves relative paths against `~/.ssh`, globs them, is
//!   depth-limited to `READCONF_MAX_DEPTH` (16), is parsed as "never match"
//!   when it sits inside an inactive block, and never clobbers the block
//!   state of the file that included it.
//! * Path expansion supports `~`, `~/.`, `%d %u %h %n %p %r %l %%` and
//!   `${ENV}`. Anything else (`%C`, `%k`, …) is an OpenSSH `fatal()`; here
//!   it drops that one identity with a warning.

mod config;
mod expand;
mod glob;

pub use expand::{percent_expand, tilde_expand, ExpandContext, ExpandError};
pub use glob::{glob, match_pattern};

use std::path::{Path, PathBuf};

use config::RawPath;

/// `readconf.c:2917-2931` — the built-in identity list, in OpenSSH's order.
pub const DEFAULT_IDENTITY_FILES: [&str; 5] = [
    "~/.ssh/id_rsa",
    "~/.ssh/id_ecdsa",
    "~/.ssh/id_ecdsa_sk",
    "~/.ssh/id_ed25519",
    "~/.ssh/id_ed25519_sk",
];

/// One candidate private key path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityFile {
    pub path: PathBuf,
    /// `identity_file_userprovided[]`: `true` for an `IdentityFile` line in
    /// `~/.ssh/config`, `false` for a built-in default.
    pub user_provided: bool,
}

/// The result of [`discover`].
#[derive(Debug, Default)]
pub struct Discovery {
    /// Ordered exactly as OpenSSH would try them.
    pub identities: Vec<IdentityFile>,
    /// `CertificateFile` entries, expanded. Kept separate: OpenSSH treats a
    /// certificate as its own identity.
    pub certificates: Vec<PathBuf>,
    /// `IdentitiesOnly`, `None` when the config never set it.
    pub identities_only: Option<bool>,
    /// Non-fatal problems worth logging (bad lines, unexpandable paths, …).
    pub warnings: Vec<String>,
}

/// Everything [`discover`] needs to know about the target.
#[derive(Clone, Debug)]
pub struct DiscoverOptions<'a> {
    pub home: &'a Path,
    /// The host as given by the user; `%h`, `%n` and `Host` patterns.
    pub host: &'a str,
    pub port: u16,
    /// `%r` — the remote login user, when the URL carried one.
    pub remote_user: Option<&'a str>,
    /// `%u`; defaults to `$USER` (or `$USERNAME`).
    pub local_user: Option<&'a str>,
    /// `%l`; defaults to `$HOSTNAME`.
    pub local_host: Option<&'a str>,
    /// Defaults to `<home>/.ssh/config`.
    pub config: Option<&'a Path>,
}

impl<'a> DiscoverOptions<'a> {
    pub fn new(home: &'a Path, host: &'a str, port: u16) -> Self {
        Self {
            home,
            host,
            port,
            remote_user: None,
            local_user: None,
            local_host: None,
            config: None,
        }
    }

    pub fn remote_user(mut self, user: Option<&'a str>) -> Self {
        self.remote_user = user;
        self
    }

    pub fn local_user(mut self, user: Option<&'a str>) -> Self {
        self.local_user = user;
        self
    }

    pub fn local_host(mut self, host: Option<&'a str>) -> Self {
        self.local_host = host;
        self
    }

    pub fn config(mut self, path: Option<&'a Path>) -> Self {
        self.config = path;
        self
    }
}

/// Find the identity files OpenSSH would try for `host:port`.
///
/// Missing files are **not** filtered out: OpenSSH only checks for them at
/// authentication time (`sshconnect2.c:1534`, `no such identity`), and a
/// caller that wants to log the same way needs the list as-is.
pub fn discover(options: &DiscoverOptions<'_>) -> Discovery {
    let local_user = options
        .local_user
        .map(str::to_owned)
        .or_else(|| std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok()))
        .unwrap_or_default();
    let local_host = options
        .local_host
        .map(str::to_owned)
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_default();
    let remote_user = options.remote_user.unwrap_or_default();

    let ctx = ExpandContext::new(options.home, options.host, options.port)
        .remote_user(remote_user)
        .local_user(&local_user)
        .local_host(&local_host);

    let default_config;
    let config_path = match options.config {
        Some(path) => path,
        None => {
            default_config = options.home.join(".ssh").join("config");
            &default_config
        }
    };

    let mut raw = config::parse_config(config_path, &ctx);
    let mut warnings = std::mem::take(&mut raw.warnings);

    // fill_default_options(): defaults only when the config named none.
    // Note that an `IdentityFile none` line counts here — it blocks the
    // defaults and only gets dropped further down (ssh.c:2402).
    if raw.identity_files.is_empty() {
        raw.identity_files = DEFAULT_IDENTITY_FILES
            .iter()
            .map(|path| RawPath {
                raw: (*path).to_string(),
                user_provided: false,
            })
            .collect();
    }

    let mut identities = Vec::with_capacity(raw.identity_files.len());
    for entry in raw.identity_files {
        if entry.raw.eq_ignore_ascii_case("none") {
            continue;
        }
        identities.push(expand_path(entry, options.home, &ctx, &mut warnings));
    }

    let mut certificates = Vec::with_capacity(raw.certificate_files.len());
    for entry in raw.certificate_files {
        if entry.raw.eq_ignore_ascii_case("none") {
            continue;
        }
        certificates.push(expand_path(entry, options.home, &ctx, &mut warnings).path);
    }

    Discovery {
        identities,
        certificates,
        identities_only: raw.identities_only,
        warnings,
    }
}

/// Convenience wrapper using the process home directory.
pub fn discover_for_host(host: &str, port: u16) -> Option<Discovery> {
    let home = std::env::home_dir()?;
    Some(discover(&DiscoverOptions::new(&home, host, port)))
}

fn expand_path(
    entry: RawPath,
    home: &Path,
    ctx: &ExpandContext<'_>,
    warnings: &mut Vec<String>,
) -> IdentityFile {
    // tilde first, then %/${} — ssh.c:2397-2399.
    let expanded = tilde_expand(&entry.raw, home);
    match percent_expand(&expanded, ctx) {
        Ok(path) => IdentityFile {
            path: PathBuf::from(path),
            user_provided: entry.user_provided,
        },
        Err(e) => {
            warnings.push(format!("identity file {}: {e}", entry.raw));
            IdentityFile {
                path: PathBuf::from(expanded),
                user_provided: entry.user_provided,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_util::TempDir;

    fn discover_in(dir: &TempDir, host: &str) -> Discovery {
        let config = dir.path(".ssh/config");
        let options = DiscoverOptions::new(&dir.path, host, 22)
            .remote_user(Some("bob"))
            .local_user(Some("me"))
            .local_host(Some("laptop"))
            .config(Some(&config));
        discover(&options)
    }

    #[test]
    fn defaults_when_no_config() {
        let dir = TempDir::new("defaults");
        let found = discover_in(&dir, "example.com");
        let paths: Vec<&Path> = found.identities.iter().map(|i| i.path.as_path()).collect();
        assert_eq!(
            paths,
            [
                dir.path.join(".ssh/id_rsa"),
                dir.path.join(".ssh/id_ecdsa"),
                dir.path.join(".ssh/id_ecdsa_sk"),
                dir.path.join(".ssh/id_ed25519"),
                dir.path.join(".ssh/id_ed25519_sk"),
            ]
        );
        assert!(found.identities.iter().all(|i| !i.user_provided));
        assert_eq!(found.identities_only, None);
        assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    }

    #[test]
    fn config_identity_file_replaces_defaults() {
        let dir = TempDir::new("config_wins");
        dir.file(".ssh/config", "Host *\nIdentityFile ~/.ssh/custom\n");
        let found = discover_in(&dir, "example.com");
        assert_eq!(found.identities.len(), 1);
        assert_eq!(found.identities[0].path, dir.path.join(".ssh/custom"));
        assert!(found.identities[0].user_provided);
    }

    #[test]
    fn identity_file_none_blocks_defaults() {
        let dir = TempDir::new("none");
        dir.file(".ssh/config", "IdentityFile none\n");
        let found = discover_in(&dir, "example.com");
        // The `none` entry still counts towards "the config named an
        // IdentityFile", so no defaults are added, and it is dropped when
        // the paths are loaded (ssh.c:2402).
        assert!(found.identities.is_empty(), "{:?}", found.identities);
    }

    #[test]
    fn expansion_is_applied_to_discovered_paths() {
        let dir = TempDir::new("expand");
        dir.file(
            ".ssh/config",
            "Host example.com\nIdentityFile ~/.ssh/%h_%p_key\nIdentityFile %u_key\n",
        );
        let found = discover_in(&dir, "example.com");
        let paths: Vec<&Path> = found.identities.iter().map(|i| i.path.as_path()).collect();
        assert_eq!(
            paths,
            [
                dir.path.join(".ssh/example.com_22_key"),
                PathBuf::from("me_key"),
            ]
        );
    }

    #[test]
    fn unexpandable_path_is_warned_about() {
        let dir = TempDir::new("bad_expand");
        dir.file(".ssh/config", "IdentityFile ~/.ssh/%C_key\nIdentityFile ~/.ssh/ok\n");
        let found = discover_in(&dir, "example.com");
        assert_eq!(found.identities.len(), 2);
        assert!(found.warnings.iter().any(|w| w.contains("%C")), "{:?}", found.warnings);
    }

    #[test]
    fn identities_only_and_certificates_are_reported() {
        let dir = TempDir::new("meta");
        dir.file(
            ".ssh/config",
            "IdentitiesOnly yes\nIdentityFile ~/.ssh/key\nCertificateFile ~/.ssh/key-cert.pub\n",
        );
        let found = discover_in(&dir, "example.com");
        assert_eq!(found.identities_only, Some(true));
        assert_eq!(found.certificates, [dir.path.join(".ssh/key-cert.pub")]);
    }

    #[test]
    fn host_scoped_discovery() {
        let dir = TempDir::new("scoped");
        dir.file(
            ".ssh/config",
            "Host example.com\nIdentityFile ~/.ssh/work\nHost *\nIdentityFile ~/.ssh/any\n",
        );
        let work = discover_in(&dir, "example.com");
        assert_eq!(work.identities.len(), 2);

        let other = discover_in(&dir, "other.test");
        assert_eq!(other.identities.len(), 1);
        assert_eq!(other.identities[0].path, dir.path.join(".ssh/any"));
    }
}
