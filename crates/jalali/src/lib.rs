//! Comprehensive Jalali (Persian / Shamsi) calendar for Rust.
//!
//! Provides [`JalaliDate`] (date), [`JalaliDateTime`] (date + time of day),
//! conversions to and from Gregorian, leap-year detection, day/month/year
//! arithmetic, season and weekday lookup, parsing, `strftime`-style format
//! and parse, and Persian/Arabic digit utilities.
//!
//! ```
//! use jalali::JalaliDate;
//!
//! let j = JalaliDate::from_gregorian(2024, 3, 20).unwrap();
//! assert_eq!((j.year(), j.month(), j.day()), (1403, 1, 1));
//! assert_eq!(j.to_string(), "1403/01/01");
//! assert_eq!(j.format("%Y %B (%K)"), "1403 فروردین (بهار)");
//! ```
//!
//! ## Optional features
//!
//! - `serde` — `Serialize`/`Deserialize` for the date and datetime types.
//! - `chrono` — interop with `chrono::NaiveDate` / `chrono::NaiveDateTime`.
//! - `timezone` — adds [`ZonedJalaliDateTime`], built on `chrono-tz` (implies
//!   `chrono`).
//! - `full` — enables all of the above.

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::fmt;

mod algorithm;
mod datetime;
pub mod digits;
mod format;
mod parse;
mod season;
mod today;
mod unix;

#[cfg(feature = "chrono")]
mod chrono_impl;
#[cfg(feature = "serde")]
mod serde_impl;
#[cfg(feature = "timezone")]
mod zoned;

pub use algorithm::{days_in_month, is_leap_year};
pub use datetime::JalaliDateTime;
pub use season::Season;

#[cfg(feature = "timezone")]
pub use zoned::ZonedJalaliDateTime;

/// Error returned when a date or conversion is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidJalaliDate {
        year: i32,
        month: u32,
        day: u32,
    },
    InvalidGregorianDate {
        year: i32,
        month: u32,
        day: u32,
    },
    InvalidTime {
        hour: u32,
        minute: u32,
        second: u32,
    },
    InvalidJalaliInput(String),
    /// Format string used a token the formatter does not recognize.
    UnknownFormatToken(char),
    /// Parse input did not match the format string.
    ParseMismatch {
        expected: String,
        found: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidJalaliDate { year, month, day } => {
                write!(f, "invalid Jalali date {year}/{month}/{day}")
            }
            Error::InvalidGregorianDate { year, month, day } => {
                write!(f, "invalid Gregorian date {year}/{month}/{day}")
            }
            Error::InvalidTime {
                hour,
                minute,
                second,
            } => write!(f, "invalid time {hour:02}:{minute:02}:{second:02}"),
            Error::InvalidJalaliInput(s) => write!(f, "could not parse Jalali date from {s:?}"),
            Error::UnknownFormatToken(c) => write!(f, "unknown format token %{c}"),
            Error::ParseMismatch { expected, found } => {
                write!(f, "parse mismatch: expected {expected}, found {found:?}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Days of the Persian week (Shanbeh-first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Saturday,
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

impl Weekday {
    /// Persian (Farsi) name of the weekday.
    pub fn persian_name(self) -> &'static str {
        match self {
            Weekday::Saturday => "شنبه",
            Weekday::Sunday => "یک‌شنبه",
            Weekday::Monday => "دوشنبه",
            Weekday::Tuesday => "سه‌شنبه",
            Weekday::Wednesday => "چهارشنبه",
            Weekday::Thursday => "پنج‌شنبه",
            Weekday::Friday => "جمعه",
        }
    }

    /// Single-character Persian abbreviation (ش، ی، د، س، چ، پ، ج).
    pub fn persian_abbreviation(self) -> &'static str {
        match self {
            Weekday::Saturday => "ش",
            Weekday::Sunday => "ی",
            Weekday::Monday => "د",
            Weekday::Tuesday => "س",
            Weekday::Wednesday => "چ",
            Weekday::Thursday => "پ",
            Weekday::Friday => "ج",
        }
    }

    /// Romanized name of the weekday.
    pub fn english_name(self) -> &'static str {
        match self {
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
        }
    }

    /// 0 for Saturday, 6 for Friday — matches the Persian week ordering.
    pub fn num_days_from_saturday(self) -> u32 {
        match self {
            Weekday::Saturday => 0,
            Weekday::Sunday => 1,
            Weekday::Monday => 2,
            Weekday::Tuesday => 3,
            Weekday::Wednesday => 4,
            Weekday::Thursday => 5,
            Weekday::Friday => 6,
        }
    }
}

/// A date on the Jalali (Persian / Shamsi) calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JalaliDate {
    year: i32,
    month: u32,
    day: u32,
}

impl JalaliDate {
    /// Construct a Jalali date, validating the components.
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, Error> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(Error::InvalidJalaliDate { year, month, day });
        }
        Ok(JalaliDate { year, month, day })
    }

    /// Construct without validation. Internal use only — callers must guarantee
    /// the components describe a real Jalali date.
    pub(crate) fn new_unchecked(year: i32, month: u32, day: u32) -> Self {
        JalaliDate { year, month, day }
    }

    /// Convert a Gregorian date to its Jalali equivalent.
    pub fn from_gregorian(gy: i32, gm: u32, gd: u32) -> Result<Self, Error> {
        if !algorithm::is_valid_gregorian(gy, gm, gd) {
            return Err(Error::InvalidGregorianDate {
                year: gy,
                month: gm,
                day: gd,
            });
        }
        let (y, m, d) = algorithm::g2j(gy, gm, gd);
        Ok(JalaliDate {
            year: y,
            month: m,
            day: d,
        })
    }

    /// Convert this Jalali date to its Gregorian equivalent `(year, month, day)`.
    pub fn to_gregorian(&self) -> (i32, u32, u32) {
        algorithm::j2g(self.year, self.month, self.day)
    }

    pub fn year(&self) -> i32 {
        self.year
    }
    pub fn month(&self) -> u32 {
        self.month
    }
    pub fn day(&self) -> u32 {
        self.day
    }

    /// Return a new date `days` away from this one (negative goes backward).
    pub fn add_days(&self, days: i32) -> Self {
        let abs = algorithm::j_to_abs(self.year, self.month, self.day) + days;
        let (y, m, d) = algorithm::abs_to_j(abs);
        JalaliDate {
            year: y,
            month: m,
            day: d,
        }
    }

    /// Number of whole days from `self` to `other` (negative if `other` precedes `self`).
    pub fn days_until(&self, other: &JalaliDate) -> i32 {
        algorithm::j_to_abs(other.year, other.month, other.day)
            - algorithm::j_to_abs(self.year, self.month, self.day)
    }

    /// Day of the week.
    pub fn weekday(&self) -> Weekday {
        // RD 1 = Mon = Persian index 2; offset by +1 so index 0 = Saturday.
        let abs = algorithm::j_to_abs(self.year, self.month, self.day);
        match (abs + 1).rem_euclid(7) {
            0 => Weekday::Saturday,
            1 => Weekday::Sunday,
            2 => Weekday::Monday,
            3 => Weekday::Tuesday,
            4 => Weekday::Wednesday,
            5 => Weekday::Thursday,
            6 => Weekday::Friday,
            _ => unreachable!(),
        }
    }

    /// Day of the year, 1..=365 (or 366 in a leap year).
    pub fn ordinal(&self) -> u32 {
        if self.month <= 6 {
            (self.month - 1) * 31 + self.day
        } else {
            6 * 31 + (self.month - 7) * 30 + self.day
        }
    }

    /// Persian name of the month.
    pub fn month_name(&self) -> &'static str {
        PERSIAN_MONTHS[(self.month - 1) as usize]
    }

    /// Whether the date's year is a leap year.
    pub fn is_leap_year(&self) -> bool {
        algorithm::is_leap_year(self.year)
    }

    /// Number of days in this date's month.
    pub fn days_in_this_month(&self) -> u32 {
        algorithm::days_in_month(self.year, self.month)
    }

    /// The season this date falls in.
    pub fn season(&self) -> Season {
        Season::from_month(self.month).expect("month is validated 1..=12")
    }

    /// Week of year (1-based, weeks start on Saturday). Week 1 contains 1
    /// Farvardin and may be partial.
    pub fn week_of_year(&self) -> u32 {
        let first_wd = JalaliDate::new_unchecked(self.year, 1, 1)
            .weekday()
            .num_days_from_saturday();
        (self.ordinal() + first_wd - 1) / 7 + 1
    }

    /// First day of this date's month.
    pub fn first_day_of_month(&self) -> Self {
        JalaliDate::new_unchecked(self.year, self.month, 1)
    }

    /// Last day of this date's month.
    pub fn last_day_of_month(&self) -> Self {
        JalaliDate::new_unchecked(self.year, self.month, self.days_in_this_month())
    }

    /// First day of this date's year (1 Farvardin).
    pub fn first_day_of_year(&self) -> Self {
        JalaliDate::new_unchecked(self.year, 1, 1)
    }

    /// Last day of this date's year (29 or 30 Esfand).
    pub fn last_day_of_year(&self) -> Self {
        let day = if algorithm::is_leap_year(self.year) {
            30
        } else {
            29
        };
        JalaliDate::new_unchecked(self.year, 12, day)
    }

    /// First day of this date's season.
    pub fn first_day_of_season(&self) -> Self {
        let (start, _) = self.season().months();
        JalaliDate::new_unchecked(self.year, start, 1)
    }

    /// Last day of this date's season.
    pub fn last_day_of_season(&self) -> Self {
        let (_, end) = self.season().months();
        JalaliDate::new_unchecked(self.year, end, algorithm::days_in_month(self.year, end))
    }

    /// Replace the year, clamping the day if the new year's Esfand has fewer
    /// days. Returns an error only if the resulting `(year, month, day)` is
    /// somehow invalid (e.g. arithmetic overflow scenarios).
    pub fn with_year(&self, year: i32) -> Result<Self, Error> {
        let max = algorithm::days_in_month(year, self.month);
        let day = self.day.min(max);
        JalaliDate::new(year, self.month, day)
    }

    /// Replace the month (1..=12), clamping the day to the new month's length.
    pub fn with_month(&self, month: u32) -> Result<Self, Error> {
        if !(1..=12).contains(&month) {
            return Err(Error::InvalidJalaliDate {
                year: self.year,
                month,
                day: self.day,
            });
        }
        let max = algorithm::days_in_month(self.year, month);
        let day = self.day.min(max);
        JalaliDate::new(self.year, month, day)
    }

    /// Replace the day. Errors if the day exceeds this month's length.
    pub fn with_day(&self, day: u32) -> Result<Self, Error> {
        JalaliDate::new(self.year, self.month, day)
    }

    /// Add (or subtract) calendar months. The day is clamped to the target
    /// month's length, so `1403/6/31 + 1 month = 1403/7/30`.
    pub fn add_months(&self, months: i32) -> Self {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months as i64;
        let new_year = total.div_euclid(12) as i32;
        let new_month = (total.rem_euclid(12) as u32) + 1;
        let max = algorithm::days_in_month(new_year, new_month);
        JalaliDate::new_unchecked(new_year, new_month, self.day.min(max))
    }

    /// Add (or subtract) calendar years. Esfand 30 in a leap year becomes
    /// Esfand 29 if the target year is not leap.
    pub fn add_years(&self, years: i32) -> Self {
        let new_year = self.year + years;
        let max = algorithm::days_in_month(new_year, self.month);
        JalaliDate::new_unchecked(new_year, self.month, self.day.min(max))
    }
}

impl fmt::Display for JalaliDate {
    /// Formats as `YYYY/MM/DD` with zero-padded month and day.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}/{:02}/{:02}", self.year, self.month, self.day)
    }
}

/// Persian month names, in order (Farvardin … Esfand).
pub const PERSIAN_MONTHS: [&str; 12] = [
    "فروردین",
    "اردیبهشت",
    "خرداد",
    "تیر",
    "مرداد",
    "شهریور",
    "مهر",
    "آبان",
    "آذر",
    "دی",
    "بهمن",
    "اسفند",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nowruz_1403_is_march_20_2024() {
        let j = JalaliDate::from_gregorian(2024, 3, 20).unwrap();
        assert_eq!(j.year(), 1403);
        assert_eq!(j.month(), 1);
        assert_eq!(j.day(), 1);
    }

    #[test]
    fn nowruz_1402_is_march_21_2023() {
        let j = JalaliDate::from_gregorian(2023, 3, 21).unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1402, 1, 1));
    }

    #[test]
    fn round_trip_known_dates() {
        let pairs = [
            ((2024, 3, 20), (1403, 1, 1)),
            ((2024, 3, 19), (1402, 12, 29)),
            ((2025, 3, 21), (1404, 1, 1)),
            ((1979, 2, 11), (1357, 11, 22)),
            ((1989, 6, 4), (1368, 3, 14)),
            ((2001, 9, 11), (1380, 6, 20)),
            ((2020, 1, 1), (1398, 10, 11)),
            ((2026, 4, 30), (1405, 2, 10)),
        ];
        for ((gy, gm, gd), (jy, jm, jd)) in pairs {
            let j = JalaliDate::from_gregorian(gy, gm, gd).unwrap();
            assert_eq!(
                (j.year(), j.month(), j.day()),
                (jy, jm, jd),
                "G->J wrong for {gy}-{gm}-{gd}"
            );
            assert_eq!(
                j.to_gregorian(),
                (gy, gm, gd),
                "J->G wrong for {jy}-{jm}-{jd}"
            );
        }
    }

    #[test]
    fn leap_year_detection() {
        // Known leap years in the Jalali calendar.
        for y in [1399, 1403, 1408, 1412, 1416, 1420, 1424] {
            assert!(is_leap_year(y), "{y} should be leap");
            assert_eq!(days_in_month(y, 12), 30);
        }
        // Known non-leap years.
        for y in [1400, 1401, 1402, 1404, 1405, 1406, 1407] {
            assert!(!is_leap_year(y), "{y} should not be leap");
            assert_eq!(days_in_month(y, 12), 29);
        }
    }

    #[test]
    fn days_in_each_month() {
        for m in 1..=6 {
            assert_eq!(days_in_month(1404, m), 31);
        }
        for m in 7..=11 {
            assert_eq!(days_in_month(1404, m), 30);
        }
        assert_eq!(days_in_month(1404, 12), 29);
        assert_eq!(days_in_month(1403, 12), 30);
    }

    #[test]
    fn invalid_dates_rejected() {
        assert!(JalaliDate::new(1404, 0, 1).is_err());
        assert!(JalaliDate::new(1404, 13, 1).is_err());
        assert!(JalaliDate::new(1404, 1, 32).is_err());
        assert!(JalaliDate::new(1404, 7, 31).is_err());
        assert!(JalaliDate::new(1404, 12, 30).is_err()); // 1404 is not leap
        assert!(JalaliDate::new(1403, 12, 30).is_ok()); // 1403 is leap
    }

    #[test]
    fn weekday_lookup() {
        // 1403/1/1 (Nowruz 1403) was Wednesday 2024-03-20.
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(j.weekday(), Weekday::Wednesday);

        // 1402/1/1 was Tuesday 2023-03-21.
        let j = JalaliDate::new(1402, 1, 1).unwrap();
        assert_eq!(j.weekday(), Weekday::Tuesday);

        // 1357/11/22 (Iranian Revolution) was Sunday 1979-02-11.
        let j = JalaliDate::new(1357, 11, 22).unwrap();
        assert_eq!(j.weekday(), Weekday::Sunday);
    }

    #[test]
    fn add_days_basic() {
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(j.add_days(1), JalaliDate::new(1403, 1, 2).unwrap());
        assert_eq!(j.add_days(31), JalaliDate::new(1403, 2, 1).unwrap());
        assert_eq!(j.add_days(-1), JalaliDate::new(1402, 12, 29).unwrap());
        // 1403 is leap, so 366 days advance one year.
        assert_eq!(j.add_days(366), JalaliDate::new(1404, 1, 1).unwrap());
        // 1404 is not leap.
        let j2 = JalaliDate::new(1404, 1, 1).unwrap();
        assert_eq!(j2.add_days(365), JalaliDate::new(1405, 1, 1).unwrap());
    }

    #[test]
    fn days_until_round_trip() {
        let a = JalaliDate::new(1403, 1, 1).unwrap();
        let b = JalaliDate::new(1404, 5, 17).unwrap();
        let n = a.days_until(&b);
        assert!(n > 0);
        assert_eq!(a.add_days(n), b);
        assert_eq!(b.days_until(&a), -n);
    }

    #[test]
    fn ordinal_day() {
        assert_eq!(JalaliDate::new(1403, 1, 1).unwrap().ordinal(), 1);
        assert_eq!(JalaliDate::new(1403, 6, 31).unwrap().ordinal(), 186);
        assert_eq!(JalaliDate::new(1403, 7, 1).unwrap().ordinal(), 187);
        assert_eq!(JalaliDate::new(1403, 12, 30).unwrap().ordinal(), 366);
        assert_eq!(JalaliDate::new(1404, 12, 29).unwrap().ordinal(), 365);
    }

    #[test]
    fn display_formatting() {
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(j.to_string(), "1403/01/01");
        let j = JalaliDate::new(1404, 12, 29).unwrap();
        assert_eq!(j.to_string(), "1404/12/29");
    }

    #[test]
    fn invalid_gregorian_rejected() {
        assert!(JalaliDate::from_gregorian(2024, 2, 30).is_err());
        assert!(JalaliDate::from_gregorian(2024, 13, 1).is_err());
        assert!(JalaliDate::from_gregorian(2023, 2, 29).is_err()); // 2023 not leap
        assert!(JalaliDate::from_gregorian(2024, 2, 29).is_ok()); // 2024 leap
    }

    #[test]
    fn month_name_is_persian() {
        assert_eq!(JalaliDate::new(1403, 1, 1).unwrap().month_name(), "فروردین");
        assert_eq!(JalaliDate::new(1403, 12, 1).unwrap().month_name(), "اسفند");
    }
}
