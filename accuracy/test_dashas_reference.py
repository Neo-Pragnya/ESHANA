# Extended dasa systems for the Ramana chart, compared with the published reference chart (owner-supplied
# screenshot of page "15. Dasha Analysis", 2026-11-21; the fixture in published.json names the program).
#
# The reference lists the running periods of each system from about 2023-2026 on as two-level labels ("Lib-Aqu":
# the mahadasha sign or lord, then the antardasha) with the local (LMT +5:21) date each period starts. The rows
# below were read from the screenshot with OCR and checked by eye.
#
# * Sign-based systems (Chara, Narayana, Navamsha, Shoola) must match to the day.
# * Ashtottari and Shodashottari may differ by one day: the reference prints the Moon 13 arcseconds further on
#   (28 19 55 against 28 19 42), which moves the dates of these long-cycle systems earlier by up to about a day.
# * Kalachakra is a known, documented difference (see the xfail reason and docs/guides/dashas.md).
from __future__ import annotations

import datetime as dt

import pytest

import lib

PUB = lib.load("published.json")
RAMANA = next(c for c in PUB["parashara_light_charts"] if c["id"] == "parashara-light-ramana-1879")
REFERENCE_UTC = "2026-11-21T00:00:00Z"
OFFSET = dt.timedelta(minutes=321)
GRAHAS = {"Su": "Sun", "Mo": "Moon", "Ma": "Mars", "Me": "Mercury", "Ju": "Jupiter", "Ve": "Venus", "Sa": "Saturn",
          "Ra": "Rahu", "Ke": "Ketu"}

# (mahadasha-antardasha, start date as printed)
REFERENCE = {
    "navamsha": [("Lib-Aqu", "29-10-2026"), ("Lib-Pis", "29-11-2026"), ("Sco-Sco", "29-12-2026"),
                 ("Sco-Lib", "30-05-2027"), ("Sco-Vir", "30-10-2027"), ("Sco-Leo", "30-03-2028"),
                 ("Sco-Can", "29-08-2028"), ("Sco-Gem", "28-01-2029"), ("Sco-Sag", "30-07-2031"),
                 ("Sag-Gem", "29-12-2031"), ("Sag-Can", "28-02-2032"), ("Cap-Can", "29-12-2033"),
                 ("Cap-Gem", "28-02-2034"), ("Aqu-Aqu", "29-12-2035"), ("Aqu-Pis", "29-01-2036")],
    "narayana": [("Ari-Tau", "29-12-2025"), ("Ari-Gem", "29-12-2026"), ("Ari-Can", "29-12-2027"),
                 ("Ari-Pis", "29-12-2035"), ("Tau-Sco", "29-12-2036"), ("Tau-Lib", "29-06-2037"),
                 ("Tau-Vir", "29-12-2037"), ("Tau-Gem", "30-06-2039"), ("Gem-Gem", "29-12-2042"),
                 ("Gem-Can", "30-05-2043"), ("Gem-Leo", "29-10-2043"), ("Can-Can", "29-12-2047"),
                 ("Can-Gem", "29-01-2048"), ("Can-Tau", "28-02-2048")],
    "shoola": [("Can-Ari", "30-03-2026"), ("Can-Pis", "29-12-2026"), ("Can-Aqu", "29-09-2027"),
               ("Can-Cap", "29-06-2028"), ("Can-Leo", "30-03-2032"), ("Leo-Leo", "29-12-2032"),
               ("Leo-Vir", "29-09-2033"), ("Vir-Vir", "29-12-2041"), ("Vir-Leo", "29-09-2042"),
               ("Lib-Lib", "29-12-2050"), ("Lib-Sco", "29-09-2051")],
    "chara": [("Sco-Sag", "29-10-2026"), ("Sco-Sco", "30-05-2027"), ("Sag-Sco", "29-12-2027"),
              ("Sag-Lib", "28-02-2028"), ("Sag-Vir", "29-04-2028"), ("Sag-Sag", "29-10-2029"),
              ("Cap-Sag", "29-12-2029"), ("Cap-Sco", "29-10-2030"), ("Cap-Lib", "30-08-2031"),
              ("Cap-Cap", "28-02-2039"), ("Aqu-Pis", "29-12-2039"), ("Aqu-Ari", "28-02-2040"),
              ("Pis-Ari", "29-12-2041"), ("Pis-Tau", "28-01-2042")],
    "ashtottari": [("Me-Ve", "08-09-2023"), ("Me-Su", "28-12-2026"), ("Me-Mo", "08-12-2027"),
                   ("Me-Ma", "19-04-2030"), ("Sa-Sa", "23-07-2031"), ("Sa-Ju", "25-06-2032"),
                   ("Sa-Ra", "29-03-2034"), ("Sa-Ve", "09-05-2035"), ("Su-Su", "22-07-2093")],
    "shodashottari": [("Ke-Ju", "19-05-2026"), ("Ke-Sa", "23-01-2028"), ("Mo-Mo", "15-11-2029"),
                      ("Mo-Me", "30-01-2032"), ("Mo-Ve", "04-06-2034"), ("Mo-Su", "27-11-2036"),
                      ("Mo-Ma", "04-06-2038"), ("Me-Me", "15-11-2045"), ("Ve-Ve", "15-11-2062")],
    "kalachakra": [("Gem-Cap", "21-11-2026"), ("Gem-Aqu", "22-08-2027"), ("Gem-Pis", "21-05-2028"),
                   ("Tau-Sco", "21-08-2030"), ("Tau-Lib", "21-12-2031"), ("Ari-Ari", "21-08-2046"),
                   ("Pis-Pis", "21-08-2053"), ("Aqu-Aqu", "21-08-2063")],
}
# The period running at the reference instant, as the first row of each list shows.
RUNNING = {"navamsha": ("Lib", "Aqu"), "narayana": ("Ari", "Tau"), "shoola": ("Can", "Ari"),
           "chara": ("Sco", "Sag"), "ashtottari": ("Me", "Ve"), "shodashottari": ("Ke", "Ju")}
TOLERANCE_DAYS = {"navamsha": 0, "narayana": 0, "shoola": 0, "chara": 0, "ashtottari": 1, "shodashottari": 1,
                  "kalachakra": 0}


def chart(reference_utc: str = REFERENCE_UTC):
    c = RAMANA
    req = {"op": "chart", "system": "vedic",
           "when": {"date": c["date"], "time": c["time"], "utc_offset_minutes": c["utc_offset_minutes"]},
           "place": {"latitude": c["lat"], "longitude": c["lon"]},
           "config": {"provider": "analytic", "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"},
           "features": ["dashas_extended", "panchanga"], "reference_utc": reference_utc}
    env = lib.run(req)
    assert env["kind"] != "error", env
    return lib.attrs(env)


def key(label: str) -> str:
    # the label the reference uses for one of the periods: a sign abbreviation or a two-letter graha
    for k, v in GRAHAS.items():
        if v == label:
            return k
    return label[:3]


def local_date(utc: str) -> dt.date:
    return (dt.datetime.fromisoformat(utc.replace("Z", "+00:00")) + OFFSET).date()


def starts(value) -> dict:
    out = {}
    for m in value["mahadashas"]:
        for a in m.get("antardashas", []):
            out.setdefault(key(m["label"]) + "-" + key(a["label"]), []).append(local_date(a["start_utc"]))
    return out


def compare(system: str):
    got = starts(chart()["vedic.dasha." + system])
    misses = []
    for label, printed in REFERENCE[system]:
        want = dt.datetime.strptime(printed, "%d-%m-%Y").date()
        cands = got.get(label, [])
        best = min((abs((d - want).days) for d in cands), default=None)
        if best is None or best > TOLERANCE_DAYS[system]:
            misses.append((label, printed, [d.isoformat() for d in cands if abs((d - want).days) < 400]))
    return misses


@pytest.mark.parametrize("system", ["navamsha", "narayana", "shoola", "chara", "ashtottari", "shodashottari"])
def test_period_starts_match_the_reference(system):
    assert compare(system) == []


@pytest.mark.parametrize("system", sorted(RUNNING))
def test_running_chain_at_the_reference_instant(system):
    cur = chart()["vedic.dasha." + system]["current"]["periods"]
    assert (key(cur[0]["label"]), key(cur[1]["label"])) == RUNNING[system]


def test_every_system_names_its_conventions():
    a = chart()
    for system in REFERENCE:
        v = a["vedic.dasha." + system]
        assert v["conventions"], system
        assert v["mahadashas"], system


def test_applicability_flags():
    a = chart()
    # Rahu (Dhanu) is 2nd from Mercury (Vrishchika), the lagna lord: not a kendra or trikona
    assert a["vedic.dasha.ashtottari"]["applicability"]["applies"] is False
    # lagna 28 43 Kanya is in the Sun hora (even sign, second half) and the Moon is waning: Shodashottari applies
    sh = a["vedic.dasha.shodashottari"]["applicability"]
    assert (sh["lagna_hora"], sh["paksha"], sh["applies"]) == ("sun", "krishna", True)


def test_kalachakra_rows_contain_the_reference_sequence():
    # The reference runs Mithuna, Vrishabha, Mesha, Mina, then Kumbha as deha with Kanya as jiva: the end of the
    # Pushya pada-1 row and the start of the pada-2 row. ESHANA has the same rows in its table.
    k = chart("2060-01-01T00:00:00Z")["vedic.dasha.kalachakra"]
    rows = [(m["nakshatra"], m["pada"], m["sign"], m["deha"], m["jiva"]) for m in k["mahadashas"]]
    p1 = [r[2] for r in rows if r[:2] == ("Pushya", 1)]
    p2 = [r for r in rows if r[:2] == ("Pushya", 2)]
    assert p1[-4:] == [2, 1, 0, 11]
    assert (p2[0][2], p2[0][3], p2[0][4]) == (10, 10, 5)


@pytest.mark.xfail(strict=True, reason=(
    "Kalachakra timing differs: ESHANA follows BPHS (janma pada Punarvasu 3 with its balance, then Punarvasu 4, then "
    "Pushya 1), which starts the Pushya pada-1 row in 2007; the reference starts that row about 44 years earlier "
    "(Mithuna 2021-08 to 2030-08) and splits each mahadasha into 12 equal sign antardashas, where BPHS uses the "
    "nine signs of the pada row in proportion. Convention not identified; recorded in docs/guides/dashas.md."))
def test_kalachakra_matches_the_reference():
    assert compare("kalachakra") == []
