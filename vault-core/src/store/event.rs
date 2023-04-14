#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Notifications,
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
    RepoFilesMove,
    Uploads,
    DirPickers,
    SpaceUsage,
}

impl Event {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Notifications,
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
            Self::RepoFilesMove,
            Self::Uploads,
            Self::DirPickers,
            Self::SpaceUsage,
        ]
    }
}
