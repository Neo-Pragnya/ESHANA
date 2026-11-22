"""Sarvatobhadra, Kota and Sanghatta chakras for the Ramana chart, compared with a published reference chart.

The reference values were transcribed from the owner-supplied screenshot of the Auxiliary Analysis page (the same chart
as accuracy/reference/published.json, id parashara-light-ramana-1879), read with on-device text recognition: the
labels and their positions give the layout of each chakra and the cell or ring sector of every graha marker. The
reference prints transit markers at its own "now", Sat 21 Nov 2026 07:57:29 (taken here as US Eastern time,
12:57:29 UTC); the chakra layout does not depend on that instant, the transit placements do.

Matched: the Sarvatobhadra layout (with the pada sounds of the top band, nakshatras round the border from Krittika, Abhijit between Uttara Ashadha and
Shravana, the twelve rashis, the five tithi groups with their weekdays), natal and transit placements in it, the Kota
layout (Janma nakshatra Punarvasu in the top-left corner, every one of the 28 places), transit placements in the Kota,
and natal placements in the Sanghatta ring. Known differences are marked xfail with their reasons.
"""
from __future__ import annotations

import pytest

import lib

PUB = lib.load("published.json")
RAMANA = next(c for c in PUB["parashara_light_charts"] if c["id"] == "parashara-light-ramana-1879")
REFERENCE_UTC = "2026-11-21T12:57:29Z"

# ── transcribed from the reference ─────────────────────────────────────────────────────────────────────────────────
# Sarvatobhadra: border nakshatras (east = top row left to right, south = right column top to bottom, west = bottom
# row right to left, north = left column bottom to top), rashis of the inner ring and the tithi groups with weekdays.
SBC_TOP = ["Krittika", "Rohini", "Mrigashira", "Ardra", "Punarvasu", "Pushya", "Ashlesha"]
SBC_RIGHT = ["Magha", "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha"]
SBC_BOTTOM_LEFT_TO_RIGHT = ["Shravana", "Abhijit", "Uttara Ashadha", "Purva Ashadha", "Mula", "Jyeshtha", "Anuradha"]
SBC_LEFT = ["Bharani", "Ashwini", "Revati", "Uttara Bhadrapada", "Purva Bhadrapada", "Shatabhisha", "Dhanishta"]
SBC_RASHI_CELLS = {(2, 3): "Vrishabha", (2, 4): "Mithuna", (3, 2): "Mesha", (4, 2): "Meena", (5, 2): "Kumbha",
                   (4, 6): "Kanya", (5, 6): "Tula", (6, 5): "Vrischika", (6, 4): "Dhanu"}
SBC_TITHI_CELLS = {(3, 4): ("Nanda", ["Sunday", "Tuesday"]), (4, 3): ("Rikta", ["Friday"]), (4, 4): ("Purna", None),
                   (4, 5): ("Bhadra", ["Monday", "Wednesday"]), (5, 4): ("Jaya", ["Thursday"])}
# pada sounds printed in the top band of the reference Sarvatobhadra, Krittika to Ashlesha (padas 1 to 4)
SBC_TOP_PADAS = [["अ", "इ", "उ", "ए"], ["ओ", "व", "वी", "वू"], ["वे", "वो", "क", "की"], ["कु", "घ", "ङ", "छ"],
                 ["के", "को", "ह", "ही"], ["हु", "हे", "हो", "ड"], ["डी", "डू", "डे", "डो"]]
# markers read in the reference Sarvatobhadra (natal and transit are drawn together there)
SBC_NATAL = {"ketu": "Punarvasu", "moon": "Punarvasu", "sun": "Purva Ashadha", "rahu": "Purva Ashadha",
             "mercury": "Jyeshtha", "lagna": "Chitra"}
SBC_TRANSIT = {"moon": "Revati", "saturn": "Uttara Bhadrapada", "venus": "Chitra", "mercury": "Swati"}
# Kota: numbered places (1 = Janma) and where the reference draws them: corner, the inward diagonal, the stambha next
# to the centre, and the axis out to the middle of the side
KOTA_PLACES = {1: ("Punarvasu", "top-left corner"), 4: ("Magha", "stambha above the centre"),
               7: ("Hasta", "middle of the top side"), 8: ("Chitra", "top-right corner"),
               11: ("Anuradha", "stambha right of the centre"), 14: ("Purva Ashadha", "middle of the right side"),
               15: ("Uttara Ashadha", "bottom-right corner"), 16: ("Abhijit", "prakara"),
               18: ("Dhanishta", "stambha below the centre"), 21: ("Uttara Bhadrapada", "middle of the bottom side"),
               22: ("Revati", "bottom-left corner"), 25: ("Krittika", "stambha left of the centre"),
               28: ("Ardra", "middle of the left side")}
KOTA_TRANSIT = {"mars": "Magha", "jupiter": "Magha", "ketu": "Magha", "rahu": "Dhanishta", "moon": "Revati"}
KOTA_SCORE = -3.9
# Sanghatta: natal markers by ring sector
SANGHATTA_NATAL = {"moon": "Punarvasu", "ketu": "Punarvasu", "mars": "Bharani", "sun": "Purva Ashadha",
                   "mercury": "Jyeshtha", "venus": "Vishakha", "jupiter": "Shatabhisha"}


def chart(provider, spk):
    c = RAMANA
    req = {"op": "chart", "system": "vedic",
           "when": {"date": c["date"], "time": c["time"], "utc_offset_minutes": c["utc_offset_minutes"]},
           "place": {"latitude": c["lat"], "longitude": c["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"},
           "features": ["chakras", "panchanga"], "sunrise": "centre_unrefracted", "reference_utc": REFERENCE_UTC}
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return lib.attrs(env)


PROVIDERS = [pytest.param(p, spk, id=p) for p, spk in lib.providers()]


def cell(sbc, r, c):
    return sbc["cells"][r * 9 + c]


@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sarvatobhadra_layout(provider, spk):
    sbc = chart(provider, spk)["vedic.chakra.sarvatobhadra"]
    assert [cell(sbc, 0, c)["name"] for c in range(1, 8)] == SBC_TOP
    assert [cell(sbc, r, 8)["name"] for r in range(1, 8)] == SBC_RIGHT
    assert [cell(sbc, 8, c)["name"] for c in range(1, 8)] == SBC_BOTTOM_LEFT_TO_RIGHT
    assert [cell(sbc, r, 0)["name"] for r in range(1, 8)] == SBC_LEFT
    for (r, c), name in SBC_RASHI_CELLS.items():
        assert cell(sbc, r, c)["name"] == name, (r, c)
    for (r, c), (name, days) in SBC_TITHI_CELLS.items():
        x = cell(sbc, r, c)
        assert x["name"] == name and x["kind"] == "tithi", (r, c)
        if days:
            assert x["weekdays"] == days
    assert cell(sbc, 4, 4)["weekdays"] == ["Saturday"]


@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sarvatobhadra_pada_sounds(provider, spk):
    sbc = chart(provider, spk)["vedic.chakra.sarvatobhadra"]
    assert [[s[0] for s in cell(sbc, 0, c)["padas"]] for c in range(1, 8)] == SBC_TOP_PADAS
    naks = [x for x in sbc["cells"] if x["kind"] == "nakshatra"]
    assert len(naks) == 28 and all(len(x["padas"]) == 4 and all(s[0] and s[1] for s in x["padas"]) for x in naks)
    # the four syllables of a nakshatra are its padas in order; no syllable is shared by two padas
    assert len({s[0] for x in naks for s in x["padas"]}) == 112


@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sarvatobhadra_placements(provider, spk):
    sbc = chart(provider, spk)["vedic.chakra.sarvatobhadra"]
    nat = {g["id"]: g["nakshatra"] for g in sbc["grahas"]["natal"]}
    tra = {g["id"]: g["nakshatra"] for g in sbc["grahas"]["transit"]}
    for g, n in SBC_NATAL.items():
        assert nat[g] == n, g
    for g, n in SBC_TRANSIT.items():
        assert tra[g] == n, g
    # the birth tithi (Krishna Dwitiya) and weekday (Monday) both fall in the Bhadra cell
    assert sbc["birth"] == {"tithi": 17, "tithi_group": "Bhadra", "weekday": "Monday"}
    assert cell(sbc, 4, 5)["birth_tithi"] and cell(sbc, 4, 5)["birth_weekday"]


@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_kota_layout_and_transits(provider, spk):
    k = chart(provider, spk)["vedic.chakra.kota"]
    assert k["janma"]["name"] == "Punarvasu"
    lay = {p["position"]: p for p in k["layout"]}
    where = {"top-left corner": (-1, -1), "top-right corner": (1, -1), "bottom-right corner": (1, 1),
             "bottom-left corner": (-1, 1), "middle of the top side": (0, -1), "middle of the right side": (1, 0),
             "middle of the bottom side": (0, 1), "middle of the left side": (-1, 0)}
    for pos, (name, place) in KOTA_PLACES.items():
        p = lay[pos]
        assert p["name"] == name, pos
        if place in where:
            assert (p["x"], p["y"]) == where[place], (pos, place)
            assert p["zone"] == "bahya"
        elif place.startswith("stambha"):
            assert p["zone"] == "stambha" and max(abs(p["x"]), abs(p["y"])) == 0.25, pos
        else:
            assert p["zone"] == place, pos
    tra = {g["id"]: g["nakshatra"] for g in k["transit"]}
    for g, n in KOTA_TRANSIT.items():
        assert tra[g] == n, g


@pytest.mark.xfail(reason="the reference prints a Kota score (-3.9) whose formula it does not publish; ESHANA lists "
                          "which grahas are inside, entering and leaving instead of inventing a score", strict=True)
@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_kota_score(provider, spk):
    k = chart(provider, spk)["vedic.chakra.kota"]
    assert k.get("score") == KOTA_SCORE


@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sanghatta_natal_placements(provider, spk):
    sg = chart(provider, spk)["vedic.chakra.sanghatta"]
    nat = {g["id"]: g["nakshatra"] for g in sg["natal"]}
    for g, n in SANGHATTA_NATAL.items():
        assert nat[g] == n, g
    assert len(sg["ring"]) == 28 and len(sg["pairs"]) == 14


@pytest.mark.xfail(reason="the reference marker for Saturn (347 deg 01 min sidereal) sits on the Uttara Bhadrapada / "
                          "Revati boundary line (346 deg 40 min) and reads as Uttara Bhadrapada; ESHANA places by "
                          "longitude, 0 deg 21 min inside Revati, as the reference planet table also does", strict=True)
@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sanghatta_saturn_as_drawn(provider, spk):
    sg = chart(provider, spk)["vedic.chakra.sanghatta"]
    assert {g["id"]: g["nakshatra"] for g in sg["natal"]}["saturn"] == "Uttara Bhadrapada"


@pytest.mark.xfail(reason="the reference Sanghatta ring is drawn with 27 equal sectors (label spacing 13 deg 20 min) "
                          "and its own inner figure, whose line pattern could not be transcribed; ESHANA draws 28 "
                          "places with Abhijit and the sapta-shalaka vedha pairs (rulesets/chakras.yaml)", strict=True)
@pytest.mark.parametrize("provider,spk", PROVIDERS)
def test_sanghatta_ring_as_drawn(provider, spk):
    sg = chart(provider, spk)["vedic.chakra.sanghatta"]
    assert len(sg["ring"]) == 27
