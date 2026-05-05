//! Zibal driver.
//!
//! Zibal is a popular alternative to ZarinPal among Iranian merchants.
//! API verified against the official Node.js SDK
//! (<https://github.com/zibalco/gateway-nodejs>):
//!
//! - **Request**: `POST https://gateway.zibal.ir/v1/request` (JSON)
//! - **Verify**:  `POST https://gateway.zibal.ir/v1/verify`  (JSON)
//! - **Redirect**: `https://gateway.zibal.ir/start/{trackId}`
//! - **Test merchant**: pass `"zibal"` as the merchant code.
//! - **Success code**: `result == 100`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "zibal";
const API: &str = "https://gateway.zibal.ir";

/// Zibal gateway driver.
pub struct Zibal {
    merchant: String,
    api_base: String,
    client: reqwest::Client,
}

impl Zibal {
    /// New driver with the given merchant code.  In production this is the
    /// merchant code from your Zibal dashboard.
    #[must_use]
    pub fn new(merchant: impl Into<String>) -> Self {
        Self {
            merchant: merchant.into(),
            api_base: API.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Use Zibal's reserved test merchant code (`"zibal"`).  Every transaction
    /// in this mode is simulated.
    #[must_use]
    pub fn sandbox() -> Self {
        Self::new("zibal")
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
impl Gateway for Zibal {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        let body = json!({
            "merchant": self.merchant,
            "amount": req.amount.as_rials(),
            "callbackUrl": req.callback_url,
            "description": req.description,
            "mobile": req.mobile,
            "orderId": req.order_id,
        });

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/v1/request", self.api_base))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: ZibalStart = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;

        if parsed.result != 100 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.result,
                message: parsed.message.unwrap_or_default(),
            });
        }
        let track_id = parsed
            .track_id
            .ok_or_else(|| Error::decode(PROVIDER, "start: missing trackId"))?;

        let payment_url = format!("{}/start/{}", self.api_base, track_id);

        Ok(StartResponse {
            authority: track_id.to_string(),
            payment_url,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let track_id: i64 = req
            .authority
            .parse()
            .map_err(|_| Error::decode(PROVIDER, "verify: trackId not a valid integer"))?;

        let body = json!({
            "merchant": self.merchant,
            "trackId": track_id,
        });

        let raw: serde_json::Value = self
            .client
            .post(format!("{}/v1/verify", self.api_base))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: ZibalVerify = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;

        if parsed.result != 100 && parsed.result != 201 {
            // 201 = "already verified" per Zibal docs.
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: parsed.result,
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
            transaction_id: parsed
                .ref_number
                .map(|n| n.to_string())
                .unwrap_or_else(|| req.authority.clone()),
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

#[derive(Debug, Deserialize, Serialize)]
struct ZibalStart {
    result: i64,
    #[serde(default, rename = "trackId")]
    track_id: Option<i64>,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZibalVerify {
    result: i64,
    #[serde(default)]
    amount: Option<i64>,
    #[serde(default, rename = "refNumber")]
    ref_number: Option<i64>,
    #[serde(default, rename = "cardNumber")]
    card_number: Option<String>,
    #[serde(default)]
    message: Option<String>,
}
