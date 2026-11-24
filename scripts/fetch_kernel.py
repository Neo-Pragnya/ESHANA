#!/usr/bin/env python3
"""Download a JPL planetary kernel from NAIF and check it against the MD5 NAIF publishes (PROV-06).

The engine and the Python package never download anything; this script lives in the repository for people who want
a checked download. The catalogue (names, sizes, checksums, spans) is the engine's own: `eshana kernel list`.

    python scripts/fetch_kernel.py de440            # into the kernel folder (eshana kernel path de440)
    python scripts/fetch_kernel.py de441-part1 --dir ~/kernels
    python scripts/fetch_kernel.py --list

An interrupted download resumes (HTTP Range). The file is written as NAME.part and renamed only after its size and
MD5 match, so a kernel in the folder is always a verified one. Then use it with `--provider spk --spk de440` (a
catalogue name is looked up in the kernel folder) or `ESHANA_SPK=/path/to/de440.bsp`.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def catalogue() -> dict:
    exe = os.environ.get("ESHANA_BIN") or shutil.which("eshana") or str(ROOT / "target" / "release" / "eshana")
    out = subprocess.run([exe, "kernel", "list", "--compact"], check=True, capture_output=True, text=True).stdout
    return json.loads(out)


def md5_of(path: Path) -> str:
    h = hashlib.md5(usedforsecurity=False)
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("name", nargs="?", help="catalogue name, e.g. de440")
    ap.add_argument("--dir", help="where to put it (default: the kernel folder)")
    ap.add_argument("--list", action="store_true")
    a = ap.parse_args()
    cat = catalogue()
    if a.list or not a.name:
        for k in cat["kernels"]:
            print(f"{k['name']:<12} {k['bytes'] / 1e6:8.1f} MB  {k['start']} … {k['end']}  {'✓ present' if k['present'] else ''}  {k['note']}")
        print(f"kernel folder: {cat['kernel_dir']}")
        return 0
    k = next((k for k in cat["kernels"] if a.name in (k["name"], k["file"])), None)
    if not k:
        print(f"unknown kernel {a.name}; one of: {', '.join(x['name'] for x in cat['kernels'])}")
        return 2
    folder = Path(a.dir).expanduser() if a.dir else Path(cat["kernel_dir"])
    folder.mkdir(parents=True, exist_ok=True)
    final, part = folder / k["file"], folder / (k["file"] + ".part")
    if final.exists() and final.stat().st_size == k["bytes"] and md5_of(final) == k["md5"]:
        print(f"{final} is already here and verified")
        return 0
    have = part.stat().st_size if part.exists() else 0
    req = urllib.request.Request(k["url"], headers={"User-Agent": "eshana-fetch-kernel", **({"Range": f"bytes={have}-"} if have else {})})
    with urllib.request.urlopen(req, timeout=60) as r, part.open("ab" if have and r.status == 206 else "wb") as out:
        done = have if r.status == 206 else 0
        while chunk := r.read(1 << 20):
            out.write(chunk)
            done += len(chunk)
            print(f"\r{k['file']}: {done / 1e6:8.1f} / {k['bytes'] / 1e6:.1f} MB", end="", flush=True)
    print()
    size, digest = part.stat().st_size, md5_of(part)
    if size != k["bytes"] or digest != k["md5"]:
        print(f"✗ {part}: {size} bytes, md5 {digest}; NAIF publishes {k['bytes']} bytes, md5 {k['md5']}. Kept as .part; delete it and retry.")
        return 1
    part.rename(final)
    print(f"✓ {final} verified (md5 {digest}). Use it with: --provider spk --spk {k['name']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
