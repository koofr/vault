use std::{collections::HashMap, rc::Rc, sync::Arc};

use crate::{
    cipher::{test_helpers as cipher_test_helpers, Cipher},
    remote::test_helpers as remote_test_helpers,
    store,
    types::RepoId,
};

use super::{mutations, selectors, state::Repo};

pub fn create_repo(
    state: &mut store::State,
    repo_id: &str,
    mount_id: &str,
    path: &str,
) -> (Repo, Arc<Cipher>, Rc<HashMap<RepoId, Arc<Cipher>>>) {
    let repo = remote_test_helpers::create_repo(repo_id, mount_id, path);

    let (notify, mut mutation_state, mutation_notify) = store::test_helpers::mutation();
    mutations::repos_loaded(
        state,
        &notify,
        &mut mutation_state,
        &mutation_notify,
        Ok(vec![repo]),
    );

    let repo = selectors::select_repo(state, &RepoId(repo_id.to_owned()))
        .unwrap()
        .clone();

    let cipher = Arc::new(cipher_test_helpers::create_cipher());

    let mut ciphers = HashMap::new();
    ciphers.insert(RepoId(repo_id.to_owned()), cipher.clone());

    (repo, cipher, Rc::new(ciphers))
}
