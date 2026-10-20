#!/bin/sh
# Runs every benches/*.spk RUNS times with the release binary; prints median/min wall ms
# and writes target/bench/results.json. Needs python3 (timing + median).
# TODO(SPK-405): JSON round-trip benchmark. TODO(SPK-506): HTTP req/s benchmark. Neither exists yet.
set -eu
RUNS=${RUNS:-5}
BIN=target/release/speak
OUT=target/bench/results.json
mkdir -p target/bench
python3 - "$RUNS" "$BIN" "$OUT" benches/*.spk <<'PY'
import json, statistics, subprocess, sys, time, os
runs, bin_, out, *files = sys.argv[1:]
res = []
for f in files:
    ts = []
    for _ in range(int(runs)):
        t = time.perf_counter()
        subprocess.run([bin_, "run", f], check=True, stdout=subprocess.DEVNULL)
        ts.append((time.perf_counter() - t) * 1000)
    r = {"name": os.path.basename(f)[:-4], "median_ms": round(statistics.median(ts), 1), "min_ms": round(min(ts), 1)}
    print("%-10s median %9.1f ms   min %9.1f ms" % (r["name"], r["median_ms"], r["min_ms"]))
    res.append(r)
json.dump({"runs": int(runs), "unit": "ms", "benchmarks": res}, open(out, "w"), indent=1)
PY
