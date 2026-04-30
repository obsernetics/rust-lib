use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jalali_calendar::JalaliDate;

fn bench_g_to_j(c: &mut Criterion) {
    c.bench_function("gregorian_to_jalali", |b| {
        b.iter(|| JalaliDate::from_gregorian(black_box(2024), black_box(3), black_box(20)).unwrap())
    });
}

fn bench_j_to_g(c: &mut Criterion) {
    let j = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("jalali_to_gregorian", |b| {
        b.iter(|| black_box(j).to_gregorian())
    });
}

fn bench_add_days_small(c: &mut Criterion) {
    let j = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("add_days_+10", |b| {
        b.iter(|| black_box(j).add_days(black_box(10)))
    });
}

fn bench_add_days_large(c: &mut Criterion) {
    let j = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("add_days_+10_000", |b| {
        b.iter(|| black_box(j).add_days(black_box(10_000)))
    });
}

fn bench_weekday(c: &mut Criterion) {
    let j = JalaliDate::new(1403, 1, 1).unwrap();
    c.bench_function("weekday", |b| b.iter(|| black_box(j).weekday()));
}

fn bench_full_year_walk(c: &mut Criterion) {
    c.bench_function("walk_one_year_366_days", |b| {
        b.iter(|| {
            let mut d = JalaliDate::new(1403, 1, 1).unwrap();
            for _ in 0..365 {
                d = d.add_days(1);
            }
            black_box(d)
        })
    });
}

criterion_group!(
    benches,
    bench_g_to_j,
    bench_j_to_g,
    bench_add_days_small,
    bench_add_days_large,
    bench_weekday,
    bench_full_year_walk,
);
criterion_main!(benches);
