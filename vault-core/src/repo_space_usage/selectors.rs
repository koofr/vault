use crate::store;

use super::state::RepoSpaceUsageInfo;

pub fn select_info<'a>(state: &'a store::State) -> Option<RepoSpaceUsageInfo<'a>> {
    state
        .repo_space_usage
        .as_ref()
        .map(|repo_space_usage| RepoSpaceUsageInfo {
            repo_id: &repo_space_usage.repo_id,
            status: (&repo_space_usage.status).into(),
            space_used: repo_space_usage.space_used,
        })
}
