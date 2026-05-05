//! Re-exports of [`parsitext`]'s Iranian validators (gated by the
//! `validators` Cargo feature, on by default).
//!
//! These cover the same surface as the popular `iranianbank` JS/PHP libraries:
//! national ID, Sheba (IBAN), bank-card (Luhn) plus issuer lookup, mobile
//! and landline phone, postal code, and vehicle plate.
//!
//! ```
//! # #[cfg(feature = "validators")]
//! # {
//! use iran_pay::validators::{bank_card, sheba, phone};
//!
//! assert!(bank_card::validate("6037-9900-0000-0006"));
//! assert_eq!(bank_card::bank("6037990000000006"), Some("Bank Melli Iran"));
//!
//! assert!(sheba::validate("IR062960000000100324200001"));
//!
//! let canon = phone::canonicalize("+989121234567");
//! assert_eq!(canon.as_deref(), Some("09121234567"));
//! # }
//! ```

pub use parsitext::validators::{
    bank_card, car_plate, landline, national_id, phone, postal_code, sheba, Operator, Plate,
    Province,
};
