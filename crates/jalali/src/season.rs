//! Persian seasons.

/// The four seasons of the Jalali year.
///
/// Each season spans three months: spring = months 1-3 (Farvardin, Ordibehesht,
/// Khordad), summer = 4-6, autumn = 7-9, winter = 10-12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Persian (Farsi) name.
    pub fn persian_name(self) -> &'static str {
        match self {
            Season::Spring => "بهار",
            Season::Summer => "تابستان",
            Season::Autumn => "پاییز",
            Season::Winter => "زمستان",
        }
    }

    /// Romanized name.
    pub fn english_name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    /// Inclusive month range that this season spans.
    pub fn months(self) -> (u32, u32) {
        match self {
            Season::Spring => (1, 3),
            Season::Summer => (4, 6),
            Season::Autumn => (7, 9),
            Season::Winter => (10, 12),
        }
    }

    /// Season containing the given Jalali month.
    pub fn from_month(month: u32) -> Option<Self> {
        match month {
            1..=3 => Some(Season::Spring),
            4..=6 => Some(Season::Summer),
            7..=9 => Some(Season::Autumn),
            10..=12 => Some(Season::Winter),
            _ => None,
        }
    }
}
