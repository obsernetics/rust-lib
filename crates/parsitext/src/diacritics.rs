//! Arabic diacritics (harakat / تشکیل) removal.
//!
//! Diacritics are combining marks that annotate vowel sounds in Arabic and
//! are occasionally present in Persian text (especially classical poetry,
//! religious texts, and language-learning material).  For most NLP tasks
//! (search, matching, sentiment analysis) they add noise rather than
//! information.

/// Remove Arabic harakat and the tatweel extender from `text`.
///
/// The following Unicode code points are stripped:
///
/// | Code point | Name | Glyph |
/// |------------|------|-------|
/// | U+064B | Arabic Fathatan | ً |
/// | U+064C | Arabic Dammatan | ٌ |
/// | U+064D | Arabic Kasratan | ٍ |
/// | U+064E | Arabic Fatha | َ |
/// | U+064F | Arabic Damma | ُ |
/// | U+0650 | Arabic Kasra | ِ |
/// | U+0651 | Arabic Shadda | ّ |
/// | U+0652 | Arabic Sukun | ْ |
/// | U+0653 | Arabic Maddah Above | ٓ |
/// | U+0654 | Arabic Hamza Above | ٔ |
/// | U+0655 | Arabic Hamza Below | ٕ |
/// | U+0640 | Arabic Tatweel | ـ |
///
/// ```
/// use parsitext::diacritics::remove_diacritics;
///
/// assert_eq!(remove_diacritics("مُحَمَّد"), "محمد");
/// assert_eq!(remove_diacritics("كِتَابٌ"), "كتاب");
/// assert_eq!(remove_diacritics("سلاـم"), "سلام"); // tatweel removed
/// ```
#[must_use]
pub fn remove_diacritics(text: &str) -> String {
    text.chars().filter(|&c| !is_diacritic(c)).collect()
}

/// Returns `true` if `c` is an Arabic diacritic or tatweel character that
/// should be stripped by [`remove_diacritics`].
#[inline]
pub fn is_diacritic(c: char) -> bool {
    matches!(
        c as u32,
        0x064B // Fathatan ً
        | 0x064C // Dammatan ٌ
        | 0x064D // Kasratan ٍ
        | 0x064E // Fatha    َ
        | 0x064F // Damma    ُ
        | 0x0650 // Kasra    ِ
        | 0x0651 // Shadda   ّ
        | 0x0652 // Sukun    ْ
        | 0x0653 // Maddah above ٓ
        | 0x0654 // Hamza above  ٔ
        | 0x0655 // Hamza below  ٕ
        | 0x0640 // Tatweel  ـ
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_fatha_kasra_shadda() {
        assert_eq!(remove_diacritics("مُحَمَّد"), "محمد");
    }

    #[test]
    fn removes_tanwin() {
        assert_eq!(remove_diacritics("كِتَابٌ"), "كتاب");
    }

    #[test]
    fn removes_tatweel() {
        assert_eq!(remove_diacritics("سلاـم"), "سلام");
    }

    #[test]
    fn clean_text_unchanged() {
        let s = "سلام خوبی؟";
        assert_eq!(remove_diacritics(s), s);
    }

    #[test]
    fn empty_string() {
        assert_eq!(remove_diacritics(""), "");
    }
}
