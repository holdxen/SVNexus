//! # ssh_identity
//!
//! Discover the SSH identity files OpenSSH would try for a given target —
//! the Rust counterpart of what `openssh-portable` does across
//! `readconf.c fill_default_options()` (line 2848), `readconf.c
//! process_config_line_depth()` (line 1134) and `ssh.c
//! load_public_identity_files()` (line 2347).
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
//! * Files: `~/.ssh/config` then `<SSHDIR>/ssh_config`, in that order, both
//!   feeding one option set — `ssh.c:572-591 process_config_files()`. A
//!   `config` given through [`DiscoverOptions::config`] is the `-F` case and
//!   suppresses the system file entirely.
//! * Defaults: `~/.ssh/id_rsa`, `id_ecdsa`, `id_ecdsa_sk`, `id_ed25519`,
//!   `id_ed25519_sk` (`readconf.c:2918-2928` + `pathnames.h:88-92`) — added
//!   only when no config named an `IdentityFile` at all, and dropped again
//!   for `IdentityFile none` (`ssh.c:2397`).
//! * `Host` patterns use `match_pattern()` semantics (`match.c:82`): `*` and
//!   `?` only, case sensitive, no `[...]`, `!` negation vetoes the line.
//! * Only `Match all` / `Match !all` are understood; every other `Match`
//!   condition is reported as a warning and its block ignored. That can *add*
//!   candidates rather than remove them: a block holding the only
//!   `IdentityFile` lines leaves the list empty, and the built-in defaults
//!   then get appended. See `config::parse_match`.
//! * `Include` resolves relative paths against `~/.ssh` (or `<SSHDIR>` in the
//!   system file), globs them, is depth-limited to `READCONF_MAX_DEPTH` (16),
//!   is parsed as "never match" when it sits inside an inactive block, and
//!   never clobbers the block state of the file that included it
//!   (`readconf.c:2117-2140`).
//! * `SSHCONF_CHECKPERM` files (`readconf.c:2636-2644`) that are group or
//!   world writable are skipped with a warning instead of the `fatal()`
//!   OpenSSH raises.
//! * Path expansion supports `~`, `~/.`, `%d %u %h %n %p %r %l %L %i %k %j
//!   %C %%` and `${ENV}` — the full `DEFAULT_CLIENT_PERCENT_EXPAND_ARGS`
//!   table of `sshconnect.h:55-71`. `%l %L %i %C` describe the local machine
//!   and this crate is dependency-free, so they need
//!   [`DiscoverOptions::local_host`] / [`DiscoverOptions::uid`] to be
//!   injected; without them the identity is dropped with a warning rather
//!   than expanded to a wrong path. `%h` takes its value from the config's
//!   own `Hostname`, `%r` from `User`, `%k` from `HostKeyAlias`, `%j` from
//!   `ProxyJump`.
//! * Where OpenSSH `fatal()`s — a bad token, a missing `${ENV}`, a `Match`
//!   it cannot parse — this crate warns and keeps going, because its caller
//!   uses the result to *log*, not to connect.

mod config;
mod expand;
mod glob;
mod hash;

pub use expand::{percent_expand, tilde_expand, ExpandContext, ExpandError};
pub use glob::{glob, match_pattern};

use std::path::{Path, PathBuf};

use config::{ConfigFile, RawConfig, RawPath};

/// `readconf.c:2918-2928` — the built-in identity list, in OpenSSH's order.
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
    /// `identity_file_userprovided[]`: `true` for a path contributed by a
    /// *user* configuration file (`SSHCONF_USERCONF`, `readconf.c:1410`),
    /// `false` for the system file and for the built-in defaults.
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
    /// The host as given by the user; `%n`, `Host` patterns and the default
    /// for `%h`.
    pub host: &'a str,
    pub port: u16,
    /// `%r` — the remote login user. `None` falls back to `local_user`,
    /// which is what OpenSSH does too (`readconf.c:654`).
    pub remote_user: Option<&'a str>,
    /// `%u` — OpenSSH's `pw->pw_name`. Defaults to `$USER` then `$LOGNAME`
    /// (both written by the login shell from that same passwd entry) or
    /// `$USERNAME` on Windows.
    pub local_user: Option<&'a str>,
    /// `%l` — OpenSSH's `gethostname()`. A dependency-free crate cannot ask
    /// the kernel, so nothing is guessed: without this, `%l`, `%L` and `%C`
    /// drop the identities that use them.
    pub local_host: Option<&'a str>,
    /// `%i` — OpenSSH's `pw->pw_uid`. No default either, same reason.
    pub uid: Option<u32>,
    /// Defaults to `<home>/.ssh/config`. Setting it explicitly is the `-F`
    /// case, which also suppresses the system configuration file.
    pub config: Option<&'a Path>,
    /// `None` reads `<SSHDIR>/ssh_config` (`/etc/ssh/ssh_config`) after the
    /// user file, unless `config` is set. `Some(path)` reads `path` instead,
    /// and `Some(None)` reads no system file at all.
    pub system_config: Option<Option<&'a Path>>,
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
            uid: None,
            config: None,
            system_config: None,
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

    pub fn uid(mut self, uid: Option<u32>) -> Self {
        self.uid = uid;
        self
    }

    pub fn config(mut self, path: Option<&'a Path>) -> Self {
        self.config = path;
        self
    }

    pub fn system_config(mut self, path: Option<Option<&'a Path>>) -> Self {
        self.system_config = path;
        self
    }
}

/// Find the identity files OpenSSH would try for `host:port`.
///
/// Missing files are **not** filtered out: OpenSSH only checks for them at
/// authentication time (`sshconnect2.c:1537`, `no such identity`), and a
/// caller that wants to log the same way needs the list as-is.
pub fn discover(options: &DiscoverOptions<'_>) -> Discovery {
    let local_user = options
        .local_user
        .map(str::to_owned)
        .or_else(|| {
            std::env::var("USER")
                .ok()
                .or_else(|| std::env::var("LOGNAME").ok())
                .or_else(|| std::env::var("USERNAME").ok())
        })
        .unwrap_or_default();
    // `%l` is gethostname() in OpenSSH (ssh.c:1422). There is no
    // dependency-free way to ask, and `$HOSTNAME` is a bash variable that is
    // not exported, so it stays whatever the caller injected.
    let local_host = options.local_host.unwrap_or("");
    // readconf.c:654 `ruser = options->user == NULL ? pw->pw_name : options->user`.
    let remote_user = options.remote_user.unwrap_or(&local_user).to_string();

    let base_ctx = ExpandContext::new(options.home, options.host, options.port)
        .remote_user(&remote_user)
        .local_user(&local_user)
        .local_host(if local_host.is_empty() {
            None
        } else {
            Some(local_host)
        })
        .uid(options.uid);

    let specs = config_files(options);
    let files: Vec<ConfigFile<'_>> = specs
        .iter()
        .map(|spec| ConfigFile {
            path: spec.path.as_path(),
            userconf: spec.userconf,
            check_perm: spec.check_perm,
        })
        .collect();
    let mut raw = config::parse(&files, &base_ctx);
    let mut warnings = std::mem::take(&mut raw.warnings);

    // ssh.c:1422-1465 is where OpenSSH turns the parsed options into the
    // strings percent expansion reads. Same here, after every file is read.
    let (hostname, host_key_alias, jump_host, user) =
        resolve_context_values(&raw, &base_ctx, &mut warnings);
    let ctx = base_ctx
        .hostname(&hostname)
        .host_key_alias(host_key_alias.as_deref())
        .jump_host(&jump_host)
        .remote_user(user.as_deref().unwrap_or(&remote_user));

    // fill_default_options() (readconf.c:2918): defaults only when no config
    // file named an IdentityFile. Note that an `IdentityFile none` line
    // counts here — it blocks the defaults and only gets dropped further
    // down (ssh.c:2397).
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
        if let Some(identity) = expand_path(entry, options.home, &ctx, &mut warnings) {
            identities.push(identity);
        }
    }

    let mut certificates = Vec::with_capacity(raw.certificate_files.len());
    for entry in raw.certificate_files {
        if entry.raw.eq_ignore_ascii_case("none") {
            continue;
        }
        if let Some(identity) = expand_path(entry, options.home, &ctx, &mut warnings) {
            certificates.push(identity.path);
        }
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

/// One configuration file to read, with the path owned so it can outlive the
/// decision logic in [`config_files`].
struct FileSpec {
    path: PathBuf,
    userconf: bool,
    check_perm: bool,
}

/// The `-F` rule of `ssh.c:572-591`: an explicit config means that file and
/// nothing else, otherwise the user file is read first and the system file
/// after it.
fn config_files(options: &DiscoverOptions<'_>) -> Vec<FileSpec> {
    let user = match options.config {
        Some(path) => FileSpec {
            path: path.to_path_buf(),
            userconf: true,
            // read_config_file() with a -F path does not set CHECKPERM
            // (ssh.c:576); only the default location does (ssh.c:585).
            check_perm: false,
        },
        None => FileSpec {
            path: options.home.join(".ssh").join("config"),
            userconf: true,
            check_perm: true,
        },
    };

    let mut files = vec![user];
    if options.config.is_some() {
        return files;
    }
    match options.system_config {
        Some(None) => {}
        Some(Some(path)) => files.push(FileSpec {
            path: path.to_path_buf(),
            userconf: false,
            check_perm: false,
        }),
        None => files.push(FileSpec {
            path: PathBuf::from(config::SSH_SYSTEM_CONFIG),
            userconf: false,
            check_perm: false,
        }),
    }
    files
}

/// Resolve the four option-derived expansion values, OpenSSH-style:
/// `%h` from `Hostname`, `%u`/`%r` from `User`, `%k` from `HostKeyAlias`,
/// `%j` from `ProxyJump`.
///
/// Deviation: OpenSSH refuses to expand `User` with `%r` or `%C` (they are
/// not in `DEFAULT_CLIENT_PERCENT_EXPAND_ARGS_NOUSER`, and `%C` is built
/// *from* the user), and `fatal()`s when it tries; here those two simply see
/// the pre-expansion values.
fn resolve_context_values(
    raw: &RawConfig,
    base: &ExpandContext<'_>,
    warnings: &mut Vec<String>,
) -> (String, Option<String>, String, Option<String>) {
    // readconf.c:658 `percent_expand(options->hostname, "h", host_arg, NULL)`.
    let hostname = match raw.hostname.as_deref() {
        Some(value) => match percent_expand(value, base) {
            Ok(expanded) => expanded,
            // OpenSSH would have failed the connection; keep the literal and
            // say so, because every identity that uses %h inherits this.
            Err(e) => {
                warnings.push(format!("hostname {value}: {e}"));
                value.to_string()
            }
        },
        None => base.host.to_string(),
    };
    // ssh.c:1429-1430: `%k` is HostKeyAlias when set, otherwise the host as
    // given — NOT the Hostname value, which is what %h gets.
    let host_key_alias = raw.host_key_alias.clone();

    // ssh.c:1444 expands User before it can be used as %r.
    let user = raw.user.as_ref().map(|value| {
        let with_hostname = ExpandContext {
            hostname: &hostname,
            ..*base
        };
        match percent_expand(value, &with_hostname) {
            Ok(expanded) => expanded,
            Err(e) => {
                warnings.push(format!("user {value}: {e}"));
                value.clone()
            }
        }
    });

    (hostname, host_key_alias, jump_host(raw), user)
}

/// `readconf.c:666` `option_clear_or_none(options->jump_host) ? "" :
/// options->jump_host`, where `parse_jump()` (readconf.c:3490-3510) stored
/// the *last* comma separated entry's bare host.
fn jump_host(raw: &RawConfig) -> String {
    let Some(spec) = raw.jump.as_deref() else {
        return String::new();
    };
    if spec.eq_ignore_ascii_case("none") {
        return String::new();
    }
    // parse_jump() truncates at '#' and right-trims before splitting.
    let spec = spec.split('#').next().unwrap_or("").trim_end();
    let last = spec.rsplit(',').next().unwrap_or("");
    let without_user = match last.rfind('@') {
        // parse_user_host_port() cuts at the *last* '@'.
        Some(at) => &last[at + 1..],
        None => last,
    };
    let host = if let Some(rest) = without_user.strip_prefix('[') {
        // hpdelim(): a bracketed literal keeps its content, drops the ']'
        // and any :port that follows.
        rest.split(']').next().unwrap_or("")
    } else {
        without_user.split(':').next().unwrap_or("")
    };
    // cleanhostname(): a single trailing dot is not part of the name.
    host.strip_suffix('.').unwrap_or(host).to_string()
}

/// `ssh.c:2402-2403`: tilde first (`tilde_expand_filename`), then the `%`
/// and `${}` tokens (`default_client_percent_dollar_expand`).
///
/// `ssh.c:221-232` turns an expansion failure into `fatal()` and ssh exits;
/// this crate drops the single identity and warns about it, which is also
/// what the module documentation promises.
fn expand_path(
    entry: RawPath,
    home: &Path,
    ctx: &ExpandContext<'_>,
    warnings: &mut Vec<String>,
) -> Option<IdentityFile> {
    // ssh.c:2397 drops `none` before any of this, so it never reaches here.
    let expanded = tilde_expand(&entry.raw, home);
    match percent_expand(&expanded, ctx) {
        Ok(path) => Some(IdentityFile {
            path: PathBuf::from(path),
            user_provided: entry.user_provided,
        }),
        Err(e) => {
            warnings.push(format!("identity file {}: {e}", entry.raw));
            None
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

    fn paths(found: &Discovery) -> Vec<&Path> {
        found.identities.iter().map(|i| i.path.as_path()).collect()
    }

    #[test]
    fn defaults_when_no_config() {
        let dir = TempDir::new("defaults");
        let found = discover_in(&dir, "example.com");
        assert_eq!(
            paths(&found),
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
        // the paths are loaded (ssh.c:2397).
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
        assert_eq!(
            paths(&found),
            [
                dir.path.join(".ssh/example.com_22_key"),
                PathBuf::from("me_key"),
            ]
        );
    }

    #[test]
    fn hostname_option_feeds_h_but_not_n() {
        // ssh.c:1431-1432: remhost is the resolved host, host_arg stays what
        // the user typed.
        let dir = TempDir::new("hostname");
        dir.file(
            ".ssh/config",
            "Host alias\nHostname real.test\nIdentityFile ~/.ssh/h_%h\nIdentityFile ~/.ssh/n_%n\n",
        );
        let found = discover_in(&dir, "alias");
        assert_eq!(
            paths(&found),
            [
                dir.path.join(".ssh/h_real.test"),
                dir.path.join(".ssh/n_alias"),
            ],
            "warnings: {:?}",
            found.warnings
        );
    }

    #[test]
    fn remote_user_falls_back_to_the_local_user() {
        // readconf.c:654.
        let dir = TempDir::new("r_fallback");
        dir.file(".ssh/config", "IdentityFile ~/.ssh/r_%r\n");
        let config = dir.path(".ssh/config");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .local_user(Some("me"))
                .config(Some(&config)),
        );
        assert_eq!(paths(&found), [dir.path.join(".ssh/r_me")]);

        // A `User` directive is the other half of readconf.c:654.
        dir.file(".ssh/config", "User alice\nIdentityFile ~/.ssh/r_%r\n");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .local_user(Some("me"))
                .config(Some(&config)),
        );
        assert_eq!(paths(&found), [dir.path.join(".ssh/r_alice")]);
    }

    #[test]
    fn alias_and_jump_tokens() {
        let dir = TempDir::new("alias_jump");
        dir.file(
            ".ssh/config",
            "HostKeyAlias tagged\nProxyJump bob@jump1,jump.example:2222\nIdentityFile ~/.ssh/k_%k_j_%j\n",
        );
        let found = discover_in(&dir, "example.com");
        assert_eq!(
            paths(&found),
            [dir.path.join(".ssh/k_tagged_j_jump.example")],
            "warnings: {:?}",
            found.warnings
        );
    }

    #[test]
    fn key_alias_falls_back_to_the_given_host_not_the_hostname() {
        let dir = TempDir::new("k_no_alias");
        dir.file(
            ".ssh/config",
            "Host gh\nHostname github.com\nIdentityFile ~/.ssh/k_%k_n_%n\n",
        );
        let found = discover_in(&dir, "gh");
        assert_eq!(paths(&found), [dir.path.join(".ssh/k_gh_n_gh")]);
    }

    #[test]
    fn proxy_jump_none_clears_j() {
        // parse_jump() stores the literal "none"; readconf.c:666
        // option_clear_or_none() then expands %j to nothing.
        let dir = TempDir::new("jump_none");
        dir.file(
            ".ssh/config",
            "ProxyJump none\nIdentityFile ~/.ssh/j_x%jx\n",
        );
        let found = discover_in(&dir, "example.com");
        assert_eq!(paths(&found), [dir.path.join(".ssh/j_xx")]);
    }

    #[test]
    fn local_machine_tokens_need_injection() {
        let dir = TempDir::new("local_tokens");
        dir.file(".ssh/config", "IdentityFile ~/.ssh/l_%l_L_%L_i_%i\n");
        let config = dir.path(".ssh/config");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .local_host(Some("laptop.example.org"))
                .uid(Some(501))
                .config(Some(&config)),
        );
        assert_eq!(
            paths(&found),
            [dir.path.join(".ssh/l_laptop.example.org_L_laptop_i_501")]
        );

        // Nothing injected: the identity is dropped rather than becoming a
        // path with holes in it.
        let found =
            discover(&DiscoverOptions::new(&dir.path, "example.com", 22).config(Some(&config)));
        assert!(found.identities.is_empty(), "{:?}", found.identities);
        assert!(
            found
                .warnings
                .iter()
                .any(|w| w.contains("no value available for %l")),
            "{:?}",
            found.warnings
        );
    }

    #[test]
    fn connection_hash_matches_openssh() {
        let dir = TempDir::new("percent_c");
        dir.file(".ssh/config", "IdentityFile ~/.ssh/%C\n");
        let config = dir.path(".ssh/config");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "nope.invalid", 22)
                .remote_user(Some("zhouguiqing"))
                .local_host(Some("zhouguiqingdeMac-mini.local"))
                .config(Some(&config)),
        );
        assert_eq!(
            paths(&found),
            [dir.path
                .join(".ssh/5cb7925d5491da22e829af92f450cb232d9db7f5")]
        );
    }

    #[test]
    fn unexpandable_path_is_dropped_and_warned_about() {
        let dir = TempDir::new("bad_expand");
        dir.file(
            ".ssh/config",
            "IdentityFile ~/.ssh/%x_key\nIdentityFile ~/.ssh/ok\n",
        );
        let found = discover_in(&dir, "example.com");
        // ssh.c:221-232 would fatal(); we lose just this one candidate.
        assert_eq!(paths(&found), [dir.path.join(".ssh/ok")]);
        assert!(
            found.warnings.iter().any(|w| w.contains("unknown key %x")),
            "{:?}",
            found.warnings
        );
    }

    #[test]
    fn missing_env_var_drops_the_identity() {
        let dir = TempDir::new("missing_env");
        dir.file(
            ".ssh/config",
            "IdentityFile ~/.ssh/${NO_SUCH_VAR_XYZ}\nIdentityFile ~/.ssh/ok\n",
        );
        let found = discover_in(&dir, "example.com");
        assert_eq!(paths(&found), [dir.path.join(".ssh/ok")]);
        assert!(
            found.warnings.iter().any(|w| w.contains("has no value")),
            "{:?}",
            found.warnings
        );
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

    #[test]
    fn system_config_is_read_after_the_user_file_and_is_not_user_provided() {
        // ssh.c:587-591: the user file first, then /etc/ssh/ssh_config.
        let dir = TempDir::new("system");
        dir.file(
            ".ssh/config",
            "IdentityFile ~/.ssh/user_key\nHostname user.test\n",
        );
        dir.file(
            "ssh_config",
            "IdentityFile /etc/ssh/host_key\nHostname system.test\n",
        );
        let config = dir.path(".ssh/config");
        let system = dir.path("ssh_config");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .config(None)
                .system_config(Some(Some(&system)))
                .local_user(Some("me")),
        );
        assert_eq!(
            paths(&found),
            [
                dir.path.join(".ssh/user_key"),
                PathBuf::from("/etc/ssh/host_key"),
            ],
            "warnings: {:?}",
            found.warnings
        );
        assert!(found.identities[0].user_provided);
        assert!(!found.identities[1].user_provided);
        // First active `Hostname` wins across both files (one Options struct).
        dir.file(".ssh/config", "IdentityFile ~/.ssh/k_%h\n");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .system_config(Some(Some(&system)))
                .local_user(Some("me"))
                .config(Some(&config)),
        );
        // -F suppresses the system file, so `%h` stays the given host.
        assert_eq!(paths(&found), [dir.path.join(".ssh/k_example.com")]);
    }

    #[test]
    fn defaults_are_suppressed_by_a_system_config_identity() {
        let dir = TempDir::new("system_defaults");
        dir.file("ssh_config", "IdentityFile /etc/ssh/host_key\n");
        let system = dir.path("ssh_config");
        let found = discover(
            &DiscoverOptions::new(&dir.path, "example.com", 22)
                .system_config(Some(Some(&system)))
                .local_user(Some("me")),
        );
        assert_eq!(paths(&found), [PathBuf::from("/etc/ssh/host_key")]);
    }

    #[test]
    fn repeated_slashes_after_tilde_are_collapsed() {
        let dir = TempDir::new("tilde_slash");
        dir.file(".ssh/config", "IdentityFile ~//%h_key\n");
        let found = discover_in(&dir, "example.com");
        // misc.c `copy += strspn(copy, "/")` — the second slash is a separator,
        // not an absolute path. `Path::join` would have replaced the home.
        assert_eq!(paths(&found), [dir.path.join("example.com_key")]);
        assert_ne!(paths(&found), [Path::new("/example.com_key")]);
    }
}
