//! Iranian utility-bill (قبض) parsing and validation.
//!
//! An Iranian payment slip carries two numbers:
//!
//! | Field | Min length | Last-digit role |
//! |-------|-----------|-----------------|
//! | `bill_id`  (شناسه قبض)    | 6 digits  | check digit over body |
//! | `pay_id`   (شناسه پرداخت) | 6 digits  | last two digits are checks |
//!
//! Both checksums use the same weighted modulo-11 algorithm:
//!
//! ```text
//! sum   = Σ d[i] * ((i mod 6) + 2)   walking from the right
//! r     = sum mod 11
//! check = r          if r < 2
//!       = 11 - r     otherwise
//! ```
//!
//! - The last digit of `bill_id` validates the remaining body.
//! - The second-to-last digit of `pay_id` validates `pay_id[..-2]`.
//! - The last digit of `pay_id` validates `bill_id ++ pay_id[..-1]`.
//!
//! The second-to-last digit of `bill_id` (the digit *before* its check) names
//! the bill **type**: water, electricity, gas, phone, mobile, municipality,
//! tax, or driving fine — see [`BillType`].

use std::fmt;

use super::extract_digits;

/// Iranian bill types as encoded in the second-to-last digit of `bill_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum BillType {
    /// آب — water utility.
    Water,
    /// برق — electricity utility.
    Electricity,
    /// گاز — gas utility.
    Gas,
    /// تلفن ثابت — fixed-line telephone.
    Phone,
    /// تلفن همراه — mobile telephone.
    Mobile,
    /// شهرداری — municipality (rates / fees).
    Municipality,
    /// مالیات — tax authority.
    Tax,
    /// جریمه راهنمایی و رانندگی — driving / traffic fine.
    DrivingFine,
    /// سایر — unrecognised type code.
    Other,
}

impl BillType {
    /// Persian display name.
    #[must_use]
    pub fn persian_name(self) -> &'static str {
        match self {
            BillType::Water => "آب",
            BillType::Electricity => "برق",
            BillType::Gas => "گاز",
            BillType::Phone => "تلفن ثابت",
            BillType::Mobile => "تلفن همراه",
            BillType::Municipality => "شهرداری",
            BillType::Tax => "مالیات",
            BillType::DrivingFine => "جریمه",
            BillType::Other => "سایر",
        }
    }

    fn from_code(code: u32) -> Self {
        match code {
            1 => BillType::Water,
            2 => BillType::Electricity,
            3 => BillType::Gas,
            4 | 9 => BillType::Phone,
            5 => BillType::Mobile,
            6 => BillType::Municipality,
            7 => BillType::Tax,
            8 => BillType::DrivingFine,
            _ => BillType::Other,
        }
    }
}

impl fmt::Display for BillType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BillType::Water => "Water",
            BillType::Electricity => "Electricity",
            BillType::Gas => "Gas",
            BillType::Phone => "Phone",
            BillType::Mobile => "Mobile",
            BillType::Municipality => "Municipality",
            BillType::Tax => "Tax",
            BillType::DrivingFine => "DrivingFine",
            BillType::Other => "Other",
        };
        f.write_str(s)
    }
}

/// Returns `true` if `bill_id` has a valid bill-checksum.
///
/// ```
/// use parsitext::validators::bill;
///
/// assert!(bill::validate_bill_id("7748317801"));
/// assert!(bill::validate_bill_id("۷۷۴۸۳۱۷۸۰۱")); // Persian digits
/// assert!(!bill::validate_bill_id("7748317800")); // wrong checksum
/// assert!(!bill::validate_bill_id("12345"));      // too short
/// ```
#[must_use]
pub fn validate_bill_id(bill_id: &str) -> bool {
    let digits = extract_digits(bill_id);
    if digits.len() < 6 {
        return false;
    }
    let bytes = digits.as_bytes();
    let body = &bytes[..bytes.len() - 1];
    let check = (bytes[bytes.len() - 1] - b'0') as u32;
    weighted_check(body) == check
}

/// Returns `true` if the `pay_id`'s self-consistent check digits agree
/// with `bill_id`.
///
/// ```
/// use parsitext::validators::bill;
///
/// assert!(bill::validate_pay_id("7748317801", "1234986"));
/// assert!(!bill::validate_pay_id("7748317801", "1234987"));
/// assert!(!bill::validate_pay_id("123", "1234986")); // bill_id too short
/// ```
#[must_use]
pub fn validate_pay_id(bill_id: &str, pay_id: &str) -> bool {
    let bill = extract_digits(bill_id);
    let pay = extract_digits(pay_id);
    if bill.len() < 6 || pay.len() < 6 {
        return false;
    }
    let bb = bill.as_bytes();
    let pb = pay.as_bytes();

    // First pay check: pay[..-2] -> pay[-2]
    let body1 = &pb[..pb.len() - 2];
    let c1 = (pb[pb.len() - 2] - b'0') as u32;
    if weighted_check(body1) != c1 {
        return false;
    }

    // Second pay check: bill ++ pay[..-1] -> pay[-1]
    let mut combined: Vec<u8> = Vec::with_capacity(bb.len() + pb.len() - 1);
    combined.extend_from_slice(bb);
    combined.extend_from_slice(&pb[..pb.len() - 1]);
    let c2 = (pb[pb.len() - 1] - b'0') as u32;
    weighted_check(&combined) == c2
}

/// Returns `true` iff [`validate_bill_id`] and [`validate_pay_id`] both hold.
#[must_use]
pub fn validate(bill_id: &str, pay_id: &str) -> bool {
    validate_bill_id(bill_id) && validate_pay_id(bill_id, pay_id)
}

/// Detect the bill type from the digit just before the bill-id check digit.
///
/// Returns `None` if `bill_id` is too short to contain a type digit.
///
/// ```
/// use parsitext::validators::bill::{bill_type, BillType};
///
/// // 7748317 8 0 1  →  type-digit 0 → Other
/// // 7748317 2 0 0  →  type-digit 2 → Electricity
/// assert_eq!(bill_type("7748317801"), Some(BillType::Other));
/// ```
#[must_use]
pub fn bill_type(bill_id: &str) -> Option<BillType> {
    let digits = extract_digits(bill_id);
    if digits.len() < 2 {
        return None;
    }
    let bytes = digits.as_bytes();
    let code = (bytes[bytes.len() - 2] - b'0') as u32;
    Some(BillType::from_code(code))
}

/// Compute the modulo-11 weighted check digit for the given digit body.
fn weighted_check(body: &[u8]) -> u32 {
    let mut sum: u32 = 0;
    for (i, b) in body.iter().rev().enumerate() {
        let d = (b - b'0') as u32;
        sum += d * ((i % 6) as u32 + 2);
    }
    let r = sum % 11;
    if r < 2 {
        r
    } else {
        11 - r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bill_id() {
        assert!(validate_bill_id("7748317801"));
    }

    #[test]
    fn invalid_bill_id_checksum() {
        assert!(!validate_bill_id("7748317800"));
        assert!(!validate_bill_id("7748317802"));
    }

    #[test]
    fn rejects_short_bill_id() {
        assert!(!validate_bill_id("12345"));
        assert!(!validate_bill_id(""));
    }

    #[test]
    fn accepts_persian_digits() {
        assert!(validate_bill_id("۷۷۴۸۳۱۷۸۰۱"));
    }

    #[test]
    fn pay_id_round_trip() {
        assert!(validate_pay_id("7748317801", "1234986"));
        assert!(!validate_pay_id("7748317801", "1234987"));
    }

    #[test]
    fn full_validate() {
        assert!(validate("7748317801", "1234986"));
        assert!(!validate("7748317800", "1234986")); // bill bad
        assert!(!validate("7748317801", "1234987")); // pay bad
    }

    #[test]
    fn pay_id_short() {
        assert!(!validate_pay_id("7748317801", "12"));
    }

    #[test]
    fn bill_type_detection() {
        // The digit before the check identifies the type.
        // Build a bill-id ending with `1<check>` (Water) and another with `2<check>` (Electricity).
        let body_w = "774831781";
        let bill_w = format!("{body_w}{}", weighted_check(body_w.as_bytes()));
        assert!(validate_bill_id(&bill_w));
        assert_eq!(bill_type(&bill_w), Some(BillType::Water));

        let body_e = "774831782";
        let bill_e = format!("{body_e}{}", weighted_check(body_e.as_bytes()));
        assert!(validate_bill_id(&bill_e));
        assert_eq!(bill_type(&bill_e), Some(BillType::Electricity));
    }

    #[test]
    fn bill_type_persian_names() {
        assert_eq!(BillType::Water.persian_name(), "آب");
        assert_eq!(BillType::Gas.persian_name(), "گاز");
        assert_eq!(BillType::Other.persian_name(), "سایر");
    }

    #[test]
    fn bill_type_display() {
        assert_eq!(format!("{}", BillType::Water), "Water");
        assert_eq!(format!("{}", BillType::DrivingFine), "DrivingFine");
    }

    #[test]
    fn bill_type_too_short() {
        assert_eq!(bill_type("1"), None);
    }
}
