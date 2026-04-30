//! Timezone-aware Jalali datetime (enable the `timezone` feature).

use std::fmt;

use chrono::{DateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

use crate::{Error, JalaliDate, JalaliDateTime, Weekday};

/// Timezone-aware Jalali date and time, anchored to a [`chrono_tz::Tz`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZonedJalaliDateTime {
    inner: DateTime<Tz>,
}

impl ZonedJalaliDateTime {
    /// Construct a zoned datetime from Jalali components and a timezone.
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        tz: Tz,
    ) -> Result<Self, Error> {
        let naive =
            JalaliDateTime::new(year, month, day, hour, minute, second)?.to_naive_datetime();
        let dt = tz
            .from_local_datetime(&naive)
            .single()
            .ok_or(Error::InvalidTime {
                hour,
                minute,
                second,
            })?;
        Ok(ZonedJalaliDateTime { inner: dt })
    }

    /// The current Jalali datetime in the given timezone.
    pub fn now(tz: Tz) -> Self {
        let utc = Utc::now();
        let in_tz = utc.with_timezone(&tz);
        ZonedJalaliDateTime { inner: in_tz }
    }

    /// Convert this datetime to another timezone, preserving the instant.
    pub fn with_timezone(&self, tz: Tz) -> Self {
        ZonedJalaliDateTime {
            inner: self.inner.with_timezone(&tz),
        }
    }

    /// Underlying [`chrono_tz::Tz`].
    pub fn timezone(&self) -> Tz {
        self.inner.timezone()
    }

    /// Local Jalali date.
    pub fn date(&self) -> JalaliDate {
        let local = self.inner.naive_local().date();
        JalaliDate::from_naive_date(local).expect("valid Gregorian date")
    }

    /// Local Jalali datetime (naive — no timezone attached).
    pub fn naive_local(&self) -> JalaliDateTime {
        let naive = self.inner.naive_local();
        JalaliDateTime::from_naive_datetime(naive).expect("valid Gregorian datetime")
    }

    pub fn year(&self) -> i32 {
        self.date().year()
    }
    pub fn month(&self) -> u32 {
        self.date().month()
    }
    pub fn day(&self) -> u32 {
        self.date().day()
    }
    pub fn hour(&self) -> u32 {
        self.inner.hour()
    }
    pub fn minute(&self) -> u32 {
        self.inner.minute()
    }
    pub fn second(&self) -> u32 {
        self.inner.second()
    }
    pub fn weekday(&self) -> Weekday {
        self.date().weekday()
    }

    /// Underlying chrono [`DateTime<Tz>`].
    pub fn to_datetime(&self) -> DateTime<Tz> {
        self.inner
    }

    pub fn to_unix_timestamp(&self) -> i64 {
        self.inner.timestamp()
    }
}

impl fmt::Display for ZonedJalaliDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let local = self.naive_local();
        write!(f, "{} {}", local, self.inner.format("%z (%Z)"))
    }
}

impl From<DateTime<Tz>> for ZonedJalaliDateTime {
    fn from(inner: DateTime<Tz>) -> Self {
        ZonedJalaliDateTime { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use chrono_tz::Asia::Tehran;
    use chrono_tz::Europe::London;

    #[test]
    fn construct_in_tehran() {
        let z = ZonedJalaliDateTime::new(1403, 1, 1, 12, 0, 0, Tehran).unwrap();
        assert_eq!(z.year(), 1403);
        assert_eq!(z.timezone(), Tehran);
    }

    #[test]
    fn convert_between_zones() {
        // 1403/1/1 12:00 in Tehran (UTC+3:30) = 08:30 in London (GMT) on Mar 20, 2024.
        let z = ZonedJalaliDateTime::new(1403, 1, 1, 12, 0, 0, Tehran).unwrap();
        let london = z.with_timezone(London);
        assert_eq!(london.hour(), 8);
        assert_eq!(london.minute(), 30);
    }

    #[test]
    fn from_naive_datetime_via_chrono() {
        let naive =
            NaiveDateTime::parse_from_str("2024-03-20 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let utc_dt = Utc.from_utc_datetime(&naive);
        let z: ZonedJalaliDateTime = utc_dt.with_timezone(&Tehran).into();
        assert_eq!(z.year(), 1403);
        assert_eq!(z.hour(), 15); // UTC 12:00 = Tehran 15:30
        assert_eq!(z.minute(), 30);
    }
}
