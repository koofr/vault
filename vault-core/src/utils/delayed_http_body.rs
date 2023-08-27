use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{ready, Future};
use http_body::{combinators::UnsyncBoxBody, Body};
use pin_project_lite::pin_project;

pin_project! {
    pub struct DelayedHttpBody<B, S> {
        #[pin]
        inner: B,
        #[pin]
        sleep: Option<S>,
    }
}

impl<B: Body + Send + 'static, S: Future<Output = ()> + Send + 'static> DelayedHttpBody<B, S> {
    pub fn new(body: B, sleep: S) -> Self {
        Self {
            inner: body,
            sleep: Some(sleep),
        }
    }

    pub fn unsync_box_body(body: B, sleep: S) -> UnsyncBoxBody<B::Data, B::Error> {
        UnsyncBoxBody::new(Self::new(body, sleep))
    }
}

impl<B, S> Body for DelayedHttpBody<B, S>
where
    B: Body,
    S: Future<Output = ()>,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let mut this = self.project();

        if let Some(fut) = this.sleep.as_mut().as_pin_mut() {
            ready!(fut.poll(cx));

            this.sleep.set(None);
        }

        this.inner.poll_data(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        self.project().inner.poll_trailers(cx)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
}
