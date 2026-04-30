//! `today()` / `now()` constructors backed by the system clock.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::JalaliDate;

impl JalaliDate {
    /// The current Jalali date in UTC.
    ///
    /// For local-timezone results enable the `timezone` feature and use
    /// [`crate::ZonedJalaliDateTime::now`].
    pub fn today() -> Self {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        JalaliDate::from_unix_timestamp(secs).expect("system clock returned a valid timestamp")
    }
}

/// Whole seconds since the Unix epoch (UTC), from the system clock.
pub(crate) fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
