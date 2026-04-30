//! `serde` impls (enable the `serde` feature).
//!
//! `JalaliDate` serializes as `"YYYY/MM/DD"`, `JalaliDateTime` as
//! `"YYYY/MM/DD HH:MM:SS"`. Both round-trip through `Display` / `FromStr`.

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

use crate::{JalaliDate, JalaliDateTime, Season, Weekday};

impl Serialize for JalaliDate {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for JalaliDate {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = JalaliDate;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a Jalali date string like \"1403/01/01\"")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<JalaliDate, E> {
                v.parse().map_err(E::custom)
            }
        }
        de.deserialize_str(V)
    }
}

impl Serialize for JalaliDateTime {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for JalaliDateTime {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = JalaliDateTime;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a Jalali datetime string like \"1403/01/01 12:34:56\"")
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<JalaliDateTime, E> {
                JalaliDateTime::parse_format(v, "%Y/%m/%d %T").map_err(E::custom)
            }
        }
        de.deserialize_str(V)
    }
}

impl Serialize for Weekday {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.english_name())
    }
}

impl Serialize for Season {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.english_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn date_serializes_as_string() {
        let d = JalaliDate::new(1403, 1, 1).unwrap();
        let v = serde_json::to_value(d).unwrap();
        assert_eq!(v, json!("1403/01/01"));
        let back: JalaliDate = serde_json::from_value(v).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn datetime_round_trip() {
        let dt = JalaliDateTime::new(1403, 1, 1, 12, 34, 56).unwrap();
        let s = serde_json::to_string(&dt).unwrap();
        let back: JalaliDateTime = serde_json::from_str(&s).unwrap();
        assert_eq!(back, dt);
    }

    #[test]
    fn invalid_string_errors() {
        assert!(serde_json::from_str::<JalaliDate>("\"not a date\"").is_err());
    }

    #[test]
    fn weekday_serializes_as_english() {
        let d = JalaliDate::new(1403, 1, 1).unwrap();
        let s = serde_json::to_string(&d.weekday()).unwrap();
        assert_eq!(s, "\"Wednesday\"");
    }
}
