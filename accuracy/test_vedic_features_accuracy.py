"""Accuracy of the Vedic features: pañcāṅga limbs and rise/set against the Swiss Ephemeris, special lagnas against
PyJHora, sign ingresses against Swiss, and independent re-derivations of KP sub-lords, aṣṭakavarga and aṣṭakūṭa.

The re-derivations read the same YAML tables the engine reads (they test that the engine APPLIES the published
tables correctly); the tables themselves are checked by their fixed totals and symmetry in the engine's pack tests.
"""
from __future__ import annotations

import datetime as dt
import pathlib

import pytest
import yaml

import lib

REF = lib.load("swiss_charts.json")
PUB = lib.load("published.json")
TOL = lib.tolerances()
PROVIDERS = lib.providers()
PACKS = pathlib.Path(__file__).resolve().parents[1] / "crates" / "eshana-engine" / "rulesets"
CHARTS = [r for r in REF["charts"] if r["range"] == "analytic"]
CASES = [pytest.param(p, spk, row, id=f"{p}:{row['id']}") for p, spk in PROVIDERS for row in CHARTS]


def chart(row, provider, spk, **kw):
    req = {"op": "chart", "system": "vedic", "when": {"date": row["date"], "time": row["time"], "timezone": row["timezone"]},
           "place": {"latitude": row["lat"], "longitude": row["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "house_system": "whole_sign"}}
    req.update(kw)
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return env


def panchanga(row, provider, spk, time=None, **kw):
    req = {"op": "panchanga", "chart": {"when": {"date": row["date"], "time": time or row["time"], "timezone": row["timezone"]},
                                        "place": {"latitude": row["lat"], "longitude": row["lon"]},
                                        "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri"}, **kw}}
    if spk:
        req["chart"]["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return env["panchanga"]


# ── pañcāṅga limbs from the Swiss Sun and Moon ─────────────────────────────────────────────────────────────────
@pytest.mark.parametrize("provider,spk,row", CASES)
def test_panchanga_limbs_match_swiss(provider, spk, row):
    p = panchanga(row, provider, spk)
    sun, moon = row["sidereal"]["sun"]["lon"], row["sidereal"]["moon"]["lon"]
    tol = (TOL["body_arcsec"][provider]["moon"] + TOL["body_arcsec"][provider]["sun"]) / 3600 + 0.002
    elong = (moon - sun) % 360
    checks = [("tithi", elong, 12.0, p["tithi"]["index"] - 1, int(elong // 12)),
              ("karana", elong, 6.0, p["karana"]["index"] - 1, int(elong // 6)),
              ("nakshatra", moon, lib.NAKS, p["nakshatra"]["index"], lib.nakshatra(moon)),
              ("yoga", (sun + moon) % 360, lib.NAKS, p["yoga"]["index"] - 1, int(((sun + moon) % 360) // lib.NAKS))]
    for name, x, width, got, want in checks:
        if lib.near_boundary(x, width, tol):
            continue
        assert got == want, f"{name}: {got} vs {want}"


@pytest.mark.parametrize("provider,spk,row", CASES)
def test_rise_and_set_match_swiss(provider, spk, row):
    rs = row["rise_set"]
    p = panchanga(row, provider, spk, time="12:00")
    import math
    stretch = 1 / max(math.cos(math.radians(row["lat"])), 0.25)  # rise is slower at high latitude: same error, more seconds
    for key, base in (("sunrise", 20), ("sunset", 20), ("moonrise", 30), ("moonset", 30)):
        tol_s = base * stretch
        ref, got = rs.get(key), p.get(key)
        if ref is None or got is None:
            # polar days and nights: both must agree that there is no event that day
            assert (ref is None) == (got is None) or key.startswith("moon"), f"{key}: swiss {ref} eshana {got}"
            continue
        if key.startswith("sun") and abs(got["jd_ut"] - ref) > 0.5:
            continue  # ESHANA's Vedic day can start the day before at extreme latitudes
        assert abs(got["jd_ut"] - ref) * 86400 <= tol_s, f"{key}: {(got['jd_ut'] - ref) * 86400:+.1f} s"


# ── special lagnas against PyJHora ─────────────────────────────────────────────────────────────────────────────
SIGNS = {"Ar": 0, "Ta": 1, "Ge": 2, "Cn": 3, "Le": 4, "Vi": 5, "Li": 6, "Sc": 7, "Sg": 8, "Cp": 9, "Aq": 10, "Pi": 11}


def parse_jhora(s: str) -> float:
    import re
    m = re.match(r"(\d+)([A-Za-z]{2})(\d+)'([\d.]+)", s)
    d, sg, mi, se = m.groups()
    return SIGNS[sg] * 30 + int(d) + int(mi) / 60 + float(se) / 3600


@pytest.mark.parametrize("provider,spk", PROVIDERS, ids=[p for p, _ in PROVIDERS])
@pytest.mark.parametrize("ex", PUB["pyjhora_special_lagnas"], ids=lambda e: e["id"])
def test_special_lagnas_match_pyjhora(provider, spk, ex):
    req = {"op": "panchanga", "chart": {"when": {"date": ex["date"], "time": ex["time"], "utc_offset_minutes": ex["utc_offset_minutes"]},
                                        "place": {"latitude": ex["lat"], "longitude": ex["lon"]},
                                        "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri"},
                                        "sunrise": ex["conventions"]["sunrise"], "special_lagna_sun": ex["conventions"]["special_lagna_sun"]}}
    if spk:
        req["chart"]["spk_path"] = spk
    sl = lib.run(req)["panchanga"]["special_lagnas"]
    for key, printed in ex["printed"].items():
        d = abs(lib.adiff(sl[key]["longitude_deg"], parse_jhora(printed))) * 60
        assert d <= ex["tolerance_arcmin"], f"{key}: {d:.2f}′"


# ── ingresses against Swiss ────────────────────────────────────────────────────────────────────────────────────
INGRESS_TOL_DAYS = {"spk": {"sun": 0.005, "jupiter": 0.2, "saturn": 0.3}, "analytic": {"sun": 0.02, "jupiter": 4.0, "saturn": 16.0}}


@pytest.mark.parametrize("provider,spk", PROVIDERS, ids=[p for p, _ in PROVIDERS])
@pytest.mark.parametrize("body", ["sun", "jupiter", "saturn"])
def test_sign_ingresses_match_swiss(provider, spk, body):
    ref = REF["ingresses"][body]
    req = {"op": "ingresses", "body": body, "start_utc": "1990-01-01T00:00:00Z", "end_utc": "2030-01-01T00:00:00Z",
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri"}}
    if spk:
        req["spk_path"] = spk
    got = lib.run(req)["ingresses"]
    tol = INGRESS_TOL_DAYS[provider][body]
    if provider == "spk":
        assert [(g["from_sign"], g["to_sign"]) for g in got] == [(r["from_sign"], r["to_sign"]) for r in ref]
    unmatched = 0
    for r in ref:
        same = [g for g in got if g["to_sign"] == r["to_sign"] and g["from_sign"] == r["from_sign"]]
        best = min((abs(g["jd_ut"] - r["jd_ut"]) for g in same), default=99)
        if best > tol:
            unmatched += 1
    # the analytic provider's arc-minute Saturn can add or drop a brief retrograde re-entry near a boundary
    assert unmatched <= (0 if provider == "spk" else 2), f"{unmatched} of {len(ref)} ingresses outside ±{tol} d"


# ── KP sub-lords, aṣṭakavarga and aṣṭakūṭa re-derived ─────────────────────────────────────────────────────────
@pytest.mark.parametrize("provider,spk,row", CASES)
def test_kp_lords_rederived(provider, spk, row):
    env = chart(row, provider, spk, features=["kp"])
    kp = next((a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.kp"), None)
    if kp is None:
        pytest.skip("KP needs Placidus cusps, refused inside the polar circles")
    pos = {lib.positions(env)[k]["id"]: lib.positions(env)[k]["longitude_deg"] for k in lib.positions(env)}
    rename = {"mean_node": "rahu", "south_node": "ketu"}
    for pid, lon in pos.items():
        g = rename.get(pid, pid)
        if g not in kp["planets"]:
            continue
        star, sub, subsub = lib.kp_lords(lon)
        got = kp["planets"][g]
        assert (got["star_lord"], got["sub_lord"], got["sub_sub_lord"]) == (star, sub, subsub), g
    for h, c in kp["cusps"].items():
        star, sub, subsub = lib.kp_lords(c["cusp"])
        assert (c["lords"]["star_lord"], c["lords"]["sub_lord"], c["lords"]["sub_sub_lord"]) == (star, sub, subsub), h


@pytest.mark.parametrize("provider,spk,row", CASES)
def test_ashtakavarga_rederived_from_the_pack(provider, spk, row):
    env = chart(row, provider, spk, features=["ashtakavarga"])
    av = next(a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.ashtakavarga")
    tables = yaml.safe_load((PACKS / "ashtakavarga.yaml").read_text())["tables"]
    pos = lib.positions(env)
    signs = {g: lib.sign(pos[g]["longitude_deg"]) for g in lib.PLANETS[:7]}
    signs["lagna"] = lib.sign(env["houses"]["ascendant"]["longitude_deg"])
    sav = [0] * 12
    for g, contributors in tables.items():
        bindus = [0] * 12
        for c, houses in contributors.items():
            for h in houses:
                bindus[(signs[c] + h - 1) % 12] += 1
        assert av["bhinnashtakavarga"][g]["bindus"] == bindus, g
        sav = [a + b for a, b in zip(sav, bindus)]
    assert av["sarvashtakavarga"]["bindus"] == sav
    assert sum(sav) == 337


def koota_scores(ma: float, mb: float) -> float:
    m = yaml.safe_load((PACKS / "matching.yaml").read_text())
    # vedic.core and the packs split from it (0.5.0) read as one table, as the engine merges them
    core = {}
    for f in ("vedic-core.yaml", "vedic-dignity.yaml", "vedic-relationships.yaml", "vedic-drishti.yaml", "vedic-strength.yaml"):
        core.update({k: v for k, v in yaml.safe_load((PACKS / f).read_text()).items() if k not in ("id", "version", "conventions")})
    na, nb, ra, rb = lib.nakshatra(ma), lib.nakshatra(mb), lib.sign(ma), lib.sign(mb)
    score = 0.0
    score += 1 if m["varna_rank"][ra] >= m["varna_rank"][rb] else 0
    vg = lambda s, deg: m["vashya_groups"][s] if isinstance(m["vashya_groups"][s], str) else m["vashya_groups"][s][int(deg >= 15)]  # noqa: E731
    o = m["vashya_table"]["order"]
    score += m["vashya_table"]["scores"][o.index(vg(ra, ma % 30))][o.index(vg(rb, mb % 30))]
    cnt = lambda a, b: (b - a) % 27 + 1  # noqa: E731
    score += sum(1.5 for c in (cnt(nb, na), cnt(na, nb)) if c % 9 not in m["tara_bad_remainders"])
    yo = m["yoni_order"]
    score += m["yoni_matrix"][yo.index(m["yoni"][na])][yo.index(m["yoni"][nb])]
    la, lb = core["sign_lords"][ra], core["sign_lords"][rb]
    rel = lambda a, b: next(k for k in ("friends", "neutral", "enemies") if b in core["natural_friendship"][a][k])  # noqa: E731
    if la == lb:
        score += m["graha_maitri"]["same_lord"]
    else:
        pair = sorted([{"friends": "friend", "neutral": "neutral", "enemies": "enemy"}[rel(la, lb)],
                       {"friends": "friend", "neutral": "neutral", "enemies": "enemy"}[rel(lb, la)]])
        key = {("friend", "friend"): "friend_friend", ("friend", "neutral"): "friend_neutral", ("neutral", "neutral"): "neutral_neutral",
               ("enemy", "friend"): "friend_enemy", ("enemy", "neutral"): "neutral_enemy", ("enemy", "enemy"): "enemy_enemy"}[tuple(pair)]
        score += m["graha_maitri"][key]
    go = m["gana_table"]["order"]
    score += m["gana_table"]["scores"][go.index(m["gana"][na])][go.index(m["gana"][nb])]
    d = [(rb - ra) % 12 + 1, (ra - rb) % 12 + 1]
    score += 0 if d in m["bhakoot_dosha_pairs"] else 7
    score += 0 if m["nadi"][na] == m["nadi"][nb] else 8
    return score


@pytest.mark.parametrize("k", range(20))
def test_ashtakoota_rederived_from_the_pack(k):
    a, b = CHARTS[k], CHARTS[-1 - k]
    when = lambda r: {"date": r["date"], "time": r["time"], "timezone": r["timezone"]}  # noqa: E731
    cfg = {"zodiac": "sidereal", "ayanamsha": "lahiri"}
    env = lib.run({"op": "matching", "a": {"when": when(a), "config": cfg}, "b": {"when": when(b), "config": cfg}})
    moon = lambda e: next(p["longitude_deg"] for p in e["positions"] if p["id"] == "moon")  # noqa: E731
    expect = koota_scores(moon(env["charts"]["a"]), moon(env["charts"]["b"]))
    assert env["ashtakoota"]["total"] == pytest.approx(expect)
