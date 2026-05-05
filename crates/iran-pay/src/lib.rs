//! # iran-pay
//!
//! Unified async SDK for Iranian payment gateways.  One [`Gateway`] trait,
//! **six production drivers** (ZarinPal, IDPay, NextPay, Pay.ir, Zibal,
//! Vandar), shared strongly-typed request/response/error types, an in-memory
//! mock gateway, security helpers (HMAC verification, constant-time compare,
//! amount-mismatch guard), and per-provider API-version pinning.  See
//! [VERSIONING.md](https://github.com/obsernetics/rust-lib/blob/main/crates/iran-pay/VERSIONING.md)
//! for the upgrade policy.
//!
//! ## At a glance
//!
//! ```no_run
//! use iran_pay::{Amount, Gateway, StartRequest, VerifyRequest};
//! use iran_pay::providers::ZarinPal;
//!
//! # async fn run() -> Result<(), iran_pay::Error> {
//! let gateway = ZarinPal::new("YOUR-MERCHANT-UUID").sandbox();
//!
//! // 1. Initiate the payment.
//! let start = gateway.start_payment(&StartRequest::builder()
//!     .amount(Amount::toman(50_000))
//!     .description("Pro subscription — May 2026")
//!     .callback_url("https://example.com/payment/callback")
//!     .order_id("ORD-12345")
//!     .build()).await?;
//!
//! // 2. Redirect the user to `start.payment_url`.
//! println!("Send user to: {}", start.payment_url);
//!
//! // 3. After they return, verify.  Pass the same amount back in to
//! //    catch tampering with the callback query string.
//! let verified = gateway.verify_payment(&VerifyRequest {
//!     authority: start.authority,
//!     amount: Amount::toman(50_000),
//! }).await?;
//!
//! println!("Paid! Transaction ID = {}", verified.transaction_id);
//! # Ok(()) }
//! ```
//!
//! ## Why a trait?
//!
//! Iranian e-commerce apps often switch gateways (or run several in parallel
//! for redundancy / fee optimisation).  Code your checkout against
//! `dyn Gateway` or `impl Gateway` and you can swap providers with one line
//! of configuration.
//!
//! ```ignore
//! fn select_gateway(provider: &str) -> Box<dyn Gateway> {
//!     match provider {
//!         "zarinpal" => Box::new(ZarinPal::new(env::var("ZP_ID").unwrap())),
//!         "idpay"    => Box::new(IDPay::new(env::var("IDPAY_KEY").unwrap())),
//!         "nextpay"  => Box::new(NextPay::new(env::var("NEXTPAY_KEY").unwrap())),
//!         "payir"    => Box::new(PayIr::new(env::var("PAYIR_KEY").unwrap())),
//!         _ => unreachable!(),
//!     }
//! }
//! ```
//!
//! ## Sandbox / test mode
//!
//! Every provider exposes `.sandbox()` to flip to its test endpoint.  No real
//! money moves.  Use this in CI and for local development.
//!
//! ## Mock gateway
//!
//! For unit tests of *your* code, use [`mock::MockGateway`] — it implements
//! [`Gateway`] without any network I/O, and lets you script success / failure
//! responses programmatically.
//!
//! ## Cargo features
//!
//! | Feature      | Default | What it enables                               |
//! |--------------|---------|-----------------------------------------------|
//! | `zarinpal`   | ✓       | The [`providers::ZarinPal`] driver            |
//! | `idpay`      | ✓       | The [`providers::IDPay`] driver               |
//! | `nextpay`    | ✓       | The [`providers::NextPay`] driver             |
//! | `payir`      | ✓       | The [`providers::PayIr`] driver               |
//! | `zibal`      | ✓       | The [`providers::Zibal`] driver               |
//! | `vandar`     | ✓       | The [`providers::Vandar`] driver              |
//! | `validators` | ✓       | Re-export `parsitext`'s Iranian validators    |
//! | `rustls-tls` | ✓       | Use rustls for HTTPS (no system OpenSSL)      |
//! | `native-tls` |         | Use the platform TLS library                  |
//!
//! Disabling all provider features still gives you the trait, types, mock,
//! and security helpers — useful if you build your own driver against this
//! abstraction.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

mod amount;
mod error;
mod gateway;
mod types;

pub mod mock;
pub mod providers;
pub mod security;

#[cfg(feature = "validators")]
#[cfg_attr(docsrs, doc(cfg(feature = "validators")))]
pub mod validators;

pub use amount::{Amount, Currency};
pub use error::Error;
pub use gateway::Gateway;
pub use types::{
    RefundRequest, RefundResponse, StartRequest, StartRequestBuilder, StartResponse, VerifyRequest,
    VerifyResponse,
};

/// Crate-wide `Result` alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Re-exports of the most commonly used types.
///
/// `use iran_pay::prelude::*;` brings everything you need for a typical
/// checkout flow into scope.
pub mod prelude {
    pub use crate::{
        Amount, Currency, Error, Gateway, RefundRequest, RefundResponse, Result, StartRequest,
        StartResponse, VerifyRequest, VerifyResponse,
    };
}
