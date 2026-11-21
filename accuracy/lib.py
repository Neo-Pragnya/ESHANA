"""Shared code for the birth-chart accuracy suite.

Three independent things meet here:

* ``reference``: committed numbers from other engines and published sources (``accuracy/reference/*.json``).
* ``eshana_chart``: what ESHANA computes for the same input, through the public Python API.
* ``rederive``: a second, deliberately simple implementation of every tradition rule ESHANA applies (sign,
  nakshatra, pada, nakshatra lord, whole-sign bhava, vargas D3-D12, Vimshottari balance and dates). It shares no code
  with the Rust engine, so agreement means two implementations of the published rules agree.
"""
from __future__ import annotations

import datetime as dt
import json
import math
import os
import pathlib
from functools import lru_cache
from typing import Any, Dict, List, Optional, Tuple

import eshana

HERE = pathlib.Path(__file__).resolve().parent
REF = HERE / "reference"
NAKS = 40.0 / 3.0
PADA = 10.0 / 3.0
DASHA_ORDER = ["ketu", "venus", "sun", "moon", "mars", "rahu", "jupiter", "saturn", "mercury"]
DASHA_YEARS = {"ketu": 7, "venus": 20, "sun": 6, "moon": 10, "mars": 7, "rahu": 18, "jupiter": 16, "saturn": 19,
               "mercury": 17}
DIVISIONS = (1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60)
PLANETS = ("sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn", "uranus", "neptune", "pluto")


def load(name: str) -> Dict[str, Any]:
    return json.loads((REF / name).read_text(encoding="utf-8"))


def tolerances() -> Dict[str, Any]:
    return json.loads((HERE / "tolerances.json").read_text(encoding="utf-8"))


def providers() -> List[Tuple[str, Optional[str]]]:
    """``analytic`` always; ``spk`` when ESHANA_SPK points at a kernel."""
    out: List[Tuple[str, Optional[str]]] = [("analytic", None)]
    spk = os.environ.get("ESHANA_SPK")
    if spk and pathlib.Path(spk).is_file():
        out.append(("spk", spk))
    return out


def adiff(a: float, b: float) -> float:
    """Signed smallest difference a-b in degrees, in (-180, 180]."""
    d = (a - b) % 360.0
    return d - 360.0 if d > 180.0 else d


def near_boundary(lon: float, width: float, tol_deg: float) -> bool:
    """True when ``lon`` is within ``tol_deg`` of a multiple of ``width``: a categorical check there is undecidable."""
    r = lon % width
    return min(r, width - r) <= tol_deg


# ── ESHANA ─────────────────────────────────────────────────────────────────────────────────────────────────────────
def _when(row: Dict[str, Any], civil: bool) -> Dict[str, Any]:
    if civil:
        return {"date": row["date"], "time": row["time"], "timezone": row["timezone"]}
    return {"utc": row["utc"]}


@lru_cache(maxsize=None)
def _cached(req_json: str) -> str:
    try:
        return json.dumps(eshana.execute(json.loads(req_json)))
    except eshana.EshanaError as e:
        return json.dumps({"kind": "error", "error": {"code": e.code, "message": e.message}})


def run(req: Dict[str, Any]) -> Dict[str, Any]:
    return json.loads(_cached(json.dumps(req, sort_keys=True)))


def eshana_chart(row: Dict[str, Any], provider: str, spk: Optional[str], *, system: str = "vedic",
                 houses: str = "whole_sign", node: str = "mean", civil: bool = True) -> Dict[str, Any]:
    cfg: Dict[str, Any] = {"provider": provider, "house_system": houses, "node_model": node}
    if system == "vedic":
        cfg.update(zodiac="sidereal", ayanamsha="lahiri")
    else:
        cfg.update(zodiac="tropical")
    req: Dict[str, Any] = {"op": "chart", "system": system, "when": _when(row, civil),
                           "place": {"latitude": row["lat"], "longitude": row["lon"]}, "config": cfg}
    if system == "vedic":
        req.update(divisions=list(DIVISIONS), dasha=True)
    if spk:
        req["spk_path"] = spk
    return run(req)


def positions(env: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    return {p["id"]: p for p in env["positions"]}


def attrs(env: Dict[str, Any]) -> Dict[str, Any]:
    return {a["id"]: a["value"] for a in env["derived_attributes"]}


# ── independent re-derivation of the tradition rules ───────────────────────────────────────────────────────────────
def sign(lon: float) -> int:
    return int(math.floor((lon % 360.0) / 30.0 + 1e-12)) % 12


def nakshatra(lon: float) -> int:
    return int(math.floor((lon % 360.0) / NAKS + 1e-12)) % 27


def pada(lon: float) -> int:
    return int(math.floor((lon % 360.0) / PADA + 1e-12)) % 4 + 1


def nak_lord(idx: int) -> str:
    return DASHA_ORDER[idx % 9]


def varga(lon: float, d: int) -> int:
    """Parashari varga sign index (0 = Aries) for D1, D3, D4, D7, D9, D10, D12, written from the textbook rules."""
    s = sign(lon)
    deg = (lon % 360.0) - 30.0 * s
    odd = s % 2 == 0  # Aries (index 0) is an odd sign
    if d == 1:
        return s
    if d == 3:
        return (s + (0, 4, 8)[min(int(deg // 10), 2)]) % 12
    if d == 4:
        return (s + 3 * min(int(deg // 7.5), 3)) % 12
    if d == 7:
        part = min(int(deg // (30.0 / 7.0)), 6)
        return ((s if odd else s + 6) + part) % 12
    if d == 9:
        return int(math.floor((lon % 360.0) / PADA + 1e-12)) % 12
    if d == 10:
        part = min(int(deg // 3.0), 9)
        return ((s if odd else s + 8) + part) % 12
    if d == 12:
        return (s + min(int(deg // 2.5), 11)) % 12
    modality = s % 3  # 0 movable (cara), 1 fixed (sthira), 2 dual (dvisvabhava)
    if d == 2:  # horā: odd sign, first half Sun (Leo) and second half Moon (Cancer); even sign the reverse
        first = deg < 15.0
        return 4 if first == odd else 3
    if d == 16:  # movable from Aries, fixed from Leo, dual from Sagittarius
        return ([0, 4, 8][modality] + min(int(deg // (30 / 16)), 15)) % 12
    if d == 20:  # movable from Aries, fixed from Sagittarius, dual from Leo
        return ([0, 8, 4][modality] + min(int(deg // 1.5), 19)) % 12
    if d == 24:  # odd from Leo, even from Cancer
        return ((4 if odd else 3) + min(int(deg // 1.25), 23)) % 12
    if d == 27:  # fire from Aries, earth from Cancer, air from Libra, water from Capricorn
        return ([0, 3, 6, 9][s % 4] + min(int(deg // (30 / 27)), 26)) % 12
    if d == 30:  # unequal parts ruled by Mars, Saturn, Jupiter, Mercury, Venus
        if odd:
            for limit, target in ((5, 0), (10, 10), (18, 8), (25, 2), (30, 6)):
                if deg < limit:
                    return target
            return 6
        for limit, target in ((5, 1), (12, 5), (20, 11), (25, 9), (30, 7)):
            if deg < limit:
                return target
        return 7
    if d == 40:  # odd from Aries, even from Libra
        return ((0 if odd else 6) + min(int(deg // 0.75), 39)) % 12
    if d == 45:  # movable from Aries, fixed from Leo, dual from Sagittarius
        return ([0, 4, 8][modality] + min(int(deg // (30 / 45)), 44)) % 12
    if d == 60:  # from the sign itself, one sign per half degree
        return (s + min(int(deg * 2), 59)) % 12
    raise ValueError(d)


def vimshottari(moon_lon: float, birth_utc: str, year_days: float = 365.25) -> Dict[str, Any]:
    idx = nakshatra(moon_lon)
    lord = nak_lord(idx)
    frac = ((moon_lon % 360.0) % NAKS) / NAKS
    balance = (1.0 - frac) * DASHA_YEARS[lord]
    birth = dt.datetime.strptime(birth_utc, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)
    start = birth - dt.timedelta(days=frac * DASHA_YEARS[lord] * year_days)
    mahas = []
    k = DASHA_ORDER.index(lord)
    t = start
    for i in range(9):
        lo = DASHA_ORDER[(k + i) % 9]
        end = t + dt.timedelta(days=DASHA_YEARS[lo] * year_days)
        mahas.append({"lord": lo, "start": max(t, birth), "end": end})
        t = end
    return {"lord": lord, "balance_years": balance, "mahadashas": mahas}


def kp_lords(lon: float) -> tuple:
    """Star, sub and sub-sub lords: each nakshatra split in Vimshottari proportions from its own lord."""
    lon = lon % 360.0
    nk = nakshatra(lon)
    within = lon - nk * NAKS

    def split(start: int, offset: float, span: float) -> tuple:
        acc = 0.0
        for k in range(9):
            i = (start + k) % 9
            length = span * DASHA_YEARS[DASHA_ORDER[i]] / 120.0
            if offset < acc + length or k == 8:
                return i, offset - acc, length
            acc += length
        raise AssertionError

    star = nk % 9
    sub, off, length = split(star, within, NAKS)
    subsub, _, _ = split(sub, off, length)
    return DASHA_ORDER[star], DASHA_ORDER[sub], DASHA_ORDER[subsub]


def parse_utc(s: str) -> dt.datetime:
    s = s.replace("Z", "+00:00")
    return dt.datetime.fromisoformat(s)
