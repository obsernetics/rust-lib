//! Add days, count days between two dates, and inspect weekday rollover.
//!
//! Run with: `cargo run --example date_arithmetic -p jalali`

use jalali::JalaliDate;

fn main() {
    let nowruz = JalaliDate::new(1403, 1, 1).unwrap();
    let in_one_week = nowruz.add_days(7);
    println!(
        "{} ({}) + 7 days = {} ({})",
        nowruz,
        nowruz.weekday().english_name(),
        in_one_week,
        in_one_week.weekday().english_name(),
    );

    let new_year_eve = JalaliDate::new(1403, 12, 30).unwrap();
    let days = nowruz.days_until(&new_year_eve);
    println!("Days from {nowruz} to {new_year_eve}: {days}");
}
