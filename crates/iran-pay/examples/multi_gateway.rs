//! Polymorphic gateway selection via `Box<dyn Gateway>`.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example multi_gateway -p iran-pay
//! ```
//!
//! Real-world Iranian e-commerce apps typically support several gateways
//! and pick one per checkout (load balancing, fee optimisation, fall-back
//! when a provider is down).  Because every driver in this crate
//! implements the same [`Gateway`] trait, you can store them all in a
//! single homogeneous collection.

use iran_pay::providers::{IDPay, NextPay, PayIr, ZarinPal};
use iran_pay::Gateway;

fn main() {
    // Build a registry of every supported provider, each in sandbox/test
    // mode.  The keys are the human-friendly names you would store in your
    // database alongside the active merchant configuration.
    let registry: Vec<(String, Box<dyn Gateway>)> = vec![
        (
            "zarinpal".to_owned(),
            Box::new(ZarinPal::new("00000000-0000-0000-0000-000000000000").sandbox()),
        ),
        (
            "idpay".to_owned(),
            Box::new(IDPay::new("0000000000000000000000000000000000").sandbox()),
        ),
        (
            "nextpay".to_owned(),
            Box::new(NextPay::new("nextpay-sandbox-key").sandbox()),
        ),
        ("payir".to_owned(), Box::new(PayIr::sandbox())),
    ];

    println!("registered gateways:");
    for (key, gw) in &registry {
        // `gw.name()` is the canonical driver name; the key is whatever
        // the merchant called this configuration row.
        println!("  - {key:<10} → driver = {}", gw.name());
    }

    println!();
    println!("→ at checkout time, look up `gw` by `key` and call");
    println!("  `gw.start_payment(&req).await` exactly as you would for");
    println!("  any single provider.  No `match` on provider type needed.");
}
