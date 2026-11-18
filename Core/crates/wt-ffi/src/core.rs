//! A running core: one thread, a current-thread tokio runtime on it, and
//! `wt-app`'s dispatcher inside. Envelopes arrive through a channel; when the
//! channel is empty the thread sleeps. An idle core has no timer and wakes
//! for nothing (000 §7).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::mpsc::sync_channel;
use std::sync::{Mutex, PoisonError};
use std::thread::{JoinHandle, ThreadId};

use prost::Message as _;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use wt_app::{Dispatcher, Outbox};
use wt_proto::v1::{CoreConfig, Envelope};

use crate::{SinkTarget, Status};

enum Message {
    Envelope(Box<Envelope>),
    Stop,
}

pub(crate) struct Core {
    sender: UnboundedSender<Message>,
    thread: Mutex<Option<JoinHandle<()>>>,
    thread_id: ThreadId,
}

impl Core {
    /// Starts the core's thread and waits until the dispatcher is up or has
    /// refused. A refusal's code is the caller's to deliver.
    pub(crate) fn start(config: CoreConfig, target: SinkTarget) -> Result<Core, &'static str> {
        let (sender, receiver) = unbounded_channel();
        let (started, starting) = sync_channel::<Result<(), &'static str>>(1);
        let thread = std::thread::Builder::new()
            .name("wt-core".to_owned())
            .spawn(move || {
                let ran = catch_unwind(AssertUnwindSafe(|| {
                    run(&config, target, receiver, &started);
                }));
                if ran.is_err() {
                    // Before the dispatcher was up: the starter hears of it.
                    // After: the shell does.
                    if started.try_send(Err("core.panic")).is_err() {
                        target.refuse("core.panic");
                    }
                }
            })
            .map_err(|_| "start.thread_failed")?;
        match starting.recv() {
            Ok(Ok(())) => Ok(Core {
                sender,
                thread_id: thread.thread().id(),
                thread: Mutex::new(Some(thread)),
            }),
            Ok(Err(code)) => {
                let _ = thread.join();
                Err(code)
            }
            Err(_) => {
                let _ = thread.join();
                Err("core.panic")
            }
        }
    }

    pub(crate) fn send(&self, envelope: Envelope) -> Status {
        match self.sender.send(Message::Envelope(Box::new(envelope))) {
            Ok(()) => Status::Ok,
            // The thread has ended: the core is as good as stopped.
            Err(_) => Status::UnknownCore,
        }
    }

    /// Whether the caller is the core's own thread: inside the sink.
    pub(crate) fn is_core_thread(&self) -> bool {
        std::thread::current().id() == self.thread_id
    }

    /// Asks the thread to end and waits for it. The sink is not called once
    /// this returns.
    pub(crate) fn stop(&self) {
        let _ = self.sender.send(Message::Stop);
        let thread = self
            .thread
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}

/// The core's thread.
fn run(
    config: &CoreConfig,
    target: SinkTarget,
    mut receiver: UnboundedReceiver<Message>,
    started: &std::sync::mpsc::SyncSender<Result<(), &'static str>>,
) {
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
    else {
        let _ = started.send(Err("start.runtime"));
        return;
    };
    let outbox: Outbox = Rc::new(move |envelope: Envelope| {
        target.deliver(&envelope.encode_to_vec());
    });
    let dispatcher = match Dispatcher::start(config, outbox) {
        Ok(dispatcher) => dispatcher,
        Err(error) => {
            let _ = started.send(Err(start_code(error.code())));
            return;
        }
    };
    let _ = started.send(Ok(()));

    let local = tokio::task::LocalSet::new();
    local.block_on(&runtime, async move {
        while let Some(first) = receiver.recv().await {
            // Everything that has already arrived is taken before anything is
            // published, so reports that came together leave as one snapshot.
            let mut next = Some(first);
            let mut stopping = false;
            while let Some(message) = next {
                match message {
                    Message::Envelope(envelope) => dispatcher.accept(*envelope),
                    Message::Stop => {
                        stopping = true;
                        break;
                    }
                }
                next = receiver.try_recv().ok();
            }
            if stopping {
                break;
            }
            dispatcher.flush();
        }
        // Leaving drops the dispatcher and, with the LocalSet, every request
        // still waiting on the shell.
    });
}

/// A start refusal's code as a `'static` string: the codes `wt-app` can
/// refuse with are known, and anything else is reported as a kind.
fn start_code(code: &str) -> &'static str {
    const KNOWN: [&str; 9] = [
        "start.contract_major",
        "start.no_engine",
        "start.data_dir",
        "start.random",
        "store.open",
        "store.migrate",
        "store.newer",
        "store.query",
        "store.corrupt",
    ];
    KNOWN
        .into_iter()
        .find(|known| *known == code)
        .unwrap_or("start.failed")
}
