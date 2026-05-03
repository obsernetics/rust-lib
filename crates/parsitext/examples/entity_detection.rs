//! Detect structured entities in Persian text (phone, date, money, URL, mention).
//!
//! Run with: `cargo run --example entity_detection -p parsitext`

use parsitext::Parsitext;

fn main() {
    let pt = Parsitext::default();

    let text = "زنگ بزن به 09121234567. قرار تاریخ ۱۵ تیر ۱۴۰۲. هزینه ۵۰۰ هزار تومان. لینک: https://example.ir. @reza123 رو ببین.";

    let result = pt.process(text);

    println!("Input text:");
    println!("  {text}");
    println!();
    println!("Normalized:");
    println!("  {}", result.normalized);
    println!();
    println!("Detected entities ({}):", result.entity_count());

    for entity in &result.entities {
        println!(
            "  kind={:?}  text={:?}  span={}..{}{}",
            entity.kind,
            entity.text,
            entity.span.start,
            entity.span.end,
            entity
                .normalized
                .as_ref()
                .map(|n| format!("  canonical={n:?}"))
                .unwrap_or_default(),
        );
    }

    println!();
    println!("Total: {} entities found", result.entity_count());
}
