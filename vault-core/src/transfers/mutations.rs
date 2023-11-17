use crate::{
    common::state::SizeInfo,
    files::file_category::{ext_to_file_category, FileCategory},
    store,
    types::{DecryptedName, DecryptedPath, RepoId},
    utils::{name_utils, path_utils, repo_path_utils},
};

use super::{
    errors::TransferError,
    selectors,
    state::{
        DownloadTransfer, RetryInitiator, Transfer, TransferState, TransferType, TransfersState,
        UploadTransfer,
    },
};

pub fn get_next_id(state: &mut store::State) -> u32 {
    // just generating next id does not need to call notify
    state.transfers.next_id.next()
}

pub enum CreateTransferType {
    Upload {
        repo_id: RepoId,
        parent_path: DecryptedPath,
        name: String,
    },
    Download {
        name: String,
    },
    DownloadReader {
        name: String,
    },
}

fn name_to_category(name: &str) -> FileCategory {
    name_utils::name_to_ext(&name.to_lowercase())
        .and_then(ext_to_file_category)
        .unwrap_or(FileCategory::Generic)
}

pub fn create_transfer(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    typ: CreateTransferType,
    size: SizeInfo,
    is_persistent: bool,
    is_retriable: bool,
    is_openable: bool,
) {
    notify(store::Event::Transfers);

    let (typ, name, category) = match typ {
        CreateTransferType::Upload {
            repo_id,
            parent_path,
            name,
        } => {
            let full_name = name;

            // name can be "path/to/file.txt" so that "path/to/file.txt" is
            // displayed in the transfers list, but not the full parent_path
            let (name_rel_path, name) = match full_name.rfind('/') {
                Some(idx) => (
                    Some(full_name[..idx].to_owned()),
                    full_name[idx + 1..].to_owned(),
                ),
                None => (None, full_name.clone()),
            };

            let parent_path = match &name_rel_path {
                Some(name_rel_path) => repo_path_utils::join_path_name(
                    &parent_path,
                    &DecryptedName(name_rel_path.to_owned()),
                ),
                None => parent_path,
            };

            (
                TransferType::Upload(UploadTransfer {
                    repo_id,
                    parent_path,
                    name_rel_path,
                    original_name: name.clone(),
                    name: name.clone(),
                }),
                full_name,
                name_to_category(&name),
            )
        }
        CreateTransferType::Download { name } => (
            TransferType::Download(DownloadTransfer { name: name.clone() }),
            name.clone(),
            name_to_category(&name),
        ),
        CreateTransferType::DownloadReader { name } => (
            TransferType::DownloadReader,
            name.clone(),
            name_to_category(&name),
        ),
    };

    let transfer = Transfer {
        id,
        typ,
        name,
        size,
        category,
        started: None,
        is_persistent,
        is_retriable,
        is_openable,
        state: TransferState::Waiting,
        transferred_bytes: 0,
        attempts: 0,
        order: state.transfers.total_count,
    };

    state.transfers.transfers.insert(id.clone(), transfer);

    state.transfers.total_count += 1;

    match size {
        SizeInfo::Exact(size) => state.transfers.total_bytes += size,
        SizeInfo::Estimate(size) => state.transfers.total_bytes += size,
        SizeInfo::Unknown => {}
    }
}

pub fn next_transfer(state: &mut store::State, notify: &store::Notify, now: i64) -> Option<u32> {
    let transfer = match selectors::select_next_transfer(state) {
        Some(transfer) => transfer,
        None => return None,
    };

    let id = transfer.id;

    start_transfer(state, notify, id, now);

    Some(id)
}

pub fn start_transfer(state: &mut store::State, notify: &store::Notify, id: u32, now: i64) {
    let transfer = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => transfer,
        None => return,
    };

    notify(store::Event::Transfers);

    transfer.started = Some(now);
    transfer.state = TransferState::Processing;
    transfer.transferred_bytes = 0;
    transfer.attempts += 1;

    state.transfers.transferring_count += 1;

    match &transfer.typ {
        TransferType::Upload(..) => state.transfers.transferring_uploads_count += 1,
        TransferType::Download(..) | TransferType::DownloadReader => {
            state.transfers.transferring_downloads_count += 1
        }
    }

    if state.transfers.started.is_none() {
        state.transfers.started = Some(now)
    }
}

pub fn upload_transfer_processed(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    size: SizeInfo,
) -> Result<String, TransferError> {
    let transfer = match state.transfers.transfers.get(&id) {
        Some(transfer) => transfer,
        None => return Err(TransferError::TransferNotFound),
    };
    let upload_transfer = match transfer.upload_transfer() {
        Some(upload_transfer) => upload_transfer,
        None => return Err(TransferError::TransferNotFound),
    };

    let name = selectors::select_unused_name(state, transfer, upload_transfer);

    let transfer = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => transfer,
        None => return Err(TransferError::TransferNotFound),
    };

    let upload_transfer = match transfer.upload_transfer_mut() {
        Some(upload_transfer) => upload_transfer,
        None => return Err(TransferError::TransferNotFound),
    };

    notify(store::Event::Transfers);

    upload_transfer.name = name.clone();

    transfer.name = match &upload_transfer.name_rel_path {
        Some(name_rel_path) => path_utils::join_path_name(&name_rel_path, &name),
        None => name.clone(),
    };

    match transfer.size {
        SizeInfo::Exact(size) => state.transfers.total_bytes -= size,
        SizeInfo::Estimate(size) => state.transfers.total_bytes -= size,
        SizeInfo::Unknown => {}
    }

    transfer.size = size;

    match size {
        SizeInfo::Exact(size) => state.transfers.total_bytes += size,
        SizeInfo::Estimate(size) => state.transfers.total_bytes += size,
        SizeInfo::Unknown => {}
    }

    transfer.state = TransferState::Transferring;

    Ok(name)
}

pub fn download_transfer_processed(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    name: String,
    size: SizeInfo,
) -> Result<(), TransferError> {
    let transfer = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => transfer,
        None => return Err(TransferError::TransferNotFound),
    };

    notify(store::Event::Transfers);

    transfer.state = TransferState::Transferring;

    transfer.name = name.clone();

    match transfer.size {
        SizeInfo::Exact(size) => state.transfers.total_bytes -= size,
        SizeInfo::Estimate(size) => state.transfers.total_bytes -= size,
        SizeInfo::Unknown => {}
    }

    transfer.size = size;

    match size {
        SizeInfo::Exact(size) => state.transfers.total_bytes += size,
        SizeInfo::Estimate(size) => state.transfers.total_bytes += size,
        SizeInfo::Unknown => {}
    }

    let download_transfer = match transfer.download_transfer_mut() {
        Some(download_transfer) => download_transfer,
        None => return Err(TransferError::TransferNotFound),
    };
    download_transfer.name = name;

    Ok(())
}

pub fn transfer_progress(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    n: i64,
    now: i64,
) {
    let transfer = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => transfer,
        None => return,
    };

    transfer.transferred_bytes += n;

    state.transfers.done_bytes += n;

    if selectors::select_should_notify_progress(state, now) {
        notify(store::Event::Transfers);
        state.transfers.last_progress_update = Some(now);
    }
}

pub fn transfer_done(state: &mut store::State, notify: &store::Notify, id: u32) -> bool {
    let remove = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => {
            notify(store::Event::Transfers);

            transfer.size = SizeInfo::Exact(transfer.transferred_bytes);

            match &transfer.typ {
                TransferType::Upload(..) => state.transfers.transferring_uploads_count -= 1,
                TransferType::Download(..) | TransferType::DownloadReader => {
                    state.transfers.transferring_downloads_count -= 1
                }
            }

            if transfer.is_persistent {
                transfer.started = None;
                transfer.state = TransferState::Done;
            }

            state.transfers.done_count += 1;
            state.transfers.transferring_count -= 1;

            !transfer.is_persistent
        }
        None => false,
    };

    if remove {
        state.transfers.transfers.remove(&id);
    }

    cleanup(state, notify);

    remove
}

pub fn transfer_failed(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    err: TransferError,
    now: i64,
) {
    let is_err_not_retriable = matches!(err, TransferError::NotRetriable);

    if let Some(transfer) = state.transfers.transfers.get_mut(&id) {
        notify(store::Event::Transfers);

        transfer.state = TransferState::Failed { error: err };

        transfer.started = None;

        state.transfers.done_bytes -= transfer.transferred_bytes;

        transfer.transferred_bytes = 0;

        match transfer.size {
            SizeInfo::Exact(size) => state.transfers.failed_bytes += size,
            SizeInfo::Estimate(size) => state.transfers.failed_bytes += size,
            SizeInfo::Unknown => {}
        }

        state.transfers.transferring_count -= 1;

        match &transfer.typ {
            TransferType::Upload(..) => state.transfers.transferring_uploads_count -= 1,
            TransferType::Download(..) | TransferType::DownloadReader => {
                state.transfers.transferring_downloads_count -= 1
            }
        }

        state.transfers.failed_count += 1;

        // this needs to be before unsetting is_retriable
        if transfer.is_retriable {
            state.transfers.retriable_count += 1;
        }

        if is_err_not_retriable {
            transfer.is_retriable = false;
        }
    }

    cleanup(state, notify);

    retry(state, notify, id, RetryInitiator::Autoretry, now);
}

pub fn abort(state: &mut store::State, notify: &store::Notify, id: u32) {
    if let Some(transfer) = state.transfers.transfers.remove(&id) {
        notify(store::Event::Transfers);

        state.transfers.total_count -= 1;

        match transfer.size {
            SizeInfo::Exact(size) => state.transfers.total_bytes -= size,
            SizeInfo::Estimate(size) => state.transfers.total_bytes -= size,
            SizeInfo::Unknown => {}
        }

        match &transfer.state {
            TransferState::Waiting => {}
            TransferState::Processing | TransferState::Transferring => {
                state.transfers.done_bytes -= transfer.transferred_bytes;
                state.transfers.transferring_count -= 1;

                match &transfer.typ {
                    TransferType::Upload(..) => state.transfers.transferring_uploads_count -= 1,
                    TransferType::Download(..) | TransferType::DownloadReader => {
                        state.transfers.transferring_downloads_count -= 1
                    }
                }
            }
            TransferState::Done => {
                match transfer.size {
                    SizeInfo::Exact(size) => state.transfers.done_bytes -= size,
                    SizeInfo::Estimate(size) => state.transfers.done_bytes -= size,
                    SizeInfo::Unknown => {}
                }

                state.transfers.done_count -= 1;
            }
            TransferState::Failed { .. } => {
                state.transfers.done_bytes -= transfer.transferred_bytes;

                match transfer.size {
                    SizeInfo::Exact(size) => state.transfers.failed_bytes -= size,
                    SizeInfo::Estimate(size) => state.transfers.failed_bytes -= size,
                    SizeInfo::Unknown => {}
                }

                state.transfers.failed_count -= 1;

                if transfer.is_retriable {
                    state.transfers.retriable_count -= 1;
                }
            }
        }
    }

    cleanup(state, notify);
}

pub fn abort_all(state: &mut store::State, notify: &store::Notify) -> Vec<u32> {
    let ids: Vec<u32> = state.transfers.transfers.keys().cloned().collect();

    for id in &ids {
        abort(state, notify, *id);
    }

    ids
}

pub fn retry(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    initiator: RetryInitiator,
    now: i64,
) {
    let transfer = match state.transfers.transfers.get(&id) {
        Some(transfer) => transfer,
        None => return,
    };

    if !selectors::can_retry(transfer)
        || (matches!(initiator, RetryInitiator::Autoretry)
            && !selectors::select_can_autoretry(state, transfer))
    {
        return;
    }

    let transfer = match state.transfers.transfers.get_mut(&id) {
        Some(transfer) => transfer,
        None => return,
    };

    notify(store::Event::Transfers);

    transfer.state = TransferState::Waiting;

    state.transfers.failed_count -= 1;

    if transfer.is_retriable {
        state.transfers.retriable_count -= 1;
    }

    match transfer.size {
        SizeInfo::Exact(size) => state.transfers.failed_bytes -= size,
        SizeInfo::Estimate(size) => state.transfers.failed_bytes -= size,
        SizeInfo::Unknown => {}
    }

    if state.transfers.started.is_none() {
        state.transfers.started = Some(now)
    }
}

pub fn retry_all(state: &mut store::State, notify: &store::Notify, now: i64) {
    for id in state
        .transfers
        .transfers
        .keys()
        .cloned()
        .collect::<Vec<_>>()
    {
        retry(state, notify, id, RetryInitiator::User, now);
    }
}

pub fn cleanup(state: &mut store::State, notify: &store::Notify) {
    if state.transfers.transferring_count == 0 && state.transfers.started.is_some() {
        notify(store::Event::Transfers);

        state.transfers.started = None;
    }

    if state.transfers.transfers.is_empty() {
        let new_transfers = TransfersState {
            // we must not reset next_id because calling abort() after all
            // transfers is finished and a new transfer is added could
            // incorrectly abort the new transfer
            next_id: state.transfers.next_id.clone(),
            ..Default::default()
        };

        if new_transfers != state.transfers {
            notify(store::Event::Transfers);

            state.transfers = new_transfers;
        }
    }
}

pub fn create_download_reader_transfer(
    state: &mut store::State,
    notify: &store::Notify,
    id: u32,
    name: String,
    size: SizeInfo,
    now: i64,
) {
    let is_persistent = false;
    let is_retriable = false;
    let is_openable = false;

    create_transfer(
        state,
        notify,
        id,
        CreateTransferType::DownloadReader { name },
        size,
        is_persistent,
        is_retriable,
        is_openable,
    );

    start_transfer(state, notify, id, now);

    if let Some(transfer) = state.transfers.transfers.get_mut(&id) {
        transfer.state = TransferState::Transferring;
    }
}

pub fn cleanup_download_reader_transfer(state: &mut store::State, notify: &store::Notify, id: u32) {
    abort(state, notify, id);
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::{
        common::state::SizeInfo,
        files::file_category::FileCategory,
        repo_files::test_helpers as repo_files_test_helpers,
        repos::test_helpers as repos_test_helpers,
        store::{self, test_helpers as store_test_helpers},
        transfers::{
            mutations::{start_transfer, upload_transfer_processed},
            state::{Transfer, TransferState, TransferType, UploadTransfer},
        },
        types::DecryptedPath,
    };

    use super::{create_transfer, CreateTransferType};

    #[test]
    fn test_upload() {
        let mut state = store::State::default();

        let (repo, cipher, ciphers) =
            repos_test_helpers::create_repo(&mut state, "r1", "m1", "/Vault");
        repo_files_test_helpers::files_loaded(
            &mut state,
            repo.id.0.as_str(),
            "/",
            ciphers.clone(),
            vec![repo_files_test_helpers::create_file("file.txt", &cipher)],
        );

        let (notify, _, _) = store_test_helpers::mutation();
        create_transfer(
            &mut state,
            &notify,
            1,
            CreateTransferType::Upload {
                repo_id: repo.id.clone(),
                parent_path: DecryptedPath("/".into()),
                name: "file.txt".into(),
            },
            SizeInfo::Exact(10),
            false,
            true,
            false,
        );

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Waiting,
                transferred_bytes: 0,
                attempts: 0,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        start_transfer(&mut state, &notify, 1, 2);

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Processing,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        let name = upload_transfer_processed(&mut state, &notify, 1, SizeInfo::Exact(11)).unwrap();
        assert_eq!(name, "file (1).txt");

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file (1).txt".into(),
                }),
                name: "file (1).txt".into(),
                size: SizeInfo::Exact(11),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Transferring,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );
    }

    #[test]
    fn test_transfer_added_upload_name_path() {
        let mut state = store::State::default();

        let (repo, cipher, ciphers) =
            repos_test_helpers::create_repo(&mut state, "r1", "m1", "/Vault");
        repo_files_test_helpers::files_loaded(
            &mut state,
            repo.id.0.as_str(),
            "/",
            ciphers.clone(),
            vec![repo_files_test_helpers::create_dir("path", &cipher)],
        );
        repo_files_test_helpers::files_loaded(
            &mut state,
            repo.id.0.as_str(),
            "/path",
            ciphers.clone(),
            vec![repo_files_test_helpers::create_dir("to", &cipher)],
        );
        repo_files_test_helpers::files_loaded(
            &mut state,
            repo.id.0.as_str(),
            "/path/to",
            ciphers.clone(),
            vec![repo_files_test_helpers::create_file("file.txt", &cipher)],
        );

        let (notify, _, _) = store_test_helpers::mutation();
        create_transfer(
            &mut state,
            &notify,
            1,
            CreateTransferType::Upload {
                repo_id: repo.id.clone(),
                parent_path: DecryptedPath("/".into()),
                name: "path/to/file.txt".into(),
            },
            SizeInfo::Exact(10),
            false,
            true,
            false,
        );

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/path/to".into()),
                    name_rel_path: Some("path/to".into()),
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "path/to/file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Waiting,
                transferred_bytes: 0,
                attempts: 0,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        start_transfer(&mut state, &notify, 1, 2);

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/path/to".into()),
                    name_rel_path: Some("path/to".into()),
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "path/to/file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Processing,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        let name = upload_transfer_processed(&mut state, &notify, 1, SizeInfo::Exact(11)).unwrap();
        assert_eq!(name, "file (1).txt");

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/path/to".into()),
                    name_rel_path: Some("path/to".into()),
                    original_name: "file.txt".into(),
                    name: "file (1).txt".into(),
                }),
                name: "path/to/file (1).txt".into(),
                size: SizeInfo::Exact(11),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Transferring,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );
    }

    #[test]
    fn test_upload_unused_name_transfers() {
        let mut state = store::State::default();

        let (repo, _, _) = repos_test_helpers::create_repo(&mut state, "r1", "m1", "/Vault");

        let (notify, _, _) = store_test_helpers::mutation();
        create_transfer(
            &mut state,
            &notify,
            1,
            CreateTransferType::Upload {
                repo_id: repo.id.clone(),
                parent_path: DecryptedPath("/".into()),
                name: "file.txt".into(),
            },
            SizeInfo::Exact(10),
            false,
            true,
            false,
        );

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: None,
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Waiting,
                transferred_bytes: 0,
                attempts: 0,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        create_transfer(
            &mut state,
            &notify,
            2,
            CreateTransferType::Upload {
                repo_id: repo.id.clone(),
                parent_path: DecryptedPath("/".into()),
                name: "file.txt".into(),
            },
            SizeInfo::Exact(10),
            false,
            true,
            false,
        );

        let (notify, _, _) = store_test_helpers::mutation();
        start_transfer(&mut state, &notify, 2, 2);

        let (notify, _, _) = store_test_helpers::mutation();
        let name = upload_transfer_processed(&mut state, &notify, 2, SizeInfo::Exact(10)).unwrap();
        assert_eq!(name, "file.txt");

        let (notify, _, _) = store_test_helpers::mutation();
        start_transfer(&mut state, &notify, 1, 2);

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file.txt".into(),
                }),
                name: "file.txt".into(),
                size: SizeInfo::Exact(10),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Processing,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );

        let (notify, _, _) = store_test_helpers::mutation();
        let name = upload_transfer_processed(&mut state, &notify, 1, SizeInfo::Exact(11)).unwrap();
        assert_eq!(name, "file (1).txt");

        assert_eq!(
            state.transfers.transfers.get(&1).unwrap(),
            &Transfer {
                id: 1,
                typ: TransferType::Upload(UploadTransfer {
                    repo_id: repo.id.clone(),
                    parent_path: DecryptedPath("/".into()),
                    name_rel_path: None,
                    original_name: "file.txt".into(),
                    name: "file (1).txt".into(),
                }),
                name: "file (1).txt".into(),
                size: SizeInfo::Exact(11),
                category: FileCategory::Text,
                started: Some(2),
                is_persistent: false,
                is_retriable: true,
                is_openable: false,
                state: TransferState::Transferring,
                transferred_bytes: 0,
                attempts: 1,
                order: 0,
            }
        );
    }
}
