use crate::{common::state::Status, remote::RemoteError};

#[derive(Debug, Clone)]
pub enum SpaceUsageSeverity {
    Normal,
    Warn,
    Critical,
}

#[derive(Debug, Clone)]
pub struct SpaceUsage {
    pub used: i64,
    pub total: i64,
    pub percentage: u8,
    pub severity: SpaceUsageSeverity,
}

#[derive(Debug, Clone, Default)]
pub struct SpaceUsageState {
    pub status: Status<RemoteError>,
    pub space_usage: Option<SpaceUsage>,
}
