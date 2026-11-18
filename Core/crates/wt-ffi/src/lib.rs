//! The C ABI every WebTiger shell links (000 §5.5). The declarations are in
//! `include/webtiger_core.h`; `tests/abi.rs` keeps the two in step.
//!
//! Four functions carry protobuf bytes of the contract: start, send, stop
//! and version. Two more are the fast path: synchronous answers an engine's
//! own threads can wait for. Every entry point runs under `catch_unwind`, so
//! a panic never crosses into Swift, C++ or the JVM. Cores are named by `u64`
//! handles rather than pointers, so a stale handle is an error code, not
//! undefined behaviour.
//!
//! Behind the ABI: one thread per core ([`core`]), running `wt-app`'s
//! dispatcher. Everything a shell can ask is answered there.
//!
//! This is the one crate allowed to contain unsafe code (Core/Cargo.toml).
//! Each unsafe block says why it is sound.
#![allow(unsafe_code)]

#[cfg(feature = "jni")]
pub mod android;
mod core;
mod fastpath;
mod registry;

use std::ffi::c_void;
use std::io::Write as _;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Once;

use prost::Message as _;
use wt_proto::v1::{CoreConfig, CoreStatus, Envelope, Event, core_status, envelope, event};
use wt_proto::{CONTRACT_MAJOR, CONTRACT_MINOR};

pub use fastpath::{
    WT_NAV_ALLOW, WT_NAV_CANCEL, WT_NAV_EXTERNAL, WT_NAV_MAIN_FRAME, WT_NAV_USER_GESTURE,
    WT_VERDICT_ALLOW, WT_VERDICT_BLOCK, WtNavVerdict, WtNavigation, WtRequest, WtVerdict,
    wt_guard_check, wt_nav_check,
};
pub use registry::MAX_CORES;

/// The longest message either way (`WT_MAX_MESSAGE_LEN`).
pub const MAX_MESSAGE_LEN: usize = 4 * 1024 * 1024;

/// What the ABI's functions return (`WT_OK` and the rest in the header).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    UnknownCore = 1,
    NullArgument = 2,
    TooLarge = 3,
    Malformed = 4,
    WrongThread = 5,
    Panic = 6,
    BadArgument = 7,
}

/// The shell's sink (`wt_sink`): receives every envelope the core sends.
pub type RawSink = extern "C" fn(context: *mut c_void, envelope: *const u8, len: usize);

/// The contract the library implements: `(major << 16) | minor`.
#[unsafe(no_mangle)]
pub extern "C" fn wt_core_version() -> u32 {
    (CONTRACT_MAJOR << 16) | CONTRACT_MINOR
}

/// Starts a core from encoded `CoreConfig` bytes; see webtiger_core.h.
///
/// # Safety
/// `config` must point to `len` readable bytes for the duration of the call,
/// or be NULL with `len` 0. `sink` and `context` must stay valid until
/// `wt_core_stop` returns for the handle this returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wt_core_start(
    config: *const u8,
    len: usize,
    sink: Option<RawSink>,
    context: *mut c_void,
) -> u64 {
    let Some(sink) = sink else {
        return 0;
    };
    install_quiet_panic_hook();
    let target = SinkTarget {
        sink,
        context: context as usize,
    };
    let started = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller guarantees `config` points to `len` readable
        // bytes for this call (or is NULL with 0).
        let bytes = unsafe { borrow_bytes(config, len) }.map_err(|_| "start.bad_config")?;
        let config = CoreConfig::decode(bytes).map_err(|_| "start.bad_config")?;
        registry::start(config, target)
    }));
    match started {
        Ok(Ok(handle)) => handle,
        Ok(Err(code)) => {
            target.refuse(code);
            0
        }
        Err(_) => {
            target.refuse("core.panic");
            0
        }
    }
}

/// Queues an encoded `Envelope`; see webtiger_core.h.
///
/// # Safety
/// `envelope` must point to `len` readable bytes for the duration of the
/// call, or be NULL with `len` 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wt_core_send(core: u64, envelope: *const u8, len: usize) -> i32 {
    let status = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: the caller guarantees `envelope` points to `len` readable
        // bytes for this call (or is NULL with 0).
        let bytes = match unsafe { borrow_bytes(envelope, len) } {
            Ok(bytes) => bytes,
            Err(status) => return status,
        };
        let Some(core) = registry::find(core) else {
            return Status::UnknownCore;
        };
        // Decoded here, on the caller's thread, so a shell hears at once that
        // what it sent was not an envelope.
        match Envelope::decode(bytes) {
            Ok(envelope) => core.send(envelope),
            Err(_) => Status::Malformed,
        }
    }));
    status.unwrap_or(Status::Panic) as i32
}

/// Stops a core; see webtiger_core.h.
#[unsafe(no_mangle)]
pub extern "C" fn wt_core_stop(core: u64) -> i32 {
    let status = catch_unwind(AssertUnwindSafe(|| registry::stop(core)));
    status.unwrap_or(Status::Panic) as i32
}

/// The shell's sink and context. The context is kept as an address so the
/// target can move to the core's thread; the shell guarantees it stays valid
/// until `wt_core_stop` returns.
#[derive(Clone, Copy)]
pub(crate) struct SinkTarget {
    sink: RawSink,
    context: usize,
}

impl SinkTarget {
    pub(crate) fn deliver(&self, envelope: &[u8]) {
        // The shell copies the bytes during the call; they are valid until
        // it returns.
        (self.sink)(
            self.context as *mut c_void,
            envelope.as_ptr(),
            envelope.len(),
        );
    }

    /// Delivers `CoreStatus` `STATE_FATAL` with `code`, for a refused start
    /// or a core that died.
    pub(crate) fn refuse(&self, code: &str) {
        let fatal = Envelope {
            id: 0,
            body: Some(envelope::Body::Event(Event {
                kind: Some(event::Kind::Status(CoreStatus {
                    state: core_status::State::Fatal as i32,
                    code: code.to_owned(),
                })),
            })),
        };
        self.deliver(&fatal.encode_to_vec());
    }
}

/// Borrows the bytes of a C argument, refusing a NULL pointer with a length
/// and anything longer than the contract allows (without reading it).
///
/// # Safety
/// When `pointer` is non-NULL it must point to `len` readable bytes that
/// stay valid and unchanged for `'a`.
pub(crate) unsafe fn borrow_bytes<'a>(pointer: *const u8, len: usize) -> Result<&'a [u8], Status> {
    if len == 0 {
        return Ok(&[]);
    }
    if pointer.is_null() {
        return Err(Status::NullArgument);
    }
    if len > MAX_MESSAGE_LEN {
        return Err(Status::TooLarge);
    }
    // SAFETY: non-NULL and, by the caller's contract, `len` readable bytes
    // valid for 'a; `len` is at most MAX_MESSAGE_LEN, far below isize::MAX.
    Ok(unsafe { std::slice::from_raw_parts(pointer, len) })
}

/// A panic inside the core is caught and reported as a code, but Rust's
/// default hook would also print its message to stderr, and a message could
/// one day carry an address. This hook prints only where it happened.
fn install_quiet_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let location = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_else(|| "an unknown location".to_owned());
            // Not eprintln!: it panics when stderr is a closed pipe, and a
            // panic inside the hook aborts the process instead of being fenced.
            let _ = writeln!(
                std::io::stderr(),
                "webtiger_core: caught a panic at {location}"
            );
        }));
    });
}
