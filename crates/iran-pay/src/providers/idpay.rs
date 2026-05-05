//! IDPay driver.
//!
//! IDPay's API uses an `X-API-KEY` header for authentication and an
//! optional `X-SANDBOX: 1` header for sandbox mode.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "idpay";
const API: &str = "https://api.idpay.ir";

/// IDPay gateway driver.
pub struct IDPay {
    api_key: String,
    sandbox: bool,
    api_base: String,
    client: reqwest::Client,
}

impl IDPay {
    /// New driver with the given API key (32-char hex string from your IDPay
    /// dashboard).
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            sandbox: false,
            api_base: API.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Switch to IDPay's sandbox.  Sets `X-SANDBOX: 1` on every request.
    #[must_use]
    pub fn sandbox(mut self) -> Self {
        self.sandbox = true;
        self
    }

    /// Override the API base URL (for tests).
    #[must_use]
    pub fn with_api_base(mut self, url: impl Into<String>) -> Self {
        self.api_base = url.into();
        self
    }

    /// Override the underlying HTTP client.
    #[must_use]
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }

    fn request<U: AsRef<str>>(&self, path: U) -> reqwest::RequestBuilder {
        let mut rb = self
            .client
            .post(format!("{}{}", self.api_base, path.as_ref()))
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json");
        if self.sandbox {
            rb = rb.header("X-SANDBOX", "1");
        }
        rb
    }
}

#[async_trait]
impl Gateway for IDPay {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        // IDPay enforces minimum 1000 Rials.
        if req.amount.as_rials() < 1_000 {
            return Err(Error::Config(format!(
                "idpay: amount must be at least 1000 Rials (got {} Rials)",
                req.amount.as_rials()
            )));
        }

        let body = json!({
            "order_id": req.order_id.clone().unwrap_or_default(),
            "amount": req.amount.as_rials(),
            "name": req.extras.get("name").cloned().unwrap_or_default(),
            "phone": req.mobile.clone().unwrap_or_default(),
            "mail": req.email.clone().unwrap_or_default(),
            "desc": req.description,
            "callback": req.callback_url,
        });

        let resp = self
            .request("/v1.1/payment")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;
        let raw: serde_json::Value = resp.json().await.map_err(|e| Error::http(PROVIDER, e))?;

        // Error case: {error_code, error_message}
        if let Some(code) = raw.get("error_code").and_then(|v| v.as_i64()) {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code,
                message: raw
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
            });
        }

        let parsed: IdpStart = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;

        Ok(StartResponse {
            authority: parsed.id,
            payment_url: parsed.link,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let body = json!({
            "id": req.authority,
            // IDPay requires the original order_id, but we don't always have
            // it at verify-time.  Empty is accepted by the sandbox; in
            // production, callers should keep the original `order_id`.
            "order_id": req.authority,
        });

        let resp = self
            .request("/v1.1/payment/verify")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;
        let raw: serde_json::Value = resp.json().await.map_err(|e| Error::http(PROVIDER, e))?;

        if let Some(code) = raw.get("error_code").and_then(|v| v.as_i64()) {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code,
                message: raw
                    .get("error_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
            });
        }

        let parsed: IdpVerify = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;

        // Status code 100 = paid & verified; 101 = already verified.
        if parsed.status != 100 && parsed.status != 101 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.status,
                message: format!("idpay status {}", parsed.status),
            });
        }

        let actual = Amount::rial(parsed.amount);
        if actual != req.amount {
            return Err(Error::AmountMismatch {
                expected: req.amount,
                actual,
            });
        }

        Ok(VerifyResponse {
            transaction_id: parsed.track_id.to_string(),
            authority: req.authority.clone(),
            amount: actual,
            card_pan: parsed.payment.as_ref().and_then(|p| p.card_no.clone()),
            card_hash: parsed
                .payment
                .as_ref()
                .and_then(|p| p.hashed_card_no.clone()),
            fee: None,
            provider: PROVIDER,
            raw,
        })
    }
}

// ── wire types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct IdpStart {
    id: String,
    link: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IdpVerify {
    status: i64,
    track_id: i64,
    amount: i64,
    #[serde(default)]
    payment: Option<IdpPayment>,
}

#[derive(Debug, Deserialize, Serialize)]
struct IdpPayment {
    #[serde(default)]
    card_no: Option<String>,
    #[serde(default)]
    hashed_card_no: Option<String>,
}
