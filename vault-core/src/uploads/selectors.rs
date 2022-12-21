use std::{cmp::min, collections::HashSet};

use crate::{repo_files::selectors as repo_files_selectors, store, utils::name_utils};

use super::state::{FileUpload, FileUploadState, RemainingTime};

const MAX_CONCURRENCY: u32 = 3;
const MAX_AUTO_ATTEMPTS: u32 = 5;
const MIN_TIME_PER_FILE_MS: u64 = 500;

pub fn select_file<'a>(state: &'a store::State, id: u32) -> Option<&'a FileUpload> {
    state.uploads.files.get(&id)
}

pub fn select_files<'a>(state: &'a store::State) -> Vec<&'a FileUpload> {
    let mut files: Vec<&'a FileUpload> = state.uploads.files.values().collect();

    files.sort_by_key(|file| file.order);

    files
}

pub fn select_is_active(state: &store::State) -> bool {
    !state.uploads.files.is_empty()
}

pub fn select_is_uploading(state: &store::State) -> bool {
    state.uploads.uploading_count > 0
}

pub fn select_is_all_done(state: &store::State) -> bool {
    state.uploads.total_count > 0 && state.uploads.total_count == state.uploads.done_count
}

pub fn select_can_retry(state: &store::State) -> bool {
    state.uploads.failed_count > 0
}

pub fn select_can_abort(state: &store::State) -> bool {
    state.uploads.total_count > state.uploads.done_count
}

pub fn select_remaining_count(state: &store::State) -> u32 {
    state.uploads.total_count
        - state.uploads.done_count
        - state.uploads.failed_count
        - state.uploads.uploading_count
}

pub fn select_remaining_bytes(state: &store::State) -> i64 {
    state.uploads.total_bytes - state.uploads.done_bytes - state.uploads.failed_bytes
}

pub fn select_bytes_per_second(state: &store::State, now: i64) -> f64 {
    match state.uploads.started {
        Some(started) => state.uploads.done_bytes as f64 / ((now - started) / 1000) as f64,
        None => 0.0,
    }
}

pub fn select_remaining_time(state: &store::State, now: i64) -> RemainingTime {
    let speed_bytes = select_bytes_per_second(state, now);
    let remaining_bytes = select_remaining_bytes(state);
    let extra_seconds = (select_remaining_count(state) as u64 * MIN_TIME_PER_FILE_MS) / 1000;

    let total_seconds = (remaining_bytes as f64 / speed_bytes) + extra_seconds as f64;

    RemainingTime::from_seconds(total_seconds)
}

pub fn select_percentage(state: &store::State) -> u8 {
    if state.uploads.total_bytes > 0 {
        min(
            ((state.uploads.done_bytes as f64 * 100.0) / state.uploads.total_bytes as f64).floor()
                as u8,
            100,
        )
    } else {
        0
    }
}

pub fn select_next_file<'a>(state: &'a store::State) -> Option<&'a FileUpload> {
    if state.uploads.uploading_count >= MAX_CONCURRENCY {
        return None;
    }

    select_files(state)
        .into_iter()
        .find(|file| match file.state {
            FileUploadState::Waiting => true,
            _ => false,
        })
}

pub fn select_can_auto_retry(state: &store::State, id: u32) -> bool {
    state
        .uploads
        .files
        .get(&id)
        .map(|file| file.attempts < MAX_AUTO_ATTEMPTS)
        .unwrap_or(false)
}

pub fn select_unused_name(state: &store::State, id: u32) -> Option<String> {
    let file = state.uploads.files.get(&id)?;

    let mut used_names = HashSet::<String>::new();

    // fill children names from repo_files
    for f in repo_files_selectors::select_files(state, &file.repo_id, &file.parent_path) {
        if let Ok(name) = f.decrypted_name() {
            used_names.insert(name.to_lowercase());
        }
    }

    let parent_id = file.parent_id();

    // fill names from upload files
    for f in select_files(state) {
        if f.id != file.id && f.parent_id() == parent_id {
            used_names.insert(f.name.to_lowercase());

            if let Some(name) = &f.autorename_name {
                used_names.insert(name.to_lowercase());
            }
        }
    }

    Some(name_utils::unused_name(&file.name, |name| {
        used_names.contains(&name.to_lowercase())
    }))
}
