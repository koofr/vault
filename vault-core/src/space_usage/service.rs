use std::{cmp, sync::Arc};

use crate::{common::state::Status, remote, store, types::MountId};

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
        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::SpaceUsage);

            state.user.status = Status::Loading {
                loaded: state.user.status.loaded(),
            };
        });

        let mount = match self.remote.get_mount(&MountId("primary".into())).await {
            Ok(mount) => mount,
            Err(err) => {
                self.store.mutate(|state, notify, _, _| {
                    notify(store::Event::SpaceUsage);

                    state.user.status = Status::Error {
                        error: err.clone(),
                        loaded: state.user.status.loaded(),
                    };
                });

                return Err(err);
            }
        };

        self.store.mutate(|state, notify, _, _| {
            notify(store::Event::SpaceUsage);

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
