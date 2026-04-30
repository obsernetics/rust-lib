//! Convert a Unix timestamp (e.g. from a database column) to a Jalali date.
//!
//! Run with: `cargo run --example unix_timestamp -p jalali`

use jalali_calendar::JalaliDate;

fn main() {
    let now_ts: i64 = 1_710_892_800; // 2024-03-20 00:00 UTC
    let j = JalaliDate::from_unix_timestamp(now_ts).unwrap();
    println!(
        "Unix {now_ts} = Jalali {j} ({} {})",
        j.day(),
        j.month_name()
    );

    let back = j.to_unix_timestamp();
    println!("Round trip: {back}");
}
