//! Iranian National ID (کد ملی) validation.
//!
//! Algorithm: 10 digits where the last digit is a checksum derived from the
//! first nine.
//!
//! ```text
//! sum    = Σ d[i] * (10 - i)   for i in 0..9
//! r      = sum mod 11
//! check  = r          if r < 2
//!        = 11 - r     otherwise
//! ```
//!
//! Trivial repunit IDs (`0000000000`, `1111111111`, …) are also rejected
//! because they accidentally satisfy the checksum but are never issued.

use super::extract_digits;

/// Returns `true` if `id` is a structurally valid Iranian national ID.
///
/// Accepts input with Persian or Latin digits and tolerates separators
/// (spaces, dashes).  Returns `false` for anything that is not exactly 10
/// digits or whose checksum does not match.
///
/// ```
/// use parsitext::validators::national_id;
///
/// assert!(national_id::validate("0499370899"));       // valid sample ID
/// assert!(national_id::validate("۰۴۹۹۳۷۰۸۹۹"));        // Persian digits
/// assert!(!national_id::validate("1234567890"));      // bad checksum
/// assert!(!national_id::validate("0000000000"));      // repunit
/// assert!(!national_id::validate("12345"));           // too short
/// ```
#[must_use]
pub fn validate(id: &str) -> bool {
    let digits = extract_digits(id);
    if digits.len() != 10 {
        return false;
    }
    if digits.chars().all(|c| c == digits.as_bytes()[0] as char) {
        return false;
    }

    let bytes = digits.as_bytes();
    let mut sum: u32 = 0;
    for (i, b) in bytes.iter().take(9).enumerate() {
        sum += (b - b'0') as u32 * (10 - i as u32);
    }
    let check = (bytes[9] - b'0') as u32;
    let r = sum % 11;
    if r < 2 {
        check == r
    } else {
        check == 11 - r
    }
}

/// Compute the expected check digit (`0..=9`) for the first nine digits of an
/// ID.  Returns `None` if `id` does not contain at least nine digits.
#[must_use]
pub fn expected_check_digit(id: &str) -> Option<u32> {
    let digits = extract_digits(id);
    if digits.len() < 9 {
        return None;
    }
    let bytes = digits.as_bytes();
    let mut sum: u32 = 0;
    for (i, b) in bytes.iter().take(9).enumerate() {
        sum += (b - b'0') as u32 * (10 - i as u32);
    }
    let r = sum % 11;
    Some(if r < 2 { r } else { 11 - r })
}

/// Issuing-office prefix (first three digits) of a national ID.
///
/// Returns `None` if `id` does not contain at least three digits.  The
/// prefix is the **issuing serial code** assigned to a city / civil-registry
/// office; it can be paired with an external lookup table to recover the
/// place of issuance.
///
/// We deliberately do **not** ship a built-in prefix → city table: the
/// authoritative dataset is sizeable, evolves as new offices open, and
/// returning a stale or partial answer is worse than returning the raw
/// prefix.
///
/// ```
/// use parsitext::validators::national_id;
///
/// assert_eq!(national_id::issuance_prefix("0499370899").as_deref(), Some("049"));
/// assert_eq!(national_id::issuance_prefix("12"), None);
/// ```
#[must_use]
pub fn issuance_prefix(id: &str) -> Option<String> {
    let digits = extract_digits(id);
    if digits.len() < 3 {
        return None;
    }
    Some(digits[..3].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_known_ids() {
        // Sample IDs whose checksums verify against the published algorithm.
        for id in ["0499370899", "0084575948", "1729374654"] {
            assert!(validate(id), "{id} should be valid");
        }
    }

    #[test]
    fn rejects_bad_checksum() {
        // Bad check digit (correct one would be 9).
        assert!(!validate("0499370891"));
        // Bad check digit (correct one would be 4).
        assert!(!validate("1729374650"));
    }

    #[test]
    fn rejects_repunit() {
        for d in '0'..='9' {
            let id: String = std::iter::repeat_n(d, 10).collect();
            assert!(!validate(&id), "{id} should be rejected");
        }
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(!validate("12345"));
        assert!(!validate("12345678901"));
        assert!(!validate(""));
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate("۰۴۹۹۳۷۰۸۹۹"));
    }

    #[test]
    fn ignores_separators() {
        assert!(validate("049-937-0899"));
        assert!(validate("049 937 0899"));
    }

    #[test]
    fn check_digit() {
        // First 9 digits of a known valid ID -> check digit 9
        assert_eq!(expected_check_digit("049937089"), Some(9));
    }

    #[test]
    fn prefix_extraction() {
        assert_eq!(issuance_prefix("0499370899").as_deref(), Some("049"));
        assert_eq!(issuance_prefix("12"), None);
    }
}
