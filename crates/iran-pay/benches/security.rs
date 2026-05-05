use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use iran_pay::security::{
    check_amount, check_authority_format, constant_time_eq, verify_hmac_sha256,
};
use iran_pay::Amount;

fn bench_constant_time_eq(c: &mut Criterion) {
    let mut g = c.benchmark_group("security/constant_time_eq");
    for size in [16usize, 64, 256, 1024] {
        let a = vec![0xABu8; size];
        let b = vec![0xABu8; size];
        g.throughput(Throughput::Bytes(size as u64));
        g.bench_function(format!("equal_{size}"), |bn| {
            bn.iter(|| constant_time_eq(black_box(&a), black_box(&b)))
        });
    }
    g.finish();
}

fn bench_check_authority(c: &mut Criterion) {
    c.bench_function("security/check_authority_format", |b| {
        b.iter(|| check_authority_format(black_box("A0000000000000000000000000000123456789")))
    });
}

fn bench_check_amount(c: &mut Criterion) {
    let a = Amount::toman(50_000);
    let b = Amount::rial(500_000);
    c.bench_function("security/check_amount", |bn| {
        bn.iter(|| check_amount(black_box(a), black_box(b)))
    });
}

fn bench_hmac_verify(c: &mut Criterion) {
    let key = b"super-secret-shared-key";
    let body = br#"{"order_id":"ORD-12345","amount":500000,"track_id":"TX-9999","status":100}"#;
    // Use an obviously-wrong signature to force the failure path; both
    // success and failure paths run the same SHA-256 + constant-time
    // compare so this is a fair benchmark of the hot loop.
    let bad_sig: String = std::iter::repeat_n('a', 64).collect();

    c.bench_function("security/verify_hmac_sha256_failure_path", |bn| {
        bn.iter(|| {
            let _ = verify_hmac_sha256(black_box(key), black_box(body), black_box(&bad_sig));
        })
    });
}

criterion_group!(
    benches,
    bench_constant_time_eq,
    bench_check_authority,
    bench_check_amount,
    bench_hmac_verify,
);
criterion_main!(benches);
