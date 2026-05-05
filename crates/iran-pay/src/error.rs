//! Strongly-typed errors returned by every [`Gateway`](crate::Gateway) call.

use thiserror::Error;

use crate::Amount;

/// Errors returned by gateway drivers.
///
/// Variants fall into three families:
///
/// 1. **Transport** — [`Error::Http`] wraps every `reqwest` failure
///    (connection refused, TLS handshake, timeout, malformed response, …).
/// 2. **Gateway** — [`Error::Gateway`] is returned when a provider's API
///    accepts the request but reports a *business* failure (insufficient
///    funds, expired authority, blocked merchant, etc.).  The contained
///    `code` is the raw provider code; check provider docs to interpret.
/// 3. **Local** — [`Error::Config`], [`Error::AmountMismatch`],
///    [`Error::Unsupported`] are produced inside the SDK before/after the
///    HTTP roundtrip.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP transport / serialisation failure.
    #[error("HTTP request to {provider} gateway failed: {source}")]
    Http {
        /// Driver name (`"zarinpal"`, `"idpay"`, …).
        provider: &'static str,
        /// Underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },

    /// Gateway returned a business-level error.
    ///
    /// `code` is the provider's native error code; consult the provider's
    /// documentation to interpret it.  `message` is the human-readable
    /// message (often Persian).
    #[error("{provider} gateway error (code {code}): {message}")]
    Gateway {
        /// Driver name.
        provider: &'static str,
        /// Provider-specific numeric error code.
        code: i64,
        /// Provider's human-readable message.
        message: String,
    },

    /// Verification was attempted with an amount that doesn't match what
    /// was originally charged.  Almost always indicates someone tampered
    /// with the callback query string.
    #[error("amount mismatch — expected {expected}, gateway reported {actual}")]
    AmountMismatch {
        /// What the merchant expected.
        expected: Amount,
        /// What the gateway reported during verification.
        actual: Amount,
    },

    /// Configuration is invalid (missing merchant ID, malformed URL, etc.).
    #[error("invalid configuration: {0}")]
    Config(String),

    /// The provider does not support this operation (e.g. refunds via Pay.ir
    /// require a separate API contract).
    #[error("{operation} is not supported by the {provider} gateway")]
    Unsupported {
        /// Driver name.
        provider: &'static str,
        /// The unsupported operation.
        operation: &'static str,
    },

    /// Response decoding failed — the provider returned a payload we couldn't
    /// match to the expected schema.  Usually means the SDK is out of date
    /// relative to the provider's API.
    #[error("could not decode {provider} response: {message}")]
    Decode {
        /// Driver name.
        provider: &'static str,
        /// Description of what went wrong.
        message: String,
    },
}

impl Error {
    /// Helper: build an [`Error::Http`] from a reqwest error and a driver
    /// name.  Used internally by every driver.
    #[allow(dead_code)] // unused when every provider feature is disabled
    pub(crate) fn http(provider: &'static str, source: reqwest::Error) -> Self {
        Self::Http { provider, source }
    }

    /// Helper: build an [`Error::Decode`] from a driver name and message.
    #[allow(dead_code)] // unused when every provider feature is disabled
    pub(crate) fn decode(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Decode {
            provider,
            message: message.into(),
        }
    }
}
