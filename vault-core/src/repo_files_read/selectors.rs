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
    use std::{collections::HashMap, rc::Rc, sync::Arc};

    use similar_asserts::assert_eq;

    use crate::{
        cipher::test_helpers as cipher_test_helpers,
        remote::test_helpers as remote_test_helpers,
        remote_files::mutations as remote_files_mutations,
        repo_files::{mutations as repo_files_mutations, selectors as repo_files_selectors},
        repos::mutations as repos_mutations,
        store,
    };

    use super::select_files_zip_name;

    #[test]
    fn test_select_files_zip_name() {
        let mut state = store::State::default();
        let repo_1 = remote_test_helpers::create_repo("r1", "m1", "/Vault");
        let cipher = Arc::new(cipher_test_helpers::create_cipher());
        let mut ciphers = HashMap::new();
        ciphers.insert(String::from("r1"), cipher.clone());
        let ciphers = Rc::new(ciphers);
        repos_mutations::repos_loaded(&mut state, vec![repo_1]);
        let notify: Rc<store::Notify> = Rc::new(Box::new(|_| {}));
        let mut mutation_state = store::MutationState::default();
        let ciphers1 = ciphers.clone();
        let mutation_notify: store::MutationNotify = Box::new(move |_, state, mutation_state| {
            let repo_files_mutation_notify: store::MutationNotify = Box::new(move |_, _, _| {});

            repo_files_mutations::handle_remote_files_mutation(
                state,
                notify.clone().as_ref(),
                mutation_state,
                &repo_files_mutation_notify,
                ciphers1.as_ref(),
            );
        });
        remote_files_mutations::bundle_loaded(
            &mut state,
            &mut mutation_state,
            &mutation_notify,
            "m1",
            "/Vault",
            remote_test_helpers::create_bundle(
                "Vault",
                Some(vec![
                    remote_test_helpers::create_dir(&cipher.encrypt_filename("D1")),
                    remote_test_helpers::create_file(&cipher.encrypt_filename("F1")),
                    remote_test_helpers::create_file(&cipher.encrypt_filename("F2")),
                ]),
            ),
        );
        let notify: Rc<store::Notify> = Rc::new(Box::new(|_| {}));
        let mut mutation_state = store::MutationState::default();
        let ciphers2 = ciphers.clone();
        let mutation_notify: store::MutationNotify = Box::new(move |_, state, mutation_state| {
            let repo_files_mutation_notify: store::MutationNotify = Box::new(move |_, _, _| {});

            repo_files_mutations::handle_remote_files_mutation(
                state,
                notify.clone().as_ref(),
                mutation_state,
                &repo_files_mutation_notify,
                ciphers2.as_ref(),
            );
        });
        remote_files_mutations::bundle_loaded(
            &mut state,
            &mut mutation_state,
            &mutation_notify,
            "m1",
            &format!("/Vault/{}", &cipher.encrypt_filename("D1")),
            remote_test_helpers::create_bundle(
                &cipher.encrypt_filename("D1"),
                Some(vec![remote_test_helpers::create_file(
                    &cipher.encrypt_filename("F3"),
                )]),
            ),
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
