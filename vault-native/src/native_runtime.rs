use std::{sync::Arc, time::Duration};

use futures::FutureExt;

pub struct NativeRuntime {
    runtime: Arc<tokio::runtime::Runtime>,
}

impl NativeRuntime {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>) -> Self {
        Self { runtime }
    }

    pub fn spawn(&self, future: futures::future::BoxFuture<'static, ()>) {
        self.runtime.spawn(future);
    }
}

impl vault_core::runtime::Runtime for NativeRuntime {
    fn spawn(&self, future: futures::future::BoxFuture<'static, ()>) {
        self.spawn(future);
    }

    fn sleep(&self, duration: Duration) -> futures::future::BoxFuture<'static, ()> {
        tokio::time::sleep(duration).boxed()
    }

    fn now_ms(&self) -> i64 {
        now_ms()
    }
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("System clock was before 1970.")
        .as_millis() as i64
}
