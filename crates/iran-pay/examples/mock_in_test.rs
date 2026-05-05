//! Drop-in `MockGateway` for testing your own checkout service.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example mock_in_test -p iran-pay
//! ```
//!
//! In a real app you would put this scenario inside a `#[tokio::test]`
//! function in `tests/`.  Running it as an example just makes the wiring
//! easier to read in isolation.
//!
//! The pattern:
//!
//! 1. Construct a `MockGateway`, optionally scripting failure scenarios.
//! 2. Hand it to your service as `Box<dyn Gateway>` or `Arc<dyn Gateway>`.
//! 3. After the test, assert against `start_call_count()` /
//!    `verify_call_count()` to confirm the service did what you expected.

use std::sync::Arc;

use iran_pay::mock::{Behavior, MockGateway};
use iran_pay::{Amount, Gateway, StartRequest, VerifyRequest};

/// A toy "checkout service" that takes any `Gateway` impl.  Real production
/// code would do the same thing: depend on the trait, not on a concrete
/// driver.
struct CheckoutService {
    gateway: Arc<dyn Gateway>,
}

impl CheckoutService {
    fn new(gateway: Arc<dyn Gateway>) -> Self {
        Self { gateway }
    }

    async fn start(&self, amount: Amount) -> Result<String, iran_pay::Error> {
        let req = StartRequest::builder()
            .amount(amount)
            .description("Pro subscription")
            .callback_url("https://example.com/cb")
            .order_id("ORD-1")
            .build();
        let resp = self.gateway.start_payment(&req).await?;
        Ok(resp.authority)
    }

    async fn finalize(&self, authority: String, amount: Amount) -> Result<String, iran_pay::Error> {
        let resp = self
            .gateway
            .verify_payment(&VerifyRequest { authority, amount })
            .await?;
        Ok(resp.transaction_id)
    }
}

#[tokio::main]
async fn main() {
    // ── happy path ──────────────────────────────────────────────────────
    let mock = Arc::new(MockGateway::new());
    let svc = CheckoutService::new(mock.clone());

    let amount = Amount::toman(50_000);
    let authority = svc.start(amount).await.expect("start ok");
    let tx = svc.finalize(authority, amount).await.expect("finalize ok");

    assert_eq!(mock.start_call_count(), 1);
    assert_eq!(mock.verify_call_count(), 1);
    assert!(tx.starts_with("MOCK-TX-"));
    println!("happy path → tx = {tx}");

    // ── scripted failure ────────────────────────────────────────────────
    let mock = Arc::new(MockGateway::new());
    mock.set_start_behavior(Behavior::FailGateway {
        code: -9,
        message: "merchant suspended".into(),
    });
    let svc = CheckoutService::new(mock.clone());

    let err = svc
        .start(Amount::toman(50_000))
        .await
        .expect_err("scripted failure");
    println!("scripted failure → {err}");
    assert_eq!(mock.start_call_count(), 1);
    assert_eq!(mock.verify_call_count(), 0);
}
