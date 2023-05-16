use std::{cmp, sync::Arc};

use crate::{common::state::Status, remote, store};

use super::state::{SpaceUsage, SpaceUsageSeverity};

pub struct SpaceUsageService {
    remote: Arc<remote::Remote>,
    store: Arc<store::Store>,
}

impl SpaceUsageService {
    pub fn new(remote: Arc<remote::Remote>, store: Arc<store::Store>) -> Self {
        Self { remote, store }
    }

    pub async fn load(&self) -> Result<(), remote::RemoteError> {
        self.store.mutate(store::Event::SpaceUsage, |state| {
            state.user.status = Status::Loading;
        });

        let mount = match self.remote.get_mount("primary").await {
            Ok(mount) => mount,
            Err(err) => {
                self.store.mutate(store::Event::SpaceUsage, |state| {
                    state.user.status = Status::Error { error: err.clone() };
                });

                return Err(err);
            }
        };

        self.store.mutate(store::Event::SpaceUsage, |state| {
            state.space_usage.space_usage = match (mount.space_used, mount.space_total) {
                (Some(used), Some(total)) => {
                    let used = used * 1024 * 1024;
                    let total = total * 1024 * 1024;

                    let percentage = if total > 0 {
                        cmp::min(((used as f64 * 100.0) / total as f64).floor() as u8, 100)
                    } else {
                        0
                    };

                    Some(SpaceUsage {
                        used,
                        total,
                        percentage,
                        severity: if percentage > 95 {
                            SpaceUsageSeverity::Critical
                        } else if percentage > 80 {
                            SpaceUsageSeverity::Warn
                        } else {
                            SpaceUsageSeverity::Normal
                        },
                    })
                }
                _ => None,
            };

            state.space_usage.status = Status::Loaded;
        });

        Ok(())
    }
}
