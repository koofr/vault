use icu_locale::{LanguageIdentifier, LocaleFallbacker};
use icu_plurals::PluralRules;
use icu_provider_adapters::fallback::LocaleFallbackProvider;

use crate::icu_data_provider::IcuDataProvider;

lazy_static::lazy_static! {
    static ref LOCALE_FALLBACK_PROVIDER: LocaleFallbackProvider<&'static IcuDataProvider> =
        LocaleFallbackProvider::new(
            &IcuDataProvider,
            LocaleFallbacker::try_new_unstable(&IcuDataProvider)
                .expect("failed to build locale fallback"),
        );
}

pub fn build_plural_rules(
    locale: &LanguageIdentifier,
) -> Result<PluralRules, icu_provider::DataError> {
    PluralRules::try_new_cardinal_unstable(&*LOCALE_FALLBACK_PROVIDER, locale.into())
}
