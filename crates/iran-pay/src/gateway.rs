//! The [`Gateway`] trait — the single abstraction every driver implements.

use async_trait::async_trait;

use crate::{
    Error, RefundRequest, RefundResponse, Result, StartRequest, StartResponse, VerifyRequest,
    VerifyResponse,
};

/// Common interface for every Iranian payment gateway driver.
///
/// All methods are `&self`-only so a single [`Gateway`] instance can be
/// shared across tasks (typically in an `Arc`).
///
/// `Gateway` is **dyn-safe**: you can hold a `Box<dyn Gateway>` or
/// `Arc<dyn Gateway>` in a HashMap to swap providers at runtime.
///
/// # Example: runtime selection
///
/// ```ignore
/// use std::sync::Arc;
/// use iran_pay::{Gateway, providers::{ZarinPal, IDPay}};
///
/// fn pick(name: &str) -> Arc<dyn Gateway> {
///     match name {
///         "zarinpal" => Arc::new(ZarinPal::new("MERCHANT")),
///         "idpay"    => Arc::new(IDPay::new("API-KEY")),
///         _ => unreachable!(),
///     }
/// }
/// ```
#[async_trait]
pub trait Gateway: Send + Sync {
    /// Driver name (`"zarinpal"`, `"idpay"`, `"nextpay"`, `"payir"`, or
    /// `"mock"`).  Useful for logging and metrics tags.
    fn name(&self) -> &'static str;

    /// Initiate a payment.  Returns an authority token and the URL you
    /// should redirect the user to.
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse>;

    /// Verify a payment after the user returns from the gateway.
    ///
    /// The driver also re-checks that the gateway-reported amount matches
    /// `req.amount` and returns [`Error::AmountMismatch`] if not — guarding
    /// against tampered callback URLs.
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse>;

    /// Refund a previously verified transaction.
    ///
    /// Default implementation returns [`Error::Unsupported`].  Drivers that
    /// support refunds override this method.
    async fn refund_payment(&self, _req: &RefundRequest) -> Result<RefundResponse> {
        Err(Error::Unsupported {
            provider: self.name(),
            operation: "refund_payment",
        })
    }
}
