use std::sync::Arc;

use futures::{future, future::BoxFuture, FutureExt};

use crate::{
    dir_pickers::{selectors as dir_pickers_selectors, state::DirPickerItem, DirPickersHelper},
    remote::RemoteError,
    remote_files::{
        errors::RemoteFilesErrors,
        selectors as remote_files_selectors,
        state::{MountType, RemoteFilesLocation},
        RemoteFilesService,
    },
    store,
    utils::path_utils,
};

use super::{selectors, state::Options};

pub struct RemoteFilesDirPickersService {
    remote_files_service: Arc<RemoteFilesService>,
    helper: Arc<DirPickersHelper<RemoteError>>,
    store: Arc<store::Store>,
}

impl RemoteFilesDirPickersService {
    pub fn new(remote_files_service: Arc<RemoteFilesService>, store: Arc<store::Store>) -> Self {
        let on_expand_remote_files_service = remote_files_service.clone();
        let on_expand_store = store.clone();

        let helper = Arc::new(DirPickersHelper::<RemoteError>::new(
            store.clone(),
            Box::new(selectors::select_items),
            Box::new(move |_, item_id| {
                let on_expand_remote_files_service = on_expand_remote_files_service.clone();
                let on_expand_store = on_expand_store.clone();

                Box::pin(async move {
                    on_expand(item_id, on_expand_remote_files_service, on_expand_store).await
                })
            }),
        ));

        Self {
            remote_files_service,
            helper,
            store,
        }
    }

    pub fn create(&self, options: Options) -> u32 {
        self.helper
            .clone()
            .create(&[store::Event::RemoteFiles], options)
    }

    pub fn destroy(&self, picker_id: u32) {
        self.helper.destroy(picker_id);
    }

    pub async fn load(&self, picker_id: u32) -> Result<(), RemoteError> {
        let only_hosted_devices = self
            .store
            .with_state(|state| {
                dir_pickers_selectors::select_picker(state, picker_id)
                    .map(|picker| selectors::get_options(picker).only_hosted_devices)
            })
            .unwrap_or(false);

        let load_places_future = self.remote_files_service.load_places();
        let load_bookmarks_future = self.remote_files_service.load_bookmarks();
        let load_shared_future = if only_hosted_devices {
            future::ready(Ok(())).boxed()
        } else {
            self.remote_files_service.load_shared().boxed()
        };

        load_places_future.await?;
        load_bookmarks_future.await?;
        load_shared_future.await?;

        Ok(())
    }

    pub async fn click(
        &self,
        picker_id: u32,
        item_id: &str,
        is_arrow: bool,
    ) -> Result<(), RemoteError> {
        self.helper.click(picker_id, item_id, is_arrow).await
    }

    pub async fn select_file(
        &self,
        picker_id: u32,
        location: &RemoteFilesLocation,
    ) -> Result<(), RemoteError> {
        let (mount_id, mount_type, path) = self
            .store
            .with_state(|state| {
                remote_files_selectors::select_file(
                    state,
                    &remote_files_selectors::get_file_id(&location.mount_id, &location.path),
                )
                .and_then(|file| {
                    let mount = remote_files_selectors::select_mount(state, &file.mount_id)?;

                    Some((mount.id.clone(), mount.typ.clone(), file.path.clone()))
                })
            })
            .unwrap_or_else(|| {
                (
                    location.mount_id.to_owned(),
                    MountType::Device,
                    location.path.to_owned(),
                )
            });

        let id_getter = match mount_type {
            MountType::Import | MountType::Export => {
                self.helper
                    .expand(picker_id, selectors::SHARED_ITEM_ID, false)
                    .await?;

                selectors::get_shared_id
            }
            _ => selectors::get_places_id,
        };

        let paths_chain = path_utils::paths_chain(&path);

        let mut expand_futures = Vec::<BoxFuture<Result<(), RemoteError>>>::new();

        for (idx, path) in paths_chain.iter().enumerate() {
            let item_id = id_getter(&remote_files_selectors::get_file_id(&mount_id, &path));

            if idx == paths_chain.len() - 1 {
                self.helper.set_selected(picker_id, &item_id);
            }

            expand_futures.push(self.helper.expand(picker_id, &item_id, false));
        }

        for expand_future in expand_futures {
            expand_future.await?;
        }

        Ok(())
    }

    pub async fn create_dir(&self, picker_id: u32, name: &str) -> Result<(), RemoteError> {
        let (mount_id, parent_path) =
            self.store
                .with_state::<_, Result<_, RemoteError>>(|state| {
                    selectors::select_check_create_dir(state, picker_id, name)?;

                    let parent_file = selectors::select_selected_file(state, picker_id)
                        .ok_or_else(RemoteFilesErrors::not_found)?;

                    Ok((parent_file.mount_id.clone(), parent_file.path.clone()))
                })?;

        self.remote_files_service
            .create_dir(&mount_id, &parent_path, name)
            .await?;

        let new_path = path_utils::join_path_name(&parent_path, name);

        self.select_file(
            picker_id,
            &RemoteFilesLocation {
                mount_id,
                path: new_path,
            },
        )
        .await?;

        Ok(())
    }
}

pub async fn on_expand(
    item: DirPickerItem,
    remote_files_service: Arc<RemoteFilesService>,
    store: Arc<store::Store>,
) -> Result<(), RemoteError> {
    if item.id == selectors::BOOKMARKS_ITEM_ID {
        remote_files_service.load_bookmarks().await?;
    } else if item.id == selectors::SHARED_ITEM_ID {
        remote_files_service.load_shared().await?;
    } else if let Some(file_id) = &item.file_id {
        if let Some((mount_id, path)) = store.with_state(|state| {
            remote_files_selectors::select_file(state, file_id)
                .map(|file| (file.mount_id.clone(), file.path.clone()))
        }) {
            remote_files_service.load_files(&mount_id, &path).await?;
        }
    }

    Ok(())
}
