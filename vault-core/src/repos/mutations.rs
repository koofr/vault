use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use urlencoding::encode;

use crate::{
    cipher::Cipher,
    common::state::Status,
    remote::{models, RemoteError},
    remote_files::selectors as remote_files_selectors,
    store,
    types::{DecryptedName, RepoId, TimeMillis},
};

use super::{
    errors::{
        LockRepoError, RemoveRepoError, RepoLockedError, RepoNotFoundError, RepoUnlockedError,
        UnlockRepoError,
    },
    repo_tree::RepoTree,
    selectors,
    state::{Repo, RepoAutoLock, RepoState},
};

fn vault_repo_to_repo(
    repo: models::VaultRepo,
    base_url: &str,
    auto_lock: Option<RepoAutoLock>,
) -> Repo {
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
        last_activity: None,
        auto_lock,
    }
}

fn repo_loaded(state: &mut store::State, repo: models::VaultRepo, auto_lock: Option<RepoAutoLock>) {
    let mut repo = vault_repo_to_repo(repo, &state.config.base_url, auto_lock);

    if let Some(existing) = state.repos.repos_by_id.get(&repo.id) {
        repo.state = existing.state.clone();
        repo.last_activity = existing.last_activity;
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

pub fn repos_loading(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
) {
    state.repos.status = Status::Loading {
        loaded: state.repos.status.loaded(),
    };

    notify(store::Event::Repos);

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);
}

pub fn repos_loaded(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    res: Result<Vec<models::VaultRepo>, RemoteError>,
    auto_locks: &HashMap<RepoId, RepoAutoLock>,
) {
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
                let auto_lock = auto_locks.get(&repo.id).cloned();

                repo_loaded(state, repo, auto_lock);
            }

            remove_repos(state, mutation_state, &remove_repo_ids);
        }
        Err(err) => {
            state.repos.status = Status::Error {
                error: err,
                loaded: state.repos.status.loaded(),
            };
        }
    }

    notify(store::Event::Repos);

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);
}

pub fn lock_repo(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_id: &RepoId,
) -> Result<(), LockRepoError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    let cipher = match &repo.state {
        RepoState::Locked => return Err(LockRepoError::RepoLocked(RepoLockedError)),
        RepoState::Unlocked { cipher } => cipher.clone(),
    };

    repo.state = RepoState::Locked;

    notify(store::Event::Repos);

    mutation_state
        .repos
        .locked_repos
        .push((repo_id.to_owned(), cipher));

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);

    Ok(())
}

pub fn check_unlock_repo<'a>(
    state: &'a mut store::State,
    repo_id: &RepoId,
) -> Result<&'a mut Repo, UnlockRepoError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    if matches!(repo.state, RepoState::Unlocked { .. }) {
        return Err(UnlockRepoError::RepoUnlocked(RepoUnlockedError));
    }

    Ok(repo)
}

pub fn unlock_repo(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_id: &RepoId,
    cipher: Arc<Cipher>,
    now: TimeMillis,
) -> Result<(), UnlockRepoError> {
    let repo = check_unlock_repo(state, repo_id)?;

    repo.state = RepoState::Unlocked {
        cipher: cipher.clone(),
    };
    repo.last_activity = Some(now);

    notify(store::Event::Repos);

    mutation_state
        .repos
        .unlocked_repos
        .push((repo_id.to_owned(), cipher));

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);

    Ok(())
}

pub fn remove_repos(
    state: &mut store::State,
    mutation_state: &mut store::MutationState,
    repo_ids: &[RepoId],
) -> Vec<Option<Repo>> {
    repo_ids
        .iter()
        .map(|repo_id| match state.repos.repos_by_id.remove(repo_id) {
            Some(repo) => {
                state.repos.repo_ids_by_remote_file_id.remove(
                    &remote_files_selectors::get_file_id(&repo.mount_id, &repo.path.to_lowercase()),
                );

                if let Some(repo_tree) = state.repos.mount_repo_trees.get_mut(&repo.mount_id) {
                    repo_tree.remove(&repo.path);
                }

                mutation_state.repos.removed_repos.push(repo_id.to_owned());

                Some(repo)
            }
            None => None,
        })
        .collect()
}

pub fn repo_created(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo: models::VaultRepo,
) {
    repo_loaded(state, repo, None);

    notify(store::Event::Repos);

    mutation_notify(store::MutationEvent::Repos, state, mutation_state);
}

pub fn repo_removed(
    state: &mut store::State,
    notify: &store::Notify,
    mutation_state: &mut store::MutationState,
    mutation_notify: &store::MutationNotify,
    repo_id: RepoId,
) -> Result<(), RemoveRepoError> {
    let res = remove_repos(state, mutation_state, &[repo_id]);

    match res.into_iter().next().flatten() {
        Some(_) => {
            notify(store::Event::Repos);

            mutation_notify(store::MutationEvent::Repos, state, mutation_state);

            Ok(())
        }
        None => Err(RemoveRepoError::RepoNotFound(RepoNotFoundError)),
    }
}

pub fn touch_repo(
    state: &mut store::State,
    repo_id: &RepoId,
    now: TimeMillis,
) -> Result<(), RepoNotFoundError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    repo.last_activity = Some(now);

    // notify is not called because last_activity is only checked by the repo
    // locker and the repo locker does not need to be notified because it uses
    // polling

    Ok(())
}

pub fn set_auto_lock(
    state: &mut store::State,
    notify: &store::Notify,
    repo_id: &RepoId,
    auto_lock: RepoAutoLock,
) -> Result<(), RepoNotFoundError> {
    let repo = selectors::select_repo_mut(state, repo_id)?;

    notify(store::Event::Repos);

    repo.auto_lock = Some(auto_lock);

    Ok(())
}
