//! Light Persian stemmer.
//!
//! Strips a curated list of common Persian suffixes (plural, possessive,
//! verb endings, comparative, superlative).  This is **not** a morphological
//! analyser — it cannot stem irregular forms or conjugated verbs.  It is
//! useful as a pre-processing step for search, deduplication, and bag-of-words
//! models, similar to Lucene's `PersianStemFilter`.
//!
//! Stems are produced by greedy longest-match suffix removal, with a
//! safeguard that prevents stemming words shorter than 4 characters.

/// Suffixes ordered longest → shortest so the longest match wins.
const SUFFIXES: &[&str] = &[
    // Possessive plurals
    "هایمان",
    "هایتان",
    "هایشان",
    // Possessives + plural marker
    "هایم",
    "هایت",
    "هایش",
    // Plural-marker + possessive (compound)
    "انمان",
    "انتان",
    "انشان",
    "انم",
    "انت",
    "انش",
    // Plural with ezafe
    "های",
    "ها",
    // Possessives
    "مان",
    "تان",
    "شان",
    "ام",
    "ات",
    "اش",
    // Comparative / superlative
    "ترین",
    "تر",
    // Verb endings (subset)
    "یم",
    "ید",
    "ند",
    "ست",
    // Older plural
    "ان",
    // Definite-article ی and pronominal
    "ای",
];

/// Minimum length of the residual stem after suffix removal.
const MIN_STEM_LEN: usize = 3;

/// Strip the longest matching Persian suffix from `word`.
///
/// Returns `word` unchanged if none of the known suffixes apply or if
/// removing the suffix would leave a stem shorter than 3 characters.
///
/// ```
/// use parsitext::stemmer;
///
/// assert_eq!(stemmer::stem("کتاب‌ها"),     "کتاب");
/// assert_eq!(stemmer::stem("کتاب‌هایم"),    "کتاب");
/// assert_eq!(stemmer::stem("بزرگ‌ترین"),   "بزرگ");
/// assert_eq!(stemmer::stem("دوستانم"),    "دوست");
/// assert_eq!(stemmer::stem("کتاب"),       "کتاب");   // no suffix
/// assert_eq!(stemmer::stem("پی"),         "پی");    // too short
/// ```
#[must_use]
pub fn stem(word: &str) -> String {
    // Strip ZWNJ before matching; treat ‌ as a no-op connector.
    let cleaned: String = word.chars().filter(|&c| c != '\u{200C}').collect();
    let chars: Vec<char> = cleaned.chars().collect();

    for suffix in SUFFIXES {
        let suffix_chars: Vec<char> = suffix.chars().collect();
        if chars.len() < suffix_chars.len() + MIN_STEM_LEN {
            continue;
        }
        if chars.ends_with(&suffix_chars) {
            let stem_len = chars.len() - suffix_chars.len();
            return chars[..stem_len].iter().collect();
        }
    }
    cleaned
}

/// Stem each token in a slice. Convenience wrapper.
///
/// ```
/// use parsitext::stemmer;
///
/// let stems = stemmer::stem_tokens(&["کتاب‌ها".to_string(), "بزرگ‌ترین".to_string()]);
/// assert_eq!(stems, vec!["کتاب", "بزرگ"]);
/// ```
#[must_use]
pub fn stem_tokens(tokens: &[String]) -> Vec<String> {
    tokens.iter().map(|t| stem(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plural_ha() {
        assert_eq!(stem("کتاب‌ها"), "کتاب");
        assert_eq!(stem("کتابها"), "کتاب");
    }

    #[test]
    fn plural_with_possessive() {
        assert_eq!(stem("کتاب‌هایم"), "کتاب");
    }

    #[test]
    fn comparative() {
        assert_eq!(stem("بزرگ‌تر"), "بزرگ");
        assert_eq!(stem("بزرگ‌ترین"), "بزرگ");
    }

    #[test]
    fn possessive() {
        // "دوستان" stems via the "تان" suffix (greedy longest-match) leaving "دوس".
        // This is a known ambiguity with the Persian "ان" plural marker —
        // morphologically richer stemming requires a lemmatiser.
        assert_eq!(stem("دوستانم"), "دوست");
        assert_eq!(stem("کتابم"), "کتابم"); // "م" alone isn't stripped (too risky)
    }

    #[test]
    fn no_suffix_returns_word() {
        assert_eq!(stem("کتاب"), "کتاب");
    }

    #[test]
    fn too_short_word_unchanged() {
        assert_eq!(stem("پی"), "پی");
        assert_eq!(stem("ها"), "ها");
    }

    #[test]
    fn batch_stems() {
        // "گلها" (4 chars after ZWNJ stripping) is at the min-stem boundary
        // so it's left alone; "روزها" (5 chars) stems to "روز".
        let tokens = vec![
            "کتاب‌ها".to_string(),
            "گل‌ها".to_string(),
            "روزها".to_string(),
        ];
        let stems = stem_tokens(&tokens);
        assert_eq!(stems, vec!["کتاب", "گلها", "روز"]);
    }
}
