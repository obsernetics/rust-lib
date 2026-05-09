//! Core pipeline orchestrator — [`Parsitext`].

use crate::{
    config::{ParsitextConfig, ProcessingMode},
    diacritics,
    entity::{Entity, EntityRecognizer},
    normalizer::Normalizer,
    sentence,
    stats::{self, TextStats},
    tokenizer, ProcessedText, ProcessingStats,
};

/// The main entry point for all Persian text processing.
///
/// Create once with [`Parsitext::new`] (or use [`Parsitext::default`]) and
/// reuse freely — all regex compilation and automaton construction happens
/// at construction time, **not** per call.
///
/// `Parsitext` is [`Send`] + [`Sync`], so it can live in an `Arc` and be
/// shared across threads.
///
/// # Example
///
/// ```
/// use parsitext::{Parsitext, ParsitextConfig};
///
/// let pt = Parsitext::default();
/// let result = pt.process("سلام داداش، قيمتش حدود 1.5 میلیون تومنه؟");
///
/// // Arabic ي is normalised → Persian ی; Latin digits → Persian.
/// assert!(result.normalized.contains('ی'));
/// assert!(result.normalized.contains('۱'));
/// ```
pub struct Parsitext {
    config: ParsitextConfig,
    normalizer: Normalizer,
    recognizer: Option<EntityRecognizer>,
}

impl Parsitext {
    /// Build a processor from the given configuration.
    ///
    /// This is the only place where regex and automaton compilation happens.
    /// Subsequent calls to any method are allocation-light.
    pub fn new(config: ParsitextConfig) -> Self {
        let normalizer = Normalizer::new(config.clone());
        let recognizer = if config.enable_entity_recognition {
            Some(EntityRecognizer::new())
        } else {
            None
        };
        Self {
            config,
            normalizer,
            recognizer,
        }
    }

    /// Return the configuration this processor was built with.
    pub fn config(&self) -> &ParsitextConfig {
        &self.config
    }

    // ── primary API ───────────────────────────────────────────────────────────

    /// Run the full pipeline: normalise, tokenise, and detect entities.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let result = Parsitext::default().process("كتاب 09121234567");
    /// assert_eq!(result.entities.len(), 1);
    /// assert_eq!(result.normalized, "کتاب ۰۹۱۲۱۲۳۴۵۶۷");
    /// ```
    pub fn process(&self, text: &str) -> ProcessedText {
        let start = std::time::Instant::now();

        let normalized = self.normalizer.normalize(text);
        let tokens = tokenizer::tokenize(&normalized);
        let entities = self.run_entity_detection(&normalized);

        let elapsed_ns = start.elapsed().as_nanos() as u64;
        let token_count = tokens.len();

        ProcessedText {
            stats: ProcessingStats {
                original_length: text.len(),
                normalized_length: normalized.len(),
                token_count,
                processing_time_ns: elapsed_ns,
            },
            original: text.to_owned(),
            normalized,
            tokens,
            entities,
        }
    }

    /// Like [`process`](Self::process) but accepts an already-owned `String`.
    pub fn process_owned(&self, text: String) -> ProcessedText {
        self.process(&text)
    }

    /// Process a slice of texts.
    ///
    /// With the `parallel` feature (enabled by default) this uses Rayon to
    /// process the slice in parallel across all available threads.
    pub fn process_batch<T: AsRef<str> + Sync>(&self, texts: &[T]) -> Vec<ProcessedText> {
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            texts.par_iter().map(|t| self.process(t.as_ref())).collect()
        }
        #[cfg(not(feature = "parallel"))]
        {
            texts.iter().map(|t| self.process(t.as_ref())).collect()
        }
    }

    /// Process `text` with a one-off [`ProcessingMode`] override, ignoring the
    /// mode set in the original config.
    ///
    /// - `MaximumSpeed` skips entity recognition entirely.
    /// - `MaximumQuality` and `Default` run the full pipeline.
    pub fn process_with_mode(&self, text: &str, mode: ProcessingMode) -> ProcessedText {
        if mode == ProcessingMode::MaximumSpeed {
            let start = std::time::Instant::now();
            let normalized = self.normalizer.normalize(text);
            let tokens = tokenizer::tokenize(&normalized);
            let elapsed_ns = start.elapsed().as_nanos() as u64;
            let token_count = tokens.len();
            ProcessedText {
                stats: ProcessingStats {
                    original_length: text.len(),
                    normalized_length: normalized.len(),
                    token_count,
                    processing_time_ns: elapsed_ns,
                },
                original: text.to_owned(),
                normalized,
                tokens,
                entities: Vec::new(),
            }
        } else {
            self.process(text)
        }
    }

    // ── convenience helpers ───────────────────────────────────────────────────

    /// Return only the normalised string (skips tokenisation and entity detection).
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().normalize_only("كتاب"), "کتاب");
    /// ```
    #[must_use]
    pub fn normalize_only(&self, text: &str) -> String {
        self.normalizer.normalize(text)
    }

    /// Normalise, then tokenise.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let tokens = Parsitext::default().tokenize_only("سلام، دنیا!");
    /// assert!(tokens.contains(&"سلام".to_string()));
    /// ```
    #[must_use]
    pub fn tokenize_only(&self, text: &str) -> Vec<String> {
        tokenizer::tokenize(&self.normalizer.normalize(text))
    }

    /// Normalise, then detect entities.
    ///
    /// Equivalent to `process(text).entities` but skips tokenisation.
    #[must_use]
    pub fn detect_entities(&self, text: &str) -> Vec<Entity> {
        let normalized = self.normalizer.normalize(text);
        self.run_entity_detection(&normalized)
    }

    /// Strip Arabic harakat from `text` (does not run the full normalisation
    /// pipeline; only the diacritics pass is applied).
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().remove_diacritics_only("مُحَمَّد"), "محمد");
    /// ```
    #[must_use]
    pub fn remove_diacritics_only(&self, text: &str) -> String {
        diacritics::remove_diacritics(text)
    }

    /// Split `text` into sentences after normalising it.
    ///
    /// Sentence boundaries are `.` `!` `؟` `؛`.  The delimiter stays attached
    /// to the preceding sentence.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let sents = Parsitext::default().split_sentences("سلام. خوبی؟");
    /// assert_eq!(sents, vec!["سلام.", "خوبی؟"]);
    /// ```
    #[must_use]
    pub fn split_sentences(&self, text: &str) -> Vec<String> {
        let normalized = self.normalizer.normalize(text);
        sentence::split_sentences(&normalized)
    }

    /// Compute text statistics after normalising `text`.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let s = Parsitext::default().text_stats("سلام دنیا!");
    /// assert_eq!(s.word_count, 2);
    /// ```
    #[must_use]
    pub fn text_stats(&self, text: &str) -> TextStats {
        let normalized = self.normalizer.normalize(text);
        stats::compute(&normalized)
    }

    // ── numbers / money / stemming convenience ──────────────────────────────

    /// Convert an integer to its Persian word form (delegates to
    /// [`crate::numbers::to_words`]).
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().number_to_words(1234),
    ///            "یک هزار و دویست و سی و چهار");
    /// ```
    #[must_use]
    pub fn number_to_words(&self, n: i64) -> String {
        crate::numbers::to_words(n)
    }

    /// Parse Persian number words back into an integer (delegates to
    /// [`crate::numbers::from_words`]).
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().number_from_words("دو میلیون و پانصد هزار"),
    ///            Some(2_500_000));
    /// ```
    #[must_use]
    pub fn number_from_words(&self, text: &str) -> Option<i64> {
        crate::numbers::from_words(text)
    }

    /// Format an integer with Persian thousand separators.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().format_number(1_234_567), "۱،۲۳۴،۵۶۷");
    /// ```
    #[must_use]
    pub fn format_number(&self, n: i64) -> String {
        crate::numbers::format(n)
    }

    /// Stem a single token using the light Persian stemmer.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().stem("کتاب‌ها"), "کتاب");
    /// ```
    #[must_use]
    pub fn stem(&self, word: &str) -> String {
        crate::stemmer::stem(word)
    }

    /// Tokenise `text` then stem each token.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let stems = Parsitext::default().stem_tokens("کتاب‌های بزرگ‌ترین");
    /// assert!(stems.contains(&"کتاب".to_string()));
    /// assert!(stems.contains(&"بزرگ".to_string()));
    /// ```
    #[must_use]
    pub fn stem_tokens(&self, text: &str) -> Vec<String> {
        let tokens = self.tokenize_only(text);
        crate::stemmer::stem_tokens(&tokens)
    }

    /// Convert Finglish (Persian written in Latin script) to Persian.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().finglish_to_persian("salam"), "سلام");
    /// ```
    #[must_use]
    pub fn finglish_to_persian(&self, text: &str) -> String {
        crate::finglish::to_persian(text)
    }

    /// Convert text (Persian or Finglish) to chat-style Persian.
    #[must_use]
    pub fn to_chat(&self, text: &str) -> String {
        crate::style::to_chat(text)
    }

    /// Convert text (Persian or Finglish) to GenZ-style Persian.
    #[must_use]
    pub fn to_genz(&self, text: &str) -> String {
        crate::style::to_genz(text)
    }

    /// Convert text (Persian or Finglish) to formal written Persian.
    #[must_use]
    pub fn to_formal(&self, text: &str) -> String {
        crate::style::to_formal(text)
    }

    /// Compute the Persian phonetic (Soundex) code for `word`.
    #[must_use]
    pub fn phonetic_code(&self, word: &str) -> String {
        crate::phonetic::soundex(word)
    }

    /// Suggest spell-check corrections from the bundled common-word list.
    #[must_use]
    pub fn spell_suggest(&self, word: &str, max_distance: usize) -> Vec<&'static str> {
        crate::spell::suggest_builtin(word, max_distance)
    }

    /// Transliterate Persian text to Latin (ASCII).
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().transliterate("سلام"), "slam");
    /// ```
    #[must_use]
    pub fn transliterate(&self, text: &str) -> String {
        crate::transliterate::to_latin(text)
    }

    /// Compute Levenshtein edit distance between two strings.
    #[must_use]
    pub fn levenshtein(&self, a: &str, b: &str) -> usize {
        crate::spell::levenshtein(a, b)
    }

    /// Insert ZWNJ at common Persian morphological boundaries (one-shot
    /// helper that calls into [`crate::zwnj_insert::insert`] regardless of
    /// the configured pipeline).
    #[must_use]
    pub fn insert_zwnj(&self, text: &str) -> String {
        crate::zwnj_insert::insert(text)
    }

    /// Parse a Jalali date from a numeric or textual Persian string.
    /// Available with the `jalali` feature.
    #[cfg(feature = "jalali")]
    #[cfg_attr(docsrs, doc(cfg(feature = "jalali")))]
    #[must_use]
    pub fn parse_jalali_date(&self, text: &str) -> Option<jalali_calendar::JalaliDate> {
        crate::jalali::parse(text)
    }

    /// Parse a Persian money expression into a structured [`crate::MoneyAmount`].
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// let m = Parsitext::default().parse_money("۲ میلیون تومان").unwrap();
    /// assert_eq!(m.value, 2_000_000);
    /// ```
    #[must_use]
    pub fn parse_money(&self, text: &str) -> Option<crate::MoneyAmount> {
        crate::money::parse(text)
    }

    // ── language detection ────────────────────────────────────────────────────

    /// Returns `true` if Persian/Arabic letters make up the majority of
    /// alphabetic characters in `text`.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert!(Parsitext::default().is_persian("سلام دنیا"));
    /// assert!(!Parsitext::default().is_persian("hello world"));
    /// ```
    #[must_use]
    pub fn is_persian(&self, text: &str) -> bool {
        let total = text.chars().filter(|c| c.is_alphabetic()).count();
        if total == 0 {
            return false;
        }
        let arabic = text.chars().filter(|c| is_arabic_letter(*c)).count();
        arabic * 2 > total
    }

    /// Returns `true` if `text` contains at least one Arabic/Persian letter.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert!(Parsitext::default().contains_persian("hello سلام"));
    /// assert!(!Parsitext::default().contains_persian("hello world 123"));
    /// ```
    #[must_use]
    pub fn contains_persian(&self, text: &str) -> bool {
        text.chars().any(is_arabic_letter)
    }

    /// Convert canonical Persian letters back to Arabic equivalents
    /// (`ک → ك`, `ی → ي`).  Inverse of the orthography pass.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().to_arabic("کتاب"), "كتاب");
    /// ```
    #[must_use]
    pub fn to_arabic(&self, text: &str) -> String {
        crate::script::to_arabic(text)
    }

    /// Returns `true` if `text` contains an Arabic-only letter that is
    /// not canonical in Persian (ة ى ي ك …).
    #[must_use]
    pub fn has_arabic(&self, text: &str) -> bool {
        crate::script::has_arabic(text)
    }

    /// Describe a signed seconds offset as a Persian relative-time phrase.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// assert_eq!(Parsitext::default().describe_time_diff(-3600), "۱ ساعت پیش");
    /// ```
    #[must_use]
    pub fn describe_time_diff(&self, seconds: i64) -> String {
        crate::time_diff::describe(seconds)
    }

    /// Percent-encode `text` for URL inclusion (RFC 3986).
    #[must_use]
    pub fn url_encode(&self, text: &str) -> String {
        crate::url_fix::encode(text)
    }

    /// Percent-decode `text` (Persian-friendly).
    #[must_use]
    pub fn url_decode(&self, text: &str) -> String {
        crate::url_fix::decode(text)
    }

    // ── internal ─────────────────────────────────────────────────────────────

    fn run_entity_detection(&self, normalized: &str) -> Vec<Entity> {
        match &self.recognizer {
            Some(er) => er.detect(normalized),
            None => Vec::new(),
        }
    }
}

impl Default for Parsitext {
    /// Equivalent to `Parsitext::new(ParsitextConfig::default())`.
    fn default() -> Self {
        Self::new(ParsitextConfig::default())
    }
}

#[inline]
fn is_arabic_letter(c: char) -> bool {
    let cp = c as u32;
    (0x0621..=0x06FF).contains(&cp)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn pt() -> Parsitext {
        Parsitext::default()
    }

    #[test]
    fn full_pipeline_orthography_and_digits() {
        let r = pt().process("سلام داداش، قيمتش ١.٥ میلیون تومنه؟");
        assert!(r.normalized.contains('ی'));
        assert!(r.normalized.contains('۱'));
        assert!(!r.tokens.is_empty());
    }

    #[test]
    fn normalize_only() {
        assert_eq!(pt().normalize_only("كتاب"), "کتاب");
    }

    #[test]
    fn remove_diacritics_only() {
        assert_eq!(pt().remove_diacritics_only("مُحَمَّد"), "محمد");
    }

    #[test]
    fn split_sentences_basic() {
        let s = pt().split_sentences("جمله اول. جمله دوم؟");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn text_stats_word_count() {
        let s = pt().text_stats("سلام دنیا خوبی");
        assert_eq!(s.word_count, 3);
        assert!(s.persian_ratio > 0.9);
    }

    #[test]
    fn is_persian_true() {
        assert!(pt().is_persian("سلام دنیا"));
    }

    #[test]
    fn is_persian_false() {
        assert!(!pt().is_persian("hello world"));
    }

    #[test]
    fn contains_persian_mixed() {
        assert!(pt().contains_persian("hello سلام"));
        assert!(!pt().contains_persian("hello 123"));
    }

    #[test]
    fn batch_length_matches() {
        let texts = vec!["سلام", "دنیا", "ایران"];
        assert_eq!(pt().process_batch(&texts).len(), 3);
    }

    #[test]
    fn default_equals_new_default() {
        let a = Parsitext::default().normalize_only("كتاب");
        let b = Parsitext::new(ParsitextConfig::default()).normalize_only("كتاب");
        assert_eq!(a, b);
    }

    #[test]
    fn process_with_mode_speed_skips_entities() {
        let r = pt().process_with_mode("09121234567", ProcessingMode::MaximumSpeed);
        assert!(r.entities.is_empty());
    }

    #[test]
    fn config_accessor() {
        let pt = Parsitext::default();
        assert!(pt.config().normalize_orthography);
    }
}
