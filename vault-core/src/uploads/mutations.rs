use std::cmp::min;

use crate::{
    file_types::file_category::{ext_to_file_category, FileCategory},
    store,
    utils::name_utils,
};

use super::{
    errors::UploadError,
    selectors,
    state::{FileUpload, FileUploadState},
};

pub fn get_next_id(state: &mut store::State) -> u32 {
    let upload_id = state.uploads.next_id;

    state.uploads.next_id += 1;

    upload_id
}

pub struct FileUploadAdded {
    pub id: u32,
    pub repo_id: String,
    pub parent_path: String,
    pub name: String,
    pub size: Option<i64>,
    pub is_persistent: bool,
}

pub fn file_upload_added(state: &mut store::State, file: FileUploadAdded, now: i64) {
    let category = name_utils::name_to_ext(&file.name.to_lowercase())
        .and_then(ext_to_file_category)
        .unwrap_or(FileCategory::Generic);

    state.uploads.files.insert(
        file.id.clone(),
        FileUpload {
            id: file.id,
            repo_id: file.repo_id,
            parent_path: file.parent_path,
            name: file.name,
            autorename_name: None,
            size: file.size,
            category,
            started: now,
            is_persistent: file.is_persistent,
            state: FileUploadState::Waiting,
            uploaded_bytes: 0,
            attempts: 0,
            order: state.uploads.total_count,
        },
    );

    state.uploads.total_count += 1;

    if let Some(size) = file.size {
        state.uploads.total_bytes += size;
    }
}

pub fn file_upload_uploading(state: &mut store::State, id: u32, now: i64) {
    let autorename_name = selectors::select_unused_name(state, id);

    if let Some(file) = state.uploads.files.get_mut(&id) {
        file.autorename_name = autorename_name;
        file.state = FileUploadState::Uploading;
        file.uploaded_bytes = 0;
        file.attempts += 1;

        state.uploads.uploading_count += 1;

        if state.uploads.started.is_none() {
            state.uploads.started = Some(now)
        }
    }
}

pub fn file_upload_progress(state: &mut store::State, id: u32, n: i64) {
    if let Some(file) = state.uploads.files.get_mut(&id) {
        file.uploaded_bytes += n;

        if let Some(size) = file.size {
            file.uploaded_bytes = min(file.uploaded_bytes, size);
        }

        state.uploads.done_bytes += n;
    }
}

pub fn file_upload_done(state: &mut store::State, id: u32) {
    if let Some(file) = state.uploads.files.get_mut(&id) {
        if file.size.is_none() {
            file.size = Some(file.uploaded_bytes);
        }
    }

    state.uploads.done_count += 1;
    state.uploads.uploading_count -= 1;

    if state
        .uploads
        .files
        .get(&id)
        .map(|file| file.is_persistent)
        .unwrap_or(false)
    {
        if let Some(file) = state.uploads.files.get_mut(&id) {
            file.state = FileUploadState::Done;
        }
    } else {
        state.uploads.files.remove(&id);
    }

    if state.uploads.uploading_count == 0 {
        state.uploads.started = None
    }

    if state.uploads.files.is_empty() {
        reset(state);
    }
}

pub fn file_upload_failed(state: &mut store::State, id: u32, err: UploadError) {
    if let Some(file) = state.uploads.files.get_mut(&id) {
        file.state = FileUploadState::Failed { error: err };

        state.uploads.done_bytes -= file.uploaded_bytes;
        if let Some(size) = file.size {
            state.uploads.failed_bytes += size;
        }
        state.uploads.uploading_count -= 1;
        state.uploads.failed_count += 1;
    }

    if state.uploads.uploading_count == 0 {
        state.uploads.started = None
    }
}

pub fn file_upload_abort(state: &mut store::State, id: u32) {
    if let Some(file) = state.uploads.files.get(&id) {
        state.uploads.total_count -= 1;

        if let Some(size) = file.size {
            state.uploads.total_bytes -= size;
        }

        match &file.state {
            FileUploadState::Waiting => {}
            FileUploadState::Uploading => {
                state.uploads.done_bytes -= file.uploaded_bytes;
                state.uploads.uploading_count -= 1;
            }
            FileUploadState::Done => {
                if let Some(size) = file.size {
                    state.uploads.done_bytes -= size;
                }

                state.uploads.done_count -= 1;
            }
            FileUploadState::Failed { .. } => {
                state.uploads.done_bytes -= file.uploaded_bytes;

                if let Some(size) = file.size {
                    state.uploads.failed_bytes -= size;
                }

                state.uploads.failed_count -= 1;
            }
        }
    }

    state.uploads.files.remove(&id);

    if state.uploads.uploading_count == 0 {
        state.uploads.started = None
    }

    if state.uploads.files.is_empty() {
        reset(state);
    }
}

pub fn file_upload_abort_all(state: &mut store::State) -> Vec<u32> {
    let ids: Vec<u32> = state.uploads.files.keys().cloned().collect();

    for id in &ids {
        file_upload_abort(state, *id);
    }

    ids
}

pub fn file_upload_retry(state: &mut store::State, id: u32, now: i64) {
    if let Some(file) = state.uploads.files.get_mut(&id) {
        file.state = FileUploadState::Waiting;
        file.uploaded_bytes;

        state.uploads.failed_count -= 1;

        if let Some(size) = file.size {
            state.uploads.failed_bytes -= size;
        }
    }

    if state.uploads.started.is_none() {
        state.uploads.started = Some(now)
    }
}

pub fn file_upload_retry_all(state: &mut store::State, now: i64) {
    for id in state.uploads.files.keys().cloned().collect::<Vec<u32>>() {
        file_upload_retry(state, id, now);
    }
}

pub fn reset(state: &mut store::State) {
    state.uploads = Default::default();
}
