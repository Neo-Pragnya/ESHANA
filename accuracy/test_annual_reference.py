"""Varṣaphala, pañca pakṣī and the KP tables against a published reference chart (Parashara’s Light output for Sri
Ramana Maharshi, supplied by the project owner as a screenshot, page "48. Auxiliary Analysis", 2026-11-21).

Matched: the pañcavargīya table of the birth year (all 35 cells), the muntha, the five office-bearers, the birth bird,
the birth day’s ruling and dying birds, the pakṣa, the yāma start times, the bird order inside a yāma, the KP star
and sub lords of every graha, and the star and sub lords of the twelve cusps that program prints (equal cusps from the
Ascendant). Known differences are marked xfail with the reason.
"""
from __future__ import annotations

import pytest

import lib

CASES = [pytest.param(p, spk, id=p) for p, spk in lib.providers()]
AB = {"Su": "sun", "Mo": "moon", "Ma": "mars", "Me": "mercury", "Ju": "jupiter", "Ve": "venus", "Sa": "saturn", "Ra": "rahu",
      "Ke": "ketu"}
SIGNS = {"Ar": 0, "Ta": 1, "Ge": 2, "Cn": 3, "Le": 4, "Vi": 5, "Li": 6, "Sc": 7, "Sg": 8, "Cp": 9, "Aq": 10, "Pi": 11}

# "Varshaphala strengths": Griha, Uchcha, Hudda, Drekkana, Navamsha, Total (the birth year, year 0)
STRENGTHS = {
    "Su": (22.50, 7.29, 7.50, 2.50, 5.00, 11.20),
    "Mo": (15.00, 13.85, 3.75, 2.50, 2.50, 9.40),
    "Ma": (30.00, 10.67, 15.00, 5.00, 2.50, 15.79),
    "Me": (15.00, 12.43, 3.75, 2.50, 3.75, 9.36),
    "Ju": (15.00, 4.61, 15.00, 2.50, 2.50, 9.90),
    "Ve": (15.00, 3.72, 7.50, 5.00, 2.50, 8.43),
    "Sa": (15.00, 3.66, 11.25, 5.00, 2.50, 9.35),
}
# "Year lord" block: Muntha Lib 28:43; the five office-bearers with the number the program prints beside each
OFFICES = {"muntha_pati": ("Ve", 13.63), "janma_lagna_pati": ("Me", 10.43), "varsha_lagna_pati": ("Ju", 16.10),
           "trirashi_pati": ("Sa", 8.37), "dinaratri_pati": ("Ju", 16.10)}
YEAR_LORD = "Ju"
# The program’s year with the muntha in Libra is the one after 145 completed years, from the solar return of
# 31 Dec 2024 to that of 31 Dec 2025; a reference instant inside it selects the same year here.
REF_IN_YEAR_145 = "2025-06-01T00:00:00Z"
# "KP Grahas": Lord (star lord) / Sub
KP_GRAHAS = {"Su": ("Ve", "Su"), "Mo": ("Ju", "Ve"), "Ma": ("Ve", "Sa"), "Me": ("Me", "Mo"), "Ju": ("Ra", "Ve"),
             "Ve": ("Ju", "Mo"), "Sa": ("Me", "Me"), "Ra": ("Ve", "Sa"), "Ke": ("Ju", "Sa")}
# "KP significators": cusps 1-12 all at 28:43:11 from Virgo; star lord / sub lord of each. The 5th is read as Ma/Sa:
# 28:43 Capricorn lies in Dhanishta (a Mars star), and the transcription Ju/Ve there breaks the four-sign cycle the
# other eleven follow.
KP_CUSPS = [("Ma", "Sa"), ("Ju", "Ve"), ("Me", "Sa"), ("Su", "Ma"), ("Ma", "Sa"), ("Ju", "Ve"), ("Me", "Sa"), ("Su", "Ma"),
            ("Ma", "Sa"), ("Ju", "Ve"), ("Me", "Sa"), ("Su", "Ma")]
# "Panch Pakshi for Tue 30 Dec 1879 01:00:00, Dark Half, Ruling bird: Peacock, Dying bird: Owl". The table starts at
# 06:27:47, the sunrise of Monday 29 Dec (the Vedic day of a birth at 01:00 on Tuesday): Cock Ruling (self), Owl
# Walking, Peacock Eating, Crow Dying, Vulture Sleeping; then 08:44:55 Walking, 11:02:03 Eating, 13:19:12 Dying,
# 15:36:20 Sleeping, ending 17:53:28.
PAKSHI_STARTS = ["06:27:47", "08:44:55", "11:02:03", "13:19:12", "15:36:20"]
PAKSHI_SELF = ["ruling", "walking", "eating", "dying", "sleeping"]
PAKSHI_FIRST_SUBS = [("cock", "ruling", "self"), ("owl", "walking", "friend"), ("peacock", "eating", "friend"),
                     ("crow", "dying", "enemy"), ("vulture", "sleeping", "enemy")]


def chart(provider, spk, reference_utc=None):
    req = {"op": "chart", "system": "vedic", "when": {"date": "1879-12-30", "time": "01:00", "utc_offset_minutes": 321},
           "place": {"latitude": 9.9333, "longitude": 78.1167},
           "config": {"provider": provider, "zodiac": "sidereal", "ayanamsha": "lahiri", "node_model": "true"},
           "features": ["annual", "kp", "panchanga"], "sunrise": "centre_unrefracted"}
    if reference_utc:
        req["reference_utc"] = reference_utc
    if spk:
        req["spk_path"] = spk
    env = lib.run(req)
    assert env["kind"] != "error", env
    return {a["id"]: a["value"] for a in env["derived_attributes"]}


def seconds(hms: str) -> int:
    h, m, s = (int(x) for x in hms.split(":"))
    return h * 3600 + m * 60 + s


@pytest.mark.parametrize("provider,spk", CASES)
def test_pancavargiya_bala_of_the_birth_year(provider, spk):
    bala = chart(provider, spk)["vedic.varshaphala"]["birth_year"]["pancavargiya_bala"]
    for ab, row in STRENGTHS.items():
        got = bala[AB[ab]]
        for key, want in zip(("griha", "uchcha", "hudda", "drekkana", "navamsha", "total"), row):
            # printed to two decimals; the uchcha part moves 0.011 per 0.1° of longitude
            assert abs(got[key] - want) <= 0.015, (ab, key, got[key], want)


@pytest.mark.parametrize("provider,spk", CASES)
def test_muntha_and_office_bearers(provider, spk):
    y = chart(provider, spk, REF_IN_YEAR_145)["vedic.varshaphala"]["current"]
    assert y["completed_years"] == 145
    assert y["muntha"]["sign"] == SIGNS["Li"]
    assert abs(y["muntha"]["longitude_deg"] - (180 + 28 + 43 / 60)) < 2 / 60
    assert {o["role"]: o["graha"] for o in y["office_bearers"]} == {k: AB[v[0]] for k, v in OFFICES.items()}
    # Jupiter as varṣa lagna pati and Saturn as trirāśi pati together imply a Sagittarius varṣa lagna
    assert y["varsha_lagna"]["sign"] == SIGNS["Sg"]


@pytest.mark.xfail(strict=True, reason="the number printed beside each office-bearer (Ju 16.10, Ve 13.63, Me 10.43, "
                   "Sa 8.37) is not the pañcavargīya total of the varṣa chart that ESHANA computes (Ju 10.91, Ve 10.05, "
                   "Me 10.57, Sa 13.53); the program does not say which measure it prints, so its year lord (Jupiter) "
                   "differs from the strongest office-bearer aspecting the varṣa lagna by ESHANA’s rule (Saturn)")
@pytest.mark.parametrize("provider,spk", CASES)
def test_office_bearer_strengths_and_year_lord(provider, spk):
    y = chart(provider, spk, REF_IN_YEAR_145)["vedic.varshaphala"]["current"]
    for o in y["office_bearers"]:
        assert abs(o["total"] - OFFICES[o["role"]][1]) <= 0.015, o["role"]
    assert y["year_lord"]["graha"] == AB[YEAR_LORD]


@pytest.mark.xfail(strict=True, reason="the program shows the muntha in Libra (145 completed years) at its ‘now’ of "
                   "21 Nov 2026, but 146 solar returns were complete on 31 Dec 2025, so ESHANA’s running year then has "
                   "the muntha in Scorpio; the program appears to count one year fewer")
@pytest.mark.parametrize("provider,spk", CASES)
def test_running_year_on_the_programs_date(provider, spk):
    y = chart(provider, spk, "2026-11-21T06:00:00Z")["vedic.varshaphala"]["current"]
    assert y["muntha"]["sign"] == SIGNS["Li"]


@pytest.mark.parametrize("provider,spk", CASES)
def test_panch_pakshi_birth_bird_day_and_yamas(provider, spk):
    p = chart(provider, spk)["vedic.panch_pakshi"]
    assert p["birth_bird"] == "cock"
    assert p["paksha"] == "krishna"
    assert p["birth_day"]["ruling_bird"] == "peacock"
    assert p["birth_day"]["dying_bird"] == "owl"
    day = [y for y in p["birth_day_table"]["yamas"] if y["half"] == "day"]
    for y, want in zip(day, PAKSHI_STARTS):
        got = y["start"]["local"][11:19]
        assert abs(seconds(got) - seconds(want)) <= 3, (got, want)
    assert abs(seconds(day[-1]["end"]["local"][11:19]) - seconds("17:53:28")) <= 3
    # inside a yāma the birds follow the reference order (a rotation of Cock, Owl, Peacock, Crow, Vulture)
    want = [b for b, _, _ in PAKSHI_FIRST_SUBS]
    for y in day:
        birds = [s["bird"] for s in y["sub_periods"]]
        r = want.index(birds[0])
        assert birds == want[r:] + want[:r]


@pytest.mark.parametrize("provider,spk", CASES)
def test_panch_pakshi_nearby_days_step_one_vedic_day(provider, spk):
    # The day picker's tables: three Vedic days either side of the reference day, in order, each a real day of its own
    # (sunrise about 24 h after the previous, the weekday one later, the previous day's next sunrise is its sunrise),
    # the day's ruling bird ruling the first yāma, and no yāma marked current except on the reference day.
    p = chart(provider, spk, "2026-11-21T00:00:00Z")["vedic.panch_pakshi"]
    by = {d["offset"]: d["table"] for d in p["nearby_days"]}
    assert sorted(by) == [-3, -2, -1, 1, 2, 3]
    by[0] = p["reference_day_table"]
    for k in range(-3, 4):
        t = by[k]
        assert len(t["yamas"]) == 10
        assert any(y["current"] for y in t["yamas"]) == (k == 0), k
        first = next(b for b in t["yamas"][0]["birds"] if b["activity"] == "ruling")["bird"]
        assert first == t["day_ruling_bird"], k
        if k > -3:
            prev = by[k - 1]
            gap = t["sunrise"]["jd_ut"] - prev["sunrise"]["jd_ut"]
            assert abs(gap - 1.0) < 0.01, (k, gap)
            assert t["weekday"] == (prev["weekday"] + 1) % 7
            assert abs(t["sunrise"]["jd_ut"] - prev["next_sunrise"]["jd_ut"]) < 1e-6


@pytest.mark.xfail(strict=True, reason="printed weekday tables disagree on the activity each bird starts the day "
                   "with; ESHANA lets the day’s ruling bird (Peacock on a dark-half Monday) rule the first yāma, while "
                   "that program starts the Cock ruling (see rulesets/annual.yaml)")
@pytest.mark.parametrize("provider,spk", CASES)
def test_panch_pakshi_activities_of_the_birth_day(provider, spk):
    t = chart(provider, spk)["vedic.panch_pakshi"]["birth_day_table"]
    day = [y for y in t["yamas"] if y["half"] == "day"]
    assert [y["activity"] for y in day] == PAKSHI_SELF
    assert [(s["bird"], s["activity"], s["relation"]) for s in day[0]["sub_periods"]] == PAKSHI_FIRST_SUBS


@pytest.mark.parametrize("provider,spk", CASES)
def test_kp_graha_lords(provider, spk):
    t = chart(provider, spk)["vedic.kp"]["significator_table"]
    got = {g["graha"]: (g["star_lord"], g["sub_lord"]) for g in t["grahas"]}
    for ab, (star, sub) in KP_GRAHAS.items():
        assert got[AB[ab]] == (AB[star], AB[sub]), ab


@pytest.mark.parametrize("provider,spk", CASES)
def test_kp_cusp_lords_with_equal_cusps(provider, spk):
    eq = chart(provider, spk)["vedic.kp"]["significator_table"]["equal_cusps_from_ascendant"]
    for row, (star, sub) in zip(eq, KP_CUSPS):
        assert abs(lib.adiff(row["cusp"] % 30, 28 + 43 / 60 + 11 / 3600)) < 2 / 60
        assert (row["star_lord"], row["sub_lord"]) == (AB[star], AB[sub]), row["house"]


@pytest.mark.xfail(strict=True, reason="that program prints equal cusps from the Ascendant on its KP page; ESHANA’s "
                   "vedic.kp uses Placidus cusps (the KP convention), so the lords of most cusps differ; the equal-cusp "
                   "view is given as equal_cusps_from_ascendant and matches (test above)")
@pytest.mark.parametrize("provider,spk", CASES)
def test_kp_placidus_cusp_lords(provider, spk):
    t = chart(provider, spk)["vedic.kp"]["significator_table"]
    for row, (star, sub) in zip(t["houses"], KP_CUSPS):
        assert (row["star_lord"], row["sub_lord"]) == (AB[star], AB[sub]), row["house"]
