//! Simple Persian-aware word tokenizer.
//!
//! Strategy:
//! - Split on ASCII/Unicode whitespace.
//! - Treat ZWNJ (U+200C) as a **non-breaking** separator — it is part of the
//!   compound word, not a boundary.
//! - Detach sentence-boundary punctuation as separate single-character tokens.

/// Tokenise `text` into a list of tokens.
///
/// The input should already be normalised (ZWNJ, spaces, digits).
/// Punctuation tokens are kept as individual elements so downstream models
/// can handle them appropriately.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in text.chars() {
        if c.is_whitespace() {
            flush(&mut current, &mut tokens);
        } else if is_detachable_punct(c) {
            flush(&mut current, &mut tokens);
            tokens.push(c.to_string());
        } else {
            current.push(c);
        }
    }

    flush(&mut current, &mut tokens);
    tokens
}

#[inline]
fn flush(current: &mut String, tokens: &mut Vec<String>) {
    if !current.is_empty() {
        tokens.push(current.clone());
        current.clear();
    }
}

/// Punctuation characters that should always become their own token.
#[inline]
fn is_detachable_punct(c: char) -> bool {
    matches!(
        c,
        '.' | '،'
            | ','
            | '!'
            | '?'
            | '؟'
            | '؛'
            | ';'
            | ':'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '«'
            | '»'
            | '"'
            | '\''
            | '—'
            | '–'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_split() {
        assert_eq!(tokenize("سلام دنیا"), vec!["سلام", "دنیا"]);
    }

    #[test]
    fn keeps_zwnj_compound() {
        // می‌روم is a single compound token
        let tokens = tokenize("می\u{200C}روم به خانه");
        assert_eq!(tokens[0], "می\u{200C}روم");
    }

    #[test]
    fn detaches_punctuation() {
        let t = tokenize("سلام، دنیا!");
        assert!(t.contains(&"،".to_string()));
        assert!(t.contains(&"!".to_string()));
    }

    #[test]
    fn empty_input() {
        assert!(tokenize("").is_empty());
    }
}
