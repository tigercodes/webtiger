//! The headless app (000 §5.2): **the core decides, the shell draws, the
//! engine renders.**
//!
//! - [`App`] holds the models: windows, tabs, what each address field shows.
//!   It changes them when a shell asks ([`App::request`]) or reports what a
//!   web view did ([`App::notify`]), and asks the shell for web views through
//!   a [`Host`].
//! - [`Dispatcher`] is the contract around it: envelopes in, envelopes out,
//!   subscriptions, the host bridge, and a panic fence around every request.
//!   It needs no thread or C ABI of its own, so the tests and the command
//!   line drive the whole contract in process. `wt-ffi` gives it a thread.
//!
//! Everything here runs on one thread. State is borrowed only between
//! awaits (the workspace denies `await_holding_refcell_ref`).

mod app;
mod dispatch;
mod error;
mod fence;
mod host;
mod model;

pub use app::{App, AppFuture, AppOptions, DEFAULT_SEARCH_TEMPLATE};
pub use dispatch::{CORE_VERSION, Dispatcher, MAX_PING_PAYLOAD, Outbox, StartError};
pub use error::{AppError, HostError, is_code};
pub use host::{HOST_TIMEOUT, Host, HostBridge, HostFuture};
