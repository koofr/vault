use vault_intl::{CatalogFormatter, LanguageIdentifier};

use crate::{
    intl::{errors::SetLocaleError, locales::IntlLocale, state::OwnershipInfo},
    store,
};

pub fn select_locales<'a>(state: &'a store::State) -> &'a [IntlLocale] {
    &state.intl.locales
}

pub fn select_locale<'a>(
    state: &'a store::State,
    locale: &LanguageIdentifier,
) -> Option<&'a IntlLocale> {
    select_locales(state).iter().find(|l| &l.locale == locale)
}

pub fn select_current_locale<'a>(state: &'a store::State) -> Option<&'a IntlLocale> {
    select_locale(
        state,
        &state
            .intl
            .current_locale
            .as_ref()
            .unwrap_or(&state.intl.default_locale),
    )
}

pub fn select_ownership_info(state: &store::State) -> OwnershipInfo {
    (&state.intl.ownership).into()
}

pub fn catalog_formatter_for_locale(
    state: &store::State,
    locale: &IntlLocale,
) -> Result<Option<CatalogFormatter>, SetLocaleError> {
    if locale.locale == state.intl.default_locale {
        return Ok(None);
    }

    let catalog_formatter = CatalogFormatter::from_str(&locale.locale, &locale.messages_json)
        .map_err(SetLocaleError::FormatterError)?;

    Ok(Some(catalog_formatter))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MESSAGES_JSON_EN: &str = r###"{
  "test.message": [
    {
      "type": 0,
      "value": "Hello"
    }
  ]
}"###;

    const TEST_MESSAGES_JSON_SL: &str = r###"{
  "test.message": [
    {
      "type": 0,
      "value": "Pozdravljeni"
    }
  ]
}"###;

    fn state_with_locales(default_locale: &str) -> store::State {
        let mut state = store::State::default();
        state.intl.locales = vec![
            IntlLocale {
                locale: "en".parse().unwrap(),
                name: "English".to_string(),
                messages_json: TEST_MESSAGES_JSON_EN.to_string(),
            },
            IntlLocale {
                locale: "sl".parse().unwrap(),
                name: "Slovenian".to_string(),
                messages_json: TEST_MESSAGES_JSON_SL.to_string(),
            },
        ];
        state.intl.default_locale = default_locale.parse().unwrap();
        state
    }

    #[test]
    fn test_catalog_formatter_for_locale_returns_none_for_default() {
        let state = state_with_locales("en");
        let locale = select_locale(&state, &"en".parse().unwrap()).unwrap();

        let formatter = catalog_formatter_for_locale(&state, locale).unwrap();
        assert!(formatter.is_none());
    }

    #[test]
    fn test_catalog_formatter_for_locale_returns_some_for_non_default() {
        let state = state_with_locales("en");
        let locale = select_locale(&state, &"sl".parse().unwrap()).unwrap();

        let formatter = catalog_formatter_for_locale(&state, locale).unwrap();
        assert!(formatter.is_some());
    }
}
