use crate::remote_files::state::RemoteFilesMutationState;

#[derive(Clone, Default)]
pub struct MutationState {
    pub remote_files: RemoteFilesMutationState,
}
