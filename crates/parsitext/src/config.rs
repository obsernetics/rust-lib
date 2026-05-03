//! Configuration types and builder for [`Parsitext`](crate::Parsitext).

use std::fmt;

// ── digit target ─────────────────────────────────────────────────────────────

/// The script to normalise digits into.
///
/// All three digit systems — Persian (۰–۹), Arabic-Indic (٠–٩), and Latin
/// (0–9) — can appear in Iranian text. This enum selects the canonical output
/// form.
///
/// ```
/// use parsitext::{Parsitext, ParsitextConfig, DigitTarget};
///
/// let pt = Parsitext::new(ParsitextConfig::builder().unify_digits(DigitTarget::Latin).build());
/// assert_eq!(pt.normalize_only("قیمت: ۱۵۰۰"), "قیمت: 1500");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DigitTarget {
    /// Convert all digits to Persian (۰–۹). **Default.**
    #[default]
    Persian,
    /// Convert all digits to Latin (0–9).
    Latin,
    /// Leave digits unchanged.
    None,
}

// ── profanity level ───────────────────────────────────────────────────────────

/// Controls how aggressively profanity is filtered.
///
/// Replacement text is always `"***"`.  Both levels use whole-word matching so
/// substrings inside unrelated words are never touched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProfanityLevel {
    /// No filtering. **Default.**
    #[default]
    None,
    /// Replace a small, high-confidence set of Persian insult words.
    Light,
    /// Replace a broader set, including stronger offensive terms.
    Medium,
}

// ── processing mode ───────────────────────────────────────────────────────────

/// Hint to the engine about the speed / quality trade-off.
///
/// The mode can be overridden per-call via
/// [`Parsitext::process_with_mode`](crate::Parsitext::process_with_mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProcessingMode {
    /// Run every pass that is enabled in the config. **Default.**
    #[default]
    Default,
    /// Skip entity recognition to maximise throughput.
    MaximumSpeed,
    /// Identical to `Default` for now; reserved for future heavier analysis.
    MaximumQuality,
}

// ── custom rule ───────────────────────────────────────────────────────────────

/// A single user-supplied text replacement applied after all built-in passes.
///
/// Patterns are matched with Aho-Corasick (leftmost-first).  When
/// `whole_word` is `true`, the match is rejected if the immediately adjacent
/// characters are Unicode alphabetic or numeric — preventing partial-word
/// replacement.
///
/// # Example
///
/// ```
/// use parsitext::{Parsitext, ParsitextConfig, CustomRule};
///
/// let rule = CustomRule { pattern: "دوستانه".into(), replacement: "دوستانه‌ای".into(), whole_word: true };
/// let pt = Parsitext::new(ParsitextConfig::builder().custom_rules(vec![rule]).build());
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomRule {
    /// Literal text to search for.
    pub pattern: String,
    /// Text to substitute in place of the match.
    pub replacement: String,
    /// When `true`, only replace at Unicode word boundaries.
    pub whole_word: bool,
}

impl CustomRule {
    /// Convenience constructor.
    pub fn new(
        pattern: impl Into<String>,
        replacement: impl Into<String>,
        whole_word: bool,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            whole_word,
        }
    }
}

impl fmt::Display for CustomRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} → {:?}", self.pattern, self.replacement)
    }
}

// ── main config ───────────────────────────────────────────────────────────────

/// Full configuration for a [`Parsitext`](crate::Parsitext) processor.
///
/// Build with [`ParsitextConfig::builder()`] or use
/// [`ParsitextConfig::default()`] for sensible production defaults.
///
/// ```
/// use parsitext::{ParsitextConfig, DigitTarget, ProfanityLevel};
///
/// let config = ParsitextConfig::builder()
///     .unify_digits(DigitTarget::Latin)
///     .profanity_level(ProfanityLevel::Light)
///     .enable_slang(true)
///     .build();
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsitextConfig {
    /// Normalise ZWNJ (U+200C) placement — strip stray ZWNJs while keeping
    /// those that correctly join compound-word components.
    pub normalize_zwnj: bool,
    /// Unify all digit scripts to a single target.
    pub unify_digits: DigitTarget,
    /// Replace Arabic character variants with Persian canonical forms
    /// (`ك→ک`, `ي→ی`, `ة→ه`, etc.).
    pub normalize_orthography: bool,
    /// Collapse multiple consecutive whitespace characters to a single space
    /// and trim the result.
    pub remove_extra_spaces: bool,
    /// Shorten emphatic character repetitions to at most two: `خیییلی→خییلی`.
    pub reduce_repetitions: bool,
    /// Strip Arabic harakat (fathah, kasrah, shadda, sukun, etc., U+064B–U+065F).
    pub remove_diacritics: bool,
    /// Insert ZWNJ at common Persian morphological boundaries (verb prefix
    /// `می`/`نمی`, plural `ها`/`های`, possessive `ام`/`ات`/`اش`, …).  Opt-in
    /// because the heuristic can over-insert on non-verb words.
    pub insert_zwnj: bool,
    /// Normalise common informal (*goftari*) phonetic variants to written
    /// Persian (*neveshtar*).
    pub enable_slang: bool,
    /// Detect and annotate structured entities (phone, date, money, …).
    pub enable_entity_recognition: bool,
    /// Profanity filter level.
    pub profanity_level: ProfanityLevel,
    /// Speed / quality trade-off hint for [`Parsitext::process`](crate::Parsitext::process).
    pub mode: ProcessingMode,
    /// Zero or more user-supplied literal replacements applied after all
    /// built-in passes.
    pub custom_rules: Vec<CustomRule>,
}

impl ParsitextConfig {
    /// Create a new [`ParsitextConfigBuilder`] pre-filled with defaults.
    ///
    /// Defaults: ZWNJ normalisation on, digits unified to Persian, orthography
    /// normalised, spaces collapsed, repetitions reduced, diacritics kept,
    /// slang off, entity recognition on, no profanity filtering, no custom
    /// rules.
    pub fn builder() -> ParsitextConfigBuilder {
        ParsitextConfigBuilder::default()
    }
}

impl Default for ParsitextConfig {
    /// Returns the default configuration (same as `ParsitextConfig::builder().build()`).
    fn default() -> Self {
        ParsitextConfigBuilder::default().build()
    }
}

// ── builder ───────────────────────────────────────────────────────────────────

/// Step-builder for [`ParsitextConfig`].
///
/// Obtain via [`ParsitextConfig::builder()`].
#[derive(Debug)]
pub struct ParsitextConfigBuilder {
    inner: ParsitextConfig,
}

impl Default for ParsitextConfigBuilder {
    fn default() -> Self {
        Self {
            inner: ParsitextConfig {
                normalize_zwnj: true,
                unify_digits: DigitTarget::Persian,
                normalize_orthography: true,
                remove_extra_spaces: true,
                reduce_repetitions: true,
                remove_diacritics: false,
                insert_zwnj: false,
                enable_slang: false,
                enable_entity_recognition: true,
                profanity_level: ProfanityLevel::None,
                mode: ProcessingMode::Default,
                custom_rules: Vec::new(),
            },
        }
    }
}

impl ParsitextConfigBuilder {
    /// Toggle ZWNJ normalisation.
    pub fn normalize_zwnj(mut self, v: bool) -> Self {
        self.inner.normalize_zwnj = v;
        self
    }

    /// Set the target digit script.
    pub fn unify_digits(mut self, target: DigitTarget) -> Self {
        self.inner.unify_digits = target;
        self
    }

    /// Toggle Arabic→Persian orthography fixes.
    pub fn normalize_orthography(mut self, v: bool) -> Self {
        self.inner.normalize_orthography = v;
        self
    }

    /// Toggle whitespace collapse.
    pub fn remove_extra_spaces(mut self, v: bool) -> Self {
        self.inner.remove_extra_spaces = v;
        self
    }

    /// Toggle emphatic-repetition reduction.
    pub fn reduce_repetitions(mut self, v: bool) -> Self {
        self.inner.reduce_repetitions = v;
        self
    }

    /// Toggle Arabic harakat removal.
    pub fn remove_diacritics(mut self, v: bool) -> Self {
        self.inner.remove_diacritics = v;
        self
    }

    /// Toggle heuristic ZWNJ insertion at morphological boundaries.
    pub fn insert_zwnj(mut self, v: bool) -> Self {
        self.inner.insert_zwnj = v;
        self
    }

    /// Toggle informal (*goftari*) slang normalisation.
    pub fn enable_slang(mut self, v: bool) -> Self {
        self.inner.enable_slang = v;
        self
    }

    /// Toggle entity recognition.
    pub fn enable_entity_recognition(mut self, v: bool) -> Self {
        self.inner.enable_entity_recognition = v;
        self
    }

    /// Set the profanity filter level.
    pub fn profanity_level(mut self, level: ProfanityLevel) -> Self {
        self.inner.profanity_level = level;
        self
    }

    /// Set the speed / quality mode.
    pub fn mode(mut self, mode: ProcessingMode) -> Self {
        self.inner.mode = mode;
        self
    }

    /// Set user-defined replacement rules (applied last in the pipeline).
    pub fn custom_rules(mut self, rules: Vec<CustomRule>) -> Self {
        self.inner.custom_rules = rules;
        self
    }

    /// Append a single custom rule.
    pub fn add_rule(mut self, rule: CustomRule) -> Self {
        self.inner.custom_rules.push(rule);
        self
    }

    /// Consume the builder and produce a [`ParsitextConfig`].
    pub fn build(self) -> ParsitextConfig {
        self.inner
    }
}
