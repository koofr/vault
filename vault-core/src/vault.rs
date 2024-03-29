use std::{sync::Arc, time::Duration};

use futures::future::BoxFuture;

use crate::{
    auth, config, dialogs, dir_pickers, eventstream, http, lifecycle, notifications, oauth2,
    rclone, relative_time, remote, remote_files, remote_files_browsers, remote_files_dir_pickers,
    repo_config_backup, repo_create, repo_files, repo_files_browsers, repo_files_details,
    repo_files_dir_pickers, repo_files_list, repo_files_move, repo_files_read, repo_files_tags,
    repo_locker, repo_remove, repo_space_usage, repo_unlock, repos, runtime, secure_storage, sort,
    space_usage, store,
    transfers::{self, downloadable::BoxDownloadable},
    types::{DecryptedName, EncryptedPath, RepoFileId, RepoId, TimeMillis},
    user,
};

pub struct Vault {
    pub store: Arc<store::Store>,
    pub http_client: Arc<Box<dyn http::HttpClient + Send + Sync>>,
    pub runtime: Arc<runtime::BoxRuntime>,
    pub secure_storage_service: Arc<secure_storage::SecureStorageService>,
    pub notifications_service: Arc<notifications::NotificationsService>,
    pub dialogs_service: Arc<dialogs::DialogsService>,
    pub oauth2_service: Arc<oauth2::OAuth2Service>,
    pub auth_provider: Arc<Box<(dyn auth::AuthProvider + Send + Sync)>>,
    pub remote: Arc<remote::Remote>,
    pub user_service: Arc<user::UserService>,
    pub eventstream_service: Arc<eventstream::EventStreamService>,
    pub transfers_service: Arc<transfers::TransfersService>,
    pub remote_files_service: Arc<remote_files::RemoteFilesService>,
    pub remote_files_dir_pickers_service:
        Arc<remote_files_dir_pickers::RemoteFilesDirPickersService>,
    pub remote_files_browsers_service: Arc<remote_files_browsers::RemoteFilesBrowsersService>,
    pub repos_service: Arc<repos::ReposService>,
    pub repo_locker_service: Arc<repo_locker::RepoLockerService>,
    pub repo_create_service: Arc<repo_create::RepoCreateService>,
    pub repo_unlock_service: Arc<repo_unlock::RepoUnlockService>,
    pub repo_remove_service: Arc<repo_remove::RepoRemoveService>,
    pub repo_config_backup_service: Arc<repo_config_backup::RepoConfigBackupService>,
    pub repo_space_usage_service: Arc<repo_space_usage::RepoSpaceUsageService>,
    pub repo_files_list_service: Arc<repo_files_list::RepoFilesListService>,
    pub repo_files_tags_service: Arc<repo_files_tags::RepoFilesTagsService>,
    pub repo_files_read_service: Arc<repo_files_read::RepoFilesReadService>,
    pub repo_files_service: Arc<repo_files::RepoFilesService>,
    pub repo_files_dir_pickers_service: Arc<repo_files_dir_pickers::RepoFilesDirPickersService>,
    pub repo_files_browsers_service: Arc<repo_files_browsers::RepoFilesBrowsersService>,
    pub repo_files_details_service: Arc<repo_files_details::RepoFilesDetailsService>,
    pub repo_files_move_service: Arc<repo_files_move::RepoFilesMoveService>,
    pub space_usage_service: Arc<space_usage::SpaceUsageService>,
    pub lifecycle_service: Arc<lifecycle::LifecycleService>,
}

impl Vault {
    pub fn new(
        base_url: String,
        oauth2_config: oauth2::OAuth2Config,
        http_client: Box<dyn http::HttpClient + Send + Sync>,
        eventstream_websocket_client: Box<dyn eventstream::WebSocketClient + Send + Sync>,
        secure_storage: Box<dyn secure_storage::SecureStorage + Send + Sync>,
        runtime: runtime::BoxRuntime,
    ) -> Self {
        let state = store::State {
            config: config::state::ConfigState {
                base_url: base_url.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let store = Arc::new(store::Store::new(state));
        let http_client = Arc::new(http_client);
        let runtime = Arc::new(runtime);
        let secure_storage_service =
            Arc::new(secure_storage::SecureStorageService::new(secure_storage));
        let notifications_service = Arc::new(notifications::NotificationsService::new(
            store.clone(),
            runtime.clone(),
        ));
        let dialogs_service = Arc::new(dialogs::DialogsService::new(store.clone()));
        let oauth2_service = Arc::new(oauth2::OAuth2Service::new(
            oauth2_config,
            secure_storage_service.clone(),
            http_client.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let auth_provider: Arc<Box<(dyn auth::AuthProvider + Send + Sync + 'static)>> = Arc::new(
            Box::new(oauth2::OAuth2AuthProvider::new(oauth2_service.clone())),
        );
        let remote = Arc::new(remote::Remote::new(
            base_url.clone(),
            http_client.clone(),
            auth_provider.clone(),
        ));
        let user_service = Arc::new(user::UserService::new(remote.clone(), store.clone()));
        let eventstream_service = eventstream::EventStreamService::new(
            base_url.clone(),
            eventstream_websocket_client,
            auth_provider.clone(),
            store.clone(),
            runtime.clone(),
        );
        let remote_files_service = Arc::new(remote_files::RemoteFilesService::new(
            remote.clone(),
            dialogs_service.clone(),
            store.clone(),
        ));
        let remote_files_browsers_service =
            Arc::new(remote_files_browsers::RemoteFilesBrowsersService::new(
                remote_files_service.clone(),
                store.clone(),
            ));
        let remote_files_dir_pickers_service =
            Arc::new(remote_files_dir_pickers::RemoteFilesDirPickersService::new(
                remote_files_service.clone(),
                store.clone(),
            ));
        let repos_service = Arc::new(repos::ReposService::new(
            remote.clone(),
            remote_files_service.clone(),
            secure_storage_service.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let repo_locker_service = repo_locker::RepoLockerService::new(
            repos_service.clone(),
            store.clone(),
            runtime.clone(),
        );
        let repo_unlock_service = Arc::new(repo_unlock::RepoUnlockService::new(
            repos_service.clone(),
            store.clone(),
        ));
        let repo_remove_service = Arc::new(repo_remove::RepoRemoveService::new(
            repos_service.clone(),
            store.clone(),
        ));
        let repo_config_backup_service = Arc::new(
            repo_config_backup::RepoConfigBackupService::new(repos_service.clone(), store.clone()),
        );
        let repo_space_usage_service = Arc::new(repo_space_usage::RepoSpaceUsageService::new(
            remote_files_service.clone(),
            store.clone(),
        ));
        let repo_files_list_service = Arc::new(repo_files_list::RepoFilesListService::new(
            repos_service.clone(),
            remote_files_service.clone(),
        ));
        let repo_files_tags_service = Arc::new(repo_files_tags::RepoFilesTagsService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            store.clone(),
        ));
        let repo_files_read_service = Arc::new(repo_files_read::RepoFilesReadService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            repo_files_list_service.clone(),
            repo_files_tags_service.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let repo_files_service = Arc::new(repo_files::RepoFilesService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            repo_files_tags_service.clone(),
            repo_files_read_service.clone(),
            dialogs_service.clone(),
            store.clone(),
        ));
        let repo_create_service = Arc::new(repo_create::RepoCreateService::new(
            repos_service.clone(),
            remote_files_service.clone(),
            remote_files_dir_pickers_service.clone(),
            store.clone(),
        ));
        let transfers_service = Arc::new(transfers::TransfersService::new(
            repos_service.clone(),
            repo_files_service.clone(),
            store.clone(),
            runtime.clone(),
        ));
        let repo_files_dir_pickers_service =
            Arc::new(repo_files_dir_pickers::RepoFilesDirPickersService::new(
                repo_files_service.clone(),
                store.clone(),
            ));
        let repo_files_move_service = Arc::new(repo_files_move::RepoFilesMoveService::new(
            repo_files_service.clone(),
            repo_files_dir_pickers_service.clone(),
            store.clone(),
        ));
        let repo_files_browsers_service =
            Arc::new(repo_files_browsers::RepoFilesBrowsersService::new(
                repo_files_service.clone(),
                repo_files_read_service.clone(),
                repo_files_move_service.clone(),
                store.clone(),
                runtime.clone(),
            ));
        let repo_files_details_service =
            Arc::new(repo_files_details::RepoFilesDetailsService::new(
                repos_service.clone(),
                repo_files_service.clone(),
                repo_files_read_service.clone(),
                dialogs_service.clone(),
                transfers_service.clone(),
                store.clone(),
                runtime.clone(),
            ));
        let space_usage_service = Arc::new(space_usage::SpaceUsageService::new(
            remote.clone(),
            store.clone(),
        ));
        let lifecycle_service = lifecycle::LifecycleService::new(
            secure_storage_service.clone(),
            notifications_service.clone(),
            oauth2_service.clone(),
            user_service.clone(),
            repos_service.clone(),
            eventstream_service.clone(),
            space_usage_service.clone(),
            remote.clone(),
            store.clone(),
        );

        Self {
            store,
            http_client,
            runtime,
            notifications_service,
            secure_storage_service,
            dialogs_service,
            oauth2_service,
            auth_provider,
            remote,
            user_service,
            eventstream_service,
            transfers_service,
            remote_files_service,
            remote_files_dir_pickers_service,
            remote_files_browsers_service,
            repos_service,
            repo_locker_service,
            repo_create_service,
            repo_unlock_service,
            repo_remove_service,
            repo_config_backup_service,
            repo_space_usage_service,
            repo_files_list_service,
            repo_files_tags_service,
            repo_files_read_service,
            repo_files_service,
            repo_files_dir_pickers_service,
            repo_files_browsers_service,
            repo_files_details_service,
            repo_files_move_service,
            space_usage_service,
            lifecycle_service,
        }
    }

    // store

    pub fn get_next_id(&self) -> u32 {
        self.store.get_next_id()
    }

    pub fn on(&self, id: u32, events: &[store::Event], callback: store::OnCallback) {
        self.store.on(id, events, callback)
    }

    pub fn remove_listener(&self, id: u32) {
        self.store.remove_listener(id)
    }

    pub fn with_state<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&store::State) -> R,
    {
        self.store.with_state(f)
    }

    // subscription

    pub fn get_subscription(&self) -> store::Subscription {
        store::Subscription::new(self.store.clone())
    }

    // lifecycle

    pub fn load(
        &self,
    ) -> Result<
        BoxFuture<'static, Result<(), lifecycle::errors::LoadError>>,
        lifecycle::errors::LoadError,
    > {
        self.lifecycle_service.clone().load()
    }

    pub fn logout(&self) -> Result<(), lifecycle::errors::LogoutError> {
        self.lifecycle_service.logout()
    }

    pub fn app_visible(&self) {
        self.lifecycle_service.app_visible()
    }

    pub fn app_hidden(&self) {
        self.lifecycle_service.app_hidden()
    }

    // relative_time

    pub fn relative_time(
        &self,
        value: TimeMillis,
        with_modifier: bool,
    ) -> relative_time::RelativeTime {
        self.with_state(|state| {
            relative_time::RelativeTime::new(
                &self.runtime,
                value,
                &state.config.locale.locale,
                with_modifier,
            )
        })
    }

    // notifications

    pub fn notifications_show(&self, message: String) {
        self.notifications_service.show(message)
    }

    pub fn notifications_remove(&self, notification_id: u32) {
        self.notifications_service.remove(notification_id)
    }

    pub async fn notifications_remove_after(&self, notification_id: u32, duration: Duration) {
        self.notifications_service
            .remove_after(notification_id, duration)
            .await
    }

    pub fn notifications_remove_all(&self) {
        self.notifications_service.remove_all()
    }

    // dialogs

    pub fn dialogs_confirm(&self, dialog_id: u32) {
        self.dialogs_service.confirm(dialog_id)
    }

    pub fn dialogs_cancel(&self, dialog_id: u32) {
        self.dialogs_service.cancel(dialog_id)
    }

    pub fn dialogs_set_input_value(&self, dialog_id: u32, value: String) {
        self.dialogs_service.set_input_value(dialog_id, value);
    }

    // oauth2

    pub fn oauth2_start_login_flow(&self) -> Result<String, oauth2::errors::OAuth2Error> {
        self.oauth2_service.start_login_flow()
    }

    pub fn oauth2_start_logout_flow(&self) -> Result<String, oauth2::errors::OAuth2Error> {
        self.oauth2_service.start_logout_flow()
    }

    pub async fn oauth2_finish_flow_url(
        &self,
        url: &str,
    ) -> Result<(), lifecycle::errors::OAuth2FinishFlowUrlError> {
        self.lifecycle_service.oauth2_finish_flow_url(url).await
    }

    // user

    pub async fn user_load(&self) -> Result<(), remote::RemoteError> {
        self.user_service.load_user().await
    }

    pub async fn user_ensure_profile_picture(&self) -> Result<(), remote::RemoteError> {
        self.user_service.ensure_profile_picture().await
    }

    // remote_files_browsers

    pub fn remote_files_browsers_create(
        &self,
        location: &remote_files_browsers::state::RemoteFilesBrowserItemId,
        options: remote_files_browsers::state::RemoteFilesBrowserOptions,
    ) -> (u32, BoxFuture<'static, Result<(), remote::RemoteError>>) {
        self.remote_files_browsers_service
            .clone()
            .create(location, options)
    }

    pub fn remote_files_browsers_destroy(&self, browser_id: u32) {
        self.remote_files_browsers_service.destroy(browser_id)
    }

    pub async fn remote_files_browsers_load(
        &self,
        browser_id: u32,
    ) -> Result<(), remote::RemoteError> {
        self.remote_files_browsers_service.load(browser_id).await
    }

    pub fn remote_files_browsers_select_item(
        &self,
        browser_id: u32,
        item_id: remote_files_browsers::state::RemoteFilesBrowserItemId,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.remote_files_browsers_service
            .select_item(browser_id, item_id, extend, range, force)
    }

    pub fn remote_files_browsers_select_all(&self, browser_id: u32) {
        self.remote_files_browsers_service.select_all(browser_id)
    }

    pub fn remote_files_browsers_clear_selection(&self, browser_id: u32) {
        self.remote_files_browsers_service
            .clear_selection(browser_id)
    }

    pub fn remote_files_browsers_set_selection(
        &self,
        browser_id: u32,
        selection: Vec<remote_files_browsers::state::RemoteFilesBrowserItemId>,
    ) {
        self.remote_files_browsers_service
            .set_selection(browser_id, selection)
    }

    pub fn remote_files_browsers_sort_by(
        &self,
        browser_id: u32,
        field: remote_files::state::RemoteFilesSortField,
        direction: Option<sort::state::SortDirection>,
    ) {
        self.remote_files_browsers_service
            .sort_by(browser_id, field, direction)
    }

    pub async fn remote_files_browsers_create_dir(
        &self,
        browser_id: u32,
    ) -> Result<
        remote_files_browsers::state::RemoteFilesBrowserItemId,
        remote_files::errors::CreateDirError,
    > {
        self.remote_files_browsers_service
            .create_dir(browser_id)
            .await
    }

    // repos

    pub async fn repos_load(&self) -> Result<(), repos::errors::LoadReposError> {
        self.repos_service.load_repos().await
    }

    pub fn repos_lock_repo(&self, repo_id: &RepoId) -> Result<(), repos::errors::LockRepoError> {
        self.repos_service.lock_repo(repo_id)
    }

    pub fn repos_touch_repo(
        &self,
        repo_id: &RepoId,
    ) -> Result<(), repos::errors::RepoNotFoundError> {
        self.repos_service.touch_repo(repo_id)
    }

    pub fn repos_set_auto_lock(
        &self,
        repo_id: &RepoId,
        auto_lock: repos::state::RepoAutoLock,
    ) -> Result<(), repos::errors::SetAutoLockError> {
        self.repos_service.set_auto_lock(repo_id, auto_lock)
    }

    pub fn repos_set_default_auto_lock(&self, auto_lock: repos::state::RepoAutoLock) {
        self.repos_service.set_default_auto_lock(auto_lock)
    }

    // repo_create

    pub fn repo_create_create(
        &self,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_create::errors::CreateLoadError>>,
    ) {
        self.repo_create_service.clone().create()
    }

    pub async fn repo_create_create_load(
        &self,
        create_id: u32,
    ) -> Result<(), repo_create::errors::CreateLoadError> {
        self.repo_create_service
            .clone()
            .create_load(create_id)
            .await
    }

    pub fn repo_create_set_location(
        &self,
        create_id: u32,
        location: remote_files::state::RemoteFilesLocation,
    ) {
        self.repo_create_service.set_location(create_id, location)
    }

    pub fn repo_create_set_password(&self, create_id: u32, password: String) {
        self.repo_create_service.set_password(create_id, password)
    }

    pub fn repo_create_set_salt(&self, create_id: u32, salt: Option<String>) {
        self.repo_create_service.set_salt(create_id, salt)
    }

    pub fn repo_create_fill_from_rclone_config(
        &self,
        create_id: u32,
        config: String,
    ) -> Result<(), rclone::config::ParseConfigError> {
        self.repo_create_service
            .fill_from_rclone_config(create_id, config)
    }

    pub async fn repo_create_location_dir_picker_show(
        &self,
        create_id: u32,
    ) -> Result<(), remote::RemoteError> {
        self.repo_create_service
            .location_dir_picker_show(create_id)
            .await
    }

    pub async fn repo_create_location_dir_picker_click(
        &self,
        create_id: u32,
        item_id: &dir_pickers::state::DirPickerItemId,
        is_arrow: bool,
    ) -> Result<(), remote::RemoteError> {
        self.repo_create_service
            .location_dir_picker_click(create_id, item_id, is_arrow)
            .await
    }

    pub fn repo_create_location_dir_picker_select(&self, create_id: u32) {
        self.repo_create_service
            .location_dir_picker_select(create_id)
    }

    pub fn repo_create_location_dir_picker_cancel(&self, create_id: u32) {
        self.repo_create_service
            .location_dir_picker_cancel(create_id)
    }

    pub async fn repo_create_location_dir_picker_create_dir(
        &self,
        create_id: u32,
    ) -> Result<(), remote_files::errors::CreateDirError> {
        self.repo_create_service
            .location_dir_picker_create_dir(create_id)
            .await
    }

    pub async fn repo_create_create_repo(&self, create_id: u32) {
        self.repo_create_service.create_repo(create_id).await
    }

    pub fn repo_create_destroy(&self, create_id: u32) {
        self.repo_create_service.destroy(create_id);
    }

    // repo_unlock

    pub fn repo_unlock_create(
        &self,
        repo_id: RepoId,
        options: repo_unlock::state::RepoUnlockOptions,
    ) -> u32 {
        self.repo_unlock_service.create(repo_id, options)
    }

    pub fn repo_unlock_unlock(
        &self,
        unlock_id: u32,
        password: &str,
    ) -> Result<(), repos::errors::UnlockRepoError> {
        self.repo_unlock_service.unlock(unlock_id, password)
    }

    pub fn repo_unlock_destroy(&self, unlock_id: u32) {
        self.repo_unlock_service.destroy(unlock_id)
    }

    // repo_remove

    pub fn repo_remove_create(&self, repo_id: RepoId) -> u32 {
        self.repo_remove_service.create(repo_id)
    }

    pub async fn repo_remove_remove(
        &self,
        remove_id: u32,
        password: &str,
    ) -> Result<(), repos::errors::RemoveRepoError> {
        self.repo_remove_service.remove(remove_id, password).await
    }

    pub fn repo_remove_destroy(&self, remove_id: u32) {
        self.repo_remove_service.destroy(remove_id)
    }

    // repo_config_backup

    pub fn repo_config_backup_create(&self, repo_id: RepoId) -> u32 {
        self.repo_config_backup_service.create(repo_id)
    }

    pub fn repo_config_backup_generate(
        &self,
        backup_id: u32,
        password: &str,
    ) -> Result<(), repos::errors::UnlockRepoError> {
        self.repo_config_backup_service
            .generate(backup_id, password)
    }

    pub fn repo_config_backup_destroy(&self, backup_id: u32) {
        self.repo_config_backup_service.destroy(backup_id)
    }

    // repo_space_usage

    pub fn repo_space_usage_create(&self, repo_id: RepoId) -> u32 {
        self.repo_space_usage_service.create(repo_id)
    }

    pub async fn repo_space_usage_calculate(
        &self,
        usage_id: u32,
    ) -> Result<(), repo_space_usage::errors::RepoSpaceUsageError> {
        self.repo_space_usage_service.calculate(usage_id).await
    }

    pub fn repo_space_usage_destroy(&self, usage_id: u32) {
        self.repo_space_usage_service.destroy(usage_id)
    }

    // repo_files

    pub async fn repo_files_load_files(
        &self,
        repo_id: &RepoId,
        path: &EncryptedPath,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_service.load_files(repo_id, path).await
    }

    pub fn repo_files_get_file_reader(
        &self,
        repo_id: &RepoId,
        path: &EncryptedPath,
    ) -> Result<
        repo_files_read::state::RepoFileReaderProvider,
        repo_files_read::errors::GetFilesReaderError,
    > {
        self.repo_files_service
            .clone()
            .get_file_reader(repo_id, path)
    }

    pub async fn repo_files_delete_files(
        &self,
        files: &[(RepoId, EncryptedPath)],
    ) -> Result<(), repo_files::errors::DeleteFileError> {
        self.repo_files_service.delete_files(files, None).await
    }

    pub async fn repo_files_rename_file(
        &self,
        repo_id: &RepoId,
        path: &EncryptedPath,
    ) -> Result<(), repo_files::errors::RenameFileError> {
        self.repo_files_service.rename_file(repo_id, path).await
    }

    // transfers

    pub fn transfers_upload(
        &self,
        repo_id: RepoId,
        parent_path: EncryptedPath,
        name: transfers::state::TransferUploadRelativeName,
        uploadable: transfers::uploadable::BoxUploadable,
    ) -> (u32, transfers::state::CreateUploadResultFuture) {
        self.transfers_service
            .clone()
            .upload(repo_id, parent_path, name, uploadable)
    }

    pub fn transfers_download(
        &self,
        reader_provider: repo_files_read::state::RepoFileReaderProvider,
        downloadable: transfers::downloadable::BoxDownloadable,
    ) -> (u32, transfers::state::CreateDownloadResultFuture) {
        self.transfers_service
            .clone()
            .download(reader_provider, downloadable)
    }

    pub fn transfers_download_reader(
        &self,
        reader: repo_files_read::state::RepoFileReader,
    ) -> transfers::state::DownloadReaderResult {
        self.transfers_service.clone().download_reader(reader)
    }

    pub fn transfers_abort(&self, id: u32) {
        self.transfers_service.clone().abort(id);
    }

    pub fn transfers_abort_all(&self) {
        self.transfers_service.clone().abort_all();
    }

    pub fn transfers_retry(&self, id: u32) {
        self.transfers_service.clone().retry(id);
    }

    pub fn transfers_retry_all(&self) {
        self.transfers_service.clone().retry_all();
    }

    pub async fn transfers_open(&self, id: u32) -> Result<(), transfers::errors::TransferError> {
        self.transfers_service.clone().open(id).await
    }

    // repo_files_browsers

    pub fn repo_files_browsers_create(
        &self,
        repo_id: RepoId,
        path: &EncryptedPath,
        options: repo_files_browsers::state::RepoFilesBrowserOptions,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_files::errors::LoadFilesError>>,
    ) {
        self.repo_files_browsers_service
            .clone()
            .create(repo_id, path, options)
    }

    pub fn repo_files_browsers_destroy(&self, browser_id: u32) {
        self.repo_files_browsers_service.destroy(browser_id)
    }

    pub async fn repo_files_browsers_load_files(
        &self,
        browser_id: u32,
    ) -> Result<(), repo_files::errors::LoadFilesError> {
        self.repo_files_browsers_service
            .load_files(browser_id)
            .await
    }

    pub fn repo_files_browsers_select_file(
        &self,
        browser_id: u32,
        file_id: RepoFileId,
        extend: bool,
        range: bool,
        force: bool,
    ) {
        self.repo_files_browsers_service
            .select_file(browser_id, file_id, extend, range, force)
    }

    pub fn repo_files_browsers_select_all(&self, browser_id: u32) {
        self.repo_files_browsers_service.select_all(browser_id)
    }

    pub fn repo_files_browsers_clear_selection(&self, browser_id: u32) {
        self.repo_files_browsers_service.clear_selection(browser_id)
    }

    pub fn repo_files_browsers_set_selection(&self, browser_id: u32, selection: Vec<RepoFileId>) {
        self.repo_files_browsers_service
            .set_selection(browser_id, selection)
    }

    pub fn repo_files_browsers_sort_by(
        &self,
        browser_id: u32,
        field: repo_files::state::RepoFilesSortField,
        direction: Option<sort::state::SortDirection>,
    ) {
        self.repo_files_browsers_service
            .sort_by(browser_id, field, direction)
    }

    pub fn repo_files_browsers_get_selected_reader(
        &self,
        browser_id: u32,
    ) -> Result<
        repo_files_read::state::RepoFileReaderProvider,
        repo_files_read::errors::GetFilesReaderError,
    > {
        self.repo_files_browsers_service
            .clone()
            .get_selected_reader(browser_id)
    }

    pub async fn repo_files_browsers_create_dir(
        &self,
        browser_id: u32,
    ) -> Result<(DecryptedName, EncryptedPath), repo_files::errors::CreateDirError> {
        self.repo_files_browsers_service
            .create_dir(browser_id)
            .await
    }

    pub async fn repo_files_browsers_create_file(
        &self,
        browser_id: u32,
        name: &str,
    ) -> Result<(DecryptedName, EncryptedPath), repo_files::errors::CreateFileError> {
        self.repo_files_browsers_service
            .create_file(browser_id, name)
            .await
    }

    pub async fn repo_files_browsers_delete_selected(
        &self,
        browser_id: u32,
    ) -> Result<(), repo_files::errors::DeleteFileError> {
        self.repo_files_browsers_service
            .delete_selected(browser_id)
            .await
    }

    pub async fn repo_files_browsers_move_selected(
        &self,
        browser_id: u32,
        mode: repo_files_move::state::RepoFilesMoveMode,
    ) -> Result<(), repo_files_move::errors::ShowError> {
        self.repo_files_browsers_service
            .move_selected(browser_id, mode)
            .await
    }

    // repo_files_details

    pub fn repo_files_details_create(
        &self,
        repo_id: RepoId,
        path: &EncryptedPath,
        is_editing: bool,
        options: repo_files_details::state::RepoFilesDetailsOptions,
    ) -> (
        u32,
        BoxFuture<'static, Result<(), repo_files_details::errors::LoadDetailsError>>,
    ) {
        self.repo_files_details_service
            .clone()
            .create(repo_id, path, is_editing, options)
    }

    pub async fn repo_files_details_destroy(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files_details::errors::SaveError> {
        self.repo_files_details_service
            .clone()
            .destroy(details_id)
            .await
    }

    pub async fn repo_files_details_load_file(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files_details::errors::LoadDetailsError> {
        self.repo_files_details_service.load_file(details_id).await
    }

    pub async fn repo_files_details_load_content(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files_details::errors::LoadContentError> {
        self.repo_files_details_service
            .clone()
            .load_content(details_id)
            .await
    }

    pub async fn repo_files_details_get_file_reader(
        &self,
        details_id: u32,
    ) -> Result<
        repo_files_read::state::RepoFileReaderProvider,
        repo_files_read::errors::GetFilesReaderError,
    > {
        self.repo_files_details_service
            .clone()
            .get_file_reader(details_id)
            .await
    }

    pub async fn repo_files_details_download(
        &self,
        details_id: u32,
        downloadable: BoxDownloadable,
    ) -> Result<(), transfers::errors::TransferError> {
        self.repo_files_details_service
            .clone()
            .download(details_id, downloadable)
            .await
    }

    pub fn repo_files_details_edit(&self, details_id: u32) {
        self.repo_files_details_service.edit(details_id);
    }

    pub async fn repo_files_details_edit_cancel(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files_details::errors::SaveError> {
        self.repo_files_details_service
            .clone()
            .edit_cancel(details_id)
            .await
    }

    pub fn repo_files_details_set_content(
        &self,
        details_id: u32,
        content: Vec<u8>,
    ) -> Result<(), repo_files_details::errors::SetContentError> {
        self.repo_files_details_service
            .clone()
            .set_content(details_id, content)
    }

    pub async fn repo_files_details_save(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files_details::errors::SaveError> {
        self.repo_files_details_service
            .clone()
            .save(details_id)
            .await
    }

    pub async fn repo_files_details_delete(
        &self,
        details_id: u32,
    ) -> Result<(), repo_files::errors::DeleteFileError> {
        self.repo_files_details_service.delete(details_id).await
    }

    // repo_files_move

    pub async fn repo_files_move_move_file(
        &self,
        repo_id: RepoId,
        path: EncryptedPath,
        mode: repo_files_move::state::RepoFilesMoveMode,
    ) -> Result<(), repo_files_move::errors::ShowError> {
        self.repo_files_move_service
            .move_file(repo_id, path, mode)
            .await
    }

    pub async fn repo_files_move_dir_picker_click(
        &self,
        item_id: &dir_pickers::state::DirPickerItemId,
        is_arrow: bool,
    ) -> Result<(), repo_files_move::errors::DirPickerClickError> {
        self.repo_files_move_service
            .dir_picker_click(item_id, is_arrow)
            .await
    }

    pub fn repo_files_move_set_dest_path(&self, dest_path: EncryptedPath) {
        self.repo_files_move_service.set_dest_path(dest_path)
    }

    pub async fn repo_files_move_move_files(
        &self,
    ) -> Result<(), repo_files::errors::MoveFileError> {
        self.repo_files_move_service.move_files().await
    }

    pub fn repo_files_move_cancel(&self) {
        self.repo_files_move_service.cancel()
    }

    pub async fn repo_files_move_create_dir(
        &self,
    ) -> Result<(), repo_files::errors::CreateDirError> {
        self.repo_files_move_service.create_dir().await
    }
}

const _: () = {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    fn assert_all() {
        assert_send::<Vault>();
        assert_sync::<Vault>();
    }
};
