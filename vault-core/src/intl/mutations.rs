use vault_intl::{CatalogFormatter, LanguageIdentifier, negotiate_language};

use crate::{
    intl::{
        errors::SetLocaleError,
        state::{ChangeLocaleStrategy, OwnershipInfo},
    },
    store,
};

use super::selectors;

pub fn set_current_locale(
    state: &mut store::State,
    notify: &store::Notify,
    strategy: ChangeLocaleStrategy,
) -> Result<(LanguageIdentifier, OwnershipInfo, Option<CatalogFormatter>), SetLocaleError> {
    notify(store::Event::Intl);

    let locale = match strategy {
        ChangeLocaleStrategy::Exact(locale) => locale,
        ChangeLocaleStrategy::Lookup(locales) => negotiate_language(
            locales.iter().collect(),
            state.intl.locales.iter().map(|l| &l.locale).collect(),
        )
        .ok_or(SetLocaleError::LookupFailed(locales))?
        .to_owned(),
    };

    let loc = selectors::select_locale(state, &locale)
        .ok_or_else(|| SetLocaleError::LocaleNotFound(locale.clone()))?;

    let catalog_formatter = selectors::catalog_formatter_for_locale(state, loc)?;

    state.intl.current_locale = Some(locale.clone());

    let ownership_info = selectors::select_ownership_info(state);

    Ok((locale, ownership_info, catalog_formatter))
}

#[cfg(test)]
mod tests {
    use crate::{intl::locales::IntlLocale, store};

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
    fn test_set_current_locale_sets_locale_when_valid() {
        let (notify, _, _) = store::test_helpers::mutation();

        let mut state = state_with_locales("en");
        let (locale, ownership_info, formatter) = set_current_locale(
            &mut state,
            &notify,
            ChangeLocaleStrategy::Exact("en".parse().unwrap()),
        )
        .unwrap();
        assert_eq!(state.intl.current_locale, Some("en".parse().unwrap()));
        assert_eq!(locale, "en".parse().unwrap());
        assert!(matches!(ownership_info, OwnershipInfo::Core));
        assert!(formatter.is_none());

        let (locale, ownership_info, formatter) = set_current_locale(
            &mut state,
            &notify,
            ChangeLocaleStrategy::Exact("sl".parse().unwrap()),
        )
        .unwrap();
        assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
        assert_eq!(locale, "sl".parse().unwrap());
        assert!(matches!(ownership_info, OwnershipInfo::Core));
        assert!(formatter.is_some());
    }

    #[test]
    fn test_set_current_locale_invalid_locale_returns_error() {
        let (notify, _, _) = store::test_helpers::mutation();

        let mut state = state_with_locales("en");
        let result = set_current_locale(
            &mut state,
            &notify,
            ChangeLocaleStrategy::Exact("fr".parse().unwrap()),
        );

        assert!(matches!(
            result,
            Err(SetLocaleError::LocaleNotFound(locale)) if locale == "fr".parse().unwrap()
        ));
        assert_eq!(state.intl.current_locale, None);
    }

    #[test]
    fn test_set_current_locale_lookup_sets_best_matching_locale() {
        let (notify, _, _) = store::test_helpers::mutation();
        let mut state = state_with_locales("en");

        let (locale, ownership_info, formatter) = set_current_locale(
            &mut state,
            &notify,
            ChangeLocaleStrategy::Lookup(vec!["fr-FR".parse().unwrap(), "sl-SI".parse().unwrap()]),
        )
        .unwrap();

        assert_eq!(locale, "sl".parse().unwrap());
        assert!(matches!(ownership_info, OwnershipInfo::Core));
        assert_eq!(state.intl.current_locale, Some("sl".parse().unwrap()));
        assert!(formatter.is_some());
    }

    #[test]
    fn test_set_current_locale_lookup_returns_error_when_no_locale_matches() {
        let (notify, _, _) = store::test_helpers::mutation();
        let mut state = state_with_locales("en");
        let lookup_locales = vec!["fr".parse().unwrap(), "de".parse().unwrap()];

        let result = set_current_locale(
            &mut state,
            &notify,
            ChangeLocaleStrategy::Lookup(lookup_locales.clone()),
        );

        assert!(matches!(
            result,
            Err(SetLocaleError::LookupFailed(locales)) if locales == lookup_locales
        ));
        assert_eq!(state.intl.current_locale, None);
    }
}
