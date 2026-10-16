//! SHA-1, used only to reproduce OpenSSH's `%C` connection hash.
//!
//! `readconf.c:345 ssh_connection_hash()` feeds five strings to
//! `SSH_DIGEST_SHA1` with no separators and hex-encodes the 20 result bytes.
//! Not for security use — it exists so `IdentityFile ~/.ssh/%C` resolves to
//! the same path `ssh` would pick.

/// `ssh_digest_memory(SSH_DIGEST_SHA1, ..)`
pub(super) fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 80];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            *word = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let (mut a, mut b, mut c, mut d, mut e) = (h[0], h[1], h[2], h[3], h[4]);
        for (i, wi) in w.iter().enumerate() {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*wi);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        for (acc, value) in h.iter_mut().zip([a, b, c, d, e]) {
            *acc = acc.wrapping_add(value);
        }
    }

    let mut out = [0u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// `readconf.c:360 tohex()` — lower case, no separators.
pub(super) fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_vectors() {
        assert_eq!(
            to_hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            to_hex(&sha1(b"")),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        // 56/57 byte inputs exercise the two-block padding branch.
        assert_eq!(
            to_hex(&sha1(&[b'a'; 56])),
            "c2db330f6083854c99d4b5bfb6e8f29f201be699"
        );
        assert_eq!(
            to_hex(&sha1(&[b'a'; 57])),
            "f08f24908d682555111be7ff6f004e78283d989a"
        );
    }

    #[test]
    fn matches_openssh_connection_hash() {
        // Observed from `ssh -vv` on 2026-10-12 with OpenSSH_10.3p1, where
        // `IdentityFile /k/%C` resolved to
        // /k/5cb7925d5491da22e829af92f450cb232d9db7f5 for
        // thishost=zhouguiqingdeMac-mini.local host=nope.invalid
        // port=22 ruser=zhouguiqing jmphost="".
        let mut input = String::new();
        for part in [
            "zhouguiqingdeMac-mini.local",
            "nope.invalid",
            "22",
            "zhouguiqing",
            "",
        ] {
            input.push_str(part);
        }
        assert_eq!(
            to_hex(&sha1(input.as_bytes())),
            "5cb7925d5491da22e829af92f450cb232d9db7f5"
        );
    }
}
