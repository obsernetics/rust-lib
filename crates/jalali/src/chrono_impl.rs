//! Interop with the [`chrono`] crate (enable the `chrono` feature).

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use crate::{Error, JalaliDate, JalaliDateTime};

impl JalaliDate {
    /// Convert from [`chrono::NaiveDate`].
    pub fn from_naive_date(d: NaiveDate) -> Result<Self, Error> {
        JalaliDate::from_gregorian(d.year(), d.month(), d.day())
    }

    /// Convert to [`chrono::NaiveDate`].
    pub fn to_naive_date(&self) -> NaiveDate {
        let (gy, gm, gd) = self.to_gregorian();
        NaiveDate::from_ymd_opt(gy, gm, gd).expect("Jalali->Gregorian produced a valid date")
    }
}

impl TryFrom<NaiveDate> for JalaliDate {
    type Error = Error;
    fn try_from(d: NaiveDate) -> Result<Self, Error> {
        JalaliDate::from_naive_date(d)
    }
}

impl From<JalaliDate> for NaiveDate {
    fn from(j: JalaliDate) -> Self {
        j.to_naive_date()
    }
}

impl JalaliDateTime {
    /// Convert from [`chrono::NaiveDateTime`] (Gregorian wall-clock).
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGregorianDate`] / [`Error::InvalidTime`] only
    /// for components that chrono itself would never produce; in practice
    /// this is infallible for any value chrono hands you.
    pub fn from_naive_datetime(dt: NaiveDateTime) -> Result<Self, Error> {
        let date = JalaliDate::from_naive_date(dt.date())?;
        JalaliDateTime::with_nanos(
            date.year(),
            date.month(),
            date.day(),
            dt.hour(),
            dt.minute(),
            dt.second(),
            dt.nanosecond(),
        )
    }

    /// Convert to [`chrono::NaiveDateTime`].
    ///
    /// Preserves nanosecond precision.
    pub fn to_naive_datetime(&self) -> NaiveDateTime {
        let date = self.date().to_naive_date();
        let time = NaiveTime::from_hms_nano_opt(
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        )
        .expect("validated time fits in NaiveTime");
        NaiveDateTime::new(date, time)
    }
}

impl TryFrom<NaiveDateTime> for JalaliDateTime {
    type Error = Error;
    fn try_from(dt: NaiveDateTime) -> Result<Self, Error> {
        JalaliDateTime::from_naive_datetime(dt)
    }
}

impl From<JalaliDateTime> for NaiveDateTime {
    fn from(dt: JalaliDateTime) -> Self {
        dt.to_naive_datetime()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn naive_date_round_trip() {
        let nd = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        let j: JalaliDate = nd.try_into().unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1403, 1, 1));
        let back: NaiveDate = j.into();
        assert_eq!(back, nd);
    }

    #[test]
    fn naive_datetime_round_trip() {
        let ndt = NaiveDate::from_ymd_opt(2024, 3, 20)
            .unwrap()
            .and_hms_opt(7, 8, 9)
            .unwrap();
        let dt = JalaliDateTime::from_naive_datetime(ndt).unwrap();
        assert_eq!(dt.year(), 1403);
        assert_eq!(dt.hour(), 7);
        assert_eq!(dt.to_naive_datetime(), ndt);
    }
}
