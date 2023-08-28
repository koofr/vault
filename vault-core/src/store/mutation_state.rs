use crate::{
    remote_files::state::RemoteFilesMutationState, repo_files::state::RepoFilesMutationState,
};

#[derive(Debug, Clone, Default)]
pub struct MutationState {
    pub remote_files: RemoteFilesMutationState,
    pub repo_files: RepoFilesMutationState,
}
