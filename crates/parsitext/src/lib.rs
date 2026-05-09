//! # parsitext
//!
//! High-performance Persian (Farsi) text processing engine for Rust.
//!
//! Built for Iranian production workloads — single-pass normalisation, nine
//! entity-recognition patterns, ZWNJ-aware tokenisation, and Rayon-parallel
//! batch processing.
//!
//! ## Quick start
//!
//! ```
//! use parsitext::{Parsitext, ParsitextConfig};
//!
//! // Use defaults: orthography + digits + ZWNJ + entity detection.
//! let pt = Parsitext::default();
//! let result = pt.process("سلام داداش، قيمتش حدود ١.٥ میلیون تومنه؟");
//!
//! // Arabic ي is normalised to Persian ی; Arabic-Indic ١ → Persian ۱.
//! assert!(result.normalized.contains('ی'));
//! assert!(result.normalized.contains('۱'));
//!
//! // Entity recognised: MoneyAmount.
//! assert!(!result.entities.is_empty());
//! println!("{}", result.entities[0]);
//! ```
//!
//! ## Normalisation pipeline
//!
//! ```text
//! Parsitext::process(text)
//!   └─ Normalizer::normalize
//!        ├─ orthography::fix_arabic_chars      (Arabic ك ي ة → Persian ک ی ه)
//!        ├─ digits::{to_persian,to_latin}      (digit script unification)
//!        ├─ zwnj::normalize_zwnj               (strip misplaced U+200C)
//!        ├─ diacritics::remove_diacritics      (harakat, opt-in)
//!        ├─ spacing::reduce_repetitions        (خیییلی → خییلی)
//!        ├─ spacing::normalize_spaces          (whitespace collapse)
//!        ├─ SlangReplacer                      (goftari→neveshtar, opt-in)
//!        ├─ ProfanityFilter                    (*** replacement, opt-in)
//!        └─ CustomRules                        (user replacements, opt-in)
//!   └─ tokenizer::tokenize                    (whitespace + punctuation split)
//!   └─ EntityRecognizer::detect               (phone, date, money, …)
//! ```
//!
//! ## Feature flags
//!
//! | Feature    | Default | Effect                                         |
//! |------------|---------|------------------------------------------------|
//! | `parallel` | ✓       | Rayon-powered [`Parsitext::process_batch`]     |
//! | `serde`    |         | `Serialize`/`Deserialize` on all public types  |
//!
//! ```toml
//! parsitext = { version = "0.1", features = ["serde"] }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

use std::fmt;

pub mod config;
pub mod diacritics;
pub mod entity;
pub mod finglish;
pub mod geo;
#[cfg(feature = "jalali")]
#[cfg_attr(docsrs, doc(cfg(feature = "jalali")))]
pub mod jalali;
pub mod money;
pub mod numbers;
pub mod phonetic;
pub mod script;
pub mod sentence;
pub mod spell;
pub mod spell_dict;
pub mod stats;
pub mod stemmer;
pub mod style;
#[cfg(feature = "tantivy")]
#[cfg_attr(docsrs, doc(cfg(feature = "tantivy")))]
pub mod tantivy_analyzer;
pub mod time_diff;
pub mod transliterate;
pub mod url_fix;
pub mod validators;
pub mod zwnj_insert;

mod data;
mod engine;
mod normalizer;
mod tokenizer;

pub use config::{
    CustomRule, DigitTarget, ParsitextConfig, ParsitextConfigBuilder, ProcessingMode,
    ProfanityLevel,
};
pub use engine::Parsitext;
pub use entity::{Entity, EntityKind, Span};
pub use money::{MoneyAmount, MoneyUnit};
pub use stats::TextStats;

// ── error ─────────────────────────────────────────────────────────────────────

/// Errors returned by this crate.
///
/// All variants implement [`fmt::Display`] and [`std::error::Error`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// An Aho-Corasick automaton failed to compile from the given pattern.
    ///
    /// The inner `String` is the pattern that triggered the failure.
    PatternBuild(String),
    /// Validation of an Iranian national ID failed (bad length, repunit, or
    /// checksum mismatch).
    InvalidNationalId(String),
    /// Validation of an Iranian Sheba/IBAN failed.
    InvalidIban(String),
    /// Validation of an Iranian phone number failed.
    InvalidPhone(String),
    /// Validation of an Iranian bank-card Luhn checksum failed.
    InvalidBankCard(String),
    /// Validation of an Iranian postal code failed.
    InvalidPostalCode(String),
    /// Parsing or validation of a vehicle plate failed.
    InvalidCarPlate(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::PatternBuild(pat) => {
                write!(f, "failed to compile pattern into automaton: {pat:?}")
            }
            Error::InvalidNationalId(s) => write!(f, "invalid Iranian national ID: {s:?}"),
            Error::InvalidIban(s) => write!(f, "invalid Iranian Sheba (IBAN): {s:?}"),
            Error::InvalidPhone(s) => write!(f, "invalid Iranian phone number: {s:?}"),
            Error::InvalidBankCard(s) => write!(f, "invalid Iranian bank card: {s:?}"),
            Error::InvalidPostalCode(s) => write!(f, "invalid Iranian postal code: {s:?}"),
            Error::InvalidCarPlate(s) => write!(f, "invalid Iranian vehicle plate: {s:?}"),
        }
    }
}

impl std::error::Error for Error {}

// ── processed text ────────────────────────────────────────────────────────────

/// The output of [`Parsitext::process`].
///
/// `Display` renders the `normalized` field.
///
/// ```
/// use parsitext::Parsitext;
///
/// let result = Parsitext::default().process("كتاب");
/// assert_eq!(result.to_string(), "کتاب");
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessedText {
    /// The original input, unchanged.
    pub original: String,
    /// The text after all enabled normalisation passes.
    pub normalized: String,
    /// Tokens produced by the Persian-aware tokeniser.
    pub tokens: Vec<String>,
    /// Structured entities detected in the normalised text.
    pub entities: Vec<Entity>,
    /// Timing and size statistics for this processing run.
    pub stats: ProcessingStats,
}

impl ProcessedText {
    /// Number of tokens in the normalised text.
    #[inline]
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Number of entities recognised.
    #[inline]
    #[must_use]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

impl fmt::Display for ProcessedText {
    /// Renders the normalised text.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.normalized)
    }
}

// ── processing stats ──────────────────────────────────────────────────────────

/// Per-document processing statistics attached to every [`ProcessedText`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProcessingStats {
    /// Length of the original text in bytes.
    pub original_length: usize,
    /// Length of the normalised text in bytes.
    pub normalized_length: usize,
    /// Number of tokens produced.
    pub token_count: usize,
    /// Wall-clock time spent in nanoseconds.
    pub processing_time_ns: u64,
}
