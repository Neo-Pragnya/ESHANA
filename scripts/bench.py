#!/usr/bin/env python3
"""Performance benchmarks with a regression gate (PERF-01).

Measures, with the built-in analytic provider:
  cold_start   the `eshana` binary answering one positions request (process start + engine + answer)
  chart        one Vedic chart with every feature, D1-D60 and the daśā, through the Python package
  batch_100    100 Vedic charts (D1, D9, daśā) for 100 different births
  batch_1000   the same for 1,000 births

Each figure is the median of several runs, in milliseconds. Two gates:
  budget    a ceiling that holds on any reasonable machine (CI included): catches a gross regression anywhere
  baseline  the numbers recorded on this machine (`--update`), compared when the machine fingerprint matches:
            fails when a figure is slower than baseline x THRESHOLD (default 1.5)

    python scripts/bench.py                 # measure and check
    python scripts/bench.py --update        # record this machine's baseline in benchmarks/perf/baseline.json
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASELINE = ROOT / "benchmarks" / "perf" / "baseline.json"
# generous ceilings (ms): a 5-10x margin over a 2024 laptop, so a slow CI runner still passes
BUDGET = {"cold_start": 500.0, "chart": 2000.0, "batch_100": 4000.0, "batch_1000": 40000.0}


def fingerprint() -> str:
    cpu = platform.processor() or platform.machine()
    if sys.platform == "darwin":
        try:
            cpu = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True).stdout.strip() or cpu
        except OSError:
            pass
    return f"{platform.system()}-{platform.machine()}-{cpu}-{os.cpu_count()}"


def median_ms(fn, runs: int) -> float:
    fn()  # warm-up (page cache, lazy tables)
    out = []
    for _ in range(runs):
        t = time.perf_counter()
        fn()
        out.append((time.perf_counter() - t) * 1000)
    return round(statistics.median(out), 2)


def births(n: int):
    for i in range(n):
        y, m, d = 1900 + i % 120, 1 + i % 12, 1 + i % 28
        yield {"op": "chart", "system": "vedic",
               "when": {"date": f"{y:04d}-{m:02d}-{d:02d}", "time": f"{i % 24:02d}:{(7 * i) % 60:02d}", "timezone": "Asia/Kolkata"},
               "place": {"latitude": 8 + (i * 0.37) % 25, "longitude": 70 + (i * 0.53) % 20},
               "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}, "divisions": [1, 9], "dasha": True}


def measure(binary: str, quick: bool) -> dict:
    from eshana import execute

    full = {"op": "chart", "system": "vedic", "when": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata"},
            "place": {"latitude": 28.6139, "longitude": 77.209}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"},
            "features": ["all"], "divisions": [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60], "dasha": True}
    b100, b1000 = list(births(100)), list(births(1000))
    cold = [binary, "positions", "--utc", "2026-01-01T00:00:00Z"]
    return {
        "cold_start": median_ms(lambda: subprocess.run(cold, check=True, capture_output=True), 5 if quick else 15),
        "chart": median_ms(lambda: execute(full), 5 if quick else 20),
        "batch_100": median_ms(lambda: [execute(r) for r in b100], 1 if quick else 5),
        "batch_1000": median_ms(lambda: [execute(r) for r in b1000], 1 if quick else 3),
    }


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bin", default=shutil.which("eshana") or str(ROOT / "target" / "release" / "eshana"))
    ap.add_argument("--update", action="store_true", help="record this machine's baseline")
    ap.add_argument("--threshold", type=float, default=float(os.environ.get("PERF_THRESHOLD", "1.5")))
    ap.add_argument("--quick", action="store_true", help="fewer runs (a smoke check)")
    ap.add_argument("--json", help="also write the measurement here")
    a = ap.parse_args(argv)

    got = measure(a.bin, a.quick)
    fp = fingerprint()
    print(f"machine: {fp}")
    problems = []
    base = json.loads(BASELINE.read_text()) if BASELINE.exists() else {}
    mine = base.get("machines", {}).get(fp)
    for k, v in got.items():
        line = f"  {k:<11} {v:>10.2f} ms   budget {BUDGET[k]:>8.0f}"
        if v > BUDGET[k]:
            problems.append(f"{k}: {v} ms is over the budget of {BUDGET[k]} ms")
        if mine and k in mine:
            ratio = v / mine[k]
            line += f"   baseline {mine[k]:>9.2f} ({ratio:.2f}x)"
            if ratio > a.threshold and not a.update:
                problems.append(f"{k}: {v} ms is {ratio:.2f}x this machine's baseline {mine[k]} ms (threshold {a.threshold}x)")
        print(line)
    if a.json:
        Path(a.json).write_text(json.dumps({"machine": fp, "ms": got}, indent=2) + "\n")
    if a.update:
        base.setdefault("machines", {})[fp] = got
        base["note"] = "Medians in ms from scripts/bench.py; one entry per machine fingerprint. Refresh with --update."
        BASELINE.parent.mkdir(parents=True, exist_ok=True)
        BASELINE.write_text(json.dumps(base, indent=2, sort_keys=True) + "\n")
        print(f"baseline recorded for {fp}")
    elif not mine:
        print("no baseline for this machine: budgets checked only (record one with --update)")
    for p in problems:
        print("REGRESSION", p, file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
