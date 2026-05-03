//! Static corpus data — slang corrections, profanity word lists.
//!
//! All slang entries are `(informal / goftari, canonical / neveshtar)` pairs.
//! Profanity entries are words replaced with `"***"` by the filter.
//! Companion JSON files in `data/` are the human-readable source of truth;
//! keep them in sync when editing here.

// ── slang ─────────────────────────────────────────────────────────────────────

/// Informal spoken-Persian (*goftari*) forms and their written equivalents.
///
/// Matched whole-word only; partial matches inside longer words are rejected.
pub const SLANG_PAIRS: &[(&str, &str)] = &[
    // ─── خواستن (to want) ──────────────────────────────────────────────────
    ("میخوام", "می‌خواهم"),
    ("میخواد", "می‌خواهد"),
    ("میخوای", "می‌خواهی"),
    ("میخوان", "می‌خواهند"),
    ("نمیخوام", "نمی‌خواهم"),
    ("نمیخواد", "نمی‌خواهد"),
    ("نمیخوای", "نمی‌خواهی"),
    ("نمیخوان", "نمی‌خواهند"),
    // ─── دانستن (to know) ──────────────────────────────────────────────────
    ("میدونم", "می‌دانم"),
    ("میدونی", "می‌دانی"),
    ("میدونه", "می‌داند"),
    ("میدونن", "می‌دانند"),
    ("نمیدونم", "نمی‌دانم"),
    ("نمیدونی", "نمی‌دانی"),
    ("نمیدونه", "نمی‌داند"),
    ("نمیدونن", "نمی‌دانند"),
    // ─── توانستن (to be able) ──────────────────────────────────────────────
    ("میتونم", "می‌توانم"),
    ("میتونی", "می‌توانی"),
    ("میتونه", "می‌تواند"),
    ("میتونن", "می‌توانند"),
    ("نمیتونم", "نمی‌توانم"),
    ("نمیتونی", "نمی‌توانی"),
    ("نمیتونه", "نمی‌تواند"),
    // ─── شدن / گفتن / داشتن ──────────────────────────────────────────────
    ("میشه", "می‌شود"),
    ("نمیشه", "نمی‌شود"),
    ("میگه", "می‌گوید"),
    ("میگن", "می‌گویند"),
    ("داره", "دارد"),
    ("دارن", "دارند"),
    // ─── بودن / هستن ─────────────────────────────────────────────────────
    ("هستن", "هستند"),
    ("نیستن", "نیستند"),
    // ─── رفتن ────────────────────────────────────────────────────────────
    ("بریم", "برویم"),
    ("بیاین", "بیایید"),
    // ─── اشاره / سؤالی ───────────────────────────────────────────────────
    ("اینجاس", "اینجاست"),
    ("اونجاس", "آنجاست"),
    ("چیه", "چیست"),
    ("کیه", "کیست"),
    ("کجاس", "کجاست"),
];

// ── profanity ─────────────────────────────────────────────────────────────────

/// High-confidence Persian insult words filtered by [`ProfanityLevel::Light`].
///
/// This list is intentionally conservative and contains only unambiguous verbal
/// insults.  Words that also carry a neutral meaning in other contexts are
/// excluded.  The filter uses whole-word matching so the strings never match
/// as sub-strings of unrelated words.
pub const PROFANITY_LIGHT: &[&str] = &[
    "احمق",  // idiot
    "کودن",  // stupid
    "ابله",  // fool
    "خنگ",   // dumb
    "نادان", // ignorant
];

/// Broader word list filtered by [`ProfanityLevel::Medium`].
///
/// Includes all [`PROFANITY_LIGHT`] words plus stronger offensive terms
/// commonly found in Iranian social-media content-moderation use cases.
pub const PROFANITY_MEDIUM: &[&str] = &[
    // Light words
    "احمق",
    "کودن",
    "ابله",
    "خنگ",
    "نادان",
    // Stronger insults
    "عوضی",
    "بی‌شعور",
    "کثیف",
    "حرامزاده",
    "بی‌ناموس",
    "جنده",
    "کیر",
    "کس",
    "کون",
    "گوه",
    "خاک‌بر",
];
