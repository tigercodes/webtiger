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

pub use engine::{FakeEngine, FakePage, FakeWebView};
pub use shell::FakeShell;

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
