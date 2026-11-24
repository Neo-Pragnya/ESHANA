#!/usr/bin/env python3
"""Regenerate benchmarks/golden/kp_horary_oracle.json: horary cusp geometry from the Swiss Ephemeris.

pyswisseph (AGPL) is an offline oracle only: this script runs outside the build and its output is numbers.

    uv venv refenv && uv pip install --python refenv/bin/python pyswisseph
    refenv/bin/python benchmarks/make_kp_oracle.py

For each case, a tropical Ascendant target, a latitude and a moment: the oracle finds the ARMC (local sidereal time)
at which swe.houses_armc puts that degree on the Ascendant, by its own bisection, and records the twelve Placidus
cusps swe.houses_armc returns there, with the true obliquity of date from swe (ECL_NUT). ESHANA's solver and house
code are compared against these numbers in crates/eshana-engine/tests/kp_oracle.rs.
"""

from __future__ import annotations

import json
from pathlib import Path

import swisseph as swe

OUT = Path(__file__).resolve().parent / "golden" / "kp_horary_oracle.json"

CASES = [  # (tropical Ascendant target, latitude, UTC y m d h)
    (168.1, 13.08, (2026, 11, 23, 10.0)),
    (24.0, 28.61, (2024, 3, 1, 4.5)),
    (300.5, 51.5, (2025, 7, 15, 20.0)),
    (95.25, -33.87, (2023, 1, 10, 2.0)),
    (210.0, 60.0, (2026, 6, 21, 12.0)),
    (0.5, 0.0, (2000, 1, 1, 12.0)),
]


def asc_at(armc: float, lat: float, eps: float) -> float:
    cusps, ascmc = swe.houses_armc(armc, lat, eps, b"P")
    return ascmc[0]


def sdiff(a: float, b: float) -> float:
    return (a - b + 180.0) % 360.0 - 180.0


def solve(target: float, lat: float, eps: float) -> float:
    prev = (0.0, sdiff(asc_at(0.0, lat, eps), target))
    for i in range(1, 361):
        x = float(i)
        fx = sdiff(asc_at(x, lat, eps), target)
        if prev[1] <= 0.0 < fx and abs(fx - prev[1]) < 90.0:
            lo, hi = prev[0], x
            for _ in range(80):
                mid = 0.5 * (lo + hi)
                if sdiff(asc_at(mid, lat, eps), target) <= 0.0:
                    lo = mid
                else:
                    hi = mid
            return 0.5 * (lo + hi)
        prev = (x, fx)
    raise RuntimeError("no root")


def main() -> None:
    rows = []
    for target, lat, (y, mo, d, h) in CASES:
        jd_ut = swe.julday(y, mo, d, h)
        jd_tt = jd_ut + swe.deltat(jd_ut)
        nut, _ = swe.calc(jd_tt, swe.ECL_NUT, 0)
        eps = nut[0]  # true obliquity
        armc = solve(target, lat, eps)
        cusps, ascmc = swe.houses_armc(armc, lat, eps, b"P")
        rows.append({"asc_target_deg": target, "latitude_deg": lat, "jd_tt": jd_tt,
                     "t_centuries_tt": (jd_tt - 2451545.0) / 36525.0, "true_obliquity_deg": eps,
                     "armc_deg": armc, "cusps_deg": list(cusps[:12])})
    OUT.write_text(json.dumps({"source": f"pyswisseph {swe.version} houses_armc (Placidus)", "cases": rows},
                              indent=1) + "\n")
    print(f"wrote {OUT} ({len(rows)} cases)")


if __name__ == "__main__":
    main()
