//! Parse a Jalali date from a string, accepting Persian or Arabic digits.

use std::str::FromStr;

use crate::{digits, Error, JalaliDate};

impl JalaliDate {
    /// Parse a date written as `YYYY{sep}MM{sep}DD`.
    ///
    /// Persian (۰-۹) and Eastern-Arabic (٠-٩) digits are accepted.
    pub fn parse(s: &str, sep: char) -> Result<Self, Error> {
        let normalized = digits::to_latin(s);
        let parts: Vec<&str> = normalized.split(sep).collect();
        if parts.len() != 3 {
            return Err(invalid(s));
        }
        let y: i32 = parts[0].trim().parse().map_err(|_| invalid(s))?;
        let m: u32 = parts[1].trim().parse().map_err(|_| invalid(s))?;
        let d: u32 = parts[2].trim().parse().map_err(|_| invalid(s))?;
        JalaliDate::new(y, m, d)
    }
}

fn invalid(s: &str) -> Error {
    Error::InvalidJalaliInput(s.to_string())
}

impl FromStr for JalaliDate {
    type Err = Error;
    /// Accepts `YYYY/MM/DD` or `YYYY-MM-DD` (with Persian/Arabic digits OK).
    fn from_str(s: &str) -> Result<Self, Error> {
        let normalized = digits::to_latin(s);
        let sep = if normalized.contains('/') {
            '/'
        } else if normalized.contains('-') {
            '-'
        } else {
            return Err(invalid(s));
        };
        JalaliDate::parse(s, sep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_with_slash() {
        let j: JalaliDate = "1403/01/01".parse().unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1403, 1, 1));
    }

    #[test]
    fn parse_with_dash() {
        let j: JalaliDate = "1404-12-29".parse().unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1404, 12, 29));
    }

    #[test]
    fn parse_persian_digits() {
        let j = JalaliDate::parse("۱۴۰۳/۰۱/۰۱", '/').unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1403, 1, 1));
    }

    #[test]
    fn parse_arabic_digits() {
        let j: JalaliDate = "١٤٠٤-١٢-٢٩".parse().unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1404, 12, 29));
    }

    #[test]
    fn parse_invalid_separator_count() {
        assert!("1403/01".parse::<JalaliDate>().is_err());
        assert!("not a date".parse::<JalaliDate>().is_err());
    }

    #[test]
    fn parse_invalid_date_value() {
        assert!("1404/13/01".parse::<JalaliDate>().is_err());
        assert!("1404/12/30".parse::<JalaliDate>().is_err()); // 1404 not leap
    }
}
