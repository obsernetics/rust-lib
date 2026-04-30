//! Timezone-aware Jalali datetime (enable the `timezone` feature).
//!
//! Available behind the `timezone` Cargo feature, which pulls in
//! [`chrono`] and [`chrono_tz`]. [`ZonedJalaliDateTime`] internally stores a
//! `chrono::DateTime<Tz>` so all of chrono's instant arithmetic is available
//! via [`ZonedJalaliDateTime::to_datetime`]; the Jalali surface is exposed
//! through getters that present the local-wall-clock view.

use std::fmt;

use chrono::{DateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

use crate::{Error, JalaliDate, JalaliDateTime, Weekday};

/// A Jalali date and time anchored to a specific [`chrono_tz::Tz`].
///
/// Construct via [`ZonedJalaliDateTime::new`] (Jalali wall-clock components +
/// timezone) or [`ZonedJalaliDateTime::now`] (current instant, projected into
/// the requested zone). Convert between zones with
/// [`ZonedJalaliDateTime::with_timezone`] — the underlying instant is
/// preserved.
///
/// All getters (`year`, `month`, `day`, `hour`, `minute`, `second`,
/// `weekday`) return values in the **local** wall-clock view of `self`'s
/// timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ZonedJalaliDateTime {
    inner: DateTime<Tz>,
}

impl ZonedJalaliDateTime {
    /// Construct a zoned datetime from Jalali wall-clock components and a
    /// timezone.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidJalaliDate`] / [`Error::InvalidTime`] for
    ///   out-of-range Jalali or time components.
    /// - [`Error::InvalidTime`] (re-used) when the local wall-clock time does
    ///   not unambiguously map to a single instant in the timezone (e.g.
    ///   skipped or repeated due to DST).
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

    /// Convert this datetime to another timezone, preserving the underlying
    /// instant. The returned value's wall-clock components reflect `tz`.
    pub fn with_timezone(&self, tz: Tz) -> Self {
        ZonedJalaliDateTime {
            inner: self.inner.with_timezone(&tz),
        }
    }

    /// Underlying [`chrono_tz::Tz`].
    pub fn timezone(&self) -> Tz {
        self.inner.timezone()
    }

    /// The local Jalali date (year/month/day) in this datetime's timezone.
    pub fn date(&self) -> JalaliDate {
        let local = self.inner.naive_local().date();
        JalaliDate::from_naive_date(local).expect("valid Gregorian date")
    }

    /// The naive local datetime in this datetime's timezone — i.e. the
    /// wall-clock view, with the timezone information stripped.
    pub fn naive_local(&self) -> JalaliDateTime {
        let naive = self.inner.naive_local();
        JalaliDateTime::from_naive_datetime(naive).expect("valid Gregorian datetime")
    }

    /// Local Jalali year.
    pub fn year(&self) -> i32 {
        self.date().year()
    }
    /// Local Jalali month, in `1..=12`.
    pub fn month(&self) -> u32 {
        self.date().month()
    }
    /// Local Jalali day of the month.
    pub fn day(&self) -> u32 {
        self.date().day()
    }
    /// Local hour of day, in `0..=23`.
    pub fn hour(&self) -> u32 {
        self.inner.hour()
    }
    /// Local minute of hour, in `0..=59`.
    pub fn minute(&self) -> u32 {
        self.inner.minute()
    }
    /// Local second of minute, in `0..=59`.
    pub fn second(&self) -> u32 {
        self.inner.second()
    }
    /// Local Persian weekday.
    pub fn weekday(&self) -> Weekday {
        self.date().weekday()
    }

    /// The underlying chrono [`DateTime<Tz>`]. Use this to plug into the
    /// rest of the chrono ecosystem (Duration arithmetic, formatting, etc.).
    pub fn to_datetime(&self) -> DateTime<Tz> {
        self.inner
    }

    /// Unix timestamp (seconds since 1970-01-01 UTC) for this instant.
    pub fn to_unix_timestamp(&self) -> i64 {
        self.inner.timestamp()
    }
}

impl fmt::Display for ZonedJalaliDateTime {
    /// Formats as `YYYY/MM/DD HH:MM:SS ±HHMM (TZ)`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let local = self.naive_local();
        write!(f, "{} {}", local, self.inner.format("%z (%Z)"))
    }
}

impl From<DateTime<Tz>> for ZonedJalaliDateTime {
    /// Wrap a `chrono::DateTime<Tz>` as a `ZonedJalaliDateTime`. Cheap — no
    /// conversion is performed up-front; the Jalali view is computed lazily
    /// in the getters.
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
