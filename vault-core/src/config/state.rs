use std::time::Duration;

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
    pub transfers: TransfersConfig,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            base_url: String::from(""),
            transfers: TransfersConfig::default(),
        }
    }
}
