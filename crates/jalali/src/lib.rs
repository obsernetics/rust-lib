//! # jalali-calendar
//!
//! A comprehensive Jalali (Persian / Shamsi) calendar library for Rust.
//!
//! The crate publishes as **`jalali-calendar`** on crates.io but its library
//! name is `jalali`, so user code imports it as `use jalali::*;`.
//!
//! ## What's in the box
//!
//! - [`JalaliDate`] — a naive Jalali calendar date (year, month, day) with
//!   validation, conversion to/from Gregorian, calendar arithmetic, and
//!   convenience helpers (weekday, ordinal, season, week-of-year, etc.).
//! - [`JalaliDateTime`] — a [`JalaliDate`] paired with a time of day.
//! - [`Weekday`], [`Season`] — value types used throughout the API.
//! - [`PERSIAN_MONTHS`] — the canonical Persian month names.
//! - [`digits`] — convert between Latin (ASCII), Persian, and Eastern-Arabic
//!   digits.
//! - `strftime`-style [`JalaliDate::format`] / [`JalaliDate::parse_format`]
//!   (and the matching [`JalaliDateTime`] methods). See the [`format`] module
//!   docs for the supported tokens.
//!
//! ## Quick example
//!
//! ```
//! use jalali::JalaliDate;
//!
//! // Convert from Gregorian.
//! let nowruz = JalaliDate::from_gregorian(2024, 3, 20).unwrap();
//! assert_eq!((nowruz.year(), nowruz.month(), nowruz.day()), (1403, 1, 1));
//!
//! // Format with strftime-style tokens (Persian month + season).
//! assert_eq!(nowruz.format("%-d %B (%K)"), "1 فروردین (بهار)");
//!
//! // Parse Persian-digit input transparently.
//! let parsed: JalaliDate = "۱۴۰۳/۰۱/۰۱".parse().unwrap();
//! assert_eq!(parsed, nowruz);
//!
//! // Calendar arithmetic with month-end clamping.
//! let next_month = JalaliDate::new(1403, 6, 31).unwrap().add_months(1);
//! assert_eq!(next_month, JalaliDate::new(1403, 7, 30).unwrap());
//! ```
//!
//! ## Date range
//!
//! The Pournader-Toossi conversion algorithm is accurate for Jalali years
//! roughly **1..=3177 AP** (covering all practical contemporary use). Outside
//! that range the leap-year approximation drifts.
//!
//! ## Cargo features
//!
//! All optional. The crate has zero required dependencies.
//!
//! | Feature    | Pulls in            | Adds                                                                 |
//! |------------|---------------------|----------------------------------------------------------------------|
//! | `serde`    | `serde`             | `Serialize`/`Deserialize` for [`JalaliDate`] and [`JalaliDateTime`]. |
//! | `chrono`   | `chrono`            | Interop with `chrono::NaiveDate` / `chrono::NaiveDateTime`.          |
//! | `timezone` | `chrono`,`chrono-tz`| [`ZonedJalaliDateTime`] anchored to a `chrono_tz::Tz`.               |
//! | `full`     | all of the above    | Convenience flag.                                                    |
//!
//! Enable via `Cargo.toml`:
//!
//! ```toml
//! jalali-calendar = { version = "0.1", features = ["serde", "chrono"] }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

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
#[cfg_attr(docsrs, doc(cfg(feature = "timezone")))]
pub use zoned::ZonedJalaliDateTime;

/// Errors returned by this crate.
///
/// Variants are stable and exhaustively matched in the crate's public API.
/// All variants implement [`Display`] (with a human-readable message) and
/// [`std::error::Error`], so they integrate with `?` and error chains.
///
/// [`Display`]: std::fmt::Display
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The provided `(year, month, day)` does not describe a real Jalali date
    /// (e.g. day 30 of Esfand in a non-leap year, or month 13).
    InvalidJalaliDate {
        /// Jalali year that was provided.
        year: i32,
        /// Jalali month that was provided.
        month: u32,
        /// Jalali day that was provided.
        day: u32,
    },
    /// The provided `(year, month, day)` does not describe a real Gregorian
    /// date (e.g. February 30, or month 0).
    InvalidGregorianDate {
        /// Gregorian year that was provided.
        year: i32,
        /// Gregorian month that was provided.
        month: u32,
        /// Gregorian day that was provided.
        day: u32,
    },
    /// The time-of-day components were out of range
    /// (`hour > 23`, `minute > 59`, `second > 59`, or
    /// `nanosecond > 999_999_999`).
    InvalidTime {
        /// Hour that was provided.
        hour: u32,
        /// Minute that was provided.
        minute: u32,
        /// Second that was provided.
        second: u32,
    },
    /// The string passed to [`JalaliDate::parse`] or [`str::parse`] (via the
    /// [`std::str::FromStr`] impl) could not be split into year/month/day
    /// components.
    ///
    /// The contained `String` is the original input.
    InvalidJalaliInput(String),
    /// A format string passed to [`JalaliDate::format`] or
    /// [`JalaliDate::parse_format`] used a `%X` token the implementation
    /// does not recognize.
    UnknownFormatToken(char),
    /// A parse against an `strftime`-style format string failed because the
    /// input did not contain the literal text or numeric field expected at
    /// that position.
    ParseMismatch {
        /// Description of what the parser was looking for.
        expected: String,
        /// The actual input (or a slice of it) that was found instead.
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

/// A day of the Persian week.
///
/// Variants are declared in Persian week order — `Saturday` (شنبه) is the
/// first day of the week and `Friday` (جمعه) is the last. This matches the
/// numbering returned by [`Weekday::num_days_from_saturday`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    /// شنبه — first day of the Persian week.
    Saturday,
    /// یک‌شنبه.
    Sunday,
    /// دوشنبه.
    Monday,
    /// سه‌شنبه.
    Tuesday,
    /// چهارشنبه.
    Wednesday,
    /// پنج‌شنبه.
    Thursday,
    /// جمعه — last day of the Persian week.
    Friday,
}

impl Weekday {
    /// Full Persian (Farsi) name of the weekday.
    ///
    /// ```
    /// # use jalali::Weekday;
    /// assert_eq!(Weekday::Saturday.persian_name(), "شنبه");
    /// assert_eq!(Weekday::Friday.persian_name(), "جمعه");
    /// ```
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

    /// Single-character Persian abbreviation (`ش`, `ی`, `د`, `س`, `چ`, `پ`,
    /// `ج`).
    ///
    /// ```
    /// # use jalali::Weekday;
    /// assert_eq!(Weekday::Wednesday.persian_abbreviation(), "چ");
    /// ```
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

    /// English name of the weekday (`"Saturday"` … `"Friday"`).
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

    /// Position in the Persian week, with Saturday = 0 and Friday = 6.
    ///
    /// Useful when computing week numbers or laying out a calendar grid.
    ///
    /// ```
    /// # use jalali::Weekday;
    /// assert_eq!(Weekday::Saturday.num_days_from_saturday(), 0);
    /// assert_eq!(Weekday::Friday.num_days_from_saturday(), 6);
    /// ```
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

/// A naive date on the Jalali (Persian / Shamsi) calendar.
///
/// "Naive" means the date carries no timezone information; it represents a
/// calendar date as a human would write it. For a date paired with a time of
/// day see [`JalaliDateTime`]; for a timezone-aware datetime enable the
/// `timezone` Cargo feature and use [`ZonedJalaliDateTime`].
///
/// Internal fields are private — values are only constructible via
/// validating constructors ([`JalaliDate::new`], [`JalaliDate::from_gregorian`],
/// [`JalaliDate::from_unix_timestamp`], the [`std::str::FromStr`] impl, or
/// the `chrono` interop methods), so a `JalaliDate` always represents a real
/// Jalali date.
///
/// `JalaliDate` is `Copy` and ordered chronologically.
///
/// ```
/// use jalali::JalaliDate;
///
/// let a = JalaliDate::new(1403, 1, 1)?;
/// let b = JalaliDate::new(1403, 12, 30)?;
/// assert!(a < b);
/// assert_eq!(a.days_until(&b), 365);
/// # Ok::<(), jalali::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JalaliDate {
    year: i32,
    month: u32,
    day: u32,
}

impl JalaliDate {
    /// Construct a Jalali date from raw components, validating month and day.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidJalaliDate`] if `month` is not in `1..=12` or
    /// `day` is outside the valid range for the given month and year (e.g.
    /// day 30 of Esfand in a non-leap year).
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// assert!(JalaliDate::new(1403, 12, 30).is_ok());  // 1403 is leap
    /// assert!(JalaliDate::new(1404, 12, 30).is_err()); // 1404 is not
    /// ```
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, Error> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(Error::InvalidJalaliDate { year, month, day });
        }
        Ok(JalaliDate { year, month, day })
    }

    /// Construct without validation. Internal use only — callers must
    /// guarantee the components describe a real Jalali date.
    pub(crate) fn new_unchecked(year: i32, month: u32, day: u32) -> Self {
        JalaliDate { year, month, day }
    }

    /// Convert a Gregorian (proleptic) date to its Jalali equivalent.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGregorianDate`] if `(gy, gm, gd)` is not a
    /// real Gregorian date.
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// let j = JalaliDate::from_gregorian(2024, 3, 20)?;
    /// assert_eq!((j.year(), j.month(), j.day()), (1403, 1, 1));
    /// # Ok::<(), jalali::Error>(())
    /// ```
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

    /// Convert this Jalali date to its Gregorian equivalent as
    /// `(year, month, day)`.
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// let g = JalaliDate::new(1403, 1, 1)?.to_gregorian();
    /// assert_eq!(g, (2024, 3, 20));
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn to_gregorian(&self) -> (i32, u32, u32) {
        algorithm::j2g(self.year, self.month, self.day)
    }

    /// Jalali year (e.g. `1403`).
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Jalali month, in `1..=12` (1 = Farvardin … 12 = Esfand).
    pub fn month(&self) -> u32 {
        self.month
    }

    /// Day of the month (`1..=31`).
    pub fn day(&self) -> u32 {
        self.day
    }

    /// Return a new date offset from this one by `days`. Negative values move
    /// backward.
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// let d = JalaliDate::new(1403, 1, 1)?;
    /// assert_eq!(d.add_days(31), JalaliDate::new(1403, 2, 1)?);
    /// assert_eq!(d.add_days(-1), JalaliDate::new(1402, 12, 29)?);
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn add_days(&self, days: i32) -> Self {
        let abs = algorithm::j_to_abs(self.year, self.month, self.day) + days;
        let (y, m, d) = algorithm::abs_to_j(abs);
        JalaliDate {
            year: y,
            month: m,
            day: d,
        }
    }

    /// Number of whole days from `self` to `other`. Negative when `other`
    /// precedes `self`.
    ///
    /// `a.add_days(a.days_until(&b)) == b` for any two valid dates.
    pub fn days_until(&self, other: &JalaliDate) -> i32 {
        algorithm::j_to_abs(other.year, other.month, other.day)
            - algorithm::j_to_abs(self.year, self.month, self.day)
    }

    /// Day of the week.
    ///
    /// ```
    /// # use jalali::{JalaliDate, Weekday};
    /// // Nowruz 1403 (2024-03-20) was a Wednesday.
    /// assert_eq!(JalaliDate::new(1403, 1, 1)?.weekday(), Weekday::Wednesday);
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn weekday(&self) -> Weekday {
        // Rata Die day 1 (Jan 1, 1 CE) was a Monday, which is Persian
        // weekday index 2 (Sat=0). Offsetting by +1 maps RD%7=0 -> Saturday.
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

    /// Day of the year, in `1..=365` (or `1..=366` in a leap year).
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// assert_eq!(JalaliDate::new(1403, 1, 1)?.ordinal(), 1);
    /// assert_eq!(JalaliDate::new(1403, 12, 30)?.ordinal(), 366);
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn ordinal(&self) -> u32 {
        if self.month <= 6 {
            (self.month - 1) * 31 + self.day
        } else {
            6 * 31 + (self.month - 7) * 30 + self.day
        }
    }

    /// Persian name of the month (e.g. `"فروردین"`).
    pub fn month_name(&self) -> &'static str {
        PERSIAN_MONTHS[(self.month - 1) as usize]
    }

    /// Whether this date's year is a Jalali leap year (Esfand has 30 days
    /// instead of 29).
    pub fn is_leap_year(&self) -> bool {
        algorithm::is_leap_year(self.year)
    }

    /// Number of days in this date's month (29, 30, or 31).
    pub fn days_in_this_month(&self) -> u32 {
        algorithm::days_in_month(self.year, self.month)
    }

    /// The [`Season`] this date falls in.
    pub fn season(&self) -> Season {
        Season::from_month(self.month).expect("month is validated 1..=12")
    }

    /// Week of the Jalali year, 1-based, weeks starting on Saturday.
    ///
    /// Week 1 contains 1 Farvardin and may be a partial week.
    pub fn week_of_year(&self) -> u32 {
        let first_wd = JalaliDate::new_unchecked(self.year, 1, 1)
            .weekday()
            .num_days_from_saturday();
        (self.ordinal() + first_wd - 1) / 7 + 1
    }

    /// First day of this date's month — `(year, month, 1)`.
    pub fn first_day_of_month(&self) -> Self {
        JalaliDate::new_unchecked(self.year, self.month, 1)
    }

    /// Last day of this date's month — `(year, month, days_in_month)`.
    pub fn last_day_of_month(&self) -> Self {
        JalaliDate::new_unchecked(self.year, self.month, self.days_in_this_month())
    }

    /// 1 Farvardin of this date's year.
    pub fn first_day_of_year(&self) -> Self {
        JalaliDate::new_unchecked(self.year, 1, 1)
    }

    /// Last day of this date's year — 30 Esfand in leap years, 29 Esfand
    /// otherwise.
    pub fn last_day_of_year(&self) -> Self {
        let day = if algorithm::is_leap_year(self.year) {
            30
        } else {
            29
        };
        JalaliDate::new_unchecked(self.year, 12, day)
    }

    /// First day of this date's [`Season`].
    pub fn first_day_of_season(&self) -> Self {
        let (start, _) = self.season().months();
        JalaliDate::new_unchecked(self.year, start, 1)
    }

    /// Last day of this date's [`Season`].
    pub fn last_day_of_season(&self) -> Self {
        let (_, end) = self.season().months();
        JalaliDate::new_unchecked(self.year, end, algorithm::days_in_month(self.year, end))
    }

    /// Return a new date with the year replaced.
    ///
    /// If the current day does not fit in the same month of the target year
    /// (only possible for `(month=12, day=30)` moving from a leap year to a
    /// non-leap one), the day is clamped to the target month's length rather
    /// than producing an error.
    ///
    /// # Errors
    ///
    /// Currently never fails for in-range inputs — the [`Result`] return
    /// type is preserved for API symmetry with [`with_month`](Self::with_month).
    pub fn with_year(&self, year: i32) -> Result<Self, Error> {
        let max = algorithm::days_in_month(year, self.month);
        let day = self.day.min(max);
        JalaliDate::new(year, self.month, day)
    }

    /// Return a new date with the month replaced. The day is clamped to the
    /// target month's length.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidJalaliDate`] if `month` is outside `1..=12`.
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

    /// Return a new date with the day replaced.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidJalaliDate`] if `day` exceeds the current
    /// month's length.
    pub fn with_day(&self, day: u32) -> Result<Self, Error> {
        JalaliDate::new(self.year, self.month, day)
    }

    /// Add (or subtract) calendar months. The day is clamped to the target
    /// month's length.
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// // Mehr (month 7) only has 30 days, so day 31 clamps.
    /// let d = JalaliDate::new(1403, 6, 31)?.add_months(1);
    /// assert_eq!(d, JalaliDate::new(1403, 7, 30)?);
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn add_months(&self, months: i32) -> Self {
        let total = self.year as i64 * 12 + (self.month as i64 - 1) + months as i64;
        let new_year = total.div_euclid(12) as i32;
        let new_month = (total.rem_euclid(12) as u32) + 1;
        let max = algorithm::days_in_month(new_year, new_month);
        JalaliDate::new_unchecked(new_year, new_month, self.day.min(max))
    }

    /// Add (or subtract) calendar years. Esfand 30 in a leap year is clamped
    /// to Esfand 29 if the target year is not leap.
    ///
    /// ```
    /// # use jalali::JalaliDate;
    /// let d = JalaliDate::new(1403, 12, 30)?; // 1403 is leap
    /// assert_eq!(d.add_years(1), JalaliDate::new(1404, 12, 29)?);
    /// # Ok::<(), jalali::Error>(())
    /// ```
    pub fn add_years(&self, years: i32) -> Self {
        let new_year = self.year + years;
        let max = algorithm::days_in_month(new_year, self.month);
        JalaliDate::new_unchecked(new_year, self.month, self.day.min(max))
    }
}

impl fmt::Display for JalaliDate {
    /// Formats as `YYYY/MM/DD` with zero-padded month and day.
    ///
    /// For richer formatting (Persian month names, AM/PM, etc.) use
    /// [`JalaliDate::format`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}/{:02}/{:02}", self.year, self.month, self.day)
    }
}

/// The twelve Persian month names in calendar order: Farvardin (`فروردین`),
/// Ordibehesht (`اردیبهشت`), …, Esfand (`اسفند`).
///
/// Indexed as `PERSIAN_MONTHS[month - 1]`.
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
        for y in [1399, 1403, 1408, 1412, 1416, 1420, 1424] {
            assert!(is_leap_year(y), "{y} should be leap");
            assert_eq!(days_in_month(y, 12), 30);
        }
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
        assert!(JalaliDate::new(1404, 12, 30).is_err());
        assert!(JalaliDate::new(1403, 12, 30).is_ok());
    }

    #[test]
    fn weekday_lookup() {
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(j.weekday(), Weekday::Wednesday);

        let j = JalaliDate::new(1402, 1, 1).unwrap();
        assert_eq!(j.weekday(), Weekday::Tuesday);

        let j = JalaliDate::new(1357, 11, 22).unwrap();
        assert_eq!(j.weekday(), Weekday::Sunday);
    }

    #[test]
    fn add_days_basic() {
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(j.add_days(1), JalaliDate::new(1403, 1, 2).unwrap());
        assert_eq!(j.add_days(31), JalaliDate::new(1403, 2, 1).unwrap());
        assert_eq!(j.add_days(-1), JalaliDate::new(1402, 12, 29).unwrap());
        assert_eq!(j.add_days(366), JalaliDate::new(1404, 1, 1).unwrap());
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
        assert!(JalaliDate::from_gregorian(2023, 2, 29).is_err());
        assert!(JalaliDate::from_gregorian(2024, 2, 29).is_ok());
    }

    #[test]
    fn month_name_is_persian() {
        assert_eq!(JalaliDate::new(1403, 1, 1).unwrap().month_name(), "فروردین");
        assert_eq!(JalaliDate::new(1403, 12, 1).unwrap().month_name(), "اسفند");
    }
}
