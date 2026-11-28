//! The Android glue (000 §5.5, 003): the JNI exports the Kotlin object
//! `com.tigercodes.webtiger.core.CoreLibrary` declares and `NativeCore` uses,
//! over the seven functions of the C ABI. Built with the `jni` feature: into
//! libwebtiger_core.so for Android, and into a host library that the Kotlin
//! client's JVM tests load, so those tests drive the real core.
//!
//! - `start` copies the configuration and keeps the Kotlin listener as a
//!   global reference beside the VM. Both are the sink's context, freed once
//!   `stop` has returned for the core (the sink is then never called again),
//!   or at once when the core refuses to start.
//! - The sink runs on the core's own thread. It attaches that thread to the
//!   VM for the thread's life, copies each envelope into a new `byte[]` and
//!   calls the listener's `onEnvelope([B)V`. Each call has a local frame of
//!   its own, so no local reference outlives it, and an exception the
//!   listener throws is cleared there: nothing may cross back into the core.
//! - Nothing here throws with words of its own or logs. A failure is a
//!   status, a zero handle, or the fast path's safe verdict; an exception the
//!   VM itself raised (out of memory) is left pending for Kotlin.
//! - Every export is fenced: a panic never unwinds into the VM.

use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Mutex, PoisonError};

use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::refs::Global;
use jni::sys::{jint, jlong};
use jni::{EnvUnowned, JValue, JavaVM, Outcome, jni_sig, jni_str};

use crate::{
    MAX_MESSAGE_LEN, Status, WT_NAV_CANCEL, WT_STORAGE_APART, WT_VERDICT_ALLOW, WtFrame,
    WtNavVerdict, WtNavigation, WtRequest, WtStorageVerdict, WtVerdict,
};

/// What the sink needs on the core's thread: the VM to attach to, and the
/// Kotlin listener.
struct Listener {
    vm: JavaVM,
    object: Global<JObject<'static>>,
}

/// The listeners of running cores, by handle: each a leaked `Box<Listener>`,
/// kept as an address until its core has stopped.
static LISTENERS: Mutex<Vec<(u64, usize)>> = Mutex::new(Vec::new());

/// Frees a listener `start` leaked.
///
/// # Safety
/// `address` must come from `Box::into_raw` in `start`, and the core must
/// never call the sink with it again.
unsafe fn free(address: usize) {
    // SAFETY: by the caller's contract this is `start`'s box, and nothing
    // uses it any more.
    drop(unsafe { Box::from_raw(address as *mut Listener) });
}

/// Runs `body`; a panic gives `fallback` instead of unwinding into the VM.
fn fenced<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|payload| {
        // A payload's own destructor may panic; it is leaked, not risked.
        std::mem::forget(payload);
        fallback
    })
}

/// What a `with_env` closure made, or `None` when JNI failed or it panicked.
fn settle<T>(outcome: Outcome<T, jni::errors::Error>) -> Option<T> {
    match outcome {
        Outcome::Ok(value) => Some(value),
        Outcome::Err(_) => None,
        Outcome::Panic(payload) => {
            // As in `fenced`.
            std::mem::forget(payload);
            None
        }
    }
}

/// A Kotlin `Long` as a core's handle or a web view's id. Neither is ever
/// negative; one that is names nothing.
fn unsigned(value: jlong) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

/// The sink (`wt_sink`): each envelope to the listener's `onEnvelope`.
extern "C" fn deliver(context: *mut c_void, envelope: *const u8, len: usize) {
    fenced((), || {
        if context.is_null() {
            return;
        }
        // SAFETY: `context` is the `Box<Listener>` `start` leaked. It lives
        // until the core has stopped, after the sink's last call.
        let listener = unsafe { &*context.cast::<Listener>() };
        // SAFETY: the core passes `len` readable bytes, valid for this call.
        let Ok(bytes) = (unsafe { crate::borrow_bytes(envelope, len) }) else {
            return;
        };
        // The closure runs in a local frame of its own, so the array is
        // released when it returns. What the listener throws is caught and
        // cleared by the attachment, and dropped here.
        let _ = listener
            .vm
            .attach_current_thread(|env| -> jni::errors::Result<()> {
                let array = env.byte_array_from_slice(bytes)?;
                env.call_method(
                    &listener.object,
                    jni_str!("onEnvelope"),
                    jni_sig!("([B)V"),
                    &[JValue::Object(&array)],
                )?;
                Ok(())
            });
    });
}

/// `CoreLibrary.version()`: `wt_core_version`, (major << 16) | minor.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_version<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jint {
    fenced(0, || jint::try_from(crate::wt_core_version()).unwrap_or(0))
}

/// `CoreLibrary.start(config, listener)`: `wt_core_start` with the listener
/// as the sink. The handle, or 0: the listener has then had the refusal's
/// `CoreStatus` when it was the core that refused, and nothing when the
/// configuration could not be read (longer than a message may be).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_start<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    config: JByteArray<'local>,
    listener: JObject<'local>,
) -> jlong {
    let prepared = env
        .with_env(
            |env| -> jni::errors::Result<Option<(Vec<u8>, Box<Listener>)>> {
                if config.len(env)? > MAX_MESSAGE_LEN {
                    return Ok(None);
                }
                let bytes = env.convert_byte_array(&config)?;
                let listener = Listener {
                    vm: env.get_java_vm()?,
                    object: env.new_global_ref(&listener)?,
                };
                Ok(Some((bytes, Box::new(listener))))
            },
        )
        .into_outcome();
    let Some(Some((bytes, listener))) = settle(prepared) else {
        return 0;
    };
    // From here on no `Env` is held: a refusal reaches the listener through
    // the sink, which attaches for itself.
    fenced(0, || {
        let context = Box::into_raw(listener);
        // SAFETY: `bytes` is a live vector for the call. `deliver` is the
        // sink for the core's life, and `context` stays valid until it is
        // freed: below when the start is refused, otherwise by `stop`.
        let handle = unsafe {
            crate::wt_core_start(bytes.as_ptr(), bytes.len(), Some(deliver), context.cast())
        };
        if handle == 0 {
            // SAFETY: a refused start called the sink before it returned, if
            // at all, and never will again.
            unsafe { free(context as usize) };
            return 0;
        }
        match jlong::try_from(handle) {
            Ok(named) => {
                LISTENERS
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push((handle, context as usize));
                named
            }
            // A handle Kotlin cannot name is a core Kotlin cannot stop.
            Err(_) => {
                crate::wt_core_stop(handle);
                // SAFETY: the core has stopped: the sink is not called again.
                unsafe { free(context as usize) };
                0
            }
        }
    })
}

/// `CoreLibrary.send(core, envelope)`: `wt_core_send`'s status.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_send<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    core: jlong,
    envelope: JByteArray<'local>,
) -> jint {
    let copied = env
        .with_env(|env| -> jni::errors::Result<Option<Vec<u8>>> {
            // A message too long to be one is refused without being copied.
            if envelope.len(env)? > MAX_MESSAGE_LEN {
                return Ok(None);
            }
            Ok(Some(env.convert_byte_array(&envelope)?))
        })
        .into_outcome();
    match settle(copied) {
        Some(Some(bytes)) => fenced(Status::Panic as jint, || {
            // SAFETY: `bytes` is a live vector for the call.
            unsafe { crate::wt_core_send(unsigned(core), bytes.as_ptr(), bytes.len()) }
        }),
        Some(None) => Status::TooLarge as jint,
        // The array could not be read.
        None => Status::NullArgument as jint,
    }
}

/// `CoreLibrary.stop(core)`: `wt_core_stop`'s status. The listener is freed
/// once the core has made its last call to it.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_stop<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    core: jlong,
) -> jint {
    fenced(Status::Panic as jint, || {
        let handle = unsigned(core);
        let status = crate::wt_core_stop(handle);
        // Only the caller that stopped the core frees its listener: another
        // that lost the race is told the handle is unknown while the core's
        // thread may still be inside the sink.
        if status == Status::Ok as jint {
            let freed = {
                let mut listeners = LISTENERS.lock().unwrap_or_else(PoisonError::into_inner);
                listeners
                    .iter()
                    .position(|(named, _)| *named == handle)
                    .map(|index| listeners.swap_remove(index).1)
            };
            if let Some(address) = freed {
                // SAFETY: `wt_core_stop` returned OK, so the core's thread
                // has ended and the sink is never called again.
                unsafe { free(address) };
            }
        }
        status
    })
}

/// `CoreLibrary.navCheck(core, url, webViewId, flags)`: `wt_nav_check`'s
/// verdict (`WT_NAV_*`). `WT_NAV_CANCEL` on any failure.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_navCheck<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    core: jlong,
    url: JString<'local>,
    web_view_id: jlong,
    flags: jint,
) -> jint {
    let refused = WT_NAV_CANCEL.cast_signed();
    let action = env
        .with_env(|env| -> jni::errors::Result<jint> {
            let Ok(flags) = u32::try_from(flags) else {
                return Ok(refused);
            };
            let url = url.mutf8_chars(env)?;
            let text = url.to_str();
            let navigation = WtNavigation {
                url: text.as_ptr(),
                url_len: text.len(),
                web_view_id: unsigned(web_view_id),
                flags,
            };
            let mut verdict = WtNavVerdict {
                action: WT_NAV_CANCEL,
            };
            // SAFETY: `navigation` and `verdict` are live for the call, and
            // `text` covers the address's bytes for as long.
            unsafe { crate::wt_nav_check(unsigned(core), &raw const navigation, &raw mut verdict) };
            Ok(jint::try_from(verdict.action).unwrap_or(refused))
        })
        .into_outcome();
    settle(action).unwrap_or(refused)
}

/// `CoreLibrary.guardCheck(core, url, sourceUrl, webViewId, resourceType)`:
/// `wt_guard_check`'s verdict (`WT_VERDICT_*`). `WT_VERDICT_ALLOW` on any
/// failure: a blocker that fails must not break the page.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_guardCheck<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    core: jlong,
    url: JString<'local>,
    source_url: JString<'local>,
    web_view_id: jlong,
    resource_type: jint,
) -> jint {
    let allowed = WT_VERDICT_ALLOW.cast_signed();
    let action = env
        .with_env(|env| -> jni::errors::Result<jint> {
            let url = url.mutf8_chars(env)?;
            let source_url = source_url.mutf8_chars(env)?;
            let (text, source) = (url.to_str(), source_url.to_str());
            let request = WtRequest {
                url: text.as_ptr(),
                url_len: text.len(),
                source_url: source.as_ptr(),
                source_url_len: source.len(),
                web_view_id: unsigned(web_view_id),
                resource_type: u32::try_from(resource_type).unwrap_or(0),
            };
            let mut verdict = WtVerdict {
                action: WT_VERDICT_ALLOW,
            };
            // SAFETY: `request` and `verdict` are live for the call, and the
            // two texts cover their addresses' bytes for as long.
            unsafe { crate::wt_guard_check(unsigned(core), &raw const request, &raw mut verdict) };
            Ok(jint::try_from(verdict.action).unwrap_or(allowed))
        })
        .into_outcome();
    settle(action).unwrap_or(allowed)
}

/// `CoreLibrary.storageCheck(core, origin, topOrigin, pageUrl, webViewId)`:
/// `wt_storage_check`'s verdict (`WT_STORAGE_*`). `WT_STORAGE_APART` on any
/// failure: a third party is let at its storage on the core's word alone.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_tigercodes_webtiger_core_CoreLibrary_storageCheck<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    core: jlong,
    origin: JString<'local>,
    top_origin: JString<'local>,
    page_url: JString<'local>,
    web_view_id: jlong,
) -> jint {
    let apart = WT_STORAGE_APART.cast_signed();
    let action = env
        .with_env(|env| -> jni::errors::Result<jint> {
            let origin = origin.mutf8_chars(env)?;
            let top_origin = top_origin.mutf8_chars(env)?;
            let page_url = page_url.mutf8_chars(env)?;
            let (of_frame, of_top, of_page) =
                (origin.to_str(), top_origin.to_str(), page_url.to_str());
            let frame = WtFrame {
                origin: of_frame.as_ptr(),
                origin_len: of_frame.len(),
                top_origin: of_top.as_ptr(),
                top_origin_len: of_top.len(),
                page_url: of_page.as_ptr(),
                page_url_len: of_page.len(),
                web_view_id: unsigned(web_view_id),
            };
            let mut verdict = WtStorageVerdict {
                action: WT_STORAGE_APART,
            };
            // SAFETY: `frame` and `verdict` are live for the call, and the
            // three texts cover their bytes for as long.
            unsafe { crate::wt_storage_check(unsigned(core), &raw const frame, &raw mut verdict) };
            Ok(jint::try_from(verdict.action).unwrap_or(apart))
        })
        .into_outcome();
    settle(action).unwrap_or(apart)
}
