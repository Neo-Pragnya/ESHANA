"""ESHANA as a Model Context Protocol server (BIND-05): `eshana-mcp` speaks MCP over stdio (newline-delimited
JSON-RPC 2.0), so an assistant or an agent can compute charts with the engine on the user's own machine.

Tools are thin request builders over `eshana.execute`: the engine computes, names every rule, refuses what it
cannot know (with a stable ESH-* code) and never interprets. Nothing leaves the machine: no network, no files.

    eshana-mcp                 # register it with an MCP client as a stdio server
"""
from __future__ import annotations

import json
import sys
from typing import Any, Callable, Dict, List, Optional, Tuple

from . import EshanaError, engine_version, execute

PROTOCOL_VERSIONS = ("2025-06-18", "2025-03-26", "2024-11-05")

_WHEN = {
    "date": {"type": "string", "description": "local date YYYY-MM-DD (a signed year such as -0043-03-15 is BCE)"},
    "time": {"type": "string", "description": "local time HH:MM[:SS]; omit for an unknown birth time"},
    "timezone": {"type": "string", "description": "IANA zone, e.g. Asia/Kolkata (ESHANA never assumes one)"},
    "utc_offset_minutes": {"type": "integer", "description": "fixed offset east of UTC, when no zone is known"},
    "utc": {"type": "string", "description": "a UTC instant instead of date/time, e.g. 2026-11-24T12:00:00Z"},
    "calendar": {"type": "string", "enum": ["gregorian", "julian", "auto"]},
}
_PLACE = {"latitude": {"type": "number"}, "longitude": {"type": "number"}}


def _schema(props: Dict[str, Any], required: List[str]) -> Dict[str, Any]:
    return {"type": "object", "properties": props, "required": required, "additionalProperties": False}


def _when(a: Dict[str, Any]) -> Dict[str, Any]:
    return {k: a[k] for k in ("date", "time", "timezone", "utc_offset_minutes", "utc", "calendar") if a.get(k) is not None}


def _place(a: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    return {"latitude": a["latitude"], "longitude": a["longitude"]} if a.get("latitude") is not None else None


def _chart(a: Dict[str, Any], system: Optional[str], cfg: Dict[str, Any], **extra: Any) -> Dict[str, Any]:
    req = {"when": _when(a), "place": _place(a), "config": {k: v for k, v in cfg.items() if v is not None}, **extra}
    if system:
        req["system"] = system
    return {k: v for k, v in req.items() if v not in (None, [], {})}


# name -> (description, input schema, request builder)
TOOLS: Dict[str, Tuple[str, Dict[str, Any], Callable[[Dict[str, Any]], Dict[str, Any]]]] = {
    "compute_positions": (
        "Apparent geocentric positions of the Sun, Moon and planets (tropical, or sidereal with an ayanamsha).",
        _schema({**_WHEN, "bodies": {"type": "array", "items": {"type": "string"}}, "zodiac": {"type": "string", "enum": ["tropical", "sidereal"]},
                 "ayanamsha": {"type": "string"}}, []),
        lambda a: {"op": "positions", **_chart(a, None, {"zodiac": a.get("zodiac"), "ayanamsha": a.get("ayanamsha")}, bodies=a.get("bodies"))},
    ),
    "create_vedic_chart": (
        "A sidereal Vedic chart: grahas, lagna, vargas, and on request Vedic features (panchanga, strength, drishti, "
        "ashtakavarga, jaimini, kp, yogas, ... or 'all'). Every value names its rule; interpretations are never invented.",
        _schema({**_WHEN, **_PLACE, "ayanamsha": {"type": "string", "default": "lahiri"},
                 "features": {"type": "array", "items": {"type": "string"}}, "divisions": {"type": "array", "items": {"type": "integer"}},
                 "dasha": {"type": "boolean"}}, ["date", "timezone"]),
        lambda a: {"op": "chart", **_chart(a, "vedic", {"zodiac": "sidereal", "ayanamsha": a.get("ayanamsha", "lahiri")},
                                           features=a.get("features"), divisions=a.get("divisions"), dasha=a.get("dasha"))},
    ),
    "create_western_chart": (
        "A tropical Western chart with houses, Ptolemaic aspects and, on request, essential dignities.",
        _schema({**_WHEN, **_PLACE, "house_system": {"type": "string", "default": "placidus"}, "aspects": {"type": "boolean", "default": True},
                 "dignities": {"type": "boolean"}}, ["date", "timezone"]),
        lambda a: {"op": "chart", **_chart(a, "western", {"zodiac": "tropical", "house_system": a.get("house_system", "placidus")},
                                           aspects=a.get("aspects", True), features=["dignities"] if a.get("dignities") else None)},
    ),
    "compare_charts": (
        "The same birth under two configurations (for example two ayanamshas or two house systems), with the differences.",
        _schema({**_WHEN, **_PLACE, "system": {"type": "string", "enum": ["vedic", "western"]},
                 "a": {"type": "object", "description": "config overrides for side A, e.g. {\"ayanamsha\": \"lahiri\"}"},
                 "b": {"type": "object", "description": "config overrides for side B"}}, ["date", "timezone", "a", "b"]),
        lambda a: {"op": "compare", "base": _chart(a, a.get("system", "vedic"), {"zodiac": "sidereal" if a.get("system", "vedic") == "vedic" else "tropical"}),
                   "a": a["a"], "b": a["b"]},
    ),
    "compute_human_design": (
        "Human Design: design instant by the 88-degree solar arc, gates, lines, channels, type, authority and profile (exact birth time required).",
        _schema({**_WHEN}, ["date", "time", "timezone"]),
        lambda a: {"op": "human_design", "when": _when(a)},
    ),
    "compute_numerology": (
        "Name and birth-date numerology under a named school (pythagorean, chaldean; hebrew_gematria or abjad for names in Hebrew or Arabic letters).",
        _schema({"name": {"type": "string"}, "date_of_birth": {"type": "string"}, "school": {"type": "string", "default": "pythagorean"}}, []),
        lambda a: {"op": "numerology", **{k: a[k] for k in ("name", "date_of_birth", "school") if a.get(k)}},
    ),
    "panchanga": (
        "Pañcāṅga for a moment and place: tithi, vāra, nakshatra, yoga, karaṇa, sunrise and sunset, horā, choghaḍiyā, Rāhu kālam.",
        _schema({**_WHEN, **_PLACE, "ayanamsha": {"type": "string", "default": "lahiri"}}, ["date", "timezone", "latitude", "longitude"]),
        lambda a: {"op": "panchanga", "chart": _chart(a, None, {"zodiac": "sidereal", "ayanamsha": a.get("ayanamsha", "lahiri")})},
    ),
    "find_eclipses": (
        "Solar and lunar eclipses between two UTC instants: type, time of greatest eclipse and magnitude.",
        _schema({"start_utc": {"type": "string"}, "end_utc": {"type": "string"}, "kinds": {"type": "array", "items": {"type": "string", "enum": ["solar", "lunar"]}}},
                ["start_utc", "end_utc"]),
        lambda a: {"op": "eclipses", **{k: a[k] for k in ("start_utc", "end_utc", "kinds") if a.get(k)}},
    ),
    "rise_and_set": (
        "Rise, upper transit and set of bodies (default Sun and Moon), and twilight, for local days at a place.",
        _schema({**_PLACE, "date": {"type": "string"}, "timezone": {"type": "string"}, "days": {"type": "integer", "default": 1},
                 "bodies": {"type": "array", "items": {"type": "string"}}}, ["latitude", "longitude", "date", "timezone"]),
        lambda a: {"op": "rise_set", "place": _place(a), **{k: a[k] for k in ("date", "timezone", "days", "bodies") if a.get(k) is not None}},
    ),
    "explain_derivation": (
        "How one derived value was made: the attribute (by id, e.g. vedic.moon.nakshatra), the rule and version that "
        "made it, the positions and houses it was derived from, and the chart's calculation and provenance.",
        _schema({**_WHEN, **_PLACE, "attribute_id": {"type": "string"}, "system": {"type": "string", "enum": ["vedic", "western"]},
                 "ayanamsha": {"type": "string", "default": "lahiri"}, "features": {"type": "array", "items": {"type": "string"}},
                 "divisions": {"type": "array", "items": {"type": "integer"}}, "dasha": {"type": "boolean"}},
                ["date", "timezone", "attribute_id"]),
        lambda a: {"op": "chart", **(_chart(a, "vedic", {"zodiac": "sidereal", "ayanamsha": a.get("ayanamsha", "lahiri")},
                                            features=a.get("features"), divisions=a.get("divisions"), dasha=a.get("dasha"))
                                     if a.get("system", "vedic") == "vedic" else
                                     _chart(a, "western", {"zodiac": "tropical"}, features=a.get("features")))},
    ),
    "eshana_request": (
        "Any ESHANA operation as a raw request object ({\"op\": ..., ...}; see the request reference): returns the envelope.",
        _schema({"request": {"type": "object"}}, ["request"]),
        lambda a: a["request"],
    ),
}


def _explain(a: Dict[str, Any], env: Dict[str, Any]) -> Dict[str, Any]:
    """Pick one attribute out of a chart envelope with everything it was derived from."""
    want = a["attribute_id"]
    attrs = {x["id"]: x for x in env.get("derived_attributes", [])}
    if want not in attrs:
        near = sorted(i for i in attrs if want.split(".")[-1] in i)[:20] or sorted(attrs)[:40]
        raise ValueError(f"no attribute {want!r} in this chart (a feature may need asking for); some that exist: {near}")
    at = attrs[want]
    srcs = set(at.get("derived_from") or [])
    pos = [p for p in env.get("positions", []) if p.get("id") in srcs]
    houses = env.get("houses") if "houses" in srcs or any(str(x).startswith("house") for x in srcs) else None
    return {"kind": "derivation", "attribute": at, "derived_from": {"positions": pos, "houses": houses,
            "other": sorted(srcs - {p["id"] for p in pos} - {"houses"})},
            "calculation": env.get("calculation"), "provenance": env.get("provenance"),
            "note": "epistemic says what kind of value this is: calculated, or a named tradition_rule applied to calculated inputs"}


# tools whose envelope is reduced before it is returned
POST: Dict[str, Callable[[Dict[str, Any], Dict[str, Any]], Dict[str, Any]]] = {"explain_derivation": _explain}


def _result(msg_id: Any, result: Any) -> Dict[str, Any]:
    return {"jsonrpc": "2.0", "id": msg_id, "result": result}


def _error(msg_id: Any, code: int, message: str) -> Dict[str, Any]:
    return {"jsonrpc": "2.0", "id": msg_id, "error": {"code": code, "message": message}}


def handle(msg: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """One JSON-RPC message in, the response out (None for a notification)."""
    method, msg_id, params = msg.get("method"), msg.get("id"), msg.get("params") or {}
    if msg_id is None:  # notifications (initialized, cancelled) need no answer
        return None
    if method == "initialize":
        asked = params.get("protocolVersion")
        return _result(msg_id, {
            "protocolVersion": asked if asked in PROTOCOL_VERSIONS else PROTOCOL_VERSIONS[0],
            "capabilities": {"tools": {"listChanged": False}},
            "serverInfo": {"name": "eshana", "version": engine_version()},
            "instructions": "ESHANA computes positions, charts and calendars locally. Results are calculations and named "
                            "tradition rules; the engine never interprets. Local times need an IANA timezone.",
        })
    if method == "ping":
        return _result(msg_id, {})
    if method == "tools/list":
        return _result(msg_id, {"tools": [{"name": n, "description": d, "inputSchema": s} for n, (d, s, _) in TOOLS.items()]})
    if method == "tools/call":
        name, args = params.get("name"), params.get("arguments") or {}
        if name not in TOOLS:
            return _error(msg_id, -32602, f"unknown tool {name!r}")
        try:
            env = execute(TOOLS[name][2](args))
            if name in POST:
                env = POST[name](args, env)
            return _result(msg_id, {"content": [{"type": "text", "text": json.dumps(env, ensure_ascii=False)}], "structuredContent": env, "isError": False})
        except EshanaError as e:  # the engine refused: say why, as a tool result the model can read
            return _result(msg_id, {"content": [{"type": "text", "text": str(e)}], "isError": True})
        except (KeyError, TypeError, ValueError) as e:
            return _result(msg_id, {"content": [{"type": "text", "text": f"bad arguments for {name}: {e}"}], "isError": True})
    return _error(msg_id, -32601, f"method not found: {method}")


def main(argv: Optional[List[str]] = None) -> int:
    """Serve MCP over stdio until the client closes stdin."""
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError as e:
            out: Optional[Dict[str, Any]] = _error(None, -32700, f"parse error: {e}")
        else:
            out = handle(msg) if isinstance(msg, dict) else _error(None, -32600, "a request must be an object")
        if out is not None:
            sys.stdout.write(json.dumps(out, ensure_ascii=False) + "\n")
            sys.stdout.flush()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
