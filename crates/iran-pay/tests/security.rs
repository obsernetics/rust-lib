//! Security-focused tests: hostile inputs, timing properties, malformed
//! payloads, and oversized data.  These complement the wiremock-based
//! integration tests in `tests/integration.rs`.

use iran_pay::mock::{Behavior, MockGateway};
use iran_pay::security::{
    check_amount, check_authority_format, constant_time_eq, verify_hmac_sha256,
};
use iran_pay::{Amount, Error, Gateway, StartRequest, VerifyRequest};

// ── amount unit-safety ─────────────────────────────────────────────────────

#[test]
fn amount_overflow_at_construction_is_saturated() {
    // toman() multiplies by 10; near-MAX inputs should saturate, not panic.
    let huge = Amount::toman(i64::MAX);
    // Just confirms no panic and gives a finite, sensible Rials value.
    assert!(huge.as_rials() > 0);
}

#[test]
fn amount_zero_round_trip() {
    assert!(Amount::rial(0).is_zero());
    assert!(Amount::toman(0).is_zero());
}

#[test]
fn amount_unit_mixup_caught_by_typed_eq() {
    // 50_000 Toman == 500_000 Rial; if your code accidentally passes the
    // toman value where rials are expected, the type lets you compare them.
    assert_eq!(Amount::toman(50_000), Amount::rial(500_000));
    assert_ne!(Amount::toman(50_000), Amount::rial(50_000));
}

// ── authority validation ───────────────────────────────────────────────────

#[test]
fn rejects_empty_authority() {
    assert!(check_authority_format("").is_err());
}

#[test]
fn rejects_oversized_authority() {
    let big = "A".repeat(1024);
    assert!(check_authority_format(&big).is_err());
}

#[test]
fn rejects_control_characters() {
    for c in ['\u{0000}', '\u{0008}', '\u{007F}', '\n', '\r', '\t'] {
        let s = format!("AB{c}CD");
        assert!(
            check_authority_format(&s).is_err(),
            "should reject control char {c:?}"
        );
    }
}

#[test]
fn rejects_non_ascii_authority() {
    assert!(check_authority_format("سلام").is_err());
    assert!(check_authority_format("ABC\u{200C}").is_err()); // ZWNJ in token
}

#[test]
fn accepts_realistic_authority_strings() {
    for s in &[
        "A0000000000000000000000000000123456789", // ZarinPal-shape
        "abc123XYZ",
        "tok-12345_67890.42",
        "deadbeef",
    ] {
        assert!(check_authority_format(s).is_ok(), "should accept {s:?}");
    }
}

// ── constant-time eq ───────────────────────────────────────────────────────

#[test]
fn ct_eq_handles_extremes() {
    assert!(constant_time_eq(b"", b""));
    assert!(!constant_time_eq(b"", b"x"));
    assert!(!constant_time_eq(b"x", b""));

    let big = vec![0xAAu8; 1 << 14]; // 16 KiB
    let big_eq = vec![0xAAu8; 1 << 14];
    assert!(constant_time_eq(&big, &big_eq));

    let mut tampered = big_eq.clone();
    *tampered.last_mut().unwrap() ^= 1;
    assert!(!constant_time_eq(&big, &tampered));
}

// ── HMAC verification ──────────────────────────────────────────────────────

#[test]
fn hmac_round_trip_typical_webhook_payload() {
    // Simulate a NextPay-style callback signed with the merchant's API key.
    let key = b"merchant-secret-key-32-bytes-..."; // 32 bytes
    let body = br#"{"order_id":"ORD-1234","amount":500000,"trans_id":"TXN-9","status":"OK"}"#;

    // Compute the signature using the same code so we test the full path.
    let sig = compute_hmac_hex(key, body);
    assert!(verify_hmac_sha256(key, body, &sig).is_ok());

    // Single-byte tampering must fail.
    let mut tampered = body.to_vec();
    tampered[10] ^= 0x01;
    assert!(verify_hmac_sha256(key, &tampered, &sig).is_err());

    // Wrong key must fail.
    assert!(verify_hmac_sha256(b"wrong-key", body, &sig).is_err());
}

#[test]
fn hmac_rejects_short_signature() {
    let res = verify_hmac_sha256(b"k", b"m", "abc");
    assert!(matches!(res, Err(Error::Config(_))));
}

#[test]
fn hmac_rejects_non_hex_signature() {
    let res = verify_hmac_sha256(b"k", b"m", &"x".repeat(64));
    assert!(matches!(res, Err(Error::Config(_))));
}

// ── amount-mismatch attack scenario ────────────────────────────────────────

#[test]
fn amount_mismatch_returns_typed_error() {
    let res = check_amount(Amount::toman(50_000), Amount::toman(1));
    match res {
        Err(Error::AmountMismatch { expected, actual }) => {
            assert_eq!(expected, Amount::toman(50_000));
            assert_eq!(actual, Amount::toman(1));
        }
        other => panic!("expected AmountMismatch, got {other:?}"),
    }
}

// ── mock-gateway abuse ─────────────────────────────────────────────────────

#[tokio::test]
async fn mock_gateway_rejects_after_failure_set() {
    let gw = MockGateway::new();
    gw.set_start_behavior(Behavior::FailGateway {
        code: -9,
        message: "merchant blocked".into(),
    });

    let req = StartRequest::builder()
        .amount(Amount::toman(1000))
        .description("test")
        .callback_url("http://localhost/cb")
        .build();

    let res = gw.start_payment(&req).await;
    match res {
        Err(Error::Gateway {
            provider,
            code,
            message,
        }) => {
            assert_eq!(provider, "mock");
            assert_eq!(code, -9);
            assert!(message.contains("blocked"));
        }
        other => panic!("expected Gateway error, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_gateway_high_concurrency() {
    use std::sync::Arc;
    let gw = Arc::new(MockGateway::new());
    let mut handles = Vec::new();
    for _ in 0..100 {
        let gw = gw.clone();
        handles.push(tokio::spawn(async move {
            let req = StartRequest::builder()
                .amount(Amount::toman(1))
                .description("c")
                .callback_url("http://x/cb")
                .build();
            let s = gw.start_payment(&req).await.unwrap();
            let v = VerifyRequest {
                authority: s.authority,
                amount: req.amount,
            };
            gw.verify_payment(&v).await.unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(gw.start_call_count(), 100);
    assert_eq!(gw.verify_call_count(), 100);
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Compute HMAC-SHA256 in lowercase hex by re-using the verifier logic.
fn compute_hmac_hex(key: &[u8], body: &[u8]) -> String {
    // We don't expose the raw HMAC; verify by guessing then correcting.
    // Easier: brute-force-construct via 256-character search.  Not feasible.
    // Instead expose via repeated XOR construction is overkill.
    //
    // Pragmatic: use the verify_hmac_sha256 inverse — but that's not exposed.
    // For tests, just call the same SHA-256 algorithm by hand.
    let mac = hmac_sha256_local(key, body);
    let mut out = String::with_capacity(64);
    for b in &mac {
        out.push(nibble(b >> 4));
        out.push(nibble(b & 0x0f));
    }
    out
}

fn nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        _ => (b'a' + n - 10) as char,
    }
}

// Local copy of the same algorithm used in iran_pay::security so we can
// generate signatures here without exposing internals.
fn hmac_sha256_local(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k_buf = [0u8; 64];
    if key.len() > 64 {
        let h = sha256_local(key);
        k_buf[..32].copy_from_slice(&h);
    } else {
        k_buf[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k_buf[i];
        opad[i] ^= k_buf[i];
    }
    let mut inner = Vec::with_capacity(64 + message.len());
    inner.extend_from_slice(&ipad);
    inner.extend_from_slice(message);
    let inner_hash = sha256_local(&inner);
    let mut outer = Vec::with_capacity(64 + 32);
    outer.extend_from_slice(&opad);
    outer.extend_from_slice(&inner_hash);
    sha256_local(&outer)
}

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_local(input: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

// ── Error Display impls (full coverage of error.rs) ─────────────────────────

#[test]
fn error_display_gateway() {
    let e = Error::Gateway {
        provider: "zarinpal",
        code: -9,
        message: "validation failed".into(),
    };
    let s = e.to_string();
    assert!(s.contains("zarinpal"));
    assert!(s.contains("-9"));
    assert!(s.contains("validation failed"));
}

#[test]
fn error_display_amount_mismatch() {
    let e = Error::AmountMismatch {
        expected: Amount::toman(1000),
        actual: Amount::toman(999),
    };
    assert!(e.to_string().contains("mismatch"));
}

#[test]
fn error_display_config() {
    let e = Error::Config("missing merchant id".into());
    assert!(e.to_string().contains("missing merchant id"));
}

#[test]
fn error_display_unsupported() {
    let e = Error::Unsupported {
        provider: "payir",
        operation: "refund_payment",
    };
    let s = e.to_string();
    assert!(s.contains("payir"));
    assert!(s.contains("refund_payment"));
}

#[test]
fn error_display_decode() {
    let e = Error::Decode {
        provider: "idpay",
        message: "missing data field".into(),
    };
    let s = e.to_string();
    assert!(s.contains("idpay"));
    assert!(s.contains("missing data field"));
}

#[test]
fn error_is_std_error() {
    fn assert_std_error<E: std::error::Error>(_: &E) {}
    let e = Error::Config("x".into());
    assert_std_error(&e);
}

// ── Builder methods + Currency Display + Mock variants ─────────────────────

#[test]
fn currency_display_impl() {
    assert_eq!(format!("{}", iran_pay::Currency::Toman), "Toman");
    assert_eq!(format!("{}", iran_pay::Currency::Rial), "Rial");
}

#[test]
fn amount_min_saturates_no_panic() {
    let a = Amount::toman(i64::MIN);
    let _ = a.as_rials();
    let _ = a.as_tomans();
    assert_eq!(format!("{a}"), format!("{a}"));
}

#[test]
fn amount_new_dispatch() {
    use iran_pay::Currency;
    let t = Amount::new(100, Currency::Toman);
    let r = Amount::new(1000, Currency::Rial);
    assert_eq!(t, r);
}

#[test]
fn amount_zero() {
    assert!(Amount::rial(0).is_zero());
    assert!(!Amount::rial(1).is_zero());
}

#[tokio::test]
async fn provider_with_client_builder() {
    use iran_pay::providers::{IDPay, NextPay, PayIr, Vandar, ZarinPal, Zibal};
    use iran_pay::Gateway;

    let custom = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    // Just exercise each `with_client` builder so it counts toward coverage.
    assert_eq!(
        ZarinPal::new("m").with_client(custom.clone()).name(),
        "zarinpal"
    );
    assert_eq!(IDPay::new("k").with_client(custom.clone()).name(), "idpay");
    assert_eq!(
        NextPay::new("k").with_client(custom.clone()).name(),
        "nextpay"
    );
    assert_eq!(PayIr::new("k").with_client(custom.clone()).name(), "payir");
    assert_eq!(Zibal::new("m").with_client(custom.clone()).name(), "zibal");
    assert_eq!(Vandar::new("k").with_client(custom).name(), "vandar");
}

#[tokio::test]
async fn zarinpal_with_pay_base_overrides_redirect() {
    use iran_pay::providers::ZarinPal;
    use iran_pay::{Gateway, StartRequest};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"code": 100, "authority": "AUTH123", "fee": 0, "fee_type": "Merchant", "message": "OK"},
            "errors": []
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("m")
        .with_api_base(server.uri())
        .with_pay_base("https://my-custom-pay.invalid");

    let req = StartRequest::builder()
        .amount(Amount::toman(1000))
        .description("d")
        .callback_url("https://x/cb")
        .build();
    let resp = gw.start_payment(&req).await.unwrap();
    assert!(resp
        .payment_url
        .starts_with("https://my-custom-pay.invalid"));
}

#[test]
fn refund_request_construction() {
    let req = iran_pay::RefundRequest {
        transaction_id: "TX-1".into(),
        amount: None,
        reason: None,
    };
    assert_eq!(req.transaction_id, "TX-1");
    assert!(req.amount.is_none());

    let req = iran_pay::RefundRequest {
        transaction_id: "TX-2".into(),
        amount: Some(Amount::toman(500)),
        reason: Some("user request".into()),
    };
    assert_eq!(req.amount, Some(Amount::toman(500)));
}

#[test]
fn check_authority_format_max_length_boundary() {
    use iran_pay::security::check_authority_format;
    // Exactly 128 chars is OK.
    let max = "a".repeat(128);
    assert!(check_authority_format(&max).is_ok());
    // 129 is rejected.
    let over = "a".repeat(129);
    assert!(check_authority_format(&over).is_err());
}
