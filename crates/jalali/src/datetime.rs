//! [`JalaliDateTime`] — a Jalali date plus a time of day, naive (no timezone).

use std::fmt;

use crate::{algorithm, today, Error, JalaliDate, Season, Weekday};

const SECONDS_PER_DAY: i64 = 86_400;

/// Jalali date paired with a time of day (hour/minute/second/nanosecond),
/// naive — i.e. without an associated timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JalaliDateTime {
    date: JalaliDate,
    hour: u32,
    minute: u32,
    second: u32,
    nanosecond: u32,
}

impl JalaliDateTime {
    /// Construct a datetime from validated components. `nanosecond` is 0..=999_999_999.
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, Error> {
        Self::with_nanos(year, month, day, hour, minute, second, 0)
    }

    /// Like [`new`](Self::new), but with explicit nanoseconds.
    pub fn with_nanos(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nanosecond: u32,
    ) -> Result<Self, Error> {
        let date = JalaliDate::new(year, month, day)?;
        if hour > 23 || minute > 59 || second > 59 || nanosecond > 999_999_999 {
            return Err(Error::InvalidTime {
                hour,
                minute,
                second,
            });
        }
        Ok(JalaliDateTime {
            date,
            hour,
            minute,
            second,
            nanosecond,
        })
    }

    /// Combine an existing date with a time of day.
    pub fn from_date_time(
        date: JalaliDate,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, Error> {
        if hour > 23 || minute > 59 || second > 59 {
            return Err(Error::InvalidTime {
                hour,
                minute,
                second,
            });
        }
        Ok(JalaliDateTime {
            date,
            hour,
            minute,
            second,
            nanosecond: 0,
        })
    }

    /// The current UTC datetime.
    pub fn now() -> Self {
        let secs = today::now_unix_seconds();
        Self::from_unix_timestamp(secs).expect("system clock returned a valid timestamp")
    }

    /// Construct from a Unix timestamp (seconds since 1970-01-01 UTC).
    pub fn from_unix_timestamp(seconds: i64) -> Result<Self, Error> {
        let days = seconds.div_euclid(SECONDS_PER_DAY);
        let secs_of_day = seconds.rem_euclid(SECONDS_PER_DAY) as u32;
        let date = JalaliDate::from_unix_timestamp(days * SECONDS_PER_DAY)?;
        let hour = secs_of_day / 3600;
        let minute = (secs_of_day / 60) % 60;
        let second = secs_of_day % 60;
        Ok(JalaliDateTime {
            date,
            hour,
            minute,
            second,
            nanosecond: 0,
        })
    }

    /// Unix timestamp (UTC seconds) for this datetime.
    pub fn to_unix_timestamp(&self) -> i64 {
        self.date.to_unix_timestamp()
            + self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64
    }

    pub fn date(&self) -> JalaliDate {
        self.date
    }
    pub fn year(&self) -> i32 {
        self.date.year()
    }
    pub fn month(&self) -> u32 {
        self.date.month()
    }
    pub fn day(&self) -> u32 {
        self.date.day()
    }
    pub fn hour(&self) -> u32 {
        self.hour
    }
    pub fn minute(&self) -> u32 {
        self.minute
    }
    pub fn second(&self) -> u32 {
        self.second
    }
    pub fn nanosecond(&self) -> u32 {
        self.nanosecond
    }
    pub fn weekday(&self) -> Weekday {
        self.date.weekday()
    }
    pub fn season(&self) -> Season {
        self.date.season()
    }
    pub fn ordinal(&self) -> u32 {
        self.date.ordinal()
    }
    pub fn week_of_year(&self) -> u32 {
        self.date.week_of_year()
    }
    pub fn is_leap_year(&self) -> bool {
        self.date.is_leap_year()
    }
    pub fn month_name(&self) -> &'static str {
        self.date.month_name()
    }

    /// Add (or subtract) days, preserving the time of day.
    pub fn add_days(&self, days: i32) -> Self {
        JalaliDateTime {
            date: self.date.add_days(days),
            ..*self
        }
    }

    /// Add (or subtract) calendar months. The day is clamped to the target
    /// month's length; the time of day is preserved.
    pub fn add_months(&self, months: i32) -> Self {
        JalaliDateTime {
            date: self.date.add_months(months),
            ..*self
        }
    }

    /// Add (or subtract) calendar years. Time of day is preserved.
    pub fn add_years(&self, years: i32) -> Self {
        JalaliDateTime {
            date: self.date.add_years(years),
            ..*self
        }
    }

    /// Add (or subtract) seconds; rolls over into days as needed.
    pub fn add_seconds(&self, seconds: i64) -> Self {
        let total = self.to_unix_timestamp() + seconds;
        let mut out = Self::from_unix_timestamp(total).expect("valid timestamp");
        out.nanosecond = self.nanosecond;
        out
    }

    /// Replace the time of day.
    pub fn with_time(&self, hour: u32, minute: u32, second: u32) -> Result<Self, Error> {
        Self::from_date_time(self.date, hour, minute, second)
    }

    pub fn with_hour(&self, hour: u32) -> Result<Self, Error> {
        if hour > 23 {
            return Err(Error::InvalidTime {
                hour,
                minute: self.minute,
                second: self.second,
            });
        }
        Ok(JalaliDateTime { hour, ..*self })
    }

    pub fn with_minute(&self, minute: u32) -> Result<Self, Error> {
        if minute > 59 {
            return Err(Error::InvalidTime {
                hour: self.hour,
                minute,
                second: self.second,
            });
        }
        Ok(JalaliDateTime { minute, ..*self })
    }

    pub fn with_second(&self, second: u32) -> Result<Self, Error> {
        if second > 59 {
            return Err(Error::InvalidTime {
                hour: self.hour,
                minute: self.minute,
                second,
            });
        }
        Ok(JalaliDateTime { second, ..*self })
    }

    /// Whole-day count to another datetime (rounded toward zero).
    pub fn days_until(&self, other: &Self) -> i32 {
        algorithm::j_to_abs(other.year(), other.month(), other.day())
            - algorithm::j_to_abs(self.year(), self.month(), self.day())
    }

    /// Total seconds to another datetime (negative if `other` precedes `self`).
    pub fn seconds_until(&self, other: &Self) -> i64 {
        other.to_unix_timestamp() - self.to_unix_timestamp()
    }
}

impl fmt::Display for JalaliDateTime {
    /// `YYYY/MM/DD HH:MM:SS`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}/{:02}/{:02} {:02}:{:02}:{:02}",
            self.year(),
            self.month(),
            self.day(),
            self.hour,
            self.minute,
            self.second,
        )
    }
}

impl From<JalaliDate> for JalaliDateTime {
    /// Promote a date to a datetime at midnight.
    fn from(date: JalaliDate) -> Self {
        JalaliDateTime {
            date,
            hour: 0,
            minute: 0,
            second: 0,
            nanosecond: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_unix() {
        let ts: i64 = 1_710_892_800 + 12 * 3600 + 34 * 60 + 56;
        let dt = JalaliDateTime::from_unix_timestamp(ts).unwrap();
        assert_eq!(
            (
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second()
            ),
            (1403, 1, 1, 12, 34, 56),
        );
        assert_eq!(dt.to_unix_timestamp(), ts);
    }

    #[test]
    fn add_seconds_rolls_over() {
        let dt = JalaliDateTime::new(1403, 1, 1, 23, 59, 59).unwrap();
        let later = dt.add_seconds(2);
        assert_eq!(
            (
                later.year(),
                later.month(),
                later.day(),
                later.hour(),
                later.minute(),
                later.second()
            ),
            (1403, 1, 2, 0, 0, 1),
        );
    }

    #[test]
    fn time_validation() {
        assert!(JalaliDateTime::new(1403, 1, 1, 24, 0, 0).is_err());
        assert!(JalaliDateTime::new(1403, 1, 1, 0, 60, 0).is_err());
        assert!(JalaliDateTime::new(1403, 1, 1, 0, 0, 60).is_err());
    }

    #[test]
    fn promote_date_to_midnight() {
        let d = JalaliDate::new(1403, 1, 1).unwrap();
        let dt: JalaliDateTime = d.into();
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.to_string(), "1403/01/01 00:00:00");
    }
}
