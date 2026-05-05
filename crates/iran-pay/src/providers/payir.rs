//! Pay.ir driver.
//!
//! Pay.ir uses form-encoded POST bodies, an `api` field for authentication,
//! and the magic test API key `"test"` for sandbox-style verification.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "payir";
const API: &str = "https://pay.ir";

/// Pay.ir gateway driver.
pub struct PayIr {
    api_key: String,
    api_base: String,
    client: reqwest::Client,
}

impl PayIr {
    /// New driver with the given Pay.ir API key.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_base: API.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Use Pay.ir's `"test"` API key — every payment in this mode is
    /// simulated and no real money moves.
    #[must_use]
    pub fn sandbox() -> Self {
        Self::new("test")
    }

    /// Override the API base URL (for tests).
    #[must_use]
    pub fn with_api_base(mut self, url: impl Into<String>) -> Self {
        self.api_base = url.into();
        self
    }

    /// Override the HTTP client.
    #[must_use]
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl Gateway for PayIr {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("api", self.api_key.clone());
        form.insert("amount", req.amount.as_rials().to_string());
        form.insert("redirect", req.callback_url.clone());
        form.insert("description", req.description.clone());
        if let Some(m) = &req.mobile {
            form.insert("mobile", m.clone());
        }
        if let Some(o) = &req.order_id {
            form.insert("factorNumber", o.clone());
        }

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/pg/send", self.api_base))
            .form(&form)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: PayIrStart = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;

        if parsed.status != 1 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.status,
                message: parsed.error_message.unwrap_or_default(),
            });
        }
        let token = parsed
            .token
            .ok_or_else(|| Error::decode(PROVIDER, "start: missing token"))?;

        let payment_url = format!("{}/pg/{}", self.api_base, token);

        Ok(StartResponse {
            authority: token,
            payment_url,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("api", self.api_key.clone());
        form.insert("token", req.authority.clone());

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/pg/verify", self.api_base))
            .form(&form)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: PayIrVerify = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;

        if parsed.status != 1 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.status,
                message: parsed.message.unwrap_or_default(),
            });
        }

        let actual = Amount::rial(parsed.amount.unwrap_or(req.amount.as_rials()));
        if actual != req.amount {
            return Err(Error::AmountMismatch {
                expected: req.amount,
                actual,
            });
        }

        Ok(VerifyResponse {
            transaction_id: parsed.trans_id.unwrap_or_else(|| req.authority.clone()),
            authority: req.authority.clone(),
            amount: actual,
            card_pan: parsed.card_number,
            card_hash: None,
            fee: None,
            provider: PROVIDER,
            raw,
        })
    }
}

// ── wire types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct PayIrStart {
    status: i64,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    error_message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PayIrVerify {
    status: i64,
    #[serde(default)]
    amount: Option<i64>,
    #[serde(default, alias = "transId")]
    trans_id: Option<String>,
    #[serde(default, alias = "cardNumber")]
    card_number: Option<String>,
    #[serde(default)]
    message: Option<String>,
}
