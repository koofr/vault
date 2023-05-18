#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MutationEvent {
    RemoteFiles,
    RepoFiles,
}
