//! The running cores, by handle. A fixed table of atomically swapped slots,
//! not a map behind a lock: the fast path looks a core up on an engine's
//! network thread for every request a page makes, and must never wait for
//! another thread to let go of anything.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use arc_swap::{ArcSwap, ArcSwapOption};
use wt_core::{GuardSwitches, WebViewId};
use wt_guard::{Guard, Kind, Verdict};
use wt_proto::v1::CoreConfig;

use crate::core::Core;
use crate::{SinkTarget, Status};

/// How many cores may run at once in one process (`WT_MAX_CORES`). A shell
/// runs one; the tests run a few side by side.
pub const MAX_CORES: usize = 64;

struct Entry {
    handle: u64,
    core: Core,
    /// The core's blocker (064): what the fast path asks, on whatever thread
    /// an engine calls. The core's own thread puts a new one in its place,
    /// at a stroke, when the rules change (066): a decision that is being
    /// made is made on the one it began with.
    guard: Arc<ArcSwap<Guard>>,
    /// Guard's switches, and which web view is whose (067): put here by the
    /// core's own thread, at a stroke, each time either changes.
    switches: Arc<ArcSwap<GuardSwitches>>,
}

static SLOTS: [ArcSwapOption<Entry>; MAX_CORES] =
    [const { ArcSwapOption::const_empty() }; MAX_CORES];

/// Handles start at 1 and are never reused, so a stale one names nothing.
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

/// Starts a core and gives it a handle.
pub(crate) fn start(config: CoreConfig, target: SinkTarget) -> Result<u64, &'static str> {
    // A full table refuses before a thread has run. (Two starts racing for
    // the last slot are settled below: the loser's core is stopped again.)
    if SLOTS.iter().all(|slot| slot.load().is_some()) {
        return Err("start.too_many_cores");
    }
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    // With no rules until the core's thread has said what its rules are,
    // which it has by the time `Core::start` returns.
    let guard = Arc::new(ArcSwap::from_pointee(Guard::default()));
    let switches = Arc::new(ArcSwap::from_pointee(GuardSwitches::default()));
    let core = Core::start(config, target, Arc::clone(&guard), Arc::clone(&switches))?;
    let entry = Arc::new(Entry {
        handle,
        core,
        guard,
        switches,
    });
    for slot in &SLOTS {
        let empty: Option<Arc<Entry>> = None;
        let previous = slot.compare_and_swap(&empty, Some(Arc::clone(&entry)));
        if previous.is_none() {
            return Ok(handle);
        }
    }
    entry.core.stop();
    Err("start.too_many_cores")
}

/// The core with a handle, to send to or stop.
pub(crate) fn find(handle: u64) -> Option<CoreRef> {
    SLOTS.iter().find_map(|slot| {
        slot.load_full()
            .filter(|entry| entry.handle == handle)
            .map(CoreRef)
    })
}

/// Whether a core with this handle runs. Lock-free and allocation-free: the
/// fast path's lookup.
pub(crate) fn is_running(handle: u64) -> bool {
    handle != 0
        && SLOTS.iter().any(|slot| {
            slot.load()
                .as_ref()
                .is_some_and(|entry| entry.handle == handle)
        })
}

/// Guard's decision for a request a page makes in a web view, by the
/// switches and the blocker of the core with this handle (064, 067): the
/// fast path's lookup. Where Guard is off for the page, by the nearest
/// switch that says anything, the request is allowed and the blocker is not
/// asked. `None` when no such core runs.
pub(crate) fn decide(
    handle: u64,
    web_view: u64,
    url: &str,
    source_url: &str,
    kind: Kind,
) -> Option<Verdict> {
    if handle == 0 {
        return None;
    }
    SLOTS.iter().find_map(|slot| {
        let entry = slot.load();
        let entry = entry.as_ref().filter(|entry| entry.handle == handle)?;
        if !entry
            .switches
            .load()
            .on_for(WebViewId(web_view), source_url)
        {
            return Some(Verdict::Allow);
        }
        Some(entry.guard.load().decide(url, source_url, kind))
    })
}

/// Whether a frame is to be kept from its site's own storage (071), by the
/// switches of the core with this handle: the fast path's lookup. `None`
/// when no such core runs.
pub(crate) fn keeps_apart(
    handle: u64,
    web_view: u64,
    frame_origin: &str,
    top_origin: &str,
    page_url: &str,
) -> Option<bool> {
    if handle == 0 {
        return None;
    }
    SLOTS.iter().find_map(|slot| {
        let entry = slot.load();
        let entry = entry.as_ref().filter(|entry| entry.handle == handle)?;
        Some(entry.switches.load().keeps_apart(
            WebViewId(web_view),
            frame_origin,
            top_origin,
            page_url,
        ))
    })
}

/// Stops the core with a handle and frees its slot.
pub(crate) fn stop(handle: u64) -> Status {
    for slot in &SLOTS {
        let current = slot.load_full();
        let Some(entry) = current.as_ref().filter(|entry| entry.handle == handle) else {
            continue;
        };
        if entry.core.is_core_thread() {
            return Status::WrongThread;
        }
        // Whoever swaps the entry out stops the core; a second caller finds
        // the slot already changed and reports an unknown handle.
        let previous = slot.compare_and_swap(&current, None);
        let won = match (&*previous, &current) {
            (Some(previous), Some(current)) => Arc::ptr_eq(previous, current),
            _ => false,
        };
        if !won {
            return Status::UnknownCore;
        }
        entry.core.stop();
        return Status::Ok;
    }
    Status::UnknownCore
}

/// A running core, kept alive while a call uses it.
pub(crate) struct CoreRef(Arc<Entry>);

impl std::ops::Deref for CoreRef {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.0.core
    }
}
