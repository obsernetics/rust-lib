use criterion::{black_box, criterion_group, criterion_main, Criterion};
use parsitext::{DigitTarget, Parsitext, ParsitextConfig};

const SAMPLE: &str = "سلام داداش، چطوري؟ قيمت خونه خيليييي گرونه، حدود 2 میلیون تومنه! \
     ممنون از كمكت، واقعاً عاليييي بود! نمیدونم چی بگم.";

fn bench_orthography(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(true)
            .unify_digits(DigitTarget::None)
            .normalize_zwnj(false)
            .reduce_repetitions(false)
            .remove_extra_spaces(false)
            .enable_entity_recognition(false)
            .build(),
    );
    c.bench_function("normalizer/orthography", |b| {
        b.iter(|| pt.normalize_only(black_box(SAMPLE)));
    });
}

fn bench_digits_to_persian(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(false)
            .unify_digits(DigitTarget::Persian)
            .normalize_zwnj(false)
            .reduce_repetitions(false)
            .remove_extra_spaces(false)
            .enable_entity_recognition(false)
            .build(),
    );
    let mixed = "قیمت: ۱٢3,456 تومان — تاریخ: 1402/٠3/15";
    c.bench_function("normalizer/digits_to_persian", |b| {
        b.iter(|| pt.normalize_only(black_box(mixed)));
    });
}

fn bench_digits_to_latin(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(false)
            .unify_digits(DigitTarget::Latin)
            .normalize_zwnj(false)
            .reduce_repetitions(false)
            .remove_extra_spaces(false)
            .enable_entity_recognition(false)
            .build(),
    );
    let persian = "قیمت: ۱۵۰۰۰۰۰ تومان";
    c.bench_function("normalizer/digits_to_latin", |b| {
        b.iter(|| pt.normalize_only(black_box(persian)));
    });
}

fn bench_zwnj(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(false)
            .unify_digits(DigitTarget::None)
            .normalize_zwnj(true)
            .reduce_repetitions(false)
            .remove_extra_spaces(false)
            .enable_entity_recognition(false)
            .build(),
    );
    let text = "می\u{200C}روم \u{200C}خانه\u{200C} می‌خواهم \u{200C}\u{200C}بروم";
    c.bench_function("normalizer/zwnj", |b| {
        b.iter(|| pt.normalize_only(black_box(text)));
    });
}

fn bench_repetitions(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(false)
            .unify_digits(DigitTarget::None)
            .normalize_zwnj(false)
            .reduce_repetitions(true)
            .remove_extra_spaces(false)
            .enable_entity_recognition(false)
            .build(),
    );
    c.bench_function("normalizer/repetitions", |b| {
        b.iter(|| pt.normalize_only(black_box(SAMPLE)));
    });
}

fn bench_diacritics(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .normalize_orthography(false)
            .unify_digits(DigitTarget::None)
            .normalize_zwnj(false)
            .reduce_repetitions(false)
            .remove_extra_spaces(false)
            .remove_diacritics(true)
            .enable_entity_recognition(false)
            .build(),
    );
    let diacritized = "مُحَمَّدٌ كَتَبَ الْكِتَابَ";
    c.bench_function("normalizer/diacritics", |b| {
        b.iter(|| pt.normalize_only(black_box(diacritized)));
    });
}

fn bench_full_normalize(c: &mut Criterion) {
    let pt = Parsitext::default();
    c.bench_function("normalizer/full_pipeline", |b| {
        b.iter(|| pt.normalize_only(black_box(SAMPLE)));
    });
}

fn bench_slang(c: &mut Criterion) {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            .enable_slang(true)
            .enable_entity_recognition(false)
            .build(),
    );
    let slang_text = "میخوام بریم خونه نمیدونم چی بگم میتونم بیام";
    c.bench_function("normalizer/slang", |b| {
        b.iter(|| pt.normalize_only(black_box(slang_text)));
    });
}

criterion_group!(
    benches,
    bench_orthography,
    bench_digits_to_persian,
    bench_digits_to_latin,
    bench_zwnj,
    bench_repetitions,
    bench_diacritics,
    bench_full_normalize,
    bench_slang,
);
criterion_main!(benches);
