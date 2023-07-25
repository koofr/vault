use std::path::PathBuf;

use futures::FutureExt;
use similar_asserts::assert_eq;

use crate::helpers::with_repo;

#[test]
fn test_transfers_download_file() {
    with_repo(|fixture| {
        async move {
            let _ = fixture.upload_file("/", "file.txt", "test").await;

            let local_base_path = fixture.get_temp_path().await;

            let local_file_path = fixture
                .transfers_download_file("/file.txt", &local_base_path, true, false)
                .await;

            assert_eq!(
                PathBuf::from(local_file_path),
                PathBuf::from(local_base_path).join("file.txt")
            );
        }
        .boxed()
    });
}

#[test]
fn test_transfers_download_file_cleanup_name() {
    with_repo(|fixture| {
        async move {
            let _ = fixture.upload_file("/", "<>", "test").await;

            let local_base_path = fixture.get_temp_path().await;

            let local_file_path = fixture
                .transfers_download_file("/<>", &local_base_path, true, false)
                .await;

            assert_eq!(
                PathBuf::from(local_file_path),
                PathBuf::from(local_base_path).join("invalid name")
            );
        }
        .boxed()
    });
}
