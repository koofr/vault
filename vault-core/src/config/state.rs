use std::time::Duration;

use crate::locale::{get_locale, BoxLocale};

#[derive(Debug, Clone)]
pub struct EventstreamConfig {
    pub reconnect_duration: Duration,
    pub ping_interval: Duration,
}

impl Default for EventstreamConfig {
    fn default() -> Self {
        Self {
            reconnect_duration: Duration::from_secs(3),
            ping_interval: Duration::from_secs(30),
        }
    }
}

pub struct LocaleConfig {
    pub name: String,
    pub locale: BoxLocale,
}

impl std::fmt::Debug for LocaleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocaleConfig")
            .field("name", &self.name)
            .finish()
    }
}

impl Clone for LocaleConfig {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            locale: get_locale(&self.name).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransfersConfig {
    pub upload_concurrency: usize,
    pub download_concurrency: usize,
    pub autoretry_attempts: usize,
    pub min_time_per_file: Duration,
    pub progress_throttle: Duration,
}

impl Default for TransfersConfig {
    fn default() -> Self {
        Self {
            upload_concurrency: 3,
            download_concurrency: 3,
            autoretry_attempts: 5,
            min_time_per_file: Duration::from_millis(500),
            progress_throttle: Duration::from_millis(100),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigState {
    pub base_url: String,
    pub locale: LocaleConfig,
    pub transfers: TransfersConfig,
    pub eventstream: EventstreamConfig,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            base_url: String::from(""),
            locale: LocaleConfig {
                name: String::from("en"),
                locale: get_locale("en").unwrap(),
            },
            transfers: TransfersConfig::default(),
            eventstream: EventstreamConfig::default(),
        }
    }
}
