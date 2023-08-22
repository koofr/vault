use std::time::Duration;

use futures::future::BoxFuture;

pub trait Runtime {
    fn spawn(&self, future: BoxFuture<'static, ()>);
    fn sleep(&self, duration: Duration) -> BoxFuture<'static, ()>;
    fn now_ms(&self) -> i64;
}

pub type BoxRuntime = Box<dyn Runtime + Send + Sync>;
