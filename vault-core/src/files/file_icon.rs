use crate::file_types::file_category::FileCategory;

#[derive(Debug, Clone, PartialEq)]
pub struct FileIconAttrs {
    pub category: FileCategory,
    pub is_dl: bool,
    pub is_ul: bool,
    pub is_export: bool,
    pub is_import: bool,
    pub is_android: bool,
    pub is_ios: bool,
    pub is_vault_repo: bool,
    pub is_error: bool,
}
