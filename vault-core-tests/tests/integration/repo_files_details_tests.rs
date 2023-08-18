use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{http::StatusCode, response::IntoResponse};
use vault_core::{
    file_types::{file_category::FileCategory, files_filter::FilesFilter},
    repo_files_details::state::RepoFilesDetailsOptions,
};
use vault_core_tests::{fixtures::repo_fixture::RepoFixture, helpers::with_vault};
use vault_fake_remote::fake_remote::interceptor::InterceptorResult;

#[test]
fn test_content_loaded_error() {
    with_vault(|user_fixture| async move {
        let download_counter = Arc::new(AtomicUsize::new(0));
        let interceptor_download_counter = download_counter.clone();

        user_fixture.fake_remote.intercept(Box::new(move |parts| {
            if parts.uri.path().contains("/content/api") && parts.uri.path().contains("/files/get")
            {
                interceptor_download_counter.fetch_add(1, Ordering::SeqCst);
                InterceptorResult::Response(StatusCode::INTERNAL_SERVER_ERROR.into_response())
            } else {
                InterceptorResult::Ignore
            }
        }));

        let repo_fixture = RepoFixture::create(user_fixture.clone()).await;

        user_fixture.load().await;

        repo_fixture.unlock().await;

        repo_fixture.upload_file("/file.txt", "text").await;

        let (_, load_future) = user_fixture.vault.repo_files_details_create(
            &repo_fixture.repo_id,
            "/file.txt",
            false,
            RepoFilesDetailsOptions {
                autosave_interval: Duration::from_secs(20),
                load_content: FilesFilter {
                    categories: vec![FileCategory::Text],
                    exts: vec![],
                },
            },
        );
        load_future.await.unwrap();

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(download_counter.load(Ordering::SeqCst), 1);
    });
}
