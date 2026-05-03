//! Iranian vehicle licence-plate validation.
//!
//! Format: `XX [letter] XXX YY` where:
//! - `XX`     — 2-digit serial (10–99)
//! - letter   — one Persian letter from a fixed alphabet
//! - `XXX`    — 3-digit serial (100–999)
//! - `YY`     — 2-digit province code (11–99)
//!
//! Separators (space, dash, dot) are tolerated between groups; Persian and
//! Latin digits are both accepted.

use std::fmt;

use super::extract_digits;

/// Letters allowed on civilian Iranian plates.
const PLATE_LETTERS: &str = "ابپتثجچحدذرزژسشصضطظعغفقکگلمنوهی";

/// A parsed vehicle plate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plate {
    /// First 2-digit group.
    pub first: u32,
    /// Persian letter character.
    pub letter: char,
    /// 3-digit group.
    pub second: u32,
    /// 2-digit province code (11–99).
    pub province: u32,
}

impl fmt::Display for Plate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}{}{:03}-{:02}",
            self.first, self.letter, self.second, self.province
        )
    }
}

/// Returns `true` if `plate` parses as a structurally valid Iranian plate.
///
/// ```
/// use parsitext::validators::car_plate;
///
/// assert!(car_plate::validate("12 ب 345 - 67"));
/// assert!(car_plate::validate("۱۲ب۳۴۵۶۷"));
/// assert!(!car_plate::validate("12 X 345 67"));    // non-Persian letter
/// assert!(!car_plate::validate("12 ب 345"));       // missing province
/// ```
#[must_use]
pub fn validate(plate: &str) -> bool {
    parse(plate).is_some()
}

/// Parse `plate` into a structured [`Plate`] value.
///
/// ```
/// use parsitext::validators::car_plate::parse;
///
/// let p = parse("12 ب 345 - 67").unwrap();
/// assert_eq!(p.first, 12);
/// assert_eq!(p.letter, 'ب');
/// assert_eq!(p.second, 345);
/// assert_eq!(p.province, 67);
/// ```
#[must_use]
pub fn parse(plate: &str) -> Option<Plate> {
    let mut letter: Option<char> = None;
    let mut digit_groups: Vec<String> = vec![String::new()];

    for c in plate.chars() {
        if c.is_whitespace() || matches!(c, '-' | '.' | '_') {
            if !digit_groups.last().unwrap().is_empty() {
                digit_groups.push(String::new());
            }
            continue;
        }
        if c.is_ascii_digit() || super::persian_or_arabic_digit_to_ascii(c).is_some() {
            let d = if c.is_ascii_digit() {
                c
            } else {
                super::persian_or_arabic_digit_to_ascii(c).unwrap()
            };
            digit_groups.last_mut().unwrap().push(d);
        } else if PLATE_LETTERS.contains(c) {
            if letter.is_some() {
                return None;
            }
            letter = Some(c);
            if !digit_groups.last().unwrap().is_empty() {
                digit_groups.push(String::new());
            }
        } else {
            return None;
        }
    }

    let letter = letter?;
    let digit_groups: Vec<String> = digit_groups.into_iter().filter(|g| !g.is_empty()).collect();

    let (first, second, province) = match digit_groups.as_slice() {
        [a, b, c] => (a.clone(), b.clone(), c.clone()),
        // Letter splits all digits into two groups: prefix (2) + (3+2 fused = 5).
        [a, fused] if a.len() == 2 && fused.len() == 5 => {
            (a.clone(), fused[..3].to_owned(), fused[3..].to_owned())
        }
        // Wholly fused single digit-string: 2 + 3 + 2 = 7 digits.
        [s] if s.len() == 7 => (s[..2].to_owned(), s[2..5].to_owned(), s[5..].to_owned()),
        _ => return None,
    };

    if first.len() != 2 || second.len() != 3 || province.len() != 2 {
        return None;
    }

    let first: u32 = first.parse().ok()?;
    let second: u32 = second.parse().ok()?;
    let province: u32 = province.parse().ok()?;

    if !(10..=99).contains(&first)
        || !(100..=999).contains(&second)
        || !(11..=99).contains(&province)
    {
        return None;
    }

    Some(Plate {
        first,
        letter,
        second,
        province,
    })
}

#[allow(dead_code)]
fn extract_digits_for_plate(s: &str) -> String {
    extract_digits(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spaced_plate() {
        let p = parse("12 ب 345 - 67").unwrap();
        assert_eq!(p.first, 12);
        assert_eq!(p.letter, 'ب');
        assert_eq!(p.second, 345);
        assert_eq!(p.province, 67);
    }

    #[test]
    fn parses_persian_digit_plate() {
        let p = parse("۱۲ب۳۴۵۶۷").unwrap();
        assert_eq!((p.first, p.second, p.province), (12, 345, 67));
    }

    #[test]
    fn display_round_trip() {
        let p = parse("12 ب 345 - 67").unwrap();
        assert_eq!(p.to_string(), "12ب345-67");
    }

    #[test]
    fn rejects_non_persian_letter() {
        assert!(parse("12 X 345 67").is_none());
    }

    #[test]
    fn rejects_missing_province() {
        assert!(parse("12 ب 345").is_none());
    }

    #[test]
    fn rejects_short_serial() {
        assert!(parse("1 ب 345 67").is_none());
    }
}
