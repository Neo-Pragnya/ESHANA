"""Attributed content and naming adapters (RES-04), kept outside the engine.

The engine computes facts and leaves ``interpretations`` empty. Text about what a fact means (a reading of a yoga, a
description of a point) lives in **content packs**: JSON files whose every entry is keyed by an immutable factor id
the engine produces, with the attribution of the text. :func:`attach` joins a pack to an envelope *beside* it, so the
envelope itself is never changed and the attribution travels with every piece of text.

Factor ids are the engine's own: a derived attribute id (``vedic.ashtakavarga``) or a rule inside one
(``vedic.yogas#gaja_kesari``, ``vedic.points#indu_lagna``). :func:`known_factor_ids` lists them from the engine's rule
packs, so a pack that refers to something the engine does not compute is refused.

A **naming adapter** maps the engine's ids to display names (IAST, English, Devanāgarī) and back, one-to-one.
"""
from __future__ import annotations

import copy
import json
import re
from importlib import resources
from typing import Any, Dict, Iterable, List, Optional

from . import ChartEngine

SCHEMA = "eshana.content@1"


class ContentError(ValueError):
    """A content pack that cannot be used."""


# The studio's point pages group engine attributes for reading; each point id names the attribute it comes from.
POINT_SOURCES = {
    "lagna": "vedic.lagna.rashi", "bhava_lagna": "vedic.panchanga", "hora_lagna": "vedic.panchanga",
    "ghati_lagna": "vedic.panchanga", "indu_lagna": "vedic.special_points", "yogi_point": "vedic.special_points",
    "avayogi": "vedic.special_points", "duplicate_yogi": "vedic.special_points", "dagdha_rashis": "vedic.special_points",
    "navamsha_64th": "vedic.special_points", "drekkana_22nd": "vedic.special_points", "functional_nature": "vedic.special_points",
    "arudha_lagna": "vedic.jaimini.arudha_padas", "upapada": "vedic.jaimini.arudha_padas", "arudha_padas": "vedic.jaimini.arudha_padas",
    "chara_karakas": "vedic.jaimini.chara_karakas", "karakamsa": "vedic.jaimini.chara_karakas", "upagrahas": "vedic.upagrahas",
}


def known_factor_ids() -> set:
    """Every rule-level factor id the engine can produce: yogas and doshas from the yogas pack, plus the attribute
    ids of a full Vedic chart."""
    ids = set()
    for p in ChartEngine.rulesets()["packs"]:
        if p["file"] == "yogas.yaml":
            for yid in re.findall(r"^\s+- id: ([a-z0-9_]+)", p["yaml"], re.M):
                ids.add(f"vedic.yogas#{yid}")
    from . import execute
    env = execute({"op": "chart", "system": "vedic", "features": ["all"], "when": {"utc": "2000-01-01T12:00:00Z"},
                   "place": {"latitude": 28.61, "longitude": 77.21}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}})
    for a in env["derived_attributes"]:
        ids.add(a["id"])
    sp = next((a["value"] for a in env["derived_attributes"] if a["id"] == "vedic.special_points"), {}) or {}
    for k in sp:
        ids.add(f"vedic.points#{k}")
    attr_ids = {a["id"] for a in env["derived_attributes"]}
    for k, src in POINT_SOURCES.items():
        if src in attr_ids:  # a point is known only when the attribute it comes from is computed
            ids.add(f"vedic.points#{k}")
    return ids


def validate(pack: Dict[str, Any], known: Optional[set] = None) -> List[str]:
    """Problems with a pack: wrong schema, no attribution, or entries keyed by ids the engine does not produce."""
    problems = []
    if pack.get("schema") != SCHEMA:
        problems.append(f"schema is not {SCHEMA}")
    att = pack.get("attribution") or {}
    if not att.get("source") or not att.get("license"):
        problems.append("attribution needs a source and a license")
    known = known if known is not None else known_factor_ids()
    for fid in pack.get("entries", {}):
        if fid not in known:
            problems.append(f"unknown factor id '{fid}'")
    return problems


def from_reading_reference() -> Dict[str, Any]:
    """The studio's bundled reading notes (yogas and points) as a content pack."""
    ref = json.loads(resources.files("eshana").joinpath("data/reading_reference.json").read_text(encoding="utf-8"))
    entries = {f"vedic.yogas#{k}": v for k, v in ref.get("yogas", {}).items()}
    entries.update({f"vedic.points#{k}": v for k, v in ref.get("points", {}).items()})
    return {"schema": SCHEMA, "id": "eshana.reading_reference", "version": "0.1",
            "attribution": {"source": ref["meta"]["attribution"], "license": "Apache-2.0 (ESHANA documentation)"},
            "epistemic": ref["meta"].get("epistemic", "interpretation"), "entries": entries}


def facts_in(env: Dict[str, Any]) -> List[str]:
    """The factor ids present in an envelope: attribute ids, and the yogas and doshas whose conditions hold."""
    out = []
    for a in env.get("derived_attributes", []):
        out.append(a["id"])
        if a["id"] == "vedic.yogas":
            for group in ("yogas", "doshas"):
                for y in a["value"].get(group, []):
                    if y.get("present"):
                        out.append(f"vedic.yogas#{y['id']}")
    return out


def attach(env: Dict[str, Any], packs: Iterable[Dict[str, Any]]) -> Dict[str, Any]:
    """Content for the facts in ``env``, beside it. The envelope is returned unchanged (a copy); each piece of text
    carries its factor id, its pack and its attribution."""
    present = set(facts_in(env))
    content = []
    for p in packs:
        for fid, text in p.get("entries", {}).items():
            if fid in present:
                content.append({"factor_id": fid, "pack": p.get("id"), "epistemic": p.get("epistemic", "interpretation"),
                                "attribution": p["attribution"], "text": text})
    return {"envelope": copy.deepcopy(env), "content": content}


# ── naming adapters ──────────────────────────────────────────────────────────────────────────────────────────────
NAMES = {
    "iast": {"sun": "Sūrya", "moon": "Candra", "mars": "Maṅgala", "mercury": "Budha", "jupiter": "Guru", "venus": "Śukra",
             "saturn": "Śani", "mean_node": "Rāhu", "true_node": "Rāhu", "south_node": "Ketu"},
    "english": {"sun": "Sun", "moon": "Moon", "mars": "Mars", "mercury": "Mercury", "jupiter": "Jupiter", "venus": "Venus",
                "saturn": "Saturn", "mean_node": "North node", "true_node": "North node (true)", "south_node": "South node"},
    "devanagari": {"sun": "सूर्य", "moon": "चन्द्र", "mars": "मंगल", "mercury": "बुध", "jupiter": "गुरु", "venus": "शुक्र",
                   "saturn": "शनि", "mean_node": "राहु", "true_node": "राहु", "south_node": "केतु"},
}
SIGNS = {
    "iast": ["Meṣa", "Vṛṣabha", "Mithuna", "Karka", "Siṃha", "Kanyā", "Tulā", "Vṛścika", "Dhanu", "Makara", "Kumbha", "Mīna"],
    "english": ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces"],
    "devanagari": ["मेष", "वृषभ", "मिथुन", "कर्क", "सिंह", "कन्या", "तुला", "वृश्चिक", "धनु", "मकर", "कुम्भ", "मीन"],
}


class Naming:
    """Display names for engine ids in one scheme (``iast``, ``english``, ``devanagari``). Display only: requests
    and results always use the engine ids."""

    def __init__(self, scheme: str = "iast"):
        if scheme not in NAMES:
            raise ContentError(f"unknown naming scheme '{scheme}' (one of: {', '.join(NAMES)})")
        self.scheme = scheme

    def body(self, engine_id: str) -> str:
        return NAMES[self.scheme].get(engine_id, engine_id)

    def sign(self, index: int) -> str:
        return SIGNS[self.scheme][index % 12]

    def sign_index(self, name: str) -> int:
        return SIGNS[self.scheme].index(name)
