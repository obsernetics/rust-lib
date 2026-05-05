use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iran_pay::mock::MockGateway;
use iran_pay::{Amount, Gateway, StartRequest, VerifyRequest};

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn bench_mock_start_payment(c: &mut Criterion) {
    let rt = rt();
    let gw = MockGateway::new();
    let req = StartRequest::builder()
        .amount(Amount::toman(50_000))
        .description("bench")
        .callback_url("https://example.com/cb")
        .build();
    c.bench_function("pipeline/mock_start_payment", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _ = gw.start_payment(black_box(&req)).await;
            });
        });
    });
}

fn bench_mock_round_trip(c: &mut Criterion) {
    let rt = rt();
    let gw = MockGateway::new();
    let req = StartRequest::builder()
        .amount(Amount::toman(50_000))
        .description("bench")
        .callback_url("https://example.com/cb")
        .build();
    c.bench_function("pipeline/mock_start_then_verify", |b| {
        b.iter(|| {
            rt.block_on(async {
                let s = gw.start_payment(&req).await.unwrap();
                let v = VerifyRequest {
                    authority: s.authority,
                    amount: req.amount,
                };
                let _ = gw.verify_payment(black_box(&v)).await;
            });
        });
    });
}

fn bench_amount_construction(c: &mut Criterion) {
    c.bench_function("amount/toman_to_rials", |b| {
        b.iter(|| Amount::toman(black_box(50_000)).as_rials())
    });
}

fn bench_request_builder(c: &mut Criterion) {
    c.bench_function("types/start_request_builder", |b| {
        b.iter(|| {
            StartRequest::builder()
                .amount(Amount::toman(black_box(50_000)))
                .description(black_box("Subscription"))
                .callback_url(black_box("https://example.com/cb"))
                .order_id(black_box("ORD-1"))
                .email(black_box("u@e.com"))
                .mobile(black_box("09121234567"))
                .build()
        })
    });
}

criterion_group!(
    benches,
    bench_mock_start_payment,
    bench_mock_round_trip,
    bench_amount_construction,
    bench_request_builder,
);
criterion_main!(benches);
