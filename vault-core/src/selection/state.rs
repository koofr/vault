use std::{collections::HashSet, fmt::Debug, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionSummary {
    None,
    Partial,
    All,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Selection<Item: Debug + Clone + PartialEq + Eq + Hash> {
    pub selection: HashSet<Item>,
    pub last_selected: Option<Item>,
    pub range_anchor: Option<Item>,
}
