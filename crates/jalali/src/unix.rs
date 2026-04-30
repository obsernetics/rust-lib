//! Conversion between Unix timestamps and Jalali dates.
//!
//! Both directions assume UTC midnight; sub-day precision is discarded.

use crate::{algorithm, Error, JalaliDate};

/// Rata-die value for 1970-01-01 (the Unix epoch).
const UNIX_EPOCH_RD: i32 = 719163;

const SECONDS_PER_DAY: i64 = 86_400;

impl JalaliDate {
    /// Construct a Jalali date from a Unix timestamp (seconds since 1970-01-01 UTC).
    /// Sub-day precision is discarded.
    pub fn from_unix_timestamp(seconds: i64) -> Result<Self, Error> {
        let days = seconds.div_euclid(SECONDS_PER_DAY);
        let rd = (days as i32)
            .checked_add(UNIX_EPOCH_RD)
            .ok_or(Error::InvalidGregorianDate {
                year: 0,
                month: 0,
                day: 0,
            })?;
        let (gy, gm, gd) = algorithm::rata_die_to_g(rd);
        let (jy, jm, jd) = algorithm::g2j(gy, gm, gd);
        Ok(JalaliDate::new_unchecked(jy, jm, jd))
    }

    /// Unix timestamp (seconds since 1970-01-01 UTC) at this date's UTC midnight.
    pub fn to_unix_timestamp(&self) -> i64 {
        let (gy, gm, gd) = self.to_gregorian();
        let rd = algorithm::g_to_rata_die(gy, gm, gd);
        (rd - UNIX_EPOCH_RD) as i64 * SECONDS_PER_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1348_10_11() {
        let j = JalaliDate::from_unix_timestamp(0).unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1348, 10, 11));
    }

    #[test]
    fn nowruz_1403_round_trip() {
        let j = JalaliDate::new(1403, 1, 1).unwrap();
        let ts = j.to_unix_timestamp();
        // 2024-03-20 00:00:00 UTC
        assert_eq!(ts, 1_710_892_800);
        assert_eq!(JalaliDate::from_unix_timestamp(ts).unwrap(), j);
    }

    #[test]
    fn sub_day_precision_discarded() {
        let a = JalaliDate::from_unix_timestamp(1_710_892_800).unwrap();
        let b = JalaliDate::from_unix_timestamp(1_710_892_800 + 86_399).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn pre_epoch_dates() {
        // -86400 = 1969-12-31 UTC = Jalali 1348/10/10
        let j = JalaliDate::from_unix_timestamp(-86_400).unwrap();
        assert_eq!((j.year(), j.month(), j.day()), (1348, 10, 10));
    }
}
