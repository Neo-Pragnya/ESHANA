"""Optional bodies (AST-07) against JPL Horizons: Ceres, Pallas, Juno and Vesta from the DE441 small-body kernel
(sb441-n16, SPK type 2) and Chiron from a Horizons-generated kernel (SPK type 21), each beside DE440.

Skipped unless the kernels are in the kernel folder: `python scripts/fetch_kernel.py de440`,
`python scripts/fetch_kernel.py sb441-n16` and `python scripts/fetch_kernel.py horizons:2060`.
"""
from __future__ import annotations

import os
import pathlib

import pytest

import eshana
from lib import load

KDIR = pathlib.Path(os.environ.get("ESHANA_KERNEL_DIR") or pathlib.Path.home() / ".cache" / "eshana" / "kernels")
NEED = [KDIR / "de440.bsp", KDIR / "sb441-n16.bsp", KDIR / "horizons-2060.bsp"]
pytestmark = pytest.mark.skipif(not all(p.exists() for p in NEED), reason="DE440, sb441-n16 and a Horizons Chiron kernel are needed")
ROWS = load("horizons_optional_bodies.json")["rows"]


@pytest.mark.parametrize("row", ROWS, ids=[f"{r['body']}-{r['utc'][:10]}" for r in ROWS])
def test_matches_horizons(row):
    kernels = ",".join(str(p) for p in NEED)
    env = eshana.execute({"op": "positions", "when": {"utc": row["utc"]}, "bodies": [row["body"]], "config": {"provider": "spk"},
                          "spk_path": kernels})
    p = env["positions"][0]
    dl = abs((p["longitude_deg"] - row["longitude_deg"] + 540) % 360 - 180) * 3600
    db = abs(p["latitude_deg"] - row["latitude_deg"]) * 3600
    # 1″: the kernels agree with Horizons' own integration; the rest is the apparent-place pipeline (4-term nutation)
    assert dl < 1.0 and db < 1.0, f"{row['body']} {row['utc']}: Δλ {dl:.2f}″, Δβ {db:.2f}″"


def test_without_the_kernel_the_body_is_refused_with_a_hint():
    with pytest.raises(eshana.EshanaError) as e:
        eshana.execute({"op": "positions", "when": {"utc": "2026-11-24T12:00:00Z"}, "bodies": ["chiron"], "config": {"provider": "spk"},
                        "spk_path": str(KDIR / "de440.bsp")})
    assert "Horizons" in str(e.value)
