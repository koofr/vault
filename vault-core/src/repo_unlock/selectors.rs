use crate::{repos::selectors as repos_selectors, store};

use super::state::RepoUnlockInfo;

pub fn select_info<'a>(state: &'a store::State, unlock_id: u32) -> Option<RepoUnlockInfo<'a>> {
    state
        .repo_unlocks
        .unlocks
        .get(&unlock_id)
        .map(|repo_unlock| RepoUnlockInfo {
            repo_id: &repo_unlock.repo_id,
            status: (&repo_unlock.status).into(),
            repo_name: repos_selectors::select_repo(state, &repo_unlock.repo_id)
                .ok()
                .map(|repo| repo.name.as_str()),
        })
}
