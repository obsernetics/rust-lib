//! Apply user-defined replacement rules for brand-name normalisation.
//!
//! Run with: `cargo run --example custom_rules -p parsitext`

use parsitext::{CustomRule, Parsitext, ParsitextConfig};

fn main() {
    let pt = Parsitext::new(
        ParsitextConfig::builder()
            // Brand name anywhere in the text (whole_word = false).
            .add_rule(CustomRule::new("دیجیکالا", "Digikala", false))
            // Brand name only as a standalone word; "اسنپ فود" keeps اسنپ replaced
            // because it is bounded by spaces, but a hypothetical "اسنپکار" would not.
            .add_rule(CustomRule::new("اسنپ", "Snapp", true))
            .build(),
    );

    let original = "خرید از دیجیکالا بهتره یا اسنپ فود؟";
    let result = pt.process(original);

    println!("Original  : {}", result.original);
    println!("Normalized: {}", result.normalized);
    // Expected output contains "Digikala" and "Snapp" in place of the Persian names.
}
