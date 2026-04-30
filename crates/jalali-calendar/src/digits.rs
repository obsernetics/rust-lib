//! Conversion between Latin (ASCII), Persian (Farsi), and Eastern-Arabic digits.
//!
//! Useful when accepting user input from Persian-language UIs where digits may
//! arrive in any of the three scripts.

const LATIN: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const PERSIAN: [char; 10] = ['۰', '۱', '۲', '۳', '۴', '۵', '۶', '۷', '۸', '۹'];
const ARABIC: [char; 10] = ['٠', '١', '٢', '٣', '٤', '٥', '٦', '٧', '٨', '٩'];

fn map_digit(c: char, from: &[char; 10], to: &[char; 10]) -> char {
    from.iter()
        .position(|&d| d == c)
        .map(|i| to[i])
        .unwrap_or(c)
}

/// Convert any Persian or Arabic digits in `s` to ASCII Latin digits.
pub fn to_latin(s: &str) -> String {
    s.chars()
        .map(|c| {
            if PERSIAN.contains(&c) {
                map_digit(c, &PERSIAN, &LATIN)
            } else if ARABIC.contains(&c) {
                map_digit(c, &ARABIC, &LATIN)
            } else {
                c
            }
        })
        .collect()
}

/// Convert any Latin or Arabic digits in `s` to Persian digits.
pub fn to_persian(s: &str) -> String {
    s.chars()
        .map(|c| {
            if LATIN.contains(&c) {
                map_digit(c, &LATIN, &PERSIAN)
            } else if ARABIC.contains(&c) {
                map_digit(c, &ARABIC, &PERSIAN)
            } else {
                c
            }
        })
        .collect()
}

/// Convert any Latin or Persian digits in `s` to Eastern-Arabic digits.
pub fn to_arabic(s: &str) -> String {
    s.chars()
        .map(|c| {
            if LATIN.contains(&c) {
                map_digit(c, &LATIN, &ARABIC)
            } else if PERSIAN.contains(&c) {
                map_digit(c, &PERSIAN, &ARABIC)
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_round_trip() {
        let s = "1404-10-06";
        assert_eq!(to_latin(&to_persian(s)), s);
        assert_eq!(to_latin(&to_arabic(s)), s);
    }

    #[test]
    fn persian_to_latin() {
        assert_eq!(to_latin("۱۴۰۴/۰۱/۰۱"), "1404/01/01");
    }

    #[test]
    fn arabic_to_latin() {
        assert_eq!(to_latin("١٤٠٤/٠١/٠١"), "1404/01/01");
    }

    #[test]
    fn mixed_input() {
        assert_eq!(to_latin("۱۴۰۴-١٠-06"), "1404-10-06");
    }

    #[test]
    fn non_digits_preserved() {
        assert_eq!(to_persian("date: 1404"), "date: ۱۴۰۴");
    }
}
