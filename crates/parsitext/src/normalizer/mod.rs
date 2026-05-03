//! The normalisation pipeline: orthography → digits → ZWNJ → diacritics →
//! repetitions → spaces → slang → profanity → custom rules.

mod digits;
mod orthography;
mod spacing;
mod zwnj;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

use crate::{
    config::{DigitTarget, ParsitextConfig, ProfanityLevel},
    data::{PROFANITY_LIGHT, PROFANITY_MEDIUM, SLANG_PAIRS},
    diacritics,
};

pub(crate) use digits::{to_latin, to_persian};
pub(crate) use orthography::fix_arabic_chars;
pub(crate) use spacing::{normalize_spaces, reduce_repetitions};
pub(crate) use zwnj::normalize_zwnj;

/// The normalisation pipeline, pre-compiled and ready for reuse.
pub(crate) struct Normalizer {
    config: ParsitextConfig,
    slang: Option<WordReplacer>,
    profanity: Option<ProfanityFilter>,
    custom: Option<WordReplacer>,
}

impl Normalizer {
    /// Build a `Normalizer` from a config.  All AhoCorasick automata are
    /// compiled exactly once here.
    pub(crate) fn new(config: ParsitextConfig) -> Self {
        let slang = if config.enable_slang {
            Some(WordReplacer::from_pairs(SLANG_PAIRS))
        } else {
            None
        };

        let profanity = match config.profanity_level {
            ProfanityLevel::None => None,
            ProfanityLevel::Light => Some(ProfanityFilter::new(PROFANITY_LIGHT)),
            ProfanityLevel::Medium => Some(ProfanityFilter::new(PROFANITY_MEDIUM)),
        };

        let custom = if config.custom_rules.is_empty() {
            None
        } else {
            let pairs: Vec<(&str, &str)> = config
                .custom_rules
                .iter()
                .map(|r| (r.pattern.as_str(), r.replacement.as_str()))
                .collect();
            // Build a WordReplacer; whole_word flag per entry is checked at
            // replace time via the stored config.
            Some(WordReplacer::from_dynamic_pairs(&pairs))
        };

        Self {
            config,
            slang,
            profanity,
            custom,
        }
    }

    /// Run every enabled pass in order and return the normalised string.
    pub(crate) fn normalize(&self, text: &str) -> String {
        let mut s = text.to_owned();

        if self.config.normalize_orthography {
            s = fix_arabic_chars(&s);
        }

        match self.config.unify_digits {
            DigitTarget::Persian => s = to_persian(&s),
            DigitTarget::Latin => s = to_latin(&s),
            DigitTarget::None => {}
        }

        if self.config.normalize_zwnj {
            s = normalize_zwnj(&s);
        }

        if self.config.remove_diacritics {
            s = diacritics::remove_diacritics(&s);
        }

        if self.config.reduce_repetitions {
            s = reduce_repetitions(&s);
        }

        if self.config.remove_extra_spaces {
            s = normalize_spaces(&s);
        }

        if self.config.insert_zwnj {
            s = crate::zwnj_insert::insert(&s);
        }

        if let Some(sr) = &self.slang {
            s = sr.replace(&s, true);
        }

        if let Some(pf) = &self.profanity {
            s = pf.replace(&s);
        }

        if let Some(cr) = &self.custom {
            // Respect the whole_word flag stored per rule.  Since we built the
            // automaton from the same order as config.custom_rules, the pattern
            // index maps directly to the rule.
            s = cr.replace_custom(&s, &self.config.custom_rules);
        }

        s
    }
}

// ── word replacer ─────────────────────────────────────────────────────────────

/// Generic whole-word-aware Aho-Corasick replacer.  Used for slang and custom
/// rules.
struct WordReplacer {
    ac: AhoCorasick,
    replacements: Vec<String>,
}

impl WordReplacer {
    fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let patterns: Vec<&str> = pairs.iter().map(|(k, _)| *k).collect();
        let replacements: Vec<String> = pairs.iter().map(|(_, v)| v.to_string()).collect();
        let ac = build_ac(&patterns);
        Self { ac, replacements }
    }

    fn from_dynamic_pairs(pairs: &[(&str, &str)]) -> Self {
        let patterns: Vec<&str> = pairs.iter().map(|(k, _)| *k).collect();
        let replacements: Vec<String> = pairs.iter().map(|(_, v)| v.to_string()).collect();
        let ac = build_ac(&patterns);
        Self { ac, replacements }
    }

    /// Replace patterns, optionally enforcing whole-word boundaries.
    fn replace(&self, text: &str, whole_word: bool) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last_end = 0;

        for m in self.ac.find_iter(text) {
            if whole_word {
                let before = text[..m.start()].chars().last();
                let after = text[m.end()..].chars().next();
                if !is_word_boundary(before) || !is_word_boundary(after) {
                    continue;
                }
            }
            out.push_str(&text[last_end..m.start()]);
            out.push_str(&self.replacements[m.pattern().as_usize()]);
            last_end = m.end();
        }

        out.push_str(&text[last_end..]);
        out
    }

    /// Replace using per-rule `whole_word` settings from `CustomRule` entries.
    fn replace_custom(&self, text: &str, rules: &[crate::config::CustomRule]) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last_end = 0;

        for m in self.ac.find_iter(text) {
            let idx = m.pattern().as_usize();
            let whole_word = rules.get(idx).map(|r| r.whole_word).unwrap_or(false);

            if whole_word {
                let before = text[..m.start()].chars().last();
                let after = text[m.end()..].chars().next();
                if !is_word_boundary(before) || !is_word_boundary(after) {
                    continue;
                }
            }

            out.push_str(&text[last_end..m.start()]);
            out.push_str(&self.replacements[idx]);
            last_end = m.end();
        }

        out.push_str(&text[last_end..]);
        out
    }
}

// ── profanity filter ──────────────────────────────────────────────────────────

struct ProfanityFilter {
    ac: AhoCorasick,
    word_count: usize,
}

impl ProfanityFilter {
    fn new(words: &[&str]) -> Self {
        let ac = build_ac(words);
        Self {
            ac,
            word_count: words.len(),
        }
    }

    fn replace(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut last_end = 0;

        for m in self.ac.find_iter(text) {
            if m.pattern().as_usize() >= self.word_count {
                continue;
            }
            let before = text[..m.start()].chars().last();
            let after = text[m.end()..].chars().next();
            if !is_word_boundary(before) || !is_word_boundary(after) {
                continue;
            }
            out.push_str(&text[last_end..m.start()]);
            out.push_str("***");
            last_end = m.end();
        }

        out.push_str(&text[last_end..]);
        out
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn build_ac(patterns: &[&str]) -> AhoCorasick {
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .build(patterns)
        .expect("parsitext: static patterns are always valid")
}

/// Returns `true` when the optional adjacent character is NOT a word character
/// (i.e. this side is a valid word boundary).
#[inline]
fn is_word_boundary(adjacent: Option<char>) -> bool {
    adjacent.is_none_or(|c| !c.is_alphabetic() && !c.is_numeric())
}
