use std::sync::Arc;

use vault_core::{common::state::Status, repo_files_details::state::RepoFilesDetails, store};

pub async fn details_wait<Filter: Fn(&RepoFilesDetails) -> bool + Send + Sync + 'static>(
    store: Arc<store::Store>,
    details_id: u32,
    filter: Filter,
) {
    store::wait_for(
        store.clone(),
        &[store::Event::RepoFilesDetails],
        move |_| {
            store.with_state(|state| {
                state
                    .repo_files_details
                    .details
                    .get(&details_id)
                    .filter(|details| filter(details))
                    .map(|_| ())
            })
        },
    )
    .await;
}

pub async fn details_wait_content_loaded(store: Arc<store::Store>, details_id: u32) {
    details_wait(store, details_id, |details| {
        matches!(
            details.location.as_ref().unwrap().content.status,
            Status::Loaded
        )
    })
    .await;
}
