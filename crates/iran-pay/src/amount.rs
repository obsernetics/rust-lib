//! Iranian currency [`Amount`].
//!
//! Iranian retail prices are quoted in **Tomans** but the official currency
//! and every payment gateway's API expects **Rials** (1 Toman = 10 Rials).
//! Mixing the two by accident is the single most common bug in Iranian
//! payment integrations.
//!
//! [`Amount`] forces you to be explicit at construction time and stores
//! everything internally in Rials, so the API surface to gateways is
//! always correct.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The two Iranian currency units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    /// تومان — what merchants quote prices in.  1 Toman = 10 Rials.
    Toman,
    /// ریال — the official unit and what every gateway API expects.
    Rial,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Currency::Toman => "Toman",
            Currency::Rial => "Rial",
        })
    }
}

/// A monetary amount, stored internally in Rials.
///
/// Construct with [`Amount::toman`] or [`Amount::rial`].  The gateway drivers
/// use [`Amount::as_rials`] to send the request body, so accidental
/// unit mix-ups are impossible.
///
/// ```
/// use iran_pay::Amount;
///
/// let price = Amount::toman(50_000);
/// assert_eq!(price.as_rials(),  500_000);
/// assert_eq!(price.as_tomans(), 50_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount {
    rials: i64,
}

impl Amount {
    /// Construct from a Toman amount (multiplied by 10 internally).
    #[must_use]
    pub const fn toman(value: i64) -> Self {
        Self {
            rials: value.saturating_mul(10),
        }
    }

    /// Construct from a Rial amount.
    #[must_use]
    pub const fn rial(value: i64) -> Self {
        Self { rials: value }
    }

    /// Construct from a value paired with a [`Currency`].
    #[must_use]
    pub const fn new(value: i64, currency: Currency) -> Self {
        match currency {
            Currency::Toman => Self::toman(value),
            Currency::Rial => Self::rial(value),
        }
    }

    /// The amount expressed in Rials (always exact).
    #[must_use]
    pub const fn as_rials(&self) -> i64 {
        self.rials
    }

    /// The amount expressed in Tomans (truncated toward zero if not divisible
    /// by 10 — but Iranian gateways always operate on multiples of 10 Rials,
    /// so in practice this is exact).
    #[must_use]
    pub const fn as_tomans(&self) -> i64 {
        self.rials / 10
    }

    /// Returns `true` if the amount is exactly zero Rials.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.rials == 0
    }
}

impl fmt::Display for Amount {
    /// `123,000 Rial` (with thousand separators).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        let abs = self.rials.unsigned_abs().to_string();
        let bytes = abs.as_bytes();
        if self.rials < 0 {
            s.push('-');
        }
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 && (bytes.len() - i).is_multiple_of(3) {
                s.push(',');
            }
            s.push(*b as char);
        }
        write!(f, "{s} Rial")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_conversion() {
        assert_eq!(Amount::toman(50_000).as_rials(), 500_000);
        assert_eq!(Amount::rial(500_000).as_tomans(), 50_000);
    }

    #[test]
    fn comparison() {
        assert!(Amount::toman(100) < Amount::toman(200));
        assert_eq!(Amount::toman(100), Amount::rial(1_000));
    }

    #[test]
    fn display_with_separators() {
        assert_eq!(Amount::rial(1_234_567).to_string(), "1,234,567 Rial");
        assert_eq!(Amount::rial(0).to_string(), "0 Rial");
    }

    #[test]
    fn negative_amount_displays() {
        assert_eq!(Amount::rial(-500).to_string(), "-500 Rial");
    }

    #[test]
    fn const_constructors() {
        const FEE: Amount = Amount::toman(1_000);
        assert_eq!(FEE.as_rials(), 10_000);
    }
}
