//! Process a large slice of Persian texts in parallel using process_batch.
//!
//! Run with: `cargo run --example batch_processing -p parsitext`

use parsitext::Parsitext;

fn main() {
    let pt = Parsitext::default();

    // Build 50 texts by repeating a short phrase with a per-item index.
    let texts: Vec<String> = (1..=50)
        .map(|i| format!("متن شماره {} - سلام داداش قيمتش ١ میلیون تومنه", i))
        .collect();

    let results = pt.process_batch(&texts);

    println!("Processed {} texts.", results.len());
    println!();

    let first = &results[0];
    println!("First result:");
    println!("  Original  : {}", first.original);
    println!("  Normalized: {}", first.normalized);
    println!("  Time (ns) : {}", first.stats.processing_time_ns);
    println!();

    let last = results.last().unwrap();
    println!("Last result:");
    println!("  Original  : {}", last.original);
    println!("  Normalized: {}", last.normalized);
}
