#!/usr/bin/env python3
"""Build the birth-chart accuracy reference set (accuracy/reference/swiss_charts.json).

The Swiss Ephemeris (pyswisseph, AGPL-3.0) is used here ONLY as an offline oracle: this script runs on a developer
machine, the numbers it prints are committed, and nothing from Swiss Ephemeris ships in ESHANA. The tests read the
committed JSON, so they need neither pyswisseph nor network access.

    uv run --no-project --with pyswisseph python scripts/build_reference.py

The chart set is deterministic: real cities across both hemispheres, high latitudes and many time zones, dates from
1850 to 2140 (spk range) with most inside 1900-2100 (analytic range), including dates near sign and nakshatra
boundaries found by search, so categorical checks are exercised where they are hardest.
"""
from __future__ import annotations

import datetime as dt
import json
import pathlib
import sys
from zoneinfo import ZoneInfo

import swisseph as swe

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "accuracy" / "reference" / "swiss_charts.json"

CITIES = [
    ("New Delhi", 28.6139, 77.2090, "Asia/Kolkata"),
    ("Chennai", 13.0827, 80.2707, "Asia/Kolkata"),
    ("Mumbai", 19.0760, 72.8777, "Asia/Kolkata"),
    ("Kathmandu", 27.7172, 85.3240, "Asia/Kathmandu"),
    ("Colombo", 6.9271, 79.8612, "Asia/Colombo"),
    ("London", 51.5074, -0.1278, "Europe/London"),
    ("New York", 40.7128, -74.0060, "America/New_York"),
    ("Fredericksburg VA", 38.3032, -77.4605, "America/New_York"),
    ("Los Angeles", 34.0522, -118.2437, "America/Los_Angeles"),
    ("Sydney", -33.8688, 151.2093, "Australia/Sydney"),
    ("Buenos Aires", -34.6037, -58.3816, "America/Argentina/Buenos_Aires"),
    ("Johannesburg", -26.2041, 28.0473, "Africa/Johannesburg"),
    ("Tokyo", 35.6762, 139.6503, "Asia/Tokyo"),
    ("Reykjavik", 64.1466, -21.9426, "Atlantic/Reykjavik"),
    ("Tromso", 69.6496, 18.9560, "Europe/Oslo"),
    ("Singapore", 1.3521, 103.8198, "Asia/Singapore"),
]
# local civil times (date, time) spread across seasons, decades and hours
LOCAL = [
    ("1905-03-21", "06:15"), ("1924-07-04", "23:50"), ("1931-12-25", "12:00"), ("1947-08-15", "00:05"),
    ("1955-01-30", "04:40"), ("1962-02-05", "17:30"), ("1969-07-20", "20:17"), ("1975-10-11", "09:09"),
    ("1984-02-29", "13:13"), ("1990-05-15", "14:30"), ("1996-12-07", "10:34"), ("2001-09-11", "08:46"),
    ("2008-06-21", "03:33"), ("2012-11-13", "21:12"), ("2020-03-20", "09:49"), ("2024-04-08", "18:18"),
    ("2033-08-01", "07:00"), ("2050-01-01", "00:00"), ("2077-06-30", "15:45"), ("2099-12-31", "23:59"),
]
SPK_ONLY = [("1852-06-01", "05:00"), ("1866-10-10", "16:20"), ("2125-04-04", "04:04"), ("2140-09-09", "19:19")]

BODIES = {"sun": swe.SUN, "moon": swe.MOON, "mercury": swe.MERCURY, "venus": swe.VENUS, "mars": swe.MARS,
          "jupiter": swe.JUPITER, "saturn": swe.SATURN, "uranus": swe.URANUS, "neptune": swe.NEPTUNE,
          "pluto": swe.PLUTO, "mean_node": swe.MEAN_NODE, "true_node": swe.TRUE_NODE}
AYAN = {"lahiri": swe.SIDM_LAHIRI, "raman": swe.SIDM_RAMAN, "krishnamurti": swe.SIDM_KRISHNAMURTI,
        "fagan_bradley": swe.SIDM_FAGAN_BRADLEY}
FLAGS = swe.FLG_MOSEPH | swe.FLG_SPEED


def to_utc(date: str, time: str, zone: str) -> dt.datetime:
    local = dt.datetime.fromisoformat(f"{date}T{time}").replace(tzinfo=ZoneInfo(zone))
    return local.astimezone(dt.timezone.utc)


def jd_ut(u: dt.datetime) -> float:
    return swe.julday(u.year, u.month, u.day, u.hour + u.minute / 60 + u.second / 3600, swe.GREG_CAL)


def chart(name: str, lat: float, lon: float, zone: str, date: str, time: str, range_: str) -> dict:
    u = to_utc(date, time, zone)
    jd = jd_ut(u)
    row = {"id": f"{date}T{time} {name}", "city": name, "lat": lat, "lon": lon, "timezone": zone, "date": date,
           "time": time, "utc": u.strftime("%Y-%m-%dT%H:%M:%SZ"), "jd_ut": jd, "range": range_,
           "delta_t_s": swe.deltat_ex(jd, swe.FLG_MOSEPH) * 86400.0,
           "tropical": {}, "sidereal": {}, "ayanamsha": {}, "houses": {}}
    for b, code in BODIES.items():
        xx, _ = swe.calc_ut(jd, code, FLAGS)
        row["tropical"][b] = {"lon": xx[0], "lat": xx[1], "speed": xx[3]}
    for a, mode in AYAN.items():
        swe.set_sid_mode(mode, 0, 0)
        row["ayanamsha"][a] = swe.get_ayanamsa_ex_ut(jd, swe.FLG_MOSEPH)[1]
        if a == "lahiri":
            for b, code in BODIES.items():
                xx, _ = swe.calc_ut(jd, code, FLAGS | swe.FLG_SIDEREAL)
                row["sidereal"][b] = {"lon": xx[0], "speed": xx[3]}
    polar = abs(lat) >= 66.0
    for hs in ("P", "E", "W"):
        if hs == "P" and polar:
            continue
        swe.set_sid_mode(swe.SIDM_LAHIRI, 0, 0)
        c_t, a_t = swe.houses_ex(jd, lat, lon, hs.encode(), swe.FLG_MOSEPH)
        c_s, a_s = swe.houses_ex(jd, lat, lon, hs.encode(), swe.FLG_MOSEPH | swe.FLG_SIDEREAL)
        row["houses"][hs] = {"tropical": {"cusps": list(c_t[:12]), "asc": a_t[0], "mc": a_t[1]},
                             "sidereal": {"cusps": list(c_s[:12]), "asc": a_s[0], "mc": a_s[1]}}
    # rise and set on the local civil date (upper limb, standard refraction: the almanac convention)
    local_mid = dt.datetime.fromisoformat(f"{date}T00:00").replace(tzinfo=ZoneInfo(zone)).astimezone(dt.timezone.utc)
    jd0 = jd_ut(local_mid)
    row["rise_set"] = {}
    for key, body, flag in (("sunrise", swe.SUN, swe.CALC_RISE), ("sunset", swe.SUN, swe.CALC_SET),
                            ("moonrise", swe.MOON, swe.CALC_RISE), ("moonset", swe.MOON, swe.CALC_SET)):
        try:
            res, tret = swe.rise_trans(jd0, body, flag, (lon, lat, 0.0), 1013.25, 10.0, swe.FLG_MOSEPH)
            t = tret[0]
            row["rise_set"][key] = t if res == 0 and t < jd0 + 1.0 else None
        except swe.Error:
            row["rise_set"][key] = None
    return row


def ingresses(body: int, start: str, end: str) -> list:
    """Sidereal (Lahiri) sign ingresses of a body, found by daily sampling and bisection to about a second."""
    swe.set_sid_mode(swe.SIDM_LAHIRI, 0, 0)
    a = jd_ut(dt.datetime.fromisoformat(start)); b = jd_ut(dt.datetime.fromisoformat(end))
    sign = lambda jd: int(swe.calc_ut(jd, body, swe.FLG_MOSEPH | swe.FLG_SIDEREAL)[0][0] // 30)  # noqa: E731
    out, t, s = [], a, sign(a)
    while t < b:
        u = min(t + 1.0, b); su = sign(u)
        if su != s:
            lo, hi = t, u
            for _ in range(40):
                m = (lo + hi) / 2
                if sign(m) == s:
                    lo = m
                else:
                    hi = m
            out.append({"jd_ut": hi, "from_sign": s, "to_sign": su})
        t, s = u, su
    return out


def main() -> int:
    rows = []
    for i, (date, time) in enumerate(LOCAL):
        # two cities per date, rotating, so every city appears and every date is used twice
        for k in (0, 7):
            name, lat, lon, zone = CITIES[(i + k) % len(CITIES)]
            rows.append(chart(name, lat, lon, zone, date, time, "analytic"))
    for i, (date, time) in enumerate(SPK_ONLY):
        name, lat, lon, zone = CITIES[(i * 3) % len(CITIES)]
        rows.append(chart(name, lat, lon, zone, date, time, "spk"))
    # the Lahiri definition epoch: 1956-03-21 00:00 TT (Indian Calendar Reform Committee: 23°15'00" true ayanamsha)
    swe.set_sid_mode(swe.SIDM_LAHIRI, 0, 0)
    doc = {
        "tool": f"pyswisseph {swe.version} (Swiss Ephemeris, AGPL-3.0), Moshier ephemeris (FLG_MOSEPH); used only as "
                "an offline oracle, numbers only, nothing shipped",
        "frame": "geocentric apparent, true equinox of date; sidereal = Swiss convention (mean ayanamsha, nutation "
                 "removed)",
        "generated": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d"),
        "charts": rows,
        "ingresses": {
            "range": ["1990-01-01T00:00:00", "2030-01-01T00:00:00"],
            "sun": ingresses(swe.SUN, "1990-01-01T00:00:00", "2030-01-01T00:00:00"),
            "jupiter": ingresses(swe.JUPITER, "1990-01-01T00:00:00", "2030-01-01T00:00:00"),
            "saturn": ingresses(swe.SATURN, "1990-01-01T00:00:00", "2030-01-01T00:00:00"),
        },
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(doc, indent=1) + "\n")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} charts", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
