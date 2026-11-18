//! A shell without a screen: it sends requests, answers the core's host
//! requests from its scripted engine, and keeps everything the core says.

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::time::Duration;

use wt_app::{Dispatcher, HOST_TIMEOUT, Outbox, StartError};
use wt_proto::CONTRACT_MAJOR;
use wt_proto::v1::{
    CoreConfig, EngineEvent, EngineInfo, EngineKind, Envelope, FakeOptions, HostResponse,
    OpenWindow, Platform, Request, SubmitAddress, Subscribe, TestMode, WindowSnapshot, WindowTopic,
    envelope, event, host_request, host_response, request, response, snapshot, subscribe,
};

use crate::engine::FakeEngine;

/// A shell driven by a test.
pub struct FakeShell {
    dispatcher: Rc<Dispatcher>,
    /// What the core has sent and the shell has not yet looked at.
    inbox: Rc<RefCell<VecDeque<Envelope>>>,
    /// The scripted web engine that answers host requests.
    pub engine: FakeEngine,
    /// Leave host requests unanswered, as a shell that is busy (or hangs)
    /// would, until [`FakeShell::release_host`].
    pub hold_host: bool,
    held: VecDeque<(u64, host_request::Kind)>,
    /// Answer the next host request with this instead of the engine's answer.
    pub answer_next_with: Option<host_response::Kind>,
    /// Every host request the core made, in order.
    pub host_requests: Vec<host_request::Kind>,
    /// Every event the core sent, in order.
    pub events: Vec<event::Kind>,
    /// Every envelope the core sent, in order, as a short label: "response",
    /// "event" or "host_request". For tests of ordering.
    pub order: Vec<&'static str>,
    responses: BTreeMap<u64, response::Kind>,
    next_request: u64,
}

impl FakeShell {
    /// A configuration for fake mode with one engine.
    #[must_use]
    pub fn config(engine: EngineKind) -> CoreConfig {
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
                kind: engine as i32,
                version: "fake".to_owned(),
            }],
        }
    }

    /// A shell in fake mode with a WebKit engine.
    #[must_use]
    pub fn fake() -> FakeShell {
        FakeShell::start(&FakeShell::config(EngineKind::Webkit)).expect("fake mode starts")
    }

    /// Starts the core with a configuration.
    ///
    /// # Errors
    /// When the core refuses to start.
    pub fn start(config: &CoreConfig) -> Result<FakeShell, StartError> {
        FakeShell::start_with(config, HOST_TIMEOUT)
    }

    /// [`FakeShell::start`] with the host bridge's patience chosen.
    ///
    /// # Errors
    /// When the core refuses to start.
    pub fn start_with(
        config: &CoreConfig,
        host_timeout: Duration,
    ) -> Result<FakeShell, StartError> {
        let inbox: Rc<RefCell<VecDeque<Envelope>>> = Rc::new(RefCell::new(VecDeque::new()));
        let sink = Rc::clone(&inbox);
        let outbox: Outbox = Rc::new(move |envelope| sink.borrow_mut().push_back(envelope));
        let dispatcher = Dispatcher::start_with(config, outbox, host_timeout)?;
        let engines = config.engines.iter().map(EngineInfo::kind).collect();
        Ok(FakeShell {
            dispatcher,
            inbox,
            engine: FakeEngine::new(engines),
            hold_host: false,
            held: VecDeque::new(),
            answer_next_with: None,
            host_requests: Vec::new(),
            events: Vec::new(),
            order: Vec::new(),
            responses: BTreeMap::new(),
            next_request: 1,
        })
    }

    /// Sends a request without waiting for its answer. Its id.
    pub fn send(&mut self, kind: request::Kind) -> u64 {
        let id = self.next_request;
        self.next_request += 1;
        self.dispatcher.accept(Envelope {
            id,
            body: Some(envelope::Body::Request(Request { kind: Some(kind) })),
        });
        id
    }

    /// Hands the core any envelope, as the wire would.
    pub fn send_raw(&mut self, envelope: Envelope) {
        self.dispatcher.accept(envelope);
    }

    /// Reports what a web view did: a notification, which gets no answer.
    pub fn report(&mut self, event: EngineEvent) {
        self.dispatcher.accept(Envelope {
            id: 0,
            body: Some(envelope::Body::Request(Request {
                kind: Some(request::Kind::EngineEvent(event)),
            })),
        });
    }

    /// Sends a request and waits for its answer.
    pub async fn request(&mut self, kind: request::Kind) -> response::Kind {
        let id = self.send(kind);
        self.settle().await;
        self.take_response(id)
            .unwrap_or_else(|| panic!("request {id} was not answered"))
    }

    /// The answer to a request, once: `None` while there is none.
    pub fn take_response(&mut self, id: u64) -> Option<response::Kind> {
        self.responses.remove(&id)
    }

    /// Lets everything that can happen, happen: host requests are answered,
    /// the engine's reports are delivered, snapshots are published, until
    /// the core has nothing more to say.
    pub async fn settle(&mut self) {
        let mut idle_rounds = 0;
        while idle_rounds < 2 {
            let worked = self.drain();
            self.dispatcher.flush();
            tokio::task::yield_now().await;
            if worked || !self.inbox.borrow().is_empty() {
                idle_rounds = 0;
            } else {
                idle_rounds += 1;
            }
        }
    }

    /// Looks at everything the core has sent. Whether there was anything.
    fn drain(&mut self) -> bool {
        let mut worked = false;
        loop {
            let Some(envelope) = self.inbox.borrow_mut().pop_front() else {
                return worked;
            };
            worked = true;
            match envelope.body {
                Some(envelope::Body::Response(response)) => {
                    self.order.push("response");
                    if let Some(kind) = response.kind {
                        self.responses.insert(envelope.id, kind);
                    }
                }
                Some(envelope::Body::Event(event)) => {
                    self.order.push("event");
                    if let Some(kind) = event.kind {
                        self.events.push(kind);
                    }
                }
                Some(envelope::Body::HostRequest(request)) => {
                    self.order.push("host_request");
                    let Some(kind) = request.kind else { continue };
                    self.host_requests.push(kind.clone());
                    if self.hold_host {
                        self.held.push_back((envelope.id, kind));
                    } else {
                        self.answer_host(envelope.id, kind);
                    }
                }
                Some(envelope::Body::Request(_) | envelope::Body::HostResponse(_)) | None => {
                    panic!("the core sent an envelope only a shell sends")
                }
            }
        }
    }

    fn answer_host(&mut self, id: u64, kind: host_request::Kind) {
        let (answer, reports) = match self.answer_next_with.take() {
            Some(answer) => (answer, Vec::new()),
            None => self.engine.answer(kind),
        };
        self.dispatcher.accept(Envelope {
            id,
            body: Some(envelope::Body::HostResponse(HostResponse {
                kind: Some(answer),
            })),
        });
        for report in reports {
            self.report(report);
        }
    }

    /// Answers the host requests held back while `hold_host` was set, in the
    /// order they were made, and stops holding.
    pub fn release_host(&mut self) {
        self.hold_host = false;
        while let Some((id, kind)) = self.held.pop_front() {
            self.answer_host(id, kind);
        }
    }

    /// Opens a window and subscribes to it. Its window, tab and
    /// subscription ids.
    pub async fn open_window(&mut self) -> (u64, u64, u64) {
        let response::Kind::WindowOpened(opened) =
            self.request(request::Kind::OpenWindow(OpenWindow {})).await
        else {
            panic!("OpenWindow answers WindowOpened");
        };
        let response::Kind::Subscribed(subscribed) = self
            .request(request::Kind::Subscribe(Subscribe {
                topic: Some(subscribe::Topic::Window(WindowTopic {
                    window_id: opened.window_id,
                })),
            }))
            .await
        else {
            panic!("Subscribe answers Subscribed");
        };
        (opened.window_id, opened.tab_id, subscribed.subscription_id)
    }

    /// Types an address into a tab's address field and waits.
    pub async fn submit(&mut self, tab: u64, text: &str) -> response::Kind {
        self.request(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: tab,
            text: text.to_owned(),
        }))
        .await
    }

    /// Every snapshot a subscription was sent, in order.
    #[must_use]
    pub fn snapshots(&self, subscription: u64) -> Vec<WindowSnapshot> {
        self.events
            .iter()
            .filter_map(|event| match event {
                event::Kind::Snapshot(snapshot) if snapshot.subscription_id == subscription => {
                    snapshot
                        .model
                        .as_ref()
                        .map(|snapshot::Model::Window(window)| window.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// The window as a subscription last saw it.
    #[must_use]
    pub fn window(&self, subscription: u64) -> WindowSnapshot {
        self.snapshots(subscription)
            .pop()
            .expect("the subscription has been sent a snapshot")
    }

    /// How many host requests the core still waits on.
    #[must_use]
    pub fn pending_host_requests(&self) -> usize {
        self.dispatcher.pending_host_requests()
    }
}
