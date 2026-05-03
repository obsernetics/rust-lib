//! Persian phonetic matching — a Soundex-style codec.
//!
//! Persian has multiple letters that share a phoneme:
//! `س` `ص` `ث` all sound like `s`; `ز` `ذ` `ض` `ظ` all sound like `z`;
//! `ت` `ط` are both `t`, etc.  Conventional spell-correction misses these
//! homophones because the surface forms are different.
//!
//! [`soundex`] reduces a Persian word to a 4-character phonetic code by:
//!
//! 1. Keeping the first letter (mapped to its phonetic group).
//! 2. Replacing each subsequent letter with a digit from a phoneme table.
//! 3. Collapsing adjacent identical digits.
//! 4. Truncating to 4 characters (left-padding with `0` if shorter).
//!
//! Two strings with the same Soundex code are likely to be Persian
//! homophones — useful for fuzzy name matching, deduplication, and
//! search indexing.
//!
//! ```
//! use parsitext::phonetic;
//!
//! // Same phonetic group → same code
//! assert_eq!(phonetic::soundex("صبر"), phonetic::soundex("سبر"));
//! // Same prefix, different consonants → different code
//! assert_ne!(phonetic::soundex("کتاب"), phonetic::soundex("کنار"));
//! ```

/// Compute the Persian Soundex code for `word` (uppercase 4-character string).
#[must_use]
pub fn soundex(word: &str) -> String {
    let cleaned: String = word
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{200C}')
        .collect();
    if cleaned.is_empty() {
        return String::from("0000");
    }
    let chars: Vec<char> = cleaned.chars().collect();
    let mut code = String::with_capacity(4);

    // First letter: keep its phonetic-group representative.
    code.push(first_letter_repr(chars[0]));

    let mut prev_digit = phoneme_code(chars[0]);
    for &c in &chars[1..] {
        let d = phoneme_code(c);
        // Drop vowels and silent letters (digit 0).
        if d == '0' {
            prev_digit = '0';
            continue;
        }
        // Collapse adjacent identical digits.
        if d == prev_digit {
            continue;
        }
        code.push(d);
        prev_digit = d;
        if code.chars().count() >= 4 {
            break;
        }
    }

    while code.chars().count() < 4 {
        code.push('0');
    }
    code
}

/// Returns `true` if `a` and `b` share the same Persian Soundex code.
#[must_use]
pub fn matches(a: &str, b: &str) -> bool {
    soundex(a) == soundex(b)
}

/// First-letter representative — promotes the letter to the canonical
/// member of its phonetic group (e.g. ص → س).
fn first_letter_repr(c: char) -> char {
    match c {
        'ص' | 'ث' => 'س',
        'ذ' | 'ض' | 'ظ' => 'ز',
        'ط' => 'ت',
        'غ' => 'ق',
        'ح' => 'ه',
        'ا' | 'آ' | 'إ' | 'أ' => 'ا',
        _ => c,
    }
}

/// Map a letter to its phoneme-group digit, or `'0'` for vowels/silent.
fn phoneme_code(c: char) -> char {
    match c {
        // Labial plosives
        'ب' | 'پ' => '1',
        // Fricatives v / f
        'ف' | 'و' => '2',
        // Dental plosives
        'ت' | 'د' | 'ط' => '3',
        // Sibilants
        'ث' | 'س' | 'ص' | 'ز' | 'ذ' | 'ض' | 'ظ' | 'ش' | 'ژ' => '4',
        // Velars / uvulars / glottals
        'ج' | 'چ' | 'ک' | 'گ' | 'ق' | 'غ' | 'خ' => '5',
        // Lateral
        'ل' => '6',
        // Nasals
        'م' | 'ن' => '7',
        // Liquid
        'ر' => '8',
        // Vowels and silent
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_length() {
        assert_eq!(soundex("ا").chars().count(), 4);
        assert_eq!(soundex("کتاب").chars().count(), 4);
        assert_eq!(soundex("سپاسگزاری").chars().count(), 4);
    }

    #[test]
    fn homophones_same_code() {
        // ص and س share the s phoneme
        assert_eq!(soundex("صبر"), soundex("سبر"));
        // ذ ز ض ظ all map to z
        assert_eq!(soundex("ذرت"), soundex("زرت"));
    }

    #[test]
    fn distinct_words_distinct_codes() {
        assert_ne!(soundex("کتاب"), soundex("کنار"));
        assert_ne!(soundex("سفر"), soundex("کتاب"));
    }

    #[test]
    fn empty_input() {
        assert_eq!(soundex(""), "0000");
    }

    #[test]
    fn matches_helper() {
        assert!(matches("صبر", "سبر"));
        assert!(!matches("کتاب", "سفر"));
    }

    #[test]
    fn pads_short_word() {
        let s = soundex("ا");
        assert_eq!(s.chars().count(), 4);
        assert!(s.ends_with("000"));
    }
}
