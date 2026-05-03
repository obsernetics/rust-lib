//! Iranian mobile phone number validation, canonicalisation, and operator
//! detection.
//!
//! Iranian mobile numbers are 11 digits beginning with `09` (or equivalently
//! `+989` / `00989`).  The first four digits identify the operator family;
//! the remaining seven are the subscriber number.

use std::fmt;

use super::extract_digits;

/// Mobile network operators in Iran.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Operator {
    /// MCI / Hamrah-e Aval — همراه اول.
    MCI,
    /// MTN Irancell — ایرانسل.
    Irancell,
    /// RighTel — رایتل.
    RighTel,
    /// Shatel Mobile — شاتل موبایل.
    ShatelMobile,
    /// Anarestan / Aptel — آپتل.
    Aptel,
    /// Other licensed MVNO or unknown sub-prefix.
    Other,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operator::MCI => "MCI",
            Operator::Irancell => "Irancell",
            Operator::RighTel => "RighTel",
            Operator::ShatelMobile => "Shatel Mobile",
            Operator::Aptel => "Aptel",
            Operator::Other => "Other",
        };
        f.write_str(s)
    }
}

impl Operator {
    /// Persian name of the operator.
    #[must_use]
    pub fn persian_name(&self) -> &'static str {
        match self {
            Operator::MCI => "همراه اول",
            Operator::Irancell => "ایرانسل",
            Operator::RighTel => "رایتل",
            Operator::ShatelMobile => "شاتل موبایل",
            Operator::Aptel => "آپتل",
            Operator::Other => "نامشخص",
        }
    }
}

/// Returns `true` if `phone` is a structurally valid Iranian mobile number.
///
/// Accepts `09…`, `+989…`, `00989…`, with or without spaces/dashes and in
/// any digit script.
///
/// ```
/// use parsitext::validators::phone;
///
/// assert!(phone::validate("09121234567"));
/// assert!(phone::validate("+989121234567"));
/// assert!(phone::validate("0098 912 123 4567"));
/// assert!(phone::validate("۰۹۱۲۱۲۳۴۵۶۷"));
/// assert!(!phone::validate("0812345678"));       // wrong leading digit
/// assert!(!phone::validate("0912123456"));       // 10 digits
/// ```
#[must_use]
pub fn validate(phone: &str) -> bool {
    canonicalize(phone).is_some()
}

/// Canonicalise to `09XXXXXXXXX` (11 ASCII digits) or return `None` if the
/// input is not a valid Iranian mobile number.
///
/// ```
/// use parsitext::validators::phone;
///
/// assert_eq!(phone::canonicalize("+989121234567"), Some("09121234567".into()));
/// assert_eq!(phone::canonicalize("۰۹۱۲۱۲۳۴۵۶۷"),  Some("09121234567".into()));
/// assert_eq!(phone::canonicalize("0812345678"),    None);
/// ```
#[must_use]
pub fn canonicalize(phone: &str) -> Option<String> {
    let digits = extract_digits(phone);
    let body = if digits.len() == 14 && digits.starts_with("0098") {
        &digits[4..] // drop "0098", keep "9..."
    } else if digits.len() == 12 && digits.starts_with("98") {
        &digits[2..] // drop "98", keep "9..."
    } else if digits.len() == 11 && digits.starts_with("09") {
        return Some(digits);
    } else {
        return None;
    };
    // body now has the trunk part starting with "9..."; prepend "0".
    if body.len() == 10 && body.starts_with('9') {
        Some(format!("0{body}"))
    } else {
        None
    }
}

/// Detect the mobile operator from a phone number.
///
/// Returns `None` if the input is not a valid Iranian mobile number.
///
/// ```
/// use parsitext::validators::{phone, Operator};
///
/// assert_eq!(phone::operator("09121234567"), Some(Operator::MCI));
/// assert_eq!(phone::operator("09301234567"), Some(Operator::Irancell));
/// assert_eq!(phone::operator("09221234567"), Some(Operator::RighTel));
/// ```
#[must_use]
pub fn operator(phone: &str) -> Option<Operator> {
    let canon = canonicalize(phone)?;
    let prefix = &canon[..4];
    Some(match prefix {
        // MCI / Hamrah-e Aval
        "0910" | "0911" | "0912" | "0913" | "0914" | "0915" | "0916" | "0917" | "0918" | "0919" => {
            Operator::MCI
        }
        "0991" | "0992" | "0994" | "0995" | "0996" | "0999" => Operator::MCI,
        // RighTel
        "0920" | "0921" | "0922" => Operator::RighTel,
        // Irancell
        "0930" | "0933" | "0935" | "0936" | "0937" | "0938" | "0939" => Operator::Irancell,
        "0901" | "0902" | "0903" | "0904" | "0905" | "0941" => Operator::Irancell,
        // Shatel Mobile
        "0998" => Operator::ShatelMobile,
        // Aptel
        "0993" => Operator::Aptel,
        _ => Operator::Other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_local_format() {
        assert!(validate("09121234567"));
    }

    #[test]
    fn validates_international_plus() {
        assert!(validate("+989121234567"));
    }

    #[test]
    fn validates_international_00() {
        assert!(validate("00989121234567"));
    }

    #[test]
    fn rejects_wrong_country() {
        assert!(!validate("0812345678"));
    }

    #[test]
    fn rejects_short() {
        assert!(!validate("0912123456"));
    }

    #[test]
    fn canonical_form() {
        assert_eq!(
            canonicalize("+989121234567").as_deref(),
            Some("09121234567")
        );
        assert_eq!(
            canonicalize("00989121234567").as_deref(),
            Some("09121234567")
        );
        assert_eq!(canonicalize("09121234567").as_deref(), Some("09121234567"));
    }

    #[test]
    fn operator_detection() {
        assert_eq!(operator("09121234567"), Some(Operator::MCI));
        assert_eq!(operator("09351234567"), Some(Operator::Irancell));
        assert_eq!(operator("09211234567"), Some(Operator::RighTel));
        assert_eq!(operator("09981234567"), Some(Operator::ShatelMobile));
    }

    #[test]
    fn operator_returns_other_for_unknown_prefix() {
        // 0900 is not currently allocated to a major operator.
        assert_eq!(operator("09001234567"), Some(Operator::Other));
    }

    #[test]
    fn operator_persian_name() {
        assert_eq!(Operator::MCI.persian_name(), "همراه اول");
        assert_eq!(Operator::Irancell.persian_name(), "ایرانسل");
    }
}
