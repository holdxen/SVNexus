use base64::Engine;

use crate::hashed::{hash_host, hash_host_with_salt};
use crate::types::{HostEntry, HostPattern, Marker};

/// HMAC salt length used by OpenSSH (`ssh_hmac_bytes(SSH_DIGEST_SHA1)`).
pub(crate) const SALT_LEN: usize = 20;

/// Wildcard/negation characters that make a host pattern unhashable,
/// matching `ssh-keygen -H`'s `strcspn(hosts, "*?!")` check.
const WILDCARD_CHARS: [char; 3] = ['*', '?', '!'];

/// How to serialize entries back to known_hosts lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteStyle {
    /// Keep the original line byte-for-byte (falls back to canonical
    /// formatting for entries built by hand with an empty `raw_line`).
    Preserve,
    /// Canonical rewrite that never introduces hashing; existing
    /// `|1|…` patterns are kept as-is (hashing is one-way).
    Plaintext,
    /// Hash plain hostnames the way `ssh-keygen -H` does:
    /// - skip the whole entry if the marker is set, if the hosts field
    ///   contains `*`, `?` or `!`, or if any pattern is already hashed;
    /// - lowercase each pattern, then HMAC-SHA1 it (fresh random salt
    ///   per pattern, or a shared caller-supplied salt), emitting one
    ///   line per host — mirroring `known_hosts_hash()` in ssh-keygen.c.
    Hashed,
}

/// Error produced when an entry cannot be serialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// The `hosts` field is empty — no legal line can be produced.
    EmptyHosts,
    /// Caller-supplied salt is not 20 bytes; payload is the length.
    InvalidSalt(usize),
    /// The style would produce more than one line (a hashed multi-host
    /// entry splits, like `ssh-keygen -H`); use `to_lines()` instead.
    MultipleLines,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::EmptyHosts => write!(f, "hosts field is empty"),
            WriteError::InvalidSalt(len) => write!(
                f,
                "invalid salt length {} (must be {} bytes)",
                len, SALT_LEN
            ),
            WriteError::MultipleLines => write!(
                f,
                "entry renders as multiple lines; use to_lines() instead"
            ),
        }
    }
}

impl std::error::Error for WriteError {}

pub(crate) fn check_salt(salt: &[u8]) -> Result<(), WriteError> {
    if salt.len() == SALT_LEN {
        Ok(())
    } else {
        Err(WriteError::InvalidSalt(salt.len()))
    }
}

impl HostEntry {
    /// Serialize this entry as a single known_hosts line (no newline).
    ///
    /// Returns [`WriteError::MultipleLines`] when `style` is
    /// [`WriteStyle::Hashed`] and the entry has several host patterns
    /// (they split into one line per host, like `ssh-keygen -H`); use
    /// [`to_lines`](Self::to_lines) instead.
    pub fn to_line(&self, style: WriteStyle) -> Result<String, WriteError> {
        self.to_line_impl(style, None)
    }

    /// Like [`to_line`](Self::to_line), but with a caller-supplied salt for
    /// deterministic hashing (tests / reproducible output). Salt must be
    /// 20 bytes.
    pub fn to_line_with_salt(
        &self,
        style: WriteStyle,
        salt: &[u8],
    ) -> Result<String, WriteError> {
        check_salt(salt)?;
        self.to_line_impl(style, Some(salt))
    }

    /// Serialize this entry as known_hosts line(s), one string per output
    /// line (no newline). A hashed multi-host entry becomes one line per
    /// host, mirroring `known_hosts_hash()` in ssh-keygen.c: each line
    /// repeats the marker, key and comment.
    pub fn to_lines(&self, style: WriteStyle) -> Result<Vec<String>, WriteError> {
        self.to_lines_impl(style, None)
    }

    /// Like [`to_lines`](Self::to_lines), but with a caller-supplied salt
    /// shared by every hashed line (deterministic output). Salt must be
    /// 20 bytes.
    pub fn to_lines_with_salt(
        &self,
        style: WriteStyle,
        salt: &[u8],
    ) -> Result<Vec<String>, WriteError> {
        check_salt(salt)?;
        self.to_lines_impl(style, Some(salt))
    }

    fn to_line_impl(
        &self,
        style: WriteStyle,
        salt: Option<&[u8]>,
    ) -> Result<String, WriteError> {
        let mut lines = self.to_lines_impl(style, salt)?;
        match lines.len() {
            1 => Ok(lines.swap_remove(0)),
            _ => Err(WriteError::MultipleLines),
        }
    }

    fn to_lines_impl(
        &self,
        style: WriteStyle,
        salt: Option<&[u8]>,
    ) -> Result<Vec<String>, WriteError> {
        if style == WriteStyle::Preserve && !self.raw_line.is_empty() {
            return Ok(vec![self.raw_line.clone()]);
        }
        if self.hosts.is_empty() {
            return Err(WriteError::EmptyHosts);
        }

        match style {
            WriteStyle::Preserve | WriteStyle::Plaintext => {
                Ok(vec![self.build_line(&self.hosts_string())])
            }
            WriteStyle::Hashed => {
                if self.should_skip_hash() {
                    return Ok(vec![self.build_line(&self.hosts_string())]);
                }
                // should_skip_hash() is false only when every pattern is
                // plain: hash each one, one output line per host —
                // mirroring known_hosts_hash() in ssh-keygen.c.
                Ok(self
                    .hosts
                    .iter()
                    .map(|pattern| {
                        let HostPattern::Plain(name) = pattern else {
                            // Unreachable: should_skip_hash() rejects
                            // entries containing hashed patterns.
                            return self.build_line(&self.hosts_string());
                        };
                        let lower = name.to_lowercase();
                        let hashed = match salt {
                            Some(salt) => hash_host_with_salt(&lower, salt),
                            None => hash_host(&lower),
                        };
                        self.build_line(&hashed)
                    })
                    .collect())
            }
        }
    }

    /// Assemble one physical known_hosts line from an already-rendered
    /// hosts field (marker + field + key + base64 + optional comment).
    fn build_line(&self, hosts_field: &str) -> String {
        let mut line = String::new();
        if !matches!(self.marker, Marker::None) {
            line.push_str(&self.marker.to_string());
            line.push(' ');
        }
        line.push_str(hosts_field);
        line.push(' ');
        line.push_str(self.key_type.as_str());
        line.push(' ');
        line.push_str(&base64::engine::general_purpose::STANDARD.encode(&self.key_data));
        if let Some(comment) = &self.comment {
            if !comment.is_empty() {
                line.push(' ');
                line.push_str(comment);
            }
        }
        line
    }

    /// `ssh-keygen -H` skip conditions: markers, wildcard/negation
    /// characters anywhere in the hosts field, or already-hashed patterns.
    fn should_skip_hash(&self) -> bool {
        if !matches!(self.marker, Marker::None) {
            return true;
        }
        let field = self.hosts_string();
        if field.contains(WILDCARD_CHARS) {
            return true;
        }
        self.hosts
            .iter()
            .any(|p| matches!(p, HostPattern::Hashed(_)))
    }
}

/// Serialize entries (comment/blank lines not applicable) as a file body:
/// each line terminated by `\n`. A hashed multi-host entry contributes
/// several lines (one per host), like `ssh-keygen -H`.
pub fn render(entries: &[HostEntry], style: WriteStyle) -> Result<String, WriteError> {
    let mut out = String::new();
    for entry in entries {
        for line in entry.to_lines(style)? {
            out.push_str(&line);
            out.push('\n');
        }
    }
    Ok(out)
}
