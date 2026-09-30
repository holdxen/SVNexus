//! # known_hosts_parser
//!
//! A small library for parsing OpenSSH `known_hosts` files.
//!
//! ## Quick start
//!
//! ```no_run
//! use known_hosts_parser::{parse, lookup, fingerprint};
//!
//! let content = std::fs::read_to_string("known_hosts").unwrap();
//! let (entries, errors) = parse(&content);
//!
//! for e in &errors {
//!     eprintln!("warn: {}", e);
//! }
//!
//! for entry in lookup(&entries, "github.com") {
//!     println!(
//!         "{} {} {}",
//!         entry.hosts_string(),
//!         entry.key_type,
//!         fingerprint(&entry.key_data),
//!     );
//! }
//! ```

mod accept;
mod check;
mod document;
mod fingerprint;
mod hashed;
mod parser;
mod types;
mod writer;

pub use accept::{
    check_and_store_hostkey, check_and_store_hostkey_in_files, HostkeyConfirm, HostkeyWarning,
    IpChangedPrompt, NewHostPrompt, StoreError, StoreResult,
};
pub use check::{check_hostkey, HostStatus};
pub use document::{parse_document, DocLine, Document};
pub use fingerprint::fingerprint;
pub use hashed::{hash_host, hash_host_with_salt, verify_hashed};
pub use parser::{parse, parse_line};
pub use types::{HostEntry, HostPattern, KeyType, Marker, ParseError};
pub use writer::{render, WriteError, WriteStyle};

/// Find all entries whose host patterns match `hostname`.
///
/// Matching follows OpenSSH's `match_maybe_hashed()` (hostfile.c):
/// - Plain patterns: the hosts field is a comma-separated pattern list
///   (`match_pattern_list`) — `*` / `?` wildcards, both sides lowercased;
///   a matching `!`-prefixed subpattern vetoes the whole entry.
/// - Hashed patterns (`|1|…`): HMAC-SHA1 verified against `hostname`,
///   and against `hostname` lowercased as a convenience for callers
///   (ssh itself lowercases the hostname before lookup, ssh.c).
/// - `[host]:port` entries match only the exact `[host]:port` form, and
///   default-port entries are stored bare — pass the same name OpenSSH
///   would use (`put_host_port`: bare for port 22, `[host]:port` else).
pub fn lookup<'a>(entries: &'a [HostEntry], hostname: &str) -> Vec<&'a HostEntry> {
    entries
        .iter()
        .filter(|entry| entry_matches(entry, hostname))
        .collect()
}

/// Whether the hosts field of `entry` matches `name`, using OpenSSH's
/// `match_maybe_hashed()` semantics.
pub(crate) fn entry_matches(entry: &HostEntry, name: &str) -> bool {
    // Whole-field hashed match: OpenSSH keys off the field's first byte
    // being '|'; the recomputed hash must equal the entire field, so a
    // leading hashed component with extra comma parts never matches.
    match entry.hosts.first() {
        Some(HostPattern::Hashed(h)) => {
            if entry.hosts.len() != 1 {
                return false;
            }
            // Lowercase fallback: ssh lowercases the hostname up-front.
            return verify_hashed(h, name) || verify_hashed(h, &name.to_lowercase());
        }
        // A field starting with '|' that we did not parse as hashed is
        // invalid for OpenSSH's extract_salt() — treated as never matching.
        Some(HostPattern::Plain(p)) if p.starts_with('|') => return false,
        _ => {}
    }

    // Plain pattern list with match_pattern_list() semantics: lowercase
    // both sides; '!' prefix negates; a matching negation vetoes the
    // whole entry, otherwise any positive subpattern matches.
    let lname = name.to_lowercase();
    let mut got_positive = false;
    for pattern in &entry.hosts {
        let text = match pattern {
            HostPattern::Plain(p) => p.as_str(),
            // Hashed component in a non-leading position: OpenSSH would
            // treat the literal text as a glob subpattern (never matches
            // a hostname).
            HostPattern::Hashed(h) => h.as_str(),
        };
        let (negated, sub) = match text.strip_prefix('!') {
            Some(rest) => (true, rest),
            None => (false, text),
        };
        if glob_match(&sub.to_lowercase(), &lname) {
            if negated {
                return false;
            }
            got_positive = true;
        }
    }
    got_positive
}

/// Simple glob matcher supporting `*` (any sequence) and `?` (any single char).
fn glob_match(pattern: &str, text: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    glob_match_inner(&p, &t)
}

fn glob_match_inner(p: &[char], t: &[char]) -> bool {
    if p.is_empty() {
        return t.is_empty();
    }
    match p[0] {
        '*' => {
            // Try consuming 0..=len characters from t.
            for i in 0..=t.len() {
                if glob_match_inner(&p[1..], &t[i..]) {
                    return true;
                }
            }
            false
        }
        '?' => !t.is_empty() && glob_match_inner(&p[1..], &t[1..]),
        c => !t.is_empty() && t[0] == c && glob_match_inner(&p[1..], &t[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_line;

    #[test]
    fn glob_exact() {
        assert!(glob_match("github.com", "github.com"));
        assert!(!glob_match("github.com", "gitlab.com"));
    }

    #[test]
    fn glob_wildcard_star() {
        assert!(glob_match("*.example.com", "foo.example.com"));
        assert!(glob_match("*.example.com", "a.b.example.com"));
        assert!(!glob_match("*.example.com", "example.com"));
    }

    #[test]
    fn glob_wildcard_question() {
        assert!(glob_match("host?.example.com", "host1.example.com"));
        assert!(!glob_match("host?.example.com", "host12.example.com"));
    }

    fn entry(hosts_field: &str) -> HostEntry {
        parse_line(&format!("{} ssh-ed25519 AAAA", hosts_field), 1).unwrap()
    }

    #[test]
    fn port_matches_exact_form_only() {
        // OpenSSH (put_host_port): port 22 → bare, else [host]:port,
        // matched literally. No cross-matching between the two forms.
        let e = entry("[ssh.github.com]:443");
        assert!(entry_matches(&e, "[ssh.github.com]:443"));
        assert!(!entry_matches(&e, "ssh.github.com"));

        let bare = entry("ssh.github.com");
        assert!(entry_matches(&bare, "ssh.github.com"));
        assert!(!entry_matches(&bare, "[ssh.github.com]:443"));
    }

    #[test]
    fn negation_vetoes_entry() {
        let e = entry("*.example.com,!evil.example.com");
        assert!(entry_matches(&e, "foo.example.com"));
        assert!(!entry_matches(&e, "evil.example.com"));

        // Positive-only list still requires a positive match.
        let e = entry("!evil.example.com");
        assert!(!entry_matches(&e, "foo.example.com"));
        assert!(!entry_matches(&e, "evil.example.com"));
    }

    #[test]
    fn leading_hashed_field_requires_whole_field() {
        // A leading hashed component means OpenSSH treats the whole field
        // as one hash; extra comma parts make the length check fail.
        let hashed = crate::hash_host("example.com");
        let e = entry(&format!("{},other.example.net", hashed));
        assert!(!entry_matches(&e, "example.com"));
        let single = entry(&hashed);
        assert!(entry_matches(&single, "example.com"));
    }
}
