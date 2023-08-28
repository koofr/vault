#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationEvent {
    RemoteFiles,
    RepoFiles,
}
