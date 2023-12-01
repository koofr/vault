#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationEvent {
    Lifecycle,
    EventstreamEvents,
    RemoteFiles,
    Repos,
    RepoFiles,
}
