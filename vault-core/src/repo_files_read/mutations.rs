use crate::{
    remote::RemoteError,
    repo_files::state::{RepoFile, RepoFileType},
    repo_files_list::{errors::FilesListRecursiveItemError, state::RepoFilesListRecursiveItem},
};

use super::{errors::GetFilesReaderError, state::RemoteZipEntry};

pub fn zip_size_estimate(entries: &[RemoteZipEntry]) -> i64 {
    // footer
    let mut size: i64 = 98;

    for entry in entries {
        match entry.typ {
            RepoFileType::Dir => {
                // file
                size += 30 + entry.filename.len() as i64;
                // directory entry
                size += 46 + entry.filename.len() as i64;
            }
            RepoFileType::File => {
                // file
                size += 66 + entry.filename.len() as i64 + entry.size;
                // directory entry
                size += 66 + entry.filename.len() as i64;
            }
        }
    }

    size
}

pub fn zip_date_time_from_millis(millis: i64) -> async_zip_futures::ZipDateTime {
    // millis < 1980-01-01 00:00:00 UTC
    if millis < 315532800000 {
        return async_zip_futures::ZipDateTime::default();
    }

    let modified = chrono::DateTime::from_naive_utc_and_offset(
        chrono::NaiveDateTime::from_timestamp_millis(millis).unwrap(),
        chrono::Utc,
    );

    async_zip_futures::ZipDateTime::from_chrono(&modified)
}

pub fn list_recursive_items_to_remote_zip_entries(
    items: Vec<RepoFilesListRecursiveItem>,
) -> Result<Vec<RemoteZipEntry>, RemoteError> {
    let mut entries: Vec<RemoteZipEntry> = Vec::with_capacity(items.len());

    for item in items {
        match item {
            RepoFilesListRecursiveItem::File {
                relative_repo_path,
                file,
            } => {
                let relative_repo_path = match relative_repo_path {
                    Ok(relative_repo_path) => relative_repo_path,
                    Err(_) => {
                        // skip non-decrypted files
                        continue;
                    }
                };

                if relative_repo_path == "/" {
                    // skip root item
                    continue;
                }

                let filename = match &file.typ {
                    RepoFileType::Dir => format!("{}/", &relative_repo_path[1..]),
                    RepoFileType::File => relative_repo_path[1..].to_owned(),
                };

                let size = match file.decrypted_size() {
                    Ok(size) => size.unwrap_or(0),
                    Err(_) => {
                        // skip non-decrypted files
                        continue;
                    }
                };

                entries.push(RemoteZipEntry {
                    mount_id: file.mount_id.clone(),
                    remote_path: file.remote_path.clone(),
                    repo_id: file.repo_id.clone(),
                    filename,
                    modified: zip_date_time_from_millis(file.modified.unwrap_or(0)),
                    typ: file.typ,
                    size,
                });
            }
            RepoFilesListRecursiveItem::Error { error, .. } => {
                match error {
                    FilesListRecursiveItemError::DecryptFilenameError(_) => {
                        // skip non-decrypted files
                        continue;
                    }
                    FilesListRecursiveItemError::RemoteError(err) => {
                        // fail on first remote error
                        return Err(err);
                    }
                }
            }
        }
    }

    Ok(entries)
}

pub fn file_to_remote_zip_entry(file: &RepoFile) -> Result<RemoteZipEntry, GetFilesReaderError> {
    Ok(RemoteZipEntry {
        mount_id: file.mount_id.clone(),
        remote_path: file.remote_path.clone(),
        repo_id: file.repo_id.clone(),
        filename: file.decrypted_name().map(str::to_string)?,
        modified: zip_date_time_from_millis(file.modified.unwrap_or(0)),
        typ: file.typ.clone(),
        size: file.decrypted_size()?.unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Timelike};

    use crate::{
        cipher::{errors::DecryptFilenameError, test_helpers as cipher_test_helpers},
        http::HttpError,
        remote::RemoteError,
        remote_files::test_helpers as remote_files_test_helpers,
        repo_files::{mutations::decrypt_file, state::RepoFileType},
        repo_files_list::{
            errors::FilesListRecursiveItemError, state::RepoFilesListRecursiveItem,
            test_helpers as repo_files_list_test_helpers,
        },
        repo_files_read::{errors::GetFilesReaderError, state::RemoteZipEntry},
    };

    use super::{
        file_to_remote_zip_entry, list_recursive_items_to_remote_zip_entries,
        zip_date_time_from_millis,
    };

    #[test]
    fn test_zip_date_time_from_millis() {
        assert_eq!(
            zip_date_time_from_millis(315532799000),
            async_zip_futures::ZipDateTime::default()
        );

        let dt = zip_date_time_from_millis(1678358492000);
        let chrono_dt: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_naive_utc_and_offset(
            chrono::NaiveDateTime::from_timestamp_millis(1678358492000).unwrap(),
            chrono::Utc,
        );
        assert_eq!(
            (
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second()
            ),
            (
                chrono_dt.date_naive().year(),
                chrono_dt.date_naive().month(),
                chrono_dt.date_naive().day(),
                chrono_dt.time().hour(),
                chrono_dt.time().minute(),
                chrono_dt.time().second(),
            )
        )
    }

    #[test]
    fn test_list_recursive_items_to_remote_zip_entries() {
        let cipher = cipher_test_helpers::create_cipher();
        let mut file_path_error = repo_files_list_test_helpers::create_list_recursive_item_file(
            "m1", "/Vault", "r1", "/D1", "/INVALID", &cipher,
        );
        match file_path_error {
            RepoFilesListRecursiveItem::File {
                ref mut relative_repo_path,
                ..
            } => {
                *relative_repo_path = Err(DecryptFilenameError::DecodeError(String::from(
                    "non-zero trailing bits at 1",
                )));
            }
            _ => {}
        };
        assert_eq!(
            list_recursive_items_to_remote_zip_entries(vec![
                repo_files_list_test_helpers::create_list_recursive_item_file(
                    "m1", "/Vault", "r1", "/D1", "/", &cipher,
                ),
                file_path_error,
                RepoFilesListRecursiveItem::Error {
                    mount_id: String::from("m1"),
                    remote_path: None,
                    error: FilesListRecursiveItemError::DecryptFilenameError(
                        DecryptFilenameError::DecodeError(String::from(
                            "non-zero trailing bits at 1",
                        ))
                    )
                },
                repo_files_list_test_helpers::create_list_recursive_item_file(
                    "m1", "/Vault", "r1", "/D1", "/F1", &cipher,
                ),
                repo_files_list_test_helpers::create_list_recursive_item_dir(
                    "m1", "/Vault", "r1", "/D1", "/D2", &cipher,
                ),
                repo_files_list_test_helpers::create_list_recursive_item_file(
                    "m1", "/Vault", "r1", "/D1", "/D2/F2", &cipher,
                ),
            ])
            .unwrap(),
            vec![
                RemoteZipEntry {
                    mount_id: String::from("m1"),
                    remote_path: format!(
                        "/Vault/{}/{}",
                        cipher.encrypt_filename("D1"),
                        cipher.encrypt_filename("F1")
                    ),
                    filename: String::from("F1"),
                    repo_id: String::from("r1"),
                    modified: async_zip_futures::ZipDateTime::default(),
                    typ: RepoFileType::File,
                    size: 52,
                },
                RemoteZipEntry {
                    mount_id: String::from("m1"),
                    remote_path: format!(
                        "/Vault/{}/{}",
                        cipher.encrypt_filename("D1"),
                        cipher.encrypt_filename("D2")
                    ),
                    filename: String::from("D2/"),
                    repo_id: String::from("r1"),
                    modified: async_zip_futures::ZipDateTime::default(),
                    typ: RepoFileType::Dir,
                    size: 0,
                },
                RemoteZipEntry {
                    mount_id: String::from("m1"),
                    remote_path: format!(
                        "/Vault/{}/{}/{}",
                        cipher.encrypt_filename("D1"),
                        cipher.encrypt_filename("D2"),
                        cipher.encrypt_filename("F2")
                    ),
                    filename: String::from("D2/F2"),
                    repo_id: String::from("r1"),
                    modified: async_zip_futures::ZipDateTime::default(),
                    typ: RepoFileType::File,
                    size: 52,
                },
            ]
        );
    }

    #[test]
    fn test_list_recursive_items_to_remote_zip_entries_remote_error() {
        let cipher = cipher_test_helpers::create_cipher();
        assert_eq!(
            list_recursive_items_to_remote_zip_entries(vec![
                repo_files_list_test_helpers::create_list_recursive_item_file(
                    "m1", "/Vault", "r1", "/D1", "/", &cipher,
                ),
                repo_files_list_test_helpers::create_list_recursive_item_file(
                    "m1", "/Vault", "r1", "/D1", "/F1", &cipher,
                ),
                RepoFilesListRecursiveItem::Error {
                    mount_id: String::from("m1"),
                    remote_path: None,
                    error: FilesListRecursiveItemError::RemoteError(RemoteError::HttpError(
                        HttpError::ResponseError(String::from("invalid json"))
                    ))
                },
            ])
            .unwrap_err(),
            RemoteError::HttpError(HttpError::ResponseError(String::from("invalid json")))
        );
    }

    #[test]
    fn test_file_to_remote_zip_entry() {
        let cipher = cipher_test_helpers::create_cipher();
        let remote_file = remote_files_test_helpers::create_file(
            "m1",
            &format!("/Vault/{}", cipher.encrypt_filename("F1")),
        );
        let file = decrypt_file("r1", "/", &remote_file, &cipher);
        assert_eq!(
            file_to_remote_zip_entry(&file).unwrap(),
            RemoteZipEntry {
                mount_id: String::from("m1"),
                remote_path: format!("/Vault/{}", cipher.encrypt_filename("F1")),
                repo_id: String::from("r1"),
                filename: String::from("F1"),
                modified: async_zip_futures::ZipDateTime::default(),
                typ: RepoFileType::File,
                size: 52,
            }
        )
    }

    #[test]
    fn test_file_to_remote_zip_entry_decrypt_error() {
        let cipher = cipher_test_helpers::create_cipher();
        let remote_file = remote_files_test_helpers::create_file("m1", "/Vault/F1");
        let file = decrypt_file("r1", "/", &remote_file, &cipher);
        assert!(matches!(
            file_to_remote_zip_entry(&file).unwrap_err(),
            GetFilesReaderError::DecryptFilenameError(_)
        ));
    }
}
