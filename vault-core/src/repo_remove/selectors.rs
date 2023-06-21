use crate::{repos::selectors as repos_selectors, store};

use super::state::RepoRemoveInfo;

pub fn select_info<'a>(state: &'a store::State, remove_id: u32) -> Option<RepoRemoveInfo<'a>> {
    state
        .repo_removes
        .removes
        .get(&remove_id)
        .map(|repo_remove| RepoRemoveInfo {
            repo_id: &repo_remove.repo_id,
            status: (&repo_remove.status).into(),
            repo_name: repos_selectors::select_repo_name(state, &repo_remove.repo_id),
        })
}
