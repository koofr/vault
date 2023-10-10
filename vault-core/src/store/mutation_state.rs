use crate::{
    remote_files::state::RemoteFilesMutationState, repo_files::state::RepoFilesMutationState,
    repos::state::ReposMutationState,
};

#[derive(Debug, Clone, Default)]
pub struct MutationState {
    pub remote_files: RemoteFilesMutationState,
    pub repos: ReposMutationState,
    pub repo_files: RepoFilesMutationState,
}
