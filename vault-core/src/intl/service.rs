use std::sync::{Arc, Mutex};

use vault_intl::{CatalogFormatter, FallbackFormatter, FormatArguments, LanguageIdentifier};

use crate::{
    intl::{
        errors::SetLocaleError,
        state::{ChangeLocaleStrategy, OwnershipInfo},
    },
    secure_storage::SecureStorageService,
    store,
};

use super::{mutations, selectors};

pub const CURRENT_LOCALE_STORAGE_KEY: &str = "vaultIntlCurrentLocale";

pub struct IntlService {
    secure_storage_service: Arc<SecureStorageService>,
    store: Arc<store::Store>,
    formatter: Arc<Mutex<FallbackFormatter>>,
}

impl IntlService {
    pub fn new(
        secure_storage_service: Arc<SecureStorageService>,
        store: Arc<store::Store>,
    ) -> Self {
        let formatter = Arc::new(Mutex::new(FallbackFormatter::new()));

        let intl_service = Self {
            secure_storage_service,
            store,
            formatter,
        };

        intl_service.init_default_formatter();
        intl_service.init_current_formatter();

        intl_service
    }

    fn init_default_formatter(&self) {
        match self.store.with_state(|state| {
            let locale = &state.intl.default_locale;

            let loc = selectors::select_locale(state, locale)
                .ok_or(SetLocaleError::LocaleNotFound(locale.clone()))?;

            CatalogFormatter::from_str(locale, &loc.messages_json)
                .map_err(SetLocaleError::FormatterError)
        }) {
            Ok(catalog_formatter) => {
                self.formatter
                    .lock()
                    .unwrap()
                    .set_default_formatter(Some(catalog_formatter));
            }
            Err(err) => {
                log::error!("Failed to create default formatter: {}", err);
            }
        }
    }

    fn init_current_formatter(&self) {
        match self.store.with_state(|state| {
            let Some(locale) = state.intl.current_locale.as_ref() else {
                return Ok(None);
            };

            let loc = selectors::select_locale(state, locale)
                .ok_or(SetLocaleError::LocaleNotFound(locale.clone()))?;

            selectors::catalog_formatter_for_locale(state, loc)
        }) {
            Ok(catalog_formatter) => {
                self.formatter
                    .lock()
                    .unwrap()
                    .set_current_formatter(catalog_formatter);
            }
            Err(err) => {
                log::warn!("Failed to create current formatter: {}", err);
            }
        }
    }

    pub fn load(&self) -> Result<(), SetLocaleError> {
        match self
            .store
            .with_state(|state| selectors::select_ownership_info(state))
        {
            OwnershipInfo::Core => {
                match self
                    .secure_storage_service
                    .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                {
                    Ok(Some(locale)) => {
                        self.set_current_locale(ChangeLocaleStrategy::Exact(locale.parse()?))?;

                        Ok(())
                    }
                    Ok(None) => {
                        // We don't have to do anything here. When Vault is created, the
                        // state.intl.current_locale is set to the preferred locale and
                        // the current formatter is initialized in
                        // IntlService::init_current_formatter(). After logout
                        // state.intl.reset() is called, which sets the
                        // state.intl.current_locale to the preferred locale, then
                        // IntlService::reset() is called, which re-initializes the
                        // current formatter.

                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            }
            OwnershipInfo::External => {
                // load() does not do anything in case of external ownership.
                Ok(())
            }
        }
    }

    // reset is called after logout after the store state.intl.reset() is
    // already called which sets state.intl.current_locale to the preferred
    // locale so we only need to re-initialize the current formatter here.
    pub fn reset(&self) {
        self.init_current_formatter();
    }

    pub fn change_locale(&self, strategy: ChangeLocaleStrategy) -> Result<(), SetLocaleError> {
        match self.set_current_locale(strategy) {
            Ok((locale, ownership_info)) => {
                match ownership_info {
                    OwnershipInfo::Core => {
                        self.secure_storage_service
                            .set(CURRENT_LOCALE_STORAGE_KEY, &locale.to_string())?;

                        Ok(())
                    }
                    OwnershipInfo::External => {
                        // Do nothing in case of external ownership.
                        Ok(())
                    }
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set_current_locale(
        &self,
        strategy: ChangeLocaleStrategy,
    ) -> Result<(LanguageIdentifier, OwnershipInfo), SetLocaleError> {
        match self
            .store
            .mutate(|state, notify, _, _| mutations::set_current_locale(state, notify, strategy))
        {
            Ok((locale, ownership_info, catalog_formatter)) => {
                self.formatter
                    .lock()
                    .unwrap()
                    .set_current_formatter(catalog_formatter);

                Ok((locale, ownership_info))
            }
            Err(err) => Err(err),
        }
    }

    pub fn format_message(&self, message_id: &str, args: FormatArguments<'_>) -> String {
        match self
            .formatter
            .lock()
            .unwrap()
            .format_message(message_id, args)
        {
            Ok(msg) => msg,
            Err(err) => format!("TRANSLATION ERROR: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use vault_intl::FormatValue;

    use crate::{
        intl::{locales::IntlLocale, state::Ownership},
        secure_storage::MemorySecureStorage,
    };

    use super::*;

    const TEST_MESSAGES_JSON_EN: &str = r###"{
  "test.message": [
    {
      "type": 0,
      "value": "Do you really want to delete "
    },
    {
      "offset": 0,
      "options": {
        "one": {
          "value": [
            {
              "type": 0,
              "value": "one file"
            }
          ]
        },
        "other": {
          "value": [
            {
              "type": 7
            },
            {
              "type": 0,
              "value": " files"
            }
          ]
        }
      },
      "pluralType": "cardinal",
      "type": 6,
      "value": "count"
    },
    {
      "type": 0,
      "value": "?"
    }
  ]
}"###;

    const TEST_MESSAGES_JSON_SL: &str = r###"{
  "test.message": [
    {
      "type": 0,
      "value": "Ali res želite odstraniti "
    },
    {
      "offset": 0,
      "options": {
        "=0": {
          "value": [
            {
              "type": 0,
              "value": "nič datotek"
            }
          ]
        },
        "few": {
          "value": [
            {
              "type": 7
            },
            {
              "type": 0,
              "value": " datoteke"
            }
          ]
        },
        "one": {
          "value": [
            {
              "type": 7
            },
            {
              "type": 0,
              "value": " datoteko"
            }
          ]
        },
        "other": {
          "value": [
            {
              "type": 7
            },
            {
              "type": 0,
              "value": " datotek"
            }
          ]
        },
        "two": {
          "value": [
            {
              "type": 7
            },
            {
              "type": 0,
              "value": " datoteki"
            }
          ]
        }
      },
      "pluralType": "cardinal",
      "type": 6,
      "value": "count"
    },
    {
      "type": 0,
      "value": "?"
    }
  ]
}"###;

    const TEST_MESSAGES_JSON_INVALID: &str = r###"{
  "test.message": [
    {
      "type": 1,
      "value": "invalid"
    }
  ]
}"###;

    struct Fixture {
        store: Arc<store::Store>,
        secure_storage_service: Arc<SecureStorageService>,
        intl_service: IntlService,
    }

    impl Fixture {
        fn new() -> Self {
            Self::new_with_state(|_| {})
        }

        fn new_with_state<F>(mutate: F) -> Self
        where
            F: FnOnce(&mut store::State),
        {
            let store = Arc::new(store::Store::new(store::State::default()));

            store.mutate(|state, _, _, _| {
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
                state.intl.default_locale = "en".parse().unwrap();
                mutate(state);
            });

            let secure_storage_service = Arc::new(SecureStorageService::new(Box::new(
                MemorySecureStorage::new(),
            )));
            let intl_service = IntlService::new(secure_storage_service.clone(), store.clone());

            Self {
                store,
                secure_storage_service,
                intl_service,
            }
        }
    }

    #[test]
    fn test_format_message() {
        let Fixture { intl_service, .. } = Fixture::new();

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );

        intl_service
            .change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
            .unwrap();

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );

        let result = intl_service.change_locale(ChangeLocaleStrategy::Exact("sr".parse().unwrap()));
        assert_eq!(
            result,
            Err(SetLocaleError::LocaleNotFound("sr".parse().unwrap()))
        );

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_format_message_invalid() {
        let Fixture {
            store,
            intl_service,
            ..
        } = Fixture::new();

        store.mutate(|state, _, _, _| {
            state.intl.locales = vec![
                IntlLocale {
                    locale: "en".parse().unwrap(),
                    name: "English".to_string(),
                    messages_json: TEST_MESSAGES_JSON_EN.to_string(),
                },
                IntlLocale {
                    locale: "sl".parse().unwrap(),
                    name: "Slovenian".to_string(),
                    messages_json: TEST_MESSAGES_JSON_INVALID.to_string(),
                },
            ];
            state.intl.default_locale = "en".parse().unwrap();
        });

        intl_service
            .change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
            .unwrap();

        assert_eq!(
            intl_service.format_message("test.message", &[]),
            "TRANSLATION ERROR: missing argument: invalid"
        );
    }

    #[test]
    fn test_init_current_formatter_non_default_language() {
        let Fixture { intl_service, .. } = Fixture::new_with_state(|state| {
            state.intl.current_locale = Some("sl".parse().unwrap());
        });

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_init_current_formatter_default_language() {
        let Fixture { intl_service, .. } = Fixture::new_with_state(|state| {
            state.intl.current_locale = Some("en".parse().unwrap());
        });

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_core_ownership_load_locale_from_storage() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new();

        secure_storage_service
            .set(CURRENT_LOCALE_STORAGE_KEY, &"sl")
            .unwrap();

        intl_service.load().unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_external_ownership_load_locale_from_storage() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new_with_state(|state| {
            state.intl.ownership = Ownership::External;
            state.intl.current_locale = Some("en".parse().unwrap());
        });

        secure_storage_service
            .set(CURRENT_LOCALE_STORAGE_KEY, &"sl")
            .unwrap();

        intl_service.load().unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("en".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_core_ownership_reset() {
        let Fixture {
            store,
            intl_service,
            ..
        } = Fixture::new_with_state(|state| {
            state.intl.ownership = Ownership::Core {
                preferred_locales: vec!["en".parse().unwrap()],
            };
            state.intl.current_locale = Some("sl".parse().unwrap());
        });

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );

        store.mutate(|state, _, _, _| {
            state.intl.reset();
        });
        intl_service.reset();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("en".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_external_ownership_reset() {
        let Fixture {
            store,
            intl_service,
            ..
        } = Fixture::new_with_state(|state| {
            state.intl.ownership = Ownership::External;
            state.intl.current_locale = Some("sl".parse().unwrap());
        });

        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );

        store.mutate(|state, _, _, _| {
            state.intl.reset();
        });
        intl_service.reset();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_core_ownership_change_locale() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new();

        // None by default
        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            None
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            None
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );

        // Change to "sl"
        intl_service
            .change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
            .unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );

        // Change to an invalid locale does not change anything
        let result = intl_service.change_locale(ChangeLocaleStrategy::Exact("xx".parse().unwrap()));
        assert_eq!(
            result,
            Err(SetLocaleError::LocaleNotFound("xx".parse().unwrap()))
        );

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );

        // Change to "en" sets current_locale to "en" and stores it to storage
        intl_service
            .change_locale(ChangeLocaleStrategy::Exact("en".parse().unwrap()))
            .unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("en".parse().unwrap())
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            Some("en".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            "Do you really want to delete one file?"
        );
    }

    #[test]
    fn test_external_ownership_change_locale() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new_with_state(|state| {
            state.intl.ownership = Ownership::External;
        });

        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            None
        );

        intl_service
            .change_locale(ChangeLocaleStrategy::Exact("sl".parse().unwrap()))
            .unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            None
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_core_ownership_change_locale_lookup() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new();

        intl_service
            .change_locale(ChangeLocaleStrategy::Lookup(vec![
                "fr".parse().unwrap(),
                "sl".parse().unwrap(),
            ]))
            .unwrap();

        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            Some("sl".parse().unwrap())
        );
        assert_eq!(
            intl_service.format_message("test.message", &[("count", FormatValue::Integer(1))]),
            // incorrect because of missing plural rules for sl
            "Ali res želite odstraniti 1 datotek?"
        );
    }

    #[test]
    fn test_core_ownership_change_locale_lookup_error() {
        let Fixture {
            store,
            secure_storage_service,
            intl_service,
        } = Fixture::new();

        let lookup_locales = vec!["fr".parse().unwrap(), "de".parse().unwrap()];
        let result =
            intl_service.change_locale(ChangeLocaleStrategy::Lookup(lookup_locales.clone()));

        assert_eq!(result, Err(SetLocaleError::LookupFailed(lookup_locales)));
        assert_eq!(
            store.with_state(|state| state.intl.current_locale.clone()),
            None
        );
        assert_eq!(
            secure_storage_service
                .get::<String>(CURRENT_LOCALE_STORAGE_KEY)
                .unwrap(),
            None
        );
    }
}
