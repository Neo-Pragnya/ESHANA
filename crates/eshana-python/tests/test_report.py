"""HTML report: structure, self-containment, honesty about missing inputs, and chart geometry."""
import json
import re

import pytest

from eshana import ChartEngine, Context
from eshana import report as R

CTX = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)


@pytest.fixture(scope="module")
def page():
    return R.birth_report(CTX, name="Tester", place_label="Delhi")


def test_every_section_is_present(page):
    for sid in ("summary", "chakra", "vedic", "divisional", "dasha", "nakshatra", "western", "human_design",
                "numerology", "provenance", "raw"):
        assert f'id="{sid}"' in page, sid


def test_self_contained_no_network(page):
    assert "<link" not in page
    assert not re.search(r'(src|href)="https?://', page)
    assert "@import" not in page


def test_embedded_json_round_trips_and_matches_engine(page):
    raw = re.search(r'<script type="application/json" id="eshana-data">(.*?)</script>', page, re.S).group(1)
    data = json.loads(raw.replace("<\\/", "</"))
    direct = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", divisions=[1, 3, 4, 7, 9, 10, 12], dasha=True).raw
    assert data["vedic"]["positions"] == direct["positions"]
    assert data["vedic"]["interpretations"] == []


def test_user_text_is_escaped():
    page = R.birth_report(CTX, name="<script>alert(1)</script>", systems=["numerology"])
    assert "<script>alert(1)</script>" not in page
    assert "&lt;script&gt;" in page


def test_unknown_time_lists_refusals_and_skips_lagna_charts():
    ctx = Context.from_local_birth("1990-05-15", None, "Asia/Kolkata", 28.6139, 77.2090)
    page = R.birth_report(ctx)
    assert "Not computed" in page and "ESH-TIME-UNCERTAIN" in page
    assert "North Indian)" not in page
    assert "omitted rather than guessed" in page


def test_sections_filter_and_unknown_section():
    env = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", dasha=True)
    html = env.to_html(sections=["summary", "dasha"])
    assert 'id="dasha"' in html and 'id="chakra"' not in html
    with pytest.raises(ValueError):
        env.to_html(sections=["horoscope"])


def test_south_indian_layout_puts_pisces_top_left_and_marks_lagna():
    svg = R.south_indian_svg({11: [("sun", "Su", "1°")]}, 11, "T")
    first_rect = re.search(r'<rect x="(\d+)" y="(\d+)"[^>]*><title>Meena', svg)
    assert first_rect and first_rect.groups() == ("0", "0")
    assert ">As<" in svg and 'data-body="sun"' in svg


def test_north_indian_house_one_holds_the_lagna_sign():
    svg = R.north_indian_svg({}, 5, "T")  # Kanya lagna
    assert "<title>House 1: Kanya</title>" in svg
    assert "<title>House 7: Meena</title>" in svg


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
                   "--systems", "vedic", "-o", str(out)]) == 0
    assert out.read_text().startswith("<!doctype html>")


def test_cli_refuses_without_time_input(capsys):
    assert R.main(["-o", "x.html"]) == 2
    assert "give --date" in capsys.readouterr().err
