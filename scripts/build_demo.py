#!/usr/bin/env python3
"""Generate the website's demo data from the REAL `eshana` binary, so the site never shows invented output.

    python3 scripts/build_demo.py website/src/data           # write
    python3 scripts/build_demo.py website/src/data --check   # fail if the committed files differ from a fresh run

Demos follow the master plan: A dual-tradition chart, B numerology trace, C Human Design trace, D graph, E method comparison,
plus uncertain birth time and refusals. Everything is deterministic (no clock, no randomness).
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BIN = Path(os.environ.get("ESHANA_BIN", ROOT / "target" / "release" / "eshana"))
BIRTH = ["--date", "1992-05-28", "--time", "20:05:00", "--tz", "Asia/Kolkata", "--lat", "17.385", "--lon", "78.487"]


# A focused graph (Moon only, with D9): the full chart has hundreds of edges and is unreadable on a page.
GRAPH = ["--date", "1992-05-28", "--time", "20:05:00", "--tz", "Asia/Kolkata", "--bodies", "moon", "--divisions", "9"]


def run(*args: str, expect_ok: bool = True) -> str:
    p = subprocess.run([str(BIN), "--compact", *args], capture_output=True, text=True, check=False)
    if expect_ok and p.returncode != 0:
        raise SystemExit(f"eshana {' '.join(args)} failed:\n{p.stdout}\n{p.stderr}")
    return p.stdout


def js(*args: str, expect_ok: bool = True):
    return json.loads(run(*args, expect_ok=expect_ok))


def accuracy_tables() -> dict:
    text = (ROOT / "benchmarks" / "golden" / "accuracy.md").read_text(encoding="utf-8")
    tables: list[dict] = []
    for block in re.split(r"\n## ", text):
        rows = re.findall(r"^\| (sun|moon|mercury|venus|mars|jupiter|saturn|uranus|neptune|pluto) \| ([\d.]+) \| ([\d.]+) \| ([\d.]+) \|$", block, re.M)
        if rows:
            tables.append({r[0]: {"max_lon_arcsec": float(r[1]), "median_lon_arcsec": float(r[2]), "max_lat_arcsec": float(r[3])} for r in rows})
    assert len(tables) == 2, "expected the analytic and spk tables in accuracy.md"
    return {"analytic": tables[0], "spk": tables[1]}


def stats() -> dict:
    features = sorted((ROOT / "acceptance" / "features").glob("*.feature"))
    scenarios = sum(len(re.findall(r"^\s*Scenario:", f.read_text(encoding="utf-8"), re.M)) for f in features)
    caps = re.findall(r"^\s*- \{ id: ", (ROOT / "acceptance" / "capabilities.yaml").read_text(encoding="utf-8"), re.M)
    rust_tests = 0
    for f in (ROOT / "crates").rglob("*.rs"):
        rust_tests += len(re.findall(r"#\[test\]", f.read_text(encoding="utf-8")))
    py_tests = sum(len(re.findall(r"^def test_", f.read_text(encoding="utf-8"), re.M))
                   for f in sorted((ROOT / "crates/eshana-python/tests").glob("test_*.py")))
    acc_tests = len(re.findall(r"^def test_", (ROOT / "accuracy/test_birth_chart_accuracy.py").read_text(encoding="utf-8"), re.M))
    backlog = json.loads((ROOT / "planning" / "backlog.json").read_text(encoding="utf-8"))
    by_status: dict[str, int] = {}
    for t in backlog["tasks"]:
        by_status[t["status"]] = by_status.get(t["status"], 0) + 1
    doc_pages = len(list((ROOT / "docs").rglob("*.md")))
    adrs = len(list((ROOT / "docs" / "adr").glob("0*.md")))
    version = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M).group(1)
    info = js("doctor")
    return {
        "version": version,
        "scenarios": scenarios,
        "capabilities": len(caps),
        "rust_tests": rust_tests,
        "python_tests": py_tests,
        "accuracy_test_functions": acc_tests,
        "reference_charts": len(json.loads((ROOT / "accuracy/reference/swiss_charts.json").read_text())["charts"]),
        "doc_pages": doc_pages,
        "adrs": adrs,
        "tasks": len(backlog["tasks"]),
        "tasks_by_status": by_status,
        "rulesets": info["rulesets"],
        "error_codes": len(info["error_codes"]),
        "ayanamshas": info["ayanamshas"],
        "tzdb_version": info["tzdb_version"],
        "expansion": info["expansion"],
    }


def build() -> dict[str, object]:
    out: dict[str, object] = {}
    out["dual.json"] = {
        "vedic": js("chart", "vedic", *BIRTH, "--ayanamsha", "lahiri", "--divisions", "1,9", "--dasha"),
        "western": js("chart", "western", *BIRTH, "--zodiac", "tropical", "--houses", "placidus", "--aspects"),
    }
    out["numerology.json"] = {
        "pythagorean": js("numerology", "--name", "Ada Lovelace", "--dob", "1815-12-10", "--year", "2026"),
        "chaldean": js("numerology", "--name", "Ada Lovelace", "--school", "chaldean"),
    }
    out["human_design.json"] = js("human-design", *BIRTH)
    out["graph.json"] = {
        "json": js("graph", "vedic", *GRAPH),
        "mermaid": run("graph", "vedic", *GRAPH, "--format", "mermaid").strip().splitlines(),
    }
    out["compare.json"] = {
        "ayanamsha": js("compare", "vedic", *BIRTH, "--a", "ayanamsha=lahiri", "--b", "ayanamsha=fagan_bradley"),
        "node": js("compare", "western", "--utc", "2024-02-29T18:45:30Z", "--zodiac", "tropical", "--a", "node_model=mean", "--b", "node_model=true"),
        "zodiac": js("compare", "western", "--utc", "2000-01-01T12:00:00Z", "--a", "zodiac=tropical", "--b", "zodiac=sidereal", "--b", "ayanamsha=lahiri"),
    }
    out["uncertain.json"] = js("chart", "vedic", "--date", "1992-05-28", "--tz", "Asia/Kolkata", "--bodies", "sun,moon,mercury")
    out["refusals.json"] = [
        {"title": "A sidereal zodiac without an ayanamsha", "command": "eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal",
         "result": js("positions", "--utc", "2000-01-01T12:00:00Z", "--zodiac", "sidereal", expect_ok=False)},
        {"title": "A local time that happens twice", "command": "eshana positions --date 2024-11-03 --time 01:30 --tz America/New_York",
         "result": js("positions", "--date", "2024-11-03", "--time", "01:30", "--tz", "America/New_York", expect_ok=False)},
        {"title": "A local time that never happened", "command": "eshana positions --date 2024-03-10 --time 02:30 --tz America/New_York --fold latest",
         "result": js("positions", "--date", "2024-03-10", "--time", "02:30", "--tz", "America/New_York", "--fold", "latest", expect_ok=False)},
        {"title": "Placidus houses inside the polar circle", "command": "eshana chart western --utc 2000-01-01T12:00:00Z --lat 75 --lon 10 --houses placidus",
         "result": js("chart", "western", "--utc", "2000-01-01T12:00:00Z", "--lat", "75", "--lon", "10", "--houses", "placidus", expect_ok=False)},
        {"title": "Human Design with an unknown birth time", "command": "eshana human-design --date 1992-05-28 --tz Asia/Kolkata",
         "result": js("human-design", "--date", "1992-05-28", "--tz", "Asia/Kolkata", expect_ok=False)},
    ]
    out["accuracy.json"] = accuracy_tables()
    out["stats.json"] = stats()
    return out


def render(obj: object) -> str:
    return json.dumps(obj, indent=1, ensure_ascii=False, sort_keys=True) + "\n"


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    check = "--check" in sys.argv
    dest = Path(args[0]) if args else ROOT / "website" / "src" / "data"
    if not BIN.exists():
        raise SystemExit(f"{BIN} not found: run `cargo build --release -p eshana-cli` (or set ESHANA_BIN)")
    files = {name: render(obj) for name, obj in build().items()}
    if check:
        bad = [n for n, text in files.items() if not (dest / n).exists() or (dest / n).read_text(encoding="utf-8") != text]
        if bad:
            print("demo data drifted from the real CLI: " + ", ".join(bad) + "\nrun `make demo` and commit the result", file=sys.stderr)
            return 1
        print(f"demo data ok: {len(files)} files match a fresh run")
        return 0
    dest.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (dest / name).write_text(text, encoding="utf-8")
    print(f"wrote {len(files)} files to {dest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
