//! Same workload for every backend. Prints `key value` lines.
use crate::{Executor, Next};
use std::{
    cell::{Cell, RefCell},
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::AsRawFd,
    rc::Rc,
    time::{Duration, Instant},
};

const MSG: usize = 16;

fn timer_accuracy<E: Executor + 'static>(ex: &Rc<E>) {
    let late = Rc::new(RefCell::new(Vec::new()));
    let start = Instant::now();
    for i in 1..=50u32 {
        let (late, d) = (late.clone(), Duration::from_millis(2 * i as u64));
        ex.timer(d, Box::new(move || late.borrow_mut().push(start.elapsed().saturating_sub(d))));
    }
    ex.run_until_idle();
    let l = late.borrow();
    let mean = l.iter().sum::<Duration>() / l.len() as u32;
    println!("timer_late_mean_us {}", mean.as_micros());
    println!("timer_late_max_us {}", l.iter().max().unwrap().as_micros());
}

fn timer_throughput<E: Executor + 'static>(ex: &Rc<E>) {
    const N: u32 = 100_000;
    let fired = Rc::new(Cell::new(0u32));
    let start = Instant::now();
    for i in 0..N {
        let fired = fired.clone();
        ex.timer(Duration::from_millis((i % 10) as u64), Box::new(move || fired.set(fired.get() + 1)));
    }
    ex.run_until_idle();
    assert_eq!(fired.get(), N);
    println!("timers_per_s {:.0}", N as f64 / start.elapsed().as_secs_f64());
}

/// Ping-pong `n` 16-byte messages between a client and an echo server on one loop.
fn echo<E: Executor + 'static>(ex: &Rc<E>, n: u32) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    client.set_nonblocking(true).unwrap();
    client.set_nodelay(true).unwrap();

    let ex2 = ex.clone();
    ex.register_io(listener.as_raw_fd(), Box::new(move || match listener.accept() {
        Ok((mut s, _)) => {
            s.set_nonblocking(true).unwrap();
            s.set_nodelay(true).unwrap();
            let mut buf = [0u8; 256];
            ex2.register_io(s.as_raw_fd(), Box::new(move || loop {
                match s.read(&mut buf) {
                    Ok(0) => return Next::Done,
                    // shortcut: assumes a 16-byte write never hits WouldBlock; fine for ping-pong.
                    Ok(k) => s.write_all(&buf[..k]).unwrap(),
                    Err(e) if e.kind() == ErrorKind::WouldBlock => return Next::Wait,
                    Err(e) => panic!("{e}"),
                }
            }));
            Next::Done // one connection is all we need
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => Next::Wait,
        Err(e) => panic!("{e}"),
    }));

    let (mut got, mut count) = (0usize, 0u32);
    let fd = client.as_raw_fd();
    let start = Instant::now();
    client.write_all(&[7; MSG]).unwrap();
    ex.register_io(fd, Box::new(move || loop {
        let mut buf = [0u8; MSG];
        match client.read(&mut buf) {
            Ok(0) => panic!("server closed early"),
            Ok(k) => {
                got += k;
                if got == MSG {
                    got = 0;
                    count += 1;
                    if count == n {
                        return Next::Done;
                    }
                    client.write_all(&[7; MSG]).unwrap();
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Next::Wait,
            Err(e) => panic!("{e}"),
        }
    }));
    ex.run_until_idle();
    println!("echo_msgs_per_s {:.0}", n as f64 / start.elapsed().as_secs_f64());
}

/// `idle` measures construct + run an empty loop; `all` runs the full workload.
pub fn main<E: Executor + 'static>(new: impl Fn() -> E) {
    let t = Instant::now();
    let ex = Rc::new(new());
    ex.run_until_idle();
    println!("idle_us {}", t.elapsed().as_micros());
    if std::env::args().nth(1).as_deref() == Some("idle") {
        return;
    }
    timer_accuracy(&ex);
    timer_throughput(&ex);
    echo(&ex, 20_000);
}
