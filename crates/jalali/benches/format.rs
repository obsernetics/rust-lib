use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jalali::{JalaliDate, JalaliDateTime};

fn bench_format_iso(c: &mut Criterion) {
    let d = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("format_%Y/%m/%d", |b| {
        b.iter(|| black_box(d).format(black_box("%Y/%m/%d")))
    });
}

fn bench_format_persian(c: &mut Criterion) {
    let d = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("format_persian_long", |b| {
        b.iter(|| black_box(d).format(black_box("%A %-d %B %Y (%K)")))
    });
}

fn bench_parse_iso(c: &mut Criterion) {
    c.bench_function("parse_%Y-%m-%d", |b| {
        b.iter(|| JalaliDate::parse_format(black_box("1403-01-01"), black_box("%Y-%m-%d")).unwrap())
    });
}

fn bench_format_datetime(c: &mut Criterion) {
    let dt = JalaliDateTime::new(1403, 1, 1, 12, 34, 56).unwrap();
    c.bench_function("format_datetime_T", |b| {
        b.iter(|| black_box(dt).format(black_box("%Y/%m/%d %T")))
    });
}

fn bench_parse_datetime(c: &mut Criterion) {
    c.bench_function("parse_datetime_T", |b| {
        b.iter(|| {
            JalaliDateTime::parse_format(black_box("1403/01/01 12:34:56"), black_box("%Y/%m/%d %T"))
                .unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_format_iso,
    bench_format_persian,
    bench_parse_iso,
    bench_format_datetime,
    bench_parse_datetime,
);
criterion_main!(benches);
