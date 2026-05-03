//! Finglish (Persian written in Latin script) → Persian conversion.
//!
//! Iranians routinely type Persian using ASCII letters because of input-method
//! limitations on phones, keyboards, and chat apps.  *"salam khoobi?"* is
//! Finglish for *"سلام خوبی؟"*.
//!
//! This module converts Finglish text back to Persian script using a
//! two-stage strategy:
//!
//! 1. **Word-level dictionary** — the most common ~200 Finglish forms (greetings,
//!    pronouns, verbs, prepositions) map to their canonical Persian spelling.
//! 2. **Character-level transliteration** — unknown words are rebuilt from a
//!    Latin → Persian phoneme table that handles digraphs (`kh`, `sh`, `ch`,
//!    `zh`, `gh`, `oo`, `ou`, `aa`, `ee`) and single characters.
//!
//! Limitations:
//! - Persian has multiple letters with the same Latin sound (`س` `ص` `ث`,
//!   `ز` `ذ` `ض` `ظ`, `ت` `ط`, `ه` `ح`, `ق` `غ`).  The character-level pass
//!   always picks the most common variant; a real spell-correct step is
//!   needed for perfect recovery.
//! - Vowels are heuristic.  Persian short vowels are not written, so
//!   `man` → `من` is recoverable but `khoob` → `خوب` requires the
//!   "long-vowel" digraph `oo`.
//!
//! For the inverse direction (Persian → Latin) see [`crate::transliterate`].
//!
//! ```
//! use parsitext::finglish;
//!
//! assert_eq!(finglish::to_persian("salam"),         "سلام");
//! assert_eq!(finglish::to_persian("chetori?"),      "چطوری?");
//! assert_eq!(finglish::to_persian("man khoobam"),   "من خوبم");
//! ```

mod dict;

/// Convert Finglish text to Persian.
///
/// ASCII letters are interpreted as Finglish and converted; whitespace,
/// punctuation, digits, and Persian characters pass through unchanged.
///
/// ```
/// use parsitext::finglish::to_persian;
///
/// assert_eq!(to_persian("salam khoobi?"), "سلام خوبی?");
/// assert_eq!(to_persian("merci :)"),       "مرسی :)");
/// ```
#[must_use]
pub fn to_persian(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    let mut current_word = String::new();

    for c in text.chars() {
        if c.is_ascii_alphabetic() || c == '\'' {
            current_word.push(c);
        } else {
            if !current_word.is_empty() {
                out.push_str(&convert_word(&current_word));
                current_word.clear();
            }
            out.push(c);
        }
    }
    if !current_word.is_empty() {
        out.push_str(&convert_word(&current_word));
    }
    out
}

fn convert_word(word: &str) -> String {
    let lower = word.to_lowercase();
    if let Some(persian) = dict::lookup(&lower) {
        return persian.to_owned();
    }
    char_level(&lower)
}

fn char_level(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::new();
    let mut i = 0;

    while i < chars.len() {
        // Try a digraph (2-char sequence) first; falls back to single char.
        if i + 1 < chars.len() {
            let pair: String = chars[i..i + 2].iter().collect();
            if let Some(persian) = digraph(&pair) {
                out.push_str(persian);
                i += 2;
                continue;
            }
        }
        out.push_str(monograph(chars[i], i, &chars));
        i += 1;
    }
    out
}

fn digraph(s: &str) -> Option<&'static str> {
    Some(match s {
        "kh" => "خ",
        "sh" => "ش",
        "ch" => "چ",
        "zh" => "ژ",
        "gh" => "ق",
        "oo" => "و",
        "ou" => "و",
        "aa" => "ا",
        "ee" => "ی",
        _ => return None,
    })
}

fn monograph(c: char, position: usize, chars: &[char]) -> &'static str {
    match c {
        // 'a' at word start often becomes آ (long alef) for stressed initial.
        'a' if position == 0 && chars.len() > 1 => "آ",
        'a' => "ا",
        'b' => "ب",
        'c' => "ک",
        'd' => "د",
        'e' if position == chars.len() - 1 => "ه", // word-final e is usually ه
        'e' => "",                                 // medial short e is silent
        'f' => "ف",
        'g' => "گ",
        'h' => "ه",
        'i' => "ی",
        'j' => "ج",
        'k' => "ک",
        'l' => "ل",
        'm' => "م",
        'n' => "ن",
        'o' => "و",
        'p' => "پ",
        'q' => "ق",
        'r' => "ر",
        's' => "س",
        't' => "ت",
        'u' => "و",
        'v' => "و",
        'w' => "و",
        'x' => "خ",
        'y' => "ی",
        'z' => "ز",
        '\'' => "ع",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dict_words_take_priority() {
        // Without the dict, "salam" would char-build to something else.
        assert_eq!(to_persian("salam"), "سلام");
        assert_eq!(to_persian("merci"), "مرسی");
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(to_persian("Salam"), "سلام");
        assert_eq!(to_persian("MERCI"), "مرسی");
    }

    #[test]
    fn punctuation_passes_through() {
        assert_eq!(to_persian("salam!"), "سلام!");
        assert_eq!(to_persian("chetori?"), "چطوری?");
    }

    #[test]
    fn whitespace_preserved() {
        assert_eq!(to_persian("man khoobam"), "من خوبم");
    }

    #[test]
    fn char_level_fallback() {
        // "ketab" isn't in the dict; medial e is silent so this round-trips.
        assert_eq!(to_persian("ketab"), "کتاب");
    }

    #[test]
    fn digraphs_kh_sh_ch() {
        assert_eq!(to_persian("khosh"), "خوش");
        assert_eq!(to_persian("char"), "چار");
    }

    #[test]
    fn empty_string() {
        assert_eq!(to_persian(""), "");
    }

    #[test]
    fn mixed_persian_passes_through() {
        assert_eq!(to_persian("سلام salam"), "سلام سلام");
    }
}
