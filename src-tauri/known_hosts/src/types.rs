use std::fmt;

/// Marker at the start of a known_hosts line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    /// No marker (normal entry).
    None,
    /// `@cert-authority` — the key is a CA key.
    CertAuthority,
    /// `@revoked` — the key is revoked.
    Revoked,
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Marker::None => Ok(()),
            Marker::CertAuthority => write!(f, "@cert-authority"),
            Marker::Revoked => write!(f, "@revoked"),
        }
    }
}

/// A host pattern entry (one element of the comma-separated hosts field).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostPattern {
    /// Plaintext hostname / IP / wildcard / `[host]:port`.
    Plain(String),
    /// Hashed hostname `|1|base64salt|base64hash` — stored as-is.
    Hashed(String),
}

impl fmt::Display for HostPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostPattern::Plain(s) => write!(f, "{}", s),
            HostPattern::Hashed(s) => write!(f, "{}", s),
        }
    }
}

/// SSH host key type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    Ed25519,
    Rsa,
    EcdsaP256,
    EcdsaP384,
    EcdsaP521,
    /// Unrecognised key type name.
    Unknown(String),
}

impl KeyType {
    /// Map an SSH key-type name to a `KeyType`.
    pub fn from_name(name: &str) -> Self {
        match name {
            "ssh-ed25519" | "ssh-ed25519-cert-v01@openssh.com" => KeyType::Ed25519,
            "ssh-rsa" | "ssh-rsa-cert-v01@openssh.com" => KeyType::Rsa,
            "ecdsa-sha2-nistp256" | "ecdsa-sha2-nistp256-cert-v01@openssh.com" => {
                KeyType::EcdsaP256
            }
            "ecdsa-sha2-nistp384" | "ecdsa-sha2-nistp384-cert-v01@openssh.com" => {
                KeyType::EcdsaP384
            }
            "ecdsa-sha2-nistp521" | "ecdsa-sha2-nistp521-cert-v01@openssh.com" => {
                KeyType::EcdsaP521
            }
            other => KeyType::Unknown(other.to_string()),
        }
    }

    /// Return the canonical SSH name for this key type.
    pub fn as_str(&self) -> &str {
        match self {
            KeyType::Ed25519 => "ssh-ed25519",
            KeyType::Rsa => "ssh-rsa",
            KeyType::EcdsaP256 => "ecdsa-sha2-nistp256",
            KeyType::EcdsaP384 => "ecdsa-sha2-nistp384",
            KeyType::EcdsaP521 => "ecdsa-sha2-nistp521",
            KeyType::Unknown(s) => s.as_str(),
        }
    }
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// One parsed line from a known_hosts file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEntry {
    /// 1-based line number in the source file.
    pub line_num: usize,
    /// Optional marker (`@cert-authority` / `@revoked`).
    pub marker: Marker,
    /// Comma-separated host patterns.
    pub hosts: Vec<HostPattern>,
    /// Key type (e.g. `ssh-ed25519`).
    pub key_type: KeyType,
    /// Decoded public key bytes (base64 decoded).
    pub key_data: Vec<u8>,
    /// Optional trailing comment.
    pub comment: Option<String>,
    /// The original line this entry was parsed from (used by
    /// `WriteStyle::Preserve` for byte-exact output).
    pub raw_line: String,
}

impl HostEntry {
    /// Return the raw hosts field as a comma-joined string (for display).
    pub fn hosts_string(&self) -> String {
        self.hosts
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// Error produced when a single line cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// 1-based line number.
    pub line_num: usize,
    /// The original line content.
    pub line: String,
    /// Human-readable reason.
    pub reason: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}: {}: {}",
            self.line_num, self.reason, self.line
        )
    }
}

impl std::error::Error for ParseError {}
