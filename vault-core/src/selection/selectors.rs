use std::{fmt::Debug, hash::Hash};

use super::state::{Selection, SelectionSummary};

pub fn select_selection_summary<Item: Debug + Clone + PartialEq + Eq + Hash>(
    state: &Selection<Item>,
    total: usize,
) -> SelectionSummary {
    let len = state.selection.len();

    if len == 0 {
        SelectionSummary::None
    } else if len == total {
        SelectionSummary::All
    } else {
        SelectionSummary::Partial
    }
}
