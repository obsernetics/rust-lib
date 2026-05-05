//! ZarinPal driver — Iran's most popular payment gateway.
//!
//! Implements ZarinPal's **v4 JSON API** at
//! `payment.zarinpal.com/pg/v4/payment/{request,verify}.json`
//! (verified against <https://www.zarinpal.com/docs/paymentGateway/connectToGateway>).
//! Sandbox endpoints (`sandbox.zarinpal.com`) are reachable via
//! [`ZarinPal::sandbox`].
//!
//! # Example
//!
//! ```no_run
//! use iran_pay::{Amount, Gateway, StartRequest};
//! use iran_pay::providers::ZarinPal;
//!
//! # async fn run() -> Result<(), iran_pay::Error> {
//! let gw = ZarinPal::new("00000000-0000-0000-0000-000000000000").sandbox();
//! let req = StartRequest::builder()
//!     .amount(Amount::toman(50_000))
//!     .description("Test payment")
//!     .callback_url("https://example.com/cb")
//!     .build();
//! let resp = gw.start_payment(&req).await?;
//! println!("{}", resp.payment_url);
//! # Ok(()) }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::instrument;

use crate::{
    Amount, Error, Gateway, Result, StartRequest, StartResponse, VerifyRequest, VerifyResponse,
};

const PROVIDER: &str = "zarinpal";
// Per https://www.zarinpal.com/docs/paymentGateway/connectToGateway, the
// canonical REST host is payment.zarinpal.com (api.zarinpal.com is reserved
// for the Payman / direct-debit API, not the standard gateway).
const PROD_API: &str = "https://payment.zarinpal.com";
const PROD_PAY: &str = "https://www.zarinpal.com";
const SANDBOX_API: &str = "https://sandbox.zarinpal.com";
const SANDBOX_PAY: &str = "https://sandbox.zarinpal.com";

/// ZarinPal gateway driver.
pub struct ZarinPal {
    merchant_id: String,
    api_base: String,
    pay_base: String,
    client: reqwest::Client,
}

impl ZarinPal {
    /// Create a driver with the given merchant UUID.
    ///
    /// In production, your merchant ID is the UUID issued by ZarinPal.
    /// For the sandbox, any UUID-shaped string works.
    #[must_use]
    pub fn new(merchant_id: impl Into<String>) -> Self {
        Self {
            merchant_id: merchant_id.into(),
            api_base: PROD_API.into(),
            pay_base: PROD_PAY.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Switch to the ZarinPal sandbox endpoints.
    #[must_use]
    pub fn sandbox(mut self) -> Self {
        self.api_base = SANDBOX_API.into();
        self.pay_base = SANDBOX_PAY.into();
        self
    }

    /// Override the API base URL (primarily for tests against `wiremock`).
    #[must_use]
    pub fn with_api_base(mut self, url: impl Into<String>) -> Self {
        self.api_base = url.into();
        self
    }

    /// Override the payment-redirect base URL (used to construct the
    /// `StartPay/{authority}` redirect).
    #[must_use]
    pub fn with_pay_base(mut self, url: impl Into<String>) -> Self {
        self.pay_base = url.into();
        self
    }

    /// Override the underlying [`reqwest::Client`] (e.g. to install a custom
    /// timeout or proxy).
    #[must_use]
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = client;
        self
    }
}

#[async_trait]
impl Gateway for ZarinPal {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, amount_rials = req.amount.as_rials()))]
    async fn start_payment(&self, req: &StartRequest) -> Result<StartResponse> {
        let url = format!("{}/pg/v4/payment/request.json", self.api_base);
        let body = json!({
            "merchant_id": self.merchant_id,
            "amount": req.amount.as_rials(),
            "callback_url": req.callback_url,
            "description": req.description,
            "metadata": {
                "email": req.email,
                "mobile": req.mobile,
                "order_id": req.order_id,
            }
        });

        let raw: serde_json::Value = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: ZpResp<ZpStartData> = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("start: {e}")))?;
        check_zp_errors(&parsed)?;

        let data = parsed
            .data
            .ok_or_else(|| Error::decode(PROVIDER, "start: missing `data`"))?;
        if data.code != 100 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: data.code,
                message: data.message,
            });
        }

        let payment_url = format!("{}/pg/StartPay/{}", self.pay_base, data.authority);

        Ok(StartResponse {
            authority: data.authority,
            payment_url,
            provider: PROVIDER,
            raw,
        })
    }

    #[instrument(skip(self, req), fields(provider = PROVIDER, authority = %req.authority))]
    async fn verify_payment(&self, req: &VerifyRequest) -> Result<VerifyResponse> {
        let url = format!("{}/pg/v4/payment/verify.json", self.api_base);
        let body = json!({
            "merchant_id": self.merchant_id,
            "authority": req.authority,
            "amount": req.amount.as_rials(),
        });

        let raw: serde_json::Value = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?
            .json()
            .await
            .map_err(|e| Error::http(PROVIDER, e))?;

        let parsed: ZpResp<ZpVerifyData> = serde_json::from_value(raw.clone())
            .map_err(|e| Error::decode(PROVIDER, format!("verify: {e}")))?;
        check_zp_errors(&parsed)?;

        let data = parsed
            .data
            .ok_or_else(|| Error::decode(PROVIDER, "verify: missing `data`"))?;

        // ZarinPal returns 100 for new verifies, 101 for "already verified".
        if data.code != 100 && data.code != 101 {
            return Err(Error::Gateway {
                provider: PROVIDER,
                code: data.code,
                message: data.message,
            });
        }

        Ok(VerifyResponse {
            transaction_id: data.ref_id.to_string(),
            authority: req.authority.clone(),
            amount: req.amount,
            card_pan: data.card_pan,
            card_hash: data.card_hash,
            fee: data.fee.map(Amount::rial),
            provider: PROVIDER,
            raw,
        })
    }
}

// ── wire types ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct ZpResp<D> {
    data: Option<D>,
    #[serde(default, deserialize_with = "deser_errors_loose")]
    errors: Vec<ZpError>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZpError {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZpStartData {
    code: i64,
    #[serde(default)]
    message: String,
    authority: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ZpVerifyData {
    code: i64,
    #[serde(default)]
    message: String,
    ref_id: i64,
    #[serde(default)]
    card_pan: Option<String>,
    #[serde(default)]
    card_hash: Option<String>,
    #[serde(default)]
    fee: Option<i64>,
}

/// ZarinPal sometimes returns `errors: []` and sometimes `errors: {}`; this
/// deserialiser accepts either as "no errors".
fn deser_errors_loose<'de, D>(d: D) -> std::result::Result<Vec<ZpError>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;
    let v = serde_json::Value::deserialize(d)?;
    match v {
        serde_json::Value::Array(a) => {
            serde_json::from_value(serde_json::Value::Array(a)).map_err(D::Error::custom)
        }
        // Object form (empty {} or wrapped error) — try as a single error.
        serde_json::Value::Object(o) if !o.is_empty() => {
            let one: ZpError =
                serde_json::from_value(serde_json::Value::Object(o)).map_err(D::Error::custom)?;
            Ok(vec![one])
        }
        _ => Ok(Vec::new()),
    }
}

fn check_zp_errors<D>(resp: &ZpResp<D>) -> Result<()> {
    if let Some(first) = resp.errors.first() {
        return Err(Error::Gateway {
            provider: PROVIDER,
            code: first.code,
            message: first.message.clone(),
        });
    }
    Ok(())
}
