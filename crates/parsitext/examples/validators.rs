//! Validate Iranian identifiers (national ID, IBAN, bank card, phone) and
//! detect issuing banks and mobile operators.
//!
//! Run with: `cargo run --example validators -p parsitext`

use parsitext::{
    money, numbers,
    validators::{bank_card, national_id, phone, postal_code, sheba, Operator},
    Parsitext,
};

fn main() {
    println!("─── National ID ───");
    for id in ["0499370899", "1234567890", "0000000000"] {
        println!("  {id}  →  valid = {}", national_id::validate(id));
    }

    println!("\n─── Sheba (IBAN) ───");
    for iban in [
        "IR062960000000100324200001",
        "IR06 2960 0000 0010 0324 2000 01",
        "IR000000000000000000000000",
    ] {
        let v = sheba::validate(iban);
        let bank = sheba::bank(iban).unwrap_or("(unknown)");
        println!("  {iban:<32}  valid={v}  bank={bank}");
    }

    println!("\n─── Bank Card ───");
    for card in [
        "6037990000000006",
        "6037-9900-0000-0006",
        "0000000000000000",
    ] {
        let v = bank_card::validate(card);
        let bank = bank_card::bank(card).unwrap_or("(unknown)");
        println!("  {card:<22}  valid={v}  bank={bank}");
    }

    println!("\n─── Mobile Phone ───");
    for p in ["09121234567", "+989351234567", "00989221234567", "08123456"] {
        if let Some(canon) = phone::canonicalize(p) {
            let op = phone::operator(p).unwrap_or(Operator::Other);
            println!("  {p:<20}  →  {canon}  ({op} / {})", op.persian_name());
        } else {
            println!("  {p:<20}  →  invalid");
        }
    }

    println!("\n─── Postal Code ───");
    for code in ["1969833114", "19698-33114", "0123456789"] {
        println!("  {code:<14}  valid={}", postal_code::validate(code));
    }

    println!("\n─── Number ↔ Words ───");
    for n in [0i64, 21, 105, 1234, 1_500_000, -42] {
        println!("  {n:>10}  →  {}", numbers::to_words(n));
    }
    for words in ["بیست و یک", "صد و پنج", "دو میلیون و پانصد هزار"]
    {
        println!("  {words:<28}  →  {:?}", numbers::from_words(words));
    }

    println!("\n─── Money parsing ───");
    for s in [
        "۵۰۰ هزار تومان",
        "۱.۵ میلیون تومن",
        "دو میلیون و پانصد هزار ریال",
    ] {
        if let Some(m) = money::parse(s) {
            println!(
                "  {s:<32}  →  {} {} (= {} Rial)",
                m.value,
                m.unit,
                m.as_rials()
            );
        }
    }

    println!("\n─── Stemmer ───");
    let pt = Parsitext::default();
    for word in ["کتاب‌ها", "کتاب‌هایم", "بزرگ‌ترین", "روزها"] {
        println!("  {word}  →  {}", pt.stem(word));
    }

    println!("\n─── Entity recognition (validates checksums) ───");
    let text = "کد ملی: 0499370899، کارت: 6037990000000006، شبا: IR062960000000100324200001، \
                موبایل: 09121234567، کد پستی: 19698-33114";
    for e in pt.detect_entities(text) {
        let extra = e.normalized.as_deref().unwrap_or("");
        println!("  [{}]  {:<35}  {}", e.kind, e.text, extra);
    }
}
