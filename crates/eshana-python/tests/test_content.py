"""Attributed content stays beside the engine's output, keyed by factor ids the engine really produces."""
import pytest

from eshana import ChartEngine, Context
from eshana import content as C

CTX = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)


@pytest.fixture(scope="module")
def known():
    return C.known_factor_ids()


def test_bundled_reading_notes_are_a_valid_attributed_pack(known):
    pack = C.from_reading_reference()
    assert C.validate(pack, known) == []
    assert len(pack["entries"]) >= 40


def test_unknown_ids_and_missing_attribution_are_refused(known):
    bad = {"schema": C.SCHEMA, "attribution": {"source": "x"}, "entries": {"vedic.yogas#no_such_yoga": "…"}}
    problems = C.validate(bad, known)
    assert any("license" in p for p in problems) and any("no_such_yoga" in p for p in problems)


def test_attach_never_touches_the_envelope():
    env = ChartEngine().vedic_chart(CTX, ayanamsha="lahiri", features=["yogas"]).raw
    before = repr(env)
    out = C.attach(env, [C.from_reading_reference()])
    assert repr(env) == before
    assert out["envelope"]["interpretations"] == [] and out["envelope"] == env
    present = set(C.facts_in(env))
    assert out["content"] and all(c["factor_id"] in present and c["attribution"]["source"] for c in out["content"])


def test_naming_is_one_to_one_for_signs():
    for scheme in ("iast", "english", "devanagari"):
        n = C.Naming(scheme)
        assert [n.sign_index(n.sign(i)) for i in range(12)] == list(range(12))
        assert n.body("saturn") != "saturn"
    with pytest.raises(C.ContentError):
        C.Naming("klingon")
