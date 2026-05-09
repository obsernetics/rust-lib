//! Validators for Iranian identifiers and document numbers.
//!
//! Each sub-module provides one validator family.  All validators accept
//! input with **either Persian or Latin digits** and tolerate common
//! separators (spaces, dashes) — they extract digits internally before
//! checking format and checksums.
//!
//! | Module | What it validates |
//! |--------|-------------------|
//! | [`national_id`] | کد ملی — Iranian personal national ID (10 digits + checksum) |
//! | [`legal_id`] | شناسه ملی — Iranian legal/company ID (11 digits + weighted checksum) |
//! | [`sheba`] | شبا — Iranian IBAN (`IR` + 24 digits + mod-97 checksum) |
//! | [`bank_card`] | کارت بانکی — 16-digit card number (Luhn checksum) |
//! | [`phone`] | شماره موبایل — 11-digit Iranian mobile number + operator detection |
//! | [`landline`] | تلفن ثابت — Iranian fixed-line + provincial area-code lookup |
//! | [`postal_code`] | کد پستی — 10-digit Iranian postal code |
//! | [`car_plate`] | پلاک خودرو — Iranian vehicle plate parser |
//! | [`bill`] | قبض — Iranian utility bill ID + pay-id checksums and bill-type detection |
//!
//! [`sheba`] and [`bank_card`] also expose `bank()` / `bank_persian()`
//! helpers that look up the issuing bank from the number.

pub mod bank_card;
mod banks;
pub mod bill;
pub mod car_plate;
pub mod landline;
pub mod legal_id;
pub mod national_id;
pub mod phone;
pub mod postal_code;
pub mod sheba;

pub use car_plate::Plate;
pub use landline::Province;
pub use phone::Operator;

// ── shared digit-extraction helpers ───────────────────────────────────────────

#[inline]
pub(crate) fn extract_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_digit() {
                Some(c)
            } else {
                persian_or_arabic_digit_to_ascii(c)
            }
        })
        .collect()
}

#[inline]
pub(crate) fn persian_or_arabic_digit_to_ascii(c: char) -> Option<char> {
    let cp = c as u32;
    if (0x06F0..=0x06F9).contains(&cp) {
        char::from_u32(cp - 0x06F0 + b'0' as u32)
    } else if (0x0660..=0x0669).contains(&cp) {
        char::from_u32(cp - 0x0660 + b'0' as u32)
    } else {
        None
    }
}
