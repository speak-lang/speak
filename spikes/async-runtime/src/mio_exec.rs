use crate::{Executor, Next, Task};
use mio::{unix::SourceFd, Events, Interest, Poll, Registry, Token};
use std::{
    cell::{Cell, RefCell},
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, VecDeque},
    os::fd::RawFd,
    time::{Duration, Instant},
};

type IoCb = Box<dyn FnMut() -> Next>;

pub struct MioExec {
    poll: RefCell<Poll>,
    registry: Registry,
    tasks: RefCell<VecDeque<Task>>,
    timers: RefCell<BinaryHeap<Reverse<(Instant, u64)>>>,
    timer_cbs: RefCell<HashMap<u64, Task>>,
    ios: RefCell<HashMap<usize, (RawFd, IoCb)>>,
    seq: Cell<u64>,
}

impl MioExec {
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        Self {
            poll: RefCell::new(poll),
            registry,
            tasks: Default::default(),
            timers: Default::default(),
            timer_cbs: Default::default(),
            ios: Default::default(),
            seq: Cell::new(0),
        }
    }

    fn next_id(&self) -> u64 {
        self.seq.set(self.seq.get() + 1);
        self.seq.get()
    }
}

impl Executor for MioExec {
    fn spawn(&self, task: Task) {
        self.tasks.borrow_mut().push_back(task);
    }

    fn timer(&self, after: Duration, cb: Task) {
        let id = self.next_id();
        self.timers.borrow_mut().push(Reverse((Instant::now() + after, id)));
        self.timer_cbs.borrow_mut().insert(id, cb);
    }

    fn register_io(&self, fd: RawFd, cb: IoCb) {
        let id = self.next_id() as usize;
        self.registry.register(&mut SourceFd(&fd), Token(id), Interest::READABLE).unwrap();
        self.ios.borrow_mut().insert(id, (fd, cb));
    }

    fn run_until_idle(&self) {
        let mut events = Events::with_capacity(256);
        loop {
            // Callbacks may call spawn/timer/register_io, so no borrow is held while one runs.
            loop {
                let t = self.tasks.borrow_mut().pop_front();
                match t {
                    Some(t) => t(),
                    None => break,
                }
            }
            let now = Instant::now();
            loop {
                let due = matches!(self.timers.borrow().peek(), Some(Reverse((w, _))) if *w <= now);
                if !due {
                    break;
                }
                let Reverse((_, id)) = self.timers.borrow_mut().pop().unwrap();
                let cb = self.timer_cbs.borrow_mut().remove(&id).unwrap();
                cb();
            }
            if !self.tasks.borrow().is_empty() {
                continue;
            }
            let next = self.timers.borrow().peek().map(|Reverse((w, _))| *w);
            if next.is_none() && self.ios.borrow().is_empty() {
                return;
            }
            let timeout = next.map(|w| w.saturating_duration_since(Instant::now()));
            self.poll.borrow_mut().poll(&mut events, timeout).unwrap();
            for ev in events.iter() {
                let id = ev.token().0;
                let Some((fd, mut cb)) = self.ios.borrow_mut().remove(&id) else { continue };
                match cb() {
                    Next::Wait => {
                        self.ios.borrow_mut().insert(id, (fd, cb));
                    }
                    Next::Done => {
                        let _ = self.registry.deregister(&mut SourceFd(&fd));
                    }
                }
            }
        }
    }
}
