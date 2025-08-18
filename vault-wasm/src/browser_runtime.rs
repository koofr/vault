use std::time::Duration;

use futures::{Future, future::BoxFuture};
use gloo_timers::future::sleep;
use wasm_bindgen_futures::spawn_local;

use vault_core::{runtime, types::TimeMillis};

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

    fn sleep(&self, duration: Duration) -> BoxFuture<'static, ()> {
        Box::into_pin(unsafe {
            std::mem::transmute::<
                Box<dyn Future<Output = ()>>,
                Box<dyn Future<Output = ()> + Send + Sync>,
            >(Box::new(sleep(duration)) as Box<dyn Future<Output = ()>>)
        })
    }

    fn now(&self) -> TimeMillis {
        now()
    }
}

pub fn now() -> TimeMillis {
    TimeMillis(js_sys::Date::now() as i64)
}
