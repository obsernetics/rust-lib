//! Spell-checking primitives.
//!
//! This module provides reusable building blocks — Levenshtein distance and
//! a dictionary-based suggestion engine — but **does not ship a dictionary**.
//! Iranian Persian dictionaries are large and licence-encumbered; users are
//! expected to provide their own word list.
//!
//! ```
//! use parsitext::spell;
//!
//! let dict = ["سلام", "سلامتی", "سفر", "سفره"];
//! let suggestions = spell::suggest("سلامن", &dict, 2);
//! assert!(suggestions.contains(&"سلام"));
//! ```

/// Compute the Levenshtein edit distance between two strings (counted in
/// Unicode scalar values, not bytes).
///
/// Uses the standard two-row dynamic-programming algorithm; `O(n*m)` time
/// and `O(min(n,m))` space.
///
/// ```
/// use parsitext::spell::levenshtein;
///
/// assert_eq!(levenshtein("سلام", "سلام"),     0);
/// assert_eq!(levenshtein("کتاب", "کتیب"),     1); // single substitution
/// assert_eq!(levenshtein("سلام", "سلامتی"),  2); // two insertions
/// ```
#[must_use]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (n, m) = (a_chars.len(), b_chars.len());

    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr: Vec<usize> = vec![0; m + 1];

    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1) // deletion
                .min(curr[j - 1] + 1) // insertion
                .min(prev[j - 1] + cost); // substitution
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

/// Suggest from the bundled high-frequency Persian word list.
///
/// Equivalent to `suggest(word, parsitext::spell_dict::dict(), max_distance)`.
///
/// ```
/// use parsitext::spell;
///
/// // The word "سلامن" is a typo of "سلام" — distance 1.
/// let s = spell::suggest_builtin("سلامن", 1);
/// assert!(s.contains(&"سلام"));
/// ```
#[must_use]
pub fn suggest_builtin(word: &str, max_distance: usize) -> Vec<&'static str> {
    suggest(word, crate::spell_dict::dict(), max_distance)
}

/// Suggest the dictionary words within `max_distance` edits of `word`,
/// sorted by ascending distance (closest first).
///
/// `dict` may be any iterable of `&str`-convertible values (so `&[&str]`,
/// `&[String]`, or `&Vec<String>` all work).  Time is `O(|dict| * |word|^2)`.
///
/// ```
/// use parsitext::spell;
///
/// let dict = ["سلام", "کتاب", "سفر"];
/// let suggestions = spell::suggest("سلامی", &dict, 1);
/// assert_eq!(suggestions, vec!["سلام"]);
/// ```
#[must_use]
pub fn suggest<'a, I, S>(word: &str, dict: I, max_distance: usize) -> Vec<&'a str>
where
    I: IntoIterator<Item = &'a S>,
    S: AsRef<str> + 'a + ?Sized,
{
    let mut hits: Vec<(usize, &'a str)> = dict
        .into_iter()
        .map(|w| w.as_ref())
        .filter_map(|w| {
            let d = levenshtein(word, w);
            (d <= max_distance).then_some((d, w))
        })
        .collect();
    hits.sort_by_key(|(d, _)| *d);
    hits.into_iter().map(|(_, w)| w).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_zero_for_equal() {
        assert_eq!(levenshtein("کتاب", "کتاب"), 0);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn distance_simple_edits() {
        assert_eq!(levenshtein("کتاب", "کتیب"), 1);
        assert_eq!(levenshtein("سلام", "سلامتی"), 2);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }

    #[test]
    fn distance_empty_inputs() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn suggest_basic() {
        let dict = ["سلام", "کتاب", "سفر"];
        let s = suggest("سلامی", &dict, 1);
        assert_eq!(s, vec!["سلام"]);
    }

    #[test]
    fn suggest_sorted_by_distance() {
        let dict = ["سفر", "سفره", "سلام"];
        let s = suggest("سفری", &dict, 2);
        // "سفر" (d=1) should come before "سفره" (d=2) and "سلام" (d=3)
        assert_eq!(s[0], "سفر");
    }

    #[test]
    fn suggest_no_matches() {
        let dict = ["alpha", "beta"];
        assert!(suggest("سلام", &dict, 1).is_empty());
    }
}
