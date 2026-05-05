//! [`MockGateway`] — a no-network [`Gateway`] for unit-testing your own code.
//!
//! Use this in your test suite to exercise your checkout flow without
//! standing up a real provider:
//!
//! ```
//! use iran_pay::{Amount, Gateway, StartRequest, VerifyRequest};
//! use iran_pay::mock::MockGateway;
//!
//! # async fn run() -> Result<(), iran_pay::Error> {
//! let gw = MockGateway::new();
//! let start = gw.start_payment(&StartRequest::builder()
//!     .amount(Amount::toman(1_000))
//!     .description("Test")
//!     .callback_url("http://localhost/cb")
//!     .build()).await?;
//!
//! let verify = gw.verify_payment(&VerifyRequest {
//!     authority: start.authority,
//!     amount: Amount::toman(1_000),
//! }).await?;
//!
//! assert_eq!(verify.amount.as_tomans(), 1_000);
//! # Ok(()) }
//! ```

use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

/// What the mock should do on the next call.
#[derive(Debug, Clone)]
pub enum Behavior {
    /// Succeed normally (the default).
    Succeed,
    /// Fail with a [`Error::Gateway`] bearing the given code/message.
    FailGateway {
        /// Provider error code to return.
        code: i64,
        /// Message to return.
        message: String,
    },
}

/// In-memory [`Gateway`] for tests.  No network I/O, no external dependencies.
///
/// The mock generates monotonically increasing `authority` and
/// `transaction_id` values and keeps a counter of calls so tests can assert
/// "exactly one call" semantics easily.
pub struct MockGateway {
    inner: Arc<MockInner>,
}

struct MockInner {
    next_id: AtomicU64,
    start_calls: AtomicU64,
    verify_calls: AtomicU64,
    refund_calls: AtomicU64,
    start_behavior: Mutex<Behavior>,
    verify_behavior: Mutex<Behavior>,
}

impl Default for MockGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl MockGateway {
    /// New mock with default (always-succeed) behaviour.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MockInner {
                next_id: AtomicU64::new(1),
                start_calls: AtomicU64::new(0),
                verify_calls: AtomicU64::new(0),
                refund_calls: AtomicU64::new(0),
                start_behavior: Mutex::new(Behavior::Succeed),
                verify_behavior: Mutex::new(Behavior::Succeed),
            }),
        }
    }

    /// Configure the next [`Gateway::start_payment`] call's outcome.
    pub fn set_start_behavior(&self, b: Behavior) {
        *self.inner.start_behavior.lock().unwrap() = b;
    }

    /// Configure the next [`Gateway::verify_payment`] call's outcome.
    pub fn set_verify_behavior(&self, b: Behavior) {
        *self.inner.verify_behavior.lock().unwrap() = b;
    }

    /// Number of `start_payment` calls observed so far.
    #[must_use]
    pub fn start_call_count(&self) -> u64 {
        self.inner.start_calls.load(Ordering::SeqCst)
    }

    /// Number of `verify_payment` calls observed so far.
    #[must_use]
    pub fn verify_call_count(&self) -> u64 {
        self.inner.verify_calls.load(Ordering::SeqCst)
    }

    /// Number of `refund_payment` calls observed so far.
    #[must_use]
    pub fn refund_call_count(&self) -> u64 {
        self.inner.refund_calls.load(Ordering::SeqCst)
    }

    fn next_id(&self) -> u64 {
        self.inner.next_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[async_trait]
impl Gateway for MockGateway {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        self.inner.start_calls.fetch_add(1, Ordering::SeqCst);
        let behavior = self.inner.start_behavior.lock().unwrap().clone();
        if let Behavior::FailGateway { code, message } = behavior {
            return Err(Error::Gateway {
                provider: "mock",
                code,
                message,
            });
        }

        let id = self.next_id();
        let authority = format!("MOCK-AUTH-{id:020}");
        Ok(StartResponse {
            authority: authority.clone(),
            payment_url: format!("https://mock.invalid/pay/{}", authority),
            provider: "mock",
            raw: serde_json::json!({
                "amount": req.amount.as_rials(),
                "description": req.description,
                "callback_url": req.callback_url,
            }),
        })
    }

    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        self.inner.verify_calls.fetch_add(1, Ordering::SeqCst);
        let behavior = self.inner.verify_behavior.lock().unwrap().clone();
        if let Behavior::FailGateway { code, message } = behavior {
            return Err(Error::Gateway {
                provider: "mock",
                code,
                message,
            });
        }

        let id = self.next_id();
        Ok(VerifyResponse {
            transaction_id: format!("MOCK-TX-{id:020}"),
            authority: req.authority.clone(),
            amount: req.amount,
            card_pan: Some("6037-99**-****-0006".into()),
            card_hash: Some(format!("mock-hash-{id}")),
            fee: Some(Amount::rial(0)),
            provider: "mock",
            raw: serde_json::json!({"mock": true}),
        })
    }

    async fn refund_payment(&self, req: &crate::RefundRequest) -> Result<crate::RefundResponse> {
        self.inner.refund_calls.fetch_add(1, Ordering::SeqCst);
        let id = self.next_id();
        Ok(crate::RefundResponse {
            refund_id: format!("MOCK-REFUND-{id:020}"),
            transaction_id: req.transaction_id.clone(),
            amount: req.amount.unwrap_or_else(|| Amount::rial(0)),
            provider: "mock",
            raw: serde_json::json!({"mock_refund": true}),
        })
    }
}
