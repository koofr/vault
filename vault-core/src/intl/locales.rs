use serde::Deserialize;

use vault_intl::LanguageIdentifier;

const LOCALES_WITH_MESSAGES_JSON: &str = include_str!(concat!(
    env!("OUT_DIR"),
    "/src/intl/locales/locales.min.json"
));

lazy_static::lazy_static! {
    static ref LOCALES: Vec<IntlLocale> = serde_json::from_str(LOCALES_WITH_MESSAGES_JSON)
        .expect("failed to parse generated locales json");
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntlLocale {
    pub locale: LanguageIdentifier,
    pub name: String,
    pub messages_json: String,
}

impl IntlLocale {
    pub fn default_locales() -> Vec<Self> {
        LOCALES.clone()
    }

    pub fn default_locale() -> LanguageIdentifier {
        LOCALES
            .first()
            .map(|locale| locale.locale.clone())
            .expect("generated locales json must include at least one locale")
    }
}
