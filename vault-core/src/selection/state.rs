use std::collections::HashSet;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SelectionSummary {
    None,
    Partial,
    All,
}

#[derive(Clone, Default, Debug)]
pub struct Selection {
    pub selection: HashSet<String>,
    pub last_selected: Option<String>,
    pub range_anchor: Option<String>,
}
