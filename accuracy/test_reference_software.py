"""A whole chart as printed by Parashara's Light (owner-supplied output), compared item by item.

Matched within stated tolerances: longitudes, Ascendant, declinations, ayanamsha, yogi/avayogi/duplicate yogi, dagdha
rashis, Indu lagna, tithi, yoga, karana, the 22nd drekkana from lagna and Moon, and the Bhava, Hora and Ghati lagnas
(with that program's conventions: centre-of-disc sunrise without refraction, Sun at sunrise). Shadbala totals are
compared but marked as a known, documented difference until the component conventions are reconciled (VED-14).
"""
from __future__ import annotations

import pytest

import lib

PUB = lib.load("published.json")
TOL = lib.tolerances()
PROVIDERS = lib.providers()
CHARTS = PUB["parashara_light_charts"]
SIGNS = {"Ar": 0, "Ta": 1, "Ge": 2, "Cn": 3, "Le": 4, "Vi": 5, "Li": 6, "Sc": 7, "Sg": 8, "Cp": 9, "Aq": 10, "Pi": 11}


def lon_of(s: str) -> float:
    sg, dms = s.split()
    d, m, sec = (int(x) for x in dms.split(":"))
    return SIGNS[sg] * 30 + d + m / 60 + sec / 3600


def angle(s: str) -> float:
    neg = s.startswith("-")
    d, m = (int(x) for x in s.lstrip("-").split(":")[:2])
    v = d + m / 60
    return -v if neg else v


def chart(c, provider, spk, **kw):
    req = {"op": "chart", "system": "vedic", "when": {"date": c["date"], "time": c["time"], "utc_offset_minutes": c["utc_offset_minutes"]},
           "place": {"latitude": c["lat"], "longitude": c["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"}, **kw}
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return env


CASES = [pytest.param(p, spk, c, id=f"{p}:{c['id']}") for p, spk in PROVIDERS for c in CHARTS]


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_longitudes_and_declinations(provider, spk, c):
    env = chart(c, provider, spk)
    pos = lib.positions(env)
    for body, printed in c["printed"]["longitudes"].items():
        got = env["houses"]["ascendant"]["longitude_deg"] if body == "lagna" else pos[body]["longitude_deg"]
        # printed to the second; the place is printed to the arc-minute, which moves the lagna by up to ~1'
        tol = 120 if body == "lagna" else TOL["body_arcsec"][provider].get(body, 60) + 30
        assert abs(lib.adiff(got, lon_of(printed))) * 3600 <= tol, f"{body}: {lib.adiff(got, lon_of(printed)) * 3600:+.0f}\""
    for body, printed in c["printed"]["declinations"].items():
        tol = 1 / 60 + TOL["body_arcsec"][provider][body] / 3600  # printed to the arc-minute
        assert abs(pos[body]["declination_deg"] - angle(printed)) <= tol + 1 / 120, body


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_ayanamsha(provider, spk, c):
    sun = lib.positions(chart(c, provider, spk))["sun"]
    eff = (sun["tropical_longitude_deg"] - sun["longitude_deg"]) % 360
    d, m, s = (int(x) for x in c["printed"]["ayanamsha"].split(":"))
    assert abs(eff - (d + m / 60 + s / 3600)) * 3600 <= 30


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_special_points(provider, spk, c):
    env = chart(c, provider, spk, features=["special_points"])
    sp = next(a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.special_points")
    pr = c["printed"]
    assert abs(lib.adiff(sp["yogi"]["point"], lon_of(pr["yogi_point"]))) * 60 <= 1.0
    assert (sp["yogi"]["yogi_graha"], sp["avayogi"]["avayogi_graha"], sp["yogi"]["duplicate_yogi"]) == (pr["yogi"], pr["avayogi"], pr["duplicate_yogi"])
    assert sp["dagdha_rashis"]["signs"] == pr["dagdha_rashis"]
    assert sp["indu_lagna"]["sign"] == pr["indu_lagna"]
    assert sp["drekkana_22nd_from_lagna"]["sign"] == pr["drekkana_22nd"]["lagna"]
    assert sp["drekkana_22nd_from_moon"]["sign"] == pr["drekkana_22nd"]["moon"]


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_upagrahas(provider, spk, c):
    if "upagraha_signs" not in c["printed"]:
        pytest.skip("no printed upagrahas")
    env = chart(c, provider, spk, features=["upagrahas"], sunrise="centre_unrefracted")
    up = next(a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.upagrahas")
    got = {k: up[k]["sign"] for k in c["printed"]["upagraha_signs"]}
    assert got == c["printed"]["upagraha_signs"]


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_panchanga_and_special_lagnas(provider, spk, c):
    conv = c["printed"]["special_lagna_conventions"]
    req = {"op": "panchanga", "chart": {"when": {"date": c["date"], "time": c["time"], "utc_offset_minutes": c["utc_offset_minutes"]},
                                        "place": {"latitude": c["lat"], "longitude": c["lon"]},
                                        "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri"},
                                        "sunrise": conv["sunrise"], "special_lagna_sun": conv["special_lagna_sun"]}}
    if spk:
        req["chart"]["spk_path"] = spk
    p = lib.run(req)["panchanga"]
    assert p["tithi"]["paksha"] == "krishna" and p["tithi"]["name"] == "Dwitiya"
    assert p["yoga"]["name"] == c["printed"]["yoga"] and p["karana"]["name"] == c["printed"]["karana"]
    for key, printed in c["printed"]["special_lagnas"].items():
        d = abs(lib.adiff(p["special_lagnas"][key]["longitude_deg"], lon_of(printed))) * 60
        assert d <= 1.5, f"{key}: {d:.2f}′"


@pytest.mark.parametrize("provider,spk,c", CASES)
def test_shadbala_required_strengths_match(provider, spk, c):
    """The printed ratios imply each graha's required strength; ESHANA's table must agree with them."""
    env = chart(c, provider, spk, features=["shadbala"])
    sb = next(a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.shadbala")["grahas"]
    for g, virupa in c["printed"]["shadbala_virupa"].items():
        implied = virupa / 60 / c["printed"]["shadbala_ratio"][g]
        assert abs(implied - sb[g]["required_rupas"]) <= 0.08, f"{g}: printed implies {implied:.2f} rupas"


@pytest.mark.xfail(strict=False, reason="shadbala component conventions not yet reconciled with this program (VED-14); the gap is reported in birth-chart-accuracy.md")
@pytest.mark.parametrize("provider,spk,c", CASES)
def test_shadbala_totals_known_difference(provider, spk, c):
    env = chart(c, provider, spk, features=["shadbala"])
    sb = next(a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.shadbala")["grahas"]
    gaps = {g: sb[g]["total_virupa"] - v for g, v in c["printed"]["shadbala_virupa"].items()}
    assert all(abs(x) <= 15 for x in gaps.values()), gaps
