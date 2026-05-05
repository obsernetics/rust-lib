//! NextPay driver.
//!
//! NextPay uses form-encoded POST bodies and an `api_key` field instead of
//! a header.  The redirect URL is `https://nextpay.org/nx/gateway/payment/{trans_id}`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "nextpay";
const API: &str = "https://nextpay.org";

/// NextPay gateway driver.
pub struct NextPay {
    api_key: String,
    api_base: String,
    client: reqwest::Client,
}

impl NextPay {
    /// New driver with the given NextPay API key.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_base: API.into(),
            client: reqwest::Client::new(),
        }
    }

    /// NextPay does not have a separate sandbox host; merchants test using
    /// a designated test API key on the production endpoint.  This method
    /// is a no-op kept for API symmetry with other providers.
    #[must_use]
    pub fn sandbox(self) -> Self {
        self
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
impl Gateway for NextPay {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        let order_id = req
            .order_id
            .clone()
            .unwrap_or_else(|| format!("ORD-{}", chrono_secs()));

        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("api_key", self.api_key.clone());
        form.insert("amount", req.amount.as_rials().to_string());
        form.insert("order_id", order_id);
        form.insert("callback_uri", req.callback_url.clone());
        form.insert("customer_phone", req.mobile.clone().unwrap_or_default());
        form.insert("custom_json_fields", "{}".into());
        form.insert(
            "payer_name",
            req.extras.get("name").cloned().unwrap_or_default(),
        );
        form.insert("payer_desc", req.description.clone());

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/nx/gateway/token", self.api_base))
            .form(&form)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: NpStart = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;

        // NextPay code -1 = success; any other negative = error.
        if parsed.code != -1 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.code,
                message: format!("nextpay code {}", parsed.code),
            });
        }
        let trans_id = parsed
            .trans_id
            .ok_or_else(|| Error::decode(PROVIDER, "start: missing trans_id"))?;

        let payment_url = format!("{}/nx/gateway/payment/{}", self.api_base, trans_id);

        Ok(StartResponse {
            authority: trans_id,
            payment_url,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let mut form: HashMap<&str, String> = HashMap::new();
        form.insert("api_key", self.api_key.clone());
        form.insert("trans_id", req.authority.clone());
        form.insert("amount", req.amount.as_rials().to_string());

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/nx/gateway/verify", self.api_base))
            .form(&form)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: NpVerify = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;

        if parsed.code != 0 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.code,
                message: format!("nextpay code {}", parsed.code),
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
            transaction_id: parsed
                .shaparak_ref_id
                .unwrap_or_else(|| req.authority.clone()),
            authority: req.authority.clone(),
            amount: actual,
            card_pan: parsed.card_holder,
            card_hash: None,
            fee: None,
            provider: PROVIDER,
            raw,
        })
    }
}

fn chrono_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── wire types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct NpStart {
    code: i64,
    #[serde(default)]
    trans_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct NpVerify {
    code: i64,
    #[serde(default)]
    amount: Option<i64>,
    #[serde(default)]
    shaparak_ref_id: Option<String>,
    #[serde(default)]
    card_holder: Option<String>,
}
