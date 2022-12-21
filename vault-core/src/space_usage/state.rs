use crate::{common::state::Status, remote::RemoteError};

#[derive(Clone)]
pub enum SpaceUsageSeverity {
    Normal,
    Warn,
    Critical,
}

#[derive(Clone)]
pub struct SpaceUsage {
    pub used: i64,
    pub total: i64,
    pub percentage: u8,
    pub severity: SpaceUsageSeverity,
}

#[derive(Clone, Default)]
pub struct SpaceUsageState {
    pub status: Status<RemoteError>,
    pub space_usage: Option<SpaceUsage>,
}
