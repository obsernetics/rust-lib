//! Iranian postal code (کد پستی) validation.
//!
//! Iranian postal codes are exactly **10 digits**.  Sanity rules:
//! - The first digit must not be `0` (the first digit encodes the province).
//! - No more than three identical digits in a row.
//!
//! These rules eliminate obvious non-codes (`0000000000`, `1111111111`) while
//! accepting every in-service postal code.

use super::extract_digits;

/// Returns `true` if `code` is a structurally valid Iranian postal code.
///
/// Accepts Persian or Latin digits and ignores common separators
/// (`-`, space).
///
/// ```
/// use parsitext::validators::postal_code;
///
/// assert!(postal_code::validate("1969833114"));
/// assert!(postal_code::validate("19698-33114"));
/// assert!(postal_code::validate("۱۹۶۹۸۳۳۱۱۴"));
/// assert!(!postal_code::validate("0123456789"));     // leading zero
/// assert!(!postal_code::validate("12342567890"));    // 11 digits
/// assert!(!postal_code::validate("1111111111"));     // repunit
/// ```
#[must_use]
pub fn validate(code: &str) -> bool {
    let digits = extract_digits(code);
    if digits.len() != 10 {
        return false;
    }
    let bytes = digits.as_bytes();
    if bytes[0] == b'0' {
        return false;
    }
    // Reject runs of 4+ identical digits.
    let mut run = 1;
    for i in 1..bytes.len() {
        if bytes[i] == bytes[i - 1] {
            run += 1;
            if run >= 4 {
                return false;
            }
        } else {
            run = 1;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_examples() {
        assert!(validate("1969833114"));
        assert!(validate("9876543219"));
    }

    #[test]
    fn rejects_leading_zero() {
        assert!(!validate("0123456789"));
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(!validate("123"));
        assert!(!validate("12345678901"));
    }

    #[test]
    fn rejects_long_run() {
        assert!(!validate("1111111111"));
        assert!(!validate("1232222345"));
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate("۱۹۶۹۸۳۳۱۱۴"));
    }
}
