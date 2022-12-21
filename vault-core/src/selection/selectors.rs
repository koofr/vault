use super::state::{Selection, SelectionSummary};

pub fn select_selection_summary(state: &Selection, total: usize) -> SelectionSummary {
    let len = state.selection.len();

    if len == 0 {
        SelectionSummary::None
    } else if len == total {
        SelectionSummary::All
    } else {
        SelectionSummary::Partial
    }
}
