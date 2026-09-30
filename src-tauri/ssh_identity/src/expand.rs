//! Path expansion: `~` and OpenSSH's `%` / `${ENV}` tokens.
//!
//! Follows `ssh.c:221 default_client_percent_dollar_expand()` →
//! `misc.c vdollar_percent_expand()`, plus `tilde_expand_filename()`:
//!
//! * `~` and `~/...` expand to the user's home directory. `~user` is left
//!   alone (we have no passwd database here).
//! * `%d` home, `%u` local user, `%h`/`%n` remote host, `%p` port,
//!   `%r` remote user, `%l` local hostname, `%%` a literal `%`.
//!   Any other escape (notably `%C`, `%k`, `%i`, `%j`, `%L`) is an error
//!   in OpenSSH (`fatal("failed")`); we surface it instead so the caller
//!   can drop that single identity and keep going.
//! * `${VAR}` expands an environment variable. Bare `$VAR` is **not**
//!   expanded — `vdollar_percent_expand()` only reacts to `${`.

use std::path::Path;

/// Values the token expansion can draw from.
#[derive(Clone, Copy, Debug)]
pub struct ExpandContext<'a> {
    pub home: &'a Path,
    /// `%h` and `%n` — the host as it was given to us.
    pub host: &'a str,
    /// `%p`
    pub port: u16,
    /// `%r` — remote (login) user, `""` when the URL carried none.
    pub remote_user: &'a str,
    /// `%u` — local user.
    pub local_user: &'a str,
    /// `%l` — local hostname.
    pub local_host: &'a str,
}

impl<'a> ExpandContext<'a> {
    pub fn new(home: &'a Path, host: &'a str, port: u16) -> Self {
        Self {
            home,
            host,
            port,
            remote_user: "",
            local_user: "",
            local_host: "",
        }
    }

    pub fn remote_user(mut self, user: &'a str) -> Self {
        self.remote_user = user;
        self
    }

    pub fn local_user(mut self, user: &'a str) -> Self {
        self.local_user = user;
        self
    }

    pub fn local_host(mut self, host: &'a str) -> Self {
        self.local_host = host;
        self
    }
}

/// Why an `IdentityFile` / `Include` path could not be expanded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpandError {
    /// `%x` for a token we do not implement (OpenSSH: `unknown key %x`).
    UnknownEscape(char),
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
            ExpandError::InvalidFormat => write!(f, "invalid format"),
            ExpandError::MissingVariable(v) => write!(f, "env var ${{{v}}} has no value"),
            ExpandError::MalformedVariable(v) => write!(f, "malformed environment variable ${{{v}}}"),
        }
    }
}

impl std::error::Error for ExpandError {}

/// `tilde_expand_filename()`: expand `~` and `~/...`.
pub fn tilde_expand(path: &str, home: &Path) -> String {
    if path == "~" {
        return home.to_string_lossy().into_owned();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if rest.is_empty() {
            return home.to_string_lossy().into_owned();
        }
        return home.join(rest).to_string_lossy().into_owned();
    }
    path.to_string()
}

/// `vdollar_percent_expand()` restricted to the tokens ssh substitutes in
/// client configuration.
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
                    'h' | 'n' => out.push_str(ctx.host),
                    'p' => out.push_str(&ctx.port.to_string()),
                    'r' => out.push_str(ctx.remote_user),
                    'l' => out.push_str(ctx.local_host),
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
                if !closed {
                    return Err(ExpandError::MalformedVariable(name));
                }
                if name.is_empty() {
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
            .local_host("laptop")
    }

    #[test]
    fn tilde_only_and_slash() {
        let home = Path::new("/home/me");
        assert_eq!(tilde_expand("~", home), "/home/me");
        assert_eq!(tilde_expand("~/", home), "/home/me");
        assert_eq!(tilde_expand("~/.ssh/id_rsa", home), "/home/me/.ssh/id_rsa");
        // `~user` is not expanded (no passwd database here).
        assert_eq!(tilde_expand("~alice/key", home), "~alice/key");
        assert_eq!(tilde_expand("/abs/key", home), "/abs/key");
    }

    #[test]
    fn percent_tokens() {
        let c = ctx();
        assert_eq!(percent_expand("%d/.ssh/id", &c).unwrap(), "/home/me/.ssh/id");
        assert_eq!(percent_expand("%h_key", &c).unwrap(), "example.com_key");
        assert_eq!(percent_expand("%n_key", &c).unwrap(), "example.com_key");
        assert_eq!(percent_expand("%p", &c).unwrap(), "2222");
        assert_eq!(percent_expand("%r", &c).unwrap(), "bob");
        assert_eq!(percent_expand("%u", &c).unwrap(), "me");
        assert_eq!(percent_expand("%l", &c).unwrap(), "laptop");
        assert_eq!(percent_expand("100%%", &c).unwrap(), "100%");
    }

    #[test]
    fn percent_unknown_is_an_error() {
        let c = ctx();
        assert_eq!(
            percent_expand("%C", &c),
            Err(ExpandError::UnknownEscape('C'))
        );
        assert_eq!(percent_expand("trailing%", &c), Err(ExpandError::InvalidFormat));
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
        assert_eq!(percent_expand("$SSH_IDENTITY_TEST_DIR", &c).unwrap(), "$SSH_IDENTITY_TEST_DIR");
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
    }

    #[test]
    fn chained_tilde_then_percent() {
        let c = ctx();
        let raw = "~/.ssh/%h_id";
        let expanded = tilde_expand(raw, PathBuf::from("/home/me").as_path());
        assert_eq!(percent_expand(&expanded, &c).unwrap(), "/home/me/.ssh/example.com_id");
    }
}
