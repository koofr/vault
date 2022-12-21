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
    RepoFiles,
    RepoFilesBrowsers,
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
            Self::RepoFiles,
            Self::RepoFilesBrowsers,
            Self::RepoFilesMove,
            Self::Uploads,
            Self::DirPickers,
            Self::SpaceUsage,
        ]
    }
}
