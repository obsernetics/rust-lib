//! Cross-cutting integration tests that exercise multiple modules at once.

use parsitext::{
    CustomRule, EntityKind, Parsitext, ParsitextConfig, ProcessingMode, ProfanityLevel,
};

#[test]
fn full_pipeline_round_trip() {
    let pt = Parsitext::default();
    let result = pt.process("قيمت گوشي يك میلیون 1402 تومانه. با 09121234567 تماس بگیر.");
    // Orthography: Arabic yeh → Persian ی
    assert!(
        result.normalized.contains('ی'),
        "expected Persian yeh in normalized"
    );
    // Digits: Latin 1402 → Persian ۱۴۰۲
    assert!(
        result.normalized.contains('۱'),
        "expected Persian digit in normalized"
    );
    assert!(!result.entities.is_empty(), "expected at least one entity");
    assert!(result.token_count() > 0, "expected non-zero token count");
}

#[test]
fn orthography_plus_digits() {
    let pt = Parsitext::default();
    // Arabic kaf (ك), Arabic yeh (ي), Arabic-Indic digits (١٢٣)
    let result = pt.normalize_only("كتاب يك ١٢٣");
    assert!(result.contains('ک'), "Arabic kaf should become Persian kaf");
    assert!(result.contains('ی'), "Arabic yeh should become Persian yeh");
    assert!(
        result.contains('۱'),
        "Arabic-Indic 1 should become Persian ۱"
    );
    assert!(
        result.contains('۲'),
        "Arabic-Indic 2 should become Persian ۲"
    );
    assert!(
        result.contains('۳'),
        "Arabic-Indic 3 should become Persian ۳"
    );
    assert!(!result.contains('ك'), "Arabic kaf must be gone");
    assert!(!result.contains('ي'), "Arabic yeh must be gone");
}

#[test]
fn zwnj_inside_compound_preserved() {
    let pt = Parsitext::default();
    // ZWNJ between می and روم (both Persian letters) must be kept.
    let input = "می\u{200C}روم به خانه";
    let normalized = pt.normalize_only(input);
    assert!(
        normalized.contains("می\u{200C}روم"),
        "ZWNJ inside می‌روم must be preserved; got: {normalized:?}"
    );
    // ZWNJ next to a space must be stripped.
    let with_bad_zwnj = "سلام \u{200C}دنیا";
    let cleaned = pt.normalize_only(with_bad_zwnj);
    assert!(
        !cleaned.contains('\u{200C}'),
        "ZWNJ next to space must be dropped; got: {cleaned:?}"
    );
}

#[test]
fn repetition_reduction() {
    let pt = Parsitext::default();
    let result = pt.normalize_only("خیییییلی");
    // Max 2 repetitions — must become خییلی, NOT خیلی.
    assert_eq!(result, "خییلی");
}

#[test]
fn diacritics_pass_off_by_default() {
    let pt = Parsitext::default();
    let input = "مُحَمَّد";
    let result = pt.normalize_only(input);
    // Default config has remove_diacritics = false, so harakat must survive.
    assert!(
        result.contains('\u{064F}') || result.contains('\u{064E}') || result.contains('\u{0651}'),
        "diacritics should NOT be removed by default; got: {result:?}"
    );
}

#[test]
fn diacritics_pass_opt_in() {
    let pt = Parsitext::new(ParsitextConfig::builder().remove_diacritics(true).build());
    let result = pt.normalize_only("مُحَمَّد");
    assert_eq!(result, "محمد");
}

#[test]
fn slang_normalisation_whole_word() {
    let pt = Parsitext::new(ParsitextConfig::builder().enable_slang(true).build());
    let result = pt.normalize_only("میخوام بریم");
    assert!(
        result.contains("می\u{200C}خواهم"),
        "میخوام should be normalised to می‌خواهم; got: {result:?}"
    );
    assert!(
        result.contains("برویم"),
        "بریم should be normalised to برویم; got: {result:?}"
    );
    // A word not in the slang list must be unchanged after normalisation.
    let result2 = pt.normalize_only("داریم کار می‌کنیم");
    assert!(
        result2.contains("داریم"),
        "داریم is not in slang list and must remain unchanged; got: {result2:?}"
    );
}

#[test]
fn profanity_light_replaces() {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .profanity_level(ProfanityLevel::Light)
            .build(),
    );
    // احمق is in PROFANITY_LIGHT.
    let result = pt.normalize_only("این احمق اشتباه کرد");
    assert!(
        result.contains("***"),
        "احمق should be replaced with ***; got: {result:?}"
    );
    // Clean text must pass through unchanged.
    let clean = pt.normalize_only("این آدم اشتباه کرد");
    assert!(
        !clean.contains("***"),
        "clean text should not be modified; got: {clean:?}"
    );
}

#[test]
fn custom_rule_whole_word() {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .add_rule(CustomRule::new("ایران", "Iran", true))
            .build(),
    );
    // Standalone match.
    let result = pt.normalize_only("کشور ایران زیباست");
    assert!(
        result.contains("Iran"),
        "standalone ایران should be replaced; got: {result:?}"
    );
    // As a sub-string of a longer word it must NOT be replaced.
    let no_match = pt.normalize_only("ایرانیان");
    assert!(
        !no_match.contains("Iran"),
        "ایران inside ایرانیان must not be replaced with whole_word=true; got: {no_match:?}"
    );
}

#[test]
fn entity_phone_detection() {
    let pt = Parsitext::default();
    let result = pt.process("شماره: 09121234567");
    let phone = result
        .entities
        .iter()
        .find(|e| e.kind == EntityKind::PhoneNumber);
    assert!(phone.is_some(), "expected PhoneNumber entity");
    let phone = phone.unwrap();
    // normalized must be the canonical Latin-digit form.
    assert_eq!(
        phone.normalized.as_deref(),
        Some("09121234567"),
        "canonical form should be 09121234567"
    );
}

#[test]
fn entity_money_detection() {
    let pt = Parsitext::default();
    let result = pt.process("هزینه ۵۰۰ هزار تومان است");
    let money = result
        .entities
        .iter()
        .find(|e| e.kind == EntityKind::MoneyAmount);
    assert!(money.is_some(), "expected MoneyAmount entity");
}

#[test]
fn entity_date_numeric() {
    let pt = Parsitext::default();
    // Latin digits in the input; default config converts them to Persian first,
    // then entity recognition runs on the normalised form.
    let result = pt.process("تاریخ: 1402/03/15 ثبت شد");
    let date = result
        .entities
        .iter()
        .find(|e| e.kind == EntityKind::JalaliDateNumeric);
    assert!(date.is_some(), "expected JalaliDateNumeric entity");
}

#[test]
fn entity_mention_hashtag() {
    let pt = Parsitext::default();
    let result = pt.process("@user123 #tag_test رو ببین");
    let has_mention = result
        .entities
        .iter()
        .any(|e| e.kind == EntityKind::Mention);
    let has_hashtag = result
        .entities
        .iter()
        .any(|e| e.kind == EntityKind::Hashtag);
    assert!(has_mention, "expected Mention entity");
    assert!(has_hashtag, "expected Hashtag entity");
}

#[test]
fn entity_url_email() {
    let pt = Parsitext::default();
    let result = pt.process("سایت https://example.ir و ایمیل info@example.com");
    let has_url = result.entities.iter().any(|e| e.kind == EntityKind::Url);
    let has_email = result.entities.iter().any(|e| e.kind == EntityKind::Email);
    assert!(has_url, "expected Url entity");
    assert!(has_email, "expected Email entity");
}

#[test]
fn batch_equals_sequential() {
    let pt = Parsitext::default();
    let texts = vec![
        "قيمت گوشي يك میلیون تومانه",
        "كتاب خوبي بود",
        "میخوام بریم بیرون",
        "09121234567 تماس بگیر",
    ];
    let batch = pt.process_batch(&texts);
    for (i, text) in texts.iter().enumerate() {
        let seq = pt.process(text);
        assert_eq!(
            batch[i].normalized, seq.normalized,
            "batch and sequential normalized differ at index {i}"
        );
    }
}

#[test]
fn max_speed_mode_skips_entities() {
    let pt = Parsitext::default();
    let result = pt.process_with_mode(
        "09121234567 https://example.ir",
        ProcessingMode::MaximumSpeed,
    );
    assert!(
        result.entities.is_empty(),
        "MaximumSpeed must skip entity recognition"
    );
}

#[test]
fn text_stats_consistency() {
    let pt = Parsitext::default();
    let text = "سلام دنیا! امروز روز خوبی است.";
    let stats = pt.text_stats(text);
    let tokens = pt.tokenize_only(text);
    // word_count counts only alphabetic/numeric tokens; tokenize_only also
    // returns punctuation tokens.  word_count ≤ tokens.len().
    assert!(
        stats.word_count <= tokens.len(),
        "word_count ({}) must not exceed token count ({})",
        stats.word_count,
        tokens.len()
    );
    assert!(
        stats.persian_ratio > 0.0 && stats.persian_ratio <= 1.0,
        "persian_ratio must be in (0, 1]; got {}",
        stats.persian_ratio
    );
}

#[test]
fn split_sentences_count() {
    let pt = Parsitext::default();
    // Three sentence-ending delimiters → three sentences.
    let sents = pt.split_sentences("سلام. چطوری؟ خوبم!");
    assert_eq!(sents.len(), 3, "expected 3 sentences, got {}", sents.len());
}

#[test]
fn is_persian_and_contains_persian() {
    let pt = Parsitext::default();
    // Empty string: is_persian must be false (no alphabetic chars).
    assert!(!pt.is_persian(""), "empty string must not be Persian");
    // All-Latin: is_persian false, contains_persian false.
    assert!(
        !pt.is_persian("hello world"),
        "all-Latin must not be Persian"
    );
    assert!(
        !pt.contains_persian("hello world 123"),
        "all-Latin must not contain Persian"
    );
    // All-Persian: both true.
    assert!(pt.is_persian("سلام دنیا"), "all-Persian must be Persian");
    assert!(
        pt.contains_persian("سلام"),
        "all-Persian must contain Persian"
    );
    // Mixed: contains_persian true; is_persian depends on majority.
    assert!(
        pt.contains_persian("hello سلام"),
        "mixed must contain Persian"
    );
    assert!(
        !pt.is_persian("hello world و"),
        "one Persian letter among many Latin must not trigger is_persian"
    );
}

#[test]
fn processed_text_display() {
    let result = Parsitext::default().process("كتاب");
    assert_eq!(
        result.to_string(),
        result.normalized,
        "Display on ProcessedText must equal its normalized field"
    );
}

#[test]
fn span_len_matches_text_len() {
    let pt = Parsitext::default();
    let result = pt.process("@user123 #tag https://example.ir 09121234567");
    for entity in &result.entities {
        assert_eq!(
            entity.span.len(),
            entity.text.len(),
            "span length {} != text byte length {} for entity {:?}",
            entity.span.len(),
            entity.text.len(),
            entity.text,
        );
    }
}

#[test]
fn processing_stats_nonzero() {
    let result = Parsitext::default().process("سلام دنیا!");
    assert!(
        result.stats.processing_time_ns > 0,
        "processing_time_ns must be > 0"
    );
    assert_eq!(
        result.stats.token_count,
        result.tokens.len(),
        "stats.token_count must match tokens.len()"
    );
}

#[cfg(feature = "serde")]
#[test]
fn serde_processed_text_round_trip() {
    let result = Parsitext::default().process("سلام");
    let json = serde_json::to_string(&result).unwrap();
    let back: parsitext::ProcessedText = serde_json::from_str(&json).unwrap();
    assert_eq!(back.normalized, result.normalized);
}
