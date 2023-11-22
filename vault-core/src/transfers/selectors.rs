use std::{cmp::min, time::Duration};

use crate::{
    common::state::RemainingTime, config::state::TransfersConfig,
    repo_files::selectors as repo_files_selectors, store, types::DecryptedName,
};

use super::state::{Transfer, TransferState, TransferType, TransfersState, UploadTransfer};

pub fn can_retry(transfer: &Transfer) -> bool {
    matches!(transfer.state, TransferState::Failed { .. }) && transfer.is_retriable
}

pub fn can_open(transfer: &Transfer) -> bool {
    matches!(transfer.state, TransferState::Done { .. }) && transfer.is_openable
}

pub fn get_percentage(done_bytes: i64, total_bytes: i64) -> u8 {
    if total_bytes > 0 {
        min(
            ((done_bytes as f64 * 100.0) / total_bytes as f64).floor() as u8,
            100,
        )
    } else {
        0
    }
}

pub fn transfer_duration(transfer: &Transfer, now: i64) -> Option<Duration> {
    transfer.started.map(|started| match now - started {
        x if x < 0 => Duration::ZERO,
        x => Duration::from_millis(x as u64),
    })
}

pub fn transfers_duration(transfers: &TransfersState, now: i64) -> Duration {
    match transfers.started {
        Some(started) => match now - started {
            x if x < 0 => Duration::ZERO,
            x => Duration::from_millis(x as u64),
        },
        None => Duration::ZERO,
    }
}

pub fn transfer_percentage(transfer: &Transfer) -> Option<u8> {
    transfer
        .size
        .exact_or_estimate()
        .map(|size| get_percentage(transfer.transferred_bytes, size))
}

pub fn select_config(state: &store::State) -> &TransfersConfig {
    &state.config.transfers
}

pub fn select_transfer<'a>(state: &'a store::State, id: u32) -> Option<&'a Transfer> {
    state.transfers.transfers.get(&id)
}

pub fn select_transfers<'a>(state: &'a store::State) -> Vec<&'a Transfer> {
    let mut files: Vec<&'a Transfer> = state.transfers.transfers.values().collect();

    files.sort_by_key(|file| file.order);

    files
}

pub fn select_should_notify_progress(state: &store::State, now: i64) -> bool {
    state.transfers.last_progress_update.is_none()
        || matches!(
            state.transfers.last_progress_update, Some(last_progress_update)
            if last_progress_update < now - select_config(state).progress_throttle.as_millis() as i64
        )
}

pub fn select_is_active(state: &store::State) -> bool {
    !state.transfers.transfers.is_empty()
}

pub fn select_is_transferring(state: &store::State) -> bool {
    state.transfers.transferring_count > 0
}

pub fn select_is_all_done(state: &store::State) -> bool {
    state.transfers.total_count > 0 && state.transfers.total_count == state.transfers.done_count
}

pub fn select_can_retry_all(state: &store::State) -> bool {
    state.transfers.retriable_count > 0
}

pub fn select_can_abort_all(state: &store::State) -> bool {
    state.transfers.total_count > 0
}

pub fn select_can_autoretry(state: &store::State, transfer: &Transfer) -> bool {
    transfer.attempts < select_config(state).autoretry_attempts
}

pub fn select_remaining_count(state: &store::State) -> usize {
    state.transfers.total_count
        - state.transfers.done_count
        - state.transfers.failed_count
        - state.transfers.transferring_count
}

pub fn select_remaining_bytes(state: &store::State) -> i64 {
    state.transfers.total_bytes - state.transfers.done_bytes - state.transfers.failed_bytes
}

pub fn select_bytes_done(state: &store::State) -> i64 {
    state.transfers.done_bytes
}

pub fn select_duration(state: &store::State, now: i64) -> Duration {
    transfers_duration(&state.transfers, now)
}

pub fn select_remaining_time(state: &store::State, now: i64) -> RemainingTime {
    let bytes_done = select_bytes_done(state);

    // before transfers actually start, the remaining time can be really large so we default to 0s
    if bytes_done == 0 {
        return RemainingTime::from_seconds(0.0);
    }

    let speed_bytes = bytes_done as f64 / select_duration(state, now).as_secs_f64();
    let remaining_bytes = select_remaining_bytes(state);
    let extra_seconds = (select_remaining_count(state) as i64
        * select_config(state).min_time_per_file.as_millis() as i64)
        / 1000;

    let total_seconds = (remaining_bytes as f64 / speed_bytes) + extra_seconds as f64;

    RemainingTime::from_seconds(total_seconds)
}

pub fn select_percentage(state: &store::State) -> u8 {
    get_percentage(state.transfers.done_bytes, state.transfers.total_bytes)
}

pub fn select_next_transfer<'a>(state: &'a store::State) -> Option<&'a Transfer> {
    select_next_upload_transfer(state).or_else(|| select_next_download_transfer(state))
}

pub fn select_next_upload_transfer<'a>(state: &'a store::State) -> Option<&'a Transfer> {
    if state.transfers.transferring_uploads_count >= select_config(state).upload_concurrency {
        return None;
    }

    select_transfers(state)
        .into_iter()
        .find(|transfer| match (&transfer.state, &transfer.typ) {
            (TransferState::Waiting, TransferType::Upload(..)) => true,
            _ => false,
        })
}

pub fn select_next_download_transfer<'a>(state: &'a store::State) -> Option<&'a Transfer> {
    if state.transfers.transferring_downloads_count >= select_config(state).download_concurrency {
        return None;
    }

    select_transfers(state)
        .into_iter()
        .find(|transfer| match (&transfer.state, &transfer.typ) {
            (TransferState::Waiting, TransferType::Download) => true,
            _ => false,
        })
}

pub fn select_unused_name(
    state: &store::State,
    transfer: &Transfer,
    upload_transfer: &UploadTransfer,
) -> DecryptedName {
    // names from repo files
    let mut used_names = repo_files_selectors::select_used_names(
        state,
        &upload_transfer.repo_id,
        &upload_transfer.parent_path,
    );

    // add names from transfers
    for t in select_transfers(state) {
        match &t.typ {
            TransferType::Upload(upload_t) => {
                // we are only interested in transfers with state Transferring.
                // we are not interested in Done transfers because their results
                // will already be in repo_files used names
                if t.id != transfer.id
                    && upload_t.parent_file_id == upload_transfer.parent_file_id
                    && matches!(t.state, TransferState::Transferring)
                {
                    used_names.insert(upload_t.current_name.to_lowercase());
                }
            }
            _ => {}
        }
    }

    repo_files_selectors::get_unused_name(used_names, &upload_transfer.original_name)
}
