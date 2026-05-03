use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use parsitext::{Parsitext, ProcessingMode};

const SHORT: &str = "سلام داداش، چطوری؟ قيمتش حدود 2 میلیون تومنه! شماره: 09121234567";

const LONG: &str = concat!(
    "سلام داداش، چطوری؟ قيمت خونه خيليييي گرونه، حدود 2 میلیون تومنه! ",
    "شماره من اينه: 09121234567 — زنگ بزن. ",
    "تاریخ: 1402/03/15. پیام رو به @reza123 بده و #ایران_آزمایش رو دنبال کن. ",
    "لینک: https://example.ir/products?id=42 ",
    "ممنون از كمكت، واقعاً عاليييي بود! نمیدونم چی بگم، خیییلی ممنونم. ",
    "كد ملي: 0023456789 — چک کن. هزینه: ۵۰۰ هزار ریال",
);

fn bench_process(c: &mut Criterion) {
    let pt = Parsitext::default();
    let mut g = c.benchmark_group("pipeline/process");

    for (label, text) in [("short", SHORT), ("long", LONG)] {
        g.throughput(Throughput::Bytes(text.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), text, |b, t| {
            b.iter(|| pt.process(black_box(t)));
        });
    }

    g.finish();
}

fn bench_process_mode_speed(c: &mut Criterion) {
    let pt = Parsitext::default();
    c.bench_function("pipeline/max_speed", |b| {
        b.iter(|| pt.process_with_mode(black_box(LONG), ProcessingMode::MaximumSpeed));
    });
}

fn bench_entity_detection(c: &mut Criterion) {
    let pt = Parsitext::default();
    c.bench_function("pipeline/entity_detect", |b| {
        b.iter(|| pt.detect_entities(black_box(LONG)));
    });
}

fn bench_sentence_split(c: &mut Criterion) {
    let pt = Parsitext::default();
    let para = "جمله اول است. جمله دوم است؟ جمله سوم است! جمله چهارم است. جمله پنجم است؟";
    c.bench_function("pipeline/sentence_split", |b| {
        b.iter(|| pt.split_sentences(black_box(para)));
    });
}

fn bench_text_stats(c: &mut Criterion) {
    let pt = Parsitext::default();
    c.bench_function("pipeline/text_stats", |b| {
        b.iter(|| pt.text_stats(black_box(LONG)));
    });
}

fn bench_batch(c: &mut Criterion) {
    let pt = Parsitext::default();
    let mut g = c.benchmark_group("pipeline/batch");

    for size in [10usize, 100, 500] {
        let batch: Vec<&str> = std::iter::repeat_n(SHORT, size).collect();
        g.throughput(Throughput::Elements(size as u64));
        g.bench_with_input(BenchmarkId::from_parameter(size), &batch, |b, texts| {
            b.iter(|| pt.process_batch(black_box(texts.as_slice())));
        });
    }

    g.finish();
}

fn bench_tokenize(c: &mut Criterion) {
    let pt = Parsitext::default();
    c.bench_function("pipeline/tokenize", |b| {
        b.iter(|| pt.tokenize_only(black_box(LONG)));
    });
}

criterion_group!(
    benches,
    bench_process,
    bench_process_mode_speed,
    bench_entity_detection,
    bench_sentence_split,
    bench_text_stats,
    bench_batch,
    bench_tokenize,
);
criterion_main!(benches);
