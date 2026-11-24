"""Historical dates (TIME-03, PROV-06): Julian and Gregorian calendars, the 1582 cut-over, ΔT before 1800 and the
DE440 kernel, against an independent oracle (accuracy/reference/historical_swiss.json, generated with the test-only
Swiss Ephemeris; it is never shipped).

Skipped unless DE440 is available: `python scripts/fetch_kernel.py de440` (or ESHANA_DE440=/path/to/de440.bsp).
"""
from __future__ import annotations

import os
import pathlib

import pytest

import eshana
from lib import load

KDIR = pathlib.Path(os.environ.get("ESHANA_KERNEL_DIR") or pathlib.Path.home() / ".cache" / "eshana" / "kernels")
DE440 = os.environ.get("ESHANA_DE440") or str(KDIR / "de440.bsp")
pytestmark = pytest.mark.skipif(not pathlib.Path(DE440).exists(), reason="DE440 kernel not available (scripts/fetch_kernel.py de440)")
CASES = load("historical_swiss.json")["cases"]


def run(case):
    return eshana.execute({"op": "positions", "spk_path": DE440, "config": {"provider": "spk"},
                           "when": {"date": case["date"], "time": case["time"], "utc_offset_minutes": case["utc_offset_minutes"],
                                    "calendar": case["calendar"]}})


@pytest.mark.parametrize("case", CASES, ids=[c["id"] for c in CASES])
def test_calendar_gives_the_oracle_julian_day(case):
    env = run(case)
    assert abs(env["input"]["jd_ut"] - case["jd_ut"]) < 1e-6, (env["input"]["instant_utc"], case)


@pytest.mark.parametrize("case", CASES, ids=[c["id"] for c in CASES])
def test_positions_agree_with_the_oracle(case):
    env = run(case)
    got = {p["id"]: p["longitude_deg"] for p in env["positions"]}
    for body, want in case["tropical_longitude_deg"].items():
        diff = abs((got[body] - want + 540) % 360 - 180) * 3600
        # 3″: the oracle's analytic series (about 1″ in the 16th century) plus ESHANA's 4-term nutation;
        # the Moon also carries the ΔT model difference (oracle vs Espenak–Meeus): ~0.55″ per second of ΔT
        tol = 3.0 + (0.6 * abs(env["input"]["delta_t_s"] - case["delta_t_s_oracle"]) if body == "moon" else 0.0)
        assert diff <= tol, f"{case['id']} {body}: {diff:.1f}″ (tolerance {tol:.1f}″)"


def test_the_skipped_days_of_1582_do_not_exist():
    with pytest.raises(eshana.EshanaError) as e:
        run({"date": "1582-10-10", "time": "12:00", "utc_offset_minutes": 0, "calendar": "auto"})
    assert "1582-10-15" in str(e.value)
