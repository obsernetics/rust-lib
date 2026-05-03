//! Persian → Latin transliteration (romanisation).
//!
//! A character-level mapping suitable for search-key generation, indexing,
//! and casual romanisation.  This is **not** a phonetic transcription —
//! Persian short vowels are not written and cannot be inferred without a
//! lexicon, so the output is consonant-faithful but vowel-bare for words
//! that lack written long vowels.
//!
//! The mapping follows a simplified UN-style romanisation:
//! `سلام دنیا` → `slam dnya`, `کتاب` → `ktab`.

/// Transliterate Persian text to Latin (ASCII) characters.
///
/// Unmapped characters (digits, punctuation, ASCII letters, whitespace) pass
/// through unchanged.  ZWNJ is dropped.
///
/// ```
/// use parsitext::transliterate;
///
/// assert_eq!(transliterate::to_latin("سلام"),  "slam");
/// assert_eq!(transliterate::to_latin("کتاب"),  "ktab");
/// assert_eq!(transliterate::to_latin("ایران"), "ayran");
/// ```
#[must_use]
pub fn to_latin(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c == '\u{200C}' {
            continue;
        }
        match map_char(c) {
            Some(s) => out.push_str(s),
            None => out.push(c),
        }
    }
    out
}

#[inline]
fn map_char(c: char) -> Option<&'static str> {
    Some(match c {
        // Vowels / vowel carriers
        'ا' | 'آ' | 'أ' | 'إ' => "a",
        'و' => "v",
        'ی' | 'ي' | 'ى' => "y",
        // Consonants
        'ب' => "b",
        'پ' => "p",
        'ت' | 'ط' => "t",
        'ث' | 'س' | 'ص' => "s",
        'ج' => "j",
        'چ' => "ch",
        'ح' | 'ه' | 'ة' => "h",
        'خ' => "kh",
        'د' => "d",
        'ذ' | 'ز' | 'ظ' | 'ض' => "z",
        'ر' => "r",
        'ژ' => "zh",
        'ش' => "sh",
        'ع' | 'ء' => "'",
        'غ' | 'ق' => "gh",
        'ف' => "f",
        'ک' | 'ك' => "k",
        'گ' => "g",
        'ل' => "l",
        'م' => "m",
        'ن' => "n",
        // Drop diacritics
        '\u{064B}'..='\u{0655}' => "",
        '\u{0640}' => "",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_words() {
        assert_eq!(to_latin("سلام"), "slam");
        assert_eq!(to_latin("کتاب"), "ktab");
        assert_eq!(to_latin("ایران"), "ayran");
    }

    #[test]
    fn drops_zwnj() {
        assert_eq!(to_latin("می\u{200C}روم"), "myrvm");
    }

    #[test]
    fn handles_compound_consonants() {
        assert_eq!(to_latin("خوش"), "khvsh");
        assert_eq!(to_latin("چای"), "chay");
    }

    #[test]
    fn passes_through_latin_and_digits() {
        assert_eq!(to_latin("hello 123"), "hello 123");
    }

    #[test]
    fn empty_input() {
        assert_eq!(to_latin(""), "");
    }
}
