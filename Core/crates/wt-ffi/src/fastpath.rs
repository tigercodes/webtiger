//! The fast path (000 §5.5): the two questions an engine asks on its own
//! threads and waits for. Synchronous, lock-free, allocation-free.
//!
//! - [`wt_nav_check`]: may this web view make this navigation? Android's
//!   `shouldOverrideUrlLoading` must answer before it returns.
//! - [`wt_guard_check`]: may this page make this request? Asked for every
//!   request on the engines that have a request callback. Guard arrives in
//!   W4; until then the answer is always "allow", and the call exists so each
//!   shell wires and measures its callback from the first build.
//!
//! The rules both will read are an immutable snapshot the core swaps when
//! they change. W0 has no rules that change, so there is no snapshot yet:
//! the navigation policy is `wt-core`'s, which depends on the address alone.

use std::panic::{AssertUnwindSafe, catch_unwind};

use wt_core::{NavigationAction, check_navigation};

use crate::{Status, registry};

/// `wt_request`: a request a page is about to make.
#[repr(C)]
pub struct WtRequest {
    pub url: *const u8,
    pub url_len: usize,
    pub source_url: *const u8,
    pub source_url_len: usize,
    pub web_view_id: u64,
    pub resource_type: u32,
}

/// `wt_verdict`.
#[repr(C)]
pub struct WtVerdict {
    pub action: u32,
}

pub const WT_VERDICT_ALLOW: u32 = 0;
pub const WT_VERDICT_BLOCK: u32 = 1;

/// `wt_navigation`: a navigation a web view is about to make.
#[repr(C)]
pub struct WtNavigation {
    pub url: *const u8,
    pub url_len: usize,
    pub web_view_id: u64,
    pub flags: u32,
}

pub const WT_NAV_MAIN_FRAME: u32 = 1;
pub const WT_NAV_USER_GESTURE: u32 = 2;

/// `wt_nav_verdict`.
#[repr(C)]
pub struct WtNavVerdict {
    pub action: u32,
}

pub const WT_NAV_ALLOW: u32 = 0;
pub const WT_NAV_CANCEL: u32 = 1;
pub const WT_NAV_EXTERNAL: u32 = 2;

/// Guard's decision for one request; see webtiger_core.h.
///
/// # Safety
/// `request` must be NULL or point to a `wt_request` whose address pointers
/// each cover their length in readable bytes for the duration of the call.
/// `out` must be NULL or point to a writable `wt_verdict`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wt_guard_check(
    core: u64,
    request: *const WtRequest,
    out: *mut WtVerdict,
) -> i32 {
    if out.is_null() {
        return Status::NullArgument as i32;
    }
    // A blocker that fails must not break the page: the verdict is ALLOW
    // unless Guard says otherwise.
    // SAFETY: `out` is non-NULL and, by the caller's contract, writable.
    unsafe { (*out).action = WT_VERDICT_ALLOW };
    if request.is_null() {
        return Status::NullArgument as i32;
    }
    let status = catch_unwind(AssertUnwindSafe(|| {
        if !registry::is_running(core) {
            return Status::UnknownCore;
        }
        // SAFETY: `request` is non-NULL and, by the caller's contract, a
        // valid `wt_request` for this call.
        let request = unsafe { &*request };
        // SAFETY: the caller guarantees each pointer covers its length.
        let addresses = unsafe {
            (
                borrow_text(request.url, request.url_len),
                borrow_text(request.source_url, request.source_url_len),
            )
        };
        match addresses {
            (Some(_), Some(_)) => Status::Ok,
            _ => Status::BadArgument,
        }
    }));
    status.unwrap_or(Status::Panic) as i32
}

/// Whether a web view may make a navigation; see webtiger_core.h.
///
/// # Safety
/// `navigation` must be NULL or point to a `wt_navigation` whose `url`
/// covers `url_len` readable bytes for the duration of the call. `out` must
/// be NULL or point to a writable `wt_nav_verdict`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wt_nav_check(
    core: u64,
    navigation: *const WtNavigation,
    out: *mut WtNavVerdict,
) -> i32 {
    if out.is_null() {
        return Status::NullArgument as i32;
    }
    // Refused unless the policy says otherwise.
    // SAFETY: `out` is non-NULL and, by the caller's contract, writable.
    unsafe { (*out).action = WT_NAV_CANCEL };
    if navigation.is_null() {
        return Status::NullArgument as i32;
    }
    let verdict = catch_unwind(AssertUnwindSafe(|| {
        if !registry::is_running(core) {
            return Err(Status::UnknownCore);
        }
        // SAFETY: `navigation` is non-NULL and, by the caller's contract, a
        // valid `wt_navigation` for this call.
        let navigation = unsafe { &*navigation };
        // SAFETY: the caller guarantees `url` covers `url_len` bytes.
        let url = unsafe { borrow_text(navigation.url, navigation.url_len) }
            .ok_or(Status::BadArgument)?;
        let main_frame = navigation.flags & WT_NAV_MAIN_FRAME != 0;
        Ok(match check_navigation(url, main_frame) {
            NavigationAction::Allow => WT_NAV_ALLOW,
            NavigationAction::Cancel => WT_NAV_CANCEL,
            NavigationAction::External => WT_NAV_EXTERNAL,
        })
    }));
    match verdict {
        Ok(Ok(action)) => {
            // SAFETY: as above.
            unsafe { (*out).action = action };
            Status::Ok as i32
        }
        Ok(Err(status)) => status as i32,
        Err(_) => Status::Panic as i32,
    }
}

/// Borrows a C string argument as text. `None` when it is NULL with a
/// length, longer than a message may be, or not UTF-8.
///
/// # Safety
/// When `pointer` is non-NULL it must point to `len` readable bytes that
/// stay valid and unchanged for `'a`.
unsafe fn borrow_text<'a>(pointer: *const u8, len: usize) -> Option<&'a str> {
    // SAFETY: the caller's contract is `borrow_bytes`'s.
    let bytes = unsafe { crate::borrow_bytes(pointer, len) }.ok()?;
    std::str::from_utf8(bytes).ok()
}
