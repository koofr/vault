use std::time::Duration;

use futures::future::BoxFuture;

use crate::types::TimeMillis;

pub trait Runtime {
    fn spawn(&self, future: BoxFuture<'static, ()>);
    fn sleep(&self, duration: Duration) -> BoxFuture<'static, ()>;
    fn now(&self) -> TimeMillis;
}

pub type BoxRuntime = Box<dyn Runtime + Send + Sync>;
