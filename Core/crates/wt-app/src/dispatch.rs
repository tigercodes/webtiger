//! The contract around the app (000 §5.5): envelopes in, envelopes out.
//!
//! The dispatcher has no thread of its own. Whoever owns it calls
//! [`Dispatcher::accept`] for each envelope a shell sends and
//! [`Dispatcher::flush`] when it has no more to hand over, from inside a
//! tokio `LocalSet` (a request that waits on the shell runs as a local
//! task). `wt-ffi` does that on the core's thread; the tests do it in line.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

use wt_core::{Clock, ManualClock, SystemClock, Timestamp, Uuid, WindowId};
use wt_proto::v1::{
    Ack, CoreConfig, CoreStatus, EngineKind, Envelope, Event, HostPing, MemoryLevel, Ping, Pong,
    Request, Response, Snapshot, Subscribe, Subscribed, TestMode, Unsubscribe, WindowSnapshot,
    core_status, envelope, event, host_request, host_response, request, response, snapshot,
    subscribe,
};
use wt_proto::{CONTRACT_MAJOR, CONTRACT_MINOR};
use wt_store::Store;

use crate::app::{App, AppFuture, AppOptions};
use crate::error::AppError;
use crate::fence::CatchUnwind;
use crate::host::{HOST_TIMEOUT, Host as _, HostBridge};
use crate::tidying::TIDY_LOOK_EVERY;

/// Where the dispatcher puts every envelope it sends, in order.
pub type Outbox = Rc<dyn Fn(Envelope)>;

/// The core's own version, as `Pong` reports it.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The largest payload a `Ping` may carry.
pub const MAX_PING_PAYLOAD: usize = 64 * 1024;

/// The store's file in the data folder.
const STORE_FILE: &str = "webtiger.sqlite";

/// Fake mode's login set: the same on every run, so a UI suite's snapshots
/// do not change.
const FAKE_LOGIN_SET: Uuid = Uuid::from_random_bytes([0x57; 16]);
/// Fake mode's "now": 2026-01-01T12:00:00Z.
const FAKE_NOW: Timestamp = Timestamp(1_767_268_800_000);

/// Fake mode's clock: it starts at [`FAKE_NOW`] and moves on a second each
/// time it is read, so what a run writes down has the same times on every
/// run, and in the order it was written. A suite that must see a day pass
/// says how far it moves instead (`FakeOptions.seconds_a_read`, 049).
struct FakeClock {
    clock: ManualClock,
    step_ms: i64,
}

impl FakeClock {
    fn new(seconds_a_read: u32) -> FakeClock {
        FakeClock {
            clock: ManualClock::new(FAKE_NOW),
            step_ms: i64::from(seconds_a_read.max(1)).saturating_mul(1_000),
        }
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Timestamp {
        let now = self.clock.now();
        self.clock.advance_millis(self.step_ms);
        now
    }
}

/// Why the core refused to start, as the code `CoreStatus` carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StartError {
    code: String,
}

impl StartError {
    #[must_use]
    pub fn new(code: &str) -> StartError {
        StartError {
            code: code.to_owned(),
        }
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }
}

struct Subscription {
    window: WindowId,
    /// What its shell was last sent, so an unchanged model is not sent again.
    last: Option<WindowSnapshot>,
}

/// The contract's dispatcher: one per running core.
pub struct Dispatcher {
    app: App,
    bridge: Rc<HostBridge>,
    outbox: Outbox,
    subscriptions: RefCell<BTreeMap<u64, Subscription>>,
    next_subscription: std::cell::Cell<u64>,
    fake: bool,
    /// The look for tabs to tidy (049) is going round: once a minute while
    /// some Space is tidied.
    tidying: std::cell::Cell<bool>,
    /// The look for tabs to put to sleep (053) is going.
    sleeping: std::cell::Cell<bool>,
    /// Fake mode, for a suite (055): how often the core acts as though it
    /// had been told the system is short of memory. `None` when it does
    /// not, and once that has been set going.
    short_every: std::cell::Cell<Option<Duration>>,
    /// How many times one of the core's own timers has woken it (060): the
    /// look for tabs to put to sleep, the look for tabs to tidy, and fake
    /// mode's word that memory is short. A core with nothing to look for
    /// has none of them going, and this stands still.
    wakeups: std::cell::Cell<u64>,
}

impl Dispatcher {
    /// Starts the app from a shell's configuration and announces
    /// `STATE_READY` through the outbox.
    ///
    /// # Errors
    /// With the code a refused start reports: `start.contract_major`,
    /// `start.no_engine`, `start.data_dir`, `start.random`, or the store's.
    pub fn start(config: &CoreConfig, outbox: Outbox) -> Result<Rc<Dispatcher>, StartError> {
        Dispatcher::start_with(config, outbox, HOST_TIMEOUT)
    }

    /// [`Dispatcher::start`] with the host bridge's patience chosen, for a
    /// test that waits for it to run out.
    ///
    /// # Errors
    /// As [`Dispatcher::start`].
    pub fn start_with(
        config: &CoreConfig,
        outbox: Outbox,
        host_timeout: Duration,
    ) -> Result<Rc<Dispatcher>, StartError> {
        Dispatcher::start_at(config, outbox, host_timeout, None)
    }

    /// [`Dispatcher::start_with`] with the clock chosen, for a test that
    /// moves it: `None` is the clock the configuration has.
    ///
    /// # Errors
    /// As [`Dispatcher::start`].
    pub fn start_at(
        config: &CoreConfig,
        outbox: Outbox,
        host_timeout: Duration,
        clock: Option<Rc<dyn Clock>>,
    ) -> Result<Rc<Dispatcher>, StartError> {
        if config.contract_major != CONTRACT_MAJOR {
            return Err(StartError::new("start.contract_major"));
        }
        let engine = config
            .engines
            .first()
            .map(wt_proto::v1::EngineInfo::kind)
            .filter(|kind| *kind != EngineKind::Unspecified)
            .ok_or_else(|| StartError::new("start.no_engine"))?;
        let fake = config.test_mode() == TestMode::Fake;
        let (mut store, new_login_set, now) = if fake {
            let store = Store::open_in_memory().map_err(|e| StartError::new(e.code()))?;
            (store, FAKE_LOGIN_SET, FAKE_NOW)
        } else {
            if config.data_dir.is_empty() {
                return Err(StartError::new("start.data_dir"));
            }
            let folder = Path::new(&config.data_dir);
            std::fs::create_dir_all(folder).map_err(|_| StartError::new("start.data_dir"))?;
            let store =
                Store::open(&folder.join(STORE_FILE)).map_err(|e| StartError::new(e.code()))?;
            let login_set = Uuid::random().map_err(|_| StartError::new("start.random"))?;
            (store, login_set, SystemClock.now())
        };
        let space = store
            .default_space(new_login_set, now)
            .map_err(|e| StartError::new(e.code()))?;
        let clock: Rc<dyn Clock> = match clock {
            Some(chosen) => chosen,
            None if fake => Rc::new(FakeClock::new(
                config
                    .fake
                    .as_ref()
                    .map_or(0, |options| options.seconds_a_read),
            )),
            None => Rc::new(SystemClock),
        };

        let bridge = Rc::new(HostBridge::new(Rc::clone(&outbox), host_timeout));
        let options = AppOptions {
            engine,
            // In fake mode nobody is there to be asked where searches go,
            // unless the run is to be a first run (020).
            search_settled: fake
                && !config
                    .fake
                    .as_ref()
                    .is_some_and(|options| options.first_run),
            fake_start: config
                .fake
                .as_ref()
                .map(|options| options.start_address.clone())
                .filter(|start| fake && !start.is_empty()),
            clock,
            session: !fake,
            sleep_after_seconds: config
                .fake
                .as_ref()
                .filter(|_| fake)
                .map_or(0, |options| options.sleep_after_seconds),
            live_tabs: crate::states::live_tabs(config, fake),
            new_login_set: if fake {
                // The same ones in the same order every run: each a step
                // on from the app's own.
                let made = std::cell::Cell::new(0u8);
                Rc::new(move || {
                    let next = made.get().checked_add(1)?;
                    made.set(next);
                    Some(Uuid::from_random_bytes([0x57u8.wrapping_add(next); 16]))
                })
            } else {
                Rc::new(|| Uuid::random().ok())
            },
        };
        // The app holds the store: the database stays open, and locked, for
        // as long as the core runs.
        let app = App::new(
            options,
            space,
            Rc::new(RefCell::new(store)),
            Rc::clone(&bridge) as Rc<dyn crate::Host>,
        );
        let dispatcher = Rc::new(Dispatcher {
            app,
            bridge,
            outbox,
            subscriptions: RefCell::new(BTreeMap::new()),
            next_subscription: std::cell::Cell::new(1),
            fake,
            tidying: std::cell::Cell::new(false),
            sleeping: std::cell::Cell::new(false),
            wakeups: std::cell::Cell::new(0),
            short_every: std::cell::Cell::new(
                config
                    .fake
                    .as_ref()
                    .filter(|_| fake)
                    .map(|options| u64::from(options.memory_short_every_seconds))
                    .filter(|seconds| *seconds > 0)
                    .map(Duration::from_secs),
            ),
        });
        dispatcher.send_event(event::Kind::Status(CoreStatus {
            state: core_status::State::Ready as i32,
            code: String::new(),
        }));
        Ok(dispatcher)
    }

    /// Takes one envelope from the shell: a request, or the answer to a host
    /// request. Anything else a shell has no business sending, and it is
    /// dropped. Must be called from inside a `LocalSet`.
    pub fn accept(self: &Rc<Self>, envelope: Envelope) {
        let id = envelope.id;
        match envelope.body {
            Some(envelope::Body::Request(Request { kind })) => self.request(id, kind),
            Some(envelope::Body::HostResponse(response)) => self.bridge.answer(id, response),
            Some(
                envelope::Body::Response(_)
                | envelope::Body::Event(_)
                | envelope::Body::HostRequest(_),
            )
            | None => {}
        }
        // Whatever it was may have put a tab out of sight (053): the look
        // for tabs to put to sleep is set going when there is one.
        self.watch_for_sleepers();
        self.say_memory_is_short();
    }

    /// How many times one of the core's own timers has woken it since it
    /// started (060): what "idle, with no timers" is counted by.
    #[must_use]
    pub fn wakeups(&self) -> u64 {
        self.wakeups.get()
    }

    /// One of the core's own timers has woken it.
    fn woken(&self) {
        self.wakeups.set(self.wakeups.get().saturating_add(1));
    }

    /// Fake mode, for a suite that must see what follows (055): from the
    /// first thing a shell sends, the core acts every so often as though it
    /// had been told the system is short of memory, by the runtime's own
    /// time, until the core stops.
    fn say_memory_is_short(self: &Rc<Self>) {
        let Some(every) = self.short_every.take() else {
            return;
        };
        let core = Rc::downgrade(self);
        tokio::task::spawn_local(async move {
            loop {
                tokio::time::sleep(every).await;
                let Some(core) = core.upgrade() else {
                    break;
                };
                core.woken();
                core.app.memory_short(MemoryLevel::Short as i32);
                core.flush();
            }
        });
    }

    /// Sets the look for tabs to put to sleep going (053), when some tab is
    /// out of sight and it is not going already: as often as the app looks,
    /// by the runtime's own time and no clock. It stops when no tab is out
    /// of sight any longer, and with the core.
    fn watch_for_sleepers(self: &Rc<Self>) {
        if self.sleeping.get() || !self.app.could_sleep() {
            return;
        }
        self.sleeping.set(true);
        let core = Rc::downgrade(self);
        let every = self.app.sleep_look_every();
        tokio::task::spawn_local(async move {
            loop {
                tokio::time::sleep(every).await;
                let Some(core) = core.upgrade() else {
                    break;
                };
                core.woken();
                if !core.app.could_sleep() {
                    core.sleeping.set(false);
                    break;
                }
                core.app.sleep_idle_tabs();
                core.flush();
            }
        });
    }

    /// Publishes a snapshot of every subscribed model that changed since the
    /// last call. Changes that arrived together leave as one snapshot.
    pub fn flush(&self) {
        // A tab that has left the front is asked for its state (051), and a
        // tab that is gone has its state dropped.
        self.app.keep_states();
        // No more tabs are alive than the device can hold (054): one that
        // has just come alive may have put another to sleep.
        self.app.sleep_over_the_limit();
        // What is published is what is kept: the session is written here,
        // when it has changed (009).
        self.app.persist();
        let closed = self.app.take_closed();
        let dirty = self.app.take_dirty();
        let mut publishing = Vec::new();
        {
            let mut subscriptions = self.subscriptions.borrow_mut();
            subscriptions.retain(|_, subscription| !closed.contains(&subscription.window));
            for (id, subscription) in subscriptions.iter_mut() {
                if !dirty.contains(&subscription.window) {
                    continue;
                }
                let now = self.app.window_snapshot(subscription.window);
                if now.is_some() && now != subscription.last {
                    subscription.last.clone_from(&now);
                    publishing.push((*id, now));
                }
            }
        }
        // The borrow is released first: an outbox is free to do anything.
        for (id, window) in publishing {
            self.send_event(event::Kind::Snapshot(Snapshot {
                subscription_id: id,
                model: window.map(snapshot::Model::Window),
            }));
        }
    }

    /// How many host requests wait for the shell's answer.
    #[must_use]
    pub fn pending_host_requests(&self) -> usize {
        self.bridge.pending()
    }

    fn request(self: &Rc<Self>, id: u64, kind: Option<request::Kind>) {
        let Some(kind) = kind else {
            return self.respond(id, Err(AppError::new("request.empty")));
        };
        let begun = catch_unwind(AssertUnwindSafe(|| self.begin(id, kind)));
        let answer = match begun {
            // Answered in line.
            Ok(None) => return,
            Ok(Some(answer)) => answer,
            Err(_) => return self.respond(id, Err(AppError::new("core.panic"))),
        };
        let this = Rc::clone(self);
        tokio::task::spawn_local(async move {
            let result = CatchUnwind::new(answer)
                .await
                .unwrap_or_else(|_| Err(AppError::new("core.panic")));
            this.respond(id, result);
            this.flush();
            // A Space may have been given a time, or the store have had one
            // from the start (049): a shell's first request sets the look
            // going.
            this.watch_for_tidying();
        });
    }

    /// Sets the look for tabs to tidy going (049), when some Space is tidied
    /// and it is not going already: once every [`TIDY_LOOK_EVERY`], by the
    /// core's own clock, and what it tidies is published as any change is.
    /// It stops when no Space is tidied any longer, and with the core.
    fn watch_for_tidying(self: &Rc<Self>) {
        if self.tidying.get() || !self.app.tidies() {
            return;
        }
        self.tidying.set(true);
        let core = Rc::downgrade(self);
        tokio::task::spawn_local(async move {
            loop {
                tokio::time::sleep(TIDY_LOOK_EVERY).await;
                let Some(core) = core.upgrade() else {
                    break;
                };
                core.woken();
                if !core.app.tidies() {
                    core.tidying.set(false);
                    break;
                }
                core.app.tidy();
                core.flush();
            }
        });
    }

    /// Starts a request. What it changes in the models has changed when this
    /// returns. `None` when it was answered in line; otherwise the wait for
    /// its answer.
    fn begin(&self, id: u64, kind: request::Kind) -> Option<AppFuture> {
        match kind {
            request::Kind::EngineEvent(event) => {
                self.app.notify(event);
                self.respond(id, Ok(response::Kind::Ack(Ack {})));
                None
            }
            request::Kind::Subscribe(subscribe) => {
                self.subscribe(id, &subscribe);
                None
            }
            request::Kind::Unsubscribe(Unsubscribe { subscription_id }) => {
                self.subscriptions.borrow_mut().remove(&subscription_id);
                self.respond(id, Ok(response::Kind::Ack(Ack {})));
                None
            }
            request::Kind::Ping(ping) => Some(self.ping(ping)),
            other => Some(self.app.request(other)),
        }
    }

    /// Answers `Subscribed`, then sends the model's first snapshot: in that
    /// order, so a shell knows the subscription before it hears from it.
    fn subscribe(&self, id: u64, subscribe: &Subscribe) {
        let Some(subscribe::Topic::Window(topic)) = &subscribe.topic else {
            return self.respond(id, Err(AppError::new("subscribe.no_topic").field("topic")));
        };
        let window = WindowId(topic.window_id);
        let Some(snapshot) = self.app.window_snapshot(window) else {
            return self.respond(id, Err(AppError::new("window.unknown").field("window_id")));
        };
        let subscription_id = self.next_subscription.get();
        self.next_subscription
            .set(subscription_id.saturating_add(1));
        self.subscriptions.borrow_mut().insert(
            subscription_id,
            Subscription {
                window,
                last: Some(snapshot.clone()),
            },
        );
        self.respond(
            id,
            Ok(response::Kind::Subscribed(Subscribed { subscription_id })),
        );
        self.send_event(event::Kind::Snapshot(Snapshot {
            subscription_id,
            model: Some(snapshot::Model::Window(snapshot)),
        }));
    }

    fn ping(&self, ping: Ping) -> AppFuture {
        if ping.panic && self.fake {
            panic_for_test();
        }
        if ping.payload.len() > MAX_PING_PAYLOAD {
            return Box::pin(std::future::ready(Err(AppError::new(
                "ping.payload_too_large",
            )
            .field("payload"))));
        }
        if !ping.via_host {
            return Box::pin(std::future::ready(Ok(pong(ping.payload, false))));
        }
        let asked = self.bridge.request(host_request::Kind::Ping(HostPing {
            payload: ping.payload,
        }));
        Box::pin(async move {
            match asked.await? {
                host_response::Kind::Pong(answer) => Ok(pong(answer.payload, true)),
                // The bridge passes on only a HostPong for a HostPing.
                _ => Err(AppError::new("host.wrong_answer")),
            }
        })
    }

    /// Sends a request's answer. A notification (id 0) gets none.
    fn respond(&self, id: u64, result: Result<response::Kind, AppError>) {
        if id == 0 {
            return;
        }
        let kind = result.unwrap_or_else(|error| response::Kind::Error(error.into_proto()));
        (self.outbox)(Envelope {
            id,
            body: Some(envelope::Body::Response(Response { kind: Some(kind) })),
        });
    }

    fn send_event(&self, kind: event::Kind) {
        (self.outbox)(Envelope {
            id: 0,
            body: Some(envelope::Body::Event(Event { kind: Some(kind) })),
        });
    }
}

fn pong(payload: Vec<u8>, via_host: bool) -> response::Kind {
    response::Kind::Pong(Pong {
        payload,
        via_host,
        core_version: CORE_VERSION.to_owned(),
        contract_major: CONTRACT_MAJOR,
        contract_minor: CONTRACT_MINOR,
    })
}

/// `Ping.panic`, the panic fence's test hook.
#[allow(clippy::panic, reason = "the contract's test hook for the panic fence")]
fn panic_for_test() {
    panic!("Ping.panic: the panic fence's test hook");
}
