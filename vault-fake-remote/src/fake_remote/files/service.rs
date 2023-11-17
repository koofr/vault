use std::{
    collections::HashMap,
    ops::RangeInclusive,
    pin::Pin,
    sync::{Arc, RwLock},
};

use futures::AsyncRead;
use http::StatusCode;
use vault_core::{
    remote::models,
    types::{MountId, RemoteName, RemotePath},
};

use crate::fake_remote::{
    context::Context,
    errors::{ApiErrorCode, FakeRemoteError},
    eventstream,
    state::FakeRemoteState,
    utils::now_ms,
    vault_repos_service::VaultReposRemoveService,
};

use super::{
    filesystem::{CreateFileConflictResolution, MoveFileConditions},
    objects::object_provider::{BoxObjectProvider, ObjectProviderError},
    Filesystem, FilesystemFile, Name, NormalizedPath, Path,
};

pub struct FilesService {
    state: Arc<RwLock<FakeRemoteState>>,
    vault_repos_remove_service: Arc<VaultReposRemoveService>,
    eventstream_listeners: Arc<eventstream::Listeners>,
    object_provider: Arc<BoxObjectProvider>,
}

impl FilesService {
    pub fn new(
        state: Arc<RwLock<FakeRemoteState>>,
        vault_repos_remove_service: Arc<VaultReposRemoveService>,
        eventstream_listeners: Arc<eventstream::Listeners>,
        object_provider: Arc<BoxObjectProvider>,
    ) -> Self {
        Self {
            state,
            vault_repos_remove_service,
            eventstream_listeners,
            object_provider,
        }
    }

    pub fn get_filesystem<'a>(
        &self,
        state: &'a FakeRemoteState,
        mount_id: &str,
    ) -> Result<&'a Filesystem, FakeRemoteError> {
        state
            .filesystems
            .get(mount_id)
            .ok_or(FakeRemoteError::ApiError(
                StatusCode::NOT_FOUND,
                ApiErrorCode::NotFound,
                "Mount not found".into(),
                None,
            ))
    }

    pub fn get_filesystem_mut<'a>(
        &self,
        state: &'a mut FakeRemoteState,
        mount_id: &str,
    ) -> Result<&'a mut Filesystem, FakeRemoteError> {
        state
            .filesystems
            .get_mut(mount_id)
            .ok_or(FakeRemoteError::ApiError(
                StatusCode::NOT_FOUND,
                ApiErrorCode::NotFound,
                "Mount not found".into(),
                None,
            ))
    }

    pub fn create_filesystem(&self) -> Filesystem {
        Filesystem {
            files: [(
                NormalizedPath::root(),
                FilesystemFile {
                    file: models::FilesFile {
                        name: RemoteName("".into()),
                        typ: "dir".into(),
                        modified: now_ms(),
                        size: 0,
                        content_type: "".into(),
                        hash: None,
                        tags: HashMap::new(),
                    },
                    children: vec![],
                    object_id: None,
                },
            )]
            .into(),
            objects: HashMap::new(),
        }
    }

    pub fn bundle(&self, mount_id: &str, path: &Path) -> Result<models::Bundle, FakeRemoteError> {
        let state = self.state.read().unwrap();

        let fs = self.get_filesystem(&state, &mount_id)?;

        let file = fs.get_file(&path.normalize())?;

        let bundle_file = self.files_file_to_bundle_file(&file.file);

        let bundle_files = match file.file.typ.as_str() {
            "dir" => Some(
                fs.get_children(file)
                    .into_iter()
                    .map(|file| self.files_file_to_bundle_file(&file.file))
                    .collect(),
            ),
            _ => None,
        };

        Ok(models::Bundle {
            file: bundle_file,
            files: bundle_files,
        })
    }

    fn files_file_to_bundle_file(&self, file: &models::FilesFile) -> models::BundleFile {
        models::BundleFile {
            name: file.name.clone(),
            typ: file.typ.clone(),
            modified: file.modified.clone(),
            size: file.size.clone(),
            content_type: file.content_type.clone(),
            hash: file.hash.clone(),
            tags: file.tags.clone(),
        }
    }

    pub fn info(&self, mount_id: &str, path: &Path) -> Result<models::FilesFile, FakeRemoteError> {
        let state = self.state.read().unwrap();

        let fs = self.get_filesystem(&state, &mount_id)?;

        let file = fs.get_file(&path.normalize())?;

        Ok(file.file.clone())
    }

    pub async fn create_file(
        &self,
        context: &Context,
        mount_id: &str,
        parent_path: &Path,
        name: Name,
        modified: Option<i64>,
        conflict_resolution: &CreateFileConflictResolution,
        reader: Pin<Box<dyn AsyncRead + Send + Sync + 'static>>,
    ) -> Result<models::FilesFile, FakeRemoteError> {
        let mount_id = {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

            fs.create_file_check(parent_path, &name, &conflict_resolution)?;

            mount_id
        };

        let object_id = uuid::Uuid::new_v4().to_string();

        let (size, hash) = self.object_provider.put(object_id.clone(), reader).await?;

        let file = {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

            fs.create_file(
                &parent_path,
                name,
                modified.unwrap_or(now_ms()),
                size as i64,
                hash,
                HashMap::new(),
                &conflict_resolution,
                object_id,
            )?
        };

        self.cleanup_objects(mount_id).await?;

        self.eventstream_listeners
            .process_event(eventstream::Event::FileCreatedEvent {
                mount_id: MountId(mount_id.to_owned()),
                path: RemotePath(parent_path.join_name(&Name(file.name.0.clone())).0),
                file: file.clone(),
                user_agent: context.user_agent.clone(),
            })
            .await;

        Ok(file)
    }

    pub fn get_file(
        &self,
        mount_id: &str,
        path: &Path,
    ) -> Result<(models::FilesFile, String), FakeRemoteError> {
        let state = self.state.read().unwrap();

        let fs = self.get_filesystem(&state, &mount_id)?;

        let file = fs.get_file(&path.normalize())?;

        let object_id = file.object_id.clone().ok_or_else(|| {
            FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::NotFile,
                "Not a file".into(),
                None,
            )
        })?;

        Ok((file.file.clone(), object_id))
    }

    pub async fn get_object_reader(
        &self,
        object_id: String,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<Pin<Box<dyn AsyncRead + Send + Sync + 'static>>, FakeRemoteError> {
        Ok(self.object_provider.get(object_id, range).await?)
    }

    pub async fn create_dir(
        &self,
        context: &Context,
        mount_id: &str,
        parent_path: &Path,
        name: Name,
    ) -> Result<(), FakeRemoteError> {
        let file = {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

            fs.create_dir(parent_path, name.clone(), now_ms())?
        };

        self.eventstream_listeners
            .process_event(eventstream::Event::FileCreatedEvent {
                mount_id: MountId(mount_id.to_owned()),
                path: RemotePath(parent_path.join_name(&name).0),
                file: file.clone(),
                user_agent: context.user_agent.clone(),
            })
            .await;

        Ok(())
    }

    pub async fn delete_file(
        &self,
        context: &Context,
        mount_id: &str,
        path: &Path,
        delete_if_empty: bool,
    ) -> Result<(), FakeRemoteError> {
        let file = {
            let (file, repo_ids) = {
                let mut state = self.state.write().unwrap();

                let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

                let file = fs.delete_file(path, delete_if_empty)?;

                let repo_ids: Vec<String> = state
                    .vault_repos
                    .values()
                    .filter_map(|repo| {
                        if Path(repo.path.0.clone()).relative_to(path).is_some() {
                            Some(repo.id.clone())
                        } else {
                            None
                        }
                    })
                    .map(|x| x.0)
                    .collect();

                (file, repo_ids)
            };

            for repo_id in repo_ids {
                self.vault_repos_remove_service
                    .remove_vault_repo(context, &repo_id)?;
            }

            file
        };

        self.cleanup_objects(mount_id).await?;

        self.eventstream_listeners
            .process_event(eventstream::Event::FileRemovedEvent {
                mount_id: MountId(mount_id.to_owned()),
                path: RemotePath(path.to_owned().0),
                file: file.clone(),
                user_agent: context.user_agent.clone(),
            })
            .await;

        Ok(())
    }

    pub async fn rename_file(
        &self,
        context: &Context,
        mount_id: &str,
        path: &Path,
        name: Name,
    ) -> Result<(), FakeRemoteError> {
        if path == &Path::root() {
            return Err(FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::NotDir,
                "Directory expected".into(),
                None,
            ));
        }

        let to_path = path.parent().unwrap().join_name(&name);

        self.move_file(
            context,
            mount_id,
            path,
            to_path,
            &MoveFileConditions {
                if_modified: None,
                if_size: None,
                if_hash: None,
            },
        )
        .await
    }

    pub async fn copy_file(
        &self,
        context: &Context,
        mount_id: &str,
        path: &Path,
        to_path: Path,
    ) -> Result<(), FakeRemoteError> {
        let file = {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

            fs.copy_file(path, to_path.clone())?
        };

        self.eventstream_listeners
            .process_event(eventstream::Event::FileCopiedEvent {
                mount_id: MountId(mount_id.to_owned()),
                path: RemotePath(path.to_owned().0),
                new_path: RemotePath(to_path.clone().0),
                file: file.clone(),
                user_agent: context.user_agent.clone(),
            })
            .await;

        Ok(())
    }

    pub async fn move_file(
        &self,
        context: &Context,
        mount_id: &str,
        path: &Path,
        to_path: Path,
        conditions: &MoveFileConditions,
    ) -> Result<(), FakeRemoteError> {
        let file = {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, &mount_id)?;

            fs.move_file(path, to_path.clone(), conditions)?
        };

        self.eventstream_listeners
            .process_event(eventstream::Event::FileMovedEvent {
                mount_id: MountId(mount_id.to_owned()),
                path: RemotePath(path.to_owned().0),
                new_path: RemotePath(to_path.clone().0),
                file: file.clone(),
                user_agent: context.user_agent.clone(),
            })
            .await;

        Ok(())
    }

    pub fn list_recursive(
        &self,
        mount_id: &str,
        path: &Path,
    ) -> Result<Vec<models::FilesListRecursiveItem>, FakeRemoteError> {
        let state = self.state.read().unwrap();

        let fs = self.get_filesystem(&state, &mount_id)?;

        let file = fs.get_file(&path.normalize())?;

        if file.file.typ != "dir" {
            return Err(FakeRemoteError::ApiError(
                StatusCode::BAD_REQUEST,
                ApiErrorCode::NotDir,
                "Directory expected".into(),
                None,
            ));
        }

        let mut files: Vec<models::FilesListRecursiveItem> = vec![];

        self.list_recursive_file(fs, &mut files, Path::root(), file);

        Ok(files)
    }

    fn list_recursive_file(
        &self,
        fs: &Filesystem,
        files: &mut Vec<models::FilesListRecursiveItem>,
        path: Path,
        file: &FilesystemFile,
    ) {
        files.push(models::FilesListRecursiveItem::File {
            path: RemotePath(path.0.clone()),
            file: file.file.clone(),
        });

        for child_path in file.children.iter() {
            if let Ok(child) = fs.get_file(&child_path) {
                self.list_recursive_file(
                    fs,
                    files,
                    path.join_name(&Name(child.file.name.0.clone())),
                    child,
                )
            }
        }
    }

    pub async fn delete_object_file(&self, object_id: String) -> Result<(), ObjectProviderError> {
        self.object_provider.delete(object_id).await
    }

    pub async fn cleanup_objects(&self, mount_id: &str) -> Result<(), FakeRemoteError> {
        let object_ids = {
            let state = self.state.read().unwrap();

            let fs = self.get_filesystem(&state, mount_id)?;

            let mut object_ids = vec![];

            for (object_id, object) in fs.objects.iter() {
                if object.refs == 0 {
                    object_ids.push(object_id.to_owned());
                }
            }

            object_ids
        };

        let mut deleted_object_ids = vec![];

        for object_id in object_ids {
            match self.delete_object_file(object_id.clone()).await {
                Ok(()) => {
                    log::debug!("Deleted local file: {}", object_id);

                    deleted_object_ids.push(object_id)
                }
                Err(ObjectProviderError::IOError(err))
                    if err.kind() == std::io::ErrorKind::NotFound =>
                {
                    log::warn!("Failed to delete local file: {:?}", err);

                    // object does not exist anymore, remove it from state
                    deleted_object_ids.push(object_id)
                }
                Err(err) => {
                    log::warn!("Failed to delete local file: {:?}", err);
                }
            }
        }

        {
            let mut state = self.state.write().unwrap();

            let fs = self.get_filesystem_mut(&mut state, mount_id)?;

            for object_id in deleted_object_ids {
                fs.objects.remove(&object_id);
            }
        }

        Ok(())
    }
}
