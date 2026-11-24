#!/usr/bin/env python3
"""Regenerate benchmarks/golden/events_oracle.json: stations and longitude crossings from the Swiss Ephemeris.

pyswisseph (AGPL) is an offline oracle only: this script runs outside the build and its output is numbers.

    uv venv refenv && uv pip install --python refenv/bin/python pyswisseph
    refenv/bin/python benchmarks/make_event_oracle.py

Stations are roots of the longitude speed Swiss Ephemeris returns with FLG_SPEED (its own analytical speed, not a
difference of ESHANA's positions), found by bisection to 0.1 s. The sidereal set uses SIDM_LAHIRI, so speed includes
the precession rate. Crossings are roots of the signed separation from a fixed point. FLG_MOSEPH needs no data files.
"""

from __future__ import annotations

import json
from pathlib import Path

import swisseph as swe

OUT = Path(__file__).resolve().parent / "golden" / "events_oracle.json"
START, END = swe.julday(2024, 1, 1, 0.0), swe.julday(2027, 1, 1, 0.0)
BODIES = {"mercury": swe.MERCURY, "venus": swe.VENUS, "mars": swe.MARS, "jupiter": swe.JUPITER, "saturn": swe.SATURN}
TOL = 0.1 / 86400


def calc(jd: float, body: int, sidereal: bool) -> tuple[float, float]:
    flags = swe.FLG_MOSEPH | swe.FLG_SPEED | (swe.FLG_SIDEREAL if sidereal else 0)
    xx, _ = swe.calc_ut(jd, body, flags)
    return xx[0], xx[3]


def bisect(f, lo: float, hi: float) -> float:
    f_lo = f(lo)
    while hi - lo > TOL:
        mid = 0.5 * (lo + hi)
        if (f(mid) > 0) == (f_lo > 0):
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def utc(jd: float) -> str:
    y, m, d, h = swe.revjul(jd)
    secs = round(h * 3600, 1)
    return f"{y:04d}-{m:02d}-{d:02d}T{int(secs // 3600):02d}:{int(secs % 3600 // 60):02d}:{secs % 60:04.1f}Z"


def stations(body: int, sidereal: bool) -> list[dict]:
    def speed(jd: float) -> float:
        return calc(jd, body, sidereal)[1]

    out, step, jd = [], 0.5, START
    prev = speed(jd)
    while jd < END:
        nxt = min(jd + step, END)
        cur = speed(nxt)
        if (prev > 0) != (cur > 0):
            root = bisect(speed, jd, nxt)
            out.append(
                {
                    "jd_ut": root,
                    "utc": utc(root),
                    "station": "direct" if cur > 0 else "retrograde",
                    "longitude_deg": calc(root, body, sidereal)[0],
                }
            )
        jd, prev = nxt, cur
    return out


def crossings(body: int, point: float, sidereal: bool) -> list[dict]:
    def sep(jd: float) -> float:
        return (calc(jd, body, sidereal)[0] - point + 180.0) % 360.0 - 180.0

    out, step, jd = [], 0.25, START
    prev = sep(jd)
    while jd < END:
        nxt = min(jd + step, END)
        cur = sep(nxt)
        if (prev > 0) != (cur > 0) and abs(prev) < 90 and abs(cur) < 90:
            root = bisect(sep, jd, nxt)
            out.append({"jd_ut": root, "utc": utc(root), "direction": "direct" if cur > prev else "retrograde"})
        jd, prev = nxt, cur
    return out


def main() -> None:
    swe.set_sid_mode(swe.SIDM_LAHIRI)
    data = {
        "tool": f"pyswisseph {swe.version} (Swiss Ephemeris, AGPL, FLG_MOSEPH) used only as an offline oracle; "
        "numbers only, nothing shipped",
        "range_utc": [utc(START), utc(END)],
        "stations": {
            frame: {name: stations(body, frame == "sidereal_lahiri") for name, body in BODIES.items()}
            for frame in ("tropical", "sidereal_lahiri")
        },
        # a point each body loops over in the range, so the retrograde triple crossing is exercised
        "crossings": [
            {"body": "jupiter", "frame": "tropical", "point_deg": 105.0},
            {"body": "saturn", "frame": "sidereal_lahiri", "point_deg": 335.0},
            {"body": "mars", "frame": "tropical", "point_deg": 125.0},
        ],
    }
    for c in data["crossings"]:
        c["events"] = crossings(BODIES[c["body"]], c["point_deg"], c["frame"] != "tropical")
    OUT.write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
