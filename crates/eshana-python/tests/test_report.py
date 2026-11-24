"""HTML chart studio: structure, self-containment, data completeness, honesty about missing inputs, SVG helpers,
and the new engine surfaces it uses (features, panchanga, matching, ephemeris, ingresses, rule packs)."""
import json
import re

import pytest

from eshana import ChartEngine, Context, EshanaError
from eshana import report as R

CTX = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)
REF = "2026-11-21T00:00:00Z"


def data_of(page):
    raw = re.search(r'<script type="application/json" id="eshana-data">(.*?)</script>', page, re.S).group(1)
    return json.loads(raw.replace("<\\/", "</"))


@pytest.fixture(scope="module")
def page():
    partner = Context.from_local_birth("1992-08-20", "09:15", "Asia/Kolkata", 19.076, 72.8777)
    return R.birth_report(CTX, name="Tester", place_label="Delhi", reference_utc=REF, partner=partner)


def test_every_section_shell_is_present(page):
    for sid in R.SECTIONS:
        assert f'id="{sid}"' in page, sid


def test_self_contained_no_network(page):
    assert "<link" not in page
    assert not re.search(r'(src|href)="https?://', page)
    assert "@import" not in page
    assert "fetch(" not in page and "XMLHttpRequest" not in page


def test_embedded_data_is_complete(page):
    d = data_of(page)
    env = d["envelopes"]
    assert set(env) >= {"vedic", "western", "human_design", "numerology", "matching"}
    ids = {a["id"] for a in env["vedic"]["derived_attributes"]}
    for need in ("vedic.panchanga", "vedic.strength", "vedic.relationships", "vedic.drishti.graha", "vedic.drishti.rashi",
                 "vedic.ashtakavarga", "vedic.jaimini.chara_karakas", "vedic.jaimini.arudha_padas", "vedic.kp",
                 "vedic.yogas", "vedic.sade_sati", "vedic.dasha.vimshottari", "vedic.dasha.yogini", "vedic.moon.d60"):
        assert need in ids, need
    assert d["reference_utc"] == REF
    assert len(d["transits"]["rows"]) > 100
    assert len(d["progressions"]["rows"]) == 91
    assert len(d["nakshatra_reference"]) == 27
    assert env["vedic"]["interpretations"] == []


def test_matches_the_engine_directly(page):
    direct = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", divisions=[1, 9]).raw
    assert data_of(page)["envelopes"]["vedic"]["positions"] == direct["positions"]


def test_user_text_is_escaped():
    page = R.birth_report(CTX, name="<script>alert(1)</script>", systems=["numerology"], transit_days=0,
                          progression_years=0)
    assert "<script>alert(1)</script>" not in page
    assert "&lt;script&gt;" in page


def test_unknown_time_lists_refusals():
    ctx = Context.from_local_birth("1990-05-15", None, "Asia/Kolkata", 28.6139, 77.2090)
    page = R.birth_report(ctx, reference_utc=REF)
    assert "Not computed" in page and "ESH-TIME-UNCERTAIN" in page
    env = data_of(page)["envelopes"]["vedic"]
    assert env["houses"] is None
    assert not any(a["id"] == "vedic.ashtakavarga" for a in env["derived_attributes"])
    assert data_of(page)["progressions"] is None


def test_sections_filter_and_unknown_section():
    chart = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", dasha=True)
    html = chart.to_html(sections=["overview", "dasha"])
    assert 'id="dasha"' in html and 'id="chakra"' not in html
    with pytest.raises(ValueError):
        chart.to_html(sections=["horoscope"])


def test_svg_helpers_still_work():
    svg = R.south_indian_svg({11: [("sun", "Su", "1°")]}, 11, "T")
    first = re.search(r'<rect x="(\d+)" y="(\d+)"[^>]*><title>Meena', svg)
    assert first and first.groups() == ("0", "0")
    assert "<title>House 1: Kanya</title>" in R.north_indian_svg({}, 5, "T")


@pytest.mark.parametrize("lon,expect", [
    (0.1, {"vargottama", "gandanta"}),             # Aries pada 1: movable sign, 1st navamsha; water-fire junction
    (21.0, {"pushkara navamsha"}),                 # Aries 7th navamsha (fire: 7th and 9th)
    (25.0, set()),                                 # Aries 8th navamsha
    (45.0, {"pushkara navamsha", "vargottama"}),   # Taurus 5th navamsha: fixed sign 5th, earth 5th
    (359.9, {"vargottama", "gandanta"}),           # Pisces 9th navamsha: dual sign 9th; junction
])
def test_markers_follow_the_template_rules(lon, expect):
    assert set(R.markers(lon)) == expect


def test_cli_writes_file(tmp_path):
    out = tmp_path / "c.html"
    assert R.main(["--date", "2000-01-01", "--time", "12:00", "--tz", "UTC", "--lat", "51.5", "--lon", "0",
                   "--systems", "vedic", "--icons", "glyph", "--reference", REF, "-o", str(out)]) == 0
    text = out.read_text()
    assert text.startswith("<!doctype html>")
    assert data_of(text)["defaults"] == {"icons": "glyph"}


def test_cli_refuses_without_time_input(capsys):
    assert R.main(["-o", "x.html"]) == 2
    assert "give --date" in capsys.readouterr().err


# ── the engine surfaces behind the studio ─────────────────────────────────────────────────────────────────────────
def test_panchanga_method():
    p = ChartEngine().panchanga(CTX)["panchanga"]
    assert p["vara"]["english"] == "Tuesday"
    assert p["tithi"]["name"] and p["karana"]["name"] and p["yoga"]["name"]
    assert p["sunrise"]["local"].startswith("1990-05-15T05:3")


def test_matching_method_is_bounded_and_symmetric_in_nadi():
    a = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.61, 77.21)
    b = Context.from_local_birth("1992-08-20", "09:15", "Asia/Kolkata", 19.08, 72.88)
    r1, r2 = ChartEngine().matching(a, b), ChartEngine().matching(b, a)
    assert 0 <= r1["ashtakoota"]["total"] <= 36
    assert r1["ashtakoota"]["nadi_dosha"] == r2["ashtakoota"]["nadi_dosha"]


def test_ephemeris_and_ingresses():
    e = ChartEngine().ephemeris("2026-01-01T00:00:00Z", 10, 5, bodies=["sun"], zodiac="sidereal", ayanamsha="lahiri")
    assert len(e["rows"]) == 5
    i = ChartEngine().ingresses("sun", "2026-01-01T00:00:00Z", "2027-01-01T00:00:00Z")
    assert len(i["ingresses"]) == 12


def test_stations_and_crossings_match_the_raw_request():
    from eshana import execute

    s = ChartEngine().stations("mercury", "2024-01-01T00:00:00Z", "2025-01-01T00:00:00Z", zodiac="tropical")
    raw = execute({"op": "stations", "body": "mercury", "start_utc": "2024-01-01T00:00:00Z",
                   "end_utc": "2025-01-01T00:00:00Z", "config": {"zodiac": "tropical"}})
    assert s == raw
    assert [x["station"] for x in s["stations"][:2]] == ["direct", "retrograde"]
    c = ChartEngine().crossings("saturn", 335.0, "2024-01-01T00:00:00Z", "2027-01-01T00:00:00Z", touch_orb_deg=0.0)
    assert [e["direction"] for e in c["events"]] == ["direct", "retrograde", "direct"]


def test_rulesets_are_yaml_and_listed():
    packs = {p["file"] for p in ChartEngine.rulesets()["packs"]}
    assert packs == {"vedic-core.yaml", "vedic-dignity.yaml", "vedic-relationships.yaml", "vedic-drishti.yaml",
                     "vedic-strength.yaml", "bala.yaml", "ashtakavarga.yaml", "panchanga.yaml", "matching.yaml", "yogas.yaml",
                     "western-dignities.yaml", "gematria.yaml", "lal-kitab.yaml"}


def test_features_are_vedic_only():
    from eshana import execute
    with pytest.raises(EshanaError) as e:
        execute({"op": "chart", "system": "western", "when": CTX.when, "place": CTX.place,
                 "config": {"zodiac": "tropical"}, "features": ["kp"]})
    assert e.value.code == "ESH-CONFIG"


def test_unknown_feature_is_refused():
    with pytest.raises(EshanaError) as e:
        ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", features=["horoscope"])
    assert e.value.code == "ESH-INPUT"


def test_studio_assets_share_no_top_level_names():
    """The studio assets are joined into one script, so a top-level name declared in two of them breaks every page."""
    import re
    from eshana import report

    seen: dict[str, str] = {}
    clashes = []
    for name in report.ASSETS:
        for m in re.finditer(r"^(?:const|let|var|function|class)\s+([A-Za-z_$][\w$]*)", report._asset(name), re.M):
            if m.group(1) in seen:
                clashes.append((m.group(1), seen[m.group(1)], name))
            seen.setdefault(m.group(1), name)
    assert not clashes, clashes


def test_every_saham_and_pakshi_activity_has_a_meaning(page):
    """The studio's saham and pañca pakṣī info boxes read their interpretation text from reading_reference.json:
    every saham the engine emits has a meaning and a varṣaphala use, every bird activity is explained, and the text
    names its sources (it is interpretation, kept out of the engine)."""
    d = data_of(page)
    ref = d["reading"]
    attrs = {a["id"]: a["value"] for a in d["envelopes"]["vedic"]["derived_attributes"]}
    ids = [r["id"] for r in attrs["vedic.aux.sahams"]["sahams"]]
    assert len(ids) == 37
    items = ref["sahams"]["items"]
    assert set(items) == set(ids)
    for k in ids:
        assert len(items[k]["signifies"]) > 10 and len(items[k]["used_for"]) > 10, k
    assert ref["sahams"]["sources"] and ref["sahams"]["general"]["what"] and len(ref["sahams"]["general"]["read"]) >= 3
    pk = ref["pakshi"]
    assert set(pk["activities"]) == {"ruling", "eating", "walking", "sleeping", "dying"}
    assert pk["sources"] and all(pk[k] for k in ("what", "birth_bird", "relations", "ruling", "dying", "strength"))
    assert [n["offset"] for n in attrs["vedic.panch_pakshi"]["nearby_days"]] == [-3, -2, -1, 1, 2, 3]


def test_gender_picks_the_kua_and_is_validated():
    import eshana
    import pytest

    e = eshana.ChartEngine()
    r = e.numerology(name="Ada Lovelace", date_of_birth="1990-05-15", extended=True, gender="female")
    kua = [a for a in r["derived_attributes"] if a["id"] == "numerology.lo_shu"][0]["value"]["kua"]
    assert r["input"]["gender"] == "female"
    assert (kua["number"], kua["group"]) == (8, "west")
    with pytest.raises(eshana.EshanaError):
        e.numerology(name="X", date_of_birth="1990-05-15", extended=True, gender="other")
