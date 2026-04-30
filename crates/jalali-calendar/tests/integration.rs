//! Cross-cutting integration tests that exercise multiple modules at once.

use jalali_calendar::{is_leap_year, JalaliDate, JalaliDateTime, Season, Weekday};

#[test]
fn full_year_round_trip_consistency() {
    // Walk every day of 1403 (a leap year) through Gregorian and back.
    for ordinal in 1..=366 {
        let d = day_n_of_year(1403, ordinal);
        let (gy, gm, gd) = d.to_gregorian();
        let back = JalaliDate::from_gregorian(gy, gm, gd).unwrap();
        assert_eq!(d, back, "round trip failed at 1403 doy={ordinal}");
        assert_eq!(d.ordinal(), ordinal);
    }
}

fn day_n_of_year(year: i32, ordinal: u32) -> JalaliDate {
    let first = JalaliDate::new(year, 1, 1).unwrap();
    first.add_days(ordinal as i32 - 1)
}

#[test]
fn add_days_consistent_with_iteration() {
    let start = JalaliDate::new(1402, 6, 15).unwrap();
    let mut d = start;
    for i in 0..1000 {
        let jumped = start.add_days(i);
        assert_eq!(d, jumped, "drift at i={i}");
        d = d.add_days(1);
    }
}

#[test]
fn weekday_advances_one_day_at_a_time() {
    let mut d = JalaliDate::new(1402, 1, 1).unwrap();
    let mut prev = d.weekday();
    for _ in 0..400 {
        d = d.add_days(1);
        let wd = d.weekday();
        let expected = match prev {
            Weekday::Saturday => Weekday::Sunday,
            Weekday::Sunday => Weekday::Monday,
            Weekday::Monday => Weekday::Tuesday,
            Weekday::Tuesday => Weekday::Wednesday,
            Weekday::Wednesday => Weekday::Thursday,
            Weekday::Thursday => Weekday::Friday,
            Weekday::Friday => Weekday::Saturday,
        };
        assert_eq!(wd, expected);
        prev = wd;
    }
}

#[test]
fn leap_year_pattern_in_33_year_cycle() {
    // The Pournader 33-year cycle has 8 leap years.
    let mut count = 0;
    for jy in 1400..1433 {
        if is_leap_year(jy) {
            count += 1;
        }
    }
    assert_eq!(count, 8);
}

#[test]
fn seasons_align_with_months() {
    for m in 1..=3u32 {
        assert_eq!(
            JalaliDate::new(1403, m, 1).unwrap().season(),
            Season::Spring
        );
    }
    for m in 4..=6u32 {
        assert_eq!(
            JalaliDate::new(1403, m, 1).unwrap().season(),
            Season::Summer
        );
    }
    for m in 7..=9u32 {
        assert_eq!(
            JalaliDate::new(1403, m, 1).unwrap().season(),
            Season::Autumn
        );
    }
    for m in 10..=12u32 {
        assert_eq!(
            JalaliDate::new(1403, m, 1).unwrap().season(),
            Season::Winter
        );
    }
}

#[test]
fn add_months_clamps_day_to_target_month_length() {
    // 1403/6/31 + 1 month = 1403/7/30 (mehr has 30 days)
    let d = JalaliDate::new(1403, 6, 31).unwrap();
    assert_eq!(d.add_months(1), JalaliDate::new(1403, 7, 30).unwrap());

    // 1403/1/31 + 11 months = 1403/12/30 (1403 is leap, esfand = 30)
    let d = JalaliDate::new(1403, 1, 31).unwrap();
    assert_eq!(d.add_months(11), JalaliDate::new(1403, 12, 30).unwrap());

    // 1404/1/31 + 11 months = 1404/12/29 (1404 is not leap)
    let d = JalaliDate::new(1404, 1, 31).unwrap();
    assert_eq!(d.add_months(11), JalaliDate::new(1404, 12, 29).unwrap());

    // Negative months crosses year boundary.
    let d = JalaliDate::new(1403, 1, 1).unwrap();
    assert_eq!(d.add_months(-1), JalaliDate::new(1402, 12, 1).unwrap());
}

#[test]
fn add_years_clamps_esfand() {
    // 1403/12/30 + 1 year = 1404/12/29 (because 1404 not leap)
    let d = JalaliDate::new(1403, 12, 30).unwrap();
    assert_eq!(d.add_years(1), JalaliDate::new(1404, 12, 29).unwrap());

    // 1403/12/30 - 1 year = 1402/12/29
    assert_eq!(d.add_years(-1), JalaliDate::new(1402, 12, 29).unwrap());
}

#[test]
fn with_methods() {
    let d = JalaliDate::new(1403, 6, 31).unwrap();
    // with_month should clamp.
    assert_eq!(
        d.with_month(7).unwrap(),
        JalaliDate::new(1403, 7, 30).unwrap()
    );
    // with_year clamping.
    let leap = JalaliDate::new(1403, 12, 30).unwrap();
    assert_eq!(
        leap.with_year(1404).unwrap(),
        JalaliDate::new(1404, 12, 29).unwrap()
    );
    // with_day errors when out of range.
    assert!(d.with_day(32).is_err());
    // with_month errors on invalid month.
    assert!(d.with_month(13).is_err());
}

#[test]
fn first_last_day_helpers() {
    let d = JalaliDate::new(1403, 5, 17).unwrap();
    assert_eq!(d.first_day_of_month(), JalaliDate::new(1403, 5, 1).unwrap());
    assert_eq!(d.last_day_of_month(), JalaliDate::new(1403, 5, 31).unwrap());
    assert_eq!(d.first_day_of_year(), JalaliDate::new(1403, 1, 1).unwrap());
    assert_eq!(d.last_day_of_year(), JalaliDate::new(1403, 12, 30).unwrap()); // leap

    let d = JalaliDate::new(1404, 5, 17).unwrap();
    assert_eq!(d.last_day_of_year(), JalaliDate::new(1404, 12, 29).unwrap()); // non-leap

    // Season helpers.
    let summer_day = JalaliDate::new(1403, 5, 17).unwrap();
    assert_eq!(
        summer_day.first_day_of_season(),
        JalaliDate::new(1403, 4, 1).unwrap()
    );
    assert_eq!(
        summer_day.last_day_of_season(),
        JalaliDate::new(1403, 6, 31).unwrap()
    );

    let autumn_day = JalaliDate::new(1403, 9, 1).unwrap();
    assert_eq!(
        autumn_day.last_day_of_season(),
        JalaliDate::new(1403, 9, 30).unwrap()
    );
}

#[test]
fn week_of_year_progression() {
    // Day 1 is week 1.
    let first = JalaliDate::new(1403, 1, 1).unwrap();
    assert_eq!(first.week_of_year(), 1);
    // After 6 days week is still 1 (Wednesday + 6 = Tuesday next week).
    // 1403/1/1 is Wednesday (offset 4); 7-4 = 3 days until next Saturday.
    // So week 2 starts at day 4.
    let day7 = JalaliDate::new(1403, 1, 7).unwrap();
    let week_of_day7 = day7.week_of_year();
    assert!(week_of_day7 >= 1);
    // Day 366 (1403 leap): week is reasonable bound.
    let last = JalaliDate::new(1403, 12, 30).unwrap();
    let w = last.week_of_year();
    assert!((52..=54).contains(&w), "week was {w}");
}

#[test]
fn datetime_seconds_arithmetic() {
    let dt = JalaliDateTime::new(1403, 1, 1, 0, 0, 0).unwrap();
    let plus_day = dt.add_seconds(86_400);
    assert_eq!(plus_day.day(), 2);
    assert_eq!(plus_day.hour(), 0);
    let plus_year = dt.add_seconds(86_400 * 366);
    assert_eq!(plus_year.year(), 1404);
    let minus_one = dt.add_seconds(-1);
    assert_eq!(
        (minus_one.year(), minus_one.month(), minus_one.day()),
        (1402, 12, 29)
    );
    assert_eq!(
        (minus_one.hour(), minus_one.minute(), minus_one.second()),
        (23, 59, 59)
    );
}

#[test]
fn datetime_with_methods() {
    let dt = JalaliDateTime::new(1403, 1, 1, 0, 0, 0).unwrap();
    assert_eq!(dt.with_hour(15).unwrap().hour(), 15);
    assert!(dt.with_hour(24).is_err());
    assert_eq!(dt.with_minute(30).unwrap().minute(), 30);
    assert!(dt.with_minute(60).is_err());
    assert_eq!(dt.with_second(45).unwrap().second(), 45);
    assert!(dt.with_second(60).is_err());
    let other = dt.with_time(23, 59, 59).unwrap();
    assert_eq!(dt.seconds_until(&other), 86_399);
}

#[test]
fn format_extensive() {
    let dt = JalaliDateTime::new(1403, 1, 1, 7, 5, 9).unwrap();
    assert_eq!(dt.format("%Y/%m/%d %T %p"), "1403/01/01 07:05:09 AM");
    assert_eq!(dt.format("%-d %B %y"), "1 فروردین 03");
    assert_eq!(dt.format("%a %A %K"), "چ چهارشنبه بهار");
    assert_eq!(dt.format("%j"), "001");
    assert_eq!(dt.format("100%% literal"), "100% literal");
    assert_eq!(dt.format("%e"), " 1");

    let pm = JalaliDateTime::new(1403, 1, 1, 23, 0, 0).unwrap();
    assert_eq!(pm.format("%p %P"), "PM pm");
}

#[test]
fn parse_format_supports_persian_month_names() {
    let d = JalaliDate::parse_format("روز 1 فروردین 1403", "روز %-d %B %Y").unwrap();
    assert_eq!((d.year(), d.month(), d.day()), (1403, 1, 1));
}

#[test]
fn parse_doy_token() {
    let d = JalaliDate::parse_format("1403-187", "%Y-%j").unwrap();
    // doy 187 = first day of month 7
    assert_eq!((d.month(), d.day()), (7, 1));
}

#[test]
fn unknown_format_token_errors_on_parse() {
    assert!(JalaliDate::parse_format("anything", "%Z").is_err());
}

#[test]
fn parse_mismatch_errors() {
    assert!(JalaliDate::parse_format("1403-01-01", "%Y/%m/%d").is_err());
}

#[cfg(feature = "serde")]
#[test]
fn serde_in_struct() {
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct Holiday {
        name: String,
        date: JalaliDate,
        starts_at: JalaliDateTime,
    }

    let h = Holiday {
        name: "Nowruz".into(),
        date: JalaliDate::new(1403, 1, 1).unwrap(),
        starts_at: JalaliDateTime::new(1403, 1, 1, 0, 0, 0).unwrap(),
    };
    let s = serde_json::to_string(&h).unwrap();
    let back: Holiday = serde_json::from_str(&s).unwrap();
    assert_eq!(h, back);
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_naive_round_trip_many_years() {
    use chrono::{Datelike, NaiveDate};
    for year in 1990..2050 {
        for (m, d) in [(1, 1), (3, 21), (6, 15), (12, 31)] {
            let nd = NaiveDate::from_ymd_opt(year, m, d).unwrap();
            let j: JalaliDate = nd.try_into().unwrap();
            let back: NaiveDate = j.into();
            assert_eq!(nd, back, "round trip failed at {year}-{m}-{d}");
            // Sanity: weekdays match.
            let nd_wd = nd.weekday();
            let nd_idx = nd_wd.num_days_from_sunday(); // Sunday=0
                                                       // Jalali Sat=0, Sun=1, ..., Fri=6 -> sunday-based: Sun=0 = Jalali 1, ...
            let expected_sat0 = (nd_idx + 1) % 7;
            assert_eq!(j.weekday().num_days_from_saturday(), expected_sat0);
        }
    }
}
