"""KP horary: the 1-249 table rebuilt independently with exact fractions, and CLI/Python/raw JSON parity."""
import json
import os
import subprocess
from fractions import Fraction

from eshana import ChartEngine, Context, execute

# Vimśottarī lords from Ketu, with years; nakṣatra lords repeat this order from Aśvinī.
LORDS = [("ketu", 7), ("venus", 20), ("sun", 6), ("moon", 10), ("mars", 7), ("rahu", 18), ("jupiter", 16),
         ("saturn", 19), ("mercury", 17)]


def reference_table():
    """Cut every nakṣatra by its subs and every sub by any sign boundary inside it: exact rational degrees."""
    nak = Fraction(40, 3)
    out = []
    for n in range(27):
        start = n * nak
        star = n % 9
        for k in range(9):
            lord, years = LORDS[(star + k) % 9]
            end = start + nak * years / 120
            cuts = [start] + [Fraction(30 * s) for s in range(13) if start < 30 * s < end] + [end]
            for a, b in zip(cuts, cuts[1:]):
                out.append((a, b, LORDS[star][0], lord))
            start = end
    return out


def test_table_matches_an_independent_construction():
    ref = reference_table()
    got = ChartEngine.kp_table()["table"]["entries"]
    assert len(ref) == len(got) == 249
    for (a, b, star, sub), e in zip(ref, got):
        assert abs(e["start_deg"] - float(a)) < 1e-12 and abs(e["end_deg"] - float(b)) < 1e-12, e
        assert (e["star_lord"], e["sub_lord"]) == (star, sub), e
        assert e["sign_index"] == int(a // 30)


def test_horary_parity_python_cli_raw():
    ctx = Context.from_utc("2026-11-23T10:00:00Z", 13.08, 80.27)
    py = ChartEngine().kp_horary(100, ctx)
    raw = execute({"op": "kp_horary", "number": 100,
                   "chart": {"when": {"utc": "2026-11-23T10:00:00Z"}, "place": {"latitude": 13.08, "longitude": 80.27},
                             "system": "vedic", "config": {"zodiac": "sidereal", "ayanamsha": "krishnamurti"}}})
    cli = json.loads(subprocess.check_output([os.environ.get("ESHANA_BIN", "eshana"), "--compact", "kp-horary", "--number", "100", "--utc",
                                              "2026-11-23T10:00:00Z", "--lat", "13.08", "--lon", "80.27"]))
    assert py == raw == cli
    assert py["ascendant"]["longitude_deg"] == py["number"]["start_deg"]


def test_studio_report_carries_the_horary_envelope():
    import re

    from eshana import report as R

    ctx = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)
    page = R.birth_report(ctx, reference_utc="2026-11-23T10:00:00Z", transit_days=0, progression_years=0,
                          systems=["vedic"], features=[], kp_number=23, sections=["overview", "horary"])
    raw = re.search(r'<script type="application/json" id="eshana-data">(.*?)</script>', page, re.S).group(1)
    data = json.loads(raw.replace("<\\/", "</"))
    hor = data["envelopes"]["kp_horary"]
    assert hor["input"]["number"] == 23 and hor["input"]["judgement_utc"] == "2026-11-23T10:00:00Z"
    assert hor["ascendant"]["longitude_deg"] == 30.0
    assert 'id="horary"' in page


def test_ashtakavarga_transits_parity_python_cli_raw():
    ctx = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)
    py = ChartEngine().ashtakavarga_transits(ctx, "2025-01-01T00:00:00Z", "2026-01-01T00:00:00Z", bodies=["saturn"])
    raw = execute({"op": "ashtakavarga_transits", "bodies": ["saturn"], "start_utc": "2025-01-01T00:00:00Z",
                   "end_utc": "2026-01-01T00:00:00Z",
                   "natal": {"when": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata"}, "system": "vedic",
                             "place": {"latitude": 28.6139, "longitude": 77.2090},
                             "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}})
    cli = json.loads(subprocess.check_output([os.environ.get("ESHANA_BIN", "eshana"), "--compact", "ashtakavarga-transits",
                                              "--start", "2025-01-01T00:00:00Z", "--end", "2026-01-01T00:00:00Z",
                                              "--transit-bodies", "saturn", "--date", "1990-05-15", "--time", "14:30",
                                              "--tz", "Asia/Kolkata", "--lat", "28.6139", "--lon", "77.2090",
                                              "--ayanamsha", "lahiri"]))
    assert py == raw == cli
