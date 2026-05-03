//! Arabic-variant → Persian-canonical character mappings.
//!
//! Common problems in Persian text from Arabic keyboards / legacy encodings:
//! - Arabic Kaf (ك U+0643) instead of Persian Keheh (ک U+06A9)
//! - Arabic Yeh (ي U+064A) or Alef Maksura (ى U+0649) instead of Farsi Yeh (ی U+06CC)
//! - Arabic Teh Marbuta (ة U+0629) instead of Heh (ه U+0647)
//! - Hamza-decorated Alefs (إ أ) instead of plain Alef (ا) for NLP normalisation

/// Replace Arabic character variants with their canonical Persian equivalents.
///
/// This is a character-level pass and runs in O(n) with a single allocation.
#[inline]
pub fn fix_arabic_chars(text: &str) -> String {
    text.chars().map(canonical).collect()
}

#[inline]
fn canonical(c: char) -> char {
    match c {
        // Kaf
        'ك' => 'ک', // U+0643 -> U+06A9
        // Yeh variants
        'ي' => 'ی', // U+064A -> U+06CC
        'ى' => 'ی', // U+0649 -> U+06CC
        // Teh Marbuta
        'ة' => 'ه', // U+0629 -> U+0647
        // Hamza-bearing Alefs (normalise for search/NLP; keep آ intact)
        'إ' => 'ا', // U+0625 -> U+0627
        'أ' => 'ا', // U+0623 -> U+0627
        // Common Arabic Presentation Forms-B that appear in legacy encodings
        '\u{FE8D}' | '\u{FE8E}' => 'ا',
        '\u{FEFB}' | '\u{FEFC}' => 'ل', // lam-alef ligature (simplified)
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_arabic_kaf() {
        assert_eq!(fix_arabic_chars("كتاب"), "کتاب");
    }

    #[test]
    fn replaces_arabic_yeh() {
        assert_eq!(fix_arabic_chars("يك"), "یک");
    }

    #[test]
    fn replaces_teh_marbuta() {
        assert_eq!(fix_arabic_chars("مدرسة"), "مدرسه");
    }

    #[test]
    fn keeps_persian_chars_intact() {
        let s = "سلام خوبی؟";
        assert_eq!(fix_arabic_chars(s), s);
    }
}
