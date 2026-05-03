//! Iranian bank databases — Sheba (IBAN) prefix codes and Bank Card BIN
//! (Bank Identification Number) prefixes.
//!
//! Sources are public Central Bank of Iran (CBI) registries.  The lists
//! cover all currently-licensed Iranian banks and credit institutions; less
//! common BINs may be missing.

/// Sheba (IBAN) bank-code lookup.
///
/// Position 5–7 (3 digits) of an Iranian IBAN identifies the bank.
/// Returns `(english_name, persian_name)`.
pub(crate) const SHEBA_CODES: &[(&str, &str, &str)] = &[
    ("010", "Central Bank of Iran", "بانک مرکزی"),
    ("011", "Sanat o Madan Bank", "بانک صنعت و معدن"),
    ("012", "Bank Mellat", "بانک ملت"),
    ("013", "Bank Refah Kargaran", "بانک رفاه کارگران"),
    ("014", "Bank Maskan", "بانک مسکن"),
    ("015", "Bank Sepah", "بانک سپه"),
    ("016", "Bank Keshavarzi", "بانک کشاورزی"),
    ("017", "Bank Melli Iran", "بانک ملی ایران"),
    ("018", "Bank Tejarat", "بانک تجارت"),
    ("019", "Bank Saderat Iran", "بانک صادرات ایران"),
    ("020", "Export Development Bank", "بانک توسعه صادرات"),
    ("021", "Post Bank", "پست بانک ایران"),
    ("022", "Tose'e Ta'avon Bank", "بانک توسعه تعاون"),
    ("051", "Tose'e Credit Institution", "موسسه اعتباری توسعه"),
    ("052", "Ghavamin Bank", "بانک قوامین"),
    ("053", "Bank Karafarin", "بانک کارآفرین"),
    ("054", "Bank Parsian", "بانک پارسیان"),
    ("055", "Bank Eghtesad Novin", "بانک اقتصاد نوین"),
    ("056", "Bank Saman", "بانک سامان"),
    ("057", "Bank Pasargad", "بانک پاسارگاد"),
    ("058", "Bank Sarmayeh", "بانک سرمایه"),
    ("059", "Bank Sina", "بانک سینا"),
    (
        "060",
        "Mehr Iran Credit Institution",
        "موسسه قرض الحسنه مهر ایران",
    ),
    ("061", "Bank Shahr", "بانک شهر"),
    ("062", "Bank Ayandeh", "بانک آینده"),
    ("063", "Bank Ansar", "بانک انصار"),
    ("064", "Bank Gardeshgari", "بانک گردشگری"),
    ("065", "Bank Hekmat Iranian", "بانک حکمت ایرانیان"),
    ("066", "Bank Day", "بانک دی"),
    ("069", "Iran Zamin Bank", "بانک ایران زمین"),
    ("070", "Bank Resalat", "بانک قرض‌الحسنه رسالت"),
    ("073", "Kowsar Credit Institution", "موسسه اعتباری کوثر"),
    ("075", "Mellat Credit Institution", "موسسه اعتباری ملل"),
    ("078", "Khavarmianeh Bank", "بانک خاورمیانه"),
    ("080", "Noor Credit Institution", "موسسه اعتباری نور"),
    (
        "095",
        "Iran Venezuela Bi-National Bank",
        "بانک ایران ونزوئلا",
    ),
];

/// Bank Card BIN (first 6 digits) → bank lookup.
///
/// Returns `(english_name, persian_name)`.  Multiple BINs can map to the
/// same bank when a bank issues several card series.
pub(crate) const CARD_BINS: &[(&str, &str, &str)] = &[
    ("603799", "Bank Melli Iran", "بانک ملی ایران"),
    ("589210", "Bank Sepah", "بانک سپه"),
    ("627648", "Tose'e Saderat Bank", "بانک توسعه صادرات"),
    ("627961", "Sanat o Madan Bank", "بانک صنعت و معدن"),
    ("603770", "Bank Keshavarzi", "بانک کشاورزی"),
    ("628023", "Bank Maskan", "بانک مسکن"),
    ("627353", "Bank Tejarat", "بانک تجارت"),
    ("603769", "Bank Saderat Iran", "بانک صادرات ایران"),
    ("610433", "Bank Mellat", "بانک ملت"),
    ("991975", "Bank Mellat", "بانک ملت"),
    ("589463", "Bank Refah Kargaran", "بانک رفاه کارگران"),
    ("627760", "Post Bank", "پست بانک"),
    ("502908", "Tose'e Ta'avon Bank", "بانک توسعه تعاون"),
    ("627412", "Bank Eghtesad Novin", "بانک اقتصاد نوین"),
    ("622106", "Bank Parsian", "بانک پارسیان"),
    ("639194", "Bank Parsian", "بانک پارسیان"),
    ("621986", "Bank Saman", "بانک سامان"),
    ("502229", "Bank Pasargad", "بانک پاسارگاد"),
    ("639347", "Bank Pasargad", "بانک پاسارگاد"),
    ("639607", "Bank Sarmayeh", "بانک سرمایه"),
    ("639346", "Bank Sina", "بانک سینا"),
    (
        "606373",
        "Mehr Iran Credit Institution",
        "موسسه قرض الحسنه مهر ایران",
    ),
    ("502806", "Bank Shahr", "بانک شهر"),
    ("504706", "Bank Shahr", "بانک شهر"),
    ("636214", "Bank Ayandeh", "بانک آینده"),
    ("627381", "Bank Ansar", "بانک انصار"),
    ("505416", "Bank Gardeshgari", "بانک گردشگری"),
    ("636949", "Hekmat Iranian Bank", "بانک حکمت ایرانیان"),
    ("502938", "Bank Day", "بانک دی"),
    ("505785", "Iran Zamin Bank", "بانک ایران زمین"),
    ("504172", "Bank Resalat", "بانک قرض الحسنه رسالت"),
    ("505801", "Kowsar Credit Institution", "موسسه اعتباری کوثر"),
    ("627488", "Bank Karafarin", "بانک کارآفرین"),
    (
        "502910",
        "Caspian Credit Institution",
        "موسسه اعتباری کاسپین",
    ),
    ("585983", "Bank Tejarat", "بانک تجارت"),
    ("585947", "Khavarmianeh Bank", "بانک خاورمیانه"),
    ("639599", "Ghavamin Bank", "بانک قوامین"),
];
