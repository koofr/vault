use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use futures::stream::{AbortHandle, Abortable};

use crate::{cipher::Cipher, repos::ReposService, runtime, store, types::RepoId};

use super::{mutations, selectors};

pub struct RepoLockerService {
    repos_service: Arc<ReposService>,
    store: Arc<store::Store>,
    runtime: Arc<runtime::BoxRuntime>,

    lifecycle_mutation_subscription_id: u32,
    repos_subscription_id: u32,
    abort_handles: Arc<Mutex<HashMap<RepoId, AbortHandle>>>,
}

impl RepoLockerService {
    pub fn new(
        repos_service: Arc<ReposService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
    ) -> Arc<Self> {
        let lifecycle_mutation_subscription_id = store.get_next_id();

        store.mutation_on(
            lifecycle_mutation_subscription_id,
            &[store::MutationEvent::Lifecycle],
            Box::new(move |state, notify, mutation_state, mutation_notify| {
                mutations::handle_lifecycle_mutation(
                    state,
                    notify,
                    mutation_state,
                    mutation_notify,
                );
            }),
        );

        let repos_subscription_id = store.get_next_id();

        let repo_locker_service = Arc::new(Self {
            repos_service,
            store: store.clone(),
            runtime,

            repos_subscription_id,
            lifecycle_mutation_subscription_id,
            abort_handles: Default::default(),
        });

        let repos_subscription_repo_locker_service = Arc::downgrade(&repo_locker_service);

        store.on(
            repos_subscription_id,
            &[store::Event::Repos],
            Box::new(move |mutation_state, add_side_effect| {
                if !mutation_state.repos.unlocked_repos.is_empty() {
                    if let Some(repo_locker_service) =
                        repos_subscription_repo_locker_service.upgrade()
                    {
                        let unlocked_repos = mutation_state.repos.unlocked_repos.clone();

                        add_side_effect(Box::new(move || {
                            repo_locker_service.handle_unlocked_repos(unlocked_repos);
                        }));
                    }
                }

                if !mutation_state.repos.locked_repos.is_empty() {
                    if let Some(repo_locker_service) =
                        repos_subscription_repo_locker_service.upgrade()
                    {
                        let locked_repos = mutation_state.repos.locked_repos.clone();

                        add_side_effect(Box::new(move || {
                            repo_locker_service.handle_locked_repos(locked_repos);
                        }));
                    }
                }
            }),
        );

        repo_locker_service
    }

    fn handle_unlocked_repos(&self, unlocked_repos: Vec<(RepoId, Arc<Cipher>)>) {
        for (repo_id, _) in unlocked_repos {
            self.start_locker(repo_id.clone());
        }
    }

    fn handle_locked_repos(&self, locked_repos: Vec<(RepoId, Arc<Cipher>)>) {
        for (repo_id, _) in locked_repos {
            self.stop_locker(&repo_id);
        }
    }

    fn start_locker(&self, repo_id: RepoId) {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();

        {
            let mut abort_handles = self.abort_handles.lock().unwrap();

            if abort_handles.contains_key(&repo_id) {
                return;
            }

            abort_handles.insert(repo_id.clone(), abort_handle);
        }

        let lock_check_interval = self
            .store
            .with_state(|state| state.config.repo_locker.lock_check_interval);

        let locker_future = Abortable::new(
            Self::locker(
                self.repos_service.clone(),
                self.store.clone(),
                self.runtime.clone(),
                repo_id,
                lock_check_interval,
            ),
            abort_registration,
        );

        self.runtime.spawn(Box::pin(async move {
            let _ = locker_future.await;
        }));
    }

    fn stop_locker(&self, repo_id: &RepoId) {
        let abort_handle = self.abort_handles.lock().unwrap().remove(repo_id);

        // separate from lock() to prevent deadlocks
        if let Some(abort_handle) = abort_handle {
            abort_handle.abort();
        }
    }

    async fn locker(
        repos_service: Arc<ReposService>,
        store: Arc<store::Store>,
        runtime: Arc<runtime::BoxRuntime>,
        repo_id: RepoId,
        lock_check_interval: Duration,
    ) {
        loop {
            let now = runtime.now();

            if store.with_state(|state| selectors::select_should_auto_lock(state, &repo_id, now)) {
                let _ = repos_service.lock_repo(&repo_id);

                return;
            }

            runtime.sleep(lock_check_interval).await;
        }
    }
}

impl Drop for RepoLockerService {
    fn drop(&mut self) {
        let abort_handles = self
            .abort_handles
            .lock()
            .unwrap()
            .drain()
            .collect::<Vec<_>>();

        // separate from lock() to prevent deadlocks
        for (_, abort_handle) in abort_handles {
            abort_handle.abort();
        }

        self.store
            .mutation_remove_listener(self.lifecycle_mutation_subscription_id);

        self.store.remove_listener(self.repos_subscription_id);
    }
}
