use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    cipher::Cipher,
    remote::{models, test_helpers as remote_test_helpers},
    remote_files::mutations as remote_files_mutations,
    repos::selectors as repos_selectors,
    store::{self, test_helpers as store_test_helpers},
    utils::path_utils,
};

use super::mutations as repo_files_mutations;

pub fn create_file(name: &str, cipher: &Cipher) -> models::FilesFile {
    remote_test_helpers::create_file(&cipher.encrypt_filename(name))
}

pub fn create_dir(name: &str, cipher: &Cipher) -> models::FilesFile {
    remote_test_helpers::create_dir(&cipher.encrypt_filename(name))
}

pub fn files_loaded(
    state: &mut store::State,
    repo_id: &str,
    path: &str,
    ciphers: Rc<HashMap<String, Arc<Cipher>>>,
    files: Vec<models::FilesFile>,
) {
    let repo = repos_selectors::select_repo(state, repo_id)
        .unwrap()
        .clone();

    let (notify, mut mutation_state, _) = store_test_helpers::mutation();

    let cipher = ciphers.get(repo_id).unwrap().clone();

    let ciphers1 = ciphers.clone();

    let mutation_notify: store::MutationNotify = Box::new(move |_, state, mutation_state| {
        let (_, _, mutation_notify) = store_test_helpers::mutation();

        repo_files_mutations::handle_remote_files_mutation(
            state,
            notify.clone().as_ref(),
            mutation_state,
            &mutation_notify,
            ciphers1.as_ref(),
        );
    });

    let remote_path = &path_utils::join_paths(&repo.path, &cipher.encrypt_path(path));

    remote_files_mutations::bundle_loaded(
        state,
        &mut mutation_state,
        &mutation_notify,
        &repo.mount_id,
        remote_path,
        remote_test_helpers::create_bundle(
            path_utils::path_to_name(&remote_path).unwrap(),
            Some(files),
        ),
    );
}
