"""Auxiliary analysis (``vedic.aux.*``) compared with a published reference chart (Parashara's Light output supplied
by the owner, "Auxiliary Analysis" page) for the Ramana chart in ``published.json``.

Values were transcribed from the screenshot (and checked by OCR of the full-resolution image). Sahams are compared
within 6 arc-minutes: they add and subtract up to three longitudes, and ESHANA's Jupiter and Saturn for 1879 differ
from that program's by about 3' and 2' (see test_reference_software.py); with the program's own printed longitudes
the engine's unit test reproduces all 37 within 1'. Vedhasthāna is printed by that program for the birth grahas (its
houses match the natal positions, not the transits at its print date), so it is compared with ``natal``.
"""
from __future__ import annotations

import pytest

import lib

PUB = lib.load("published.json")
CHART = next(c for c in PUB["parashara_light_charts"] if c["id"] == "parashara-light-ramana-1879")
PROVIDERS = lib.providers()
SIGNS = ["Ari", "Tau", "Gem", "Can", "Leo", "Vir", "Lib", "Sco", "Sag", "Cap", "Aqu", "Pis"]

SAHAMS = """Punya Pis 15:59:01; Vidya Tau 11:27:21; Yasa Sco 28:13:03; Mitra Sag 29:58:26; Mahatmya Sag 04:42:17;
Asha Tau 12:08:10; Samartha Ari 29:51:31; Bhratru Leo 28:08:46; Gaurava Leo 29:13:20; Pitru Gem 27:15:12;
Raja Gem 27:15:12; Matru Aqu 00:52:00; Putra Pis 10:33:47; Jeeva Leo 28:08:46; Karma Ari 29:51:31;
Roga Aqu 09:59:22; Kali Cap 04:12:09; Sastra Cap 23:40:52; Bandhu Aqu 23:29:53; Mrityu Cap 16:52:14;
Paradesa Tau 25:48:15; Artha Vir 26:23:02; Paradara Leo 13:36:10; Vanik Gem 03:56:30; Karyasiddhi Leo 11:50:16;
Vivaha Tau 12:08:10; Santapa Sco 16:52:14; Sraddha Ari 07:13:38; Preeti Sag 24:11:31; Jadya Lib 18:11:54;
Vyapara Pis 27:34:52; Satru Leo 23:48:39; Jalapathana Gem 00:46:46; Bandhana Lib 29:47:45; Apamrityu Vir 22:32:54;
Prasava Can 05:20:28; Labha Sco 27:57:04"""
SAHAM_LORD = {"Punya": "jupiter", "Vidya": "venus", "Yasa": "mars", "Matru": "saturn", "Prasava": "moon",
              "Karyasiddhi": "sun", "Artha": "mercury", "Labha": "mars"}
MRITYU = {"lagna": ("Vir", "26:43"), "sun": ("Sag", "06:24"), "moon": ("Gem", "15:19"), "mars": ("Ari", "02:58"),
          "mercury": ("Sco", "13:06"), "jupiter": ("Aqu", "01:29"), "venus": ("Sco", "05:31"),
          "saturn": ("Pis", "02:03"), "rahu": ("Sag", "13:20"), "ketu": ("Gem", "03:20")}
PINDA = {"sun": (80, 23), "moon": (177, 92), "mars": (100, 73), "mercury": (137, 71), "jupiter": (133, 65),
         "venus": (104, 44), "saturn": (92, 15), "lagna": (153, 111)}
VEDHA = ["Su 7 Inausp", "Mo 1 Ausp", "Ma 11 Ausp", "Me 6 Ausp, X by Ju in 9", "Ju 9 Ausp, X by Sa in 10",
         "Ve 6 Inausp", "Sa 10 Inausp"]


def lon_of(sign: str, dms: str) -> float:
    d, m, s = (int(x) for x in dms.split(":"))
    return SIGNS.index(sign) * 30 + d + m / 60 + s / 3600


def aux(provider, spk):
    c = CHART
    req = {"op": "chart", "system": "vedic",
           "when": {"date": c["date"], "time": c["time"], "utc_offset_minutes": c["utc_offset_minutes"]},
           "place": {"latitude": c["lat"], "longitude": c["lon"]},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"},
           "features": ["auxiliary"], "sunrise": "centre_unrefracted", "reference_utc": "2026-11-21T12:00:00Z"}
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return {k: v for k, v in lib.attrs(env).items() if k.startswith("vedic.aux.")}


CASES = [pytest.param(p, spk, id=p) for p, spk in PROVIDERS]


@pytest.mark.parametrize("provider,spk", CASES)
def test_sahams(provider, spk):
    v = aux(provider, spk)["vedic.aux.sahams"]
    assert v["birth_in"] == "night"
    got = {r["name"]: r for r in v["sahams"]}
    assert len(got) == 37
    for item in SAHAMS.replace("\n", " ").split(";"):
        name, sign, dms = item.split()
        want = lon_of(sign, dms)
        diff = abs(lib.adiff(got[name]["longitude"], want)) * 60
        assert diff <= 6.0, f"{name}: {diff:.1f}'"
        assert got[name]["sign"] == SIGNS.index(sign), name
    for name, lord in SAHAM_LORD.items():
        assert got[name]["lord"] == lord, name


@pytest.mark.parametrize("provider,spk", CASES)
def test_avakahada(provider, spk):
    a = aux(provider, spk)["vedic.aux.avakahada"]
    assert (a["ascendant"]["sign_name"], a["ascendant"]["lord"]) == ("Kanya", "mercury")
    assert (a["moon_sign"]["sign_name"], a["moon_sign"]["lord"]) == ("Mithuna", "mercury")
    assert (a["nakshatra"]["name"], a["nakshatra"]["pada"]) == ("Punarvasu", 3)
    assert (a["yoga"], a["karana"]["name"], a["karana"]["animal"]) == ("Indra", "Taitila", "donkey")
    assert (a["yoni"]["name"], a["gana"]) == ("Marjara", "Deva")
    assert (a["nadi"], a["varna"]) == ("Adya", "Shudra")
    assert (a["vashya"], a["varga"]["name"]) == ("Manava", "Mesha")
    assert (a["yunja"], a["hansak"]) == ("Madhya", "Vaayu")
    assert a["naamakshara"] == "Ha"  # printed "Hah"
    assert (a["paya"]["by_sign"], a["paya"]["by_nakshatra"]) == ("Copper", "Silver")
    assert a["sunsign_western"] == "Capricorn"


@pytest.mark.parametrize("provider,spk", CASES)
def test_male_female(provider, spk):
    mf = aux(provider, spk)["vedic.aux.male_female"]
    assert set(mf["female"]) == {"lagna", "mercury", "venus", "saturn"}
    assert set(mf["male"]) == {"sun", "ketu", "moon", "mars", "jupiter", "rahu"}


@pytest.mark.parametrize("provider,spk", CASES)
def test_mrityu_bhaga(provider, spk):
    rows = {r["id"]: r for r in aux(provider, spk)["vedic.aux.mrityu_bhaga"]["points"]}
    for pid, (sign, dist) in MRITYU.items():
        d, m = (int(x) for x in dist.split(":"))
        assert rows[pid]["sign"] == SIGNS.index(sign), pid
        # printed to the arc-minute; ESHANA's Jupiter and Saturn differ from that program's by ~3' and ~2'
        assert abs(rows[pid]["distance_deg"] - (d + m / 60)) * 60 <= 4.0, pid


@pytest.mark.parametrize("provider,spk", CASES)
def test_ghata_chakra(provider, spk):
    g = aux(provider, spk)["vedic.aux.ghata_chakra"]
    assert (g["month"], g["tithis"], g["weekday_name"]) == ("Ashadha", [2, 7, 12], "Monday")
    assert (g["nakshatra_name"], g["yoga"], g["karana"], g["karana_animal"]) == ("Swati", "Parigha", "Kaulava", "pig")
    assert (g["prahara"], g["moon_sign_name"]) == (3, "Kumbha")


@pytest.mark.parametrize("provider,spk", CASES)
def test_vedhasthana(provider, spk):
    v = aux(provider, spk)["vedic.aux.vedhasthana"]
    assert [r["summary"] for r in v["natal"]] == VEDHA
    assert v["transit"] and len(v["transit"]["grahas"]) == 7


@pytest.mark.parametrize("provider,spk", CASES)
def test_jiva_deha(provider, spk):
    j = aux(provider, spk)["vedic.aux.jiva_deha"]
    assert (j["jiva"]["sign_name"], j["deha"]["sign_name"], j["paramayus_years"]) == ("Mithuna", "Vrishabha", 83)


@pytest.mark.parametrize("provider,spk", CASES)
def test_shodhana_pindas(provider, spk):
    s = aux(provider, spk)["vedic.aux.shodhana"]["by_chart"]
    for g, (rashi, graha) in PINDA.items():
        assert (s[g]["rashi_pinda"], s[g]["graha_pinda"], s[g]["shodhya_pinda"]) == (rashi, graha, rashi + graha), g
