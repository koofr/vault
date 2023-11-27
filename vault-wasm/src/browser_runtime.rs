use std::time::Duration;

use futures::{future::BoxFuture, Future};
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
            Box::from_raw(
                Box::into_raw(Box::new(sleep(duration)) as Box<dyn Future<Output = ()>>)
                    as *mut (dyn Future<Output = ()> + Send + Sync),
            )
        })
    }

    fn now(&self) -> TimeMillis {
        now()
    }
}

pub fn now() -> TimeMillis {
    TimeMillis(js_sys::Date::now() as i64)
}
