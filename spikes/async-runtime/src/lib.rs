//! SPK-301 spike: the executor interface Speak's event loop (SPK-302) would sit behind.
//!
//! Callback-based on purpose: the tree-walking interpreter will turn `await` into a
//! continuation stored in its own task table, not a Rust `Future`, so the loop only
//! needs to run closures. Everything is single-threaded (`!Send`).
use std::time::Duration;

pub type Task = Box<dyn FnOnce()>;

/// What an I/O callback tells the loop after it ran.
pub enum Next {
    /// Source drained (saw `WouldBlock`); call again on the next readiness event.
    Wait,
    /// Deregister the source and drop the callback.
    Done,
}

pub trait Executor {
    /// Queue a task to run on the loop thread, after the current one returns.
    fn spawn(&self, task: Task);
    /// Run `cb` once, no earlier than `after` from now.
    fn timer(&self, after: Duration, cb: Task);
    /// Call `cb` whenever `fd` is readable. Edge-triggered: `cb` must read until
    /// `WouldBlock` before returning `Next::Wait`. The caller keeps ownership of
    /// the fd and must keep it open until `cb` returns `Done`.
    /// Not on wasm32: browsers have no file descriptors; sockets there are
    /// WebSocket/fetch objects whose `onmessage` callbacks call `spawn`.
    #[cfg(unix)]
    fn register_io(&self, fd: std::os::fd::RawFd, cb: Box<dyn FnMut() -> Next>);
    /// Run until no tasks, timers or registered sources remain. Native only;
    /// on wasm32 the browser owns the loop, so this returns immediately.
    fn run_until_idle(&self);
}

#[cfg(feature = "tokio-rt")]
pub mod tokio_exec;
#[cfg(all(feature = "mio-rt", unix))]
pub mod mio_exec;
#[cfg(all(feature = "browser", target_arch = "wasm32"))]
pub mod browser;
#[cfg(unix)]
pub mod bench;
