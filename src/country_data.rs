//! Comprehensive country metadata including capitals, continents, currencies, etc.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Country metadata structure
#[derive(Debug, Clone, Copy)]
pub struct CountryMetadata {
    pub name: &'static str,
    pub official_name: &'static str,
    pub iso_code3: &'static str,
    pub capital: &'static str,
    pub continent_code: &'static str,
    pub continent_name: &'static str,
    pub calling_code: &'static str,
    pub tld: &'static str,
    pub currency_code: &'static str,
    pub currency_name: &'static str,
    pub currency_symbol: &'static str,
    pub languages: &'static str,
    pub flag_emoji: &'static str,
    pub is_eu: bool,
}

/// Static mapping of ISO 3166-1 alpha-2 country codes to metadata
pub static COUNTRY_DATA: LazyLock<HashMap<&'static str, CountryMetadata>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // North America
    m.insert(
        "US",
        CountryMetadata {
            name: "United States",
            official_name: "United States of America",
            iso_code3: "USA",
            capital: "Washington, D.C.",
            continent_code: "NA",
            continent_name: "North America",
            calling_code: "+1",
            tld: ".us",
            currency_code: "USD",
            currency_name: "US Dollar",
            currency_symbol: "$",
            languages: "en-US,es-US",
            flag_emoji: "🇺🇸",
            is_eu: false,
        },
    );
    m.insert(
        "CA",
        CountryMetadata {
            name: "Canada",
            official_name: "Canada",
            iso_code3: "CAN",
            capital: "Ottawa",
            continent_code: "NA",
            continent_name: "North America",
            calling_code: "+1",
            tld: ".ca",
            currency_code: "CAD",
            currency_name: "Canadian Dollar",
            currency_symbol: "$",
            languages: "en-CA,fr-CA",
            flag_emoji: "🇨🇦",
            is_eu: false,
        },
    );
    m.insert(
        "MX",
        CountryMetadata {
            name: "Mexico",
            official_name: "United Mexican States",
            iso_code3: "MEX",
            capital: "Mexico City",
            continent_code: "NA",
            continent_name: "North America",
            calling_code: "+52",
            tld: ".mx",
            currency_code: "MXN",
            currency_name: "Mexican Peso",
            currency_symbol: "$",
            languages: "es-MX",
            flag_emoji: "🇲🇽",
            is_eu: false,
        },
    );

    // Europe
    m.insert(
        "GB",
        CountryMetadata {
            name: "United Kingdom",
            official_name: "United Kingdom of Great Britain and Northern Ireland",
            iso_code3: "GBR",
            capital: "London",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+44",
            tld: ".uk",
            currency_code: "GBP",
            currency_name: "British Pound",
            currency_symbol: "£",
            languages: "en-GB",
            flag_emoji: "🇬🇧",
            is_eu: false,
        },
    );
    m.insert(
        "DE",
        CountryMetadata {
            name: "Germany",
            official_name: "Federal Republic of Germany",
            iso_code3: "DEU",
            capital: "Berlin",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+49",
            tld: ".de",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "de-DE",
            flag_emoji: "🇩🇪",
            is_eu: true,
        },
    );
    m.insert(
        "FR",
        CountryMetadata {
            name: "France",
            official_name: "French Republic",
            iso_code3: "FRA",
            capital: "Paris",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+33",
            tld: ".fr",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "fr-FR",
            flag_emoji: "🇫🇷",
            is_eu: true,
        },
    );
    m.insert(
        "IT",
        CountryMetadata {
            name: "Italy",
            official_name: "Italian Republic",
            iso_code3: "ITA",
            capital: "Rome",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+39",
            tld: ".it",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "it-IT",
            flag_emoji: "🇮🇹",
            is_eu: true,
        },
    );
    m.insert(
        "ES",
        CountryMetadata {
            name: "Spain",
            official_name: "Kingdom of Spain",
            iso_code3: "ESP",
            capital: "Madrid",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+34",
            tld: ".es",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "es-ES",
            flag_emoji: "🇪🇸",
            is_eu: true,
        },
    );
    m.insert(
        "PT",
        CountryMetadata {
            name: "Portugal",
            official_name: "Portuguese Republic",
            iso_code3: "PRT",
            capital: "Lisbon",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+351",
            tld: ".pt",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "pt-PT",
            flag_emoji: "🇵🇹",
            is_eu: true,
        },
    );
    m.insert(
        "NL",
        CountryMetadata {
            name: "Netherlands",
            official_name: "Kingdom of the Netherlands",
            iso_code3: "NLD",
            capital: "Amsterdam",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+31",
            tld: ".nl",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "nl-NL",
            flag_emoji: "🇳🇱",
            is_eu: true,
        },
    );
    m.insert(
        "BE",
        CountryMetadata {
            name: "Belgium",
            official_name: "Kingdom of Belgium",
            iso_code3: "BEL",
            capital: "Brussels",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+32",
            tld: ".be",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "nl-BE,fr-BE,de-BE",
            flag_emoji: "🇧🇪",
            is_eu: true,
        },
    );
    m.insert(
        "AT",
        CountryMetadata {
            name: "Austria",
            official_name: "Republic of Austria",
            iso_code3: "AUT",
            capital: "Vienna",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+43",
            tld: ".at",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "de-AT",
            flag_emoji: "🇦🇹",
            is_eu: true,
        },
    );
    m.insert(
        "CH",
        CountryMetadata {
            name: "Switzerland",
            official_name: "Swiss Confederation",
            iso_code3: "CHE",
            capital: "Bern",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+41",
            tld: ".ch",
            currency_code: "CHF",
            currency_name: "Swiss Franc",
            currency_symbol: "CHF",
            languages: "de-CH,fr-CH,it-CH,rm-CH",
            flag_emoji: "🇨🇭",
            is_eu: false,
        },
    );
    m.insert(
        "SE",
        CountryMetadata {
            name: "Sweden",
            official_name: "Kingdom of Sweden",
            iso_code3: "SWE",
            capital: "Stockholm",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+46",
            tld: ".se",
            currency_code: "SEK",
            currency_name: "Swedish Krona",
            currency_symbol: "kr",
            languages: "sv-SE",
            flag_emoji: "🇸🇪",
            is_eu: true,
        },
    );
    m.insert(
        "NO",
        CountryMetadata {
            name: "Norway",
            official_name: "Kingdom of Norway",
            iso_code3: "NOR",
            capital: "Oslo",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+47",
            tld: ".no",
            currency_code: "NOK",
            currency_name: "Norwegian Krone",
            currency_symbol: "kr",
            languages: "nb-NO,nn-NO",
            flag_emoji: "🇳🇴",
            is_eu: false,
        },
    );
    m.insert(
        "DK",
        CountryMetadata {
            name: "Denmark",
            official_name: "Kingdom of Denmark",
            iso_code3: "DNK",
            capital: "Copenhagen",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+45",
            tld: ".dk",
            currency_code: "DKK",
            currency_name: "Danish Krone",
            currency_symbol: "kr",
            languages: "da-DK",
            flag_emoji: "🇩🇰",
            is_eu: true,
        },
    );
    m.insert(
        "FI",
        CountryMetadata {
            name: "Finland",
            official_name: "Republic of Finland",
            iso_code3: "FIN",
            capital: "Helsinki",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+358",
            tld: ".fi",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "fi-FI,sv-FI",
            flag_emoji: "🇫🇮",
            is_eu: true,
        },
    );
    m.insert(
        "PL",
        CountryMetadata {
            name: "Poland",
            official_name: "Republic of Poland",
            iso_code3: "POL",
            capital: "Warsaw",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+48",
            tld: ".pl",
            currency_code: "PLN",
            currency_name: "Polish Zloty",
            currency_symbol: "zł",
            languages: "pl-PL",
            flag_emoji: "🇵🇱",
            is_eu: true,
        },
    );
    m.insert(
        "CZ",
        CountryMetadata {
            name: "Czechia",
            official_name: "Czech Republic",
            iso_code3: "CZE",
            capital: "Prague",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+420",
            tld: ".cz",
            currency_code: "CZK",
            currency_name: "Czech Koruna",
            currency_symbol: "Kč",
            languages: "cs-CZ",
            flag_emoji: "🇨🇿",
            is_eu: true,
        },
    );
    m.insert(
        "GR",
        CountryMetadata {
            name: "Greece",
            official_name: "Hellenic Republic",
            iso_code3: "GRC",
            capital: "Athens",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+30",
            tld: ".gr",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "el-GR",
            flag_emoji: "🇬🇷",
            is_eu: true,
        },
    );
    m.insert(
        "IE",
        CountryMetadata {
            name: "Ireland",
            official_name: "Republic of Ireland",
            iso_code3: "IRL",
            capital: "Dublin",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+353",
            tld: ".ie",
            currency_code: "EUR",
            currency_name: "Euro",
            currency_symbol: "€",
            languages: "en-IE,ga-IE",
            flag_emoji: "🇮🇪",
            is_eu: true,
        },
    );
    m.insert(
        "RU",
        CountryMetadata {
            name: "Russia",
            official_name: "Russian Federation",
            iso_code3: "RUS",
            capital: "Moscow",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+7",
            tld: ".ru",
            currency_code: "RUB",
            currency_name: "Russian Ruble",
            currency_symbol: "₽",
            languages: "ru-RU",
            flag_emoji: "🇷🇺",
            is_eu: false,
        },
    );
    m.insert(
        "UA",
        CountryMetadata {
            name: "Ukraine",
            official_name: "Ukraine",
            iso_code3: "UKR",
            capital: "Kyiv",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+380",
            tld: ".ua",
            currency_code: "UAH",
            currency_name: "Ukrainian Hryvnia",
            currency_symbol: "₴",
            languages: "uk-UA",
            flag_emoji: "🇺🇦",
            is_eu: false,
        },
    );
    m.insert(
        "RO",
        CountryMetadata {
            name: "Romania",
            official_name: "Romania",
            iso_code3: "ROU",
            capital: "Bucharest",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+40",
            tld: ".ro",
            currency_code: "RON",
            currency_name: "Romanian Leu",
            currency_symbol: "lei",
            languages: "ro-RO",
            flag_emoji: "🇷🇴",
            is_eu: true,
        },
    );
    m.insert(
        "HU",
        CountryMetadata {
            name: "Hungary",
            official_name: "Hungary",
            iso_code3: "HUN",
            capital: "Budapest",
            continent_code: "EU",
            continent_name: "Europe",
            calling_code: "+36",
            tld: ".hu",
            currency_code: "HUF",
            currency_name: "Hungarian Forint",
            currency_symbol: "Ft",
            languages: "hu-HU",
            flag_emoji: "🇭🇺",
            is_eu: true,
        },
    );

    // Asia
    m.insert(
        "JP",
        CountryMetadata {
            name: "Japan",
            official_name: "Japan",
            iso_code3: "JPN",
            capital: "Tokyo",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+81",
            tld: ".jp",
            currency_code: "JPY",
            currency_name: "Japanese Yen",
            currency_symbol: "¥",
            languages: "ja-JP",
            flag_emoji: "🇯🇵",
            is_eu: false,
        },
    );
    m.insert(
        "CN",
        CountryMetadata {
            name: "China",
            official_name: "People's Republic of China",
            iso_code3: "CHN",
            capital: "Beijing",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+86",
            tld: ".cn",
            currency_code: "CNY",
            currency_name: "Chinese Yuan",
            currency_symbol: "¥",
            languages: "zh-CN",
            flag_emoji: "🇨🇳",
            is_eu: false,
        },
    );
    m.insert(
        "KR",
        CountryMetadata {
            name: "South Korea",
            official_name: "Republic of Korea",
            iso_code3: "KOR",
            capital: "Seoul",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+82",
            tld: ".kr",
            currency_code: "KRW",
            currency_name: "South Korean Won",
            currency_symbol: "₩",
            languages: "ko-KR",
            flag_emoji: "🇰🇷",
            is_eu: false,
        },
    );
    m.insert(
        "IN",
        CountryMetadata {
            name: "India",
            official_name: "Republic of India",
            iso_code3: "IND",
            capital: "New Delhi",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+91",
            tld: ".in",
            currency_code: "INR",
            currency_name: "Indian Rupee",
            currency_symbol: "₹",
            languages: "hi-IN,en-IN",
            flag_emoji: "🇮🇳",
            is_eu: false,
        },
    );
    m.insert(
        "SG",
        CountryMetadata {
            name: "Singapore",
            official_name: "Republic of Singapore",
            iso_code3: "SGP",
            capital: "Singapore",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+65",
            tld: ".sg",
            currency_code: "SGD",
            currency_name: "Singapore Dollar",
            currency_symbol: "$",
            languages: "en-SG,zh-SG,ms-SG,ta-SG",
            flag_emoji: "🇸🇬",
            is_eu: false,
        },
    );
    m.insert(
        "HK",
        CountryMetadata {
            name: "Hong Kong",
            official_name: "Hong Kong Special Administrative Region",
            iso_code3: "HKG",
            capital: "Hong Kong",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+852",
            tld: ".hk",
            currency_code: "HKD",
            currency_name: "Hong Kong Dollar",
            currency_symbol: "$",
            languages: "zh-HK,en-HK",
            flag_emoji: "🇭🇰",
            is_eu: false,
        },
    );
    m.insert(
        "TW",
        CountryMetadata {
            name: "Taiwan",
            official_name: "Republic of China (Taiwan)",
            iso_code3: "TWN",
            capital: "Taipei",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+886",
            tld: ".tw",
            currency_code: "TWD",
            currency_name: "New Taiwan Dollar",
            currency_symbol: "NT$",
            languages: "zh-TW",
            flag_emoji: "🇹🇼",
            is_eu: false,
        },
    );
    m.insert(
        "TH",
        CountryMetadata {
            name: "Thailand",
            official_name: "Kingdom of Thailand",
            iso_code3: "THA",
            capital: "Bangkok",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+66",
            tld: ".th",
            currency_code: "THB",
            currency_name: "Thai Baht",
            currency_symbol: "฿",
            languages: "th-TH",
            flag_emoji: "🇹🇭",
            is_eu: false,
        },
    );
    m.insert(
        "VN",
        CountryMetadata {
            name: "Vietnam",
            official_name: "Socialist Republic of Vietnam",
            iso_code3: "VNM",
            capital: "Hanoi",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+84",
            tld: ".vn",
            currency_code: "VND",
            currency_name: "Vietnamese Dong",
            currency_symbol: "₫",
            languages: "vi-VN",
            flag_emoji: "🇻🇳",
            is_eu: false,
        },
    );
    m.insert(
        "ID",
        CountryMetadata {
            name: "Indonesia",
            official_name: "Republic of Indonesia",
            iso_code3: "IDN",
            capital: "Jakarta",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+62",
            tld: ".id",
            currency_code: "IDR",
            currency_name: "Indonesian Rupiah",
            currency_symbol: "Rp",
            languages: "id-ID",
            flag_emoji: "🇮🇩",
            is_eu: false,
        },
    );
    m.insert(
        "MY",
        CountryMetadata {
            name: "Malaysia",
            official_name: "Malaysia",
            iso_code3: "MYS",
            capital: "Kuala Lumpur",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+60",
            tld: ".my",
            currency_code: "MYR",
            currency_name: "Malaysian Ringgit",
            currency_symbol: "RM",
            languages: "ms-MY,en-MY",
            flag_emoji: "🇲🇾",
            is_eu: false,
        },
    );
    m.insert(
        "PH",
        CountryMetadata {
            name: "Philippines",
            official_name: "Republic of the Philippines",
            iso_code3: "PHL",
            capital: "Manila",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+63",
            tld: ".ph",
            currency_code: "PHP",
            currency_name: "Philippine Peso",
            currency_symbol: "₱",
            languages: "tl-PH,en-PH",
            flag_emoji: "🇵🇭",
            is_eu: false,
        },
    );
    m.insert(
        "AE",
        CountryMetadata {
            name: "United Arab Emirates",
            official_name: "United Arab Emirates",
            iso_code3: "ARE",
            capital: "Abu Dhabi",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+971",
            tld: ".ae",
            currency_code: "AED",
            currency_name: "UAE Dirham",
            currency_symbol: "د.إ",
            languages: "ar-AE,en-AE",
            flag_emoji: "🇦🇪",
            is_eu: false,
        },
    );
    m.insert(
        "SA",
        CountryMetadata {
            name: "Saudi Arabia",
            official_name: "Kingdom of Saudi Arabia",
            iso_code3: "SAU",
            capital: "Riyadh",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+966",
            tld: ".sa",
            currency_code: "SAR",
            currency_name: "Saudi Riyal",
            currency_symbol: "﷼",
            languages: "ar-SA",
            flag_emoji: "🇸🇦",
            is_eu: false,
        },
    );
    m.insert(
        "IL",
        CountryMetadata {
            name: "Israel",
            official_name: "State of Israel",
            iso_code3: "ISR",
            capital: "Jerusalem",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+972",
            tld: ".il",
            currency_code: "ILS",
            currency_name: "Israeli Shekel",
            currency_symbol: "₪",
            languages: "he-IL,ar-IL",
            flag_emoji: "🇮🇱",
            is_eu: false,
        },
    );
    m.insert(
        "TR",
        CountryMetadata {
            name: "Turkey",
            official_name: "Republic of Türkiye",
            iso_code3: "TUR",
            capital: "Ankara",
            continent_code: "AS",
            continent_name: "Asia",
            calling_code: "+90",
            tld: ".tr",
            currency_code: "TRY",
            currency_name: "Turkish Lira",
            currency_symbol: "₺",
            languages: "tr-TR",
            flag_emoji: "🇹🇷",
            is_eu: false,
        },
    );

    // Oceania
    m.insert(
        "AU",
        CountryMetadata {
            name: "Australia",
            official_name: "Commonwealth of Australia",
            iso_code3: "AUS",
            capital: "Canberra",
            continent_code: "OC",
            continent_name: "Oceania",
            calling_code: "+61",
            tld: ".au",
            currency_code: "AUD",
            currency_name: "Australian Dollar",
            currency_symbol: "$",
            languages: "en-AU",
            flag_emoji: "🇦🇺",
            is_eu: false,
        },
    );
    m.insert(
        "NZ",
        CountryMetadata {
            name: "New Zealand",
            official_name: "New Zealand",
            iso_code3: "NZL",
            capital: "Wellington",
            continent_code: "OC",
            continent_name: "Oceania",
            calling_code: "+64",
            tld: ".nz",
            currency_code: "NZD",
            currency_name: "New Zealand Dollar",
            currency_symbol: "$",
            languages: "en-NZ,mi-NZ",
            flag_emoji: "🇳🇿",
            is_eu: false,
        },
    );

    // South America
    m.insert(
        "BR",
        CountryMetadata {
            name: "Brazil",
            official_name: "Federative Republic of Brazil",
            iso_code3: "BRA",
            capital: "Brasília",
            continent_code: "SA",
            continent_name: "South America",
            calling_code: "+55",
            tld: ".br",
            currency_code: "BRL",
            currency_name: "Brazilian Real",
            currency_symbol: "R$",
            languages: "pt-BR",
            flag_emoji: "🇧🇷",
            is_eu: false,
        },
    );
    m.insert(
        "AR",
        CountryMetadata {
            name: "Argentina",
            official_name: "Argentine Republic",
            iso_code3: "ARG",
            capital: "Buenos Aires",
            continent_code: "SA",
            continent_name: "South America",
            calling_code: "+54",
            tld: ".ar",
            currency_code: "ARS",
            currency_name: "Argentine Peso",
            currency_symbol: "$",
            languages: "es-AR",
            flag_emoji: "🇦🇷",
            is_eu: false,
        },
    );
    m.insert(
        "CL",
        CountryMetadata {
            name: "Chile",
            official_name: "Republic of Chile",
            iso_code3: "CHL",
            capital: "Santiago",
            continent_code: "SA",
            continent_name: "South America",
            calling_code: "+56",
            tld: ".cl",
            currency_code: "CLP",
            currency_name: "Chilean Peso",
            currency_symbol: "$",
            languages: "es-CL",
            flag_emoji: "🇨🇱",
            is_eu: false,
        },
    );
    m.insert(
        "CO",
        CountryMetadata {
            name: "Colombia",
            official_name: "Republic of Colombia",
            iso_code3: "COL",
            capital: "Bogotá",
            continent_code: "SA",
            continent_name: "South America",
            calling_code: "+57",
            tld: ".co",
            currency_code: "COP",
            currency_name: "Colombian Peso",
            currency_symbol: "$",
            languages: "es-CO",
            flag_emoji: "🇨🇴",
            is_eu: false,
        },
    );

    // Africa
    m.insert(
        "ZA",
        CountryMetadata {
            name: "South Africa",
            official_name: "Republic of South Africa",
            iso_code3: "ZAF",
            capital: "Pretoria",
            continent_code: "AF",
            continent_name: "Africa",
            calling_code: "+27",
            tld: ".za",
            currency_code: "ZAR",
            currency_name: "South African Rand",
            currency_symbol: "R",
            languages: "en-ZA,af-ZA,zu-ZA",
            flag_emoji: "🇿🇦",
            is_eu: false,
        },
    );
    m.insert(
        "NG",
        CountryMetadata {
            name: "Nigeria",
            official_name: "Federal Republic of Nigeria",
            iso_code3: "NGA",
            capital: "Abuja",
            continent_code: "AF",
            continent_name: "Africa",
            calling_code: "+234",
            tld: ".ng",
            currency_code: "NGN",
            currency_name: "Nigerian Naira",
            currency_symbol: "₦",
            languages: "en-NG",
            flag_emoji: "🇳🇬",
            is_eu: false,
        },
    );
    m.insert(
        "EG",
        CountryMetadata {
            name: "Egypt",
            official_name: "Arab Republic of Egypt",
            iso_code3: "EGY",
            capital: "Cairo",
            continent_code: "AF",
            continent_name: "Africa",
            calling_code: "+20",
            tld: ".eg",
            currency_code: "EGP",
            currency_name: "Egyptian Pound",
            currency_symbol: "£",
            languages: "ar-EG",
            flag_emoji: "🇪🇬",
            is_eu: false,
        },
    );
    m.insert(
        "KE",
        CountryMetadata {
            name: "Kenya",
            official_name: "Republic of Kenya",
            iso_code3: "KEN",
            capital: "Nairobi",
            continent_code: "AF",
            continent_name: "Africa",
            calling_code: "+254",
            tld: ".ke",
            currency_code: "KES",
            currency_name: "Kenyan Shilling",
            currency_symbol: "KSh",
            languages: "sw-KE,en-KE",
            flag_emoji: "🇰🇪",
            is_eu: false,
        },
    );

    m
});

/// Get country metadata by ISO 3166-1 alpha-2 code
#[must_use]
pub fn get_country_metadata(country_code: Option<&str>) -> Option<&'static CountryMetadata> {
    country_code.and_then(|code| COUNTRY_DATA.get(code.to_uppercase().as_str()))
}

/// Get country metadata with fallback for unknown countries
/// Returns metadata if available, otherwise creates minimal fallback data
#[must_use]
pub fn get_country_metadata_or_fallback(country_code: Option<&str>) -> Option<CountryMetadata> {
    match country_code {
        Some(code) => {
            let upper = code.to_uppercase();
            if let Some(meta) = COUNTRY_DATA.get(upper.as_str()) {
                Some(*meta)
            } else {
                // Return minimal fallback for unknown country codes
                Some(CountryMetadata {
                    name: "Unknown",
                    official_name: "Unknown",
                    iso_code3: "UNK",
                    capital: "Unknown",
                    continent_code: "XX",
                    continent_name: "Unknown",
                    calling_code: "",
                    tld: "",
                    currency_code: "",
                    currency_name: "",
                    currency_symbol: "",
                    languages: "",
                    flag_emoji: "🏳️",
                    is_eu: false,
                })
            }
        }
        None => None,
    }
}

/// Get flag path for a country code
/// Returns a relative path suitable for static file serving
/// Use with flag-icons (https://github.com/lipis/flag-icons) or similar
#[must_use]
pub fn get_flag_path(country_code: &str) -> String {
    format!("/static/flags/{}.svg", country_code.to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_country_metadata_us() {
        let meta = get_country_metadata(Some("US")).unwrap();
        assert_eq!(meta.name, "United States");
        assert_eq!(meta.capital, "Washington, D.C.");
        assert_eq!(meta.currency_code, "USD");
        assert!(!meta.is_eu);
    }

    #[test]
    fn test_get_country_metadata_de() {
        let meta = get_country_metadata(Some("DE")).unwrap();
        assert_eq!(meta.name, "Germany");
        assert!(meta.is_eu);
        assert_eq!(meta.currency_code, "EUR");
    }

    #[test]
    fn test_get_country_metadata_lowercase() {
        let meta = get_country_metadata(Some("se")).unwrap();
        assert_eq!(meta.name, "Sweden");
    }

    #[test]
    fn test_get_country_metadata_unknown() {
        assert!(get_country_metadata(Some("XX")).is_none());
    }

    #[test]
    fn test_get_country_metadata_none() {
        assert!(get_country_metadata(None).is_none());
    }

    #[test]
    fn test_get_country_metadata_or_fallback_known() {
        let meta = get_country_metadata_or_fallback(Some("US")).unwrap();
        assert_eq!(meta.name, "United States");
        assert_eq!(meta.currency_code, "USD");
    }

    #[test]
    fn test_get_country_metadata_or_fallback_unknown() {
        let meta = get_country_metadata_or_fallback(Some("XX")).unwrap();
        assert_eq!(meta.name, "Unknown");
        assert_eq!(meta.flag_emoji, "🏳️");
        assert!(!meta.is_eu);
    }

    #[test]
    fn test_get_country_metadata_or_fallback_none() {
        assert!(get_country_metadata_or_fallback(None).is_none());
    }

    #[test]
    fn test_get_flag_path() {
        assert_eq!(get_flag_path("US"), "/static/flags/us.svg");
    }
}
