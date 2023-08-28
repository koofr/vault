#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Notifications,
    Dialogs,
    Auth,
    User,
    RemoteFiles,
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
            Self::Notifications,
            Self::Dialogs,
            Self::Auth,
            Self::User,
            Self::RemoteFiles,
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
