//! Browser backend sketch: the browser owns the loop, so there is nothing to run.
//! Compiles for wasm32-unknown-unknown; NOT run in a browser (no JS runtime used here).
use crate::{Executor, Task};
use std::time::Duration;
use wasm_bindgen::{prelude::*, JsValue};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = setTimeout)]
    fn set_timeout(cb: &JsValue, ms: i32) -> i32;
}

pub struct BrowserExec;

fn after(ms: i32, task: Task) {
    set_timeout(&Closure::once_into_js(task), ms);
}

impl Executor for BrowserExec {
    // setTimeout(0) yields to the browser event loop (a macrotask). Production would
    // use wasm_bindgen_futures::spawn_local / queueMicrotask for cheaper spawns.
    fn spawn(&self, task: Task) {
        after(0, task);
    }
    fn timer(&self, d: Duration, cb: Task) {
        after(d.as_millis().min(i32::MAX as u128) as i32, cb);
    }
    // register_io is absent: a WebSocket's `onmessage` closure calls `spawn` instead.
    fn run_until_idle(&self) {}
}
