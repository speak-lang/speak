# 0001: Async runtime for Speak's event loop

Status: proposed (SPK-301). Code: `spikes/async-runtime/` (`sh run.sh` reproduces the numbers).

## Context

Speak will get a single-threaded, Node-style event loop (SPK-302) with localized
`async`/`await` keywords (SPK-303). Constraints:

- Offline classroom use on Raspberry Pis: small binary, fast start, low idle cost.
- `core/` must keep building for wasm32 (browser and mobile apps). Native-only
  features may compile out there; in the browser the loop must be the browser's own.
- The interpreter is a tree walker. It will keep `await` continuations in its own
  task table, so the loop only has to run closures, not Rust `Future`s.

## Options

(a) tokio, `current_thread` runtime plus `LocalSet`, behind our trait.
(b) A hand-rolled loop on `mio` (task queue, timer heap, `poll`), behind our trait.

Both were implemented against the same trait and ran the same workload: 50 timers at
2..100 ms, 100,000 timers at 0..9 ms, and a localhost TCP echo of 20,000 16-byte
messages (ping-pong, client and server on one loop).

## Measurements

Apple Silicon Mac (macOS, kqueue), release build (`lto`, `codegen-units=1`, `strip`),
3 runs, typical values. Not a Raspberry Pi: absolute numbers will differ, ratios
should roughly hold.

| | tokio current_thread | custom mio loop |
|---|---|---|
| Binary size (bench binary, stripped) | 470 KB | 353 KB |
| Startup to idle loop, in-process (build + run empty loop) | 18-88 us | 4-21 us |
| Startup, whole process (100 runs of `idle`) | 2.7 ms | 2.7 ms |
| Timer lateness, mean / max (50 timers) | 1.8 ms / 2.4 ms | 0.41 ms / 0.5-0.9 ms |
| Timer throughput (100k timers) | 0.9-1.1 M/s | 2.9-3.4 M/s |
| TCP echo, 16-byte ping-pong | 35-36 k msg/s | 36-38 k msg/s |

Reading: process start is dominated by the OS, not the loop. Echo is bound by
syscalls and is a tie. Tokio's timers have 1 ms granularity (it rounds up), hence
about 1.8 ms mean lateness; the mio loop is limited by the OS poll timeout
(about 0.4 ms). Timer throughput differences do not matter at classroom scale.
The size gap (about 115 KB) is real but small; the bench binaries include the
bench code in both.

## wasm32-unknown-unknown

`cargo build --lib --target wasm32-unknown-unknown`:

| Backend | Result |
|---|---|
| tokio (`rt`, `time`, `sync`) | Compiles. With `net` it fails (mio). Not run: tokio's `block_on` and `Instant` do not work on this target, so it would still need a browser driver. |
| mio | Fails: "This wasm target is unsupported by mio". |
| browser (`wasm-bindgen`, `setTimeout`) | Compiles. Not run in a browser (no JS runtime here). |

Browser mapping, as in `src/browser.rs`: `spawn` and `timer` become `setTimeout`
callbacks (production: `wasm_bindgen_futures::spawn_local` or `queueMicrotask` for
spawn); `run_until_idle` is a no-op because the browser owns the loop;
`register_io` does not exist, and a WebSocket or fetch `onmessage` closure calls
`spawn` instead. So native I/O (fds) is the only part of the trait that is cfg'd
out on wasm32. aarch64 cross-compile (Raspberry Pi) was skipped: needs a linker
setup that is not trivial here. The pilot should re-run `run.sh` on a real Pi.

## Decision

Build the loop on our own small `mio` implementation behind the `Executor` trait;
do not adopt tokio.

Reasons:
- Tokio gives no capability we need: the interpreter does not use Rust futures, and
  it is single-threaded, so tokio's scheduler, work stealing and macros are unused weight.
- Mio is smaller and has better timer accuracy and throughput; echo is equal.
- The loop spike was about 100 lines (`mio_exec.rs`), so the maintenance cost is low.
- Neither option helps wasm32: tokio needs a replacement driver there anyway, and
  mio does not build. The trait is what makes wasm work, so the cost is the same.
- Mio is only compiled on native targets, so `core/` can depend on it behind
  `cfg(not(target_arch = "wasm32"))` with nothing to feature-gate in the interpreter.

Risks: we own the correctness of the timer heap and edge-triggered I/O (callbacks
must drain to `WouldBlock`). Windows is supported by mio but untested here. If the
loop later needs thread pools, signals or DNS, revisit; tokio could then be added
as a second backend behind the same trait, which the spike shows is about 80 lines.

## Proposed trait

```rust
pub type Task = Box<dyn FnOnce()>;
pub enum Next { Wait, Done }

pub trait Executor {
    fn spawn(&self, task: Task);
    fn timer(&self, after: Duration, cb: Task);
    #[cfg(unix)] // native only; no fds in the browser
    fn register_io(&self, fd: RawFd, cb: Box<dyn FnMut() -> Next>);
    fn run_until_idle(&self); // no-op on wasm32
}
```

Callbacks, not futures; single-threaded (`!Send`). Callbacks may call back into the
executor. Details are in `spikes/async-runtime/src/lib.rs`.

## What SPK-302 should do next

1. Add the trait and the mio loop to the interpreter crate (native only), with a
   no-op/browser stub for wasm32 so `core/` still builds there.
2. Keep an interpreter task table: `await` stores a continuation and registers a
   timer or I/O source whose callback resumes it via `spawn`.
3. Make I/O sources portable: take `RawFd` on unix and `RawSocket` on Windows, or
   wrap in a small `Source` type. Handle partial writes (the spike assumes small
   writes succeed).
4. Add a Windows build and a real Raspberry Pi run of `run.sh` before the pilot.
5. Decide idle semantics for `run_until_idle` in the REPL (the loop must not exit
   while the prompt waits for input).
