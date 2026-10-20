use crate::{Executor, Task};
use std::{cell::Cell, rc::Rc, time::Duration};
use tokio::{runtime::{Builder, Runtime}, sync::Notify, task::LocalSet};
#[cfg(unix)]
use {crate::Next, std::os::fd::{AsRawFd, RawFd}, tokio::io::{unix::AsyncFd, Interest}};

pub struct TokioExec {
    rt: Runtime,
    local: LocalSet,
    pending: Rc<Cell<usize>>,
    idle: Rc<Notify>,
}

#[cfg(unix)]
struct Fd(RawFd);
#[cfg(unix)]
impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl TokioExec {
    pub fn new() -> Self {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        Self { rt, local: LocalSet::new(), pending: Rc::default(), idle: Rc::new(Notify::new()) }
    }

    /// Count every unit of work so `run_until_idle` knows when the loop is empty.
    fn track(&self, fut: impl std::future::Future<Output = ()> + 'static) {
        self.pending.set(self.pending.get() + 1);
        let (p, n) = (self.pending.clone(), self.idle.clone());
        self.local.spawn_local(async move {
            fut.await;
            p.set(p.get() - 1);
            if p.get() == 0 {
                n.notify_one();
            }
        });
    }
}

impl Executor for TokioExec {
    fn spawn(&self, task: Task) {
        self.track(async move { task() });
    }

    fn timer(&self, after: Duration, cb: Task) {
        self.track(async move {
            tokio::time::sleep(after).await;
            cb();
        });
    }

    #[cfg(unix)]
    fn register_io(&self, fd: RawFd, mut cb: Box<dyn FnMut() -> Next>) {
        self.track(async move {
            #[allow(deprecated)] // shortcut: raw fd; caller guarantees it outlives cb
            let afd = AsyncFd::with_interest(Fd(fd), Interest::READABLE).unwrap();
            loop {
                let mut guard = afd.readable().await.unwrap();
                match cb() {
                    Next::Wait => guard.clear_ready(),
                    Next::Done => break,
                }
            }
            drop(afd); // deregister before cb (which owns the socket) is dropped
        });
    }

    fn run_until_idle(&self) {
        let (p, n) = (self.pending.clone(), self.idle.clone());
        self.local.block_on(&self.rt, async move {
            while p.get() > 0 {
                n.notified().await;
            }
        });
    }
}
