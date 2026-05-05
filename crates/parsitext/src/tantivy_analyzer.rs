//! [Tantivy](https://docs.rs/tantivy) tokenizer and stemmer for Persian text
//! (gated by the `tantivy` Cargo feature).
//!
//! Tantivy ships built-in analyzers for English, German, French, Chinese,
//! and a handful of others — but none for Persian.  This module fills that
//! gap with a [`PersianTokenizer`] that:
//!
//! - Tokenises on whitespace and punctuation **but not** ZWNJ (so compound
//!   words like *می‌روم* stay intact, which is the cardinal sin of every
//!   ASCII-only Persian search index).
//! - Tracks correct UTF-8 byte offsets for highlighting.
//! - Optionally applies the [`crate::stemmer`] light stemmer per token.
//! - Optionally pre-normalises Arabic character variants so `كتاب` and
//!   `کتاب` index identically.
//!
//! ## Usage
//!
//! ```ignore
//! use tantivy::{schema::*, doc, Index};
//! use tantivy::tokenizer::TextAnalyzer;
//! use parsitext::tantivy_analyzer::PersianTokenizer;
//!
//! let mut schema = SchemaBuilder::default();
//! let body = schema.add_text_field("body", TEXT);
//! let schema = schema.build();
//!
//! let index = Index::create_in_ram(schema);
//! index.tokenizers().register(
//!     "persian",
//!     TextAnalyzer::from(PersianTokenizer::new().with_stem(true).with_normalize(true)),
//! );
//! ```
//!
//! Then in your text-field options use
//! `TextFieldIndexing::default().set_tokenizer("persian")`.

use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

use crate::{stemmer, Parsitext, ParsitextConfig};

/// Persian-aware [`Tokenizer`] for tantivy.
///
/// Cheap to clone (which tantivy does per indexing thread) — the heavy
/// regex / automaton compilation in the optional pre-normaliser happens
/// once per [`PersianTokenizer::new`] call.
#[derive(Clone)]
pub struct PersianTokenizer {
    stem: bool,
    /// Pre-normaliser shared across clones for cheap copying.
    normalizer: Option<std::sync::Arc<Parsitext>>,
}

impl Default for PersianTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PersianTokenizer {
    /// New tokenizer with stemming and normalisation off.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stem: false,
            normalizer: None,
        }
    }

    /// Apply [`stemmer::stem`] to each token before indexing.
    #[must_use]
    pub fn with_stem(mut self, stem: bool) -> Self {
        self.stem = stem;
        self
    }

    /// Pre-normalise text (orthography + ZWNJ + digits) before tokenising.
    ///
    /// When `true`, each call constructs a [`Parsitext`] using
    /// [`ParsitextConfig::default`] but with entity recognition disabled
    /// (it would just be wasted work for indexing).  Reuses one instance
    /// across all clones via `Arc`.
    #[must_use]
    pub fn with_normalize(mut self, normalize: bool) -> Self {
        self.normalizer = if normalize {
            let cfg = ParsitextConfig::builder()
                .enable_entity_recognition(false)
                .build();
            Some(std::sync::Arc::new(Parsitext::new(cfg)))
        } else {
            None
        };
        self
    }
}

/// Streaming side of [`PersianTokenizer`].
pub struct PersianTokenStream {
    /// Pre-built tokens — building eagerly is simpler and fast enough,
    /// since indexed Persian docs are usually short paragraphs.
    tokens: Vec<TokenSpan>,
    cursor: usize,
    current: Token,
}

#[derive(Debug, Clone)]
struct TokenSpan {
    text: String,
    byte_start: usize,
    byte_end: usize,
}

impl Tokenizer for PersianTokenizer {
    type TokenStream<'a> = PersianTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let normalized;
        let working: &str = if let Some(pt) = &self.normalizer {
            normalized = pt.normalize_only(text);
            // Note: byte offsets after normalisation no longer correspond to
            // the original text.  We document this and let users opt in.
            normalized.as_str()
        } else {
            text
        };

        let raw_spans = collect_spans(working);
        let tokens: Vec<TokenSpan> = if self.stem {
            raw_spans
                .into_iter()
                .map(|s| TokenSpan {
                    text: stemmer::stem(&s.text),
                    byte_start: s.byte_start,
                    byte_end: s.byte_end,
                })
                .collect()
        } else {
            raw_spans
        };

        PersianTokenStream {
            tokens,
            cursor: 0,
            current: Token::default(),
        }
    }
}

impl TokenStream for PersianTokenStream {
    fn advance(&mut self) -> bool {
        if self.cursor >= self.tokens.len() {
            return false;
        }
        let s = &self.tokens[self.cursor];
        self.current.offset_from = s.byte_start;
        self.current.offset_to = s.byte_end;
        self.current.position = self.cursor;
        self.current.text.clear();
        self.current.text.push_str(&s.text);
        self.cursor += 1;
        true
    }

    fn token(&self) -> &Token {
        &self.current
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.current
    }
}

/// Collect token spans from `text` with byte offsets, ZWNJ-aware.
///
/// Tokens break on whitespace and structural punctuation; ZWNJ (U+200C)
/// is preserved inside tokens so compound words like *می‌روم* stay whole.
fn collect_spans(text: &str) -> Vec<TokenSpan> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let mut buf = String::new();

    for (i, c) in text.char_indices() {
        if is_token_break(c) {
            if let Some(s) = start.take() {
                if !buf.is_empty() {
                    out.push(TokenSpan {
                        text: std::mem::take(&mut buf),
                        byte_start: s,
                        byte_end: i,
                    });
                }
            }
        } else {
            if start.is_none() {
                start = Some(i);
            }
            buf.push(c);
        }
    }
    if let Some(s) = start {
        if !buf.is_empty() {
            out.push(TokenSpan {
                text: buf,
                byte_start: s,
                byte_end: text.len(),
            });
        }
    }
    out
}

#[inline]
fn is_token_break(c: char) -> bool {
    if c == '\u{200C}' {
        return false; // ZWNJ glues compound words together
    }
    c.is_whitespace()
        || matches!(
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

    fn tokens(s: &str, mut tk: PersianTokenizer) -> Vec<String> {
        let mut stream = tk.token_stream(s);
        let mut out = Vec::new();
        while stream.advance() {
            out.push(stream.token().text.clone());
        }
        out
    }

    #[test]
    fn splits_on_whitespace_and_punct() {
        let r = tokens("سلام، دنیا!", PersianTokenizer::new());
        assert_eq!(r, vec!["سلام", "دنیا"]);
    }

    #[test]
    fn keeps_zwnj_inside_token() {
        let r = tokens("می\u{200C}روم به خانه", PersianTokenizer::new());
        assert_eq!(r[0], "می\u{200C}روم");
    }

    #[test]
    fn stem_filter() {
        let r = tokens("کتاب‌ها را خواندم", PersianTokenizer::new().with_stem(true));
        assert!(r[0].contains("کتاب"));
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert!(tokens("", PersianTokenizer::new()).is_empty());
        assert!(tokens("   ", PersianTokenizer::new()).is_empty());
    }

    #[test]
    fn byte_offsets_correct() {
        let text = "سلام دنیا";
        let mut tk = PersianTokenizer::new();
        let mut stream = tk.token_stream(text);
        assert!(stream.advance());
        let t = stream.token();
        assert_eq!(t.offset_from, 0);
        // "سلام" is 4 chars × 2 bytes = 8 bytes.
        assert_eq!(t.offset_to, 8);
        assert!(stream.advance());
        let t = stream.token();
        // After "سلام " (9 bytes): start of "دنیا".
        assert_eq!(t.offset_from, 9);
    }
}
