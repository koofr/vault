use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http_body::{combinators::UnsyncBoxBody, Body};

pub struct AbortHttpBody<F, E> {
    build_error: F,
    error_type: PhantomData<E>,
}

impl<F, E> AbortHttpBody<F, E> {
    pub fn new(build_error: F) -> Self
    where
        F: Fn() -> E + Send + 'static,
        E: Send + 'static,
    {
        Self {
            build_error,
            error_type: PhantomData,
        }
    }

    pub fn unsync_box_body(build_error: F) -> UnsyncBoxBody<Bytes, E>
    where
        F: Fn() -> E + Send + 'static,
        E: Send + 'static,
    {
        UnsyncBoxBody::new(Self {
            build_error,
            error_type: PhantomData,
        })
    }
}

impl<F, E> Body for AbortHttpBody<F, E>
where
    F: Fn() -> E + Send + 'static,
    E: Send,
{
    type Data = Bytes;
    type Error = E;

    fn poll_data(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Poll::Ready(Some(Err((self.build_error)())))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Err((self.build_error)()))
    }
}
