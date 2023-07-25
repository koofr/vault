use std::sync::{Arc, Mutex};

use futures::{channel::oneshot, future::BoxFuture, FutureExt};
use vault_core_tests::fake_remote::FakeRemote;
use vault_mobile::{
    MobileVault, Repo, RepoCreateInfo, RepoFile, RepoFilesBrowserItem, RepoFilesBrowserOptions,
    RepoUnlockMode, RepoUnlockOptions, RepoUnlockUnlocked, Status, SubscriptionCallback,
    TransfersDownloadDone,
};

use super::user_fixture::UserFixture;

pub struct RepoFixture {
    pub user_fixture: Arc<UserFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub mobile_vault: Arc<MobileVault>,
    pub repo_id: String,
}

impl RepoFixture {
    pub async fn create(user_fixture: Arc<UserFixture>) -> Arc<Self> {
        let fake_remote = user_fixture.fake_remote.clone();
        let mobile_vault = user_fixture.mobile_vault.clone();

        let create_id = mobile_vault.clone().repo_create_create();
        user_fixture
            .wait(
                |v, cb| v.repo_create_info_subscribe(create_id, cb),
                |v, id| {
                    v.repo_create_info_data(id).filter(|info| match info {
                        RepoCreateInfo::Form { form } => {
                            matches!(form.create_load_status, Status::Loaded)
                        }
                        RepoCreateInfo::Created { .. } => false,
                    })
                },
            )
            .await;
        mobile_vault.repo_create_set_password(create_id, "password".into());
        mobile_vault.repo_create_set_salt(create_id, Some("salt".into()));
        mobile_vault.clone().repo_create_create_repo(create_id);
        let info = user_fixture
            .wait(
                |v, cb| v.repo_create_info_subscribe(create_id, cb),
                |v, id| {
                    v.repo_create_info_data(id).filter(|info| match info {
                        RepoCreateInfo::Form { form } => !matches!(
                            form.create_repo_status,
                            Status::Initial | Status::Loading { .. }
                        ),
                        RepoCreateInfo::Created { .. } => true,
                    })
                },
            )
            .await;
        let created = match info {
            RepoCreateInfo::Created { created } => created,
            RepoCreateInfo::Form { .. } => panic!("expected created"),
        };
        let repo_id = created.repo_id;
        mobile_vault.repo_create_destroy(create_id);

        Arc::new(Self {
            user_fixture,

            fake_remote,
            mobile_vault,
            repo_id,
        })
    }

    pub fn wait<T: Clone + Send + Sync + 'static>(
        &self,
        subscribe: impl FnOnce(&MobileVault, Box<dyn SubscriptionCallback>) -> u32,
        get_data: impl Fn(&MobileVault, u32) -> Option<T> + Send + Sync + 'static,
    ) -> BoxFuture<'static, T> {
        self.user_fixture.wait(subscribe, get_data)
    }

    pub async fn unlock(&self) {
        let unlock_id = self.mobile_vault.clone().repo_unlock_create(
            self.repo_id.clone(),
            RepoUnlockOptions {
                mode: RepoUnlockMode::Unlock,
            },
        );

        let (sender, receiver) = oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender)));

        self.mobile_vault.clone().repo_unlock_unlock(
            unlock_id,
            "password".into(),
            repo_unlock_unlocked(move || {
                if let Some(sender) = sender.lock().unwrap().take() {
                    let _ = sender.send(());
                }
            }),
        );

        receiver.await.unwrap();

        let info = self
            .wait(
                |v, cb| v.repo_unlock_info_subscribe(unlock_id, cb),
                |v, id| v.repo_unlock_info_data(id),
            )
            .await;
        assert_eq!(info.status, Status::Loaded);

        self.mobile_vault.repo_unlock_destroy(unlock_id);
    }

    pub async fn get_repo(&self) -> Repo {
        let repo_id = self.repo_id.clone();

        let repo_info = self
            .wait(
                |v, cb| v.repos_repo_subscribe(repo_id.clone(), cb),
                |v, id| v.repos_repo_data(id),
            )
            .await;

        assert!(matches!(repo_info.status, Status::Loaded));

        repo_info.repo.unwrap()
    }

    pub fn wait_for_repo_browser_items<T: Clone + Send + Sync + 'static>(
        &self,
        path: &str,
        f: impl Fn(Vec<RepoFilesBrowserItem>) -> Option<T> + Send + Sync + 'static,
    ) -> BoxFuture<'static, T> {
        let f = Arc::new(f);

        let browser_id = self.mobile_vault.clone().repo_files_browsers_create(
            self.repo_id.clone(),
            path.to_owned(),
            RepoFilesBrowserOptions { select_name: None },
        );

        let mobile_vault = self.mobile_vault.clone();

        self.wait(
            |v, cb| v.repo_files_browsers_info_subscribe(browser_id, cb),
            move |v, id| {
                v.repo_files_browsers_info_data(id)
                    .and_then(|info| f(info.items))
            },
        )
        .map(move |res| {
            mobile_vault.repo_files_browsers_destroy(browser_id);

            res
        })
        .boxed()
    }

    pub fn wait_for_file(&self, parent_path: &str, name: &str) -> BoxFuture<'static, RepoFile> {
        let name = name.to_owned();

        self.wait_for_repo_browser_items(parent_path, move |items| {
            items
                .into_iter()
                .find(|item| item.file.name == name)
                .map(|item| item.file)
        })
    }

    pub async fn upload_file(&self, parent_path: &str, name: &str, content: &str) -> RepoFile {
        let wait_future = self.wait_for_file(parent_path, name);

        let bytes = content.as_bytes().to_vec();

        self.mobile_vault.clone().transfers_upload_bytes(
            self.repo_id.clone(),
            parent_path.to_owned(),
            name.to_owned(),
            bytes,
        );

        wait_future.await
    }

    pub async fn get_temp_path(&self) -> String {
        let local_base_path = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

        tokio::fs::create_dir_all(&local_base_path).await.unwrap();

        local_base_path.to_str().unwrap().to_owned()
    }

    pub async fn transfers_download_file(
        &self,
        path: &str,
        local_base_path: &str,
        append_name: bool,
        autorename: bool,
    ) -> String {
        let (sender, receiver) = oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender)));

        self.mobile_vault.clone().transfers_download_file(
            self.repo_id.clone(),
            path.to_owned(),
            local_base_path.to_owned(),
            append_name,
            autorename,
            None,
            transfers_download_done(move |local_file_path| {
                if let Some(sender) = sender.lock().unwrap().take() {
                    let _ = sender.send(local_file_path);
                }
            }),
        );

        receiver.await.unwrap()
    }
}

fn repo_unlock_unlocked(f: impl Fn() + Send + Sync + 'static) -> Box<dyn RepoUnlockUnlocked> {
    struct RepoUnlockUnlockedFn {
        f: Box<dyn Fn() + Send + Sync>,
    }

    impl RepoUnlockUnlocked for RepoUnlockUnlockedFn {
        fn on_unlocked(&self) {
            (self.f)();
        }
    }

    impl std::fmt::Debug for RepoUnlockUnlockedFn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("RepoUnlockUnlockedFn").finish()
        }
    }

    Box::new(RepoUnlockUnlockedFn { f: Box::new(f) })
}

pub fn transfers_download_done(
    f: impl Fn(String) + Send + Sync + 'static,
) -> Box<dyn TransfersDownloadDone> {
    struct TransfersDownloadDoneFn {
        f: Box<dyn Fn(String) + Send + Sync>,
    }

    impl TransfersDownloadDone for TransfersDownloadDoneFn {
        fn on_done(&self, path: String, _content_type: Option<String>) {
            (self.f)(path);
        }
    }

    impl std::fmt::Debug for TransfersDownloadDoneFn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("TransfersDownloadDoneFn").finish()
        }
    }

    Box::new(TransfersDownloadDoneFn { f: Box::new(f) })
}
