use super::file_category::FileCategory;

#[derive(Clone, Debug)]
pub struct FilesFilter {
    pub categories: Vec<FileCategory>,
    pub exts: Vec<String>,
}

impl FilesFilter {
    pub fn matches(&self, ext: Option<&str>, category: &FileCategory) -> bool {
        self.categories.iter().any(|x| x == category)
            || ext
                .map(|ext| self.exts.iter().any(|x| x == ext))
                .unwrap_or(false)
    }
}
