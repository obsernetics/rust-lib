//! Conversion primitives between Jalali and Gregorian calendars.
//!
//! Uses the Pournader-Toossi algorithm for J↔G conversion (accurate for
//! Jalali years 1..3177) and the standard 400-year-cycle Gregorian
//! algorithm for rata-die based arithmetic.

const G_MONTH_OFFSET: [i32; 13] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365];

pub fn is_gregorian_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

pub fn days_in_gregorian_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_gregorian_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

pub fn is_valid_gregorian(y: i32, m: u32, d: u32) -> bool {
    (1..=12).contains(&m) && d >= 1 && d <= days_in_gregorian_month(y, m)
}

/// Whether the given Jalali year is a leap year.
///
/// Derived from the cumulative leap-day count `(Y/33)*8 + ((Y%33)+3)/4`
/// embedded in the Pournader algorithm: a year is leap iff that count
/// increases by 1 when stepping from `jy` to `jy + 1`.
pub fn is_leap_year(jy: i32) -> bool {
    let y = jy + 1595;
    let yn = y + 1;
    let prev = (y / 33) * 8 + ((y % 33) + 3) / 4;
    let next = (yn / 33) * 8 + ((yn % 33) + 3) / 4;
    next - prev == 1
}

pub fn days_in_month(jy: i32, m: u32) -> u32 {
    match m {
        1..=6 => 31,
        7..=11 => 30,
        12 => {
            if is_leap_year(jy) {
                30
            } else {
                29
            }
        }
        _ => 0,
    }
}

/// Gregorian → Jalali (Pournader-Toossi).
pub fn g2j(gy: i32, gm: u32, gd: u32) -> (i32, u32, u32) {
    let gy2 = if gm > 2 { gy + 1 } else { gy };
    let mut days = 355666 + 365 * gy + (gy2 + 3) / 4 - (gy2 + 99) / 100
        + (gy2 + 399) / 400
        + gd as i32
        + G_MONTH_OFFSET[(gm - 1) as usize];

    let mut jy = -1595 + 33 * (days / 12053);
    days %= 12053;
    jy += 4 * (days / 1461);
    days %= 1461;
    if days > 365 {
        jy += (days - 1) / 365;
        days = (days - 1) % 365;
    }
    if days < 186 {
        (jy, 1 + (days / 31) as u32, 1 + (days % 31) as u32)
    } else {
        (
            jy,
            7 + ((days - 186) / 30) as u32,
            1 + ((days - 186) % 30) as u32,
        )
    }
}

/// Jalali → Gregorian (Pournader-Toossi).
pub fn j2g(jy: i32, jm: u32, jd: u32) -> (i32, u32, u32) {
    let y = jy + 1595;
    let mut days = -355668
        + 365 * y
        + (y / 33) * 8
        + ((y % 33) + 3) / 4
        + jd as i32
        + if jm < 7 {
            (jm as i32 - 1) * 31
        } else {
            (jm as i32 - 7) * 30 + 186
        };

    let mut gy = 400 * (days / 146097);
    days %= 146097;
    if days > 36524 {
        days -= 1;
        gy += 100 * (days / 36524);
        days %= 36524;
        if days >= 365 {
            days += 1;
        }
    }
    gy += 4 * (days / 1461);
    days %= 1461;
    if days > 365 {
        gy += (days - 1) / 365;
        days = (days - 1) % 365;
    }
    let mut gd = days + 1;
    let dim: [i32; 13] = if is_gregorian_leap(gy) {
        [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut gm: u32 = 1;
    while gm <= 12 && gd > dim[gm as usize] {
        gd -= dim[gm as usize];
        gm += 1;
    }
    (gy, gm, gd as u32)
}

/// Gregorian rata die: day count where RD 1 = 1 CE Jan 1 (Monday).
pub fn g_to_rata_die(y: i32, m: u32, d: u32) -> i32 {
    let yp = y - 1;
    365 * yp + yp.div_euclid(4) - yp.div_euclid(100)
        + yp.div_euclid(400)
        + G_MONTH_OFFSET[(m - 1) as usize]
        + (if m > 2 && is_gregorian_leap(y) { 1 } else { 0 })
        + d as i32
}

/// Inverse of `g_to_rata_die` using the standard 400-year cycle decomposition.
pub fn rata_die_to_g(rd: i32) -> (i32, u32, u32) {
    let n = rd - 1;
    let n400 = n.div_euclid(146097);
    let d1 = n.rem_euclid(146097);
    let n100 = d1 / 36524;
    let d2 = d1 - n100 * 36524;
    let n4 = d2 / 1461;
    let d3 = d2 - n4 * 1461;
    let n1 = d3 / 365;
    let d4 = d3 - n1 * 365;

    let year_zero_indexed = 400 * n400 + 100 * n100 + 4 * n4 + n1;
    if n100 == 4 || n1 == 4 {
        return (year_zero_indexed, 12, 31);
    }
    let y = year_zero_indexed + 1;
    let mut doy = d4 + 1;
    let dim: [i32; 13] = if is_gregorian_leap(y) {
        [0, 31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut m: u32 = 1;
    while m <= 12 && doy > dim[m as usize] {
        doy -= dim[m as usize];
        m += 1;
    }
    (y, m, doy as u32)
}

/// Rata-die day count for a Jalali date (via Gregorian).
pub fn j_to_abs(jy: i32, jm: u32, jd: u32) -> i32 {
    let (gy, gm, gd) = j2g(jy, jm, jd);
    g_to_rata_die(gy, gm, gd)
}

/// Inverse of [`j_to_abs`].
pub fn abs_to_j(rd: i32) -> (i32, u32, u32) {
    let (gy, gm, gd) = rata_die_to_g(rd);
    g2j(gy, gm, gd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rata_die_round_trips() {
        for (y, m, d) in [
            (1, 1, 1),
            (1979, 2, 11),
            (2000, 1, 1),
            (2020, 12, 31),
            (2024, 2, 29),
            (2024, 3, 20),
            (2100, 3, 1),
        ] {
            let rd = g_to_rata_die(y, m, d);
            assert_eq!(rata_die_to_g(rd), (y, m, d), "rd={rd} for {y}-{m}-{d}");
        }
    }

    #[test]
    fn rd_for_known_dates() {
        // 1 CE Jan 1 (Monday) is RD 1.
        assert_eq!(g_to_rata_die(1, 1, 1), 1);
        // 2024-03-20 (Wed): RD 738965.
        assert_eq!(g_to_rata_die(2024, 3, 20), 738965);
    }
}
