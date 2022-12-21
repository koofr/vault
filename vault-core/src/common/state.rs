#[derive(Clone, PartialEq, Eq)]
pub enum Status<E: Clone> {
    Initial,
    Loading,
    Loaded,
    Reloading,
    Error { error: E },
}

impl<E: Clone> Default for Status<E> {
    fn default() -> Self {
        Self::Initial
    }
}
