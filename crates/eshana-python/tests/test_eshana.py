"""Python facade tests, including cross-language parity with the CLI and schema validation."""

import json
import os
import pathlib
import subprocess

import jsonschema
import pytest

import eshana
from eshana import ChartEngine, Context, EshanaError

ROOT = pathlib.Path(__file__).resolve().parents[3]
SCHEMA = json.loads((ROOT / "schemas" / "envelope.schema.json").read_text())
CLI = pathlib.Path(os.environ.get("ESHANA_BIN", ROOT / "target" / "debug" / "eshana"))
needs_cli = pytest.mark.skipif(not CLI.exists(), reason="build the CLI first: cargo build -p eshana-cli")

CTX = Context.from_local_birth(
    date="1992-05-28", time="20:05:00", timezone="Asia/Kolkata", latitude=17.385, longitude=78.487
)


def cli(*args):
    r = subprocess.run([str(CLI), "--compact", *args], capture_output=True, text=True, check=False)
    return r.returncode, json.loads(r.stdout) if r.stdout.strip().startswith("{") else r.stdout


def test_version_matches_the_crate():
    assert eshana.__version__ == "0.4.0"
    assert eshana.EXPANSION.startswith("Extensible Spatiotemporal Hypersystem")


def test_vedic_chart_reads_naturally():
    chart = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
    assert chart.position("moon")["sign"]["name"] == "Meena"
    assert chart.attribute("vedic.moon.nakshatra")["name"] == "Revati"
    assert chart.attribute("vedic.lagna.rashi")["name"] == "Dhanu"
    assert chart.attribute("vedic.dasha.vimshottari")["starting_lord"] == "Mercury"
    assert chart.attribute_full("vedic.moon.d9")["rule_version"] == "vedic.parashari@0.3"
    assert chart.provenance["rulesets"]
    assert chart.provenance["assumptions"]


def test_ayanamsha_is_required_for_vedic():
    with pytest.raises(TypeError):
        ChartEngine().vedic_chart(CTX)  # type: ignore[call-arg]


def test_sidereal_without_ayanamsha_is_refused_with_a_code():
    with pytest.raises(EshanaError) as e:
        ChartEngine().positions(CTX, zodiac="sidereal")
    assert e.value.code == "ESH-CONFIG"


def test_western_chart_with_aspects():
    chart = ChartEngine().western_chart(CTX, houses="placidus")
    assert chart.houses and chart.houses["system"] == "placidus"
    assert any(a["id"].startswith("western.aspect.") for a in chart.derived)


def test_unknown_birth_time_never_invents_houses():
    ctx = Context.from_local_birth(date="1992-05-28", timezone="Asia/Kolkata", latitude=17.385, longitude=78.487)
    chart = ChartEngine().western_chart(ctx)
    assert chart.houses is None
    assert chart.warnings[0]["code"] == "ESH-TIME-UNCERTAIN"


def test_human_design_shape():
    hd = ChartEngine().human_design(CTX)
    bg = hd["derived_attributes"][0]["value"]
    assert bg["type"] in {"generator", "manifesting_generator", "manifestor", "projector", "reflector"}
    assert len(hd["positions"]["design"]["activations"]) == 13


def test_numerology():
    r = ChartEngine().numerology(name="Ada Lovelace", date_of_birth="1815-12-10", reference_year=2026)
    assert [a["id"] for a in r["derived_attributes"]][0] == "numerology.name"


def test_graph_formats():
    e = ChartEngine()
    g = e.graph(CTX, "vedic", ayanamsha="lahiri", zodiac="sidereal")
    assert g["kind"] == "symbolic_graph" and g["nodes"]
    assert e.graph(CTX, "vedic", fmt="mermaid", ayanamsha="lahiri", zodiac="sidereal").startswith("flowchart LR")


def test_compare_lists_configuration_differences():
    r = ChartEngine().compare(CTX, "vedic", a={}, b={"ayanamsha": "raman"}, zodiac="sidereal", ayanamsha="lahiri")
    assert r["configuration_differences"] == ["ayanamsha"]


def test_info_has_error_catalogue():
    info = ChartEngine.info()
    assert {e["code"] for e in info["error_codes"]} >= {"ESH-CONFIG", "ESH-LATITUDE", "ESH-TIME-AMBIGUOUS"}


@pytest.mark.parametrize(
    "request_",
    [
        {"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "config": {"zodiac": "tropical"}},
        {"op": "chart", "system": "vedic", "assume_defaults": True, "when": CTX.when, "place": CTX.place, "divisions": [1, 9], "dasha": True},
        {"op": "chart", "system": "western", "aspects": True, "when": CTX.when, "place": CTX.place, "config": {"zodiac": "tropical", "house_system": "placidus"}},
        {"op": "human_design", "when": CTX.when, "place": CTX.place},
        {"op": "numerology", "name": "Ada Lovelace", "date_of_birth": "1815-12-10", "reference_year": 2026},
        {"op": "info"},
    ],
)
def test_every_envelope_validates_against_the_schema(request_):
    out = eshana.execute(request_)
    jsonschema.validate(out, SCHEMA)


def test_error_envelopes_validate_too():
    raw = json.loads(eshana._native.execute(json.dumps({"op": "positions", "when": {"utc": "1500-01-01T00:00:00Z"}})))
    jsonschema.validate(raw, SCHEMA)
    assert raw["error"]["code"] == "ESH-RANGE"


@needs_cli
def test_cli_and_python_return_identical_json():
    py = eshana.execute(
        {"op": "chart", "system": "vedic", "assume_defaults": True, "when": CTX.when, "place": CTX.place, "divisions": [1, 9], "dasha": True}
    )
    code, out = cli("chart", "vedic", "--date", "1992-05-28", "--time", "20:05:00", "--tz", "Asia/Kolkata",
                    "--lat", "17.385", "--lon", "78.487", "--divisions", "1,9", "--dasha")
    assert code == 0
    assert out == py


@needs_cli
def test_cli_numerology_matches_python():
    py = ChartEngine().numerology(name="Ada Lovelace", date_of_birth="1815-12-10", reference_year=2026)
    code, out = cli("numerology", "--name", "Ada Lovelace", "--dob", "1815-12-10", "--year", "2026")
    assert code == 0 and out == py


@needs_cli
def test_cli_error_exits_2_with_an_envelope():
    code, out = cli("positions", "--utc", "1500-01-01T00:00:00Z")
    assert code == 2 and out["error"]["code"] == "ESH-RANGE"
