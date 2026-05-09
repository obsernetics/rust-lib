//! Script-detection helpers: distinguish Arabic, Persian, and Latin
//! characters; convert in either direction.
//!
//! Persian and Arabic share most of the Arabic Unicode block but each script
//! has its own canonical letters for a few sounds:
//!
//! | Sound | Persian | Arabic |
//! |-------|---------|--------|
//! | k     | ک (U+06A9) | ك (U+0643) |
//! | y     | ی (U+06CC) | ي (U+064A) |
//!
//! Persian additionally has پ چ ژ گ that Arabic lacks; Arabic has ة ى ي ك
//! that Persian renames.  These predicates are character-level and run in
//! O(n).

/// Returns `true` iff `c` is a letter unique to Persian (پ چ ژ گ ی ک).
#[must_use]
pub fn is_persian_letter(c: char) -> bool {
    matches!(c, 'پ' | 'چ' | 'ژ' | 'گ' | 'ی' | 'ک')
}

/// Returns `true` iff `c` is an Arabic-only letter that Persian does not
/// use canonically (ة, ى, ي, ك).
#[must_use]
pub fn is_arabic_only_letter(c: char) -> bool {
    matches!(c, 'ة' | 'ى' | 'ي' | 'ك' | 'إ' | 'أ' | 'ؤ' | 'ئ')
}

/// Returns `true` iff `c` belongs to the Arabic / Persian Unicode block
/// (`U+0600..=U+06FF`, plus presentation forms in `U+FB50..=U+FDFF` and
/// `U+FE70..=U+FEFF`).
#[must_use]
pub fn is_arabic_or_persian(c: char) -> bool {
    let cp = c as u32;
    (0x0600..=0x06FF).contains(&cp)
        || (0xFB50..=0xFDFF).contains(&cp)
        || (0xFE70..=0xFEFF).contains(&cp)
}

/// Returns `true` if `text` contains at least one Arabic-only letter
/// (ة ى ي ك …).  A common quality check before persisting user input.
///
/// ```
/// use parsitext::script::has_arabic;
///
/// assert!(has_arabic("ك"));
/// assert!(has_arabic("مدرسة"));
/// assert!(!has_arabic("سلام دنیا"));
/// ```
#[must_use]
pub fn has_arabic(text: &str) -> bool {
    text.chars().any(is_arabic_only_letter)
}

/// Returns `true` if `text` contains at least one Persian-specific letter
/// (پ چ ژ گ ی ک).
///
/// ```
/// use parsitext::script::has_persian;
///
/// assert!(has_persian("پارسی"));
/// assert!(!has_persian("hello"));
/// ```
#[must_use]
pub fn has_persian(text: &str) -> bool {
    text.chars().any(is_persian_letter)
}

/// Returns `true` if every alphabetic character is Persian or shared
/// Arabic-block (i.e. no Latin or other scripts mixed in).  Whitespace,
/// digits and punctuation are ignored.
///
/// ```
/// use parsitext::script::is_pure_persian;
///
/// assert!(is_pure_persian("سلام دنیا"));
/// assert!(is_pure_persian("سلام، خوبی؟"));   // Persian punctuation OK
/// assert!(!is_pure_persian("hello سلام"));   // Latin mixed in
/// ```
#[must_use]
pub fn is_pure_persian(text: &str) -> bool {
    let mut had_letter = false;
    for c in text.chars() {
        if c.is_alphabetic() {
            had_letter = true;
            if !is_arabic_or_persian(c) {
                return false;
            }
        }
    }
    had_letter
}

/// Convert canonical Persian letters to their Arabic equivalents
/// (ک → ك, ی → ي).  Useful when emitting text for an Arabic-only
/// keyboard / display.
///
/// This is the inverse of the `orthography::fix_arabic_chars` pass that
/// runs in the standard normalisation pipeline.
///
/// ```
/// use parsitext::script::to_arabic;
///
/// assert_eq!(to_arabic("کتاب"), "كتاب");
/// assert_eq!(to_arabic("یا علی"), "يا علي");
/// ```
#[must_use]
pub fn to_arabic(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'ک' => 'ك',
            'ی' => 'ي',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_arabic_letter() {
        assert!(has_arabic("هذة مدرسة"));
        assert!(!has_arabic("این مدرسه است")); // Persian ی (U+06CC), not Arabic ي
    }

    #[test]
    fn detects_persian_letter() {
        assert!(has_persian("پارسی زبان است"));
        assert!(!has_persian("هذا كتاب"));
    }

    #[test]
    fn pure_persian_detection() {
        assert!(is_pure_persian("سلام دنیا"));
        assert!(is_pure_persian("یک، دو، سه!"));
        assert!(!is_pure_persian("hello سلام"));
    }

    #[test]
    fn pure_persian_empty_no_letters() {
        // No alphabetic chars at all — not "pure Persian".
        assert!(!is_pure_persian(""));
        assert!(!is_pure_persian("12345"));
    }

    #[test]
    fn to_arabic_basic() {
        assert_eq!(to_arabic("کتاب"), "كتاب");
        assert_eq!(to_arabic("یا علی"), "يا علي");
        assert_eq!(to_arabic("سلام"), "سلام"); // no change
    }

    #[test]
    fn classification_predicates() {
        assert!(is_persian_letter('پ'));
        assert!(!is_persian_letter('ك'));
        assert!(is_arabic_only_letter('ك'));
        assert!(is_arabic_or_persian('ا'));
        assert!(!is_arabic_or_persian('a'));
    }
}
