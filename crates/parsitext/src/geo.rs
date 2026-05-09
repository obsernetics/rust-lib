//! Iranian geographic data: provinces, capitals, and a curated set of major
//! cities for province lookup.
//!
//! Inspired by the `sarzamin` crate, but kept deliberately compact — we
//! embed only provinces and a few hundred major cities so the binary impact
//! is small.  For full census-level division data (counties, districts,
//! rural districts) use `sarzamin` directly.
//!
//! All data is `static`, zero-allocation, and lookup-friendly.
//!
//! ```
//! use parsitext::geo;
//!
//! let p = geo::find_province_by_id(1).unwrap();
//! assert_eq!(p.name_en, "Tehran");
//! assert_eq!(p.capital_fa, "تهران");
//!
//! // Lookup by Persian or English city name (case-insensitive).
//! let p = geo::find_province_by_city("اصفهان").unwrap();
//! assert_eq!(p.name_en, "Isfahan");
//! ```

/// A single Iranian province with its capital and telephone area code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Province {
    /// 1-based stable identifier (1..=31), matches the order in [`PROVINCES`].
    pub id: u32,
    /// URL-safe English slug, e.g. `"east-azerbaijan"`.
    pub slug: &'static str,
    /// English name, e.g. `"Tehran"`.
    pub name_en: &'static str,
    /// Persian name, e.g. `"تهران"`.
    pub name_fa: &'static str,
    /// English capital city, e.g. `"Tehran"`.
    pub capital_en: &'static str,
    /// Persian capital city, e.g. `"تهران"`.
    pub capital_fa: &'static str,
    /// Provincial telephone area code, e.g. `"21"` for Tehran.
    pub tel_prefix: &'static str,
}

/// All 31 Iranian provinces, in stable id order.
pub const PROVINCES: &[Province] = &[
    Province {
        id: 1,
        slug: "tehran",
        name_en: "Tehran",
        name_fa: "تهران",
        capital_en: "Tehran",
        capital_fa: "تهران",
        tel_prefix: "21",
    },
    Province {
        id: 2,
        slug: "qom",
        name_en: "Qom",
        name_fa: "قم",
        capital_en: "Qom",
        capital_fa: "قم",
        tel_prefix: "25",
    },
    Province {
        id: 3,
        slug: "markazi",
        name_en: "Markazi",
        name_fa: "مرکزی",
        capital_en: "Arak",
        capital_fa: "اراک",
        tel_prefix: "86",
    },
    Province {
        id: 4,
        slug: "qazvin",
        name_en: "Qazvin",
        name_fa: "قزوین",
        capital_en: "Qazvin",
        capital_fa: "قزوین",
        tel_prefix: "28",
    },
    Province {
        id: 5,
        slug: "gilan",
        name_en: "Gilan",
        name_fa: "گیلان",
        capital_en: "Rasht",
        capital_fa: "رشت",
        tel_prefix: "13",
    },
    Province {
        id: 6,
        slug: "ardabil",
        name_en: "Ardabil",
        name_fa: "اردبیل",
        capital_en: "Ardabil",
        capital_fa: "اردبیل",
        tel_prefix: "45",
    },
    Province {
        id: 7,
        slug: "zanjan",
        name_en: "Zanjan",
        name_fa: "زنجان",
        capital_en: "Zanjan",
        capital_fa: "زنجان",
        tel_prefix: "24",
    },
    Province {
        id: 8,
        slug: "east-azerbaijan",
        name_en: "East Azerbaijan",
        name_fa: "آذربایجان شرقی",
        capital_en: "Tabriz",
        capital_fa: "تبریز",
        tel_prefix: "41",
    },
    Province {
        id: 9,
        slug: "west-azerbaijan",
        name_en: "West Azerbaijan",
        name_fa: "آذربایجان غربی",
        capital_en: "Urmia",
        capital_fa: "ارومیه",
        tel_prefix: "44",
    },
    Province {
        id: 10,
        slug: "kurdistan",
        name_en: "Kurdistan",
        name_fa: "کردستان",
        capital_en: "Sanandaj",
        capital_fa: "سنندج",
        tel_prefix: "87",
    },
    Province {
        id: 11,
        slug: "hamadan",
        name_en: "Hamadan",
        name_fa: "همدان",
        capital_en: "Hamadan",
        capital_fa: "همدان",
        tel_prefix: "81",
    },
    Province {
        id: 12,
        slug: "kermanshah",
        name_en: "Kermanshah",
        name_fa: "کرمانشاه",
        capital_en: "Kermanshah",
        capital_fa: "کرمانشاه",
        tel_prefix: "83",
    },
    Province {
        id: 13,
        slug: "ilam",
        name_en: "Ilam",
        name_fa: "ایلام",
        capital_en: "Ilam",
        capital_fa: "ایلام",
        tel_prefix: "84",
    },
    Province {
        id: 14,
        slug: "lorestan",
        name_en: "Lorestan",
        name_fa: "لرستان",
        capital_en: "Khorramabad",
        capital_fa: "خرم‌آباد",
        tel_prefix: "66",
    },
    Province {
        id: 15,
        slug: "khuzestan",
        name_en: "Khuzestan",
        name_fa: "خوزستان",
        capital_en: "Ahvaz",
        capital_fa: "اهواز",
        tel_prefix: "61",
    },
    Province {
        id: 16,
        slug: "chaharmahal",
        name_en: "Chaharmahal & Bakhtiari",
        name_fa: "چهارمحال و بختیاری",
        capital_en: "Shahrekord",
        capital_fa: "شهرکرد",
        tel_prefix: "38",
    },
    Province {
        id: 17,
        slug: "kohgiluyeh",
        name_en: "Kohgiluyeh & Boyer-Ahmad",
        name_fa: "کهگیلویه و بویراحمد",
        capital_en: "Yasuj",
        capital_fa: "یاسوج",
        tel_prefix: "74",
    },
    Province {
        id: 18,
        slug: "bushehr",
        name_en: "Bushehr",
        name_fa: "بوشهر",
        capital_en: "Bushehr",
        capital_fa: "بوشهر",
        tel_prefix: "77",
    },
    Province {
        id: 19,
        slug: "fars",
        name_en: "Fars",
        name_fa: "فارس",
        capital_en: "Shiraz",
        capital_fa: "شیراز",
        tel_prefix: "71",
    },
    Province {
        id: 20,
        slug: "hormozgan",
        name_en: "Hormozgan",
        name_fa: "هرمزگان",
        capital_en: "Bandar Abbas",
        capital_fa: "بندرعباس",
        tel_prefix: "76",
    },
    Province {
        id: 21,
        slug: "sistan-baluchestan",
        name_en: "Sistan & Baluchestan",
        name_fa: "سیستان و بلوچستان",
        capital_en: "Zahedan",
        capital_fa: "زاهدان",
        tel_prefix: "54",
    },
    Province {
        id: 22,
        slug: "kerman",
        name_en: "Kerman",
        name_fa: "کرمان",
        capital_en: "Kerman",
        capital_fa: "کرمان",
        tel_prefix: "34",
    },
    Province {
        id: 23,
        slug: "yazd",
        name_en: "Yazd",
        name_fa: "یزد",
        capital_en: "Yazd",
        capital_fa: "یزد",
        tel_prefix: "35",
    },
    Province {
        id: 24,
        slug: "isfahan",
        name_en: "Isfahan",
        name_fa: "اصفهان",
        capital_en: "Isfahan",
        capital_fa: "اصفهان",
        tel_prefix: "31",
    },
    Province {
        id: 25,
        slug: "semnan",
        name_en: "Semnan",
        name_fa: "سمنان",
        capital_en: "Semnan",
        capital_fa: "سمنان",
        tel_prefix: "23",
    },
    Province {
        id: 26,
        slug: "mazandaran",
        name_en: "Mazandaran",
        name_fa: "مازندران",
        capital_en: "Sari",
        capital_fa: "ساری",
        tel_prefix: "11",
    },
    Province {
        id: 27,
        slug: "golestan",
        name_en: "Golestan",
        name_fa: "گلستان",
        capital_en: "Gorgan",
        capital_fa: "گرگان",
        tel_prefix: "17",
    },
    Province {
        id: 28,
        slug: "north-khorasan",
        name_en: "North Khorasan",
        name_fa: "خراسان شمالی",
        capital_en: "Bojnurd",
        capital_fa: "بجنورد",
        tel_prefix: "58",
    },
    Province {
        id: 29,
        slug: "razavi-khorasan",
        name_en: "Razavi Khorasan",
        name_fa: "خراسان رضوی",
        capital_en: "Mashhad",
        capital_fa: "مشهد",
        tel_prefix: "51",
    },
    Province {
        id: 30,
        slug: "south-khorasan",
        name_en: "South Khorasan",
        name_fa: "خراسان جنوبی",
        capital_en: "Birjand",
        capital_fa: "بیرجند",
        tel_prefix: "56",
    },
    Province {
        id: 31,
        slug: "alborz",
        name_en: "Alborz",
        name_fa: "البرز",
        capital_en: "Karaj",
        capital_fa: "کرج",
        tel_prefix: "26",
    },
];

/// Curated major cities for province lookup.  Each entry is
/// `(en, fa, province_id)`.  Provincial capitals are present here too so
/// `find_province_by_city("Tehran")` and friends always succeed.
const CITIES: &[(&str, &str, u32)] = &[
    // Tehran province
    ("Tehran", "تهران", 1),
    ("Rey", "ری", 1),
    ("Shemiranat", "شمیرانات", 1),
    ("Eslamshahr", "اسلامشهر", 1),
    ("Pakdasht", "پاکدشت", 1),
    ("Damavand", "دماوند", 1),
    // Qom
    ("Qom", "قم", 2),
    // Markazi
    ("Arak", "اراک", 3),
    ("Saveh", "ساوه", 3),
    ("Khomein", "خمین", 3),
    ("Mahalat", "محلات", 3),
    // Qazvin
    ("Qazvin", "قزوین", 4),
    ("Takestan", "تاکستان", 4),
    // Gilan
    ("Rasht", "رشت", 5),
    ("Anzali", "بندر انزلی", 5),
    ("Lahijan", "لاهیجان", 5),
    ("Astara", "آستارا", 5),
    ("Langarud", "لنگرود", 5),
    // Ardabil
    ("Ardabil", "اردبیل", 6),
    ("Meshgin Shahr", "مشگین شهر", 6),
    ("Parsabad", "پارس‌آباد", 6),
    // Zanjan
    ("Zanjan", "زنجان", 7),
    ("Abhar", "ابهر", 7),
    // East Azerbaijan
    ("Tabriz", "تبریز", 8),
    ("Maragheh", "مراغه", 8),
    ("Marand", "مرند", 8),
    ("Ahar", "اهر", 8),
    ("Bonab", "بناب", 8),
    // West Azerbaijan
    ("Urmia", "ارومیه", 9),
    ("Khoy", "خوی", 9),
    ("Mahabad", "مهاباد", 9),
    ("Miandoab", "میاندوآب", 9),
    ("Salmas", "سلماس", 9),
    // Kurdistan
    ("Sanandaj", "سنندج", 10),
    ("Saqqez", "سقز", 10),
    ("Marivan", "مریوان", 10),
    ("Bijar", "بیجار", 10),
    // Hamadan
    ("Hamadan", "همدان", 11),
    ("Malayer", "ملایر", 11),
    ("Nahavand", "نهاوند", 11),
    ("Tuyserkan", "تویسرکان", 11),
    // Kermanshah
    ("Kermanshah", "کرمانشاه", 12),
    ("Eslamabad-e Gharb", "اسلام‌آباد غرب", 12),
    ("Sonqor", "سنقر", 12),
    // Ilam
    ("Ilam", "ایلام", 13),
    ("Dehloran", "دهلران", 13),
    // Lorestan
    ("Khorramabad", "خرم‌آباد", 14),
    ("Borujerd", "بروجرد", 14),
    ("Aligudarz", "الیگودرز", 14),
    ("Dorud", "دورود", 14),
    // Khuzestan
    ("Ahvaz", "اهواز", 15),
    ("Abadan", "آبادان", 15),
    ("Khorramshahr", "خرمشهر", 15),
    ("Dezful", "دزفول", 15),
    ("Mahshahr", "ماهشهر", 15),
    ("Shushtar", "شوشتر", 15),
    ("Behbahan", "بهبهان", 15),
    // Chaharmahal & Bakhtiari
    ("Shahrekord", "شهرکرد", 16),
    ("Borujen", "بروجن", 16),
    // Kohgiluyeh & Boyer-Ahmad
    ("Yasuj", "یاسوج", 17),
    ("Dogonbadan", "دوگنبدان", 17),
    // Bushehr
    ("Bushehr", "بوشهر", 18),
    ("Borazjan", "برازجان", 18),
    ("Genaveh", "گناوه", 18),
    ("Asaluyeh", "عسلویه", 18),
    // Fars
    ("Shiraz", "شیراز", 19),
    ("Marvdasht", "مرودشت", 19),
    ("Jahrom", "جهرم", 19),
    ("Kazerun", "کازرون", 19),
    ("Fasa", "فسا", 19),
    ("Lar", "لار", 19),
    // Hormozgan
    ("Bandar Abbas", "بندرعباس", 20),
    ("Bandar Lengeh", "بندر لنگه", 20),
    ("Minab", "میناب", 20),
    ("Qeshm", "قشم", 20),
    ("Kish", "کیش", 20),
    // Sistan & Baluchestan
    ("Zahedan", "زاهدان", 21),
    ("Zabol", "زابل", 21),
    ("Iranshahr", "ایرانشهر", 21),
    ("Chabahar", "چابهار", 21),
    // Kerman
    ("Kerman", "کرمان", 22),
    ("Rafsanjan", "رفسنجان", 22),
    ("Bam", "بم", 22),
    ("Sirjan", "سیرجان", 22),
    ("Jiroft", "جیرفت", 22),
    ("Zarand", "زرند", 22),
    // Yazd
    ("Yazd", "یزد", 23),
    ("Meybod", "میبد", 23),
    ("Ardakan", "اردکان", 23),
    ("Bafq", "بافق", 23),
    // Isfahan
    ("Isfahan", "اصفهان", 24),
    ("Kashan", "کاشان", 24),
    ("Najafabad", "نجف‌آباد", 24),
    ("Khomeyni Shahr", "خمینی‌شهر", 24),
    ("Shahin Shahr", "شاهین‌شهر", 24),
    ("Falavarjan", "فلاورجان", 24),
    ("Mobarakeh", "مبارکه", 24),
    ("Natanz", "نطنز", 24),
    // Semnan
    ("Semnan", "سمنان", 25),
    ("Shahrud", "شاهرود", 25),
    ("Damghan", "دامغان", 25),
    ("Garmsar", "گرمسار", 25),
    // Mazandaran
    ("Sari", "ساری", 26),
    ("Babol", "بابل", 26),
    ("Amol", "آمل", 26),
    ("Qaem Shahr", "قائم‌شهر", 26),
    ("Behshahr", "بهشهر", 26),
    ("Tonekabon", "تنکابن", 26),
    ("Chalus", "چالوس", 26),
    ("Nowshahr", "نوشهر", 26),
    ("Ramsar", "رامسر", 26),
    // Golestan
    ("Gorgan", "گرگان", 27),
    ("Gonbad-e Kavus", "گنبد کاووس", 27),
    ("Aliabad-e Katul", "علی‌آباد کتول", 27),
    ("Bandar Torkaman", "بندر ترکمن", 27),
    // North Khorasan
    ("Bojnurd", "بجنورد", 28),
    ("Esfarayen", "اسفراین", 28),
    ("Shirvan", "شیروان", 28),
    // Razavi Khorasan
    ("Mashhad", "مشهد", 29),
    ("Neyshabur", "نیشابور", 29),
    ("Sabzevar", "سبزوار", 29),
    ("Torbat-e Heydarieh", "تربت حیدریه", 29),
    ("Quchan", "قوچان", 29),
    ("Kashmar", "کاشمر", 29),
    ("Gonabad", "گناباد", 29),
    // South Khorasan
    ("Birjand", "بیرجند", 30),
    ("Qaen", "قائن", 30),
    ("Ferdows", "فردوس", 30),
    // Alborz
    ("Karaj", "کرج", 31),
    ("Nazarabad", "نظرآباد", 31),
    ("Hashtgerd", "هشتگرد", 31),
    ("Eshtehard", "اشتهارد", 31),
];

/// Return all 31 provinces.
#[must_use]
pub fn get_all_provinces() -> &'static [Province] {
    PROVINCES
}

/// Find a province by stable 1-based id.
#[must_use]
pub fn find_province_by_id(id: u32) -> Option<&'static Province> {
    PROVINCES.iter().find(|p| p.id == id)
}

/// Find a province by URL-slug (e.g. `"razavi-khorasan"`).  Case-insensitive.
#[must_use]
pub fn find_province_by_slug(slug: &str) -> Option<&'static Province> {
    let s = slug.to_ascii_lowercase();
    PROVINCES.iter().find(|p| p.slug.eq_ignore_ascii_case(&s))
}

/// Find a province by **any** name match: English name, Persian name,
/// English capital, Persian capital, or any city in the embedded city list.
///
/// Comparisons ignore ASCII case for English input.
///
/// ```
/// use parsitext::geo::find_province_by_city;
///
/// // Capital
/// assert_eq!(find_province_by_city("Mashhad").unwrap().slug, "razavi-khorasan");
/// // Persian city name
/// assert_eq!(find_province_by_city("اصفهان").unwrap().slug, "isfahan");
/// // Non-capital city
/// assert_eq!(find_province_by_city("Borujerd").unwrap().slug, "lorestan");
/// // Persian non-capital
/// assert_eq!(find_province_by_city("کاشان").unwrap().slug, "isfahan");
/// // No match
/// assert!(find_province_by_city("Atlantis").is_none());
/// ```
#[must_use]
pub fn find_province_by_city(name: &str) -> Option<&'static Province> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Direct city match first.
    for &(en, fa, pid) in CITIES {
        if en.eq_ignore_ascii_case(trimmed) || fa == trimmed {
            return find_province_by_id(pid);
        }
    }
    // Fallback: match province / capital names directly.
    PROVINCES.iter().find(|p| {
        p.name_en.eq_ignore_ascii_case(trimmed)
            || p.capital_en.eq_ignore_ascii_case(trimmed)
            || p.name_fa == trimmed
            || p.capital_fa == trimmed
    })
}

/// Return all known cities for a province (capital first if present).
#[must_use]
pub fn get_cities_of_province(province_id: u32) -> Vec<(&'static str, &'static str)> {
    CITIES
        .iter()
        .filter_map(|(en, fa, pid)| {
            if *pid == province_id {
                Some((*en, *fa))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_provinces_present() {
        assert_eq!(PROVINCES.len(), 31);
    }

    #[test]
    fn ids_are_dense_and_unique() {
        let mut ids: Vec<u32> = PROVINCES.iter().map(|p| p.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids, (1..=31).collect::<Vec<_>>());
    }

    #[test]
    fn slugs_are_unique() {
        let mut slugs: Vec<&str> = PROVINCES.iter().map(|p| p.slug).collect();
        slugs.sort();
        let n = slugs.len();
        slugs.dedup();
        assert_eq!(slugs.len(), n, "duplicate province slug");
    }

    #[test]
    fn find_by_id_works() {
        let p = find_province_by_id(1).unwrap();
        assert_eq!(p.name_en, "Tehran");
    }

    #[test]
    fn find_by_id_unknown() {
        assert!(find_province_by_id(99).is_none());
    }

    #[test]
    fn find_by_slug_case_insensitive() {
        let p = find_province_by_slug("EAST-AZERBAIJAN").unwrap();
        assert_eq!(p.id, 8);
    }

    #[test]
    fn find_by_city_capital() {
        assert_eq!(find_province_by_city("Tehran").unwrap().id, 1);
        assert_eq!(find_province_by_city("تهران").unwrap().id, 1);
    }

    #[test]
    fn find_by_city_non_capital() {
        assert_eq!(find_province_by_city("Borujerd").unwrap().id, 14);
        assert_eq!(find_province_by_city("کاشان").unwrap().slug, "isfahan");
    }

    #[test]
    fn find_by_city_unknown() {
        assert!(find_province_by_city("Atlantis").is_none());
        assert!(find_province_by_city("").is_none());
        assert!(find_province_by_city("   ").is_none());
    }

    #[test]
    fn cities_of_province_includes_capital() {
        let cities = get_cities_of_province(19); // Fars
        assert!(cities.iter().any(|(en, _)| *en == "Shiraz"));
        assert!(cities.iter().any(|(_, fa)| *fa == "کازرون"));
    }

    #[test]
    fn every_province_has_at_least_one_city() {
        for p in PROVINCES {
            let n = get_cities_of_province(p.id).len();
            assert!(n >= 1, "province {} has no cities", p.name_en);
        }
    }

    #[test]
    fn city_province_ids_are_valid() {
        for &(_en, _fa, pid) in CITIES {
            assert!(find_province_by_id(pid).is_some(), "bad province id {pid}");
        }
    }
}
