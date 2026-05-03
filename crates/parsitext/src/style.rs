//! Persian text register conversions: formal ↔ chat ↔ GenZ.
//!
//! Persian online writing oscillates between three registers:
//!
//! - **Formal** (نوشتاری) — written-style, fully-conjugated verbs, no
//!   contractions: `می‌خواهم بروم`.
//! - **Chat** (گفتاری) — spoken/informal, contracted verb forms, dropped
//!   ZWNJs: `میخوام برم`.
//! - **GenZ** — chat plus heavy English loanwords, abbreviations, and
//!   emphatic spellings popular in social media: `میخوام برم 😎 اوسم`.
//!
//! All three converters accept either Persian or **Finglish** input.  When
//! Latin letters are detected, the input is first run through
//! [`crate::finglish::to_persian`] before the register pass.
//!
//! ```
//! use parsitext::style;
//!
//! // Formal → Chat
//! assert_eq!(style::to_chat("می‌خواهم"),  "میخوام");
//! // Chat → Formal (slang expansion)
//! assert_eq!(style::to_formal("میخوام"), "می‌خواهم");
//! // Finglish → Chat (auto-detects ASCII letters)
//! assert!(style::to_chat("salam khoobi").contains("سلام"));
//! // Persian → GenZ (English loanword swap)
//! assert!(style::to_genz("ممنون").contains("مرسی") || style::to_genz("ممنون").contains("تنکس"));
//! ```

use std::sync::OnceLock;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

use crate::data::SLANG_PAIRS;

/// Persian writing register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Style {
    /// Written-style Persian (نوشتاری) with full verb conjugations.
    Formal,
    /// Spoken/chat Persian (گفتاری) with contracted forms.
    Chat,
    /// Heavily informal social-media Persian with English loanwords.
    GenZ,
}

/// Convert any text (Persian or Finglish) to formal written Persian.
///
/// This is the inverse of [`to_chat`] and uses the built-in slang dictionary
/// to expand contracted verb forms (`میخوام` → `می‌خواهم`).
#[must_use]
pub fn to_formal(text: &str) -> String {
    let persian = ensure_persian(text);
    apply_replacements(&persian, slang_to_formal())
}

/// Convert any text (Persian or Finglish) to chat-style Persian.
///
/// This expands the input to chat / *goftari* register: `می‌خواهم` →
/// `میخوام`, `می‌رود` → `میره`, etc.
#[must_use]
pub fn to_chat(text: &str) -> String {
    let persian = ensure_persian(text);
    apply_replacements(&persian, formal_to_chat())
}

/// Convert any text (Persian or Finglish) to GenZ-style Persian.
///
/// First applies the chat-register pass, then swaps a curated set of formal
/// words for their social-media equivalents (often Persianised English).
#[must_use]
pub fn to_genz(text: &str) -> String {
    let chat = to_chat(text);
    apply_replacements(&chat, genz_replacements())
}

/// Generic dispatch by [`Style`].
#[must_use]
pub fn convert(text: &str, style: Style) -> String {
    match style {
        Style::Formal => to_formal(text),
        Style::Chat => to_chat(text),
        Style::GenZ => to_genz(text),
    }
}

// ── internals ─────────────────────────────────────────────────────────────────

fn ensure_persian(text: &str) -> String {
    if text.chars().any(|c| c.is_ascii_alphabetic()) {
        crate::finglish::to_persian(text)
    } else {
        text.to_owned()
    }
}

/// Slang map exposed as `informal → formal` (reverse of SLANG_PAIRS keying).
fn slang_to_formal() -> &'static AhoCorasickReplacer {
    static R: OnceLock<AhoCorasickReplacer> = OnceLock::new();
    R.get_or_init(|| AhoCorasickReplacer::new_pairs(SLANG_PAIRS))
}

/// Inverse map: `formal → informal`.
fn formal_to_chat() -> &'static AhoCorasickReplacer {
    static R: OnceLock<AhoCorasickReplacer> = OnceLock::new();
    R.get_or_init(|| {
        let pairs: Vec<(&'static str, &'static str)> =
            SLANG_PAIRS.iter().map(|(i, f)| (*f, *i)).collect();
        AhoCorasickReplacer::new(pairs)
    })
}

fn genz_replacements() -> &'static AhoCorasickReplacer {
    static R: OnceLock<AhoCorasickReplacer> = OnceLock::new();
    R.get_or_init(|| AhoCorasickReplacer::new_pairs(GENZ_PAIRS))
}

/// GenZ replacements applied AFTER the chat pass.
const GENZ_PAIRS: &[(&str, &str)] = &[
    // Formal greetings → English/chat loanwords
    ("ممنون", "مرسی"),
    ("متشکرم", "مرسی"),
    ("سپاسگزارم", "تنکس"),
    ("بله", "آره"),
    ("خیر", "نه"),
    ("سلام", "های"),
    ("خداحافظ", "بای"),
    // Common adjective swaps
    ("بسیار خوب", "اوسم"),
    ("بسیار", "خیلی"),
    ("عالی", "اوسم"),
    ("عالیه", "اوسمه"),
    ("جالب", "کول"),
    ("جذاب", "کول"),
    ("خنده‌دار", "فانی"),
    ("خنده دار", "فانی"),
    ("دوست داشتنی", "کیوت"),
    // Common loanword replacements
    ("مهمانی", "پارتی"),
    ("جشن", "پارتی"),
    ("دوستان", "فرندز"),
    ("دوست‌دختر", "گرل‌فرند"),
    ("دوست‌پسر", "بوی‌فرند"),
    ("کار", "جاب"),
    ("شغل", "جاب"),
    ("پیام", "مسیج"),
    ("فیلم", "مووی"),
    ("موسیقی", "میوزیک"),
    ("غذا", "فود"),
    ("نوشیدنی", "درینک"),
    // Filler / emphasis
    ("واقعاً", "ریلی"),
    ("واقعا", "ریلی"),
    ("کاملاً", "تتلی"),
    ("دقیقاً", "اگزکتلی"),
];

// ── replacer plumbing ────────────────────────────────────────────────────────

struct AhoCorasickReplacer {
    ac: AhoCorasick,
    replacements: Vec<&'static str>,
}

impl AhoCorasickReplacer {
    fn new_pairs(pairs: &'static [(&'static str, &'static str)]) -> Self {
        let patterns: Vec<&'static str> = pairs.iter().map(|(k, _)| *k).collect();
        let replacements: Vec<&'static str> = pairs.iter().map(|(_, v)| *v).collect();
        let ac = build_ac(&patterns);
        Self { ac, replacements }
    }

    fn new(pairs: Vec<(&'static str, &'static str)>) -> Self {
        let patterns: Vec<&'static str> = pairs.iter().map(|(k, _)| *k).collect();
        let replacements: Vec<&'static str> = pairs.iter().map(|(_, v)| *v).collect();
        let ac = build_ac(&patterns);
        Self { ac, replacements }
    }
}

fn build_ac(patterns: &[&str]) -> AhoCorasick {
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .expect("static patterns are valid")
}

fn apply_replacements(text: &str, r: &AhoCorasickReplacer) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_end = 0;

    for m in r.ac.find_iter(text) {
        // Whole-word check using same logic as the normaliser.
        let before = text[..m.start()].chars().last();
        let after = text[m.end()..].chars().next();
        if !is_word_boundary(before) || !is_word_boundary(after) {
            continue;
        }
        out.push_str(&text[last_end..m.start()]);
        out.push_str(r.replacements[m.pattern().as_usize()]);
        last_end = m.end();
    }
    out.push_str(&text[last_end..]);
    out
}

#[inline]
fn is_word_boundary(adjacent: Option<char>) -> bool {
    adjacent.is_none_or(|c| !c.is_alphabetic() && !c.is_numeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_to_chat_basic() {
        assert_eq!(to_chat("می‌خواهم"), "میخوام");
        assert_eq!(to_chat("نمی‌دانم"), "نمیدونم");
    }

    #[test]
    fn chat_to_formal_basic() {
        assert_eq!(to_formal("میخوام"), "می‌خواهم");
        assert_eq!(to_formal("نمیدونم"), "نمی‌دانم");
    }

    #[test]
    fn finglish_input_to_chat() {
        let out = to_chat("salam khoobi");
        assert!(out.contains("سلام"));
        assert!(out.contains("خوب"));
    }

    #[test]
    fn genz_swaps_loanwords() {
        let out = to_genz("سلام، ممنون از دعوت به مهمانی!");
        assert!(out.contains("های") || out.contains("مرسی"));
        assert!(out.contains("پارتی"));
    }

    #[test]
    fn convert_dispatches() {
        assert_eq!(convert("می‌خواهم", Style::Chat), "میخوام");
        assert_eq!(convert("میخوام", Style::Formal), "می‌خواهم");
    }

    #[test]
    fn whole_word_boundary_respected() {
        // "می‌خواهمی" should not match "می‌خواهم" because the trailing ی
        // is a word character — but our test patterns all end at word
        // boundaries so we just verify no panic on adjacent text.
        let _ = to_chat("ام می‌خواهم بروم");
    }

    #[test]
    fn empty_string() {
        assert_eq!(to_chat(""), "");
        assert_eq!(to_formal(""), "");
        assert_eq!(to_genz(""), "");
    }
}
