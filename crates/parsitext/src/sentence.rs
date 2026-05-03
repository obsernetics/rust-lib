//! Sentence boundary detection for Persian text.
//!
//! Persian sentence boundaries are marked by `.` (full stop), `!`
//! (exclamation mark), `؟` (Arabic question mark, U+061F), and `؛` (Arabic
//! semicolon, U+061B).  ZWNJ (U+200C) is **not** a sentence boundary — it
//! connects compound-word components and must be preserved inside tokens.
//!
//! The splitter keeps each sentence delimiter attached to its sentence (same
//! convention as many NLP toolkits).

/// Split `text` into sentences on `.` `!` `؟` `؛`.
///
/// - The delimiter stays attached to the preceding sentence.
/// - Leading and trailing whitespace is trimmed from each sentence.
/// - Empty sentences (e.g. caused by consecutive delimiters) are discarded.
///
/// ```
/// use parsitext::sentence::split_sentences;
///
/// let sents = split_sentences("سلام. چطوری؟ خوبم!");
/// assert_eq!(sents, vec!["سلام.", "چطوری؟", "خوبم!"]);
/// ```
#[must_use]
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in text.chars() {
        current.push(c);
        if is_sentence_boundary(c) {
            let trimmed = current.trim().to_owned();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // Flush any trailing fragment without a terminal punctuation.
    let tail = current.trim().to_owned();
    if !tail.is_empty() {
        sentences.push(tail);
    }

    sentences
}

/// Returns `true` for characters that end a Persian sentence.
#[inline]
fn is_sentence_boundary(c: char) -> bool {
    matches!(c, '.' | '!' | '؟' | '؛')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_split() {
        let r = split_sentences("سلام. چطوری؟ خوبم!");
        assert_eq!(r, vec!["سلام.", "چطوری؟", "خوبم!"]);
    }

    #[test]
    fn no_delimiter() {
        assert_eq!(
            split_sentences("یک جمله بدون نقطه"),
            vec!["یک جمله بدون نقطه"]
        );
    }

    #[test]
    fn trailing_whitespace_trimmed() {
        let r = split_sentences("  سلام.   خداحافظ؟  ");
        assert_eq!(r, vec!["سلام.", "خداحافظ؟"]);
    }

    #[test]
    fn empty_input() {
        assert!(split_sentences("").is_empty());
    }

    #[test]
    fn zwnj_not_a_boundary() {
        // می‌روم is one token; the ZWNJ must NOT split it.
        let r = split_sentences("می\u{200C}روم به خانه.");
        assert_eq!(r.len(), 1);
        assert!(r[0].contains('\u{200C}'));
    }

    #[test]
    fn arabic_semicolon_splits() {
        let r = split_sentences("اول؛ دوم.");
        assert_eq!(r.len(), 2);
    }
}
