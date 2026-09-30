use sha2::{Digest, Sha256};

/// Compute the SHA256 fingerprint of raw key bytes, returned in the same
/// base64 (unpadded) format that `ssh-keygen -lf` / `ssh -v` display.
///
/// Example output: `SHA256:nThbg6kXUpJWGl7E1IGOCspRomTxdCARLviKw6E5SY8`
pub fn fingerprint(key_data: &[u8]) -> String {
    let digest = Sha256::digest(key_data);
    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, digest);
    // SSH fingerprints strip the trailing '=' padding.
    let trimmed = b64.trim_end_matches('=');
    format!("SHA256:{}", trimmed)
}
