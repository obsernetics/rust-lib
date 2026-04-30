//! `strftime`-style format and parse for [`JalaliDate`] and [`JalaliDateTime`].
//!
//! # Tokens
//!
//! | token | meaning                                        |
//! |-------|------------------------------------------------|
//! | `%Y`  | 4-digit Jalali year                            |
//! | `%y`  | last 2 digits of year                          |
//! | `%m`  | 2-digit month (01-12)                          |
//! | `%-m` | month without padding                          |
//! | `%d`  | 2-digit day (01-31)                            |
//! | `%-d` | day without padding                            |
//! | `%e`  | day, space-padded                              |
//! | `%j`  | 3-digit day of year (001-366)                  |
//! | `%B`  | full Persian month name (e.g. `فروردین`)       |
//! | `%b`  | abbreviated Persian month (first 3 letters)    |
//! | `%A`  | full Persian weekday (e.g. `چهارشنبه`)         |
//! | `%a`  | single-character Persian weekday (e.g. `چ`)   |
//! | `%K`  | Persian season name (`بهار`, `تابستان`, …)     |
//! | `%H`  | 2-digit hour (00-23)                           |
//! | `%M`  | 2-digit minute                                 |
//! | `%S`  | 2-digit second                                 |
//! | `%T`  | `%H:%M:%S`                                     |
//! | `%p`  | uppercase AM/PM                                |
//! | `%P`  | lowercase am/pm                                |
//! | `%%`  | literal `%`                                    |

use crate::{digits, Error, JalaliDate, JalaliDateTime, PERSIAN_MONTHS};

impl JalaliDate {
    /// Format using `strftime`-style tokens. See the [crate-level docs][crate].
    pub fn format(&self, fmt: &str) -> String {
        format_internal(fmt, Some(self), None)
    }

    /// Parse a date from `input` using `fmt`. Time tokens in `fmt` are ignored
    /// (only the date components are extracted).
    pub fn parse_format(input: &str, fmt: &str) -> Result<Self, Error> {
        let parts = parse_internal(input, fmt)?;
        let year = parts.year.ok_or_else(|| Error::ParseMismatch {
            expected: "year".into(),
            found: input.to_string(),
        })?;
        let month = parts.month.ok_or_else(|| Error::ParseMismatch {
            expected: "month".into(),
            found: input.to_string(),
        })?;
        let day = parts.day.ok_or_else(|| Error::ParseMismatch {
            expected: "day".into(),
            found: input.to_string(),
        })?;
        JalaliDate::new(year, month, day)
    }
}

impl JalaliDateTime {
    /// Format using `strftime`-style tokens.
    pub fn format(&self, fmt: &str) -> String {
        format_internal(fmt, Some(&self.date()), Some(self))
    }

    /// Parse a datetime from `input` using `fmt`.
    pub fn parse_format(input: &str, fmt: &str) -> Result<Self, Error> {
        let parts = parse_internal(input, fmt)?;
        let year = parts.year.ok_or_else(|| Error::ParseMismatch {
            expected: "year".into(),
            found: input.to_string(),
        })?;
        let month = parts.month.ok_or_else(|| Error::ParseMismatch {
            expected: "month".into(),
            found: input.to_string(),
        })?;
        let day = parts.day.ok_or_else(|| Error::ParseMismatch {
            expected: "day".into(),
            found: input.to_string(),
        })?;
        JalaliDateTime::new(
            year,
            month,
            day,
            parts.hour.unwrap_or(0),
            parts.minute.unwrap_or(0),
            parts.second.unwrap_or(0),
        )
    }
}

fn format_internal(fmt: &str, date: Option<&JalaliDate>, dt: Option<&JalaliDateTime>) -> String {
    let mut out = String::with_capacity(fmt.len() + 16);
    let mut iter = fmt.chars().peekable();
    while let Some(c) = iter.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        let mut pad = true;
        if iter.peek() == Some(&'-') {
            pad = false;
            iter.next();
        }
        let token = match iter.next() {
            Some(t) => t,
            None => {
                out.push('%');
                break;
            }
        };
        emit_token(token, pad, date, dt, &mut out);
    }
    out
}

fn emit_token(
    token: char,
    pad: bool,
    date: Option<&JalaliDate>,
    dt: Option<&JalaliDateTime>,
    out: &mut String,
) {
    use std::fmt::Write;
    match token {
        '%' => out.push('%'),
        'Y' => {
            if let Some(d) = date {
                let _ = write!(out, "{:04}", d.year());
            }
        }
        'y' => {
            if let Some(d) = date {
                let _ = write!(out, "{:02}", d.year().rem_euclid(100));
            }
        }
        'm' => {
            if let Some(d) = date {
                if pad {
                    let _ = write!(out, "{:02}", d.month());
                } else {
                    let _ = write!(out, "{}", d.month());
                }
            }
        }
        'd' => {
            if let Some(d) = date {
                if pad {
                    let _ = write!(out, "{:02}", d.day());
                } else {
                    let _ = write!(out, "{}", d.day());
                }
            }
        }
        'e' => {
            if let Some(d) = date {
                let _ = write!(out, "{:>2}", d.day());
            }
        }
        'j' => {
            if let Some(d) = date {
                let _ = write!(out, "{:03}", d.ordinal());
            }
        }
        'B' => {
            if let Some(d) = date {
                out.push_str(PERSIAN_MONTHS[(d.month() - 1) as usize]);
            }
        }
        'b' => {
            if let Some(d) = date {
                let name = PERSIAN_MONTHS[(d.month() - 1) as usize];
                out.extend(name.chars().take(3));
            }
        }
        'A' => {
            if let Some(d) = date {
                out.push_str(d.weekday().persian_name());
            }
        }
        'a' => {
            if let Some(d) = date {
                out.push_str(d.weekday().persian_abbreviation());
            }
        }
        'K' => {
            if let Some(d) = date {
                out.push_str(d.season().persian_name());
            }
        }
        'H' => {
            if let Some(t) = dt {
                let _ = write!(out, "{:02}", t.hour());
            }
        }
        'M' => {
            if let Some(t) = dt {
                let _ = write!(out, "{:02}", t.minute());
            }
        }
        'S' => {
            if let Some(t) = dt {
                let _ = write!(out, "{:02}", t.second());
            }
        }
        'T' => {
            if let Some(t) = dt {
                let _ = write!(out, "{:02}:{:02}:{:02}", t.hour(), t.minute(), t.second());
            }
        }
        'p' => {
            if let Some(t) = dt {
                out.push_str(if t.hour() < 12 { "AM" } else { "PM" });
            }
        }
        'P' => {
            if let Some(t) = dt {
                out.push_str(if t.hour() < 12 { "am" } else { "pm" });
            }
        }
        other => {
            // Unknown — emit verbatim so callers see what was wrong.
            out.push('%');
            out.push(other);
        }
    }
}

#[derive(Default, Debug)]
struct Parts {
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    hour: Option<u32>,
    minute: Option<u32>,
    second: Option<u32>,
}

fn parse_internal(input: &str, fmt: &str) -> Result<Parts, Error> {
    let normalized = digits::to_latin(input);
    let mut parts = Parts::default();
    let mut input_iter = normalized.chars().peekable();
    let mut fmt_iter = fmt.chars().peekable();

    while let Some(fc) = fmt_iter.next() {
        if fc != '%' {
            match input_iter.next() {
                Some(ic) if ic == fc => continue,
                Some(ic) => {
                    return Err(Error::ParseMismatch {
                        expected: fc.to_string(),
                        found: ic.to_string(),
                    });
                }
                None => {
                    return Err(Error::ParseMismatch {
                        expected: fc.to_string(),
                        found: "<end of input>".into(),
                    });
                }
            }
        }
        // Consume optional '-' (width modifier — accept both padded and not).
        if fmt_iter.peek() == Some(&'-') {
            fmt_iter.next();
        }
        let token = fmt_iter.next().ok_or(Error::UnknownFormatToken('%'))?;
        match token {
            '%' => match input_iter.next() {
                Some('%') => {}
                Some(ic) => {
                    return Err(Error::ParseMismatch {
                        expected: "%".into(),
                        found: ic.to_string(),
                    });
                }
                None => {
                    return Err(Error::ParseMismatch {
                        expected: "%".into(),
                        found: "<end of input>".into(),
                    });
                }
            },
            'Y' => parts.year = Some(read_int(&mut input_iter, 4) as i32),
            'y' => {
                let y = read_int(&mut input_iter, 2) as i32;
                parts.year = Some(if y < 70 { 1400 + y } else { 1300 + y });
            }
            'm' => parts.month = Some(read_int_var(&mut input_iter, 2) as u32),
            'd' => parts.day = Some(read_int_var(&mut input_iter, 2) as u32),
            'e' => {
                while input_iter.peek() == Some(&' ') {
                    input_iter.next();
                }
                parts.day = Some(read_int_var(&mut input_iter, 2) as u32);
            }
            'j' => {
                let doy = read_int(&mut input_iter, 3) as u32;
                let (m, d) = doy_to_month_day(doy);
                parts.month = Some(m);
                parts.day = Some(d);
            }
            'B' => {
                let name = read_persian_word(&mut input_iter);
                let m = month_from_name(&name).ok_or_else(|| Error::ParseMismatch {
                    expected: "Persian month name".into(),
                    found: name,
                })?;
                parts.month = Some(m);
            }
            'b' => {
                let name = read_persian_word(&mut input_iter);
                let m = month_from_name(&name)
                    .or_else(|| {
                        PERSIAN_MONTHS
                            .iter()
                            .position(|full| full.starts_with(name.as_str()))
                            .map(|i| (i as u32) + 1)
                    })
                    .ok_or_else(|| Error::ParseMismatch {
                        expected: "Persian month name".into(),
                        found: name,
                    })?;
                parts.month = Some(m);
            }
            'A' | 'a' | 'K' => {
                // Informational only — consume the persian word and move on.
                let _ = read_persian_word(&mut input_iter);
            }
            'H' => parts.hour = Some(read_int_var(&mut input_iter, 2) as u32),
            'M' => parts.minute = Some(read_int_var(&mut input_iter, 2) as u32),
            'S' => parts.second = Some(read_int_var(&mut input_iter, 2) as u32),
            'T' => {
                parts.hour = Some(read_int_var(&mut input_iter, 2) as u32);
                expect_char(&mut input_iter, ':')?;
                parts.minute = Some(read_int_var(&mut input_iter, 2) as u32);
                expect_char(&mut input_iter, ':')?;
                parts.second = Some(read_int_var(&mut input_iter, 2) as u32);
            }
            'p' | 'P' => {
                let mut s = String::new();
                while let Some(c) = input_iter.peek() {
                    if c.is_ascii_alphabetic() {
                        s.push(*c);
                        input_iter.next();
                        if s.len() == 2 {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                let lower = s.to_ascii_lowercase();
                if let Some(h) = parts.hour {
                    if lower == "pm" && h < 12 {
                        parts.hour = Some(h + 12);
                    } else if lower == "am" && h == 12 {
                        parts.hour = Some(0);
                    }
                }
            }
            other => return Err(Error::UnknownFormatToken(other)),
        }
    }
    Ok(parts)
}

fn read_int(iter: &mut std::iter::Peekable<std::str::Chars>, max_chars: usize) -> i64 {
    let mut s = String::new();
    while s.len() < max_chars {
        match iter.peek() {
            Some(c) if c.is_ascii_digit() => {
                s.push(*c);
                iter.next();
            }
            _ => break,
        }
    }
    s.parse().unwrap_or(0)
}

/// Like [`read_int`], but only consumes as many digits as are present (no padding required).
fn read_int_var(iter: &mut std::iter::Peekable<std::str::Chars>, max_chars: usize) -> i64 {
    read_int(iter, max_chars)
}

fn expect_char(iter: &mut std::iter::Peekable<std::str::Chars>, c: char) -> Result<(), Error> {
    match iter.next() {
        Some(actual) if actual == c => Ok(()),
        Some(actual) => Err(Error::ParseMismatch {
            expected: c.to_string(),
            found: actual.to_string(),
        }),
        None => Err(Error::ParseMismatch {
            expected: c.to_string(),
            found: "<end of input>".into(),
        }),
    }
}

fn read_persian_word(iter: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut s = String::new();
    while let Some(c) = iter.peek() {
        if is_persian_letter(*c) {
            s.push(*c);
            iter.next();
        } else {
            break;
        }
    }
    s
}

fn is_persian_letter(c: char) -> bool {
    matches!(c, '\u{0600}'..='\u{06FF}' | '\u{200C}' | '\u{200D}')
}

fn month_from_name(name: &str) -> Option<u32> {
    PERSIAN_MONTHS
        .iter()
        .position(|m| *m == name)
        .map(|i| (i as u32) + 1)
}

fn doy_to_month_day(doy: u32) -> (u32, u32) {
    if doy <= 186 {
        let m = (doy - 1) / 31 + 1;
        let d = (doy - 1) % 31 + 1;
        (m, d)
    } else {
        let m = 7 + (doy - 187) / 30;
        let d = (doy - 187) % 30 + 1;
        (m, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_format_basic() {
        let d = JalaliDate::new(1403, 1, 1).unwrap();
        assert_eq!(d.format("%Y/%m/%d"), "1403/01/01");
        assert_eq!(d.format("%-d %B %Y"), "1 فروردین 1403");
        assert_eq!(d.format("%A"), "چهارشنبه");
        assert_eq!(d.format("%K"), "بهار");
        assert_eq!(d.format("%j"), "001");
    }

    #[test]
    fn datetime_format() {
        let dt = JalaliDateTime::new(1403, 1, 1, 7, 5, 9).unwrap();
        assert_eq!(dt.format("%Y-%m-%d %T"), "1403-01-01 07:05:09");
        assert_eq!(dt.format("%H:%M %p"), "07:05 AM");
        let pm = JalaliDateTime::new(1403, 1, 1, 14, 0, 0).unwrap();
        assert_eq!(pm.format("%H:%M %p"), "14:00 PM");
    }

    #[test]
    fn date_parse_basic() {
        let d = JalaliDate::parse_format("1403/01/01", "%Y/%m/%d").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1403, 1, 1));

        let d = JalaliDate::parse_format("1 فروردین 1403", "%-d %B %Y").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1403, 1, 1));

        // Persian digits also parse.
        let d = JalaliDate::parse_format("۱۴۰۳-۰۱-۰۱", "%Y-%m-%d").unwrap();
        assert_eq!((d.year(), d.month(), d.day()), (1403, 1, 1));
    }

    #[test]
    fn datetime_parse_basic() {
        let dt = JalaliDateTime::parse_format("1403/01/01 12:34:56", "%Y/%m/%d %T").unwrap();
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
    }

    #[test]
    fn parse_handles_pm() {
        let dt = JalaliDateTime::parse_format("1403/01/01 03:00 PM", "%Y/%m/%d %H:%M %p").unwrap();
        assert_eq!(dt.hour(), 15);
        let dt = JalaliDateTime::parse_format("1403/01/01 12:00 AM", "%Y/%m/%d %H:%M %p").unwrap();
        assert_eq!(dt.hour(), 0);
    }
}
