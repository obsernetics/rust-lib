//! Convert a Gregorian date to Jalali and print the result.
//!
//! Run with: `cargo run --example basic_conversion -p jalali`

use jalali_calendar::JalaliDate;

fn main() {
    let j = JalaliDate::from_gregorian(2024, 3, 20).expect("valid Gregorian date");
    println!("Gregorian 2024-03-20 = Jalali {j} ({})", j.month_name());
    // → Gregorian 2024-03-20 = Jalali 1403/01/01 (فروردین)

    let (gy, gm, gd) = JalaliDate::new(1403, 11, 22).unwrap().to_gregorian();
    println!("Jalali 1403/11/22 = Gregorian {gy}-{gm:02}-{gd:02}");
}
