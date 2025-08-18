// Copied from
// https://github.com/ranile/gloo/blob/d600d3ac934b1c62b7009c22eabe2f91be4bd974/crates/timers/src/callback.rs and
// https://github.com/ranile/gloo/blob/d600d3ac934b1c62b7009c22eabe2f91be4bd974/crates/timers/src/future.rs

use std::{
    convert::TryFrom,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::channel::oneshot;
use js_sys::Function;
use wasm_bindgen::{JsCast, JsValue, prelude::*};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout", catch)]
    pub fn set_timeout(handler: &Function, timeout: i32) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "clearTimeout")]
    pub fn clear_timeout(handle: JsValue) -> JsValue;
}

/// A scheduled timeout.
///
/// See `Timeout::new` for scheduling new timeouts.
///
/// Once scheduled, you can [`drop`] the [`Timeout`] to clear it or [`forget`](Timeout::forget) to leak it.
/// This pattern is known as Resource Acquisition Is Initialization (RAII).
#[derive(Debug)]
#[must_use = "timeouts cancel on drop; either call `forget` or `drop` explicitly"]
pub struct Timeout {
    id: Option<JsValue>,
    closure: Option<Closure<dyn FnMut()>>,
}

impl Drop for Timeout {
    /// Disposes of the timeout, dually cancelling this timeout by calling
    /// `clearTimeout` directly.
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            clear_timeout(id);
        }
    }
}

impl Timeout {
    /// Schedule a timeout to invoke `callback` in `millis` milliseconds from
    /// now.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::callback::Timeout;
    ///
    /// let timeout = Timeout::new(1_000, move || {
    ///     // Do something...
    /// });
    /// ```
    pub fn new<F>(millis: u32, callback: F) -> Timeout
    where
        F: 'static + FnOnce(),
    {
        let closure = Closure::once(callback);

        let id = set_timeout(
            closure.as_ref().unchecked_ref::<js_sys::Function>(),
            millis as i32,
        )
        .unwrap_throw();

        Timeout {
            id: Some(id),
            closure: Some(closure),
        }
    }

    /// Forgets this resource without clearing the timeout.
    ///
    /// Returns the identifier returned by the original `setTimeout` call, and
    /// therefore you can still cancel the timeout by calling `clearTimeout`
    /// directly (perhaps via `web_sys::clear_timeout_with_handle`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::callback::Timeout;
    ///
    /// // We definitely want to do stuff, and aren't going to ever cancel this
    /// // timeout.
    /// Timeout::new(1_000, || {
    ///     // Do stuff...
    /// }).forget();
    /// ```
    pub fn forget(mut self) -> JsValue {
        let id = self.id.take().unwrap_throw();
        self.closure.take().unwrap_throw().forget();
        id
    }

    /// Cancel this timeout so that the callback is not invoked after the time
    /// is up.
    ///
    /// The scheduled callback is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::callback::Timeout;
    ///
    /// let timeout = Timeout::new(1_000, || {
    ///     // Do stuff...
    /// });
    ///
    /// // If actually we didn't want to set a timer, then cancel it.
    /// if nevermind() {
    ///     timeout.cancel();
    /// }
    /// # fn nevermind() -> bool { true }
    /// ```
    pub fn cancel(mut self) -> Closure<dyn FnMut()> {
        self.closure.take().unwrap_throw()
    }
}

/// A scheduled timeout as a `Future`.
///
/// See `TimeoutFuture::new` for scheduling new timeouts.
///
/// Once scheduled, if you change your mind and don't want the timeout to fire,
/// you can `drop` the future.
///
/// A timeout future will never resolve to `Err`. Its only failure mode is when
/// the timeout is so long that it is effectively infinite and never fires.
///
/// # Example
///
/// ```no_run
/// use gloo_timers::future::TimeoutFuture;
/// use futures_util::future::{select, Either};
/// use wasm_bindgen_futures::spawn_local;
///
/// spawn_local(async {
///     match select(TimeoutFuture::new(1_000), TimeoutFuture::new(2_000)).await {
///         Either::Left((val, b)) => {
///             // Drop the `2_000` ms timeout to cancel its timeout.
///             drop(b);
///         }
///         Either::Right((a, val)) => {
///             panic!("the `1_000` ms timeout should have won this race");
///         }
///     }
/// });
/// ```
#[derive(Debug)]
#[must_use = "futures do nothing unless polled or spawned"]
pub struct TimeoutFuture {
    _inner: Timeout,
    rx: oneshot::Receiver<()>,
}

impl TimeoutFuture {
    /// Create a new timeout future.
    ///
    /// Remember that futures do nothing unless polled or spawned, so either
    /// pass this future to `wasm_bindgen_futures::spawn_local` or use it inside
    /// another future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::future::TimeoutFuture;
    /// use wasm_bindgen_futures::spawn_local;
    ///
    /// spawn_local(async {
    ///     TimeoutFuture::new(1_000).await;
    ///     // Do stuff after one second...
    /// });
    /// ```
    pub fn new(millis: u32) -> TimeoutFuture {
        let (tx, rx) = oneshot::channel();
        let inner = Timeout::new(millis, move || {
            // if the receiver was dropped we do nothing.
            tx.send(()).unwrap_throw();
        });
        TimeoutFuture { _inner: inner, rx }
    }
}

/// Waits until the specified duration has elapsed.
///
/// # Panics
///
/// This function will panic if the specified [`Duration`] cannot be casted into a u32 in
/// milliseconds.
///
/// # Example
///
/// ```compile_fail
/// use std::time::Duration;
/// use gloo_timers::future::sleep;
///
/// sleep(Duration::from_secs(1)).await;
/// ```
pub fn sleep(dur: Duration) -> TimeoutFuture {
    let millis = u32::try_from(dur.as_millis())
        .expect_throw("failed to cast the duration into a u32 with Duration::as_millis.");

    TimeoutFuture::new(millis)
}

impl Future for TimeoutFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        Future::poll(Pin::new(&mut self.rx), cx).map(|t| t.unwrap_throw())
    }
}
