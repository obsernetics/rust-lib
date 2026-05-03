//! Persian number ↔ word conversion and formatting.
//!
//! - [`to_words`]   — `1234`         → `"یک هزار و دویست و سی و چهار"`
//! - [`from_words`] — `"دو میلیون"`   → `Some(2_000_000)`
//! - [`format()`]   — `1_234_567`    → `"۱،۲۳۴،۵۶۷"`
//! - [`ordinal`]    — `5`            → `"پنجم"`

mod from_words;
mod to_words;

pub use from_words::from_words;
pub use to_words::{ordinal, to_words};

/// Format an integer with Persian thousand separators (`،`) and Persian
/// digits.
///
/// ```
/// use parsitext::numbers;
///
/// assert_eq!(numbers::format(1_234_567), "۱،۲۳۴،۵۶۷");
/// assert_eq!(numbers::format(-1_000),     "-۱،۰۰۰");
/// assert_eq!(numbers::format(42),         "۴۲");
/// ```
#[must_use]
pub fn format(n: i64) -> String {
    let negative = n < 0;
    let abs = if n == i64::MIN {
        // i64::MIN.unsigned_abs() handles overflow correctly.
        i64::MIN.unsigned_abs()
    } else {
        n.unsigned_abs()
    };

    let raw = abs.to_string();
    let len = raw.len();
    let mut out = String::with_capacity(len + len / 3 + 1);
    if negative {
        out.push('-');
    }
    for (i, c) in raw.chars().enumerate() {
        let idx_from_end = len - i;
        if i > 0 && idx_from_end % 3 == 0 {
            out.push('،'); // U+060C ARABIC COMMA
        }
        out.push(latin_to_persian_digit(c));
    }
    out
}

#[inline]
fn latin_to_persian_digit(c: char) -> char {
    if c.is_ascii_digit() {
        char::from_u32(c as u32 - b'0' as u32 + 0x06F0).unwrap_or(c)
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_separators() {
        assert_eq!(format(1_234_567), "۱،۲۳۴،۵۶۷");
        assert_eq!(format(0), "۰");
        assert_eq!(format(999), "۹۹۹");
        assert_eq!(format(1000), "۱،۰۰۰");
    }

    #[test]
    fn formats_negative() {
        assert_eq!(format(-1_000), "-۱،۰۰۰");
    }

    #[test]
    fn formats_i64_min() {
        // Should not panic on i64::MIN edge case.
        let s = format(i64::MIN);
        assert!(s.starts_with('-'));
    }
}
