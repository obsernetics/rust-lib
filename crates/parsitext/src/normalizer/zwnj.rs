//! Zero Width Non-Joiner (ZWNJ, U+200C) normalization.
//!
//! ZWNJ is essential in Persian for compound words, verb prefixes (می‌), and
//! plural suffixes (‌ها).  It is also routinely misused: placed next to spaces,
//! duplicated, or left at token boundaries where it is meaningless.
//!
//! This pass **removes** misplaced ZWNJs; it does not insert missing ones
//! (insertion requires morphological analysis and is out of scope here).

const ZWNJ: char = '\u{200C}';

/// Remove misplaced ZWNJs, keeping only those that sit between two
/// Arabic/Persian alphabetic code points.
///
/// When consecutive ZWNJs occur between two Arabic letters (e.g. double ZWNJ
/// in a compound word), only the first one is kept.
pub fn normalize_zwnj(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(text.len());

    for i in 0..n {
        let c = chars[i];
        if c != ZWNJ {
            out.push(c);
            continue;
        }

        // Dedup: skip this ZWNJ if the immediately preceding raw char was also ZWNJ.
        if i > 0 && chars[i - 1] == ZWNJ {
            continue;
        }

        // Look for the nearest non-ZWNJ characters on each side.
        let prev_real = chars[..i]
            .iter()
            .rev()
            .find(|&&ch| ch != ZWNJ)
            .copied()
            .unwrap_or('\0');
        let next_real = chars[i + 1..]
            .iter()
            .find(|&&ch| ch != ZWNJ)
            .copied()
            .unwrap_or('\0');

        // Keep only when joining two Arabic/Persian letters.
        if is_arabic_letter(prev_real) && is_arabic_letter(next_real) {
            out.push(ZWNJ);
        }
    }

    out
}

/// Returns `true` for characters in the primary Arabic block (U+0600–U+06FF),
/// which covers all Persian letters.
#[inline]
fn is_arabic_letter(c: char) -> bool {
    let cp = c as u32;
    (0x0600..=0x06FF).contains(&cp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_valid_zwnj() {
        // می‌روم — ZWNJ between ی and ر (both Arabic block)
        let s = "می\u{200C}روم";
        assert_eq!(normalize_zwnj(s), s);
    }

    #[test]
    fn drops_zwnj_next_to_space() {
        assert_eq!(normalize_zwnj("سلام \u{200C}دنیا"), "سلام دنیا");
        assert_eq!(normalize_zwnj("سلام\u{200C} دنیا"), "سلام دنیا");
    }

    #[test]
    fn drops_leading_trailing_zwnj() {
        assert_eq!(normalize_zwnj("\u{200C}سلام\u{200C}"), "سلام");
    }

    #[test]
    fn collapses_consecutive_zwnj() {
        // Double ZWNJ between two Persian letters → only one survives
        let s = "می\u{200C}\u{200C}رود";
        assert_eq!(normalize_zwnj(s), "می\u{200C}رود");
    }

    #[test]
    fn drops_zwnj_next_to_latin() {
        assert_eq!(normalize_zwnj("ایران\u{200C}Air"), "ایرانAir");
    }
}
