use futures::future::BoxFuture;

pub trait Runtime {
    fn spawn(&self, future: BoxFuture<'static, ()>);
    fn sleep(&self, duration_ms: i32) -> BoxFuture<'static, ()>;
}
