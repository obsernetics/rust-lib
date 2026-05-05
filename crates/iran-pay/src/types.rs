//! Common request / response types used by every [`Gateway`](crate::Gateway).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::Amount;

// ── start ────────────────────────────────────────────────────────────────────

/// Inputs for [`Gateway::start_payment`](crate::Gateway::start_payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartRequest {
    /// The amount to charge.
    pub amount: Amount,
    /// Short human-readable description shown to the user on the gateway page.
    pub description: String,
    /// HTTPS URL the gateway will redirect the user to after payment.
    pub callback_url: String,
    /// Optional buyer e-mail (used by some providers for receipts).
    pub email: Option<String>,
    /// Optional buyer mobile number (Iranian format, used by some providers
    /// to pre-fill the OTP step).
    pub mobile: Option<String>,
    /// Optional merchant-side order ID — echoed back in the verify response
    /// where supported.  Strongly recommended for reconciliation.
    pub order_id: Option<String>,
    /// Free-form provider-specific extras (forwarded as-is to drivers that
    /// support metadata).
    #[serde(default)]
    pub extras: HashMap<String, String>,
}

impl StartRequest {
    /// Start a builder.
    #[must_use]
    pub fn builder() -> StartRequestBuilder {
        StartRequestBuilder::default()
    }
}

/// Step-builder for [`StartRequest`].
#[derive(Debug, Default)]
pub struct StartRequestBuilder {
    amount: Option<Amount>,
    description: Option<String>,
    callback_url: Option<String>,
    email: Option<String>,
    mobile: Option<String>,
    order_id: Option<String>,
    extras: HashMap<String, String>,
}

impl StartRequestBuilder {
    /// Set the amount (required).
    #[must_use]
    pub fn amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the description (required).
    #[must_use]
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Set the callback URL (required).
    #[must_use]
    pub fn callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }

    /// Set the buyer e-mail (optional).
    #[must_use]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the buyer mobile number (optional).
    #[must_use]
    pub fn mobile(mut self, mobile: impl Into<String>) -> Self {
        self.mobile = Some(mobile.into());
        self
    }

    /// Set the merchant order ID (optional but recommended).
    #[must_use]
    pub fn order_id(mut self, id: impl Into<String>) -> Self {
        self.order_id = Some(id.into());
        self
    }

    /// Add a single provider-specific extra metadata key/value.
    #[must_use]
    pub fn extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extras.insert(key.into(), value.into());
        self
    }

    /// Build the request.  Panics if `amount`, `description`, or
    /// `callback_url` were not set — these are always required by every
    /// supported gateway.
    pub fn build(self) -> StartRequest {
        StartRequest {
            amount: self.amount.expect("StartRequest: amount is required"),
            description: self
                .description
                .expect("StartRequest: description is required"),
            callback_url: self
                .callback_url
                .expect("StartRequest: callback_url is required"),
            email: self.email,
            mobile: self.mobile,
            order_id: self.order_id,
            extras: self.extras,
        }
    }
}

/// Output of [`Gateway::start_payment`](crate::Gateway::start_payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResponse {
    /// Provider-issued token / authority for this payment session.  Save it
    /// — you'll need it to call [`Gateway::verify_payment`](crate::Gateway::verify_payment).
    pub authority: String,
    /// HTTPS URL to redirect the user to.  After they pay, the gateway
    /// redirects back to your `callback_url`.
    pub payment_url: String,
    /// Driver name that produced this response.
    pub provider: &'static str,
    /// Full provider response, retained for debugging / logging.
    #[serde(default)]
    pub raw: serde_json::Value,
}

// ── verify ───────────────────────────────────────────────────────────────────

/// Inputs for [`Gateway::verify_payment`](crate::Gateway::verify_payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    /// The `authority` you received in [`StartResponse::authority`].
    pub authority: String,
    /// The amount you originally charged.  Re-confirming the amount catches
    /// callback-tampering attacks where an attacker swaps the authority for
    /// one tied to a smaller transaction.
    pub amount: Amount,
}

/// Output of [`Gateway::verify_payment`](crate::Gateway::verify_payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    /// Gateway's permanent transaction reference (`RefId` for ZarinPal,
    /// `track_id` for IDPay, etc.).  Persist this for refunds and audits.
    pub transaction_id: String,
    /// The original authority that was just verified.
    pub authority: String,
    /// Final settled amount (should match what you sent — the SDK already
    /// double-checks).
    pub amount: Amount,
    /// Masked card PAN if the gateway returned it.
    pub card_pan: Option<String>,
    /// Hash of the payer's card (lets you fingerprint repeat customers
    /// without storing PANs).
    pub card_hash: Option<String>,
    /// Gateway fee, when reported.
    pub fee: Option<Amount>,
    /// Driver name.
    pub provider: &'static str,
    /// Full provider response for debugging / audit logging.
    #[serde(default)]
    pub raw: serde_json::Value,
}

// ── refund ───────────────────────────────────────────────────────────────────

/// Inputs for [`Gateway::refund_payment`](crate::Gateway::refund_payment).
///
/// Not every Iranian gateway supports automated refunds; drivers that don't
/// will return [`Error::Unsupported`](crate::Error::Unsupported).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    /// The `transaction_id` from a successful [`VerifyResponse`].
    pub transaction_id: String,
    /// Optional partial refund amount.  Omit for a full refund.
    pub amount: Option<Amount>,
    /// Optional reason string — surfaced to the merchant dashboard.
    pub reason: Option<String>,
}

/// Output of [`Gateway::refund_payment`](crate::Gateway::refund_payment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResponse {
    /// Gateway's refund reference number.
    pub refund_id: String,
    /// The transaction that was refunded.
    pub transaction_id: String,
    /// How much was actually refunded.
    pub amount: Amount,
    /// Driver name.
    pub provider: &'static str,
    /// Full provider response.
    #[serde(default)]
    pub raw: serde_json::Value,
}
