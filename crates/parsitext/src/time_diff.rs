//! Persian relative-time descriptions ("۲ روز پیش", "۳ ساعت دیگر").
//!
//! Two entry points:
//!
//! - [`describe`] — given a duration in seconds (positive = future,
//!   negative = past) returns a Persian phrase.
//! - [`describe_between`] — same idea but takes two epoch-seconds values
//!   (`from`, `to`) and reports `to - from`.
//!
//! Granularities (in seconds): minute (60), hour (3600), day (86400), week
//! (7d), month (30d), year (365d).  The largest non-zero unit wins.
//!
//! The helper uses Persian digits and the standard `پیش` / `دیگر` suffix
//! convention.

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;
const MONTH: i64 = 30 * DAY;
const YEAR: i64 = 365 * DAY;

/// Describe a signed `seconds` offset as a Persian relative-time phrase.
///
/// - `seconds > 0` → future (`دیگر`).
/// - `seconds < 0` → past (`پیش`).
/// - `|seconds| < 60` → `همین الان`.
///
/// ```
/// use parsitext::time_diff::describe;
///
/// assert_eq!(describe(0), "همین الان");
/// assert_eq!(describe(-90), "۱ دقیقه پیش");
/// assert_eq!(describe(-3 * 3600), "۳ ساعت پیش");
/// assert_eq!(describe(2 * 86400), "۲ روز دیگر");
/// assert_eq!(describe(-365 * 86400 * 5), "۵ سال پیش");
/// ```
#[must_use]
pub fn describe(seconds: i64) -> String {
    let abs = seconds.unsigned_abs();
    if abs < MINUTE as u64 {
        return "همین الان".to_owned();
    }

    let (n, unit) = pick_unit(abs);
    let suffix = if seconds < 0 { "پیش" } else { "دیگر" };
    format!("{} {} {}", to_persian_digits(n), unit, suffix)
}

/// Describe `to - from`, both in epoch seconds, as a Persian relative-time
/// phrase.
///
/// ```
/// use parsitext::time_diff::describe_between;
///
/// assert_eq!(describe_between(1_000_000, 1_000_000 + 3600), "۱ ساعت دیگر");
/// assert_eq!(describe_between(1_000_000, 1_000_000 - 7 * 86400), "۱ هفته پیش");
/// ```
#[must_use]
pub fn describe_between(from: i64, to: i64) -> String {
    describe(to.saturating_sub(from))
}

fn pick_unit(abs_seconds: u64) -> (u64, &'static str) {
    if abs_seconds >= YEAR as u64 {
        (abs_seconds / YEAR as u64, "سال")
    } else if abs_seconds >= MONTH as u64 {
        (abs_seconds / MONTH as u64, "ماه")
    } else if abs_seconds >= WEEK as u64 {
        (abs_seconds / WEEK as u64, "هفته")
    } else if abs_seconds >= DAY as u64 {
        (abs_seconds / DAY as u64, "روز")
    } else if abs_seconds >= HOUR as u64 {
        (abs_seconds / HOUR as u64, "ساعت")
    } else {
        (abs_seconds / MINUTE as u64, "دقیقه")
    }
}

fn to_persian_digits(mut n: u64) -> String {
    if n == 0 {
        return "۰".to_owned();
    }
    let mut buf = Vec::with_capacity(8);
    while n > 0 {
        let d = (n % 10) as u32;
        buf.push(char::from_u32(0x06F0 + d).unwrap_or('?'));
        n /= 10;
    }
    buf.iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_now() {
        assert_eq!(describe(0), "همین الان");
        assert_eq!(describe(30), "همین الان");
        assert_eq!(describe(-59), "همین الان");
    }

    #[test]
    fn minutes() {
        assert_eq!(describe(-MINUTE), "۱ دقیقه پیش");
        assert_eq!(describe(15 * MINUTE), "۱۵ دقیقه دیگر");
    }

    #[test]
    fn hours_days_weeks() {
        assert_eq!(describe(-HOUR), "۱ ساعت پیش");
        assert_eq!(describe(2 * DAY), "۲ روز دیگر");
        assert_eq!(describe(-3 * WEEK), "۳ هفته پیش");
    }

    #[test]
    fn months_years() {
        assert_eq!(describe(2 * MONTH), "۲ ماه دیگر");
        assert_eq!(describe(-5 * YEAR), "۵ سال پیش");
    }

    #[test]
    fn between_helper() {
        let now = 1_700_000_000;
        assert_eq!(describe_between(now, now + 3600), "۱ ساعت دیگر");
        assert_eq!(describe_between(now, now - 86400), "۱ روز پیش");
    }

    #[test]
    fn between_saturates_no_overflow() {
        // i64::MIN .. 0 must not overflow.
        let s = describe_between(0, i64::MIN);
        assert!(s.ends_with("پیش"));
    }

    #[test]
    fn persian_digits_basic() {
        assert_eq!(to_persian_digits(0), "۰");
        assert_eq!(to_persian_digits(123), "۱۲۳");
    }
}
