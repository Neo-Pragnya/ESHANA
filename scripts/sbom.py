#!/usr/bin/env python3
"""A CycloneDX 1.5 SBOM for the eshana wheel (SEC-03): every Rust crate compiled into the extension module and the
CLI (normal dependencies only, resolved by `cargo metadata` for the locked graph), with versions, licences and the
crates.io purl, plus the SHA-256 of each file in dist/ when given. The wheel has no Python runtime dependencies.

    python scripts/sbom.py --out dist/eshana-0.5.0.cdx.json --dist dist
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SHIPPED = ("eshana-python", "eshana-cli")


def metadata() -> dict:
    out = subprocess.run(["cargo", "metadata", "--format-version", "1", "--locked"], cwd=ROOT, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def shipped_packages(meta: dict) -> list[dict]:
    """The packages reachable from the shipped crates through normal (not dev, not build) dependency edges."""
    pkgs = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    roots = [p["id"] for p in meta["packages"] if p["name"] in SHIPPED]
    seen, todo = set(), list(roots)
    while todo:
        pid = todo.pop()
        if pid in seen:
            continue
        seen.add(pid)
        for d in nodes[pid]["deps"]:
            if any(k.get("kind") is None for k in d["dep_kinds"]):
                todo.append(d["pkg"])
    return sorted((pkgs[i] for i in seen), key=lambda p: (p["name"], p["version"]))


def component(p: dict) -> dict:
    local = p.get("source") is None
    c = {"type": "library", "bom-ref": f"{p['name']}@{p['version']}", "name": p["name"], "version": p["version"]}
    if p.get("license"):
        c["licenses"] = [{"expression": p["license"]}]
    if not local:
        c["purl"] = f"pkg:cargo/{p['name']}@{p['version']}"
    if p.get("repository"):
        c["externalReferences"] = [{"type": "vcs", "url": p["repository"]}]
    return c


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", required=True)
    ap.add_argument("--dist", help="hash the wheels and sdist in this directory into the SBOM")
    a = ap.parse_args(argv)
    meta = metadata()
    comps = shipped_packages(meta)
    version = next(p["version"] for p in meta["packages"] if p["name"] == "eshana-python")
    files = []
    if a.dist:
        for f in sorted(Path(a.dist).glob("eshana-*")):
            if f.suffix in (".whl", ".gz"):
                files.append({"type": "file", "bom-ref": f"file:{f.name}", "name": f.name,
                              "hashes": [{"alg": "SHA-256", "content": hashlib.sha256(f.read_bytes()).hexdigest()}]})
    bom = {
        "bomFormat": "CycloneDX", "specVersion": "1.5", "serialNumber": f"urn:uuid:{uuid.uuid4()}", "version": 1,
        "metadata": {
            "timestamp": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "tools": {"components": [{"type": "application", "name": "eshana scripts/sbom.py"}]},
            "component": {"type": "library", "bom-ref": f"eshana@{version}", "name": "eshana", "version": version,
                          "purl": f"pkg:pypi/eshana@{version}", "licenses": [{"expression": "Apache-2.0"}]},
        },
        "components": [component(p) for p in comps] + files,
        "dependencies": [{"ref": f"eshana@{version}", "dependsOn": [f"{p['name']}@{p['version']}" for p in comps]}],
    }
    Path(a.out).parent.mkdir(parents=True, exist_ok=True)
    Path(a.out).write_text(json.dumps(bom, indent=2) + "\n")
    print(f"wrote {a.out}: {len(comps)} crates, {len(files)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
