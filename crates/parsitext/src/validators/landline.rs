//! Iranian landline (fixed-line) phone validation and province lookup.
//!
//! Iranian landline numbers are 11 digits beginning with `0XX` where `XX` is
//! a 2-digit area code (`21` for Tehran, `26` for Karaj, `31` for Isfahan, …).
//! The remaining 8 digits are the local subscriber number.

use std::fmt;

use super::extract_digits;

/// Province / major-city area codes.
///
/// The list covers all 31 Iranian provincial centres.  Less common regional
/// codes within a province may not be enumerated; in those cases
/// [`province`] returns `None` even though [`validate`] still returns `true`.
const PROVINCES: &[(&str, Province, &str, &str)] = &[
    ("21", Province::Tehran, "Tehran", "تهران"),
    ("26", Province::Alborz, "Alborz", "البرز"),
    ("11", Province::Mazandaran, "Mazandaran", "مازندران"),
    ("13", Province::Gilan, "Gilan", "گیلان"),
    ("17", Province::Golestan, "Golestan", "گلستان"),
    ("23", Province::Semnan, "Semnan", "سمنان"),
    ("24", Province::Zanjan, "Zanjan", "زنجان"),
    ("25", Province::Qom, "Qom", "قم"),
    ("28", Province::Qazvin, "Qazvin", "قزوین"),
    ("31", Province::Isfahan, "Isfahan", "اصفهان"),
    ("34", Province::Kerman, "Kerman", "کرمان"),
    ("35", Province::Yazd, "Yazd", "یزد"),
    (
        "38",
        Province::ChaharmahalBakht,
        "Chaharmahal & Bakhtiari",
        "چهارمحال و بختیاری",
    ),
    (
        "41",
        Province::EastAzerbaijan,
        "East Azerbaijan",
        "آذربایجان شرقی",
    ),
    (
        "44",
        Province::WestAzerbaijan,
        "West Azerbaijan",
        "آذربایجان غربی",
    ),
    ("45", Province::Ardabil, "Ardabil", "اردبیل"),
    (
        "51",
        Province::RazaviKhorasan,
        "Razavi Khorasan",
        "خراسان رضوی",
    ),
    (
        "54",
        Province::SistanBalu,
        "Sistan & Baluchestan",
        "سیستان و بلوچستان",
    ),
    (
        "56",
        Province::SouthKhorasan,
        "South Khorasan",
        "خراسان جنوبی",
    ),
    (
        "57",
        Province::NorthKhorasan,
        "North Khorasan",
        "خراسان شمالی",
    ),
    (
        "58",
        Province::NorthKhorasan,
        "North Khorasan",
        "خراسان شمالی",
    ),
    ("61", Province::Khuzestan, "Khuzestan", "خوزستان"),
    ("66", Province::Lorestan, "Lorestan", "لرستان"),
    ("71", Province::Fars, "Fars", "فارس"),
    (
        "74",
        Province::KohgiluyehBoyer,
        "Kohgiluyeh & Boyer-Ahmad",
        "کهگیلویه و بویراحمد",
    ),
    ("76", Province::Hormozgan, "Hormozgan", "هرمزگان"),
    ("77", Province::Bushehr, "Bushehr", "بوشهر"),
    ("81", Province::Hamadan, "Hamadan", "همدان"),
    ("83", Province::Kermanshah, "Kermanshah", "کرمانشاه"),
    ("84", Province::Ilam, "Ilam", "ایلام"),
    ("86", Province::Markazi, "Markazi", "مرکزی"),
    ("87", Province::Kurdistan, "Kurdistan", "کردستان"),
];

/// Iranian provinces / major regions reachable as fixed-line area codes.
///
/// Variant names match the standard English province names; use
/// [`Province::persian_name`] for Persian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)] // 31 variants — names are self-describing
pub enum Province {
    Tehran,
    Alborz,
    Mazandaran,
    Gilan,
    Golestan,
    Semnan,
    Zanjan,
    Qom,
    Qazvin,
    Isfahan,
    Kerman,
    Yazd,
    ChaharmahalBakht,
    EastAzerbaijan,
    WestAzerbaijan,
    Ardabil,
    RazaviKhorasan,
    SistanBalu,
    SouthKhorasan,
    NorthKhorasan,
    Khuzestan,
    Lorestan,
    Fars,
    KohgiluyehBoyer,
    Hormozgan,
    Bushehr,
    Hamadan,
    Kermanshah,
    Ilam,
    Markazi,
    Kurdistan,
}

impl fmt::Display for Province {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.english_name().unwrap_or("Unknown"))
    }
}

impl Province {
    /// English name of the province.
    #[must_use]
    pub fn english_name(self) -> Option<&'static str> {
        PROVINCES
            .iter()
            .find(|(_, p, _, _)| *p == self)
            .map(|(_, _, en, _)| *en)
    }

    /// Persian name of the province.
    #[must_use]
    pub fn persian_name(self) -> Option<&'static str> {
        PROVINCES
            .iter()
            .find(|(_, p, _, _)| *p == self)
            .map(|(_, _, _, fa)| *fa)
    }
}

/// Returns `true` if `phone` is a structurally valid Iranian landline (11
/// digits starting with `0` and a non-`9` second digit).
///
/// ```
/// use parsitext::validators::landline;
///
/// assert!(landline::validate("02112345678"));     // Tehran
/// assert!(landline::validate("021-12345678"));    // separator tolerated
/// assert!(landline::validate("۰۲۱۱۲۳۴۵۶۷۸"));     // Persian digits
/// assert!(!landline::validate("09121234567"));    // mobile, not landline
/// assert!(!landline::validate("0211234567"));     // 10 digits
/// ```
#[must_use]
pub fn validate(phone: &str) -> bool {
    canonicalize(phone).is_some()
}

/// Canonical 11-digit form (`0XXYYYYYYYY`) with Latin digits, or `None` if
/// invalid.
#[must_use]
pub fn canonicalize(phone: &str) -> Option<String> {
    let digits = extract_digits(phone);
    if digits.len() != 11 {
        return None;
    }
    if !digits.starts_with('0') {
        return None;
    }
    // Must NOT be a mobile number (which starts with 09).
    if digits.starts_with("09") {
        return None;
    }
    Some(digits)
}

/// Detect the Iranian province from a landline phone's area code.
///
/// Returns `None` if the phone is invalid or its area code is not in the
/// known table.
///
/// ```
/// use parsitext::validators::landline::{province, Province};
///
/// assert_eq!(province("02112345678"), Some(Province::Tehran));
/// assert_eq!(province("03112345678"), Some(Province::Isfahan));
/// ```
#[must_use]
pub fn province(phone: &str) -> Option<Province> {
    let canon = canonicalize(phone)?;
    let area = &canon[1..3];
    PROVINCES
        .iter()
        .find(|(code, _, _, _)| *code == area)
        .map(|(_, p, _, _)| *p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tehran_landline() {
        assert!(validate("02112345678"));
    }

    #[test]
    fn rejects_mobile() {
        assert!(!validate("09121234567"));
    }

    #[test]
    fn rejects_short() {
        assert!(!validate("0211234"));
    }

    #[test]
    fn province_lookup() {
        assert_eq!(province("02112345678"), Some(Province::Tehran));
        assert_eq!(province("03112345678"), Some(Province::Isfahan));
        assert_eq!(province("04112345678"), Some(Province::EastAzerbaijan));
    }

    #[test]
    fn accepts_separators() {
        assert!(validate("021-12345678"));
        assert!(validate("(021) 1234-5678"));
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate("۰۲۱۱۲۳۴۵۶۷۸"));
    }

    #[test]
    fn province_persian_name() {
        assert_eq!(Province::Tehran.persian_name(), Some("تهران"));
    }
}
