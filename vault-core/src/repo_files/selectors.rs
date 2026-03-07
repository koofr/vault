use std::collections::{HashMap, HashSet};

use crate::{
    cipher::Cipher,
    files::{
        content_type::ext_to_content_type,
        file_category::{FileCategory, ext_to_file_category},
    },
    intl,
    remote::RemoteError,
    remote_files::{selectors as remote_files_selectors, state::RemoteFile},
    repos::{errors::RepoNotFoundError, selectors as repos_selectors},
    sort::state::SortGrouping,
    store,
    types::{
        DecryptedName, DecryptedNameLower, EncryptedName, EncryptedPath, MountId, RemotePath,
        RepoFileId, RepoId,
    },
    utils::{name_utils, remote_path_utils, repo_encrypted_path_utils},
};

use super::{
    errors::{FileNameError, RenameFileError, RepoFilesErrors},
    state::{RepoFile, RepoFileType, RepoFilesBreadcrumb, RepoFilesSort, RepoFilesSortField},
};

pub fn get_file_id(repo_id: &RepoId, path: &EncryptedPath) -> RepoFileId {
    RepoFileId(format!("{}:{}", repo_id.0, path.0))
}

pub fn get_file_unique_name(remote_file_unique_id: &str, ext: Option<&str>) -> String {
    match ext {
        Some(ext) => format!("{}.{}", remote_file_unique_id, ext),
        None => remote_file_unique_id.to_owned(),
    }
}

pub fn get_file_ext_content_type_category<'a>(
    name_lower: &'a str,
) -> (Option<String>, Option<String>, FileCategory) {
    let ext = name_utils::name_to_ext(name_lower);

    (
        ext.map(str::to_string),
        ext.and_then(ext_to_content_type).map(str::to_string),
        ext.and_then(ext_to_file_category)
            .unwrap_or(FileCategory::Generic),
    )
}

pub fn select_children<'a>(
    state: &'a store::State,
    file_id: &RepoFileId,
) -> Option<&'a Vec<RepoFileId>> {
    state.repo_files.children.get(file_id)
}

pub fn select_files<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
    path: &EncryptedPath,
) -> impl Iterator<Item = &'a RepoFile> {
    match select_children(state, &get_file_id(repo_id, path)) {
        Some(ids) => select_files_from_ids(state, ids),
        None => select_files_from_ids(state, &[]),
    }
}

pub fn select_recent<'a>(state: &'a store::State, repo_id: &RepoId) -> Option<&'a Vec<RepoFileId>> {
    state.repo_files.recent.get(repo_id)
}

pub fn select_recent_files<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
) -> impl Iterator<Item = &'a RepoFile> {
    match select_recent(state, repo_id) {
        Some(ids) => select_files_from_ids(state, ids),
        None => select_files_from_ids(state, &[]),
    }
}

pub fn select_files_from_ids<'a>(
    state: &'a store::State,
    ids: &'a [RepoFileId],
) -> impl Iterator<Item = &'a RepoFile> {
    ids.iter().filter_map(|id| select_file(state, id))
}

pub fn select_file<'a>(state: &'a store::State, file_id: &RepoFileId) -> Option<&'a RepoFile> {
    state.repo_files.files.get(file_id)
}

pub fn select_file_name<'a>(
    state: &'a store::State,
    file: &'a RepoFile,
) -> Result<&'a DecryptedName, FileNameError> {
    if file.encrypted_path.is_root() {
        Ok(repos_selectors::select_repo(state, &file.repo_id).map(|repo| &repo.name)?)
    } else {
        Ok(file.decrypted_name()?)
    }
}

pub fn select_remote_file<'a>(
    state: &'a store::State,
    file: &'a RepoFile,
) -> Option<&'a RemoteFile> {
    remote_files_selectors::select_file(
        state,
        &remote_files_selectors::get_file_id(&file.mount_id, &file.remote_path.to_lowercase()),
    )
}

pub fn select_repo_path_to_mount_path(
    state: &store::State,
    repo_id: &RepoId,
    path: &EncryptedPath,
) -> Result<(MountId, RemotePath), RepoNotFoundError> {
    let repo = repos_selectors::select_repo(state, repo_id)?;

    let remote_path = remote_path_utils::join_paths(&repo.path, &RemotePath(path.0.clone()));

    Ok((repo.mount_id.clone(), remote_path))
}

pub fn select_repo_path_to_remote_file<'a>(
    state: &'a store::State,
    repo_id: &RepoId,
    path: &EncryptedPath,
) -> Option<&'a RemoteFile> {
    select_repo_path_to_mount_path(state, repo_id, path)
        .ok()
        .and_then(|(mount_id, remote_path)| {
            remote_files_selectors::select_file(
                state,
                &remote_files_selectors::get_file_id(&mount_id, &remote_path.to_lowercase()),
            )
        })
}

pub fn select_mount_path_to_repo_id<'a>(
    state: &'a store::State,
    mount_id: &MountId,
    path: &RemotePath,
) -> Option<&'a RepoId> {
    for path in remote_path_utils::paths_chain(path) {
        if let Some(repo_id) =
            state
                .repos
                .repo_ids_by_remote_file_id
                .get(&remote_files_selectors::get_file_id(
                    &mount_id,
                    &path.to_lowercase(),
                ))
        {
            return Some(repo_id);
        }
    }

    None
}

pub fn select_is_root_loaded(state: &store::State, repo_id: &RepoId, path: &EncryptedPath) -> bool {
    state
        .repo_files
        .loaded_roots
        .contains(&get_file_id(&repo_id, &path))
}

pub fn select_is_recent_loaded(state: &store::State, repo_id: &RepoId) -> bool {
    state.repo_files.recent.contains_key(repo_id)
}

pub fn check_name_valid(name: &DecryptedName) -> Result<(), RemoteError> {
    name_utils::validate_name(&name.0).map_err(|_| RepoFilesErrors::invalid_path())
}

pub fn select_check_new_name_valid(
    state: &store::State,
    repo_id: &RepoId,
    parent_path: &EncryptedPath,
    new_name: &DecryptedName,
    encrypted_new_name: &EncryptedName,
) -> Result<(), RemoteError> {
    check_name_valid(new_name)?;

    let new_path = repo_encrypted_path_utils::join_path_name(parent_path, encrypted_new_name);

    match select_children(state, &get_file_id(repo_id, parent_path)) {
        Some(ids) => {
            if ids.contains(&get_file_id(repo_id, &new_path)) {
                Err(RepoFilesErrors::already_exists())
            } else {
                Ok(())
            }
        }
        None => Ok(()),
    }
}

pub fn select_check_rename_file(
    state: &store::State,
    repo_id: &RepoId,
    path: &EncryptedPath,
    name: &DecryptedName,
    encrypted_new_name: &EncryptedName,
) -> Result<(), RenameFileError> {
    select_file(state, &get_file_id(repo_id, path)).ok_or_else(RepoFilesErrors::not_found)?;

    let parent_path = match repo_encrypted_path_utils::parent_path(path) {
        Some(parent_path) => parent_path,
        None => return Err(RenameFileError::RenameRoot),
    };

    select_check_new_name_valid(state, repo_id, &parent_path, name, encrypted_new_name)?;

    Ok(())
}

pub fn select_breadcrumbs(
    state: &store::State,
    repo_id: &RepoId,
    path: &EncryptedPath,
    cipher: &Cipher,
) -> Vec<RepoFilesBreadcrumb> {
    let repo = match repos_selectors::select_repo(state, repo_id) {
        Ok(repo) => repo,
        Err(_) => {
            return vec![];
        }
    };

    let paths = repo_encrypted_path_utils::paths_chain(path);
    let paths_len: usize = paths.len();

    paths
        .into_iter()
        .enumerate()
        .map(|(i, path)| {
            let id = get_file_id(repo_id, &path);
            let name = match repo_encrypted_path_utils::path_to_name(&path) {
                Some(name) => cipher
                    .decrypt_filename(&name)
                    .map(|x| x.0)
                    .unwrap_or(name.0),
                None => repo.name.0.clone(),
            };

            RepoFilesBreadcrumb {
                id,
                repo_id: repo_id.to_owned(),
                path,
                name,
                last: i == paths_len - 1,
            }
        })
        .collect()
}

pub fn get_recent_breadcrumbs(
    repo_id: &RepoId,
    intl_service: &intl::IntlService,
) -> Vec<RepoFilesBreadcrumb> {
    let name = intl::format_message!(
        intl_service,
        "core.repo_files.items.recent",
        "Label for the Recent files in the breadcrumbs.",
        "Recent files"
    );

    vec![RepoFilesBreadcrumb {
        id: RepoFileId(format!("recent:{}", repo_id.0)),
        repo_id: repo_id.to_owned(),
        path: EncryptedPath("/".into()),
        name: name,
        last: true,
    }]
}

pub fn select_sorted_files<RFI, FI>(
    repo_files_files: &HashMap<RepoFileId, RepoFile>,
    file_ids: FI,
    sort: &RepoFilesSort,
) -> Vec<RepoFileId>
where
    RFI: std::ops::Deref<Target = RepoFileId>,
    FI: IntoIterator<Item = RFI>,
{
    let RepoFilesSort {
        field,
        direction,
        grouping,
    } = sort;

    match grouping {
        SortGrouping::DirsFirst => {
            let (mut dirs, mut files): (Vec<_>, Vec<_>) = file_ids
                .into_iter()
                .filter_map(|id| repo_files_files.get(&id))
                .partition(|f| f.typ == RepoFileType::Dir);

            match field {
                RepoFilesSortField::Name => {
                    dirs.sort_by(|a, b| {
                        direction.ordering(natord::compare(a.name_lower_force(), b.name_lower_force()))
                    });
                    files.sort_by(|a, b| {
                        direction.ordering(natord::compare(a.name_lower_force(), b.name_lower_force()))
                    });
                }
                RepoFilesSortField::Size => {
                    dirs.sort_by(|a, b| a.name_lower_force().cmp(b.name_lower_force()));
                    files.sort_by(|a, b| direction.ordering(a.size_force().cmp(&b.size_force())));
                }
                RepoFilesSortField::Modified => {
                    dirs.sort_by(|a, b| a.name_lower_force().cmp(b.name_lower_force()));
                    files.sort_by(|a, b| direction.ordering(a.modified.cmp(&b.modified)));
                }
            }

            dirs.into_iter()
                .map(|file| file.id.clone())
                .chain(files.into_iter().map(|file| file.id.clone()))
                .collect()
        }
        SortGrouping::NoGrouping => {
            let mut files: Vec<_> = file_ids
                .into_iter()
                .filter_map(|id| repo_files_files.get(&id))
                .collect();

            match field {
                RepoFilesSortField::Name => {
                    files.sort_by(|a, b| {
                        direction.ordering(natord::compare(a.name_lower_force(), b.name_lower_force()))
                    });
                }
                RepoFilesSortField::Size => {
                    files.sort_by(|a, b| direction.ordering(a.size_force().cmp(&b.size_force())));
                }
                RepoFilesSortField::Modified => {
                    files.sort_by(|a, b| direction.ordering(a.modified.cmp(&b.modified)));
                }
            }

            files.into_iter().map(|file| file.id.clone()).collect()
        }
    }
}

pub fn select_used_names(
    state: &store::State,
    repo_id: &RepoId,
    parent_path: &EncryptedPath,
) -> HashSet<DecryptedNameLower> {
    let mut used_names = HashSet::new();

    for f in select_files(state, repo_id, parent_path) {
        if let Ok(name) = f.decrypted_name() {
            used_names.insert(name.to_lowercase());
        }
    }

    used_names
}

pub fn get_unused_name(
    used_names: HashSet<DecryptedNameLower>,
    name: &DecryptedName,
) -> DecryptedName {
    DecryptedName(name_utils::unused_name(&name.0, |name| {
        used_names.contains(&DecryptedNameLower(name.to_lowercase()))
    }))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        files::file_category::FileCategory,
        repo_files::state::{RepoFile, RepoFileName, RepoFilePath, RepoFileSize, RepoFileType},
        repo_files::state::{RepoFilesSort, RepoFilesSortField},
        sort::state::{SortDirection, SortGrouping},
        types::{
            DecryptedName, DecryptedPath, EncryptedPath, MountId, RemotePath, RepoFileId, RepoId,
        },
    };

    use super::*;

    #[allow(dead_code)]
    fn create_repo_file(
        typ: RepoFileType,
        name: &str,
        size: Option<i64>,
        modified: Option<i64>,
    ) -> RepoFile {
        let id = RepoFileId(format!("r1:/{}", name));
        let path = format!("/{}", name);

        RepoFile {
            id,
            mount_id: MountId("m1".into()),
            remote_path: RemotePath(path.clone()),
            repo_id: RepoId("r1".into()),
            encrypted_path: EncryptedPath(path.clone()),
            path: RepoFilePath::Decrypted {
                path: DecryptedPath(path),
            },
            name: RepoFileName::Decrypted {
                name: DecryptedName(name.into()),
                name_lower: name.to_lowercase(),
            },
            ext: None,
            content_type: None,
            typ: typ.clone(),
            size: size.map(|size| RepoFileSize::Decrypted { size }),
            modified,
            tags: None,
            unique_name: name.into(),
            remote_hash: None,
            category: match typ {
                RepoFileType::Dir => FileCategory::Folder,
                RepoFileType::File => FileCategory::Generic,
            },
        }
    }

    fn sort_names(files: Vec<RepoFile>, sort: RepoFilesSort) -> Vec<String> {
        let file_ids: Vec<RepoFileId> = files.iter().map(|f| f.id.clone()).collect();
        let files_map: HashMap<RepoFileId, RepoFile> =
            files.into_iter().map(|f| (f.id.clone(), f)).collect();

        select_sorted_files(&files_map, file_ids.iter(), &sort)
            .into_iter()
            .map(|id| {
                files_map
                    .get(&id)
                    .unwrap()
                    .decrypted_name()
                    .unwrap()
                    .0
                    .clone()
            })
            .collect()
    }

    #[test]
    fn test_select_sorted_files_dirs_first_name() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::DirsFirst,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Desc,
                grouping: SortGrouping::DirsFirst,
            },
        );

        assert_eq!(asc, vec!["a-dir", "z-dir", "a.txt", "b.txt", "c.txt"]);
        assert_eq!(desc, vec!["z-dir", "a-dir", "c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_select_sorted_files_dirs_first_size() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Size,
                direction: SortDirection::Asc,
                grouping: SortGrouping::DirsFirst,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Size,
                direction: SortDirection::Desc,
                grouping: SortGrouping::DirsFirst,
            },
        );

        assert_eq!(asc, vec!["a-dir", "z-dir", "a.txt", "b.txt", "c.txt"]);
        assert_eq!(desc, vec!["a-dir", "z-dir", "c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_select_sorted_files_dirs_first_modified() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Modified,
                direction: SortDirection::Asc,
                grouping: SortGrouping::DirsFirst,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Modified,
                direction: SortDirection::Desc,
                grouping: SortGrouping::DirsFirst,
            },
        );

        assert_eq!(asc, vec!["a-dir", "z-dir", "a.txt", "b.txt", "c.txt"]);
        assert_eq!(desc, vec!["a-dir", "z-dir", "c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_select_sorted_files_no_grouping_name() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Desc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["a-dir", "a.txt", "b.txt", "c.txt", "z-dir"]);
        assert_eq!(desc, vec!["z-dir", "c.txt", "b.txt", "a.txt", "a-dir"]);
    }

    #[test]
    fn test_select_sorted_files_no_grouping_size() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Size,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Size,
                direction: SortDirection::Desc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["a.txt", "b.txt", "c.txt", "a-dir", "z-dir"]);
        assert_eq!(desc, vec!["z-dir", "a-dir", "c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_select_sorted_files_no_grouping_modified() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(30), Some(30)),
            create_repo_file(RepoFileType::Dir, "z-dir", Some(50), Some(50)),
            create_repo_file(RepoFileType::File, "a.txt", Some(10), Some(10)),
            create_repo_file(RepoFileType::Dir, "a-dir", Some(40), Some(40)),
            create_repo_file(RepoFileType::File, "b.txt", Some(20), Some(20)),
        ];

        let asc = sort_names(
            files.clone(),
            RepoFilesSort {
                field: RepoFilesSortField::Modified,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );
        let desc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Modified,
                direction: SortDirection::Desc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["a.txt", "b.txt", "c.txt", "a-dir", "z-dir"]);
        assert_eq!(desc, vec!["z-dir", "a-dir", "c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_select_sorted_files_name_case_insensitive() {
        let files = vec![
            create_repo_file(RepoFileType::File, "c.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "B.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "a.txt", Some(1), Some(1)),
        ];

        let asc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["a.txt", "B.txt", "c.txt"]);
    }

    #[test]
    fn test_select_sorted_files_name_numeric_prefix() {
        let files = vec![
            create_repo_file(RepoFileType::File, "2foo.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "1foo.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "19foo.txt", Some(1), Some(1)),
        ];

        let asc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["1foo.txt", "2foo.txt", "19foo.txt"]);
    }

    #[test]
    fn test_select_sorted_files_name_numeric_prefix_with_space() {
        let files = vec![
            create_repo_file(RepoFileType::File, "2 foo.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "1 foo.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "19 foo.txt", Some(1), Some(1)),
        ];

        let asc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["1 foo.txt", "2 foo.txt", "19 foo.txt"]);
    }

    #[test]
    fn test_select_sorted_files_name_numeric_suffix() {
        let files = vec![
            create_repo_file(RepoFileType::File, "foo2.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "foo1.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "foo19.txt", Some(1), Some(1)),
        ];

        let asc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["foo1.txt", "foo2.txt", "foo19.txt"]);
    }

    #[test]
    fn test_select_sorted_files_name_numeric_suffix_with_space() {
        let files = vec![
            create_repo_file(RepoFileType::File, "foo 2.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "foo 1.txt", Some(1), Some(1)),
            create_repo_file(RepoFileType::File, "foo 19.txt", Some(1), Some(1)),
        ];

        let asc = sort_names(
            files,
            RepoFilesSort {
                field: RepoFilesSortField::Name,
                direction: SortDirection::Asc,
                grouping: SortGrouping::NoGrouping,
            },
        );

        assert_eq!(asc, vec!["foo 1.txt", "foo 2.txt", "foo 19.txt"]);
    }
}
