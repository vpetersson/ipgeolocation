use std::collections::HashMap;
use std::sync::LazyLock;

/// Static mapping of ISO 3166-1 alpha-2 country codes to primary language codes
/// Format: "primary-REGION,fallback" (e.g., "en-US,en")
pub static COUNTRY_LANGUAGES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Major countries with specific language mappings
    m.insert("US", "en-US,en");
    m.insert("GB", "en-GB,en");
    m.insert("CA", "en-CA,en,fr-CA,fr");
    m.insert("AU", "en-AU,en");
    m.insert("NZ", "en-NZ,en");
    m.insert("IE", "en-IE,en,ga");

    // European countries
    m.insert("DE", "de-DE,de");
    m.insert("AT", "de-AT,de");
    m.insert("CH", "de-CH,de,fr-CH,fr,it-CH,it");
    m.insert("FR", "fr-FR,fr");
    m.insert("BE", "nl-BE,nl,fr-BE,fr,de-BE,de");
    m.insert("NL", "nl-NL,nl");
    m.insert("ES", "es-ES,es");
    m.insert("IT", "it-IT,it");
    m.insert("PT", "pt-PT,pt");
    m.insert("PL", "pl-PL,pl");
    m.insert("SE", "sv-SE,sv");
    m.insert("NO", "nb-NO,no,nn-NO");
    m.insert("DK", "da-DK,da");
    m.insert("FI", "fi-FI,fi,sv-FI,sv");
    m.insert("GR", "el-GR,el");
    m.insert("CZ", "cs-CZ,cs");
    m.insert("SK", "sk-SK,sk");
    m.insert("HU", "hu-HU,hu");
    m.insert("RO", "ro-RO,ro");
    m.insert("BG", "bg-BG,bg");
    m.insert("HR", "hr-HR,hr");
    m.insert("SI", "sl-SI,sl");
    m.insert("RS", "sr-RS,sr");
    m.insert("UA", "uk-UA,uk");
    m.insert("RU", "ru-RU,ru");
    m.insert("BY", "be-BY,be,ru-BY,ru");
    m.insert("LT", "lt-LT,lt");
    m.insert("LV", "lv-LV,lv");
    m.insert("EE", "et-EE,et");
    m.insert("IS", "is-IS,is");
    m.insert("LU", "lb-LU,lb,fr-LU,fr,de-LU,de");
    m.insert("MT", "mt-MT,mt,en-MT,en");
    m.insert("CY", "el-CY,el,tr-CY,tr");
    m.insert("AL", "sq-AL,sq");
    m.insert("MK", "mk-MK,mk");
    m.insert("BA", "bs-BA,bs,hr-BA,hr,sr-BA,sr");
    m.insert("ME", "sr-ME,sr");
    m.insert("XK", "sq-XK,sq,sr-XK,sr");
    m.insert("MD", "ro-MD,ro");

    // Asia
    m.insert("CN", "zh-CN,zh");
    m.insert("TW", "zh-TW,zh");
    m.insert("HK", "zh-HK,zh,en-HK,en");
    m.insert("JP", "ja-JP,ja");
    m.insert("KR", "ko-KR,ko");
    m.insert("IN", "hi-IN,hi,en-IN,en");
    m.insert("PK", "ur-PK,ur,en-PK,en");
    m.insert("BD", "bn-BD,bn");
    m.insert("ID", "id-ID,id");
    m.insert("MY", "ms-MY,ms,en-MY,en");
    m.insert("SG", "en-SG,en,zh-SG,zh,ms-SG,ms,ta-SG,ta");
    m.insert("TH", "th-TH,th");
    m.insert("VN", "vi-VN,vi");
    m.insert("PH", "tl-PH,tl,en-PH,en");
    m.insert("MM", "my-MM,my");
    m.insert("KH", "km-KH,km");
    m.insert("LA", "lo-LA,lo");
    m.insert("NP", "ne-NP,ne");
    m.insert("LK", "si-LK,si,ta-LK,ta");
    m.insert("MN", "mn-MN,mn");
    m.insert("KZ", "kk-KZ,kk,ru-KZ,ru");
    m.insert("UZ", "uz-UZ,uz,ru-UZ,ru");
    m.insert("TM", "tk-TM,tk");
    m.insert("TJ", "tg-TJ,tg");
    m.insert("KG", "ky-KG,ky,ru-KG,ru");
    m.insert("AF", "ps-AF,ps,fa-AF,fa");

    // Middle East
    m.insert("TR", "tr-TR,tr");
    m.insert("IR", "fa-IR,fa");
    m.insert("IQ", "ar-IQ,ar,ku-IQ,ku");
    m.insert("SA", "ar-SA,ar");
    m.insert("AE", "ar-AE,ar,en-AE,en");
    m.insert("IL", "he-IL,he,ar-IL,ar");
    m.insert("JO", "ar-JO,ar");
    m.insert("LB", "ar-LB,ar,fr-LB,fr");
    m.insert("SY", "ar-SY,ar");
    m.insert("KW", "ar-KW,ar");
    m.insert("QA", "ar-QA,ar");
    m.insert("BH", "ar-BH,ar");
    m.insert("OM", "ar-OM,ar");
    m.insert("YE", "ar-YE,ar");
    m.insert("EG", "ar-EG,ar");

    // Africa
    m.insert("ZA", "en-ZA,en,af-ZA,af,zu-ZA,zu");
    m.insert("NG", "en-NG,en");
    m.insert("KE", "sw-KE,sw,en-KE,en");
    m.insert("ET", "am-ET,am");
    m.insert("GH", "en-GH,en");
    m.insert("TZ", "sw-TZ,sw,en-TZ,en");
    m.insert("UG", "en-UG,en,sw-UG,sw");
    m.insert("MA", "ar-MA,ar,fr-MA,fr");
    m.insert("DZ", "ar-DZ,ar,fr-DZ,fr");
    m.insert("TN", "ar-TN,ar,fr-TN,fr");
    m.insert("LY", "ar-LY,ar");
    m.insert("SD", "ar-SD,ar");
    m.insert("SN", "fr-SN,fr");
    m.insert("CI", "fr-CI,fr");
    m.insert("CM", "fr-CM,fr,en-CM,en");
    m.insert("CD", "fr-CD,fr");
    m.insert("AO", "pt-AO,pt");
    m.insert("MZ", "pt-MZ,pt");
    m.insert("ZW", "en-ZW,en,sn-ZW,sn");
    m.insert("ZM", "en-ZM,en");
    m.insert("BW", "en-BW,en,tn-BW,tn");
    m.insert("NA", "en-NA,en,af-NA,af");
    m.insert("MU", "en-MU,en,fr-MU,fr");
    m.insert("MG", "mg-MG,mg,fr-MG,fr");
    m.insert("RW", "rw-RW,rw,en-RW,en,fr-RW,fr");

    // Americas
    m.insert("MX", "es-MX,es");
    m.insert("BR", "pt-BR,pt");
    m.insert("AR", "es-AR,es");
    m.insert("CO", "es-CO,es");
    m.insert("PE", "es-PE,es");
    m.insert("VE", "es-VE,es");
    m.insert("CL", "es-CL,es");
    m.insert("EC", "es-EC,es");
    m.insert("BO", "es-BO,es");
    m.insert("PY", "es-PY,es,gn-PY,gn");
    m.insert("UY", "es-UY,es");
    m.insert("GY", "en-GY,en");
    m.insert("SR", "nl-SR,nl");
    m.insert("CR", "es-CR,es");
    m.insert("PA", "es-PA,es");
    m.insert("NI", "es-NI,es");
    m.insert("HN", "es-HN,es");
    m.insert("SV", "es-SV,es");
    m.insert("GT", "es-GT,es");
    m.insert("BZ", "en-BZ,en,es-BZ,es");
    m.insert("CU", "es-CU,es");
    m.insert("DO", "es-DO,es");
    m.insert("PR", "es-PR,es,en-PR,en");
    m.insert("JM", "en-JM,en");
    m.insert("TT", "en-TT,en");
    m.insert("HT", "fr-HT,fr,ht-HT,ht");

    // Oceania
    m.insert("FJ", "en-FJ,en,fj-FJ,fj");
    m.insert("PG", "en-PG,en");
    m.insert("NC", "fr-NC,fr");
    m.insert("PF", "fr-PF,fr");
    m.insert("GU", "en-GU,en,ch-GU,ch");

    m
});

/// Get languages string for a country code
/// Returns empty string if country code is not found
#[must_use]
pub fn get_languages(country_code: Option<&str>) -> String {
    match country_code {
        Some(code) => COUNTRY_LANGUAGES
            .get(code.to_uppercase().as_str())
            .copied()
            .unwrap_or("")
            .to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_languages_us() {
        assert_eq!(get_languages(Some("US")), "en-US,en");
        assert_eq!(get_languages(Some("us")), "en-US,en");
    }

    #[test]
    fn test_get_languages_sweden() {
        assert_eq!(get_languages(Some("SE")), "sv-SE,sv");
    }

    #[test]
    fn test_get_languages_unknown() {
        assert_eq!(get_languages(Some("XX")), "");
    }

    #[test]
    fn test_get_languages_none() {
        assert_eq!(get_languages(None), "");
    }
}
