use crate::store;

use super::state::RepoConfigBackupInfo;

pub fn select_info<'a>(state: &'a store::State) -> Option<RepoConfigBackupInfo<'a>> {
    state
        .repo_config_backup
        .as_ref()
        .map(|repo_config_backup| RepoConfigBackupInfo {
            repo_id: &repo_config_backup.repo_id,
            status: (&repo_config_backup.status).into(),
            config: repo_config_backup.config.as_ref(),
        })
}
