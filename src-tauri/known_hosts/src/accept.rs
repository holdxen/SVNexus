//! Accept-and-store flow modelled on OpenSSH's `check_host_key()`
//! (sshconnect.c): read known_hosts, classify the presented key, invoke
//! a confirmation callback exactly where OpenSSH would prompt, and append
//! the entry when the user accepts.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::check::{check_hostkey, key_eq, key_loadable, HostStatus};
use crate::entry_matches;
use crate::fingerprint::fingerprint;
use crate::types::{HostEntry, HostPattern, KeyType, Marker};
use crate::writer::{WriteError, WriteStyle};

/// Characters OpenSSH refuses to record in a host name
/// (sshconnect.c:985-989 sets the write path readonly for these).
/// `[` / `]` are absent because `host` may legitimately be the
/// `put_host_port()` form (`[host]:port`), which OpenSSH validates
/// before adding those brackets — the caller is responsible for not
/// passing raw bracketed names.
const INVALID_HOST_CHARS: [char; 8] = ['@', '?', '*', '#', '|', '\'', '"', '\\'];

/// Certificate type-name suffix (`sshkey_is_cert`); OpenSSH never
/// auto-stores certificates (`readonly || want_cert`, sshconnect.c:1153).
const CERT_SUFFIX: &str = "-cert-v01@openssh.com";

/// Cap on `known_elsewhere`, mirroring `MAX_OTHER_NAMES` (sshconnect.c:821).
const MAX_OTHER_NAMES: usize = 8;

/// Payload of OpenSSH's first-connection prompt
/// (sshconnect.c:1166-1212, `StrictHostKeyChecking ask` + `HOST_NEW`).
#[derive(Debug, Clone)]
pub struct NewHostPrompt<'a> {
    /// Host name being recorded (`put_host_port` + lowercased form).
    pub host: &'a str,
    /// IP address, when the caller supplied one.
    pub ip: Option<&'a str>,
    /// Type of the presented key.
    pub key_type: &'a KeyType,
    /// Wire blob of the presented key.
    pub key_data: &'a [u8],
    /// Fingerprint of the presented key (e.g. `SHA256:…`).
    pub fingerprint: String,
    /// Other known_hosts names that already trust this same key
    /// (`other_hostkeys_message`, cap 8). Non-empty means the status is
    /// [`MismatchHost`](HostStatus::MismatchHost): first contact for
    /// *this* name, but the key is known elsewhere.
    pub known_elsewhere: Vec<String>,
    /// Whether the host already has known keys of a *different* type
    /// (`show_other_keys`: "but keys of different type are already
    /// known for this host.").
    pub other_type_known: bool,
}

/// Payload of OpenSSH's IP-mismatch prompt
/// (sshconnect.c:1388-1406): the host key is trusted (or was just
/// accepted), but the key known for the IP address differs.
#[derive(Debug, Clone)]
pub struct IpChangedPrompt<'a> {
    /// Host name whose key is trusted / just accepted.
    pub host: &'a str,
    /// IP address whose known_hosts key differs.
    pub ip: &'a str,
    /// Type of the presented (host) key.
    pub key_type: &'a KeyType,
    /// Wire blob of the presented (host) key.
    pub key_data: &'a [u8],
    /// Fingerprint of the presented key.
    pub fingerprint: String,
}

/// Every point at which OpenSSH's `check_host_key()` calls `confirm()`
/// (sshconnect.c:562). The callback receives this enum so it can render
/// the right prompt for each situation.
#[derive(Debug, Clone)]
pub enum HostkeyConfirm<'a> {
    /// Unknown host under `StrictHostKeyChecking ask` — "The authenticity
    /// of host … can't be established. Are you sure you want to continue?"
    NewHost(NewHostPrompt<'a>),
    /// Host key trusted but the IP-scope key differs — "… differs from
    /// the key for the IP address … Are you sure you want to continue?"
    IpChanged(IpChangedPrompt<'a>),
}

/// Warnings OpenSSH prints as `error()`/`logit()` output **without**
/// asking for confirmation, delivered through the `warning` callback.
/// Line references are to sshconnect.c; suggested wording is modelled
/// on OpenSSH's own output. The callback returns `()` — like OpenSSH's
/// log lines, warnings cannot change control flow (refusal/failure
/// comes from the status, prompts from [`HostkeyConfirm`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostkeyWarning<'a> {
    /// `HOST_REVOKED` — "@@@@ WARNING: REVOKED HOST KEY DETECTED! @@@@"
    /// (sshconnect.c:1246-1253). Connection must be refused; OpenSSH
    /// never offers an override here.
    HostKeyRevoked {
        /// Host name checked.
        host: &'a str,
        /// Type name of the presented key (e.g. `ssh-ed25519`).
        key_type: &'a str,
    },
    /// `HOST_CHANGED` — `warn_changed_key()` banner "REMOTE HOST
    /// IDENTIFICATION HAS CHANGED!" (sshconnect.c:1301, 1670-1690).
    /// Connection must be refused.
    ///
    /// When `dns_spoofing` is true (`HOST_CHANGED_IP_DIFFERS`), OpenSSH
    /// prints the extra "POSSIBLE DNS SPOOFING DETECTED" banner first
    /// (sshconnect.c:1280-1299) and `ip` is the peer address involved.
    HostKeyChanged {
        /// Host name checked.
        host: &'a str,
        /// Type name of the presented (unexpected) key.
        key_type: &'a str,
        /// Fingerprint of the presented key, as `warn_changed_key` shows.
        fingerprint: String,
        /// Peer address, when the caller supplied one.
        ip: Option<&'a str>,
        /// True when the IP-scope key disagrees too (DNS-spoofing signal).
        dns_spoofing: bool,
    },
    /// Entry appended — "Warning: Permanently added '\<host\>' (\<type\>)
    /// to the list of known hosts." (sshconnect.c:1243-1244).
    PermanentlyAdded {
        /// Host name stored.
        host: &'a str,
        /// Type name stored.
        key_type: &'a str,
        /// Fingerprint of the stored key.
        fingerprint: String,
        /// Whether the line was written hashed (`|1|…`).
        hashed: bool,
    },
    /// `HOST_OK` but the IP scope has no key — sshconnect.c:1114-1116
    /// logs "… host key for IP address '…' not in list of known hosts"
    /// (this crate never writes IP lines, so the condition persists
    /// across connections; OpenSSH with `CheckHostIP=yes` would have
    /// added one on first contact).
    IpKeyUnknown {
        /// Host name that matched.
        host: &'a str,
        /// IP address whose scope has no key.
        ip: &'a str,
        /// Type name of the known (matching) key.
        key_type: &'a str,
    },
}

/// Outcome of [`check_and_store_hostkey`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreResult {
    /// Host-scope classification from [`check_hostkey`](crate::check_hostkey).
    pub status: HostStatus,
    /// Whether a line for `host` was appended in this call.
    pub stored: bool,
    /// `false` if the user declined any prompt. Note that a declined
    /// [`IpChanged`](HostkeyConfirm::IpChanged) prompt can follow an
    /// accepted [`NewHost`](HostkeyConfirm::NewHost) — OpenSSH writes the
    /// entry first (sshconnect.c:1234) and fails the connection later
    /// (sshconnect.c:1405), so `stored` may be `true` here.
    pub accepted: bool,
}

/// Error produced by [`check_and_store_hostkey`].
#[derive(Debug)]
pub enum StoreError {
    /// Reading or opening known_hosts failed (a missing file is not an
    /// error — it is treated as empty).
    Io(io::Error),
    /// Serializing the entry failed.
    Write(WriteError),
    /// The host name contains characters OpenSSH refuses to record
    /// (sshconnect.c:985-989): no prompt, no write.
    InvalidHost,
    /// The presented key is a certificate; OpenSSH never auto-stores
    /// those (`want_cert`, sshconnect.c:1153): no prompt, no write.
    Certificate,
    /// No user known_hosts file is configured to write to. Mirrors
    /// OpenSSH's `readonly` handling: with `num_user_hostfiles == 0`
    /// a new host key fails before any prompt
    /// (sshconnect.c:1051-1053, 1153).
    ReadOnly,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(e) => write!(f, "known_hosts I/O error: {e}"),
            StoreError::Write(e) => write!(f, "known_hosts write error: {e}"),
            StoreError::InvalidHost => write!(
                f,
                "host name contains characters OpenSSH refuses to record"
            ),
            StoreError::Certificate => write!(f, "certificate host keys are never stored"),
            StoreError::ReadOnly => {
                write!(f, "no user known_hosts file available for writing")
            }
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Io(e) => Some(e),
            StoreError::Write(e) => Some(e),
            StoreError::InvalidHost | StoreError::Certificate | StoreError::ReadOnly => None,
        }
    }
}

impl From<io::Error> for StoreError {
    fn from(e: io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<WriteError> for StoreError {
    fn from(e: WriteError) -> Self {
        StoreError::Write(e)
    }
}

/// Check a presented host key against `path` and, where OpenSSH would
/// prompt the user, ask `confirm`; on acceptance append the entry.
///
/// Flow (mirroring `check_host_key()` in sshconnect.c):
/// 1. Invalid host characters → [`StoreError::InvalidHost`]; certificate
///    type names → [`StoreError::Certificate`] (both before any prompt).
/// 2. Read `path` (a missing file counts as empty) and classify with
///    [`check_hostkey`](crate::check_hostkey).
/// 3. [`HOST_NEW`](HostStatus::HOST_NEW) /
///    [`MismatchHost`](HostStatus::MismatchHost) → invoke
///    [`HostkeyConfirm::NewHost`]; declining returns without writing,
///    accepting appends one line for `host` (plain or hashed per
///    `hash_known_hosts`, aligned with `ssh-keygen -H` / `HashKnownHosts`).
/// 4. `HOST_OK` / `HOST_CHANGED` / `HOST_CHANGED_IP_DIFFERS` /
///    `HOST_REVOKED` → no prompt, no write (OpenSSH never offers
///    yes/no there).
/// 5. When `ip` is `Some`, the host status is not `HOST_CHANGED` (nor
///    its IP-differs form), and the IP scope alone evaluates to
///    `HOST_CHANGED` → invoke [`HostkeyConfirm::IpChanged`]
///    (sshconnect.c:1388-1406). Declining sets
///    [`accepted`](StoreResult::accepted) to `false` but leaves any
///    already-written `NewHost` entry in place — exactly what OpenSSH
///    does (write at :1234, fail at :1405).
///
/// Notes:
/// - `host` must be the final `put_host_port()` + lowercased form, the
///   same contract as [`lookup`](crate::lookup) and
///   [`check_hostkey`](crate::check_hostkey).
/// - Only the `host` line is ever written; the CheckHostIP silent
///   ip-line additions (sshconnect.c:1112-1126, 1218-1237) are not
///   implemented.
/// - `UpdateHostkeys ask` rotation (clientloop.c) is a different flow
///   and is out of scope.
/// - This is the single-file form; for `UserKnownHostsFile` /
///   `GlobalKnownHostsFile` lists use
///   [`check_and_store_hostkey_in_files`], which merges user files
///   first, then system files (read), and writes only the first user
///   file.
/// - `confirm` is called at most twice (once per prompt kind). Policy
///   lives in the callback: `accept-new` is `true` for every `NewHost`,
///   strict prompting is a real question, `no` is always `false`.
/// - `warning` reports the messages OpenSSH logs without asking (see
///   [`HostkeyWarning`]); it returns `()` and cannot change the
///   outcome.
///
/// ```no_run
/// use std::path::Path;
/// use known_hosts_parser::{check_and_store_hostkey, HostkeyConfirm, StoreError};
///
/// # fn demo(key_blob: &[u8]) -> Result<(), StoreError> {
/// let result = check_and_store_hostkey(
///     Path::new("known_hosts"),
///     "example.com",
///     None,
///     "ssh-ed25519",
///     key_blob,
///     true, // HashKnownHosts
///     |confirm| match confirm {
///         HostkeyConfirm::NewHost(prompt) => {
///             println!("trust {} ({})?", prompt.host, prompt.fingerprint);
///             true // accept-new style
///         }
///         HostkeyConfirm::IpChanged(prompt) => {
///             println!("ip {} key differs for {}", prompt.ip, prompt.host);
///             true
///         }
///     },
///     |_| {}, // warning callback — display-only, see HostkeyWarning
/// )?;
/// println!(
///     "status={:?} stored={} accepted={}",
///     result.status, result.stored, result.accepted
/// );
/// Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn check_and_store_hostkey<'a, F, W>(
    path: &Path,
    host: &'a str,
    ip: Option<&'a str>,
    key_type: &'a str,
    key_data: &[u8],
    hash_known_hosts: bool,
    confirm: F,
    warning: W,
) -> Result<StoreResult, StoreError>
where
    F: FnMut(&HostkeyConfirm<'_>) -> bool,
    W: FnMut(&HostkeyWarning<'a>),
{
    let user_files = [path.to_path_buf()];
    check_and_store_hostkey_in_files(
        &user_files,
        &[],
        host,
        ip,
        key_type,
        key_data,
        hash_known_hosts,
        confirm,
        warning,
    )
}

/// Check a presented host key against OpenSSH's *file lists* and, where
/// OpenSSH would prompt, ask `confirm`; on acceptance append the entry
/// to `user_files[0]`.
///
/// The lists mirror `UserKnownHostsFile` / `GlobalKnownHostsFile`
/// (`user_hostfiles` / `system_hostfiles`, up to `SSH_MAX_HOSTS_FILES`
/// = 32 entries each, readconf.h:21):
/// - Reads merge **all user files first, then all system files** into
///   one list scanned in order (sshconnect.c:1007-1010), exactly
///   OpenSSH's single `check_key_in_hostkeys()` pass — earlier entries
///   win ties (user files beat system files), and `@revoked` in *any*
///   file wins over everything.
/// - Writes (on acceptance) go **only to `user_files[0]`**
///   (sshconnect.c:1117, 1223-1234); system files and later user
///   files are never modified.
/// - Missing files count as empty; other read errors are
///   [`StoreError::Io`].
/// - If `user_files` is empty and the status would require a prompt
///   (`HOST_NEW` / `MismatchHost`), the call fails with
///   [`StoreError::ReadOnly`] **without invoking `confirm`** — OpenSSH
///   sets `readonly` when `num_user_hostfiles == 0` and fails new hosts
///   before prompting (sshconnect.c:1051-1053, 1153). Read-only
///   statuses (`HOST_OK`, `HOST_CHANGED`, `HOST_REVOKED`) still
///   classify normally.
///
/// Prompting, decline, and IP-mismatch semantics are identical to
/// [`check_and_store_hostkey`] (which see for the step-by-step flow
/// and callback-policy notes). The `warning` callback behaves the same
/// as there — see [`HostkeyWarning`].
///
/// ```no_run
/// use std::path::PathBuf;
/// use known_hosts_parser::{check_and_store_hostkey_in_files, StoreError};
///
/// # fn demo(key_blob: &[u8]) -> Result<(), StoreError> {
/// let user = [PathBuf::from("/home/me/.ssh/known_hosts")];
/// let system = [PathBuf::from("/etc/ssh/ssh_known_hosts")];
/// let result = check_and_store_hostkey_in_files(
///     &user,
///     &system,
///     "example.com",
///     None,
///     "ssh-ed25519",
///     key_blob,
///     true, // HashKnownHosts
///     |_| true, // accept-new style
///     |_| {},   // warning callback — display-only
/// )?;
/// println!("status={:?}", result.status);
/// # Ok(())
/// # }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn check_and_store_hostkey_in_files<'a, F, W>(
    user_files: &[PathBuf],
    system_files: &[PathBuf],
    host: &'a str,
    ip: Option<&'a str>,
    key_type: &'a str,
    key_data: &[u8],
    hash_known_hosts: bool,
    mut confirm: F,
    mut warning: W,
) -> Result<StoreResult, StoreError>
where
    F: FnMut(&HostkeyConfirm<'_>) -> bool,
    W: FnMut(&HostkeyWarning<'a>),
{
    if host.is_empty() || host.chars().any(|c| INVALID_HOST_CHARS.contains(&c)) {
        return Err(StoreError::InvalidHost);
    }
    if key_type.ends_with(CERT_SUFFIX) {
        return Err(StoreError::Certificate);
    }

    let mut entries = Vec::new();
    let mut write_content = String::new();
    for (i, path) in user_files.iter().enumerate() {
        let content = read_known_hosts(path)?;
        let (mut parsed, _errors) = crate::parse(&content);
        entries.append(&mut parsed);
        if i == 0 {
            write_content = content;
        }
    }
    for path in system_files {
        let content = read_known_hosts(path)?;
        let (mut parsed, _errors) = crate::parse(&content);
        entries.append(&mut parsed);
    }

    let kt = KeyType::from_name(key_type);
    let status = check_hostkey(&entries, Some(host), ip, &kt, key_data);

    // Non-interactive banners at the points OpenSSH logs them
    // (sshconnect.c:1246-1253 revoked, :1301/:1670-1690 changed).
    match status {
        HostStatus::HOST_REVOKED => warning(&HostkeyWarning::HostKeyRevoked {
            host,
            key_type,
        }),
        HostStatus::HOST_CHANGED | HostStatus::HOST_CHANGED_IP_DIFFERS => {
            warning(&HostkeyWarning::HostKeyChanged {
                host,
                key_type,
                fingerprint: fingerprint(key_data),
                ip,
                dns_spoofing: status == HostStatus::HOST_CHANGED_IP_DIFFERS,
            });
        }
        _ => {}
    }

    let mut stored = false;
    let mut accepted = true;

    if matches!(status, HostStatus::HOST_NEW | HostStatus::MismatchHost) {
        // No user file to write to: fail before prompting, like
        // OpenSSH's readonly check (sshconnect.c:1153).
        let Some(write_path) = user_files.first() else {
            return Err(StoreError::ReadOnly);
        };

        let known_elsewhere: Vec<String> = entries
            .iter()
            .filter(|e| {
                key_loadable(e)
                    && e.marker == Marker::None
                    && key_eq(e, &kt, key_data)
                    && !entry_matches(e, host)
                    && !ip.is_some_and(|addr| entry_matches(e, addr))
            })
            .map(|e| e.hosts_string())
            .take(MAX_OTHER_NAMES)
            .collect();
        let other_type_known = entries.iter().any(|e| {
            entry_matches(e, host) && key_loadable(e) && e.key_type != kt
        });

        let prompt = NewHostPrompt {
            host,
            ip,
            key_type: &kt,
            key_data,
            fingerprint: fingerprint(key_data),
            known_elsewhere,
            other_type_known,
        };
        if !confirm(&HostkeyConfirm::NewHost(prompt)) {
            return Ok(StoreResult {
                status,
                stored: false,
                accepted: false,
            });
        }

        let entry = HostEntry {
            line_num: 0,
            marker: Marker::None,
            hosts: vec![HostPattern::Plain(host.to_string())],
            key_type: kt.clone(),
            key_data: key_data.to_vec(),
            comment: None,
            raw_line: String::new(),
        };
        let style = if hash_known_hosts {
            WriteStyle::Hashed
        } else {
            WriteStyle::Plaintext
        };
        let lines = entry.to_lines(style).map_err(StoreError::Write)?;
        append_lines(write_path, &write_content, &lines)?;
        stored = true;
        warning(&HostkeyWarning::PermanentlyAdded {
            host,
            key_type,
            fingerprint: fingerprint(key_data),
            hashed: hash_known_hosts,
        });
    }

    // sshconnect.c:1388-1406 — host key trusted but the IP-scope key
    // differs (only ever asked when the host status is not CHANGED;
    // HOST_REVOKED also skips — OpenSSH has already hard-failed there).
    let skip_ip = matches!(
        status,
        HostStatus::HOST_CHANGED
            | HostStatus::HOST_CHANGED_IP_DIFFERS
            | HostStatus::HOST_REVOKED
    );
    if !skip_ip {
        if let Some(addr) = ip {
            let ip_status = check_hostkey(&entries, None, Some(addr), &kt, key_data);
            if ip_status == HostStatus::HOST_CHANGED {
                let prompt = IpChangedPrompt {
                    host,
                    ip: addr,
                    key_type: &kt,
                    key_data,
                    fingerprint: fingerprint(key_data),
                };
                if !confirm(&HostkeyConfirm::IpChanged(prompt)) {
                    accepted = false;
                }
            } else if status == HostStatus::HOST_OK
                && matches!(
                    ip_status,
                    HostStatus::HOST_NEW | HostStatus::MismatchHost
                )
            {
                // sshconnect.c:1112-1116 — IP key not in list (this
                // crate never writes IP lines, so it persists).
                // MismatchHost here also means the IP scope is empty
                // (the key merely exists under the host name), which
                // OpenSSH's IP-only check reports as HOST_NEW.
                warning(&HostkeyWarning::IpKeyUnknown {
                    host,
                    ip: addr,
                    key_type,
                });
            }
        }
    }

    Ok(StoreResult {
        status,
        stored,
        accepted,
    })
}

/// Read a known_hosts file; a missing file counts as empty.
fn read_known_hosts(path: &Path) -> Result<String, StoreError> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(StoreError::Io(e)),
    }
}

/// Append `lines` to `path` with `add_host_to_hostfile()` semantics:
/// create the file if missing, ensure a terminating newline first, then
/// write each line followed by `\n`. `content` is the bytes last read
/// from the file, used to decide the newline top-up.
fn append_lines(path: &Path, content: &str, lines: &[String]) -> Result<(), StoreError> {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(StoreError::Io)?;
    if !content.is_empty() && !content.ends_with('\n') {
        file.write_all(b"\n").map_err(StoreError::Io)?;
    }
    for line in lines {
        file.write_all(line.as_bytes()).map_err(StoreError::Io)?;
        file.write_all(b"\n").map_err(StoreError::Io)?;
    }
    Ok(())
}
