//! Text statistics for Persian documents.

use crate::sentence::split_sentences;
use crate::tokenizer;

/// Per-document text statistics returned by
/// [`Parsitext::text_stats`](crate::Parsitext::text_stats).
///
/// All counts are computed on the **normalised** form of the input unless
/// stated otherwise.
///
/// ```
/// use parsitext::Parsitext;
///
/// let stats = Parsitext::default().text_stats("سلام دنیا! امروز شنبه است.");
/// assert_eq!(stats.word_count, 5);
/// assert!(stats.persian_ratio > 0.9);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextStats {
    /// Total number of Unicode scalar values (characters).
    pub char_count: usize,
    /// Total number of UTF-8 bytes.
    pub byte_count: usize,
    /// Number of whitespace characters.
    pub space_count: usize,
    /// Number of Unicode alphabetic characters (any script).
    pub letter_count: usize,
    /// Number of characters in the Arabic/Persian Unicode block (U+0600–U+06FF).
    pub persian_letter_count: usize,
    /// Number of ASCII alphabetic characters.
    pub latin_letter_count: usize,
    /// Number of Unicode decimal-digit characters (any script).
    pub digit_count: usize,
    /// Approximate word count (whitespace-delimited tokens, punctuation
    /// excluded from the count).
    pub word_count: usize,
    /// Number of sentences detected by [`split_sentences`].
    pub sentence_count: usize,
    /// Fraction of alphabetic characters that are Persian/Arabic (`0.0–1.0`).
    /// Returns `0.0` when there are no alphabetic characters.
    pub persian_ratio: f64,
    /// Number of distinct tokens (case-sensitive).
    pub unique_token_count: usize,
}

/// Compute [`TextStats`] for `text` (should already be normalised).
pub(crate) fn compute(text: &str) -> TextStats {
    let mut char_count = 0usize;
    let mut space_count = 0usize;
    let mut letter_count = 0usize;
    let mut persian_letter_count = 0usize;
    let mut latin_letter_count = 0usize;
    let mut digit_count = 0usize;

    for c in text.chars() {
        char_count += 1;
        if c.is_whitespace() {
            space_count += 1;
        } else if c.is_alphabetic() {
            letter_count += 1;
            if is_arabic_block(c) {
                persian_letter_count += 1;
            } else if c.is_ascii_alphabetic() {
                latin_letter_count += 1;
            }
        } else if c.is_numeric() {
            digit_count += 1;
        }
    }

    let persian_ratio = if letter_count > 0 {
        persian_letter_count as f64 / letter_count as f64
    } else {
        0.0
    };

    let tokens = tokenizer::tokenize(text);
    let word_count = tokens
        .iter()
        .filter(|t| t.chars().any(|c| c.is_alphabetic() || c.is_numeric()))
        .count();

    let unique_token_count = {
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for t in &tokens {
            seen.insert(t.as_str());
        }
        seen.len()
    };

    let sentence_count = split_sentences(text).len();

    TextStats {
        char_count,
        byte_count: text.len(),
        space_count,
        letter_count,
        persian_letter_count,
        latin_letter_count,
        digit_count,
        word_count,
        sentence_count,
        persian_ratio,
        unique_token_count,
    }
}

#[inline]
fn is_arabic_block(c: char) -> bool {
    (0x0600u32..=0x06FF).contains(&(c as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_counts() {
        let s = compute("سلام دنیا");
        assert_eq!(s.word_count, 2);
        assert_eq!(s.space_count, 1);
        assert!(s.persian_ratio > 0.9);
    }

    #[test]
    fn mixed_script() {
        let s = compute("hello سلام");
        assert!(s.latin_letter_count > 0);
        assert!(s.persian_letter_count > 0);
        assert!(s.persian_ratio > 0.0 && s.persian_ratio < 1.0);
    }

    #[test]
    fn digit_count() {
        let s = compute("۱۲۳ abc");
        assert_eq!(s.digit_count, 3);
    }

    #[test]
    fn sentence_count() {
        let s = compute("جمله اول. جمله دوم؟");
        assert_eq!(s.sentence_count, 2);
    }

    #[test]
    fn empty() {
        let s = compute("");
        assert_eq!(s.char_count, 0);
        assert_eq!(s.persian_ratio, 0.0);
    }
}
