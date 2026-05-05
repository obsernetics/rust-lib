//! Iranian bank card number validation and issuer lookup.
//!
//! Card numbers are 16 digits validated with the **Luhn algorithm** (the
//! same checksum used for Visa/Mastercard).  The first 6 digits (BIN) name
//! the issuing bank.

use super::extract_digits;

/// Returns `true` if `card` is a structurally valid 16-digit card number
/// whose Luhn checksum verifies.
///
/// Accepts Persian or Latin digits and ignores spaces / dashes.
///
/// ```
/// use parsitext::validators::bank_card;
///
/// assert!(bank_card::validate("6037990000000006"));   // Bank Melli sample
/// assert!(bank_card::validate("6037-9900-0000-0006"));
/// assert!(bank_card::validate("۶۰۳۷۹۹۰۰۰۰۰۰۰۰۰۶"));
/// assert!(!bank_card::validate("6037990000000007")); // bad Luhn
/// assert!(!bank_card::validate("12345"));            // too short
/// ```
#[must_use]
pub fn validate(card: &str) -> bool {
    let digits = extract_digits(card);
    if digits.len() != 16 {
        return false;
    }
    luhn_ok(&digits)
}

/// English name of the issuing bank, e.g. `"Bank Melli Iran"`.
///
/// Returns `None` if the input is not a valid 16-digit card or the BIN is not
/// in the lookup table.
#[must_use]
pub fn bank(card: &str) -> Option<&'static str> {
    bank_entry(card).map(|(_, en, _)| *en)
}

/// Persian name of the issuing bank, e.g. `"بانک ملی ایران"`.
#[must_use]
pub fn bank_persian(card: &str) -> Option<&'static str> {
    bank_entry(card).map(|(_, _, fa)| *fa)
}

// ── internals ─────────────────────────────────────────────────────────────────

fn luhn_ok(digits: &str) -> bool {
    let bytes = digits.as_bytes();
    let mut sum: u32 = 0;
    for (i, b) in bytes.iter().rev().enumerate() {
        let d = (*b - b'0') as u32;
        if i % 2 == 1 {
            let doubled = d * 2;
            sum += if doubled > 9 { doubled - 9 } else { doubled };
        } else {
            sum += d;
        }
    }
    sum.is_multiple_of(10)
}

fn bank_entry(card: &str) -> Option<&'static (&'static str, &'static str, &'static str)> {
    let digits = extract_digits(card);
    if digits.len() != 16 || !luhn_ok(&digits) {
        return None;
    }
    let bin = &digits[..6];
    super::banks::CARD_BINS.iter().find(|(b, _, _)| *b == bin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_valid_card() {
        assert!(validate("6037990000000006"));
    }

    #[test]
    fn rejects_bad_luhn() {
        assert!(!validate("6037997599999990"));
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(!validate("12345"));
        assert!(!validate("60379900000000060"));
    }

    #[test]
    fn accepts_separators() {
        assert!(validate("6037-9900-0000-0006"));
        assert!(validate("6037 9900 0000 0006"));
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate("۶۰۳۷۹۹۰۰۰۰۰۰۰۰۰۶"));
    }

    #[test]
    fn bank_lookup_returns_melli() {
        // BIN 603799 = Bank Melli
        assert_eq!(bank("6037990000000006"), Some("Bank Melli Iran"));
        assert_eq!(bank_persian("6037990000000006"), Some("بانک ملی ایران"));
    }

    #[test]
    fn bank_lookup_invalid_card() {
        assert_eq!(bank("6037997599999990"), None);
    }
}
