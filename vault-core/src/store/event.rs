#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Lifecycle,
    Notifications,
    Dialogs,
    Auth,
    User,
    Eventstream,
    RemoteFiles,
    RemoteFilesBrowsers,
    Repos,
    RepoCreate,
    RepoUnlock,
    RepoRemove,
    RepoConfigBackup,
    RepoSpaceUsage,
    RepoFiles,
    RepoFilesBrowsers,
    RepoFilesDetails,
    RepoFilesDetailsContentData,
    RepoFilesMove,
    Transfers,
    DirPickers,
    SpaceUsage,
}

impl Event {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Lifecycle,
            Self::Notifications,
            Self::Dialogs,
            Self::Auth,
            Self::User,
            Self::Eventstream,
            Self::RemoteFiles,
            Self::RemoteFilesBrowsers,
            Self::Repos,
            Self::RepoCreate,
            Self::RepoUnlock,
            Self::RepoRemove,
            Self::RepoConfigBackup,
            Self::RepoSpaceUsage,
            Self::RepoFiles,
            Self::RepoFilesBrowsers,
            Self::RepoFilesDetails,
            Self::RepoFilesDetailsContentData,
            Self::RepoFilesMove,
            Self::Transfers,
            Self::DirPickers,
            Self::SpaceUsage,
        ]
    }
}
