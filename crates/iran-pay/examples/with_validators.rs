//! Pre-flight: validate buyer-supplied identifiers with `iran_pay::validators`.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example with_validators -p iran-pay
//! ```
//!
//! The `validators` Cargo feature (on by default) re-exports the Iranian
//! validators from the sibling `parsitext` crate — bank-card / Sheba /
//! national-ID checksums, mobile-operator detection, postal codes, etc.
//! Use them to guard your checkout form *before* you call
//! [`Gateway::start_payment`], so users get an immediate, local error
//! instead of a round-trip to the gateway.

use iran_pay::providers::ZarinPal;
use iran_pay::validators::{bank_card, phone, Operator};
use iran_pay::{Amount, Gateway, StartRequest};

#[tokio::main]
async fn main() {
    // ── 1. Bank card ────────────────────────────────────────────────────
    let card = "6037-9900-0000-0006";
    println!("card '{card}'");
    println!("  Luhn-valid : {}", bank_card::validate(card));
    println!("  issuer     : {:?}", bank_card::bank(card));

    // ── 2. Mobile phone + operator detection ────────────────────────────
    let raw_mobile = "+98 912 123 4567";
    let canonical = phone::canonicalize(raw_mobile);
    println!("\nmobile '{raw_mobile}'");
    println!("  canonical  : {canonical:?}");
    println!("  validates  : {}", phone::validate(raw_mobile));

    let op = phone::operator(raw_mobile);
    println!(
        "  operator   : {} ({})",
        op.map(|o| match o {
            Operator::MCI => "MCI / Hamrah-e Aval",
            Operator::Irancell => "Irancell",
            Operator::RighTel => "RighTel",
            Operator::ShatelMobile => "Shatel Mobile",
            Operator::Aptel => "Aptel",
            Operator::Other => "Other / MVNO",
        })
        .unwrap_or("unknown"),
        canonical.as_deref().unwrap_or("invalid"),
    );

    // ── 3. Now build the StartRequest with the validated, canonical mobile.
    let Some(mobile) = canonical else {
        eprintln!("invalid mobile — would reject at the form layer.");
        return;
    };

    let gateway = ZarinPal::new("00000000-0000-0000-0000-000000000000").sandbox();
    let req = StartRequest::builder()
        .amount(Amount::toman(50_000))
        .description("Pro subscription — May 2026")
        .callback_url("https://example.com/payment/callback")
        .order_id("ORD-12345")
        .mobile(mobile)
        .build();

    println!(
        "\nbuilt StartRequest with validated mobile = {:?}",
        req.mobile
    );

    // We don't actually fire the request here — the merchant UUID is fake.
    match gateway.start_payment(&req).await {
        Ok(resp) => println!("payment URL: {}", resp.payment_url),
        Err(err) => println!("(would call ZarinPal here; received: {err})"),
    }
}
