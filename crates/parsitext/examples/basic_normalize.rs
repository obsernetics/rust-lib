//! Normalize Persian text — orthography, digits, ZWNJ, and repetition.
//!
//! Run with: `cargo run --example basic_normalize -p parsitext`

use parsitext::Parsitext;

fn main() {
    let pt = Parsitext::default();

    let original = "سلام داداش، چطوري؟ قيمتش حدود 1.5 میلیون تومنه، خيليييي گرونه!";
    let result = pt.process(original);

    println!("Original  : {}", result.original);
    println!("Normalized: {}", result.normalized);
    println!();

    // Highlight what changed:
    // 1. Arabic ي → Persian ی  (orthography)
    println!(
        "Contains Persian yeh (ی): {}",
        result.normalized.contains('ی')
    );
    // 2. Latin 1.5 → Persian ۱.۵  (digit unification)
    println!(
        "Contains Persian digit (۱): {}",
        result.normalized.contains('۱')
    );
    // 3. خيليييي (5× ي) → خییلی  (repetition capped at 2, Arabic yeh → Persian ی)
    println!(
        "Emphatic repetition reduced (no 3-in-a-row): {}",
        !result
            .normalized
            .chars()
            .collect::<Vec<_>>()
            .windows(3)
            .any(|w| w[0] == w[1] && w[1] == w[2])
    );
    println!();

    let tokens = pt.tokenize_only(original);
    println!("Token count: {}", tokens.len());
    println!("Tokens     : {:?}", tokens);
}
