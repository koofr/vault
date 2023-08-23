use std::collections::HashSet;

use crate::{
    repo_files::{
        selectors as repo_files_selectors,
        state::{RepoFile, RepoFilePath},
    },
    repos::selectors as repos_selectors,
    store,
    utils::path_utils,
};

pub fn select_files_zip_name(state: &store::State, files: &[RepoFile]) -> String {
    let files_len = files.len();

    let file_ids_set = files
        .iter()
        .map(|file| file.id.clone())
        .collect::<HashSet<String>>();

    let mut parent_ids_set = files
        .iter()
        .filter_map(|file| {
            file.decrypted_path()
                .ok()
                .and_then(|path| path_utils::parent_path(path))
                .map(|parent_path| repo_files_selectors::get_file_id(&file.repo_id, parent_path))
        })
        .collect::<HashSet<String>>();

    if parent_ids_set.len() == 1 {
        let parent_id = parent_ids_set.drain().next().unwrap();

        let parent_name = repo_files_selectors::select_file(state, &parent_id).and_then(|parent| {
            match &parent.path {
                RepoFilePath::Decrypted { path } if path == "/" => {
                    repos_selectors::select_repo(state, &parent.repo_id)
                        .ok()
                        .map(|repo| repo.name.as_str())
                }
                _ => parent.decrypted_name().ok(),
            }
        });

        let is_all_children = repo_files_selectors::select_children(state, &parent_id)
            .map(|children_ids| {
                children_ids.iter().cloned().collect::<HashSet<String>>() == file_ids_set
            })
            .unwrap_or(false);

        match (parent_name, is_all_children) {
            (Some(parent_name), true) => format!("{}.zip", parent_name),
            (Some(parent_name), false) => {
                format!("{}-{}-selected-items.zip", parent_name, files_len)
            }
            (None, _) => format!("{}-selected-items.zip", files_len),
        }
    } else {
        format!("{}-selected-items.zip", files_len)
    }
}

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::{
        repo_files::{selectors as repo_files_selectors, test_helpers as repo_files_test_helpers},
        repos::test_helpers as repos_test_helpers,
        store,
    };

    use super::select_files_zip_name;

    #[test]
    fn test_select_files_zip_name() {
        let mut state = store::State::default();
        let (repo, cipher, ciphers) =
            repos_test_helpers::create_repo(&mut state, "r1", "m1", "/Vault");
        repo_files_test_helpers::files_loaded(
            &mut state,
            &repo.id,
            "/",
            ciphers.clone(),
            vec![
                repo_files_test_helpers::create_dir("D1", &cipher),
                repo_files_test_helpers::create_file("F1", &cipher),
                repo_files_test_helpers::create_file("F2", &cipher),
            ],
        );
        repo_files_test_helpers::files_loaded(
            &mut state,
            &repo.id,
            "/D1",
            ciphers.clone(),
            vec![repo_files_test_helpers::create_file("F3", &cipher)],
        );
        let d1 = repo_files_selectors::select_file(&state, "r1:/D1").unwrap();
        let f1 = repo_files_selectors::select_file(&state, "r1:/F1").unwrap();
        let f2 = repo_files_selectors::select_file(&state, "r1:/F2").unwrap();
        let f3 = repo_files_selectors::select_file(&state, "r1:/D1/F3").unwrap();

        assert_eq!(
            select_files_zip_name(&state, &[d1.clone(), f1.clone(), f2.clone()]),
            "Vault.zip"
        );
        assert_eq!(
            select_files_zip_name(&state, &[d1.clone(), f1.clone()]),
            "Vault-2-selected-items.zip"
        );
        assert_eq!(
            select_files_zip_name(&state, &[d1.clone(), f1.clone()]),
            "Vault-2-selected-items.zip"
        );
        assert_eq!(
            select_files_zip_name(&state, &[d1.clone(), f1.clone(), f2.clone(), f3.clone()]),
            "4-selected-items.zip"
        );
    }
}
