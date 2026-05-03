//! Structured parsing of Persian money expressions.
//!
//! Recognises both numeric (`۲ میلیون تومان`, `1,500,000 ریال`) and written
//! (`دو میلیون و پانصد هزار تومان`) amounts and converts them to a structured
//! [`MoneyAmount`] with the currency unit (`Toman` or `Rial`).
//!
//! 1 Toman = 10 Rials by Iranian convention.

use std::fmt;

use crate::{
    numbers::{format as format_persian, from_words},
    validators::extract_digits,
};

/// Iranian currency units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MoneyUnit {
    /// تومان — 10 Rials.
    Toman,
    /// ریال — base currency unit.
    Rial,
}

impl fmt::Display for MoneyUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            MoneyUnit::Toman => "Toman",
            MoneyUnit::Rial => "Rial",
        })
    }
}

/// A parsed monetary value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoneyAmount {
    /// The amount in the original unit (`unit`).
    pub value: i64,
    /// The currency unit named in the input.
    pub unit: MoneyUnit,
    /// The original substring this was parsed from.
    pub raw: String,
}

impl MoneyAmount {
    /// Value expressed in Rials (multiplies by 10 if the original unit was Toman).
    #[must_use]
    pub fn as_rials(&self) -> i64 {
        match self.unit {
            MoneyUnit::Toman => self.value.saturating_mul(10),
            MoneyUnit::Rial => self.value,
        }
    }

    /// Value expressed in Tomans (divides by 10 if the original unit was Rial,
    /// truncated toward zero).
    #[must_use]
    pub fn as_tomans(&self) -> i64 {
        match self.unit {
            MoneyUnit::Toman => self.value,
            MoneyUnit::Rial => self.value / 10,
        }
    }
}

impl fmt::Display for MoneyAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", format_persian(self.value), self.unit)
    }
}

/// Format an integer amount with the given currency unit using Persian
/// thousand separators.
///
/// ```
/// use parsitext::money::{format, MoneyUnit};
///
/// assert_eq!(format(2_500_000, MoneyUnit::Toman), "۲،۵۰۰،۰۰۰ تومان");
/// assert_eq!(format(500,        MoneyUnit::Rial),  "۵۰۰ ریال");
/// ```
#[must_use]
pub fn format(value: i64, unit: MoneyUnit) -> String {
    let unit_word = match unit {
        MoneyUnit::Toman => "تومان",
        MoneyUnit::Rial => "ریال",
    };
    format!("{} {}", format_persian(value), unit_word)
}

/// Parse a Persian monetary expression.
///
/// Recognised forms:
/// - Numeric with multiplier: `"۵۰۰ هزار تومان"`, `"۱.۵ میلیون تومن"`, `"۲ میلیارد ریال"`
/// - Numeric without multiplier: `"1500000 ریال"`, `"۲۰۰،۰۰۰ تومان"`
/// - Written words: `"دو میلیون و پانصد هزار تومان"`
///
/// Returns `None` if the unit (تومان / تومن / ریال) is missing or the
/// numeric part cannot be parsed.
///
/// ```
/// use parsitext::money::{parse, MoneyUnit};
///
/// let m = parse("۵۰۰ هزار تومان").unwrap();
/// assert_eq!(m.value, 500_000);
/// assert_eq!(m.unit,  MoneyUnit::Toman);
/// assert_eq!(m.as_rials(), 5_000_000);
///
/// let m = parse("دو میلیون ریال").unwrap();
/// assert_eq!(m.value, 2_000_000);
/// assert_eq!(m.unit,  MoneyUnit::Rial);
/// ```
#[must_use]
pub fn parse(text: &str) -> Option<MoneyAmount> {
    let raw = text.trim().to_owned();
    let (body, unit) = strip_unit(&raw)?;
    let value = parse_value(body.trim())?;
    Some(MoneyAmount { value, unit, raw })
}

fn strip_unit(s: &str) -> Option<(String, MoneyUnit)> {
    for (suffix, unit) in [
        ("تومان", MoneyUnit::Toman),
        ("تومن", MoneyUnit::Toman),
        ("ریال", MoneyUnit::Rial),
    ] {
        if let Some(idx) = s.rfind(suffix) {
            let before = &s[..idx];
            return Some((before.to_owned(), unit));
        }
    }
    None
}

fn parse_value(body: &str) -> Option<i64> {
    let body = body.trim();
    if body.is_empty() {
        return None;
    }

    // Detect a thousand/million/billion multiplier word at the end.
    let (numeric_part, multiplier) = {
        let words: Vec<&str> = body.split_whitespace().collect();
        if let Some(last) = words.last() {
            let m = match *last {
                "هزار" => Some(1_000i64),
                "میلیون" => Some(1_000_000),
                "میلیارد" => Some(1_000_000_000),
                _ => None,
            };
            if let Some(mult) = m {
                let prefix = words[..words.len() - 1].join(" ");
                (prefix, mult)
            } else {
                (body.to_owned(), 1)
            }
        } else {
            (body.to_owned(), 1)
        }
    };

    let numeric_str = numeric_part.trim();
    if numeric_str.is_empty() {
        // e.g. just "هزار" → 1 * mult
        return Some(multiplier);
    }

    // Try digit parse first (handles "۱۵۰۰", "1,500", "1.5").
    if let Some(n) = parse_digit_value(numeric_str, multiplier) {
        return Some(n);
    }

    // Fall back to spelled-out words.  Strip a trailing multiplier from `body`
    // again in case the words form already contained "هزار" etc.
    if let Some(n) = from_words(body) {
        return Some(n);
    }
    if let Some(n) = from_words(numeric_str) {
        return n.checked_mul(multiplier);
    }
    None
}

fn parse_digit_value(s: &str, multiplier: i64) -> Option<i64> {
    let cleaned = extract_digits(s);
    if cleaned.is_empty() {
        // Maybe a fractional like "۱.۵"
        return parse_fractional(s, multiplier);
    }
    if !s.contains('.') && !s.contains('٫') {
        return cleaned.parse::<i64>().ok()?.checked_mul(multiplier);
    }
    parse_fractional(s, multiplier)
}

fn parse_fractional(s: &str, multiplier: i64) -> Option<i64> {
    // Accept ASCII period or Arabic decimal separator U+066B.
    let sep_idx = s.find(['.', '٫'])?;
    let int_str = extract_digits(&s[..sep_idx]);
    let frac_str = extract_digits(&s[sep_idx + s[sep_idx..].chars().next().unwrap().len_utf8()..]);
    if int_str.is_empty() && frac_str.is_empty() {
        return None;
    }
    let int_part: i64 = if int_str.is_empty() {
        0
    } else {
        int_str.parse().ok()?
    };
    let denom: i64 = 10i64.checked_pow(frac_str.len() as u32)?;
    let frac_part: i64 = if frac_str.is_empty() {
        0
    } else {
        frac_str.parse().ok()?
    };
    let value = int_part
        .checked_mul(multiplier)?
        .checked_add(frac_part.checked_mul(multiplier)? / denom)?;
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numeric_with_multiplier() {
        let m = parse("۵۰۰ هزار تومان").unwrap();
        assert_eq!(m.value, 500_000);
        assert_eq!(m.unit, MoneyUnit::Toman);
    }

    #[test]
    fn parse_with_million() {
        let m = parse("۲ میلیون تومان").unwrap();
        assert_eq!(m.value, 2_000_000);
    }

    #[test]
    fn parse_with_decimal_million() {
        let m = parse("۱.۵ میلیون تومان").unwrap();
        assert_eq!(m.value, 1_500_000);
    }

    #[test]
    fn parse_rial() {
        let m = parse("200 ریال").unwrap();
        assert_eq!(m.value, 200);
        assert_eq!(m.unit, MoneyUnit::Rial);
    }

    #[test]
    fn parse_words() {
        let m = parse("دو میلیون و پانصد هزار تومان").unwrap();
        assert_eq!(m.value, 2_500_000);
    }

    #[test]
    fn rial_to_toman_conversion() {
        let m = parse("100 ریال").unwrap();
        assert_eq!(m.as_tomans(), 10);
        assert_eq!(m.as_rials(), 100);
    }

    #[test]
    fn toman_to_rial_conversion() {
        let m = parse("۱۰ تومان").unwrap();
        assert_eq!(m.as_rials(), 100);
    }

    #[test]
    fn missing_unit_returns_none() {
        assert!(parse("500").is_none());
    }

    #[test]
    fn display_format() {
        let m = parse("۲ میلیون تومان").unwrap();
        assert!(m.to_string().contains("Toman"));
    }
}
