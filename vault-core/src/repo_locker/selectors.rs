use std::time::Duration;

use crate::{
    repos, store,
    types::{RepoId, TimeMillis},
};

pub fn select_should_auto_lock(state: &store::State, repo_id: &RepoId, now: TimeMillis) -> bool {
    let repo = match repos::selectors::select_repo(state, repo_id) {
        Ok(repo) => repo,
        Err(_) => return false,
    };

    if let Some(last_activity) = repo.last_activity {
        let auto_lock = repo
            .auto_lock
            .as_ref()
            .unwrap_or_else(|| repos::selectors::select_default_auto_lock(state));

        if let Some(duration) = auto_lock.after.map(Into::<Duration>::into) {
            if now > last_activity + duration {
                return true;
            }
        }
    }

    false
}
