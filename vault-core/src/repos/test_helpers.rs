use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    cipher::{test_helpers as cipher_test_helpers, Cipher},
    remote::test_helpers as remote_test_helpers,
    store,
    types::RepoId,
};

use super::{
    mutations, selectors,
    state::{Repo, RepoState},
};

pub fn create_repo(
    state: &mut store::State,
    repo_id: &str,
    mount_id: &str,
    path: &str,
) -> (Repo, Arc<Cipher>, Rc<HashMap<RepoId, Arc<Cipher>>>) {
    let repo = remote_test_helpers::create_repo(repo_id, mount_id, path);

    let repo_id = RepoId(repo_id.to_owned());

    let (notify, mut mutation_state, mutation_notify) = store::test_helpers::mutation();
    mutations::repos_loaded(
        state,
        &notify,
        &mut mutation_state,
        &mutation_notify,
        Ok(vec![repo]),
        &HashMap::new(),
    );

    let repo = selectors::select_repo(state, &repo_id).unwrap().clone();

    let cipher = Arc::new(cipher_test_helpers::create_cipher());

    let mut ciphers = HashMap::new();
    ciphers.insert(repo_id.clone(), cipher.clone());

    state.repos.repos_by_id.get_mut(&repo_id).unwrap().state = RepoState::Unlocked {
        cipher: cipher.clone(),
    };

    (repo, cipher, Rc::new(ciphers))
}
