use std::{collections::HashSet, fmt::Debug, hash::Hash};

use super::state::Selection;

pub fn clear_selection<Item: Debug + Clone + PartialEq + Eq + Hash>(state: &mut Selection<Item>) {
    state.selection.clear();
    state.last_selected = None;
    state.range_anchor = None;
}

pub fn set_selection<Item: Debug + Clone + PartialEq + Eq + Hash>(
    state: &mut Selection<Item>,
    items: Vec<Item>,
) {
    clear_selection(state);

    for item in items {
        state.selection.insert(item);
    }
}

pub fn update_selection<Item: Debug + Clone + PartialEq + Eq + Hash>(
    state: &mut Selection<Item>,
    items: HashSet<Item>,
) -> bool {
    let mut dirty = false;

    let old_len = state.selection.len();

    state.selection.retain(|item| items.contains(item));

    if state.selection.len() != old_len {
        dirty = true;
    }

    if let Some(last_selected) = &state.last_selected {
        if !items.contains(last_selected) {
            state.last_selected = None;
            dirty = true;
        }
    }

    if let Some(range_anchor) = &state.range_anchor {
        if !items.contains(range_anchor) {
            state.range_anchor = None;
            dirty = true;
        }
    }

    dirty
}

/// extend: ctrl_key || checkbox
/// range: shift_key
pub fn select_item<Item: Debug + Clone + PartialEq + Eq + Hash>(
    state: &mut Selection<Item>,
    items: Vec<Item>,
    item: Item,
    extend: bool,
    range: bool,
    force: bool,
) {
    let selected_count = state.selection.len();
    let was_empty = selected_count == 0;
    let was_single_selection = selected_count == 1;
    let was_selected = state.selection.contains(&item);

    if !range || (range && was_empty) || state.last_selected.is_none() {
        state.last_selected = None;
        state.range_anchor = None;

        if was_empty {
            state.selection.insert(item.clone());
            state.last_selected = Some(item);
        } else if was_single_selection {
            if was_selected {
                if force {
                    state.last_selected = Some(item);
                } else {
                    state.selection.clear();
                }
            } else {
                if extend {
                    state.selection.insert(item.clone());
                    state.last_selected = Some(item);
                } else {
                    state.selection.clear();
                    state.selection.insert(item.clone());
                    state.last_selected = Some(item);
                }
            }
        } else {
            if was_selected {
                if force {
                    state.last_selected = Some(item);
                } else {
                    if extend {
                        state.selection.remove(&item);
                        state.last_selected = Some(item);
                    } else {
                        state.selection.clear();
                        state.selection.insert(item.clone());
                        state.last_selected = Some(item);
                    }
                }
            } else {
                if extend {
                    state.selection.insert(item.clone());
                    state.last_selected = Some(item);
                } else {
                    state.selection.clear();
                    state.selection.insert(item.clone());
                    state.last_selected = Some(item);
                }
            }
        }
    } else {
        if let Some(end) = state.range_anchor.as_ref().or(state.last_selected.as_ref()) {
            let start_idx = items.iter().position(|it| it == &item);
            let end_idx = items.iter().position(|it| it == end);

            if let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) {
                let mut range_start = start_idx;
                let mut range_end = end_idx + 1;

                if start_idx > end_idx {
                    range_start = end_idx;
                    range_end = start_idx + 1;
                }

                if !extend {
                    state.selection.clear();
                }

                for idx in range_start..range_end {
                    state.selection.insert(items[idx].clone());
                }
            }
        }

        if state.range_anchor.is_none() {
            state.range_anchor = state.last_selected.clone();
        }

        state.last_selected = Some(item.to_owned());
    }
}
