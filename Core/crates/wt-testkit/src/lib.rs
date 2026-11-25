//! Test support (000 §8): a **fake shell** that drives the whole contract in
//! process, with a **scripted web engine** behind it. "Open a tab, load a
//! page that redirects, go back" is a unit test that runs in microseconds,
//! on any machine, with no web view.
//!
//! ```ignore
//! wt_testkit::run(async {
//!     let mut shell = FakeShell::fake();
//!     shell.engine.page("https://example.com/", "Example");
//!     let (window, tab) = shell.open_window().await;
//!     …
//! });
//! ```
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test support: a failure here is a failing test"
)]

mod engine;
mod shell;

use std::future::Future;

pub use engine::{FakeEngine, FakePage, FakeWebView, picture_of};
pub use shell::FakeShell;

/// The first bytes of a PNG, as an icon is handed over (046): its header,
/// which says it is this many pixels each way, and a mark that tells one
/// picture from another. (Nothing decodes it: the core reads what a picture
/// says of itself, and keeps the bytes.)
#[must_use]
pub fn png(width: u32, height: u32, mark: u8) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13_u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 6, 0, 0, 0, mark, mark, mark, mark]);
    bytes
}

/// Runs a test's future the way the core's thread runs the dispatcher: on a
/// current-thread runtime, inside a `LocalSet`. Time is paused, so a timeout
/// elapses as soon as nothing else can make progress.
pub fn run<F: Future>(future: F) -> F::Output {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .expect("a current-thread runtime");
    tokio::task::LocalSet::new().block_on(&runtime, future)
}
