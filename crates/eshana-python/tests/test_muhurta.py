"""Muhurta windows and the personal calendar: CLI, Python and raw JSON agree; the studio page is keyboard-driven."""
import json
import os
import re
import subprocess

from eshana import ChartEngine, Context, execute

BIN = os.environ.get("ESHANA_BIN", "eshana")
NATAL = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.61, 77.21)


def test_muhurta_parity():
    py = ChartEngine().muhurta_windows(19.07, 72.88, "2026-11-01T00:00:00Z", "2026-11-03T00:00:00Z",
                                       timezone="Asia/Kolkata", profile="daily_avoidances")
    raw = execute({"op": "muhurta_windows", "place": {"latitude": 19.07, "longitude": 72.88}, "timezone": "Asia/Kolkata",
                   "start_utc": "2026-11-01T00:00:00Z", "end_utc": "2026-11-03T00:00:00Z", "profile": "daily_avoidances",
                   "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}})
    cli = json.loads(subprocess.check_output([BIN, "--compact", "muhurta", "--lat", "19.07", "--lon", "72.88", "--tz",
                                              "Asia/Kolkata", "--start", "2026-11-01T00:00:00Z", "--end",
                                              "2026-11-03T00:00:00Z", "--profile", "daily_avoidances"]))
    assert py == raw == cli
    assert py["windows"]


def test_calendar_parity_and_ics_from_the_cli():
    py = ChartEngine().personal_calendar(NATAL, 19.07, 72.88, "2026-11-01T00:00:00Z", "2026-11-08T00:00:00Z",
                                         timezone="Asia/Kolkata", format="ics")
    cli = subprocess.check_output([BIN, "calendar", "--birth", "1990-05-15,14:30,Asia/Kolkata,28.61,77.21", "--lat",
                                   "19.07", "--lon", "72.88", "--tz", "Asia/Kolkata", "--start", "2026-11-01T00:00:00Z",
                                   "--end", "2026-11-08T00:00:00Z", "--profile", "personal_general", "--format", "ics"]
                                  ).decode("utf-8")
    assert cli == py["content"] + "\n"  # the CLI prints the file and a final newline
    assert py["content"].count("BEGIN:VEVENT") == len(py["windows"])


def test_studio_calendar_page():
    from eshana import report as R

    page = R.birth_report(NATAL, reference_utc="2026-11-01T00:00:00Z", transit_days=0, progression_years=0,
                          systems=["vedic"], features=[], calendar_days=14, sections=["overview", "calendar"])
    raw = re.search(r'<script type="application/json" id="eshana-data">(.*?)</script>', page, re.S).group(1)
    cal = json.loads(raw.replace("<\\/", "</"))["envelopes"]["personal_calendar"]
    assert len(cal["days"]) >= 14
    assert cal["input"]["birth"]["place_role"] == "birth" and cal["input"]["event_place"]["place_role"] == "event"
    assert "ArrowRight" in page and "aria-pressed" in page and 'id="calendar"' in page


def test_pair_calendar_and_supplementary_matching_parity():
    b = Context.from_local_birth("1991-01-10", "09:15", "Asia/Kolkata", 19.076, 72.8777)
    py = ChartEngine().pair_calendar(NATAL, b, 19.07, 72.88, "2026-11-01T00:00:00Z", "2027-01-01T00:00:00Z",
                                     timezone="Asia/Kolkata")
    cli = json.loads(subprocess.check_output([BIN, "--compact", "pair-calendar", "--a", "1990-05-15,14:30,Asia/Kolkata,28.61,77.21",
                                              "--b", "1991-01-10,09:15,Asia/Kolkata,19.076,72.8777", "--lat", "19.07",
                                              "--lon", "72.88", "--tz", "Asia/Kolkata", "--start", "2026-11-01T00:00:00Z",
                                              "--end", "2027-01-01T00:00:00Z"]))
    assert py["windows"] == cli["windows"] and py["windows"]
    m = ChartEngine().matching(NATAL, b)
    assert {f["id"] for f in m["supplementary"]["moon_factors"]} == {"rajju", "vedha", "mahendra", "stri_dirgha"}


def test_research_window_score_orders_calendar_windows():
    # the model reads choghaḍiyā, horā, Abhijit and vāra too: ask for them on every window
    cal = ChartEngine().personal_calendar(NATAL, 19.07, 72.88, "2026-11-01T00:00:00Z", "2026-11-15T00:00:00Z",
                                          timezone="Asia/Kolkata", show=["choghadiya", "hora", "abhijit", "vara"])
    r = ChartEngine.research_window_score(cal["windows"])
    assert r["status"] == "experimental" and r["epistemic"] == "research_hypothesis"
    scores = [x["score"] for x in r["ranked"] if isinstance(x["score"], float)]
    assert scores == sorted(scores, reverse=True) and len(scores) == len(cal["windows"])
    for x in r["ranked"]:
        assert abs(sum(x["contributions"].values()) - x["score"]) < 1e-9
        assert x["coverage"] == 1.0  # a personal calendar has every factor the model reads
