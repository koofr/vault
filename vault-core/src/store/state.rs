use crate::{
    config::state::ConfigState, dialogs::state::DialogsState, dir_pickers::state::DirPickersState,
    notifications::state::NotificationsState, oauth2::state::OAuth2State,
    remote_files::state::RemoteFilesState, repo_config_backup::state::RepoConfigBackupState,
    repo_create::state::RepoCreateState, repo_files::state::RepoFilesState,
    repo_files_browsers::state::RepoFilesBrowsersState,
    repo_files_details::state::RepoFilesDetailsState, repo_files_move::state::RepoFilesMoveState,
    repo_remove::state::RepoRemoveState, repo_space_usage::state::RepoSpaceUsageState,
    repo_unlock::state::RepoUnlockState, repos::state::ReposState,
    space_usage::state::SpaceUsageState, uploads::state::UploadsState, user::state::UserState,
};

#[derive(Clone, Default)]
pub struct State {
    pub config: ConfigState,
    pub notifications: NotificationsState,
    pub dialogs: DialogsState,
    pub oauth2: OAuth2State,
    pub user: UserState,
    pub remote_files: RemoteFilesState,
    pub repos: ReposState,
    pub repo_create: Option<RepoCreateState>,
    pub repo_unlock: Option<RepoUnlockState>,
    pub repo_remove: Option<RepoRemoveState>,
    pub repo_config_backup: Option<RepoConfigBackupState>,
    pub repo_space_usage: Option<RepoSpaceUsageState>,
    pub repo_files: RepoFilesState,
    pub repo_files_browsers: RepoFilesBrowsersState,
    pub repo_files_details: RepoFilesDetailsState,
    pub repo_files_move: Option<RepoFilesMoveState>,
    pub uploads: UploadsState,
    pub dir_pickers: DirPickersState,
    pub space_usage: SpaceUsageState,
}

impl State {
    pub fn reset(&mut self) {
        // config is not reset
        self.notifications = Default::default();
        self.dialogs = Default::default();
        self.oauth2 = Default::default();
        self.user = Default::default();
        self.remote_files = Default::default();
        self.repos = Default::default();
        self.repo_create = Default::default();
        self.repo_unlock = Default::default();
        self.repo_remove = Default::default();
        self.repo_config_backup = Default::default();
        self.repo_space_usage = Default::default();
        self.repo_files = Default::default();
        self.repo_files_browsers = Default::default();
        self.repo_files_details = Default::default();
        self.repo_files_move = Default::default();
        self.uploads = Default::default();
        self.dir_pickers = Default::default();
        self.space_usage = Default::default();
    }
}
