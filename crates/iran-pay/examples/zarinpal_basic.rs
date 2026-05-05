//! ZarinPal — minimal start-payment flow.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example zarinpal_basic -p iran-pay
//! ```
//!
//! Note: this example uses a placeholder merchant UUID and points at the
//! ZarinPal sandbox.  No real money moves.  The sandbox may also reject the
//! fake merchant ID outright — that's fine; the example is about showing the
//! shape of the API, not about actually contacting ZarinPal.

use iran_pay::providers::ZarinPal;
use iran_pay::{Amount, Gateway, StartRequest};

#[tokio::main]
async fn main() {
    // Replace with your real UUID in production; any UUID-shaped string works
    // for the sandbox.
    let gateway = ZarinPal::new("00000000-0000-0000-0000-000000000000").sandbox();

    // Build a typed request.  `Amount::toman(...)` is converted to Rials
    // automatically before hitting the wire — no unit-mix-up bugs.
    let req = StartRequest::builder()
        .amount(Amount::toman(50_000))
        .description("Pro subscription — May 2026")
        .callback_url("https://example.com/payment/callback")
        .order_id("ORD-12345")
        .email("buyer@example.com")
        .mobile("09121234567")
        .build();

    println!("calling ZarinPal sandbox …");
    match gateway.start_payment(&req).await {
        Ok(resp) => {
            println!("authority   : {}", resp.authority);
            println!("payment URL : {}", resp.payment_url);
            println!("provider    : {}", resp.provider);
            println!("→ in a real app, redirect the user to `payment_url`.");
        }
        Err(err) => {
            // The fake merchant UUID will almost certainly be rejected by
            // the sandbox.  This branch demonstrates how `iran-pay` surfaces
            // gateway errors without panicking.
            println!("(would call ZarinPal here; received: {err})");
        }
    }
}
