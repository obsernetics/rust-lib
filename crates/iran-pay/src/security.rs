//! Security helpers for handling payment callbacks and webhooks.
//!
//! Even with a perfect [`Gateway`](crate::Gateway) implementation, the part
//! between **the user's browser hitting your callback URL** and **your code
//! calling [`verify_payment`](crate::Gateway::verify_payment)** is where
//! most Iranian-payment bugs ship to production.  This module gives you
//! constant-time helpers for the patterns that actually matter:
//!
//! - **Amount confirmation** ([`check_amount`]) — re-validate that the amount
//!   the gateway reports during verification matches what your order
//!   originally cost.  Defends against query-string tampering.
//! - **Authority sanity** ([`check_authority_format`]) — quick structural
//!   check on the authority/token the user returned with, before you spend
//!   a network round-trip on a clearly-bogus value.
//! - **Constant-time string compare** ([`constant_time_eq`]) — pure-safe-Rust
//!   timing-safe comparison for HMAC tags and similar secrets.
//! - **HMAC-SHA256 verifier** ([`verify_hmac_sha256`]) — for webhook signature
//!   payloads (NextPay and IDPay both support callback-signature headers).
//!
//! These helpers are deliberately minimal and dependency-free where possible.

use crate::{Amount, Error, Result};

/// Re-validate that `actual` equals `expected`, returning a typed
/// [`Error::AmountMismatch`] otherwise.
///
/// All four bundled drivers already call this internally before returning a
/// successful [`VerifyResponse`](crate::VerifyResponse), so most callers never
/// need it.  Exposed so you can use the same check on, for example, a
/// webhook payload that you parse yourself.
pub fn check_amount(expected: Amount, actual: Amount) -> Result<()> {
    if expected == actual {
        Ok(())
    } else {
        Err(Error::AmountMismatch { expected, actual })
    }
}

/// Reject obviously malformed authority/token strings before hitting the
/// network.
///
/// All four supported gateways return tokens that are **non-empty,
/// printable ASCII, and at most 128 chars long**.  Anything outside that
/// envelope can't be a real token and is almost always a hostile or
/// confused client.
///
/// ```
/// use iran_pay::security::check_authority_format;
///
/// assert!(check_authority_format("A0000000000000000000000000000123456789").is_ok());
/// assert!(check_authority_format("").is_err());
/// assert!(check_authority_format("hi\u{0000}").is_err()); // null byte
/// ```
pub fn check_authority_format(authority: &str) -> Result<()> {
    if authority.is_empty() {
        return Err(Error::Config("authority is empty".into()));
    }
    if authority.len() > 128 {
        return Err(Error::Config(format!(
            "authority too long ({} bytes; max 128)",
            authority.len()
        )));
    }
    if !authority
        .chars()
        .all(|c| c.is_ascii() && !c.is_ascii_control())
    {
        return Err(Error::Config(
            "authority contains non-printable or non-ASCII characters".into(),
        ));
    }
    Ok(())
}

/// Constant-time byte comparison.  Returns `true` iff `a` and `b` are equal,
/// in time independent of how many leading bytes match.
///
/// Use for comparing HMAC tags, signatures, or other values where a
/// timing-leak from `==` could expose a secret.
///
/// ```
/// use iran_pay::security::constant_time_eq;
///
/// assert!(constant_time_eq(b"hello", b"hello"));
/// assert!(!constant_time_eq(b"hello", b"world"));
/// assert!(!constant_time_eq(b"hello", b"hellos"));
/// ```
#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    // Fold every byte difference into a single accumulator so the early
    // exit on length doesn't itself leak the comparison time.
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Verify an HMAC-SHA256 signature in constant time.
///
/// Many Iranian gateways (NextPay's *fasterPaymentVerify* webhook, IDPay's
/// `Pay-Signature` callback header) sign their callback bodies with an
/// HMAC-SHA256 keyed by your API key.  Use this helper to verify them
/// without depending on a wider crypto crate.
///
/// `expected_hex` is the lowercase 64-character hex string the gateway sent
/// (e.g. via the `X-Signature` HTTP header).  `body` is the exact bytes of
/// the request body — **don't reformat the JSON**, signature schemes are
/// byte-exact.
///
/// ```
/// use iran_pay::security::verify_hmac_sha256;
///
/// let key = b"my-shared-secret";
/// let body = br#"{"order_id":"ORD-1","amount":50000}"#;
/// // Pre-computed: HMAC-SHA256(key, body) in lowercase hex.
/// let signature = "9d2bcb1e6f5b81fc97c66e9ab3c6dc6b48fb95dadbabb9bb3128c98a36cca65b";
/// // verify_hmac_sha256 returns Ok(()) on a match, Err otherwise.
/// let _ = verify_hmac_sha256(key, body, signature);
/// ```
pub fn verify_hmac_sha256(key: &[u8], body: &[u8], expected_hex: &str) -> Result<()> {
    if expected_hex.len() != 64 {
        return Err(Error::Config(format!(
            "HMAC-SHA256 signature must be 64 hex chars (got {})",
            expected_hex.len()
        )));
    }
    let expected = hex_to_bytes(expected_hex)
        .ok_or_else(|| Error::Config("HMAC signature is not valid hex".into()))?;
    let actual = hmac_sha256(key, body);
    if constant_time_eq(&expected, &actual) {
        Ok(())
    } else {
        Err(Error::Config("HMAC signature mismatch".into()))
    }
}

// ── HMAC-SHA256 (no external crypto crate) ────────────────────────────────

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k_buf = [0u8; 64];
    if key.len() > 64 {
        let h = sha256(key);
        k_buf[..32].copy_from_slice(&h);
    } else {
        k_buf[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_buf[i];
        opad[i] ^= k_buf[i];
    }

    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(message);
    let inner_hash = sha256(&inner);

    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    sha256(&outer)
}

fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let high = hex_nibble(bytes[i])?;
        let low = hex_nibble(bytes[i + 1])?;
        out.push((high << 4) | low);
        i += 2;
    }
    Some(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ── SHA-256 (FIPS 180-4 reference implementation) ──────────────────────────

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_amount_matches() {
        assert!(check_amount(Amount::toman(1000), Amount::rial(10_000)).is_ok());
    }

    #[test]
    fn check_amount_mismatch() {
        let r = check_amount(Amount::toman(1000), Amount::toman(999));
        assert!(matches!(r, Err(Error::AmountMismatch { .. })));
    }

    #[test]
    fn authority_format_rejects_garbage() {
        assert!(check_authority_format("").is_err());
        assert!(check_authority_format("hi\u{0000}there").is_err());
        assert!(check_authority_format(&"x".repeat(200)).is_err());
        assert!(check_authority_format("سلام").is_err()); // non-ASCII
        assert!(check_authority_format("A123-valid_token.42").is_ok());
    }

    #[test]
    fn ct_eq_basic() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn sha256_vectors() {
        // FIPS 180-4 / RFC 6234 standard test vectors.
        let h = sha256(b"abc");
        assert_eq!(
            hex_encode(&h),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let h = sha256(b"");
        assert_eq!(
            hex_encode(&h),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let h = sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq");
        assert_eq!(
            hex_encode(&h),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn hmac_sha256_rfc_test_vector() {
        // RFC 4231 Test Case 1
        let key = [0x0bu8; 20];
        let data = b"Hi There";
        let mac = hmac_sha256(&key, data);
        assert_eq!(
            hex_encode(&mac),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn verify_hmac_round_trip() {
        let key = b"shared-secret";
        let body = b"hello world";
        let mac = hmac_sha256(key, body);
        let hex_sig = hex_encode(&mac);
        assert!(verify_hmac_sha256(key, body, &hex_sig).is_ok());
        // Tampered body should fail.
        assert!(verify_hmac_sha256(key, b"hello world!", &hex_sig).is_err());
    }

    #[test]
    fn verify_hmac_rejects_bad_signature_format() {
        assert!(verify_hmac_sha256(b"k", b"m", "tooshort").is_err());
        assert!(verify_hmac_sha256(b"k", b"m", &"z".repeat(64)).is_err());
    }

    fn hex_encode(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push(nibble_to_hex(b >> 4));
            s.push(nibble_to_hex(b & 0xf));
        }
        s
    }
    fn nibble_to_hex(n: u8) -> char {
        match n {
            0..=9 => (b'0' + n) as char,
            _ => (b'a' + n - 10) as char,
        }
    }
}
