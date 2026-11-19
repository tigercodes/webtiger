//! The host bridge (000 §5.5): the core asks the shell for something as a
//! `HostRequest` and waits for the one `HostResponse` that carries its id.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

use tokio::sync::oneshot;
use wt_proto::v1::{Envelope, HostRequest, HostResponse, envelope, host_request, host_response};

use crate::dispatch::Outbox;
use crate::error::{HostError, is_code};

/// How long the shell has to answer. Creating the first web view can mean
/// starting an engine, so this is generous; a shell that is alive answers in
/// milliseconds.
pub const HOST_TIMEOUT: Duration = Duration::from_secs(30);

/// A host request's answer, when it comes.
pub type HostFuture = Pin<Box<dyn Future<Output = Result<host_response::Kind, HostError>>>>;

/// What the core asks the platform for. The request is sent before this
/// returns, so requests reach the shell in the order they were made; the
/// future is only the wait for the answer.
pub trait Host {
    fn request(&self, kind: host_request::Kind) -> HostFuture;
}

/// The [`Host`] the dispatcher gives the app: host requests leave through
/// the outbox, and [`HostBridge::answer`] completes them.
pub struct HostBridge {
    outbox: Outbox,
    pending: Rc<RefCell<BTreeMap<u64, oneshot::Sender<HostResponse>>>>,
    next_id: Cell<u64>,
    timeout: Duration,
}

impl HostBridge {
    pub(crate) fn new(outbox: Outbox, timeout: Duration) -> HostBridge {
        HostBridge {
            outbox,
            pending: Rc::new(RefCell::new(BTreeMap::new())),
            next_id: Cell::new(1),
            timeout,
        }
    }

    /// Hands a shell's `HostResponse` to the request it answers. An id
    /// nothing waits for (answered twice, or after its timeout) is dropped.
    pub(crate) fn answer(&self, id: u64, response: HostResponse) {
        let waiting = self.pending.borrow_mut().remove(&id);
        if let Some(waiting) = waiting {
            // The receiver is gone when the request's future was dropped.
            let _ = waiting.send(response);
        }
    }

    /// How many host requests wait for an answer.
    pub(crate) fn pending(&self) -> usize {
        self.pending.borrow().len()
    }
}

impl Host for HostBridge {
    fn request(&self, kind: host_request::Kind) -> HostFuture {
        let id = self.next_id.get();
        self.next_id.set(id.saturating_add(1));
        let expected = Expected::of(&kind);
        let (sender, receiver) = oneshot::channel();
        self.pending.borrow_mut().insert(id, sender);
        (self.outbox)(Envelope {
            id,
            body: Some(envelope::Body::HostRequest(HostRequest {
                kind: Some(kind),
            })),
        });
        let pending = Rc::clone(&self.pending);
        let timeout = self.timeout;
        Box::pin(async move {
            match tokio::time::timeout(timeout, receiver).await {
                Err(_) => {
                    pending.borrow_mut().remove(&id);
                    Err(HostError::new("host.timeout"))
                }
                // The bridge went away with the core.
                Ok(Err(_)) => Err(HostError::new("host.stopped")),
                Ok(Ok(response)) => expected.check(response),
            }
        })
    }
}

/// The answer a host request must get.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected {
    Pong,
    Ack,
}

impl Expected {
    fn of(kind: &host_request::Kind) -> Expected {
        match kind {
            host_request::Kind::Ping(_) => Expected::Pong,
            host_request::Kind::WebViewCreate(_)
            | host_request::Kind::WebViewLoad(_)
            | host_request::Kind::WebViewCommand(_)
            | host_request::Kind::WebViewDestroy(_)
            | host_request::Kind::OpenOutside(_)
            | host_request::Kind::WebViewFind(_) => Expected::Ack,
        }
    }

    fn check(self, response: HostResponse) -> Result<host_response::Kind, HostError> {
        match (self, response.kind) {
            (_, Some(host_response::Kind::Error(error))) => Err(if is_code(&error.code) {
                HostError::new(&error.code)
            } else {
                HostError::new("host.failed")
            }),
            (Expected::Pong, Some(kind @ host_response::Kind::Pong(_)))
            | (Expected::Ack, Some(kind @ host_response::Kind::Ack(_))) => Ok(kind),
            _ => Err(HostError::new("host.wrong_answer")),
        }
    }
}
