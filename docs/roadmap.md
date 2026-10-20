# Speak: Ticket Backlog for a Full Interpreted Language

## How to use this backlog

This backlog takes Speak from a teaching interpreter to a general-purpose interpreted language with Node-style async networking, in 5 milestones and 72 tickets. Networking, files and processes arrive as standard-library modules written in Rust behind a native API, so the core language stays small.

- IDs: `SPK-<epic><nn>`, e.g. SPK-501 is the first networking ticket. Use the ID as the prefix of PR titles and branch names.
- Size: S = up to 3 days, M = 1 to 2 weeks, L = 3 to 6 weeks, XL = split before starting.
- Priority: P0 = needed for the networked-Speak milestone, P1 = needed for 1.0, P2 = after 1.0.
- Needs: tickets that must land first. The critical path is in the milestones section.
- Labels: one per epic (core, native-api, async, stdlib, net, pkg, tooling, i18n, security).

One rule applies to every ticket: no hard-coded English names. Every keyword, builtin, stdlib function name, type name and error message goes through the locale files, so a Swahili program can open a socket as easily as an English one.
A second rule protects Speak's edge with beginners: advanced features arrive as modules, so a beginner's FizzBuzz looks the same after this work as it does today.

## Where Speak stands today

Speak is a 4,400-line Rust tree-walking interpreter with a sound lexer and parser, but its scoping, memory model and native-function design must change before it can host async networking. Findings below come from reading the main branch of speak-lang/speak (lexer 974 lines, parser 1,359, evaluator 1,466, runtime 470).

| Area | Today | Why it blocks a full language |
| --- | --- | --- |
| Scoping | A function value stores only its syntax node. A call pushes its arguments onto the caller's stack, so scoping is dynamic. | Callbacks and async handlers need closures that capture their defining scope. |
| Stack and memory | push_frame clones the whole frame chain on every call. Objects and arrays are copied, not shared. | Cost grows with stack depth. Two handlers cannot share a connection or a cache. |
| Values | f64 numbers, bool, string, named objects, typed arrays, functions. | No integers, bytes, maps or error values. Sockets and files need bytes and handles. |
| Native functions | Plain fn pointers that see only the stack. Builtins: print, println, sprint, len, mod. | No captured state, no runtime access, no async. |
| Modules | The mod builtin is `unimplemented!()` and panics. TODO.md lists "Small stdlib" as open. | No imports, so no libraries. |
| Errors | An error is a reason plus a message. No stack trace, no way to catch an error in Speak code. | Network code fails often and must recover. |
| Types | Signatures are typed, but the return value is never checked against the declared return type. | The README promises this check. |
| Unicode | len counts bytes. for over a string casts each character to a u8. | Breaks Amharic, Yoruba diacritics and any non-ASCII script. |
| Locale | SPEAK env var sets one process-global locale via rust_i18n::set_locale. Error strings exist in en.yml and sw.yml. | Files in different languages cannot share a process or import each other. |
| Tests and tooling | 9 Rust unit tests, a samples folder, a REPL with a hard-coded English exit. | No conformance suite to catch regressions during a rewrite. |

Tail calls are already optimized through thunks, and for … in with break and continue works. Both should survive the changes below.

## Epic 1: Core language and runtime

Closures, shared references and modules come first, because every later epic builds on them. Start with the spec and the test suite so the rewrite has a safety net.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-101 | Write language spec v0.2 | Grammar, scoping rules, value types and evaluation order are written in docs/spec.md, with examples in English and Swahili. | M | P0 |  |
| SPK-102 | Conformance test suite | Every sample runs as a golden test in every locale in CI. New tickets add a test before the fix. | M | P0 |  |
| SPK-103 | Lexical scoping and real closures | Functions capture their defining environment. Environments are shared `Rc<RefCell<Env>>` and calls no longer clone the frame chain. A counter-closure test passes. | L | P0 | 101, 102 |
| SPK-104 | Shared references for objects and arrays | Objects and arrays live on a shared heap, so two variables can point at one object. Memory strategy is decided (see open decisions) and cycles do not leak in a 1-hour soak test. | L | P0 | 103 |
| SPK-105 | Benchmark harness | make bench runs fib(30), string building, JSON round-trip and later HTTP req/s, and stores results in CI to catch regressions. | S | P0 |  |
| SPK-106 | Errors as values plus catch | Speak code can raise and catch errors with localized keywords. Errors carry a type, message and stack trace with file:line. | L | P0 | 103 |
| SPK-107 | Error output with source context | Errors print the offending line with a caret under the column, in the active locale. | M | P1 | 106 |
| SPK-108 | New core types: integer, bytes, map | Integers coexist with floats. A bytes type supports slicing and encoding. A map type takes arbitrary string keys without a declared object name. | L | P0 | 104 |
| SPK-109 | Unicode correctness | len counts characters, for over a string yields one-character strings, and slicing is character-based. Tests cover Amharic and Yoruba text. | S | P0 | 102 |
| SPK-110 | Enforce declared return types | A function returning the wrong type raises a localized runtime error, as the README promises. | S | P1 | 102 |
| SPK-111 | Ranges and an unbounded loop | A range builtin and a localized loop-forever form exist, for servers and counters. | S | P1 | 103 |
| SPK-112 | Module system | The mod builtin no longer panics. Imports resolve file-relative paths, cache modules, expose explicit exports and report import cycles. | M | P0 | 103 |
| SPK-113 | Bytecode compiler and VM | Only if SPK-105 shows the tree-walker misses targets after SPK-103 and SPK-104. AST compiles to bytecode with at least 3x speedup on fib(30). | XL | P1 | 105 |

## Epic 2: Native extension API

This epic turns today's fn-pointer builtins into a real extension API, so files, sockets and processes can be written as Rust libraries. It is the bridge between the core language and everything in Epics 3 to 5.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-201 | Native function API v1 | Native functions are boxed closures with a runtime handle, argument-checking helpers and localized names. Existing builtins are ported. | M | P0 | 103 |
| SPK-202 | Native module registry | A Rust crate registers a module such as std/fs with its functions and a name table per locale. Speak code imports it like any module. | M | P0 | 112, 201 |
| SPK-203 | Resource handle type | An opaque handle value wraps native resources such as sockets and files, with explicit close and automatic cleanup when dropped. | M | P0 | 104, 201 |
| SPK-204 | Async native functions | A native function can return a pending task that the event loop completes. | M | P0 | 201, 302 |
| SPK-205 | Move builtins into std/io | print, println, sprint and len live in a std module, closing the TODO.md item. A prelude keeps beginner programs unchanged. | S | P0 | 202 |
| SPK-206 | Third-party native extensions | Outside crates can ship native modules without forking the interpreter. Mechanism follows the open decision on dynamic libraries versus WASM components. | XL | P2 | 202 |

## Epic 3: Async runtime and event loop

Speak gets a single-threaded event loop like Node: the script runs, then the loop keeps going while timers, sockets or tasks are pending. Async/await from day one avoids Node's callback-then-promise detour.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-301 | Runtime spike and decision | A 2-day spike compares tokio's current-thread runtime with a custom mio loop, records the choice and measures startup cost. | S | P0 |  |
| SPK-302 | Event loop integration | speak run executes the script, then runs the loop until no tasks or handles remain. The REPL keeps the loop alive between lines. | L | P0 | 103, 301 |
| SPK-303 | Tasks and async/await | A task value and localized async/await keywords exist. Awaiting inside a function suspends it without blocking the loop. | L | P0 | 302 |
| SPK-304 | Timers | Sleep, timeout and interval exist with cancellation, all localized. | S | P0 | 303 |
| SPK-305 | Streams and async iteration | for chunk in stream works over async sources with backpressure, so network reads use the same loop syntax as arrays. | L | P0 | 303 |
| SPK-306 | Failures and shutdown | Unhandled task errors print a trace and exit non-zero. Tasks can be cancelled, and Ctrl+C closes open handles cleanly. | M | P0 | 106, 303 |
| SPK-307 | Background workers | CPU-heavy work runs on a worker pool or isolated worker contexts that pass messages, without blocking the loop. | L | P2 | 303 |

## Epic 4: Standard library

Nine std modules give Speak the everyday reach of Node or Python. Each module ships with English and Swahili name tables and its own tests.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-401 | std/fs | Read and write files as text or bytes, list and create directories, check existence. Async versions exist alongside sync ones. | M | P0 | 202, 204 |
| SPK-402 | std/os and std/path | Script arguments, environment variables, working directory, platform name, and path join and split. | S | P0 | 202 |
| SPK-403 | std/process | Spawn child processes with piped stdin, stdout and stderr, read exit codes, and exit the current process. | M | P1 | 204, 305 |
| SPK-404 | std/time | Wall clock, monotonic clock, durations and date formatting. | S | P0 | 202 |
| SPK-405 | std/json | Parse JSON into maps and arrays, and serialize back, with clear localized errors on bad input. | M | P0 | 108, 202 |
| SPK-406 | std/text | Split, join, trim, replace, case changes, string formatting, and regex through the existing regex dependency. | M | P0 | 109, 202 |
| SPK-407 | std/math and std/random | Common math functions, integer helpers and a seedable random generator. | S | P1 | 108 |
| SPK-408 | std/crypto and std/encoding | SHA-256, HMAC, secure random bytes, base64 and hex. | M | P1 | 108 |
| SPK-409 | std/assert | Assertion helpers with readable, localized failure messages, used by the test runner. | S | P0 | 106 |

## Epic 5: Networking

The target demo is an HTTP JSON API written entirely in Swahili Speak. Native code does the protocol work, using Rust crates such as tokio, rustls and hyper, while Speak exposes it through localized streams and handlers.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-501 | std/net TCP | Listen, accept and connect. Connections are handles with async read and write and stream iteration. An echo server passes a 1,000-connection test. | L | P0 | 203, 305, 108 |
| SPK-502 | DNS resolution | Async hostname lookup returns all addresses, with timeouts. | S | P0 | 501 |
| SPK-503 | TLS | Client and server TLS through rustls, with certificate loading and verification on by default. | M | P0 | 501 |
| SPK-504 | URL and query parsing | Parse and build URLs, query strings and form encoding. | S | P0 | 406 |
| SPK-505 | std/http client | GET, POST and friends with headers, JSON bodies, timeouts and redirects. Works over TLS. | M | P0 | 502, 503, 504, 405 |
| SPK-506 | std/http server | A handler receives a request and returns a response. Supports keep-alive and streaming bodies, and reaches the req/s target set in SPK-105. | L | P0 | 501, 504 |
| SPK-507 | Router library in Speak | A small routing and middleware library written in Speak itself, to prove the language can carry real libraries. | M | P1 | 506, 112 |
| SPK-508 | UDP | Bind, send and receive datagrams. | M | P1 | 501 |
| SPK-509 | WebSockets | Client and server, with messages as an async stream. | M | P1 | 506 |
| SPK-510 | Networking examples | Echo server, chat server and JSON API, each in English and Swahili, run in CI. | S | P0 | 506 |

## Epic 6: Packages

Start with git-based packages and a lockfile, and add a hosted registry only once there are packages worth hosting. The toml crate is already a dependency, so the manifest can use TOML.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-601 | Package manifest | A speak.toml declares name, version, entry file, dependencies and the locales the package's public names support. | S | P1 | 112 |
| SPK-602 | Install from git | `speak add <git-url>` fetches a package at a tag. A lockfile records versions and content hashes. | L | P1 | 601 |
| SPK-603 | Package resolution | Imports resolve from a global content-addressed cache rather than a per-project folder. Two versions of one package can coexist. | M | P1 | 602 |
| SPK-604 | Scaffolding | speak new creates a project with a manifest, an entry file and a test, in the chosen locale. | S | P1 | 601 |
| SPK-605 | Hosted registry | Publish, search and semantic-version resolution through a registry service. | XL | P2 | 603 |

## Epic 7: Developer tooling

A test runner and a better REPL come first because they speed up every other epic. Editor support through LSP is what makes Speak feel like a real language to newcomers.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-701 | speak test | Finds `*_test.spk` files, runs them, reports pass and fail counts in the active locale, and exits non-zero on failure. | M | P0 | 409, 112 |
| SPK-702 | REPL upgrade | Line editing and history through rustyline, multi-line input, and a localized exit command, replacing the hard-coded English exit in main.rs. | S | P1 |  |
| SPK-703 | speak fmt | Formats code consistently and keeps each file's keywords in the language they were written in. | M | P1 | 101 |
| SPK-704 | Language server | Diagnostics, go-to-definition, hover with localized docs, and completion of keywords and std names in the file's locale. | L | P1 | 112, 202 |
| SPK-705 | Release targets | The release workflow adds Linux ARM64, macOS Apple Silicon and Android (Termux) to the current Windows, Linux x86-64 and Intel macOS builds, plus an install script. | S | P1 |  |
| SPK-706 | speak doc | Generates reference pages from doc comments, one set per locale. | M | P2 | 704 |
| SPK-707 | Debugger | Breakpoints and stepping through the Debug Adapter Protocol, usable from VS Code. | XL | P2 | 704 |

## Epic 8: Multilingual integrity

This epic keeps Speak's reason for existing intact as the language grows. Each stdlib function gets one canonical ID with a name per locale, so English and Swahili code call the same function and can import each other.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-801 | Per-file locale | Each file declares its language through a header line or file extension. The process-global set_locale is gone, and two files in different languages run in one program. | M | P0 | 112 |
| SPK-802 | Canonical stdlib IDs | Every std function and type has a stable ID plus a name per locale. A missing translation falls back to English with a warning. | M | P0 | 202, 801 |
| SPK-803 | Cross-language imports | A Swahili file imports an English package and calls it with Swahili names, and the reverse, through the canonical IDs. | L | P1 | 802, 603 |
| SPK-804 | Locale completeness check | CI fails when any locale file is missing a key that en.yml has. | S | P0 |  |
| SPK-805 | Localized system errors | OS and network errors such as connection refused or file not found map to locale strings rather than raw English from Rust. | M | P0 | 106, 802 |
| SPK-806 | speak translate | Rewrites a file's keywords and std names into another locale without touching user identifiers or strings. | M | P1 | 802 |
| SPK-807 | Language-mixing mode | A file can declare several locales, such as Swahili and English, and use keywords from both. The lexer rejects words that are a keyword in one declared locale and an identifier in another. | L | P1 | 801 |
| SPK-808 | Locale pack guide | A template and guide let a native speaker add a language by editing YAML, with no Rust knowledge. | S | P1 | 802 |

## Epic 9: Security and the WASM playground

Once Speak can touch files and networks, scripts need permissions, and the browser playground must keep compiling. The playground embeds the interpreter as WASM, so tokio and raw sockets must sit behind feature flags.

| ID | Ticket | Done when | Size | Pri | Needs |
| --- | --- | --- | --- | --- | --- |
| SPK-901 | Keep the WASM build green | CI builds core for wasm32. Native-only modules compile out behind feature flags, and the playground runs every non-network sample. | M | P0 | 202 |
| SPK-902 | Browser-backed std modules | In the playground, std/http client calls go through the browser's fetch and std/fs uses an in-memory file system. | M | P1 | 901, 505 |
| SPK-903 | Simulated network for teaching | An in-memory network lets several Speak "machines" in one playground exchange messages, for teaching system design without real sockets. | M | P1 | 901, 501 |
| SPK-904 | Permission flags | File, network, environment and process access are denied by default and granted with flags such as --allow-net, as Deno does. | M | P0 | 202 |
| SPK-905 | Resource limits | Limits on memory, run time and recursion depth, for the playground and classrooms. | M | P1 | 302 |
| SPK-906 | Fuzzing | cargo-fuzz targets for the lexer, parser and JSON module run nightly. | S | P1 | 405 |
| SPK-907 | Package integrity | Installs verify lockfile hashes, and packages can be signed. | M | P2 | 602 |

## Milestones and sequencing

| Milestone | Tickets | Scope | Gate |
| --- | --- | --- | --- |
| M1 Foundations · v0.2 | SPK-101 to 106, 108, 109, 112, 801, 804 (11) | Spec and conformance tests, lexical closures, shared heap, catchable errors, integers/bytes/maps, Unicode fixes, working imports, per-file locale | Every sample passes in English and Swahili; closure and soak tests are green |
| M2 Runtime · v0.3 | SPK-201 to 205, 301 to 306, 401, 402, 404 to 406, 409, 701, 802, 805, 901, 904 (22) | Native extension API, event loop, async/await, timers, streams, fs/os/time/json/text modules, test runner, permissions, WASM build kept green | An async file read and a timer work in both locales, and the playground still builds |
| M3 Networked Speak · v0.4 | SPK-501 to 506, 510 (7) | TCP, DNS, TLS, URL parsing, HTTP client and server; echo, chat and JSON API examples in English and Swahili | The Swahili JSON API reaches the req/s target set in SPK-105 |
| M4 Ecosystem · v0.5 | SPK-107, 110, 111, 403, 407, 408, 507 to 509, 601 to 604, 702 to 705, 803, 806, 808, 902, 903, 905, 906 (24) | Packages from git, router in Speak, UDP, WebSockets, processes, crypto, formatter, language server, cross-language imports, translate, teaching network | A package installs from git and imports cleanly from a file in another locale |
| M5 Speak 1.0 | SPK-807, 113 (2) | Language-mixing mode, then a spec freeze; bytecode VM only if benchmarks demand it | — |
| After 1.0 (P2) | SPK-206, 307, 605, 706, 707, 907 | Hosted registry, debugger, docs generator, workers, third-party native code, package signing | — |

The 40 P0 tickets fill M1 to M3, ending in a networked Speak that serves a Swahili HTTP API. M4 and M5 carry the 26 P1 tickets to 1.0, and the 6 P2 tickets wait until after it.

The longest chain runs SPK-103, 302, 303, 305, 501, 506, 510. SPK-104, 201 and 203 must also be done before 501 starts, so closures and the shared heap are the first work to begin.

## Decisions

Four of the six choices that shape the tickets above are settled. The two still open are post-1.0 (P2) or M5 work, so they don't block SPK-103.

| Decision | Ticket | Choice | Why |
| --- | --- | --- | --- |
| Memory model | SPK-104 | Reference counting (`Rc<RefCell<..>>`) with a cycle collector | Smallest change from today's code, and deterministic cleanup closes sockets and files promptly. The 1-hour soak test checks that cycles are collected. |
| Async style | SPK-303 | `async`/`await` keywords, localized in every locale | Explicit, familiar from JS and Python, and works with a tree-walker without stackful coroutines, which keeps the WASM build simple. |
| Numbers | SPK-108 | Add `i64` integers alongside `f64` | Bytes, indexing and ports need integers. Literals without a dot are integers; integer division yields a float. |
| Interpreter | SPK-113 | Keep the tree-walker; decide after SPK-105 benchmarks | Fix scoping and the heap first, then let measured numbers decide whether a bytecode VM is worth it. |

### Still open

- Third-party native code (SPK-206). Native dynamic libraries are fast but unsafe. WASM components are sandboxed and also run in the playground. Suggested: WASM components.
- Language-mixing rules (SPK-807). Decide what happens when a word is a keyword in one declared locale and a normal word in another. Suggested: reject at lex time with a localized message naming both meanings.
