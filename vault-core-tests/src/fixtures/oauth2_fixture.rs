use std::sync::Arc;

use vault_core::{
    common::state::Status,
    oauth2::{self, errors::OAuth2Error},
    Vault,
};

use crate::fake_remote::FakeRemote;

use super::user_fixture::UserFixture;

pub struct OAuth2Fixture {
    pub user_fixture: Arc<UserFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub vault: Arc<Vault>,
    pub user_id: String,
    pub reqwest_client: Arc<reqwest::Client>,
}

impl OAuth2Fixture {
    pub fn create(user_fixture: Arc<UserFixture>) -> Arc<Self> {
        let vault = user_fixture.vault.clone();
        let fake_remote = user_fixture.fake_remote.clone();
        let user_id = user_fixture.user_id.clone();

        let reqwest_client = Arc::new(
            reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        );

        Arc::new(Self {
            user_fixture,

            fake_remote,
            vault,
            user_id,
            reqwest_client,
        })
    }

    pub async fn login(&self) {
        let url = self
            .oauth2_request(format!(
                "{}&user_id={}",
                self.vault.oauth2_start_login_flow().unwrap(),
                self.user_id
            ))
            .await;

        self.vault.oauth2_finish_flow_url(&url).await.unwrap();
    }

    pub async fn logout(&self) {
        let url = self
            .oauth2_request(self.vault.oauth2_start_logout_flow().unwrap())
            .await;

        self.vault.oauth2_finish_flow_url(&url).await.unwrap();
    }

    pub async fn oauth2_request(&self, url: String) -> String {
        let res = self
            .reqwest_client
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .unwrap();

        assert_eq!(res.status(), reqwest::StatusCode::SEE_OTHER);

        res.headers()
            .get(reqwest::header::LOCATION)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn get_status(&self) -> Status<OAuth2Error> {
        self.vault
            .store
            .with_state(|state| oauth2::selectors::select_status(state).clone())
    }
}
