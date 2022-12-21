use futures::{future::BoxFuture, Future, FutureExt};
use wasm_bindgen_futures::{spawn_local, JsFuture};

use vault_core::runtime;

use crate::helpers;

pub struct BrowserRuntime {}

impl BrowserRuntime {
    pub fn new() -> Self {
        Self {}
    }
}

impl runtime::Runtime for BrowserRuntime {
    fn spawn(&self, future: BoxFuture<'static, ()>) {
        spawn_local(future)
    }

    fn sleep(&self, duration_ms: i32) -> BoxFuture<'static, ()> {
        Box::into_pin(unsafe {
            Box::from_raw(Box::into_raw(Box::new(
                JsFuture::from(helpers::sleep(duration_ms)).map(|_| ()),
            ) as Box<dyn Future<Output = ()>>)
                as *mut (dyn Future<Output = ()> + Send + Sync))
        })
    }
}
