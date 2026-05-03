//! Heuristic ZWNJ **insertion** — the inverse of the normaliser's ZWNJ
//! removal pass.
//!
//! Persian compound-word components should be glued with U+200C, not regular
//! spaces.  This module inserts ZWNJ in three high-confidence positions:
//!
//! 1. After the verb prefix `می` or `نمی` at the start of a word.
//! 2. Before the plural suffix `ها` or `های` at the end of a word.
//! 3. Before the possessive endings `ام/ات/اش/مان/تان/شان` at the end of a
//!    word, when they extend a Persian-letter base.
//!
//! Word lookups are deliberately heuristic — no verb dictionary is consulted —
//! so over-insertion is possible on edge cases.  The pass is **opt-in** via
//! [`crate::ParsitextConfig::insert_zwnj`].

const ZWNJ: &str = "\u{200C}";

/// Insert ZWNJ at common Persian morphological boundaries.
///
/// The function preserves whitespace and punctuation between tokens.
/// Existing ZWNJs are left in place (the function never duplicates them).
///
/// ```
/// use parsitext::zwnj_insert::insert;
///
/// assert_eq!(insert("میروم"),    "می\u{200C}روم");
/// assert_eq!(insert("نمیدانم"),  "نمی\u{200C}دانم");
/// assert_eq!(insert("کتابها"),  "کتاب\u{200C}ها");
/// assert_eq!(insert("کتابهای"), "کتاب\u{200C}های");
/// ```
///
/// Multi-suffix words (e.g. `کتابهایم` = book + plural + 1sg possessive) need
/// a morphological analyser to fully glue, which is out of scope; only the
/// outermost suffix gets a ZWNJ.
#[must_use]
pub fn insert(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 8);
    let mut current = String::new();

    for c in text.chars() {
        if is_word_break(c) {
            if !current.is_empty() {
                out.push_str(&apply(&current));
                current.clear();
            }
            out.push(c);
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        out.push_str(&apply(&current));
    }
    out
}

#[inline]
fn is_word_break(c: char) -> bool {
    c.is_whitespace() || c.is_ascii_punctuation() || matches!(c, '،' | '؛' | '؟' | '«' | '»')
}

/// Apply the three insertion rules to a single token.
fn apply(word: &str) -> String {
    if word.contains(ZWNJ) {
        // Already has explicit ZWNJ; trust the author.
        return word.to_owned();
    }

    let mut s = word.to_owned();

    // Rule 1: می / نمی verb prefix.
    s = insert_after_verb_prefix(&s, "نمی");
    s = insert_after_verb_prefix(&s, "می");

    // Rule 2: ها / های plural suffix.
    s = insert_before_suffix(&s, "های");
    s = insert_before_suffix(&s, "ها");

    // Rule 3: pronominal possessive endings.
    for suffix in &["مان", "تان", "شان", "ام", "ات", "اش"] {
        s = insert_before_suffix(&s, suffix);
    }

    s
}

fn insert_after_verb_prefix(word: &str, prefix: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let prefix_chars: Vec<char> = prefix.chars().collect();
    if chars.len() < prefix_chars.len() + 3 {
        return word.to_owned();
    }
    if !chars[..prefix_chars.len()]
        .iter()
        .zip(prefix_chars.iter())
        .all(|(a, b)| a == b)
    {
        return word.to_owned();
    }
    // Require the next char to be a Persian letter.
    let next = chars[prefix_chars.len()];
    if !is_persian_letter(next) {
        return word.to_owned();
    }
    let head: String = chars[..prefix_chars.len()].iter().collect();
    let tail: String = chars[prefix_chars.len()..].iter().collect();
    format!("{head}{ZWNJ}{tail}")
}

fn insert_before_suffix(word: &str, suffix: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let suffix_chars: Vec<char> = suffix.chars().collect();
    // Require ≥3 chars of stem before the suffix to avoid stray matches.
    if chars.len() < suffix_chars.len() + 3 {
        return word.to_owned();
    }
    if !chars[chars.len() - suffix_chars.len()..]
        .iter()
        .zip(suffix_chars.iter())
        .all(|(a, b)| a == b)
    {
        return word.to_owned();
    }
    let stem_end = chars.len() - suffix_chars.len();
    if !is_persian_letter(chars[stem_end - 1]) {
        return word.to_owned();
    }
    let stem: String = chars[..stem_end].iter().collect();
    let suf: String = chars[stem_end..].iter().collect();
    format!("{stem}{ZWNJ}{suf}")
}

#[inline]
fn is_persian_letter(c: char) -> bool {
    let cp = c as u32;
    (0x0621..=0x06FF).contains(&cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_after_mi_prefix() {
        assert_eq!(insert("میروم"), "می\u{200C}روم");
    }

    #[test]
    fn inserts_after_nemi_prefix() {
        assert_eq!(insert("نمیدانم"), "نمی\u{200C}دانم");
    }

    #[test]
    fn inserts_before_ha() {
        assert_eq!(insert("کتابها"), "کتاب\u{200C}ها");
    }

    #[test]
    fn inserts_before_haye() {
        assert_eq!(insert("کتابهای"), "کتاب\u{200C}های");
    }

    #[test]
    fn inserts_before_possessive_am() {
        assert_eq!(insert("کتابام"), "کتاب\u{200C}ام");
    }

    #[test]
    fn skips_when_already_has_zwnj() {
        assert_eq!(insert("می\u{200C}روم"), "می\u{200C}روم");
    }

    #[test]
    fn preserves_whitespace_and_punct() {
        assert_eq!(insert("میروم به خانه."), "می\u{200C}روم به خانه.");
    }

    #[test]
    fn skips_too_short_words() {
        // "ما" is 2 chars and doesn't extend with verb stem
        assert_eq!(insert("ما"), "ما");
    }
}
