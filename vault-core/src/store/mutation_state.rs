use crate::{
    eventstream::state::{EventstreamEventsMutationState, EventstreamMutationState},
    remote_files::state::RemoteFilesMutationState,
    repo_files::state::RepoFilesMutationState,
    repos::state::ReposMutationState,
};

#[derive(Debug, Clone, Default)]
pub struct MutationState {
    pub eventstream: EventstreamMutationState,
    pub eventstream_events: EventstreamEventsMutationState,
    pub remote_files: RemoteFilesMutationState,
    pub repos: ReposMutationState,
    pub repo_files: RepoFilesMutationState,
}
