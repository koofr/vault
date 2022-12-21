use crate::{repos::selectors as repos_selectors, store};

use super::state::RepoRemoveInfo;

pub fn select_info<'a>(state: &'a store::State) -> Option<RepoRemoveInfo<'a>> {
    state
        .repo_remove
        .as_ref()
        .map(|repo_remove| RepoRemoveInfo {
            repo_id: &repo_remove.repo_id,
            status: (&repo_remove.status).into(),
            repo_name: repos_selectors::select_repo(state, &repo_remove.repo_id)
                .ok()
                .map(|repo| repo.name.as_str()),
        })
}
