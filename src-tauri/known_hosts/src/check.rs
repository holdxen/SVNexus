use crate::entry_matches;
use crate::types::{HostEntry, KeyType, Marker};

/// Result of checking a presented host key against known_hosts entries.
///
/// Variant names mirror OpenSSH's `HostStatus` (hostfile.h) for easy
/// cross-referencing, plus two additions:
/// - `HOST_CHANGED_IP_DIFFERS` — the `host_ip_differ` DNS-spoofing signal
///   from sshconnect.c (status stays "changed/reject", with extra warning);
/// - `MismatchHost` — the key is known, but only under host/ip patterns
///   that match neither the given `host` nor `ip`.
///
/// Each variant's doc comment contains a suggested end-user message,
/// modelled on what OpenSSH itself prints (sshconnect.c).
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStatus {
    /// Key matches an entry in the host scope — trusted (HOST_OK).
    ///
    /// **Caller should:** allow the connection. Optionally show the
    /// matched entry's fingerprint for confirmation, e.g. by calling
    /// [`fingerprint`](crate::fingerprint) on the key. No warning needed.
    ///
    /// OpenSSH says:
    /// > Host 'example.com' is known and matches the ED25519 host key.
    HOST_OK,

    /// No relevant entries in scope and the key is unknown globally (HOST_NEW).
    ///
    /// **Caller should:** on first connection, show the key fingerprint and
    /// ask the user to confirm trust before persisting the entry
    /// (like OpenSSH's `StrictHostKeyChecking ask` prompt). Example:
    ///
    /// > The authenticity of host 'example.com (192.0.2.1)' can't be
    /// > established.
    /// > ED25519 key fingerprint is `SHA256:…`.
    /// > Are you sure you want to continue connecting (yes/no)?
    ///
    /// If the caller auto-trusts (e.g. `accept-new` mode), just record the
    /// entry and inform:
    ///
    /// > Warning: Permanently added 'example.com' (ED25519) to the list
    /// > of known hosts.
    HOST_NEW,

    /// Scope has entries but none match this key — would be replaced (HOST_CHANGED).
    ///
    /// **Caller should:** refuse the connection and display a prominent
    /// warning — this is the man-in-the-middle / key-rotation case, and
    /// the user must NOT be invited to blindly continue. Point them at
    /// the offending entry so they can update or remove it. Example
    /// (OpenSSH's banner):
    ///
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > @    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > IT IS POSSIBLE THAT SOMEONE IS DOING SOMETHING NASTY!
    /// > Offending ED25519 key in known_hosts:12
    /// > remove that key and connect again, or add the correct key.
    ///
    /// (When the key merely rotated, the legitimate fix is to tell the
    /// user to verify the new key out-of-band, then replace the entry —
    /// which is exactly what [`HostEntry`](crate::HostEntry) writing with
    /// [`WriteStyle`](crate::WriteStyle) can do.)
    HOST_CHANGED,

    /// Key matches an `@revoked` entry in scope (HOST_REVOKED, highest priority).
    ///
    /// **Caller should:** hard-refuse the connection with a security
    /// warning and do not offer an "anyway" override. Example (OpenSSH):
    ///
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > @       WARNING: REVOKED HOST KEY DETECTED!               @
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > The ED25519 host key for example.com is marked as revoked.
    /// > This could mean that a stolen key is being used to
    /// > impersonate this host.
    HOST_REVOKED,

    /// `HOST_CHANGED` and the IP scope disagrees — ssh's
    /// "POSSIBLE DNS SPOOFING" signal.
    ///
    /// **Caller should:** behave exactly as for [`HOST_CHANGED`](Self::HOST_CHANGED)
    /// (refuse + prominent warning), and additionally explain that the key
    /// seen for the IP address differs from the one known for the hostname —
    /// the classic DNS-hijack pattern. Example (OpenSSH):
    ///
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > @       WARNING: POSSIBLE DNS SPOOFING DETECTED!          @
    /// > @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    /// > The ED25519 host key for example.com has changed,
    /// > and the key for the corresponding IP address 192.0.2.1
    /// > is unknown / is unchanged / has a different value.
    /// > This could either mean that DNS SPOOFING is happening, or the
    /// > IP address for the host and its host key have changed at the
    /// > same time.
    HOST_CHANGED_IP_DIFFERS,

    /// Key exists in known_hosts, but only under entries matching
    /// neither `host` nor `ip`.
    ///
    /// **Caller should:** treat as a first-contact for *this* name, but
    /// surface that the key is already trusted elsewhere — it often means
    /// the user reached the same machine via a different alias (bastion,
    /// internal vs. public name) or that someone is re-using a known key
    /// under a new name. Show both facts and ask for explicit confirmation:
    ///
    /// > The host 'alias.example.com' is unknown, but the presented
    /// > ED25519 key already matches the known host 'example.com'.
    /// > Do you want to trust 'alias.example.com' with this same key
    /// > (yes/no)?
    ///
    /// (Modelled on OpenSSH's `other_hostkeys_message`, which appends
    /// "but host key … does match a key …" to the HOST_NEW prompt.)
    MismatchHost,
}

/// Classification of the scope evaluation (per OpenSSH precedence).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeStatus {
    Ok,
    Changed,
    New,
    Revoked,
}

/// Byte-exact key comparison: type name + wire blob (known_hosts column 3).
pub(crate) fn key_eq(entry: &HostEntry, key_type: &KeyType, key_data: &[u8]) -> bool {
    entry.key_type == *key_type && entry.key_data == key_data
}

/// Whether OpenSSH would be able to load this entry's key at all
/// (`hostfile_read_key` / `sshkey_read` fails → INVALID → not loaded).
///
/// This checks: known key type (not `Unknown`), and a wire blob whose
/// length-prefixed fields parse exactly, starting with the type name
/// (or its certificate form).
pub(crate) fn key_loadable(entry: &HostEntry) -> bool {
    if matches!(entry.key_type, KeyType::Unknown(_)) {
        return false;
    }
    let type_name = entry.key_type.as_str().as_bytes();
    let cert_name = format!("{}-cert-v01@openssh.com", entry.key_type.as_str());
    let data = &entry.key_data;
    let mut pos = 0usize;
    let mut saw_type = false;
    while pos < data.len() {
        if data.len() - pos < 4 {
            return false;
        }
        let len =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        if len > data.len() - pos {
            return false;
        }
        if !saw_type {
            let field = &data[pos..pos + len];
            if field != type_name && field != cert_name.as_bytes() {
                return false;
            }
            saw_type = true;
        }
        pos += len;
    }
    saw_type
}

/// Evaluate a (pre-filtered) scope exactly like OpenSSH's
/// `check_hostkeys_by_key_or_type()`:
/// 1. any `@revoked` entry with an equal key → Revoked (overrides);
/// 2. otherwise scan `Marker::None` entries (CA entries are skipped —
///    certificates are not supported): equal key → Ok;
/// 3. at least one such entry but none equal → Changed, with the *last*
///    plain entry as "found" (matching ssh's found-pointer updates);
/// 4. no plain entries → New.
fn eval_scope<'a>(
    scope: &[&'a HostEntry],
    key_type: &KeyType,
    key_data: &[u8],
) -> (ScopeStatus, Option<&'a HostEntry>) {
    if scope
        .iter()
        .any(|e| e.marker == Marker::Revoked && key_eq(e, key_type, key_data))
    {
        return (ScopeStatus::Revoked, None);
    }
    let mut last_plain: Option<&HostEntry> = None;
    for entry in scope {
        if entry.marker != Marker::None {
            continue;
        }
        if key_eq(entry, key_type, key_data) {
            return (ScopeStatus::Ok, Some(entry));
        }
        last_plain = Some(entry);
    }
    if last_plain.is_some() {
        (ScopeStatus::Changed, last_plain)
    } else {
        (ScopeStatus::New, None)
    }
}

/// Check a presented host key against known_hosts entries.
///
/// - `host` / `ip` are optional names used to scope the check (glob,
///   hashed and `[host]:port` patterns all apply, as in [`lookup`](crate::lookup)).
/// - The returned status is derived from the **host** scope alone, like
///   OpenSSH's acceptance decision; `ip` only contributes
///   `HOST_CHANGED_IP_DIFFERS` when the host scope says `HOST_CHANGED`
///   and the IP scope disagrees (ssh's DNS-spoofing warning).
/// - `key_data` must be the full wire blob (known_hosts column 3),
///   including the length-prefixed type name; equality is byte-exact
///   against `entry.key_data`, and `key_type` must match too.
/// - `@cert-authority` entries are ignored (no certificate support).
/// - If both `host` and `ip` are `None` the whole entry list is the
///   scope and `MismatchHost` can never be returned.
pub fn check_hostkey(
    entries: &[HostEntry],
    host: Option<&str>,
    ip: Option<&str>,
    key_type: &KeyType,
    key_data: &[u8],
) -> HostStatus {
    // Host scope: host, falling back to ip; both None → everything.
    // Entries whose keys OpenSSH would fail to load (unknown type,
    // malformed wire blob) are excluded — they never enter hostkeys.
    let primary = host.or(ip);
    let scope: Vec<&HostEntry> = entries
        .iter()
        .filter(|e| key_loadable(e))
        .filter(|e| primary.is_none_or(|name| entry_matches(e, name)))
        .collect();

    let (status, found) = eval_scope(&scope, key_type, key_data);
    match status {
        ScopeStatus::Revoked => return HostStatus::HOST_REVOKED,
        ScopeStatus::Ok => return HostStatus::HOST_OK,
        ScopeStatus::New => {
            // Key unknown for this scope: distinguish "belongs to
            // somebody else" from genuinely new. Entries matching the
            // given host or ip do not count as a mismatch.
            let mismatch = entries.iter().any(|e| {
                key_loadable(e)
                    && e.marker == Marker::None
                    && key_eq(e, key_type, key_data)
                    && !host.is_some_and(|h| entry_matches(e, h))
                    && !ip.is_some_and(|a| entry_matches(e, a))
            });
            return if mismatch {
                HostStatus::MismatchHost
            } else {
                HostStatus::HOST_NEW
            };
        }
        ScopeStatus::Changed => {}
    }

    // HOST_CHANGED + optional IP cross-check (sshconnect.c:1063-1067):
    // host_ip_differ = ip_status != CHANGED || found keys differ.
    if let (Some(_h), Some(a)) = (host, ip) {
        let ip_scope: Vec<&HostEntry> = entries
            .iter()
            .filter(|e| key_loadable(e))
            .filter(|e| entry_matches(e, a))
            .collect();
        let (ip_status, ip_found) = eval_scope(&ip_scope, key_type, key_data);
        let ip_changed = ip_status == ScopeStatus::Changed;
        let keys_differ = match (found, ip_found) {
            (Some(hf), Some(if_)) => !(hf.key_type == if_.key_type && hf.key_data == if_.key_data),
            // One side has no "found" entry while claiming Changed —
            // impossible for Changed, but treat as disagreeing.
            _ => true,
        };
        if !ip_changed || keys_differ {
            return HostStatus::HOST_CHANGED_IP_DIFFERS;
        }
    }
    HostStatus::HOST_CHANGED
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hash_host, parse_line};

    const KEY_A: &str =
        "AAAAC3NzaC1lZDI1NTE5AAAAIK9ks7jkua5YWIwByRnnnc6UPJQWI75O0e/UJdPYU1JI";
    const KEY_B: &str =
        "AAAAC3NzaC1lZDI1NTE5AAAAIBlYfExtYZAPqYvYdrlpGlSWhh/XNHcH3v3c2JzsVNbB";

    fn key_of(blob: &str) -> (KeyType, Vec<u8>) {
        let e = parse_line(&format!("h ssh-ed25519 {}", blob), 1).unwrap();
        (e.key_type, e.key_data)
    }

    fn entries(lines: &[&str]) -> Vec<HostEntry> {
        lines
            .iter()
            .enumerate()
            .map(|(i, l)| parse_line(l, i + 1).unwrap())
            .collect()
    }

    #[test]
    fn empty_is_new() {
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&[], Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_NEW
        );
    }

    #[test]
    fn ok_when_key_matches() {
        let es = entries(&[&format!("example.com ssh-ed25519 {}", KEY_A)]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_OK
        );
        // Scope is per-host: the key is known, but under example.com only.
        assert_eq!(
            check_hostkey(&es, Some("other.example.org"), None, &kt, &kd),
            HostStatus::MismatchHost
        );
    }

    #[test]
    fn changed_when_different_key() {
        let es = entries(&[&format!("example.com ssh-ed25519 {}", KEY_A)]);
        let (kt, kd) = key_of(KEY_B);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_CHANGED
        );
    }

    #[test]
    fn revoked_overrides_ok() {
        let es = entries(&[
            &format!("example.com ssh-ed25519 {}", KEY_A),
            &format!("@revoked example.com ssh-ed25519 {}", KEY_A),
        ]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_REVOKED
        );
    }

    #[test]
    fn mismatch_host_when_key_belongs_elsewhere() {
        let es = entries(&[&format!("other.example.net ssh-ed25519 {}", KEY_A)]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::MismatchHost
        );
        // An entry matching the ip is NOT a mismatch.
        let es = entries(&[&format!("192.0.2.1 ssh-ed25519 {}", KEY_A)]);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), Some("192.0.2.1"), &kt, &kd),
            HostStatus::HOST_NEW
        );
    }

    #[test]
    fn ip_differs_when_ip_scope_disagrees() {
        let (kt, kd_b) = key_of(KEY_B);
        // Host scope: only old key A → CHANGED; ip scope: new key B → OK.
        let es = entries(&[
            &format!("example.com ssh-ed25519 {}", KEY_A),
            &format!("192.0.2.1 ssh-ed25519 {}", KEY_B),
        ]);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), Some("192.0.2.1"), &kt, &kd_b),
            HostStatus::HOST_CHANGED_IP_DIFFERS
        );

        // Both scopes changed with the SAME offending key → plain CHANGED.
        let es = entries(&[
            &format!("example.com ssh-ed25519 {}", KEY_A),
            &format!("192.0.2.1 ssh-ed25519 {}", KEY_A),
        ]);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), Some("192.0.2.1"), &kt, &kd_b),
            HostStatus::HOST_CHANGED
        );

        // No ip → no escalation.
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd_b),
            HostStatus::HOST_CHANGED
        );
    }

    #[test]
    fn ca_entries_ignored() {
        let es = entries(&[&format!(
            "@cert-authority example.com ssh-ed25519 {}",
            KEY_A
        )]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_NEW
        );
    }

    #[test]
    fn hashed_and_uppercase_scope() {
        let hashed = hash_host("example.com");
        let es = entries(&[&format!("{} ssh-ed25519 {}", hashed, KEY_A)]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_OK
        );
        assert_eq!(
            check_hostkey(&es, Some("EXAMPLE.COM"), None, &kt, &kd),
            HostStatus::HOST_OK
        );
    }

    #[test]
    fn both_none_uses_whole_set() {
        let es = entries(&[&format!("example.com ssh-ed25519 {}", KEY_A)]);
        let (kt_a, kd_a) = key_of(KEY_A);
        let (_, kd_b) = key_of(KEY_B);
        assert_eq!(
            check_hostkey(&es, None, None, &kt_a, &kd_a),
            HostStatus::HOST_OK
        );
        assert_eq!(
            check_hostkey(&es, None, None, &kt_a, &kd_b),
            HostStatus::HOST_CHANGED
        );
    }

    #[test]
    fn malformed_and_unknown_entries_ignored_in_scope() {
        // Both entries parse structurally but OpenSSH's key loader
        // would reject them (truncated blob / unknown type) → not in
        // hostkeys → status is HOST_NEW, not HOST_CHANGED.
        let es = entries(&[
            "example.com ssh-ed25519 AAAATgAAAAdz",
            "example.com ssh-XXX AAAATgAAAAdzc2gtWFhY",
        ]);
        let (kt, kd) = key_of(KEY_A);
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt, &kd),
            HostStatus::HOST_NEW
        );
    }

    #[test]
    fn port_form_scope_exact() {
        // `[example.com]:2222` entry is in scope only for the exact
        // `[example.com]:2222` form (put_host_port), never for bare.
        let es = entries(&[&format!("[example.com]:2222 ssh-ed25519 {}", KEY_A)]);
        let (kt_a, kd_a) = key_of(KEY_A);
        let (kt_b, kd_b) = key_of(KEY_B);

        // Exact form → in scope → OK.
        assert_eq!(
            check_hostkey(&es, Some("[example.com]:2222"), None, &kt_a, &kd_a),
            HostStatus::HOST_OK
        );
        // Bare name → entry not in scope; presenting its key surfaces
        // the "known elsewhere" signal (OpenSSH other_hostkeys_message).
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt_a, &kd_a),
            HostStatus::MismatchHost
        );
        // Bare name + a key unknown anywhere → NEW (would be CHANGED
        // if the ported entry leaked into the bare-name scope).
        assert_eq!(
            check_hostkey(&es, Some("example.com"), None, &kt_b, &kd_b),
            HostStatus::HOST_NEW
        );
    }
}
