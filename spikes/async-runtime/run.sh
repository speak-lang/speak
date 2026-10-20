#!/bin/sh
# Build both backends in release mode and print the numbers used in docs/decisions/0001-async-runtime.md.
set -e
cd "$(dirname "$0")"
cargo build --release --features tokio-rt,mio-rt
for b in tokio_rt mio_rt; do
  echo "== $b size_bytes $(wc -c < target/release/$b | tr -d ' ')"
  for i in 1 2 3; do target/release/$b | tr '\n' ' '; echo; done
  s=$(python3 -c 'import time;print(time.time())')
  for i in $(seq 100); do target/release/$b idle >/dev/null; done
  e=$(python3 -c 'import time;print(time.time())')
  echo "process_ms_per_idle_run $(python3 -c "print(round(($e-$s)*10,2))")"
done
