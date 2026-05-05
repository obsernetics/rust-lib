//! Vandar driver.
//!
//! API verified against Vandar's official IPG documentation
//! (<https://vandarpay.github.io/docs/ipg/>):
//!
//! - **Request**: `POST https://ipg.vandar.io/api/v3/send` (JSON)
//! - **Verify**:  `POST https://ipg.vandar.io/api/v3/verify` (JSON)
//! - **Redirect**: `https://ipg.vandar.io/v3/{token}`
//! - **Auth**: `api_key` is sent as a JSON body field, **not** a header.
//! - **Success status**: `status == 1`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "vandar";
const API: &str = "https://ipg.vandar.io";

/// Vandar gateway driver.
pub struct Vandar {
    api_key: String,
    api_base: String,
    client: reqwest::Client,
}

impl Vandar {
    /// New driver with the given Vandar API key (from your business dashboard).
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_base: API.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Vandar does not currently expose a separate sandbox host; use a test
    /// API key on the production endpoint per their docs.  Kept for API
    /// symmetry with other providers.
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
impl Gateway for Vandar {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        // Vandar enforces a minimum of 1000 Rials.
        if req.amount.as_rials() < 1_000 {
            return Err(Error::Config(format!(
                "vandar: amount must be at least 1000 Rials (got {} Rials)",
                req.amount.as_rials()
            )));
        }

        let body = json!({
            "api_key": self.api_key,
            "amount": req.amount.as_rials(),
            "callback_url": req.callback_url,
            "description": req.description,
            "mobile_number": req.mobile,
            "factorNumber": req.order_id,
            "national_code": req.extras.get("national_code"),
        });

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/api/v3/send", self.api_base))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: VandarStart = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;

        if parsed.status != 1 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.status,
                message: parsed
                    .errors
                    .as_ref()
                    .and_then(|v| v.first().cloned())
                    .unwrap_or_default(),
            });
        }
        let token = parsed
            .token
            .ok_or_else(|| Error::decode(PROVIDER, "start: missing token"))?;

        let payment_url = format!("{}/v3/{}", self.api_base, token);

        Ok(StartResponse {
            authority: token,
            payment_url,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let body = json!({
            "api_key": self.api_key,
            "token": req.authority,
        });

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/api/v3/verify", self.api_base))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: VandarVerify = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;

        if parsed.status != 1 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.status,
                message: parsed
                    .errors
                    .as_ref()
                    .and_then(|v| v.first().cloned())
                    .unwrap_or_default(),
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
                .trans_id
                .map(|n| n.to_string())
                .unwrap_or_else(|| req.authority.clone()),
            authority: req.authority.clone(),
            amount: actual,
            card_pan: parsed.card_number,
            card_hash: parsed.cid,
            fee: None,
            provider: PROVIDER,
            raw,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct VandarStart {
    status: i64,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    errors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VandarVerify {
    status: i64,
    #[serde(default)]
    amount: Option<i64>,
    #[serde(default, rename = "transId")]
    trans_id: Option<i64>,
    #[serde(default, rename = "cardNumber")]
    card_number: Option<String>,
    /// Hashed card number ("CID") if provided.
    #[serde(default, rename = "CID")]
    cid: Option<String>,
    #[serde(default)]
    errors: Option<Vec<String>>,
}
