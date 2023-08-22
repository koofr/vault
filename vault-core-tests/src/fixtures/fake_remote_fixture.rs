use std::sync::Arc;

use crate::fake_remote::FakeRemote;

pub struct FakeRemoteFixture {
    pub tokio_runtime: Arc<tokio::runtime::Runtime>,

    pub fake_remote: Arc<FakeRemote>,
    pub base_url: String,
}

impl FakeRemoteFixture {
    pub async fn create(tokio_runtime: Arc<tokio::runtime::Runtime>) -> Arc<Self> {
        let fake_remote = Arc::new(FakeRemote::new(tokio_runtime.clone()));

        let base_url = fake_remote.start().await.unwrap();

        Arc::new(Self {
            tokio_runtime,

            fake_remote,
            base_url,
        })
    }
}
