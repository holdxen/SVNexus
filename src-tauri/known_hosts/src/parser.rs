use base64::Engine;

use crate::types::{HostEntry, HostPattern, KeyType, Marker, ParseError};

const CA_MARKER: &str = "@cert-authority";
const REVOKE_MARKER: &str = "@revoked";

/// Parse an entire known_hosts file.
///
/// Returns successfully-parsed entries and a list of per-line errors.
/// Invalid lines do not abort the overall parse.
pub fn parse(content: &str) -> (Vec<HostEntry>, Vec<ParseError>) {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        // Skip blank lines and comments.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match parse_line(trimmed, line_num) {
            Ok(entry) => entries.push(entry),
            Err(err) => errors.push(err),
        }
    }
    (entries, errors)
}

/// Parse a single known_hosts line (already trimmed of leading/trailing whitespace).
pub fn parse_line(line: &str, line_num: usize) -> Result<HostEntry, ParseError> {
    let err = |reason: &str| ParseError {
        line_num,
        line: line.to_string(),
        reason: reason.to_string(),
    };

    // --- 1. optional marker -------------------------------------------
    let mut rest = line;
    let mut marker = Marker::None;
    if rest.starts_with('@') {
        let (token, remainder) = split_first_token(rest)
            .ok_or_else(|| err("marker not followed by whitespace"))?;
        marker = match token {
            CA_MARKER => Marker::CertAuthority,
            REVOKE_MARKER => Marker::Revoked,
            _ => return Err(err(&format!("unknown marker `{}`", token))),
        };
        rest = remainder;
    }

    // --- 2. hosts field -------------------------------------------------
    let (hosts_field, remainder) =
        split_first_token(rest).ok_or_else(|| err("missing host field"))?;
    if hosts_field.is_empty() {
        return Err(err("empty host field"));
    }

    let hosts: Vec<HostPattern> = hosts_field
        .split(',')
        .map(|part| {
            if part.starts_with("|1|") {
                HostPattern::Hashed(part.to_string())
            } else {
                HostPattern::Plain(part.to_string())
            }
        })
        .collect();

    // --- 3. key type ----------------------------------------------------
    let (keytype_field, remainder) =
        split_first_token(remainder).ok_or_else(|| err("missing key type"))?;
    let key_type = KeyType::from_name(keytype_field);

    // --- 4. base64 key data --------------------------------------------
    let (b64_field, remainder) =
        split_first_token(remainder).ok_or_else(|| err("missing key data"))?;
    let key_data = base64::engine::general_purpose::STANDARD
        .decode(b64_field)
        .map_err(|e| err(&format!("invalid base64 key data: {}", e)))?;

    // --- 5. optional comment -------------------------------------------
    let comment = {
        let c = remainder.trim();
        if c.is_empty() {
            None
        } else {
            Some(c.to_string())
        }
    };

    Ok(HostEntry {
        line_num,
        marker,
        hosts,
        key_type,
        key_data,
        comment,
        raw_line: line.to_string(),
    })
}

/// Split the first whitespace-delimited token from `s`.
/// Returns `(token, rest)` where `rest` starts after the whitespace.
/// If there is no whitespace, returns `(s, "")`.
fn split_first_token(s: &str) -> Option<(&str, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }
    match s.find(char::is_whitespace) {
        Some(pos) => {
            let token = &s[..pos];
            let rest = s[pos..].trim_start();
            Some((token, rest))
        }
        None => Some((s, "")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_first_token_basic() {
        assert_eq!(split_first_token("a b c"), Some(("a", "b c")));
        assert_eq!(split_first_token("  a\tb"), Some(("a", "b")));
        assert_eq!(split_first_token("single"), Some(("single", "")));
        assert_eq!(split_first_token(""), None);
        assert_eq!(split_first_token("   "), None);
    }
}
