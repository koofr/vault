use std::{collections::HashSet, sync::Arc};

use urlencoding::encode;

use crate::{
    cipher::Cipher,
    common::state::Status,
    remote::{models, RemoteError},
    remote_files::selectors as remote_files_selectors,
    store,
    types::{DecryptedName, RepoId},
};

use super::{
    errors::RepoNotFoundError,
    repo_tree::RepoTree,
    selectors,
    state::{Repo, RepoState},
};

fn vault_repo_to_repo(repo: models::VaultRepo, base_url: &str) -> Repo {
    let models::VaultRepo {
        id,
        name,
        mount_id,
        path,
        salt,
        password_validator,
        password_validator_encrypted,
        added,
    } = repo;

    let web_url = format!(
        "{}/app/storage/{}?path={}",
        base_url,
        &mount_id.0,
        encode(&path.0)
    );

    Repo {
        id,
        name: DecryptedName(name),
        mount_id,
        path,
        salt,
        added,
        password_validator,
        password_validator_encrypted,
        state: RepoState::Locked,
        web_url,
    }
}

pub fn repo_loaded(state: &mut store::State, notify: &store::Notify, repo: models::VaultRepo) {
    notify(store::Event::Repos);

    let mut repo = vault_repo_to_repo(repo, &state.config.base_url);

    if let Some(existing) = state.repos.repos_by_id.get(&repo.id) {
        repo.state = existing.state.clone();
    }

    state.repos.repo_ids_by_remote_file_id.insert(
        remote_files_selectors::get_file_id(&repo.mount_id, &repo.path.to_lowercase()),
        repo.id.clone(),
    );

    let repo_tree = state
        .repos
        .mount_repo_trees
        .entry(repo.mount_id.clone())
        .or_insert_with(|| RepoTree::new());

    repo_tree.set(&repo.path, repo.id.clone());

    state.repos.repos_by_id.insert(repo.id.clone(), repo);
}

pub fn repos_loading(state: &mut store::State, notify: &store::Notify) {
    notify(store::Event::Repos);

    state.repos.status = Status::Loading {
        loaded: state.repos.status.loaded(),
    };
}

pub fn repos_loaded(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    res: Result<Vec<models::VaultRepo>, RemoteError>,
) {
    notify(store::Event::Repos);

    match res {
        Ok(repos) => {
            state.repos.status = Status::Loaded;

            let remove_repo_ids = {
                let new_repo_ids = repos.iter().map(|repo| &repo.id).collect::<HashSet<_>>();

                state
                    .repos
                    .repos_by_id
                    .iter()
                    .filter_map(|(repo_id, _)| {
                        if new_repo_ids.contains(repo_id) {
                            None
                        } else {
                            Some(repo_id.to_owned())
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for repo in repos {
                repo_loaded(state, notify, repo);
            }

            remove_repos(
                state,
                notify,
                mutation_state,
                mutation_notify,
                &remove_repo_ids,
            );
        }
        Err(err) => {
            state.repos.status = Status::Error {
                error: err,
                loaded: state.repos.status.loaded(),
            };
        }
    }
}

pub fn lock_repo(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_id: &RepoId,
    cipher: Arc<Cipher>,
) -> Result<(), RepoNotFoundError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    if matches!(repo.state, RepoState::Locked) {
        return Ok(());
    }

    notify(store::Event::Repos);

    repo.state = RepoState::Locked;

    mutation_state
        .repos
        .locked_repos
        .push((repo_id.to_owned(), cipher));

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);

    Ok(())
}

pub fn unlock_repo(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_id: &RepoId,
    cipher: Arc<Cipher>,
) -> Result<(), RepoNotFoundError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    notify(store::Event::Repos);

    if matches!(repo.state, RepoState::Unlocked) {
        return Ok(());
    }

    repo.state = RepoState::Unlocked;

    mutation_state
        .repos
        .unlocked_repos
        .push((repo_id.to_owned(), cipher));

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);

    Ok(())
}

pub fn remove_repos(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_ids: &[RepoId],
) {
    let mut removed = false;

    for repo_id in repo_ids {
        if let Some(repo) = state.repos.repos_by_id.remove(repo_id) {
            state
                .repos
                .repo_ids_by_remote_file_id
                .remove(&remote_files_selectors::get_file_id(
                    &repo.mount_id,
                    &repo.path.to_lowercase(),
                ));

            if let Some(repo_tree) = state.repos.mount_repo_trees.get_mut(&repo.mount_id) {
                repo_tree.remove(&repo.path);
            }

            mutation_state.repos.removed_repos.push(repo_id.to_owned());

            removed = true;
        }
    }

    if removed {
        notify(store::Event::Repos);

        mutation_notify(store::MutationEvent::Repos, state, mutation_state);
    }
}
