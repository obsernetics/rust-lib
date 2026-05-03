//! Structured-entity recognition for Iranian Persian text.
//!
//! All patterns operate on normalised text (after orthography and digit
//! passes).  Byte spans are valid UTF-8 boundaries because every pattern
//! matches complete Unicode scalar sequences.

use std::fmt;

use regex::Regex;

// ── span ─────────────────────────────────────────────────────────────────────

/// A `[start, end)` byte range within the normalised text string.
///
/// Byte offsets are returned directly from `regex` and are always valid
/// UTF-8 character boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: usize,
    /// Exclusive end byte offset.
    pub end: usize,
}

impl Span {
    /// Length of the span in bytes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns `true` if the span covers zero bytes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

// ── entity kind ───────────────────────────────────────────────────────────────

/// The semantic kind of a recognised entity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntityKind {
    /// Iranian mobile or landline phone number (validated by
    /// [`crate::validators::phone`]).
    PhoneNumber,
    /// Jalali (Shamsi) date in numeric format, e.g. `۱۴۰۲/۰۳/۱۵`.
    JalaliDateNumeric,
    /// Jalali date with a written Persian month name, e.g. `۱۵ تیر ۱۴۰۲`.
    JalaliDateTextual,
    /// A monetary amount followed by an Iranian currency word
    /// (تومان / تومن / ریال).  See [`crate::money::parse`] for structured
    /// parsing.
    MoneyAmount,
    /// Iranian national identification number (validated by
    /// [`crate::validators::national_id`]).
    NationalId,
    /// 16-digit Iranian bank card number (Luhn-validated).  See
    /// [`crate::validators::bank_card`].
    BankCard,
    /// Iranian Sheba / IBAN (`IR` + 24 digits, mod-97 checksum).  See
    /// [`crate::validators::sheba`].
    Iban,
    /// 10-digit Iranian postal code (validated by
    /// [`crate::validators::postal_code`]).
    PostalCode,
    /// Iranian vehicle plate (`12 ب 345 - 67`); validated by
    /// [`crate::validators::car_plate`].
    CarPlate,
    /// Persian time expression (`ساعت ۸ صبح`, `۱۲:۳۰`, `نیمه شب`).
    TimeExpression,
    /// 10- or 13-digit ISBN (validated by checksum).
    Isbn,
    /// IPv4 address (`192.168.1.1`).
    Ipv4,
    /// `@username` mention.
    Mention,
    /// `#hashtag`.
    Hashtag,
    /// HTTP or HTTPS URL.
    Url,
    /// E-mail address.
    Email,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EntityKind::PhoneNumber => "PhoneNumber",
            EntityKind::JalaliDateNumeric => "JalaliDate",
            EntityKind::JalaliDateTextual => "JalaliDate",
            EntityKind::MoneyAmount => "MoneyAmount",
            EntityKind::NationalId => "NationalId",
            EntityKind::BankCard => "BankCard",
            EntityKind::Iban => "IBAN",
            EntityKind::PostalCode => "PostalCode",
            EntityKind::CarPlate => "CarPlate",
            EntityKind::TimeExpression => "TimeExpression",
            EntityKind::Isbn => "ISBN",
            EntityKind::Ipv4 => "IPv4",
            EntityKind::Mention => "Mention",
            EntityKind::Hashtag => "Hashtag",
            EntityKind::Url => "URL",
            EntityKind::Email => "Email",
        };
        f.write_str(s)
    }
}

// ── entity ────────────────────────────────────────────────────────────────────

/// A single recognised entity.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entity {
    /// Semantic kind.
    pub kind: EntityKind,
    /// The exact substring as it appears in the normalised text.
    pub text: String,
    /// A normalised / structured rendering of the value, when available.
    ///
    /// - `PhoneNumber` — canonical `09XXXXXXXXX` form (Latin digits).
    /// - Other kinds — `None` for now.
    pub normalized: Option<String>,
    /// Position in the normalised text (byte offsets).
    pub span: Span,
}

impl fmt::Display for Entity {
    /// Formats as `[Kind: "raw text"]`.
    ///
    /// ```
    /// use parsitext::Parsitext;
    ///
    /// // Mention text without digits avoids digit-script conversion in the assertion.
    /// let entities = Parsitext::default().detect_entities("@reza");
    /// assert_eq!(entities[0].to_string(), "[Mention: \"@reza\"]");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}: \"{}\"]", self.kind, self.text)
    }
}

// ── regex patterns ────────────────────────────────────────────────────────────

// Iranian mobile: 09XXXXXXXXX, +989XXXXXXXXX (and Persian-digit equivalents).
// Uses character classes [0۰] / [9۹] / [8۸] so the pattern works regardless
// of whether digit normalisation has been applied.  `\d` matches both Latin
// and Persian decimal digits (Unicode-aware).
const PHONE_RE: &str = r"(?:\+[9۹][8۸]|[0۰][0۰][9۹][8۸]|[0۰])[9۹]\d{9}";

// Numeric Jalali date: 1402/03/15 or ۱۴۰۲/۰۳/۱۵
const DATE_NUM_RE: &str = r"\d{4}[/\-]\d{1,2}[/\-]\d{1,2}";

// Textual Jalali date: ۱۵ تیر ۱۴۰۲
const DATE_TEXT_RE: &str =
    r"\d{1,2}\s*(?:فروردین|اردیبهشت|خرداد|تیر|مرداد|شهریور|مهر|آبان|آذر|دی|بهمن|اسفند)\s*\d{2,4}";

// Money: "۵۰۰ هزار تومان", "۱.۵ میلیون تومن", "۲۰۰ ریال"
const MONEY_RE: &str = r"\d[\d,،.]*(?:\s*(?:هزار|میلیون|میلیارد))?\s*(?:تومان|تومن|ریال)";

// National ID: exactly 10 digits, word-boundary delimited.
const NATIONAL_ID_RE: &str = r"\b\d{10}\b";

// Bank card: 16 digits with optional `-` or space separators, word-boundary
// delimited.  Validation is then performed via the Luhn algorithm.
const BANK_CARD_RE: &str = r"\b\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}\b";

// Iranian IBAN (Sheba): IR + 24 digits with optional spaces/dashes
// (2-digit checksum + 5 four-digit groups + 2-digit tail = 24 digits).
const IBAN_RE: &str =
    r"IR[\s\-]?\d{2}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{4}[\s\-]?\d{2}";

// Postal code with explicit XXXXX-XXXXX hyphen.
const POSTAL_CODE_RE: &str = r"\b\d{5}-\d{5}\b";

// Iranian car plate: 2 digits + Persian letter + 3 digits + 2-digit province
// code, with optional whitespace / dash separators between groups.
const CAR_PLATE_RE: &str = r"\d{2}\s*[ابپتثجچحدذرزژسشصضطظعغفقکگلمنوهی]\s*\d{3}\s*[-]?\s*\d{2}";

// Persian time expressions: clock formats and named-time phrases.
const TIME_RE: &str = concat!(
    r"(?:ساعت\s*)?\d{1,2}[:٫.]\d{2}(?:[:٫.]\d{2})?\s*(?:صبح|ظهر|بعد\s*از\s*ظهر|عصر|شب|بامداد)?",
    r"|(?:ساعت\s*)?\d{1,2}\s*(?:صبح|ظهر|بعد\s*از\s*ظهر|عصر|شب|بامداد)",
    r"|نیمه\s*شب|نیمروز|نصف\s*روز|ظهر|بامداد"
);

// ISBN: 10 or 13 digits with optional dashes/spaces and possible 'X' check.
const ISBN_RE: &str = r"\b(?:97[89][\s\-]?\d{1,5}[\s\-]?\d{1,7}[\s\-]?\d{1,7}[\s\-]?\d|\d{1,5}[\s\-]?\d{1,7}[\s\-]?\d{1,7}[\s\-]?[\dXx])\b";

// IPv4 address.
const IPV4_RE: &str = r"\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b";

// Mention: @handle (Latin + Persian letters/digits/underscore)
const MENTION_RE: &str = r"@[\p{L}\p{N}_]{1,50}";

// Hashtag: #topic
const HASHTAG_RE: &str = r"#[\p{L}\p{N}_]{1,100}";

// URL: http/https only (conservative; avoids false positives)
const URL_RE: &str = r#"https?://[^\s<>"]*[^\s<>".,;!?؟،]"#;

// Email: user@domain.tld
const EMAIL_RE: &str = r"[\w.+\-]+@[\w\-]+(?:\.[\w\-]+)+";

// ── recogniser ────────────────────────────────────────────────────────────────

/// Compiled entity-recognition patterns.
///
/// The recogniser is built once (inside [`Parsitext::new`](crate::Parsitext::new)) and is reusable
/// across many [`detect`](EntityRecognizer::detect) calls.  It is both `Send`
/// and `Sync`.
pub struct EntityRecognizer {
    phone: Regex,
    date_num: Regex,
    date_text: Regex,
    money: Regex,
    national_id: Regex,
    bank_card: Regex,
    iban: Regex,
    postal_code: Regex,
    car_plate: Regex,
    time_expr: Regex,
    isbn: Regex,
    ipv4: Regex,
    mention: Regex,
    hashtag: Regex,
    url: Regex,
    email: Regex,
}

impl EntityRecognizer {
    /// Compile all nine patterns.
    ///
    /// This is called once by [`Parsitext::new`](crate::Parsitext::new).  It panics only if a static
    /// pattern string is invalid, which cannot happen in production builds.
    pub fn new() -> Self {
        Self {
            phone: Regex::new(PHONE_RE).expect("phone regex"),
            date_num: Regex::new(DATE_NUM_RE).expect("date_num regex"),
            date_text: Regex::new(DATE_TEXT_RE).expect("date_text regex"),
            money: Regex::new(MONEY_RE).expect("money regex"),
            national_id: Regex::new(NATIONAL_ID_RE).expect("national_id regex"),
            bank_card: Regex::new(BANK_CARD_RE).expect("bank_card regex"),
            iban: Regex::new(IBAN_RE).expect("iban regex"),
            postal_code: Regex::new(POSTAL_CODE_RE).expect("postal_code regex"),
            car_plate: Regex::new(CAR_PLATE_RE).expect("car_plate regex"),
            time_expr: Regex::new(TIME_RE).expect("time_expr regex"),
            isbn: Regex::new(ISBN_RE).expect("isbn regex"),
            ipv4: Regex::new(IPV4_RE).expect("ipv4 regex"),
            mention: Regex::new(MENTION_RE).expect("mention regex"),
            hashtag: Regex::new(HASHTAG_RE).expect("hashtag regex"),
            url: Regex::new(URL_RE).expect("url regex"),
            email: Regex::new(EMAIL_RE).expect("email regex"),
        }
    }

    /// Detect all entities in `text`, sorted by start position with
    /// overlapping spans resolved (earliest start wins; ties prefer longest).
    ///
    /// `NationalId`, `BankCard`, `Iban`, and `PostalCode` matches are
    /// **post-validated** against their respective checksums; matches that
    /// fail validation are silently dropped to reduce false positives.
    pub fn detect(&self, text: &str) -> Vec<Entity> {
        let mut entities: Vec<Entity> = Vec::new();

        // Collection order determines priority when patterns overlap.
        self.collect(text, &self.url, EntityKind::Url, &mut entities);
        self.collect(text, &self.email, EntityKind::Email, &mut entities);
        self.collect(text, &self.iban, EntityKind::Iban, &mut entities);
        self.collect(text, &self.bank_card, EntityKind::BankCard, &mut entities);
        self.collect(text, &self.phone, EntityKind::PhoneNumber, &mut entities);
        self.collect(
            text,
            &self.date_text,
            EntityKind::JalaliDateTextual,
            &mut entities,
        );
        self.collect(
            text,
            &self.date_num,
            EntityKind::JalaliDateNumeric,
            &mut entities,
        );
        self.collect(text, &self.money, EntityKind::MoneyAmount, &mut entities);
        self.collect(
            text,
            &self.national_id,
            EntityKind::NationalId,
            &mut entities,
        );
        self.collect(
            text,
            &self.postal_code,
            EntityKind::PostalCode,
            &mut entities,
        );
        self.collect(text, &self.car_plate, EntityKind::CarPlate, &mut entities);
        self.collect(
            text,
            &self.time_expr,
            EntityKind::TimeExpression,
            &mut entities,
        );
        self.collect(text, &self.ipv4, EntityKind::Ipv4, &mut entities);
        self.collect(text, &self.isbn, EntityKind::Isbn, &mut entities);
        self.collect(text, &self.mention, EntityKind::Mention, &mut entities);
        self.collect(text, &self.hashtag, EntityKind::Hashtag, &mut entities);

        // Drop entities whose validators say no.
        entities.retain(|e| validate_entity(&e.kind, &e.text));

        remove_overlaps(entities)
    }

    fn collect(&self, text: &str, re: &Regex, kind: EntityKind, out: &mut Vec<Entity>) {
        for m in re.find_iter(text) {
            let matched = m.as_str().to_owned();
            let normalized = normalize_entity(&kind, &matched);
            out.push(Entity {
                kind: kind.clone(),
                text: matched,
                normalized,
                span: Span {
                    start: m.start(),
                    end: m.end(),
                },
            });
        }
    }
}

impl Default for EntityRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Sort by start position; drop any span that overlaps the previously accepted
/// span (keep earliest start, then longest).
fn remove_overlaps(mut entities: Vec<Entity>) -> Vec<Entity> {
    entities.sort_by(|a, b| {
        a.span
            .start
            .cmp(&b.span.start)
            .then(b.span.end.cmp(&a.span.end))
    });

    let mut result: Vec<Entity> = Vec::with_capacity(entities.len());
    let mut last_end = 0usize;

    for e in entities {
        if e.span.start >= last_end {
            last_end = e.span.end;
            result.push(e);
        }
    }

    result
}

/// Returns `false` when an entity match should be dropped because its
/// checksum does not validate.
fn validate_entity(kind: &EntityKind, raw: &str) -> bool {
    match kind {
        EntityKind::NationalId => crate::validators::national_id::validate(raw),
        EntityKind::BankCard => crate::validators::bank_card::validate(raw),
        EntityKind::Iban => crate::validators::sheba::validate(raw),
        EntityKind::PostalCode => crate::validators::postal_code::validate(raw),
        EntityKind::CarPlate => crate::validators::car_plate::validate(raw),
        EntityKind::Isbn => validate_isbn(raw),
        _ => true,
    }
}

/// ISBN-10 and ISBN-13 checksum validation.
fn validate_isbn(raw: &str) -> bool {
    let chars: Vec<char> = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == 'X' || *c == 'x')
        .collect();
    match chars.len() {
        10 => {
            let mut sum = 0i32;
            for (i, c) in chars.iter().enumerate() {
                let d = if (*c == 'X' || *c == 'x') && i == 9 {
                    10
                } else if let Some(n) = c.to_digit(10) {
                    n as i32
                } else {
                    return false;
                };
                sum += d * (10 - i as i32);
            }
            sum % 11 == 0
        }
        13 => {
            let mut sum = 0i32;
            for (i, c) in chars.iter().enumerate() {
                let d = c.to_digit(10).map(|n| n as i32).unwrap_or(-1);
                if d < 0 {
                    return false;
                }
                sum += d * if i % 2 == 0 { 1 } else { 3 };
            }
            sum % 10 == 0
        }
        _ => false,
    }
}

/// Best-effort structured normalisation for each entity kind.
fn normalize_entity(kind: &EntityKind, raw: &str) -> Option<String> {
    match kind {
        EntityKind::PhoneNumber => crate::validators::phone::canonicalize(raw),
        EntityKind::MoneyAmount => {
            crate::money::parse(raw).map(|m| format!("{} {}", m.value, m.unit))
        }
        EntityKind::BankCard => crate::validators::bank_card::bank(raw).map(|name| name.to_owned()),
        EntityKind::Iban => crate::validators::sheba::bank(raw).map(|n| n.to_owned()),
        _ => None,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn er() -> EntityRecognizer {
        EntityRecognizer::new()
    }

    #[test]
    fn detects_phone_mobile() {
        let entities = er().detect("شماره من 09121234567 هست");
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].kind, EntityKind::PhoneNumber);
        assert_eq!(entities[0].normalized.as_deref(), Some("09121234567"));
    }

    #[test]
    fn detects_phone_with_plus98() {
        let entities = er().detect("+989121234567");
        assert!(entities.iter().any(|e| e.kind == EntityKind::PhoneNumber));
    }

    #[test]
    fn detects_jalali_numeric() {
        let entities = er().detect("تاریخ: 1402/03/15");
        assert!(entities
            .iter()
            .any(|e| e.kind == EntityKind::JalaliDateNumeric));
    }

    #[test]
    fn detects_jalali_textual() {
        let entities = er().detect("۱۵ تیر ۱۴۰۲");
        assert!(entities
            .iter()
            .any(|e| e.kind == EntityKind::JalaliDateTextual));
    }

    #[test]
    fn detects_money_toman() {
        let entities = er().detect("قیمت ۵۰۰ هزار تومان");
        assert!(entities.iter().any(|e| e.kind == EntityKind::MoneyAmount));
    }

    #[test]
    fn detects_money_rial() {
        let entities = er().detect("هزینه: 200 ریال");
        assert!(entities.iter().any(|e| e.kind == EntityKind::MoneyAmount));
    }

    #[test]
    fn detects_mention() {
        let entities = er().detect("پیام @reza123 رو خوندم");
        assert!(entities.iter().any(|e| e.kind == EntityKind::Mention));
    }

    #[test]
    fn detects_hashtag_persian() {
        let entities = er().detect("#فارسی_آزمایش");
        assert!(entities.iter().any(|e| e.kind == EntityKind::Hashtag));
    }

    #[test]
    fn detects_url() {
        let entities = er().detect("به https://example.ir مراجعه کنید");
        assert!(entities.iter().any(|e| e.kind == EntityKind::Url));
    }

    #[test]
    fn detects_email() {
        let entities = er().detect("ایمیل: reza@example.com");
        assert!(entities.iter().any(|e| e.kind == EntityKind::Email));
    }

    #[test]
    fn no_overlap_phone_vs_national_id() {
        let entities = er().detect("09121234567");
        let kinds: Vec<_> = entities.iter().map(|e| &e.kind).collect();
        assert!(
            !kinds.contains(&&EntityKind::NationalId) || !kinds.contains(&&EntityKind::PhoneNumber)
        );
    }

    #[test]
    fn span_len_matches_text() {
        let entities = er().detect("@hello");
        assert!(!entities.is_empty());
        let e = &entities[0];
        assert_eq!(e.span.len(), e.text.len());
    }

    #[test]
    fn display_entity() {
        let entities = er().detect("@reza123");
        assert_eq!(entities[0].to_string(), "[Mention: \"@reza123\"]");
    }

    #[test]
    fn span_is_empty_false() {
        let s = Span { start: 0, end: 4 };
        assert!(!s.is_empty());
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn span_is_empty_true() {
        let s = Span { start: 3, end: 3 };
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }
}
