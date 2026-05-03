//! End-to-end showcase of the newer features added in v0.2:
//! car plates, time expressions, ZWNJ insertion, transliteration, spell
//! suggestions, money formatting, landline operator detection, and (when
//! the `jalali` feature is on) Jalali date parsing.
//!
//! Run with: `cargo run --example showcase -p parsitext --all-features`

use parsitext::{
    money::{self, MoneyUnit},
    spell, transliterate,
    validators::{car_plate, landline, phone},
    zwnj_insert, Parsitext,
};

fn main() {
    let pt = Parsitext::default();

    println!("─── ZWNJ insertion ───");
    for w in ["میروم", "نمیدانم", "کتابها", "کتابهای"] {
        println!("  {w}  →  {}", zwnj_insert::insert(w));
    }

    println!("\n─── Transliteration ───");
    for w in ["سلام", "کتاب", "ایران", "خوش آمدید"] {
        println!("  {w}  →  {}", transliterate::to_latin(w));
    }

    println!("\n─── Spell suggestions ───");
    let dict = ["سلام", "سلامتی", "سفر", "سفره", "کتاب", "کتیبه"];
    for query in ["سلامن", "سفری", "کتیب"] {
        println!("  {query}  →  {:?}", spell::suggest(query, &dict, 2));
    }

    println!("\n─── Car plate ───");
    for plate in ["12 ب 345 - 67", "۱۲ب۳۴۵۶۷", "12 X 345 67"] {
        match car_plate::parse(plate) {
            Some(p) => println!("  {plate:<22}  → {} (province {})", p, p.province),
            None => println!("  {plate:<22}  → invalid"),
        }
    }

    println!("\n─── Landline operator detection ───");
    for p in ["02112345678", "03112345678", "04112345678", "05112345678"] {
        let prov = landline::province(p)
            .map(|p| {
                format!(
                    "{} / {}",
                    p.english_name().unwrap_or("?"),
                    p.persian_name().unwrap_or("?")
                )
            })
            .unwrap_or_else(|| "(unknown)".into());
        println!("  {p}  →  {prov}");
    }

    println!("\n─── Mobile + Landline together ───");
    for p in ["09121234567", "02144556677", "+989351234567"] {
        let kind = if phone::validate(p) {
            format!("mobile / {}", phone::operator(p).unwrap())
        } else if landline::validate(p) {
            format!(
                "landline / {}",
                landline::province(p)
                    .and_then(|p| p.english_name())
                    .unwrap_or("unknown province")
            )
        } else {
            "invalid".into()
        };
        println!("  {p:<20}  →  {kind}");
    }

    println!("\n─── Money formatting ───");
    println!("  {}", money::format(2_500_000, MoneyUnit::Toman));
    println!("  {}", money::format(500, MoneyUnit::Rial));
    println!("  {}", money::format(-1_000_000, MoneyUnit::Toman));

    println!("\n─── Levenshtein distance ───");
    for (a, b) in [("کتاب", "کتیب"), ("سلام", "سلامتی"), ("kitten", "sitting")] {
        println!("  {a:<10} ↔ {b:<10} → {}", pt.levenshtein(a, b));
    }

    #[cfg(feature = "jalali")]
    {
        println!("\n─── Jalali date parsing (with `jalali` feature) ───");
        for s in [
            "1402/03/15",
            "۱۵ تیر ۱۴۰۲",
            "1404/12/30 (invalid: not leap)",
        ] {
            let core = s.split(" (").next().unwrap();
            match pt.parse_jalali_date(core) {
                Some(d) => println!("  {s:<40}  → {} ({})", d, d.month_name()),
                None => println!("  {s:<40}  → invalid"),
            }
        }
    }
}
