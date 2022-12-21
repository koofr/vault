use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Bookmarks {
    pub bookmarks: Vec<super::Bookmark>,
}
