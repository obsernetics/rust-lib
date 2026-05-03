//! Optional integration with the [`jalali-calendar`](https://docs.rs/jalali-calendar)
//! crate.
//!
//! Enabled with the `jalali` Cargo feature.  Provides date validation and
//! parsing helpers that turn the **regex-detected** date entities into
//! structured [`JalaliDate`] values.
//!
//! ```ignore
//! # // Requires the `jalali` feature.
//! use parsitext::Parsitext;
//!
//! let pt = Parsitext::default();
//! let jd = pt.parse_jalali_date("1402/03/15").unwrap();
//! assert_eq!((jd.year(), jd.month(), jd.day()), (1402, 3, 15));
//! ```

use jalali_calendar::JalaliDate;

const PERSIAN_MONTHS: &[(&str, u32)] = &[
    ("فروردین", 1),
    ("اردیبهشت", 2),
    ("خرداد", 3),
    ("تیر", 4),
    ("مرداد", 5),
    ("شهریور", 6),
    ("مهر", 7),
    ("آبان", 8),
    ("آذر", 9),
    ("دی", 10),
    ("بهمن", 11),
    ("اسفند", 12),
];

/// Parse a Jalali date from a numeric string (`1402/03/15`, `۱۴۰۲-۰۳-۱۵`)
/// or a textual one (`۱۵ تیر ۱۴۰۲`).  Persian and Latin digits are accepted.
///
/// Returns `None` if the parts can't be extracted **or** if they don't
/// represent a real Jalali date (e.g. day 30 of Esfand in a non-leap year).
#[must_use]
pub fn parse(text: &str) -> Option<JalaliDate> {
    parse_textual(text).or_else(|| parse_numeric(text))
}

fn parse_numeric(text: &str) -> Option<JalaliDate> {
    let normalized: String = text
        .chars()
        .map(|c| {
            let cp = c as u32;
            if (0x06F0..=0x06F9).contains(&cp) {
                char::from_u32(cp - 0x06F0 + b'0' as u32).unwrap_or(c)
            } else if (0x0660..=0x0669).contains(&cp) {
                char::from_u32(cp - 0x0660 + b'0' as u32).unwrap_or(c)
            } else {
                c
            }
        })
        .collect();

    for sep in ['/', '-'] {
        if let Ok(d) = JalaliDate::parse(normalized.trim(), sep) {
            return Some(d);
        }
    }
    None
}

fn parse_textual(text: &str) -> Option<JalaliDate> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    let day_str = digits_to_latin(parts[0]);
    let month_name = parts[1];
    let year_str = digits_to_latin(parts[2]);

    let day: u32 = day_str.parse().ok()?;
    let year: i32 = year_str.parse().ok()?;
    let month: u32 = PERSIAN_MONTHS
        .iter()
        .find(|(name, _)| *name == month_name)
        .map(|(_, n)| *n)?;

    JalaliDate::new(year, month, day).ok()
}

fn digits_to_latin(s: &str) -> String {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (0x06F0..=0x06F9).contains(&cp) {
                char::from_u32(cp - 0x06F0 + b'0' as u32).unwrap_or(c)
            } else if (0x0660..=0x0669).contains(&cp) {
                char::from_u32(cp - 0x0660 + b'0' as u32).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numeric_latin() {
        let d = parse("1402/03/15").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1402, 3, 15));
    }

    #[test]
    fn parse_numeric_persian() {
        let d = parse("۱۴۰۲/۰۳/۱۵").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1402, 3, 15));
    }

    #[test]
    fn parse_dash_separator() {
        let d = parse("1403-01-01").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1403, 1, 1));
    }

    #[test]
    fn parse_textual_persian() {
        let d = parse("۱۵ تیر ۱۴۰۲").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1402, 4, 15));
    }

    #[test]
    fn rejects_invalid_calendar_date() {
        // Esfand 30 only exists in leap years.  1404 is not leap.
        assert!(parse("1404/12/30").is_none());
        assert!(parse("1403/12/30").is_some()); // 1403 IS leap
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("not a date").is_none());
        assert!(parse("99/99/99").is_none());
    }
}
