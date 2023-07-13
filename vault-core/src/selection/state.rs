use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionSummary {
    None,
    Partial,
    All,
}

#[derive(Debug, Clone, Default)]
pub struct Selection {
    pub selection: HashSet<String>,
    pub last_selected: Option<String>,
    pub range_anchor: Option<String>,
}
