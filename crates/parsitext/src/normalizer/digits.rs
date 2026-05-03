//! Digit-script unification.
//!
//! Three digit systems appear in Persian texts:
//! - **Latin**: U+0030–U+0039 (0–9)
//! - **Arabic-Indic**: U+0660–U+0669 (٠–٩)
//! - **Persian/Extended Arabic**: U+06F0–U+06F9 (۰–۹)

const PERSIAN_ZERO: u32 = 0x06F0;
const ARABIC_ZERO: u32 = 0x0660;
const LATIN_ZERO: u32 = '0' as u32;

/// Converts Latin (0–9) and Arabic-Indic (٠–٩) digits to Persian (۰–۹).
#[inline]
pub fn to_persian(text: &str) -> String {
    text.chars()
        .map(|c| {
            let cp = c as u32;
            if (LATIN_ZERO..=LATIN_ZERO + 9).contains(&cp) {
                char::from_u32(cp - LATIN_ZERO + PERSIAN_ZERO).unwrap_or(c)
            } else if (ARABIC_ZERO..=ARABIC_ZERO + 9).contains(&cp) {
                char::from_u32(cp - ARABIC_ZERO + PERSIAN_ZERO).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

/// Converts Persian (۰–۹) and Arabic-Indic (٠–٩) digits to Latin (0–9).
#[inline]
pub fn to_latin(text: &str) -> String {
    text.chars()
        .map(|c| {
            let cp = c as u32;
            if (PERSIAN_ZERO..=PERSIAN_ZERO + 9).contains(&cp) {
                char::from_u32(cp - PERSIAN_ZERO + LATIN_ZERO).unwrap_or(c)
            } else if (ARABIC_ZERO..=ARABIC_ZERO + 9).contains(&cp) {
                char::from_u32(cp - ARABIC_ZERO + LATIN_ZERO).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latin_to_persian() {
        assert_eq!(to_persian("1402/03/15"), "۱۴۰۲/۰۳/۱۵");
    }

    #[test]
    fn arabic_indic_to_persian() {
        assert_eq!(to_persian("١٢٣٤"), "۱۲۳۴");
    }

    #[test]
    fn persian_to_latin() {
        assert_eq!(to_latin("۱۴۰۲"), "1402");
    }

    #[test]
    fn mixed_to_latin() {
        assert_eq!(to_latin("۱٢3"), "123");
    }
}
