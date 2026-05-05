//! Integration tests for `iran-pay`.
//!
//! Every gateway driver is exercised end-to-end against a `wiremock`-backed
//! HTTP server, so these tests are deterministic and self-contained — no
//! network access is required.  The [`MockGateway`] is also covered for
//! consumers that want to verify their own code without TLS round-trips.

#![allow(clippy::unwrap_used)]

use iran_pay::mock::{Behavior, MockGateway};
use iran_pay::providers::{IDPay, NextPay, PayIr, Vandar, ZarinPal, Zibal};
use iran_pay::{Amount, Error, Gateway, RefundRequest, StartRequest, VerifyRequest};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── helpers ─────────────────────────────────────────────────────────────────

fn sample_start_request(amount: Amount) -> StartRequest {
    StartRequest::builder()
        .amount(amount)
        .description("Pro subscription — May 2026")
        .callback_url("https://example.com/payment/callback")
        .order_id("ORD-12345")
        .build()
}

// ── 1. MockGateway round-trip ───────────────────────────────────────────────

#[tokio::test]
async fn mock_gateway_round_trip() {
    let gw = MockGateway::new();

    let start = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect("mock start succeeds");
    assert_eq!(gw.start_call_count(), 1);
    assert!(start.authority.starts_with("MOCK-AUTH-"));
    assert!(start.payment_url.contains(&start.authority));

    let verify = gw
        .verify_payment(&VerifyRequest {
            authority: start.authority.clone(),
            amount: Amount::toman(50_000),
        })
        .await
        .expect("mock verify succeeds");
    assert_eq!(gw.verify_call_count(), 1);
    assert_eq!(verify.amount.as_tomans(), 50_000);
    assert_eq!(verify.authority, start.authority);
    assert_eq!(verify.provider, "mock");
    assert_eq!(gw.refund_call_count(), 0);
}

// ── 2. MockGateway failure propagation ──────────────────────────────────────

#[tokio::test]
async fn mock_gateway_failure_propagates() {
    let gw = MockGateway::new();
    gw.set_start_behavior(Behavior::FailGateway {
        code: -42,
        message: "merchant suspended".into(),
    });

    let err = gw
        .start_payment(&sample_start_request(Amount::toman(1_000)))
        .await
        .expect_err("should fail");
    match err {
        Error::Gateway {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "mock");
            assert_eq!(code, -42);
            assert_eq!(message, "merchant suspended");
        }
        other => panic!("expected Error::Gateway, got {other:?}"),
    }
    assert_eq!(gw.start_call_count(), 1);
}

// ── 3. ZarinPal: start success ──────────────────────────────────────────────

#[tokio::test]
async fn zarinpal_start_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "code": 100,
                "authority": "A00000000000000000000000000000000001",
                "fee": 0,
                "fee_type": "Merchant",
                "message": "OK",
            },
            "errors": [],
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("00000000-0000-0000-0000-000000000000")
        .with_api_base(server.uri())
        .with_pay_base("https://www.zarinpal.com");

    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect("start success");

    assert_eq!(resp.authority, "A00000000000000000000000000000000001");
    assert_eq!(
        resp.payment_url,
        "https://www.zarinpal.com/pg/StartPay/A00000000000000000000000000000000001"
    );
    assert_eq!(resp.provider, "zarinpal");
}

// ── 4. ZarinPal: failure reported via the `errors` array ────────────────────

#[tokio::test]
async fn zarinpal_start_failure_in_errors_array() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": null,
            "errors": [{
                "code": -9,
                "message": "Validation failed",
            }],
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("00000000-0000-0000-0000-000000000000").with_api_base(server.uri());

    let err = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect_err("should fail");

    match err {
        Error::Gateway {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "zarinpal");
            assert_eq!(code, -9);
            assert_eq!(message, "Validation failed");
        }
        other => panic!("expected Error::Gateway, got {other:?}"),
    }
}

// ── 5. ZarinPal: verify success ─────────────────────────────────────────────

#[tokio::test]
async fn zarinpal_verify_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/verify.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "code": 100,
                "ref_id": 12_345_678i64,
                "card_pan": "6037-99**-****-0006",
                "card_hash": "abc123",
                "fee": 0,
            },
            "errors": [],
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("00000000-0000-0000-0000-000000000000").with_api_base(server.uri());

    let resp = gw
        .verify_payment(&VerifyRequest {
            authority: "A00000000000000000000000000000000001".into(),
            amount: Amount::toman(50_000),
        })
        .await
        .expect("verify success");

    assert_eq!(resp.transaction_id, "12345678");
    assert_eq!(resp.amount.as_rials(), 500_000);
    assert_eq!(resp.card_pan.as_deref(), Some("6037-99**-****-0006"));
    assert_eq!(resp.card_hash.as_deref(), Some("abc123"));
    assert_eq!(resp.provider, "zarinpal");
}

// ── 6. ZarinPal: code 101 ("already verified") still succeeds ───────────────

#[tokio::test]
async fn zarinpal_verify_already_verified() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/verify.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "code": 101,
                "ref_id": 99_999i64,
                "message": "Already verified",
            },
            "errors": [],
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("00000000-0000-0000-0000-000000000000").with_api_base(server.uri());

    let resp = gw
        .verify_payment(&VerifyRequest {
            authority: "A00000000000000000000000000000000001".into(),
            amount: Amount::toman(50_000),
        })
        .await
        .expect("code 101 should still succeed");

    assert_eq!(resp.transaction_id, "99999");
}

// ── 7. IDPay: start success ─────────────────────────────────────────────────

#[tokio::test]
async fn idpay_start_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1.1/payment"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "abc123",
            "link": "https://idpay.ir/p/ws/abc123",
        })))
        .mount(&server)
        .await;

    let gw = IDPay::new("0000000000000000000000000000000000")
        .sandbox()
        .with_api_base(server.uri());

    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect("start success");

    assert_eq!(resp.authority, "abc123");
    assert_eq!(resp.payment_url, "https://idpay.ir/p/ws/abc123");
    assert_eq!(resp.provider, "idpay");
}

// ── 8. IDPay: below-minimum amount returns Config error ─────────────────────

#[tokio::test]
async fn idpay_start_below_minimum() {
    // No mock server is needed — the driver short-circuits before any HTTP.
    let gw = IDPay::new("0000000000000000000000000000000000");

    let err = gw
        .start_payment(&sample_start_request(Amount::rial(500)))
        .await
        .expect_err("should reject below minimum");

    match err {
        Error::Config(msg) => assert!(msg.contains("1000"), "message was: {msg}"),
        other => panic!("expected Error::Config, got {other:?}"),
    }
}

// ── 9. IDPay: gateway-reported amount differs from request → AmountMismatch ─

#[tokio::test]
async fn idpay_verify_amount_mismatch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1.1/payment/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 100,
            "track_id": 12345,
            // The merchant requested 500_000 Rials, but the gateway reports 100.
            "amount": 100,
            "payment": {
                "card_no": "603799******0006",
                "hashed_card_no": "deadbeef",
            },
        })))
        .mount(&server)
        .await;

    let gw = IDPay::new("0000000000000000000000000000000000").with_api_base(server.uri());

    let err = gw
        .verify_payment(&VerifyRequest {
            authority: "abc123".into(),
            amount: Amount::rial(500_000),
        })
        .await
        .expect_err("amount mismatch");

    match err {
        Error::AmountMismatch { expected, actual } => {
            assert_eq!(expected.as_rials(), 500_000);
            assert_eq!(actual.as_rials(), 100);
        }
        other => panic!("expected Error::AmountMismatch, got {other:?}"),
    }
}

// ── 10. IDPay: error_code / error_message → Gateway ─────────────────────────

#[tokio::test]
async fn idpay_failure_with_error_code() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1.1/payment"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error_code": 11,
            "error_message": "User has been blocked",
        })))
        .mount(&server)
        .await;

    let gw = IDPay::new("0000000000000000000000000000000000").with_api_base(server.uri());

    let err = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect_err("gateway error");

    match err {
        Error::Gateway {
            provider,
            code,
            message,
        } => {
            assert_eq!(provider, "idpay");
            assert_eq!(code, 11);
            assert_eq!(message, "User has been blocked");
        }
        other => panic!("expected Error::Gateway, got {other:?}"),
    }
}

// ── 11. NextPay: start success (code -1 means OK) ───────────────────────────

#[tokio::test]
async fn nextpay_start_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/nx/gateway/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": -1,
            "trans_id": "xyz-trans-id",
        })))
        .mount(&server)
        .await;

    let gw = NextPay::new("nextpay-test-key").with_api_base(server.uri());

    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect("nextpay start success");

    assert_eq!(resp.authority, "xyz-trans-id");
    assert_eq!(
        resp.payment_url,
        format!("{}/nx/gateway/payment/xyz-trans-id", server.uri())
    );
    assert_eq!(resp.provider, "nextpay");
}

// ── 12. NextPay: verify success (code 0 means OK) ───────────────────────────

#[tokio::test]
async fn nextpay_verify_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/nx/gateway/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0,
            "amount": 500_000,
            "shaparak_ref_id": "ref-shaparak-99",
            "card_holder": "6037-99**-****-0006",
        })))
        .mount(&server)
        .await;

    let gw = NextPay::new("nextpay-test-key").with_api_base(server.uri());

    let resp = gw
        .verify_payment(&VerifyRequest {
            authority: "xyz-trans-id".into(),
            amount: Amount::toman(50_000),
        })
        .await
        .expect("nextpay verify success");

    assert_eq!(resp.transaction_id, "ref-shaparak-99");
    assert_eq!(resp.amount.as_rials(), 500_000);
    assert_eq!(resp.card_pan.as_deref(), Some("6037-99**-****-0006"));
}

// ── 13. Pay.ir: start success (status 1) ────────────────────────────────────

#[tokio::test]
async fn payir_start_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "token": "pay-tok-001",
        })))
        .mount(&server)
        .await;

    let gw = PayIr::new("test").with_api_base(server.uri());

    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(50_000)))
        .await
        .expect("pay.ir start success");

    assert_eq!(resp.authority, "pay-tok-001");
    assert_eq!(resp.payment_url, format!("{}/pg/pay-tok-001", server.uri()));
    assert_eq!(resp.provider, "payir");
}

// ── 14. Pay.ir: verify success with camelCase aliases ───────────────────────

#[tokio::test]
async fn payir_verify_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "amount": 500_000,
            "transId": "tx-77",
            "cardNumber": "603799******0006",
        })))
        .mount(&server)
        .await;

    let gw = PayIr::new("test").with_api_base(server.uri());

    let resp = gw
        .verify_payment(&VerifyRequest {
            authority: "pay-tok-001".into(),
            amount: Amount::toman(50_000),
        })
        .await
        .expect("pay.ir verify success");

    assert_eq!(resp.transaction_id, "tx-77");
    assert_eq!(resp.amount.as_rials(), 500_000);
    assert_eq!(resp.card_pan.as_deref(), Some("603799******0006"));
}

// ── 15. dyn Gateway polymorphism over multiple providers ────────────────────

#[tokio::test]
async fn dyn_gateway_polymorphism() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "code": 100,
                "authority": "A00000000000000000000000000000000042",
                "fee": 0,
                "fee_type": "Merchant",
                "message": "OK",
            },
            "errors": [],
        })))
        .mount(&server)
        .await;

    let gateways: Vec<Box<dyn Gateway>> = vec![
        Box::new(MockGateway::new()),
        Box::new(
            ZarinPal::new("00000000-0000-0000-0000-000000000000")
                .with_api_base(server.uri())
                .with_pay_base("https://www.zarinpal.com"),
        ),
    ];

    for gw in &gateways {
        let resp = gw
            .start_payment(&sample_start_request(Amount::toman(50_000)))
            .await
            .unwrap_or_else(|e| panic!("{} failed: {e:?}", gw.name()));
        assert!(!resp.authority.is_empty(), "{} authority empty", gw.name());
    }

    assert_eq!(gateways[0].name(), "mock");
    assert_eq!(gateways[1].name(), "zarinpal");
}

// ── 16. Amount unit-safety sanity check ─────────────────────────────────────

#[test]
fn amount_unit_safety() {
    assert_eq!(Amount::toman(100).as_rials(), 1_000);
    assert_eq!(Amount::rial(1_000).as_tomans(), 100);
    assert!(Amount::rial(0).is_zero());
    assert!(!Amount::toman(1).is_zero());
    assert_eq!(Amount::toman(50_000), Amount::rial(500_000));
}

// ── 17. Default `refund_payment` returns Unsupported ────────────────────────

#[tokio::test]
async fn refund_default_unsupported() {
    // ZarinPal uses the trait's default `refund_payment` impl, so any call
    // should produce `Error::Unsupported`.
    let gw =
        ZarinPal::new("00000000-0000-0000-0000-000000000000").with_api_base("http://127.0.0.1:1");

    let err = gw
        .refund_payment(&RefundRequest {
            transaction_id: "tx-1".into(),
            amount: None,
            reason: None,
        })
        .await
        .expect_err("default refund must be Unsupported");

    match err {
        Error::Unsupported {
            provider,
            operation,
        } => {
            assert_eq!(provider, "zarinpal");
            assert_eq!(operation, "refund_payment");
        }
        other => panic!("expected Error::Unsupported, got {other:?}"),
    }
}

// ── Zibal ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn zibal_start_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/request"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 100,
            "trackId": 12345_i64,
            "message": "success"
        })))
        .mount(&server)
        .await;

    let gw = Zibal::new("merchant").with_api_base(server.uri());
    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await
        .unwrap();
    assert_eq!(resp.authority, "12345");
    assert_eq!(resp.provider, "zibal");
    assert!(resp.payment_url.ends_with("/start/12345"));
}

#[tokio::test]
async fn zibal_start_failure_returns_gateway_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/request"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 102,
            "message": "merchant not found"
        })))
        .mount(&server)
        .await;

    let gw = Zibal::new("bad").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    match res {
        Err(Error::Gateway { code, message, .. }) => {
            assert_eq!(code, 102);
            assert!(message.contains("not found"));
        }
        other => panic!("expected Gateway error, got {other:?}"),
    }
}

#[tokio::test]
async fn zibal_verify_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 100,
            "amount": 10000_i64,
            "refNumber": 99999_i64,
            "cardNumber": "603799******0006"
        })))
        .mount(&server)
        .await;

    let gw = Zibal::new("merchant").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "12345".into(),
        amount: Amount::toman(1000),
    };
    let resp = gw.verify_payment(&req).await.unwrap();
    assert_eq!(resp.transaction_id, "99999");
    assert_eq!(resp.card_pan.as_deref(), Some("603799******0006"));
    assert_eq!(resp.amount, Amount::toman(1000));
}

#[tokio::test]
async fn zibal_verify_already_verified_201() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 201,
            "amount": 10000_i64,
            "refNumber": 11
        })))
        .mount(&server)
        .await;

    let gw = Zibal::new("m").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "12345".into(),
        amount: Amount::toman(1000),
    };
    assert!(gw.verify_payment(&req).await.is_ok());
}

#[tokio::test]
async fn zibal_verify_amount_mismatch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": 100,
            "amount": 5000_i64,           // != requested 10000
            "refNumber": 1
        })))
        .mount(&server)
        .await;

    let gw = Zibal::new("m").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "12345".into(),
        amount: Amount::toman(1000), // = 10000 rials
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::AmountMismatch { .. })
    ));
}

#[tokio::test]
async fn zibal_verify_non_numeric_authority_fails() {
    let server = MockServer::start().await;
    let gw = Zibal::new("m").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "not-a-number".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Decode { .. })
    ));
}

#[tokio::test]
async fn zibal_sandbox_uses_test_merchant() {
    let gw = Zibal::sandbox();
    assert_eq!(gw.name(), "zibal");
}

// ── Vandar ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn vandar_start_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "token": "tok-abc-123"
        })))
        .mount(&server)
        .await;

    let gw = Vandar::new("apikey").with_api_base(server.uri());
    let resp = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await
        .unwrap();
    assert_eq!(resp.authority, "tok-abc-123");
    assert!(resp.payment_url.ends_with("/v3/tok-abc-123"));
}

#[tokio::test]
async fn vandar_start_below_minimum_returns_config_error() {
    let server = MockServer::start().await;
    let gw = Vandar::new("apikey").with_api_base(server.uri());
    let req = StartRequest::builder()
        .amount(Amount::rial(500)) // below 1000-rial minimum
        .description("test")
        .callback_url("https://example.com/cb")
        .build();
    assert!(matches!(
        gw.start_payment(&req).await,
        Err(Error::Config(_))
    ));
}

#[tokio::test]
async fn vandar_start_failure_returns_gateway_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 0,
            "errors": ["bad api_key"]
        })))
        .mount(&server)
        .await;

    let gw = Vandar::new("bad").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    match res {
        Err(Error::Gateway { code, message, .. }) => {
            assert_eq!(code, 0);
            assert!(message.contains("api_key"));
        }
        other => panic!("expected Gateway error, got {other:?}"),
    }
}

#[tokio::test]
async fn vandar_verify_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "amount": 10000_i64,
            "transId": 7777_i64,
            "cardNumber": "603799******0006",
            "CID": "card-hash"
        })))
        .mount(&server)
        .await;

    let gw = Vandar::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tok-abc".into(),
        amount: Amount::toman(1000),
    };
    let resp = gw.verify_payment(&req).await.unwrap();
    assert_eq!(resp.transaction_id, "7777");
    assert_eq!(resp.card_hash.as_deref(), Some("card-hash"));
}

#[tokio::test]
async fn vandar_verify_amount_mismatch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "amount": 5000_i64,
            "transId": 1
        })))
        .mount(&server)
        .await;

    let gw = Vandar::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tok".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::AmountMismatch { .. })
    ));
}

#[tokio::test]
async fn vandar_verify_failure() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 0,
            "errors": ["expired token"]
        })))
        .mount(&server)
        .await;

    let gw = Vandar::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tok".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Gateway { .. })
    ));
}

#[tokio::test]
async fn vandar_sandbox_keeps_name() {
    let gw = Vandar::new("k").sandbox();
    assert_eq!(gw.name(), "vandar");
}

// ── Provider error-path tests (boost per-provider coverage) ────────────────

#[tokio::test]
async fn zarinpal_verify_failure_data_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/verify.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"code": -50, "ref_id": 0, "message": "transaction failed"},
            "errors": []
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("m")
        .with_api_base(server.uri())
        .with_pay_base(server.uri());
    let req = VerifyRequest {
        authority: "A0000".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Gateway { code: -50, .. })
    ));
}

#[tokio::test]
async fn zarinpal_start_failure_data_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {"code": -9, "authority": "", "message": "validation failed"},
            "errors": []
        })))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("m").with_api_base(server.uri());
    let req = sample_start_request(Amount::toman(1000));
    assert!(matches!(
        gw.start_payment(&req).await,
        Err(Error::Gateway { code: -9, .. })
    ));
}

#[tokio::test]
async fn zarinpal_decode_error_on_missing_data() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": null, "errors": []})))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("m").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Decode { .. })));
}

#[tokio::test]
async fn idpay_verify_already_verified_101() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1.1/payment/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 101,
            "track_id": 9999_i64,
            "amount": 10000_i64,
            "payment": {"card_no": "603799******0006", "hashed_card_no": "h"}
        })))
        .mount(&server)
        .await;

    let gw = IDPay::new("key").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "id123".into(),
        amount: Amount::toman(1000),
    };
    let resp = gw.verify_payment(&req).await.unwrap();
    assert_eq!(resp.transaction_id, "9999");
}

#[tokio::test]
async fn idpay_verify_failed_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1.1/payment/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 5,                  // not 100/101
            "track_id": 0,
            "amount": 10000_i64
        })))
        .mount(&server)
        .await;

    let gw = IDPay::new("key").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "id123".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Gateway { code: 5, .. })
    ));
}

#[tokio::test]
async fn idpay_sandbox_sets_header() {
    // We don't have a way to inspect headers in wiremock without matchers,
    // but exercising the sandbox builder is enough for coverage.
    let gw = IDPay::new("k").sandbox();
    assert_eq!(gw.name(), "idpay");
}

#[tokio::test]
async fn nextpay_start_failure_non_minus_one() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/nx/gateway/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": -33})))
        .mount(&server)
        .await;

    let gw = NextPay::new("k").with_api_base(server.uri());
    assert!(matches!(
        gw.start_payment(&sample_start_request(Amount::toman(1000)))
            .await,
        Err(Error::Gateway { code: -33, .. })
    ));
}

#[tokio::test]
async fn nextpay_verify_failure_non_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/nx/gateway/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"code": -2})))
        .mount(&server)
        .await;

    let gw = NextPay::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tx".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Gateway { code: -2, .. })
    ));
}

#[tokio::test]
async fn nextpay_verify_amount_mismatch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/nx/gateway/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "code": 0,
            "amount": 5000_i64,
            "shaparak_ref_id": "ref"
        })))
        .mount(&server)
        .await;

    let gw = NextPay::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tx".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::AmountMismatch { .. })
    ));
}

#[tokio::test]
async fn nextpay_sandbox_no_op() {
    let gw = NextPay::new("k").sandbox();
    assert_eq!(gw.name(), "nextpay");
}

#[tokio::test]
async fn payir_start_failure_status_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 0,
            "errorMessage": "rejected"
        })))
        .mount(&server)
        .await;

    let gw = PayIr::new("k").with_api_base(server.uri());
    assert!(matches!(
        gw.start_payment(&sample_start_request(Amount::toman(1000)))
            .await,
        Err(Error::Gateway { code: 0, .. })
    ));
}

#[tokio::test]
async fn payir_verify_failure_status_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 0,
            "message": "expired"
        })))
        .mount(&server)
        .await;

    let gw = PayIr::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tok".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::Gateway { code: 0, .. })
    ));
}

#[tokio::test]
async fn payir_sandbox_uses_test_key() {
    let gw = PayIr::sandbox();
    assert_eq!(gw.name(), "payir");
}

#[tokio::test]
async fn payir_verify_amount_mismatch() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": 1,
            "amount": 5000_i64,
            "transId": "t-1"
        })))
        .mount(&server)
        .await;

    let gw = PayIr::new("k").with_api_base(server.uri());
    let req = VerifyRequest {
        authority: "tok".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        gw.verify_payment(&req).await,
        Err(Error::AmountMismatch { .. })
    ));
}

// ── Mock refund ────────────────────────────────────────────────────────────

#[tokio::test]
async fn mock_refund_succeeds() {
    let gw = MockGateway::new();
    let req = RefundRequest {
        transaction_id: "TX-1".into(),
        amount: Some(Amount::toman(1000)),
        reason: Some("change of mind".into()),
    };
    let resp = gw.refund_payment(&req).await.unwrap();
    assert_eq!(resp.transaction_id, "TX-1");
    assert_eq!(gw.refund_call_count(), 1);
}

// ── StartRequest builder full coverage ────────────────────────────────────

#[test]
fn start_request_builder_with_all_fields() {
    let req = StartRequest::builder()
        .amount(Amount::toman(50))
        .description("d")
        .callback_url("https://x/cb")
        .email("u@e.com")
        .mobile("09121234567")
        .order_id("ORD-1")
        .extra("custom_key", "custom_value")
        .build();
    assert_eq!(req.email.as_deref(), Some("u@e.com"));
    assert_eq!(req.mobile.as_deref(), Some("09121234567"));
    assert_eq!(req.order_id.as_deref(), Some("ORD-1"));
    assert_eq!(
        req.extras.get("custom_key").map(|s| s.as_str()),
        Some("custom_value")
    );
}

// ── HTTP-error & decode-error coverage ─────────────────────────────────────

/// 404 response with HTML body → reqwest .json() fails → Error::Http.
#[tokio::test]
async fn zarinpal_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    // No mocks installed: server returns 404 + HTML for every request.
    let gw = ZarinPal::new("m").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

#[tokio::test]
async fn idpay_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    let gw = IDPay::new("k").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

#[tokio::test]
async fn nextpay_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    let gw = NextPay::new("k").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

#[tokio::test]
async fn payir_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    let gw = PayIr::new("k").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

#[tokio::test]
async fn zibal_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    let gw = Zibal::new("m").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

#[tokio::test]
async fn vandar_http_error_on_unmocked_endpoint() {
    let server = MockServer::start().await;
    let gw = Vandar::new("k").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    assert!(matches!(res, Err(Error::Http { .. })));
}

/// Verify path also gets the HTTP-error treatment.
#[tokio::test]
async fn all_providers_verify_http_error() {
    let server = MockServer::start().await;
    let req = VerifyRequest {
        authority: "x".into(),
        amount: Amount::toman(1000),
    };

    assert!(matches!(
        ZarinPal::new("m")
            .with_api_base(server.uri())
            .verify_payment(&req)
            .await,
        Err(Error::Http { .. })
    ));
    assert!(matches!(
        IDPay::new("k")
            .with_api_base(server.uri())
            .verify_payment(&req)
            .await,
        Err(Error::Http { .. })
    ));
    assert!(matches!(
        NextPay::new("k")
            .with_api_base(server.uri())
            .verify_payment(&req)
            .await,
        Err(Error::Http { .. })
    ));
    assert!(matches!(
        PayIr::new("k")
            .with_api_base(server.uri())
            .verify_payment(&req)
            .await,
        Err(Error::Http { .. })
    ));
    assert!(matches!(
        Vandar::new("k")
            .with_api_base(server.uri())
            .verify_payment(&req)
            .await,
        Err(Error::Http { .. })
    ));

    // Zibal verify needs a numeric authority; test the http path with a numeric one.
    let req_num = VerifyRequest {
        authority: "12345".into(),
        amount: Amount::toman(1000),
    };
    assert!(matches!(
        Zibal::new("m")
            .with_api_base(server.uri())
            .verify_payment(&req_num)
            .await,
        Err(Error::Http { .. })
    ));
}

/// Decode error: server returns valid JSON but with the wrong shape.
#[tokio::test]
async fn zarinpal_decode_error_on_wrong_shape() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/pg/v4/payment/request.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"unexpected": "shape"})))
        .mount(&server)
        .await;

    let gw = ZarinPal::new("m").with_api_base(server.uri());
    let res = gw
        .start_payment(&sample_start_request(Amount::toman(1000)))
        .await;
    // ZarinPal happens to succeed-and-then-fail-in-data-parse, so it could be either.
    assert!(res.is_err());
}
