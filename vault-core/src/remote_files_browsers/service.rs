use std::sync::Arc;

use futures::{
    future::{self, BoxFuture},
    join, FutureExt,
};

use crate::{
    remote,
    remote_files::{
        errors::{CreateDirError, RemoteFilesErrors},
        state::RemoteFilesSortField,
        RemoteFilesService,
    },
    sort::state::SortDirection,
    store,
};

use super::{
    mutations, selectors,
    state::{RemoteFilesBrowserItemId, RemoteFilesBrowserLocation, RemoteFilesBrowserOptions},
};

pub struct RemoteFilesBrowsersService {
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
    remote_files_mutation_subscription_id: u32,
}

impl RemoteFilesBrowsersService {
    pub fn new(remote_files_service: Arc<RemoteFilesService>, store: Arc<store::Store>) -> Self {
        let remote_files_mutation_subscription_id = store.get_next_id();

        store.mutation_on(
            remote_files_mutation_subscription_id,
            &[store::MutationEvent::RemoteFiles],
            Box::new(|state, notify, _, _| {
                mutations::handle_remote_files_mutation(state, notify);
            }),
        );

        Self {
            remote_files_service,
            store,
            remote_files_mutation_subscription_id,
        }
    }

    pub fn create(
        self: Arc<Self>,
        location: &RemoteFilesBrowserItemId,
        options: RemoteFilesBrowserOptions,
    ) -> (u32, BoxFuture<'static, Result<(), remote::RemoteError>>) {
        let browser_id = self.store.mutate(|state, notify, mutation_state, _| {
            mutations::create(state, notify, mutation_state, options, location)
        });

        let load_self = self.clone();

        let load_future = async move { load_self.load(browser_id).await }.boxed();

        (browser_id, load_future)
    }

    pub fn destroy(&self, browser_id: u32) {
        self.store.mutate(|state, notify, mutation_state, _| {
            mutations::destroy(state, notify, mutation_state, browser_id);
        });
    }

    pub async fn load(&self, browser_id: u32) -> Result<(), remote::RemoteError> {
        let (location, options) = match self.store.with_state(|state| {
            selectors::select_browser(state, browser_id)
                .map(|browser| (browser.location.clone(), browser.options.clone()))
        }) {
            Some((Some(location), options)) => (location, options),
            _ => return Ok(()),
        };

        let res = match &location {
            RemoteFilesBrowserLocation::Home => {
                let load_places_future = self.remote_files_service.load_places();
                let load_bookmarks_future = self.remote_files_service.load_bookmarks();
                let load_shared_future = if options.only_hosted_devices {
                    future::ready(Ok(())).boxed()
                } else {
                    self.remote_files_service.load_shared().boxed()
                };

                let (load_places_res, load_bookmarks_res, load_shared_res) = join!(
                    load_places_future,
                    load_bookmarks_future,
                    load_shared_future
                );

                load_places_res.or(load_bookmarks_res).or(load_shared_res)
            }
            RemoteFilesBrowserLocation::Bookmarks => {
                self.remote_files_service.load_bookmarks().await
            }
            RemoteFilesBrowserLocation::Files(location) => {
                self.remote_files_service
                    .load_files(&location.mount_id, &location.path)
                    .await
            }
            RemoteFilesBrowserLocation::Shared => self.remote_files_service.load_shared().await,
        };

        self.store.mutate(|state, notify, _, _| {
            mutations::loaded(state, notify, browser_id, &location, res.clone());
        });

        res
    }

    pub fn select_item(
        &self,
        browser_id: u32,
        item_id: RemoteFilesBrowserItemId,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.store.mutate(|state, notify, _, _| {
            mutations::select_item(state, notify, browser_id, item_id, extend, range, force);
        });
    }

    pub fn select_all(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::select_all(state, notify, browser_id);
        });
    }

    pub fn clear_selection(&self, browser_id: u32) {
        self.store.mutate(|state, notify, _, _| {
            mutations::clear_selection(state, notify, browser_id);
        });
    }

    pub fn set_selection(&self, browser_id: u32, selection: Vec<RemoteFilesBrowserItemId>) {
        self.store.mutate(|state, notify, _, _| {
            mutations::set_selection(state, notify, browser_id, selection);
        });
    }

    pub fn sort_by(
        &self,
        browser_id: u32,
        field: RemoteFilesSortField,
        direction: Option<SortDirection>,
    ) {
        self.store.mutate(|state, notify, _, _| {
            mutations::sort_by(state, notify, browser_id, field, direction);
        });
    }

    pub async fn create_dir(
        &self,
        browser_id: u32,
    ) -> Result<RemoteFilesBrowserItemId, CreateDirError> {
        let (item_id_prefix, mount_id, parent_path) = self.store.with_state(|state| {
            match selectors::select_browser_location(state, browser_id) {
                Some(RemoteFilesBrowserLocation::Files(location)) => Ok((
                    location.item_id_prefix.clone(),
                    location.mount_id.clone(),
                    location.path.clone(),
                )),
                _ => Err(RemoteFilesErrors::invalid_path()),
            }
        })?;

        let (_, path) = self
            .remote_files_service
            .create_dir(&mount_id, &parent_path)
            .await?;

        Ok(selectors::get_file_item_id(
            &item_id_prefix,
            &mount_id,
            &path,
        ))
    }
}

impl Drop for RemoteFilesBrowsersService {
    fn drop(&mut self) {
        self.store
            .mutation_remove_listener(self.remote_files_mutation_subscription_id)
    }
}
