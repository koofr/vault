use std::sync::Arc;

use futures::Future;
use tokio::runtime::Runtime;
use vault_core::user_error::UserError;

use crate::mobile_errors::MobileErrors;

pub struct MobileSpawn {
    runtime: Arc<Runtime>,
    errors: Arc<MobileErrors>,
}

impl MobileSpawn {
    pub fn new(runtime: Arc<Runtime>, errors: Arc<MobileErrors>) -> Self {
        Self { runtime, errors }
    }

    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static)
    where
        T: Send + 'static,
    {
        self.runtime.spawn(future);
    }

    pub fn spawn_result(
        self: Arc<Self>,
        future: impl Future<Output = Result<(), impl UserError>> + Send + 'static,
    ) {
        self.clone()
            .spawn(async move { self.errors.handle_result(future.await) });
    }

    pub fn spawn_blocking(&self, func: impl FnOnce() + Send + 'static) {
        self.runtime.spawn_blocking(func);
    }
}
