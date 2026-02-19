use vault_intl::{LanguageIdentifier, negotiate_language};

use crate::intl::locales::IntlLocale;

pub enum IntlConfigOwnership {
    Core {
        preferred_locales: Vec<LanguageIdentifier>,
    },
    External,
}

pub struct IntlConfig {
    pub ownership: IntlConfigOwnership,
}

pub enum ChangeLocaleStrategy {
    Exact(LanguageIdentifier),
    Lookup(Vec<LanguageIdentifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Core {
        // preferred_locales is a list of locales in order of preference (e.g. from
        // browser's navigator.languages)
        preferred_locales: Vec<LanguageIdentifier>,
    },
    External,
}

#[derive(Debug, Clone)]
pub enum OwnershipInfo {
    Core,
    External,
}

impl From<&Ownership> for OwnershipInfo {
    fn from(ownership: &Ownership) -> Self {
        match ownership {
            Ownership::Core { .. } => Self::Core,
            Ownership::External => Self::External,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntlState {
    // locales is a list of all available locales
    pub locales: Vec<IntlLocale>,
    // default_locale is always set to a valid locale
    pub default_locale: LanguageIdentifier,
    // current_locale is always a valid locale. catalog_formatter is not
    // created if current_locale matches default_locale, as
    // FallbackFormatter already handles the default locale.
    pub current_locale: Option<LanguageIdentifier>,
    // ownership is used to determine if the state is managed by the core or by
    // externally
    pub ownership: Ownership,
}

impl IntlState {
    pub fn new(config: IntlConfig) -> Self {
        let mut state = Self {
            locales: IntlLocale::default_locales(),
            default_locale: IntlLocale::default_locale(),
            current_locale: None,
            ownership: match config.ownership {
                IntlConfigOwnership::Core { preferred_locales } => {
                    Ownership::Core { preferred_locales }
                }
                IntlConfigOwnership::External => Ownership::External,
            },
        };

        state.init_current_locale();

        state
    }

    fn init_current_locale(&mut self) {
        match &self.ownership {
            Ownership::Core { preferred_locales } => {
                self.current_locale = negotiate_language(
                    preferred_locales.iter().collect(),
                    self.locales.iter().map(|l| &l.locale).collect(),
                )
                .cloned();
            }
            Ownership::External => {}
        }
    }

    pub fn reset(&mut self) {
        self.init_current_locale()
    }
}

impl Default for IntlState {
    fn default() -> Self {
        Self {
            locales: IntlLocale::default_locales(),
            default_locale: IntlLocale::default_locale(),
            current_locale: None,
            ownership: Ownership::Core {
                preferred_locales: Vec::new(),
            },
        }
    }
}
