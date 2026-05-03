//! Persian-words-to-integer conversion.

/// Parse Persian number words into an `i64`.
///
/// Returns `None` if `text` does not look like a valid Persian number
/// expression. Tolerates extra whitespace, ZWNJ, and the conjunction
/// "و" between components.
///
/// ```
/// use parsitext::numbers;
///
/// assert_eq!(numbers::from_words("صفر"),                  Some(0));
/// assert_eq!(numbers::from_words("بیست و یک"),            Some(21));
/// assert_eq!(numbers::from_words("صد و پنج"),             Some(105));
/// assert_eq!(numbers::from_words("یک هزار و دویست"),       Some(1200));
/// assert_eq!(numbers::from_words("دو میلیون"),            Some(2_000_000));
/// assert_eq!(numbers::from_words("منفی پنجاه"),           Some(-50));
/// assert_eq!(numbers::from_words("nope"),                 None);
/// ```
#[must_use]
pub fn from_words(text: &str) -> Option<i64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (negative, body) = if let Some(rest) = trimmed.strip_prefix("منفی") {
        (true, rest.trim_start())
    } else {
        (false, trimmed)
    };

    let tokens: Vec<&str> = body
        .split(|c: char| c.is_whitespace() || c == '\u{200C}')
        .filter(|t| !t.is_empty() && *t != "و")
        .collect();
    if tokens.is_empty() {
        return None;
    }

    let mut total: i128 = 0;
    let mut current: i128 = 0;

    for tok in tokens {
        if let Some(v) = unit_value(tok) {
            // Small unit (0..1000): accumulate into current.
            current = current.checked_add(v as i128)?;
        } else if let Some(scale) = scale_value(tok) {
            // Multiply current group by scale and roll into total.
            // If current is 0 (e.g. "هزار" alone), treat as 1.
            let group = if current == 0 { 1 } else { current };
            total = total.checked_add(group.checked_mul(scale as i128)?)?;
            current = 0;
        } else {
            return None;
        }
    }
    total = total.checked_add(current)?;

    let signed = if negative { -total } else { total };
    if signed > i64::MAX as i128 || signed < i64::MIN as i128 {
        None
    } else {
        Some(signed as i64)
    }
}

fn unit_value(word: &str) -> Option<u64> {
    Some(match word {
        "صفر" => 0,
        "یک" | "یه" => 1,
        "دو" => 2,
        "سه" => 3,
        "چهار" => 4,
        "پنج" => 5,
        "شش" | "شیش" => 6,
        "هفت" => 7,
        "هشت" => 8,
        "نه" => 9,
        "ده" => 10,
        "یازده" => 11,
        "دوازده" => 12,
        "سیزده" => 13,
        "چهارده" => 14,
        "پانزده" | "پونزده" => 15,
        "شانزده" | "شونزده" => 16,
        "هفده" | "هیفده" => 17,
        "هجده" | "هیجده" => 18,
        "نوزده" => 19,
        "بیست" => 20,
        "سی" => 30,
        "چهل" => 40,
        "پنجاه" => 50,
        "شصت" => 60,
        "هفتاد" => 70,
        "هشتاد" => 80,
        "نود" => 90,
        "صد" | "یکصد" => 100,
        "دویست" => 200,
        "سیصد" => 300,
        "چهارصد" => 400,
        "پانصد" => 500,
        "ششصد" => 600,
        "هفتصد" => 700,
        "هشتصد" => 800,
        "نهصد" => 900,
        _ => return None,
    })
}

fn scale_value(word: &str) -> Option<u64> {
    Some(match word {
        "هزار" => 1_000,
        "میلیون" => 1_000_000,
        "میلیارد" => 1_000_000_000,
        "تریلیون" => 1_000_000_000_000,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small() {
        assert_eq!(from_words("صفر"), Some(0));
        assert_eq!(from_words("هفت"), Some(7));
        assert_eq!(from_words("بیست و یک"), Some(21));
    }

    #[test]
    fn hundreds() {
        assert_eq!(from_words("صد و پنج"), Some(105));
        assert_eq!(from_words("سیصد و پنجاه و دو"), Some(352));
    }

    #[test]
    fn thousands() {
        assert_eq!(from_words("هزار"), Some(1_000));
        assert_eq!(from_words("یک هزار و دویست"), Some(1_200));
        assert_eq!(from_words("دو هزار و پانصد"), Some(2_500));
    }

    #[test]
    fn millions() {
        assert_eq!(from_words("یک میلیون"), Some(1_000_000));
        assert_eq!(from_words("دو میلیون و پانصد هزار"), Some(2_500_000));
    }

    #[test]
    fn round_trip() {
        for n in [0i64, 1, 21, 100, 999, 1234, 1_000_000, 2_500_000] {
            let words = crate::numbers::to_words(n);
            assert_eq!(from_words(&words), Some(n), "roundtrip failed for {n}");
        }
    }

    #[test]
    fn negative_prefix() {
        assert_eq!(from_words("منفی پنجاه"), Some(-50));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(from_words(""), None);
        assert_eq!(from_words("nope"), None);
        assert_eq!(from_words("hello world"), None);
    }
}
