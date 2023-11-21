use crate::{
    config::state::ConfigState, dialogs::state::DialogsState, dir_pickers::state::DirPickersState,
    eventstream::state::EventstreamState, notifications::state::NotificationsState,
    oauth2::state::OAuth2State, remote_files::state::RemoteFilesState,
    remote_files_browsers::state::RemoteFilesBrowsersState,
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
    pub eventstream: EventstreamState,
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
        // config is kept
        // notifications are kept so that any errors are displayed after logout
        self.dialogs.reset();
        self.oauth2.reset();
        self.user.reset();
        self.eventstream.reset();
        self.remote_files.reset();
        self.remote_files_browsers.reset();
        self.repos.reset();
        self.repo_creates.reset();
        self.repo_unlocks.reset();
        self.repo_removes.reset();
        self.repo_config_backups.reset();
        self.repo_space_usages.reset();
        self.repo_files.reset();
        self.repo_files_browsers.reset();
        self.repo_files_details.reset();
        self.repo_files_move = None;
        self.transfers.reset();
        self.dir_pickers.reset();
        self.space_usage.reset();
    }
}
