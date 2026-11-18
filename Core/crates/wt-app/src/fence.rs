//! The panic fence for asynchronous work: a future whose poll panics
//! completes with the panic instead of unwinding into the runtime, so the
//! core can answer the request and carry on.

use std::any::Any;
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::task::{Context, Poll};

/// What a panic carries. Never printed: it could hold an address.
pub(crate) type Panic = Box<dyn Any + Send + 'static>;

/// Completes with `Err` when polling `F` panics; `F` isn't polled again.
pub(crate) struct CatchUnwind<F> {
    future: Option<F>,
}

impl<F: Future + Unpin> CatchUnwind<F> {
    pub(crate) fn new(future: F) -> CatchUnwind<F> {
        CatchUnwind {
            future: Some(future),
        }
    }
}

impl<F: Future + Unpin> Future for CatchUnwind<F> {
    type Output = Result<F::Output, Panic>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Some(future) = self.future.as_mut() else {
            return Poll::Pending;
        };
        match catch_unwind(AssertUnwindSafe(|| Pin::new(future).poll(cx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(output)) => {
                self.future = None;
                Poll::Ready(Ok(output))
            }
            Err(panic) => {
                // A future that panicked may panic again as it is dropped.
                let broken = self.future.take();
                let _ = catch_unwind(AssertUnwindSafe(move || drop(broken)));
                Poll::Ready(Err(panic))
            }
        }
    }
}
