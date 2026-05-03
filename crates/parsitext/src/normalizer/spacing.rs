//! Whitespace normalisation and emphatic-repetition reduction.

/// Collapse runs of whitespace (space, tab, NBSP, ZWSP, BOM, ideographic space)
/// to a single ASCII space, and strip leading/trailing whitespace.
pub fn normalize_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_space = false;

    for c in text.chars() {
        if is_collapsible_space(c) {
            if !in_space {
                out.push(' ');
            }
            in_space = true;
        } else {
            out.push(c);
            in_space = false;
        }
    }

    // Trim leading / trailing spaces added above.
    let trimmed = out.trim_matches(' ');
    if trimmed.len() == out.len() {
        out
    } else {
        trimmed.to_owned()
    }
}

/// Shorten runs of the same character to at most 2 repetitions.
///
/// Affects all characters except newlines (whose runs carry formatting meaning).
/// Examples:
/// - "خیییییلی" → "خیلی"   (Persian letter run)
/// - "!!!!!!" → "!!"        (punctuation run)
/// - "آآآآ" → "آآ"          (emphatic elongation)
pub fn reduce_repetitions(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;

    while i < n {
        let c = chars[i];
        // Count run length.
        let mut run = 1;
        while i + run < n && chars[i + run] == c {
            run += 1;
        }

        if run > 2 && is_reducible(c) {
            out.push(c);
            out.push(c);
        } else {
            for _ in 0..run {
                out.push(c);
            }
        }

        i += run;
    }

    out
}

#[inline]
fn is_collapsible_space(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\u{00A0}' // NO-BREAK SPACE
            | '\u{200B}' // ZERO WIDTH SPACE
            | '\u{FEFF}' // BOM / ZERO WIDTH NO-BREAK SPACE
            | '\u{3000}' // IDEOGRAPHIC SPACE
    )
}

#[inline]
fn is_reducible(c: char) -> bool {
    // Preserve newline runs (paragraph/line structure) and digit runs (IDs,
    // bank cards, money values, postal codes — repeated digits are
    // semantically meaningful).
    !matches!(c, '\n' | '\r') && !c.is_numeric()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapses_multiple_spaces() {
        assert_eq!(normalize_spaces("سلام  دنیا"), "سلام دنیا");
    }

    #[test]
    fn trims_edges() {
        assert_eq!(normalize_spaces("  سلام  "), "سلام");
    }

    #[test]
    fn collapses_tab_nbsp() {
        assert_eq!(normalize_spaces("a\t\u{00A0}b"), "a b");
    }

    #[test]
    fn reduces_letter_run() {
        // 5× ی collapses to 2× ی (not to 1; the rule is "max 2").
        assert_eq!(reduce_repetitions("خیییییلی"), "خییلی");
    }

    #[test]
    fn reduces_punctuation_run() {
        assert_eq!(reduce_repetitions("!!!!!!"), "!!");
    }

    #[test]
    fn keeps_two_repetitions() {
        assert_eq!(reduce_repetitions("آآ"), "آآ");
    }

    #[test]
    fn preserves_newlines() {
        assert_eq!(reduce_repetitions("a\n\n\nb"), "a\n\n\nb");
    }
}
