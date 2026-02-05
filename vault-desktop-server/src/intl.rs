use std::str::FromStr;

use vault_core::intl::{IntlConfig, IntlConfigOwnership, LanguageIdentifier};

pub fn get_intl_config() -> IntlConfig {
    IntlConfig {
        ownership: IntlConfigOwnership::Core {
            preferred_locales: get_preferred_locales(),
        },
    }
}

fn get_preferred_locales() -> Vec<LanguageIdentifier> {
    let Some(locale) = sys_locale::get_locale() else {
        return vec![];
    };

    let locale = match LanguageIdentifier::from_str(&locale.replace("_", "-")) {
        Ok(locale) => locale,
        Err(err) => {
            log::warn!("Failed to get preferred locales: {}", err);

            return vec![];
        }
    };

    vec![locale]
}
