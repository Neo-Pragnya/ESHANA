"""Maitri tables, the nakshatra spatial matrix and the dignities in the sixteen divisions, compared with the
published reference chart (Parashara's Light output for Sri Ramana Maharshi, supplied by the project owner as a
screenshot, 2026-11-21; transcribed cell by cell below).

Conventions matched (named in crates/eshana-engine/rulesets/maitri.yaml): the node friendship table, the tārā count
from the column's nakshatra to the row's, node dignity signs, Mercury's split of Virgo, mūlatrikoṇa by the degree
inside the varga sign (by sign in D30) and the temporary relationship taken inside each varga.
"""
from __future__ import annotations

import pytest

import lib

PUB = lib.load("published.json")
C = next(c for c in PUB["parashara_light_charts"] if c["id"] == "parashara-light-ramana-1879")
PROVIDERS = lib.providers()
GRAHAS = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu"]
POINTS = ["lagna"] + GRAHAS
ABBR = {"GF": "great_friend", "F": "friend", "N": "neutral", "E": "enemy", "GE": "great_enemy",
        "Ex": "exalted", "Db": "debilitated", "MT": "moolatrikona", "Own": "own"}
PM = "Parama mitra"  # printed "P.mitra"

# "Compound Relationships (Panchadha)": row = the graha, columns Su Mo Ma Me Ju Ve Sa Ra Ke.
COMPOUND = {
    "sun":     ["-", "N", "N", "F", "GF", "N", "N", "GE", "GE"],
    "moon":    ["N", "-", "F", "N", "E", "E", "F", "GE", "GE"],
    "mars":    ["N", "GF", "-", "GE", "GF", "E", "F", "GE", "GF"],
    "mercury": ["GF", "GE", "E", "-", "F", "N", "E", "F", "E"],
    "jupiter": ["GF", "N", "GF", "N", "-", "N", "F", "F", "E"],
    "venus":   ["N", "GE", "E", "N", "F", "-", "N", "GF", "N"],
    "saturn":  ["N", "N", "N", "N", "F", "N", "-", "GF", "N"],
    "rahu":    ["GE", "GE", "GE", "F", "GF", "GF", "GF", "-", "GE"],
    "ketu":    ["GE", "GE", "GF", "E", "E", "N", "N", "GE", "-"],
}

# "Nakshatra Spatial Matrix": row, columns As Su Mo Ma Me Ju Ve Sa Ra Ke.
TARA = {
    "lagna":   ["-", "Kshema", "Mitra", "Kshema", "Sadhaka", PM, "Mitra", "Sadhaka", "Kshema", "Mitra"],
    "sun":     ["Vadha", "-", "Pratyari", "Janma", "Vipat", "Sadhaka", "Pratyari", "Vipat", "Janma", "Pratyari"],
    "moon":    ["Vipat", "Sadhaka", "-", "Sadhaka", "Mitra", "Sampat", "Janma", "Mitra", "Sadhaka", "Janma"],
    "mars":    ["Vadha", "Janma", "Pratyari", "-", "Vipat", "Sadhaka", "Pratyari", "Vipat", "Janma", "Pratyari"],
    "mercury": ["Pratyari", "Mitra", "Vipat", "Mitra", "-", "Kshema", "Vipat", "Janma", "Mitra", "Vipat"],
    "jupiter": ["Sampat", "Pratyari", PM, "Pratyari", "Vadha", "-", PM, "Vadha", "Pratyari", PM],
    "venus":   ["Vipat", "Sadhaka", "Janma", "Sadhaka", "Mitra", "Sampat", "-", "Mitra", "Sadhaka", "Janma"],
    "saturn":  ["Pratyari", "Mitra", "Vipat", "Mitra", "Janma", "Kshema", "Vipat", "-", "Mitra", "Vipat"],
    "rahu":    ["Vadha", "Janma", "Pratyari", "Janma", "Vipat", "Sadhaka", "Pratyari", "Vipat", "-", "Pratyari"],
    "ketu":    ["Vipat", "Sadhaka", "Janma", "Sadhaka", "Mitra", "Sampat", "Janma", "Mitra", "Sadhaka", "-"],
}

# "Dignities in the 16 divisions": division, columns Su Mo Ma Me Ju Ve Sa Ra Ke.
DIGNITY = {
    1:  ["GF", "N", "Own", "E", "F", "E", "F", "N", "N"],
    2:  ["N", "Own", "Db", "GF", "Ex", "GE", "N", "N", "N"],
    3:  ["Ex", "E", "N", "GE", "N", "F", "GE", "N", "N"],
    4:  ["F", "E", "F", "GF", "GF", "F", "GF", "Own", "Own"],
    7:  ["GF", "E", "N", "N", "GE", "Own", "E", "Ex", "Ex"],
    9:  ["Own", "N", "F", "F", "F", "N", "F", "Db", "Db"],
    10: ["N", "E", "Own", "F", "Ex", "GE", "Db", "N", "N"],
    12: ["F", "Ex", "N", "GF", "GF", "F", "GF", "Own", "Own"],
    16: ["MT", "F", "GF", "N", "GF", "GE", "GF", "N", "MT"],
    20: ["F", "E", "N", "Db", "N", "F", "GE", "Db", "Ex"],
    24: ["MT", "N", "Ex", "E", "GE", "N", "GE", "Own", "N"],
    27: ["F", "Db", "Own", "Own", "Own", "GF", "Db", "N", "N"],
    30: ["N", "E", "GE", "F", "MT", "Own", "F", "MT", "N"],
    40: ["GF", "Ex", "GE", "E", "Db", "Own", "GE", "Db", "Ex"],
    45: ["GF", "N", "N", "Own", "GF", "N", "Own", "Db", "Ex"],
    60: ["N", "F", "Own", "Ex", "N", "E", "Own", "N", "N"],
}

# Cells that differ for a stated positional reason (not a convention): Jupiter is printed at Aq 16°29'10"; the
# built-in ephemeris puts it about 3' later (the JPL kernel agrees with the reference), just past the 16°30' start of the 22nd khavedāṃśa, so its D40 sign is Aquarius (neutral)
# instead of Capricorn (debilitated).
KNOWN = {("jupiter", 40)}


def maitri(provider, spk):
    req = {"op": "chart", "system": "vedic",
           "when": {"date": C["date"], "time": C["time"], "utc_offset_minutes": C["utc_offset_minutes"]},
           "place": {"latitude": C["lat"], "longitude": C["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"},
           "features": ["maitri"]}
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return {a["id"]: a["value"] for a in env["derived_attributes"] if a["id"].startswith("vedic.maitri.")}


def dignities(provider, spk):
    v = maitri(provider, spk)["vedic.maitri.varga_dignities"]
    return {(g, c["d"]): c["dignity"] for g, cells in v["grahas"].items() for c in cells}


CASES = [pytest.param(p, spk, id=p) for p, spk in PROVIDERS]


@pytest.mark.parametrize("provider,spk", CASES)
def test_compound_relationships(provider, spk):
    rows = maitri(provider, spk)["vedic.maitri.relationships"]["rows"]
    got = {(r["from"], r["to"]): r["compound"] for r in rows}
    bad = [(a, b, ABBR[v], got[(a, b)]) for a, vals in COMPOUND.items() for b, v in zip(GRAHAS, vals)
           if v != "-" and got[(a, b)] != ABBR[v]]
    assert not bad, bad
    assert len(rows) == 72


@pytest.mark.parametrize("provider,spk", CASES)
def test_nakshatra_spatial_matrix(provider, spk):
    rows = maitri(provider, spk)["vedic.maitri.tara_matrix"]["rows"]
    got = {(r["row"], r["column"]): r["name"] for r in rows}
    bad = [(r, c, v, got[(r, c)]) for r, vals in TARA.items() for c, v in zip(POINTS, vals) if v != "-" and got[(r, c)] != v]
    assert not bad, bad
    assert len(rows) == 90


@pytest.mark.parametrize("provider,spk", CASES)
def test_dignities_in_sixteen_divisions(provider, spk):
    got = dignities(provider, spk)
    bad = [(g, d, ABBR[v], got[(g, d)]) for d, vals in DIGNITY.items() for g, v in zip(GRAHAS, vals)
           if (g, d) not in KNOWN and got[(g, d)] != ABBR[v]]
    assert not bad, bad
    assert len(got) == 16 * 9


# Until 0.5.0 the built-in ephemeris put Jupiter about 3' late, across the D40 boundary at 16°30' (printed
# Aq 16°29'10"); with VSOP87 both providers match the reference here.
KNOWN_CASES = [pytest.param(p, spk, id=p) for p, spk in PROVIDERS]


@pytest.mark.parametrize("provider,spk", KNOWN_CASES)
def test_dignity_known_difference(provider, spk):
    got = dignities(provider, spk)
    for g, d in KNOWN:
        assert got[(g, d)] == ABBR[DIGNITY[d][GRAHAS.index(g)]], (g, d)
