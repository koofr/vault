use crate::{
    config::state::ConfigState, dialogs::state::DialogsState, dir_pickers::state::DirPickersState,
    notifications::state::NotificationsState, oauth2::state::OAuth2State,
    remote_files::state::RemoteFilesState, remote_files_browsers::state::RemoteFilesBrowsersState,
    repo_config_backup::state::RepoConfigBackupsState, repo_create::state::RepoCreatesState,
    repo_files::state::RepoFilesState, repo_files_browsers::state::RepoFilesBrowsersState,
    repo_files_details::state::RepoFilesDetailsState, repo_files_move::state::RepoFilesMoveState,
    repo_remove::state::RepoRemovesState, repo_space_usage::state::RepoSpaceUsagesState,
    repo_unlock::state::RepoUnlocksState, repos::state::ReposState,
    space_usage::state::SpaceUsageState, transfers::state::TransfersState, user::state::UserState,
};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub config: ConfigState,
    pub notifications: NotificationsState,
    pub dialogs: DialogsState,
    pub oauth2: OAuth2State,
    pub user: UserState,
    pub remote_files: RemoteFilesState,
    pub remote_files_browsers: RemoteFilesBrowsersState,
    pub repos: ReposState,
    pub repo_creates: RepoCreatesState,
    pub repo_unlocks: RepoUnlocksState,
    pub repo_removes: RepoRemovesState,
    pub repo_config_backups: RepoConfigBackupsState,
    pub repo_space_usages: RepoSpaceUsagesState,
    pub repo_files: RepoFilesState,
    pub repo_files_browsers: RepoFilesBrowsersState,
    pub repo_files_details: RepoFilesDetailsState,
    pub repo_files_move: Option<RepoFilesMoveState>,
    pub transfers: TransfersState,
    pub dir_pickers: DirPickersState,
    pub space_usage: SpaceUsageState,
}

impl State {
    pub fn reset(&mut self) {
        *self = State {
            config: self.config.clone(),
            ..Default::default()
        };
    }
}
