//! Concrete [`Gateway`](crate::Gateway) drivers.
//!
//! Each provider lives behind its own Cargo feature and can be opted out of
//! to keep your binary smaller.
//!
//! | Provider                    | Default | API style          |
//! |-----------------------------|---------|--------------------|
//! | [`ZarinPal`]                | ✓       | JSON v4            |
//! | [`IDPay`]                   | ✓       | JSON v1.1          |
//! | [`NextPay`]                 | ✓       | Form-encoded       |
//! | [`PayIr`]                   | ✓       | Form-encoded       |
//!
//! All four implement the same [`Gateway`](crate::Gateway) trait so your
//! checkout code can be provider-agnostic.

#[cfg(feature = "idpay")]
#[cfg_attr(docsrs, doc(cfg(feature = "idpay")))]
mod idpay;

#[cfg(feature = "nextpay")]
#[cfg_attr(docsrs, doc(cfg(feature = "nextpay")))]
mod nextpay;

#[cfg(feature = "payir")]
#[cfg_attr(docsrs, doc(cfg(feature = "payir")))]
mod payir;

#[cfg(feature = "vandar")]
#[cfg_attr(docsrs, doc(cfg(feature = "vandar")))]
mod vandar;

#[cfg(feature = "zarinpal")]
#[cfg_attr(docsrs, doc(cfg(feature = "zarinpal")))]
mod zarinpal;

#[cfg(feature = "zibal")]
#[cfg_attr(docsrs, doc(cfg(feature = "zibal")))]
mod zibal;

#[cfg(feature = "idpay")]
#[cfg_attr(docsrs, doc(cfg(feature = "idpay")))]
pub use idpay::IDPay;

#[cfg(feature = "nextpay")]
#[cfg_attr(docsrs, doc(cfg(feature = "nextpay")))]
pub use nextpay::NextPay;

#[cfg(feature = "payir")]
#[cfg_attr(docsrs, doc(cfg(feature = "payir")))]
pub use payir::PayIr;

#[cfg(feature = "vandar")]
#[cfg_attr(docsrs, doc(cfg(feature = "vandar")))]
pub use vandar::Vandar;

#[cfg(feature = "zarinpal")]
#[cfg_attr(docsrs, doc(cfg(feature = "zarinpal")))]
pub use zarinpal::ZarinPal;

#[cfg(feature = "zibal")]
#[cfg_attr(docsrs, doc(cfg(feature = "zibal")))]
pub use zibal::Zibal;
