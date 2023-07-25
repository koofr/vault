use futures::FutureExt;
use similar_asserts::assert_eq;
use vault_mobile::{Repo, RepoState, Status};

use crate::{
    fixtures::repo_fixture::RepoFixture,
    helpers::{with_repo, with_user},
};

#[test]
fn test_repos() {
    with_repo(|fixture| {
        async move {
            let repos = fixture
                .wait(|v, cb| v.repos_subscribe(cb), |v, id| v.repos_data(id))
                .await;

            assert!(matches!(repos.status, Status::Loaded));

            assert_eq!(repos.repos.len(), 1);
        }
        .boxed()
    });
}

#[test]
fn test_repo() {
    with_user(|fixture| {
        async move {
            let repo_fixture = RepoFixture::create(fixture.clone()).await;

            let repo = repo_fixture.get_repo().await;

            assert_eq!(
                repo,
                Repo {
                    id: repo_fixture.repo_id.clone(),
                    name: "My safe box".into(),
                    mount_id: repo.mount_id.clone(),
                    path: repo.path.clone(),
                    state: RepoState::Locked,
                    added: repo.added,
                    web_url: repo.web_url.clone(),
                }
            );

            repo_fixture.unlock().await;

            let repo = repo_fixture.get_repo().await;

            assert_eq!(
                repo,
                Repo {
                    id: repo_fixture.repo_id.clone(),
                    name: "My safe box".into(),
                    mount_id: repo.mount_id.clone(),
                    path: repo.path.clone(),
                    state: RepoState::Unlocked,
                    added: repo.added,
                    web_url: repo.web_url.clone(),
                }
            );
        }
        .boxed()
    });
}
