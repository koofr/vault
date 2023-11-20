#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationEvent {
    EventstreamEvents,
    RemoteFiles,
    Repos,
    RepoFiles,
}
