use crate::{repo_unlock::state::RepoUnlockInfo, repos::selectors as repos_selectors, store};

use super::state::RepoConfigBackupInfo;

pub fn select_info<'a>(
    state: &'a store::State,
    backup_id: u32,
) -> Option<RepoConfigBackupInfo<'a>> {
    state
        .repo_config_backups
        .backups
        .get(&backup_id)
        .map(|backup| RepoConfigBackupInfo {
            unlock_info: RepoUnlockInfo {
                repo_id: &backup.repo_id,
                status: (&backup.status).into(),
                repo_name: repos_selectors::select_repo_name(state, &backup.repo_id),
            },
            config: backup.config.as_ref(),
        })
}
