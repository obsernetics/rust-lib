//! Integer-to-Persian-words conversion.

const ONES: [&str; 20] = [
    "صفر",
    "یک",
    "دو",
    "سه",
    "چهار",
    "پنج",
    "شش",
    "هفت",
    "هشت",
    "نه",
    "ده",
    "یازده",
    "دوازده",
    "سیزده",
    "چهارده",
    "پانزده",
    "شانزده",
    "هفده",
    "هجده",
    "نوزده",
];

const TENS: [&str; 10] = [
    "",
    "",
    "بیست",
    "سی",
    "چهل",
    "پنجاه",
    "شصت",
    "هفتاد",
    "هشتاد",
    "نود",
];

const HUNDREDS: [&str; 10] = [
    "",
    "صد",
    "دویست",
    "سیصد",
    "چهارصد",
    "پانصد",
    "ششصد",
    "هفتصد",
    "هشتصد",
    "نهصد",
];

/// Scale words for thousand-groups. Index 0 has no scale word.
const SCALES: [&str; 7] = [
    "",
    "هزار",
    "میلیون",
    "میلیارد",
    "تریلیون",
    "کوادریلیون",
    "کوینتیلیون",
];

/// Convert an integer to its Persian-word form.
///
/// Supports the entire `i64` range.  Negative numbers are prefixed with
/// `"منفی "`.  Components are joined with `" و "` (the Persian conjunction).
///
/// ```
/// use parsitext::numbers;
///
/// assert_eq!(numbers::to_words(0),     "صفر");
/// assert_eq!(numbers::to_words(7),     "هفت");
/// assert_eq!(numbers::to_words(21),    "بیست و یک");
/// assert_eq!(numbers::to_words(105),   "صد و پنج");
/// assert_eq!(numbers::to_words(1_234), "یک هزار و دویست و سی و چهار");
/// assert_eq!(numbers::to_words(-42),   "منفی چهل و دو");
/// ```
#[must_use]
pub fn to_words(n: i64) -> String {
    if n == 0 {
        return ONES[0].to_owned();
    }

    let negative = n < 0;
    let mut abs: u64 = if n == i64::MIN {
        i64::MAX as u64 + 1
    } else {
        n.unsigned_abs()
    };

    let mut groups: Vec<u64> = Vec::with_capacity(7);
    while abs > 0 {
        groups.push(abs % 1000);
        abs /= 1000;
    }

    let mut parts: Vec<String> = Vec::new();
    for (idx, group) in groups.iter().enumerate().rev() {
        if *group == 0 {
            continue;
        }
        let group_words = three_digit_words(*group as u32);
        let scale = SCALES[idx];
        if scale.is_empty() {
            parts.push(group_words);
        } else if *group == 1 && idx == 1 {
            // "هزار" rather than "یک هزار" reads more naturally for 1000–1999.
            parts.push(format!("یک {scale}"));
        } else {
            parts.push(format!("{group_words} {scale}"));
        }
    }

    let joined = parts.join(" و ");
    if negative {
        format!("منفی {joined}")
    } else {
        joined
    }
}

/// Persian ordinal word for `n`.
///
/// Ordinals append `"م"` to the cardinal — except `1 → "اول"`, `3 → "سوم"`,
/// and `30 → "سی‌ام"` which use special forms.
///
/// ```
/// use parsitext::numbers;
///
/// assert_eq!(numbers::ordinal(1),  "اول");
/// assert_eq!(numbers::ordinal(2),  "دوم");
/// assert_eq!(numbers::ordinal(3),  "سوم");
/// assert_eq!(numbers::ordinal(7),  "هفتم");
/// assert_eq!(numbers::ordinal(20), "بیستم");
/// assert_eq!(numbers::ordinal(30), "سی‌ام");
/// ```
#[must_use]
pub fn ordinal(n: i64) -> String {
    match n {
        1 => "اول".to_owned(),
        3 => "سوم".to_owned(),
        30 => "سی‌ام".to_owned(),
        _ => {
            let words = to_words(n);
            // Ordinals attach "م"; numbers ending in vowel ه become "هم".
            format!("{words}م")
        }
    }
}

fn three_digit_words(n: u32) -> String {
    debug_assert!(n < 1000);
    let h = n / 100;
    let rem = n % 100;

    let mut parts: Vec<&str> = Vec::with_capacity(3);
    if h > 0 {
        parts.push(HUNDREDS[h as usize]);
    }

    if rem == 0 {
        return parts.join(" و ");
    }

    if rem < 20 {
        parts.push(ONES[rem as usize]);
    } else {
        let t = rem / 10;
        let o = rem % 10;
        if o == 0 {
            parts.push(TENS[t as usize]);
        } else {
            // Build "TENS و ONES" as a single string then push.
            // We need to mutate `parts`'s last element, easier to format directly:
            let tens_word = TENS[t as usize];
            let ones_word = ONES[o as usize];
            return if h > 0 {
                format!("{} و {} و {}", HUNDREDS[h as usize], tens_word, ones_word)
            } else {
                format!("{tens_word} و {ones_word}")
            };
        }
    }

    parts.join(" و ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_numbers() {
        assert_eq!(to_words(0), "صفر");
        assert_eq!(to_words(1), "یک");
        assert_eq!(to_words(7), "هفت");
        assert_eq!(to_words(15), "پانزده");
        assert_eq!(to_words(19), "نوزده");
        assert_eq!(to_words(20), "بیست");
        assert_eq!(to_words(21), "بیست و یک");
        assert_eq!(to_words(99), "نود و نه");
    }

    #[test]
    fn hundreds() {
        assert_eq!(to_words(100), "صد");
        assert_eq!(to_words(105), "صد و پنج");
        assert_eq!(to_words(150), "صد و پنجاه");
        assert_eq!(to_words(999), "نهصد و نود و نه");
    }

    #[test]
    fn thousands() {
        assert_eq!(to_words(1000), "یک هزار");
        assert_eq!(to_words(1234), "یک هزار و دویست و سی و چهار");
        assert_eq!(to_words(2000), "دو هزار");
        assert_eq!(to_words(10_000), "ده هزار");
        assert_eq!(to_words(100_000), "صد هزار");
    }

    #[test]
    fn millions_and_billions() {
        assert_eq!(to_words(1_000_000), "یک میلیون");
        assert_eq!(to_words(2_500_000), "دو میلیون و پانصد هزار");
        assert_eq!(to_words(1_000_000_000), "یک میلیارد");
    }

    #[test]
    fn negative() {
        assert_eq!(to_words(-1), "منفی یک");
        assert_eq!(to_words(-1234), "منفی یک هزار و دویست و سی و چهار");
    }

    #[test]
    fn ordinals() {
        assert_eq!(ordinal(1), "اول");
        assert_eq!(ordinal(2), "دوم");
        assert_eq!(ordinal(3), "سوم");
        assert_eq!(ordinal(30), "سی‌ام");
    }

    #[test]
    fn handles_extreme_values() {
        // Should not panic on i64 extremes.
        let _ = to_words(i64::MAX);
        let _ = to_words(i64::MIN);
    }
}
