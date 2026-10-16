//! Path expansion: `~` and OpenSSH's `%` / `${ENV}` tokens.
//!
//! Follows `ssh.c:221 default_client_percent_dollar_expand()` →
//! `misc.c vdollar_percent_expand()`, plus `tilde_expand_filename()`:
//!
//! * `~` and `~/...` expand to the user's home directory. `~user` is left
//!   alone (we have no passwd database here).
//! * The token set is exactly `sshconnect.h:55-71
//!   DEFAULT_CLIENT_PERCENT_EXPAND_ARGS`: `%% %C %d %h %i %j %k %L %l %n %p
//!   %r %u`.
//! * `%l %L %i %C` describe the *local* machine and this crate is
//!   dependency-free, so it cannot call `gethostname()`/`getpwuid()`: the
//!   caller injects them through [`ExpandContext`]. A token whose value was
//!   never injected is an error, which drops that one identity — expanding
//!   it to an empty string would produce a wrong path instead.
//! * Anything else is an OpenSSH `fatal()`; here it drops that one identity
//!   with a warning.
//! * `${VAR}` expands an environment variable. Bare `$VAR` is **not**
//!   expanded — `vdollar_percent_expand()` only reacts to `${`.

use std::path::Path;

use crate::hash::{sha1, to_hex};

/// Values the token expansion can draw from.
///
/// Field sources in OpenSSH, all of them `ssh.c:1422-1436` /
/// `readconf.c:654-673`:
#[derive(Clone, Copy, Debug)]
pub struct ExpandContext<'a> {
    /// `%d` — `pw->pw_dir`.
    pub home: &'a Path,
    /// `%n` — the host as it was given, `options.host_arg`.
    pub host: &'a str,
    /// `%h` — `options.hostname` when the config sets it, else [`Self::host`].
    pub hostname: &'a str,
    /// `%p` — `options.port`.
    pub port: u16,
    /// `%r` — `options.user`, falling back to the local user.
    pub remote_user: &'a str,
    /// `%u` — `pw->pw_name`.
    pub local_user: &'a str,
    /// `%l` — `gethostname()`; `None` when nobody injected it.
    pub local_host: Option<&'a str>,
    /// `%i` — `pw->pw_uid`.
    pub uid: Option<u32>,
    /// `%k` — `options.host_key_alias`, else [`Self::host`].
    pub host_key_alias: Option<&'a str>,
    /// `%j` — the last `ProxyJump` entry's bare host, `""` when unset.
    pub jump_host: &'a str,
}

impl<'a> ExpandContext<'a> {
    pub fn new(home: &'a Path, host: &'a str, port: u16) -> Self {
        Self {
            home,
            host,
            hostname: host,
            port,
            remote_user: "",
            local_user: "",
            local_host: None,
            uid: None,
            host_key_alias: None,
            jump_host: "",
        }
    }

    pub fn hostname(mut self, hostname: &'a str) -> Self {
        self.hostname = hostname;
        self
    }

    pub fn remote_user(mut self, user: &'a str) -> Self {
        self.remote_user = user;
        self
    }

    pub fn local_user(mut self, user: &'a str) -> Self {
        self.local_user = user;
        self
    }

    /// `%l`; `%L` and `%C` are derived from it, so it gates all three.
    pub fn local_host(mut self, host: Option<&'a str>) -> Self {
        self.local_host = host;
        self
    }

    pub fn uid(mut self, uid: Option<u32>) -> Self {
        self.uid = uid;
        self
    }

    pub fn host_key_alias(mut self, alias: Option<&'a str>) -> Self {
        self.host_key_alias = alias;
        self
    }

    pub fn jump_host(mut self, jump: &'a str) -> Self {
        self.jump_host = jump;
        self
    }

    /// `ssh.c:1424-1425` — `thishost[strcspn(thishost, ".")] = '\0'`.
    fn short_host(&self) -> Option<&'a str> {
        let name = self.local_host?;
        Some(match name.find('.') {
            Some(dot) => &name[..dot],
            None => name,
        })
    }

    /// `readconf.c:345 ssh_connection_hash()`: SHA-1 over the five strings
    /// with no separators, hex encoded. `%l` is the first component, so an
    /// uninjected `%l` means no `%C` either.
    fn connection_hash(&self) -> Option<String> {
        let thishost = self.local_host?;
        let mut input = String::with_capacity(64);
        input.push_str(thishost);
        input.push_str(self.hostname);
        input.push_str(&self.port.to_string());
        input.push_str(self.remote_user);
        input.push_str(self.jump_host);
        Some(to_hex(&sha1(input.as_bytes())))
    }
}

/// Why an `IdentityFile` / `Include` path could not be expanded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpandError {
    /// `%x` for a token OpenSSH does not accept here (`unknown key %x`).
    UnknownEscape(char),
    /// A token OpenSSH accepts but whose value nobody supplied: `%l`/`%L`/
    /// `%i`/`%C` need `gethostname()`/`getpwuid()` we deliberately do not
    /// call.
    Unavailable(char),
    /// Trailing `%` with nothing after it (`invalid format`).
    InvalidFormat,
    /// `${VAR}` with no value in the environment.
    MissingVariable(String),
    /// `${` without a closing `}`, or `${}`.
    MalformedVariable(String),
}

impl std::fmt::Display for ExpandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpandError::UnknownEscape(c) => write!(f, "unknown key %{c}"),
            ExpandError::Unavailable(c) => {
                write!(f, "no value available for %{c} (not injected)")
            }
            ExpandError::InvalidFormat => write!(f, "invalid format"),
            ExpandError::MissingVariable(v) => write!(f, "env var ${{{v}}} has no value"),
            ExpandError::MalformedVariable(v) => {
                write!(f, "malformed environment variable ${{{v}}}")
            }
        }
    }
}

impl std::error::Error for ExpandError {}

/// `tilde_expand_filename()` → `misc.c:tilde_expand()`.
pub fn tilde_expand(path: &str, home: &Path) -> String {
    let Some(rest) = path.strip_prefix('~') else {
        return path.to_string();
    };
    // `~user/...` needs getpwnam(); leave it exactly as written.
    if !rest.is_empty() && !rest.starts_with('/') {
        return path.to_string();
    }
    // `copy += strspn(copy, "/")` — collapse every slash, not just one.
    let sub = rest.trim_start_matches('/');
    join_home(home, if sub.is_empty() { None } else { Some(sub) })
}

/// `misc.c:tilde_expand()`'s `"%s%s%s"` with the trailing-slash rule: a
/// bare `~` or `~/` yields the home directory *plus* the separator OpenSSH
/// inserts, i.e. `/home/me/`.
fn join_home(home: &Path, path: Option<&str>) -> String {
    let mut out = home.to_string_lossy().into_owned();
    if !out.ends_with('/') {
        out.push('/');
    }
    out.push_str(path.unwrap_or(""));
    out
}

/// `vdollar_percent_expand()` over the client's default token set.
pub fn percent_expand(input: &str, ctx: &ExpandContext<'_>) -> Result<String, ExpandError> {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let Some(next) = chars.next() else {
                    return Err(ExpandError::InvalidFormat);
                };
                match next {
                    '%' => out.push('%'),
                    'd' => out.push_str(&ctx.home.to_string_lossy()),
                    'u' => out.push_str(ctx.local_user),
                    'h' => out.push_str(ctx.hostname),
                    'n' => out.push_str(ctx.host),
                    'p' => out.push_str(&ctx.port.to_string()),
                    'r' => out.push_str(ctx.remote_user),
                    'j' => out.push_str(ctx.jump_host),
                    'k' => out.push_str(ctx.host_key_alias.unwrap_or(ctx.host)),
                    'l' => {
                        let Some(name) = ctx.local_host else {
                            return Err(ExpandError::Unavailable('l'));
                        };
                        out.push_str(name);
                    }
                    'L' => {
                        let Some(name) = ctx.short_host() else {
                            return Err(ExpandError::Unavailable('L'));
                        };
                        out.push_str(name);
                    }
                    'i' => {
                        let Some(uid) = ctx.uid else {
                            return Err(ExpandError::Unavailable('i'));
                        };
                        out.push_str(&uid.to_string());
                    }
                    'C' => {
                        let Some(hash) = ctx.connection_hash() else {
                            return Err(ExpandError::Unavailable('C'));
                        };
                        out.push_str(&hash);
                    }
                    other => return Err(ExpandError::UnknownEscape(other)),
                }
            }
            '$' if chars.peek() == Some(&'{') => {
                chars.next(); // consume '{'
                let mut name = String::new();
                let mut closed = false;
                for c in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    name.push(c);
                }
                if !closed || name.is_empty() {
                    return Err(ExpandError::MalformedVariable(name));
                }
                match std::env::var(&name) {
                    Ok(value) => out.push_str(&value),
                    Err(_) => return Err(ExpandError::MissingVariable(name)),
                }
            }
            other => out.push(other),
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn ctx() -> ExpandContext<'static> {
        ExpandContext::new(Path::new("/home/me"), "example.com", 2222)
            .remote_user("bob")
            .local_user("me")
            .local_host(Some("laptop.example.org"))
            .uid(Some(501))
    }

    #[test]
    fn tilde_only_and_slash() {
        let home = Path::new("/home/me");
        assert_eq!(tilde_expand("~", home), "/home/me/");
        assert_eq!(tilde_expand("~/", home), "/home/me/");
        assert_eq!(tilde_expand("~/.ssh/id_rsa", home), "/home/me/.ssh/id_rsa");
        // misc.c collapses repeated slashes (`strspn(copy, "/")`).
        assert_eq!(tilde_expand("~//.ssh/id_rsa", home), "/home/me/.ssh/id_rsa");
        assert_eq!(tilde_expand("~///x", home), "/home/me/x");
        // A home directory with a trailing slash must not gain a second one.
        assert_eq!(tilde_expand("~/", Path::new("/home/me/")), "/home/me/");
        // `~user` is not expanded (no passwd database here).
        assert_eq!(tilde_expand("~alice/key", home), "~alice/key");
        assert_eq!(tilde_expand("/abs/key", home), "/abs/key");
    }

    #[test]
    fn percent_tokens() {
        let c = ctx();
        assert_eq!(
            percent_expand("%d/.ssh/id", &c).unwrap(),
            "/home/me/.ssh/id"
        );
        assert_eq!(percent_expand("%h_key", &c).unwrap(), "example.com_key");
        assert_eq!(percent_expand("%n_key", &c).unwrap(), "example.com_key");
        assert_eq!(percent_expand("%p", &c).unwrap(), "2222");
        assert_eq!(percent_expand("%r", &c).unwrap(), "bob");
        assert_eq!(percent_expand("%u", &c).unwrap(), "me");
        assert_eq!(percent_expand("%l", &c).unwrap(), "laptop.example.org");
        assert_eq!(percent_expand("%L", &c).unwrap(), "laptop");
        assert_eq!(percent_expand("%i", &c).unwrap(), "501");
        assert_eq!(percent_expand("%k", &c).unwrap(), "example.com");
        assert_eq!(percent_expand("%j", &c).unwrap(), "");
        assert_eq!(percent_expand("100%%", &c).unwrap(), "100%");
    }

    #[test]
    fn hostname_and_host_arg_are_different_tokens() {
        // ssh.c:1432 cinfo->remhost = host (the resolved Hostname) while
        // ssh.c:1431 cinfo->host_arg stays what the user typed.
        let c = ExpandContext::new(Path::new("/home/me"), "gh", 22).hostname("github.com");
        assert_eq!(percent_expand("%h", &c).unwrap(), "github.com");
        assert_eq!(percent_expand("%n", &c).unwrap(), "gh");
    }

    #[test]
    fn key_alias_falls_back_to_the_host_arg() {
        // ssh.c:1429-1430 `options.host_key_alias ? : options.host_arg` —
        // the host as given, never the Hostname value.
        let c = ExpandContext::new(Path::new("/home/me"), "gh", 22).host_key_alias(Some("alias"));
        assert_eq!(percent_expand("%k", &c).unwrap(), "alias");

        let c = ExpandContext::new(Path::new("/home/me"), "gh", 22).hostname("github.com");
        assert_eq!(percent_expand("%k", &c).unwrap(), "gh");
    }

    #[test]
    fn connection_hash_matches_openssh() {
        // Same inputs as hash::tests::matches_openssh_connection_hash.
        let c = ExpandContext::new(Path::new("/home/me"), "nope.invalid", 22)
            .remote_user("zhouguiqing")
            .local_host(Some("zhouguiqingdeMac-mini.local"));
        assert_eq!(
            percent_expand("/k/%C", &c).unwrap(),
            "/k/5cb7925d5491da22e829af92f450cb232d9db7f5"
        );
    }

    #[test]
    fn uninjected_tokens_are_an_error_not_an_empty_string() {
        let c = ExpandContext::new(Path::new("/home/me"), "example.com", 22);
        assert_eq!(percent_expand("%l", &c), Err(ExpandError::Unavailable('l')));
        assert_eq!(percent_expand("%L", &c), Err(ExpandError::Unavailable('L')));
        assert_eq!(percent_expand("%i", &c), Err(ExpandError::Unavailable('i')));
        assert_eq!(percent_expand("%C", &c), Err(ExpandError::Unavailable('C')));
    }

    #[test]
    fn percent_unknown_is_an_error() {
        let c = ctx();
        assert_eq!(
            percent_expand("%x", &c),
            Err(ExpandError::UnknownEscape('x'))
        );
        assert_eq!(
            percent_expand("trailing%", &c),
            Err(ExpandError::InvalidFormat)
        );
    }

    #[test]
    fn dollar_braces_only() {
        let c = ctx();
        std::env::set_var("SSH_IDENTITY_TEST_DIR", "/srv/keys");
        assert_eq!(
            percent_expand("${SSH_IDENTITY_TEST_DIR}/id", &c).unwrap(),
            "/srv/keys/id"
        );
        // Bare `$VAR` is left alone, exactly like vdollar_percent_expand().
        assert_eq!(
            percent_expand("$SSH_IDENTITY_TEST_DIR", &c).unwrap(),
            "$SSH_IDENTITY_TEST_DIR"
        );
        assert_eq!(
            percent_expand("${SSH_IDENTITY_TEST_MISSING}", &c),
            Err(ExpandError::MissingVariable(
                "SSH_IDENTITY_TEST_MISSING".into()
            ))
        );
        assert_eq!(
            percent_expand("${oops", &c),
            Err(ExpandError::MalformedVariable("oops".into()))
        );
        assert_eq!(
            percent_expand("${}", &c),
            Err(ExpandError::MalformedVariable(String::new()))
        );
    }

    #[test]
    fn chained_tilde_then_percent() {
        let c = ctx();
        let raw = "~/.ssh/%h_id";
        let expanded = tilde_expand(raw, PathBuf::from("/home/me").as_path());
        assert_eq!(
            percent_expand(&expanded, &c).unwrap(),
            "/home/me/.ssh/example.com_id"
        );
    }
}
