use std::sync::{Arc, Mutex};

use futures::{channel::oneshot, future::BoxFuture};
use vault_core_tests::fake_remote::FakeRemote;
use vault_mobile::{MobileVault, OAuth2FinishFlowDone, SubscriptionCallback};

use super::mobile_vault_fixture::MobileVaultFixture;

pub struct UserFixture {
    pub mobile_vault_fixture: Arc<MobileVaultFixture>,

    pub fake_remote: Arc<FakeRemote>,
    pub mobile_vault: Arc<MobileVault>,
    pub user_id: String,
    pub mount_id: String,
    pub reqwest_client: Arc<reqwest::Client>,
}

impl UserFixture {
    pub fn create(mobile_vault_fixture: Arc<MobileVaultFixture>) -> Arc<Self> {
        let mobile_vault = mobile_vault_fixture.mobile_vault.clone();
        let fake_remote = mobile_vault_fixture.fake_remote_fixture.fake_remote.clone();

        let (user_id, mount_id) = fake_remote.app_state.users_service.create_user(None, None);

        {
            let mut state = fake_remote.app_state.state.write().unwrap();

            state.default_user_id = Some(user_id.clone());
        };

        let reqwest_client = Arc::new(
            reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        );

        Arc::new(Self {
            mobile_vault_fixture,

            fake_remote,
            mobile_vault,
            user_id,
            mount_id,
            reqwest_client,
        })
    }

    pub fn wait<T: Clone + Send + Sync + 'static>(
        &self,
        subscribe: impl FnOnce(&MobileVault, Box<dyn SubscriptionCallback>) -> u32,
        get_data: impl Fn(&MobileVault, u32) -> Option<T> + Send + Sync + 'static,
    ) -> BoxFuture<'static, T> {
        self.mobile_vault_fixture.wait(subscribe, get_data)
    }

    pub async fn login(&self) {
        let url = self
            .oauth2_request(self.mobile_vault.oauth2_start_login_flow().unwrap())
            .await;

        self.oauth2_finish_flow_url(url).await;
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

    pub async fn oauth2_finish_flow_url(&self, url: String) {
        let (sender, receiver) = oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender)));

        self.mobile_vault.clone().oauth2_finish_flow_url(
            url,
            oauth2_finish_flow_done(move || {
                if let Some(sender) = sender.lock().unwrap().take() {
                    let _ = sender.send(());
                }
            }),
        );

        receiver.await.unwrap();
    }
}

fn oauth2_finish_flow_done(f: impl Fn() + Send + Sync + 'static) -> Box<dyn OAuth2FinishFlowDone> {
    struct OAuth2FinishFlowDoneFn {
        f: Box<dyn Fn() + Send + Sync>,
    }

    impl OAuth2FinishFlowDone for OAuth2FinishFlowDoneFn {
        fn on_done(&self) {
            (self.f)();
        }
    }

    impl std::fmt::Debug for OAuth2FinishFlowDoneFn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("OAuth2FinishFlowDoneFn").finish()
        }
    }

    Box::new(OAuth2FinishFlowDoneFn { f: Box::new(f) })
}
