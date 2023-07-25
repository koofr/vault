use std::sync::{Arc, Mutex};

use futures::{
    channel::oneshot,
    future::{self, BoxFuture},
    FutureExt,
};
use vault_core_tests::fixtures::fake_remote_fixture::FakeRemoteFixture;
use vault_mobile::{
    memory_secure_storage::MemorySecureStorage, MobileVault, Status, SubscriptionCallback,
};

pub struct MobileVaultFixture {
    pub fake_remote_fixture: Arc<FakeRemoteFixture>,

    pub mobile_vault: Arc<MobileVault>,
}

impl MobileVaultFixture {
    pub fn create(fake_remote_fixture: Arc<FakeRemoteFixture>) -> Arc<Self> {
        let base_url = fake_remote_fixture.base_url.clone();
        let app_name = "vault-mobile-tests".into();
        let oauth2_auth_base_url = fake_remote_fixture.base_url.clone();
        let oauth2_client_id = "7ZEK2BNCEVYEJIZC5OR3TR6PQDUJ4NP3".into();
        let oauth2_client_secret =
            "VWTMENEWUYWH6G523CEV5CWOCHH7FMECW36PPQENOASYYZOQJOSGQXSR2Y62N3HB".into();
        let oauth2_redirect_uri = "http://127.0.0.1:5173/oauth2callback".into();
        let secure_storage = Box::new(MemorySecureStorage::new());
        let tokio_runtime = fake_remote_fixture.tokio_runtime.clone();

        let mobile_vault = Arc::new(MobileVault::new_with_options(
            base_url,
            app_name,
            oauth2_auth_base_url,
            oauth2_client_id,
            oauth2_client_secret,
            oauth2_redirect_uri,
            secure_storage,
            tokio_runtime,
        ));

        Arc::new(Self {
            fake_remote_fixture,

            mobile_vault,
        })
    }

    pub fn wait<T: Clone + Send + Sync + 'static>(
        &self,
        subscribe: impl FnOnce(&MobileVault, Box<dyn SubscriptionCallback>) -> u32,
        get_data: impl Fn(&MobileVault, u32) -> Option<T> + Send + Sync + 'static,
    ) -> BoxFuture<'static, T> {
        let get_data = Arc::new(get_data);

        let (sender, receiver) = oneshot::channel();
        let sender = Arc::new(Mutex::new(Some(sender)));

        let current_id = Arc::new(Mutex::new(None));
        let cb_id = current_id.clone();
        let cb_mobile_vault = self.mobile_vault.clone();
        let cb_get_data = get_data.clone();

        let id = subscribe(
            &self.mobile_vault,
            subscription_callback(move || {
                if let Some(id) = *cb_id.lock().unwrap() {
                    if let Some(data) = cb_get_data(&cb_mobile_vault, id) {
                        if let Some(sender) = sender.lock().unwrap().take() {
                            let _ = sender.send(data);
                        }
                    }
                }
            }),
        );

        *current_id.lock().unwrap() = Some(id);

        match get_data(&self.mobile_vault, id) {
            Some(data) => {
                self.mobile_vault.unsubscribe(id);

                future::ready(data).boxed()
            }
            None => {
                let mobile_vault = self.mobile_vault.clone();

                async move {
                    let res = receiver.await.unwrap();

                    mobile_vault.unsubscribe(id);

                    res
                }
                .boxed()
            }
        }
    }

    pub async fn load(&self) {
        self.mobile_vault.clone().load();

        self.wait(
            |v, cb| v.oauth2_status_subscribe(cb),
            |v, id| {
                v.oauth2_status_data(id)
                    .filter(|status| matches!(status, Status::Initial))
            },
        )
        .await;
    }
}

pub fn subscription_callback(
    f: impl Fn() + Send + Sync + 'static,
) -> Box<dyn SubscriptionCallback> {
    struct SubscriptionCallbackFn {
        f: Box<dyn Fn() + Send + Sync>,
    }

    impl SubscriptionCallback for SubscriptionCallbackFn {
        fn on_change(&self) {
            (self.f)();
        }
    }

    impl std::fmt::Debug for SubscriptionCallbackFn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SubscriptionCallbackFn").finish()
        }
    }

    Box::new(SubscriptionCallbackFn { f: Box::new(f) })
}
