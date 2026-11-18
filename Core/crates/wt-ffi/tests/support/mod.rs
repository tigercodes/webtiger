//! A shell over the C ABI, for the tests: it starts a core with a sink that
//! decodes what arrives, and answers host requests the way a shell with a
//! working engine would.
#![allow(
    unsafe_code,
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test support: it calls the C ABI, and a failure here is a failing test"
)]

use std::ffi::c_void;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use prost::Message as _;
use webtiger_core::{wt_core_send, wt_core_start, wt_core_stop};
use wt_proto::CONTRACT_MAJOR;
use wt_proto::v1::{
    Ack, CoreConfig, EngineEvent, EngineInfo, EngineKind, Envelope, FakeOptions, HostPong,
    HostResponse, Platform, Request, Response, TestMode, envelope, event, host_request,
    host_response, request, response,
};

/// How long a test waits for the core to say something.
pub(crate) const PATIENCE: Duration = Duration::from_secs(10);

/// What the sink is given as its context.
struct Collector {
    sender: Sender<Envelope>,
}

extern "C" fn sink(context: *mut c_void, envelope: *const u8, len: usize) {
    // SAFETY: the context is the `Collector` `TestShell` boxed, alive until
    // after `wt_core_stop`; the bytes are valid for this call.
    let (collector, bytes) = unsafe {
        (
            &*(context.cast::<Collector>()),
            std::slice::from_raw_parts(envelope, len),
        )
    };
    let envelope = Envelope::decode(bytes).expect("the core sends well-formed envelopes");
    let _ = collector.sender.send(envelope);
}

pub(crate) fn fake_config() -> CoreConfig {
    CoreConfig {
        contract_major: CONTRACT_MAJOR,
        contract_minor: 0,
        platform: Platform::Cli as i32,
        locale: "en-GB".to_owned(),
        data_dir: String::new(),
        test_mode: TestMode::Fake as i32,
        fake: Some(FakeOptions::default()),
        app_version: "0.0.0-test".to_owned(),
        engines: vec![EngineInfo {
            kind: EngineKind::Webkit as i32,
            version: "test".to_owned(),
        }],
    }
}

pub(crate) struct TestShell {
    pub(crate) handle: u64,
    received: Receiver<Envelope>,
    collector: *mut Collector,
    next_id: u64,
    /// Events seen while waiting for something else.
    pub(crate) events: Vec<event::Kind>,
    /// Host requests seen, in order.
    pub(crate) host_requests: Vec<host_request::Kind>,
    /// Answer host requests as they arrive (the default), or leave them.
    pub(crate) answer_host: bool,
    stopped: bool,
}

impl TestShell {
    /// Starts a core from encoded configuration bytes. The handle is 0 when
    /// the core refused; what it sent is still there to read.
    pub(crate) fn start_bytes(config: &[u8]) -> TestShell {
        let (sender, received) = channel();
        let collector = Box::into_raw(Box::new(Collector { sender }));
        // SAFETY: `config` is a live slice; the collector outlives the core
        // (it is freed in `Drop`, after `wt_core_stop`).
        let handle =
            unsafe { wt_core_start(config.as_ptr(), config.len(), Some(sink), collector.cast()) };
        TestShell {
            handle,
            received,
            collector,
            next_id: 1,
            events: Vec::new(),
            host_requests: Vec::new(),
            answer_host: true,
            stopped: handle == 0,
        }
    }

    pub(crate) fn start(config: &CoreConfig) -> TestShell {
        TestShell::start_bytes(&config.encode_to_vec())
    }

    /// A core in fake mode, past its READY event.
    pub(crate) fn fake() -> TestShell {
        let mut shell = TestShell::start(&fake_config());
        assert_ne!(shell.handle, 0, "fake mode starts");
        shell.next_event();
        shell
    }

    pub(crate) fn send_bytes(&self, bytes: &[u8]) -> i32 {
        // SAFETY: `bytes` is a live slice.
        unsafe { wt_core_send(self.handle, bytes.as_ptr(), bytes.len()) }
    }

    pub(crate) fn send_envelope(&self, envelope: &Envelope) -> i32 {
        self.send_bytes(&envelope.encode_to_vec())
    }

    /// Sends a request. Its id.
    pub(crate) fn send(&mut self, kind: request::Kind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let status = self.send_envelope(&Envelope {
            id,
            body: Some(envelope::Body::Request(Request { kind: Some(kind) })),
        });
        assert_eq!(status, 0, "wt_core_send");
        id
    }

    /// Reports what a web view did: a notification.
    pub(crate) fn report(&self, event: EngineEvent) {
        let status = self.send_envelope(&Envelope {
            id: 0,
            body: Some(envelope::Body::Request(Request {
                kind: Some(request::Kind::EngineEvent(event)),
            })),
        });
        assert_eq!(status, 0, "wt_core_send");
    }

    /// The next envelope, or `None` when the core stays silent.
    pub(crate) fn try_next(&self, patience: Duration) -> Option<Envelope> {
        self.received.recv_timeout(patience).ok()
    }

    /// Sends a request and waits for its answer, answering host requests
    /// and keeping events on the way.
    pub(crate) fn request(&mut self, kind: request::Kind) -> response::Kind {
        let id = self.send(kind);
        self.response(id)
    }

    pub(crate) fn response(&mut self, id: u64) -> response::Kind {
        loop {
            let envelope = self
                .try_next(PATIENCE)
                .unwrap_or_else(|| panic!("request {id} was not answered"));
            match envelope.body {
                Some(envelope::Body::Response(Response { kind: Some(kind) }))
                    if envelope.id == id =>
                {
                    return kind;
                }
                other => self.keep(envelope.id, other),
            }
        }
    }

    /// The next event, answering host requests on the way.
    pub(crate) fn next_event(&mut self) -> event::Kind {
        loop {
            if !self.events.is_empty() {
                return self.events.remove(0);
            }
            let envelope = self.try_next(PATIENCE).expect("an event");
            self.keep(envelope.id, envelope.body);
        }
    }

    fn keep(&mut self, id: u64, body: Option<envelope::Body>) {
        match body {
            Some(envelope::Body::Event(event)) => self.events.extend(event.kind),
            Some(envelope::Body::HostRequest(request)) => {
                let Some(kind) = request.kind else { return };
                let answer = match &kind {
                    host_request::Kind::Ping(ping) => host_response::Kind::Pong(HostPong {
                        payload: ping.payload.clone(),
                    }),
                    _ => host_response::Kind::Ack(Ack {}),
                };
                self.host_requests.push(kind);
                if self.answer_host {
                    self.send_envelope(&Envelope {
                        id,
                        body: Some(envelope::Body::HostResponse(HostResponse {
                            kind: Some(answer),
                        })),
                    });
                }
            }
            _ => {}
        }
    }

    pub(crate) fn stop(&mut self) -> i32 {
        self.stopped = true;
        wt_core_stop(self.handle)
    }
}

impl Drop for TestShell {
    fn drop(&mut self) {
        if !self.stopped {
            wt_core_stop(self.handle);
        }
        // SAFETY: the core has stopped (or never started), so the sink is
        // not called again and nothing else holds the collector.
        drop(unsafe { Box::from_raw(self.collector) });
    }
}
