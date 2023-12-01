use crate::{
    lifecycle::state::AppVisibility,
    repos::{self, state::RepoState},
    store,
};

pub fn handle_lifecycle_mutation(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
) {
    let app_visibility = &state.lifecycle.app_visibility;

    let mut lock_repo_ids = Vec::new();

    if matches!(app_visibility, AppVisibility::Hidden) {
        let default_auto_lock = repos::selectors::select_default_auto_lock(state);

        for repo in state.repos.repos_by_id.values() {
            if matches!(&repo.state, RepoState::Unlocked { .. }) {
                let auto_lock = repo.auto_lock.as_ref().unwrap_or(default_auto_lock);

                if auto_lock.on_app_hidden {
                    lock_repo_ids.push(repo.id.clone());
                }
            }
        }
    }

    for repo_id in lock_repo_ids {
        let _ =
            repos::mutations::lock_repo(state, notify, mutation_state, mutation_notify, &repo_id);
    }
}
