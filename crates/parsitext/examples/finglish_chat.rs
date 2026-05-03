//! Finglish (Persian-in-Latin) and chat / GenZ register conversions.
//!
//! Run with: `cargo run --example finglish_chat -p parsitext`

use parsitext::{finglish, phonetic, spell, style};

fn main() {
    println!("─── Finglish → Persian ───");
    for s in [
        "salam khoobi?",
        "merci, man khoobam",
        "chetori? mikham beram khoone",
        "kheyli mamnoonam az komaketoon",
        "bye, fardaa mibinamet",
    ] {
        println!("  {s:<40}  →  {}", finglish::to_persian(s));
    }

    println!("\n─── Formal → Chat ───");
    for s in [
        "می‌خواهم به خانه بروم",
        "نمی‌دانم چه می‌گویند",
        "می‌توانم به شما کمک کنم",
    ] {
        println!("  {s:<35}  →  {}", style::to_chat(s));
    }

    println!("\n─── Chat → Formal (slang expansion) ───");
    for s in ["میخوام برم خونه", "نمیدونم چیه", "میتونم کمکت کنم"]
    {
        println!("  {s:<25}  →  {}", style::to_formal(s));
    }

    println!("\n─── Persian → GenZ ───");
    for s in [
        "ممنون از دعوت به مهمانی! خیلی عالی بود.",
        "بله، واقعاً خنده‌دار بود",
        "این فیلم بسیار جذاب بود",
    ] {
        println!("  {s}");
        println!("    →  {}", style::to_genz(s));
    }

    println!("\n─── Finglish → GenZ Persian (composed) ───");
    for s in ["salam, mikhay biay party emshab?", "merci! kheyli ali bood"] {
        println!("  {s:<40}  →  {}", style::to_genz(s));
    }

    println!("\n─── Phonetic matching (Persian Soundex) ───");
    for (a, b) in [
        ("صبر", "سبر"),  // both [s][b][r] phonetically
        ("ذرت", "زرت"),  // both [z][r][t]
        ("کتاب", "سفر"), // distinct
    ] {
        let same = phonetic::matches(a, b);
        println!(
            "  {a} ↔ {b}  →  {} (codes {} / {})",
            if same { "MATCH" } else { "differ" },
            phonetic::soundex(a),
            phonetic::soundex(b)
        );
    }

    println!("\n─── Spell-check (built-in dict) ───");
    for typo in ["سلامن", "کتابب", "خوبتر"] {
        let suggestions = spell::suggest_builtin(typo, 1);
        println!("  {typo}  →  {suggestions:?}");
    }
}
