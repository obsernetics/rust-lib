//! Iranian legal/company national ID (شناسه ملی) validation.
//!
//! Issued by the Iranian National Organization for Civil Registration to
//! every legally-registered entity (company, foundation, NGO).  Format is
//! exactly 11 digits with a position-weighted checksum:
//!
//! ```text
//! decimal = d[9] + 2
//! sum     = Σ (d[i] + decimal) * w[i]   for i in 0..10
//!           with w = [29, 27, 23, 19, 17, 29, 27, 23, 19, 17]
//! r       = sum mod 11
//! check   = r          if r < 2
//!         = 11 - r     otherwise
//! valid iff d[10] == check
//! ```
//!
//! Trivially repunit IDs (`00000000000`, `11111111111`, …) are rejected even
//! though some satisfy the checksum.

use super::extract_digits;

const WEIGHTS: [u32; 10] = [29, 27, 23, 19, 17, 29, 27, 23, 19, 17];

/// Returns `true` if `id` is a structurally valid Iranian legal ID.
///
/// Accepts Persian or Latin digits and ignores spaces / dashes.
///
/// ```
/// use parsitext::validators::legal_id;
///
/// assert!(legal_id::validate("10380284790"));     // sample valid
/// assert!(legal_id::validate("۱۰۳۸۰۲۸۴۷۹۰"));      // Persian digits
/// assert!(!legal_id::validate("12345678901"));    // bad checksum
/// assert!(!legal_id::validate("00000000000"));    // repunit
/// assert!(!legal_id::validate("123"));            // too short
/// ```
#[must_use]
pub fn validate(id: &str) -> bool {
    let digits = extract_digits(id);
    if digits.len() != 11 {
        return false;
    }
    if digits.chars().all(|c| c == digits.as_bytes()[0] as char) {
        return false;
    }

    let bytes = digits.as_bytes();
    let decimal = (bytes[9] - b'0') as u32 + 2;
    let mut sum: u32 = 0;
    for (i, b) in bytes.iter().take(10).enumerate() {
        sum += ((*b - b'0') as u32 + decimal) * WEIGHTS[i];
    }
    let check = (bytes[10] - b'0') as u32;
    let r = sum % 11;
    let expected = if r < 2 { r } else { 11 - r };
    check == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_known_ids() {
        for id in [
            "10380284790",
            "10103367860",
            "10861974687",
            "14002892636",
            "10086685006",
        ] {
            assert!(validate(id), "{id} should validate");
        }
    }

    #[test]
    fn rejects_bad_checksum() {
        assert!(!validate("12345678901"));
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(!validate(""));
        assert!(!validate("1234567890")); // 10 digits — looks like a personal ID
        assert!(!validate("123456789012"));
    }

    #[test]
    fn rejects_repunit() {
        for d in '0'..='9' {
            let id: String = std::iter::repeat_n(d, 11).collect();
            assert!(!validate(&id), "{id} should be rejected");
        }
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate("۱۰۳۸۰۲۸۴۷۹۰"));
    }

    #[test]
    fn accepts_separators() {
        assert!(validate("103-8028-4790"));
        assert!(validate("103 8028 4790"));
    }
}
