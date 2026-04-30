//! Parse a date with Persian digits and Persian-style separators.
//!
//! Run with: `cargo run --example parse_persian_input -p jalali`

use jalali::{digits, JalaliDate};

fn main() {
    // Form input might arrive with Persian digits and a slash separator.
    let user_input = "۱۴۰۳/۰۱/۰۱";
    let j: JalaliDate = user_input.parse().expect("valid Jalali date");

    let weekday = j.weekday();
    println!(
        "Parsed: {j} — {} ({})",
        j.month_name(),
        weekday.persian_name()
    );

    // Round-trip back to Persian digits for display.
    println!("Display: {}", digits::to_persian(&j.to_string()));
}
