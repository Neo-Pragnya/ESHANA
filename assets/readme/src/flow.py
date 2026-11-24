"""How ESHANA answers a question: an animated concept flow for the README.

Rendered with the marvellous-aws-architecture-docs skill (cards + live numbered flows, no cloud icons):

    python <Skills>/marvellous-aws-architecture-docs/scripts/build_arch.py assets/readme/src/flow.py --out /tmp/eshana-flow --no-stages
    cp /tmp/eshana-flow/svg/eshana-flow.svg assets/readme/
"""
from archlib import Flow, Node, Spec

INK, GOLD, VIOLET, TEAL, GREEN, RED, SLATE = "#0F1B2D", "#B7791F", "#6D28D9", "#0E7490", "#0B8A6F", "#B4322A", "#334155"


def card(id, title, body, x, y, color, w=1.7, sub="", **kw):
    return Node(id, None, title, sub=sub, x=x, y=y, body=body, color=color, w=w, **kw)


SPECS = [Spec(
    id="eshana-flow", title="How ESHANA answers a question",
    subtitle="One Rust engine behind every surface. Calculated, derived and interpretive stay apart in the data, and every result says how it was made.",
    cols=8.2, rows=3.3,
    nodes=[
        card("ask", "Your question", ["CLI · Python · Studio (WebAssembly)", "eshana-report new (a local form)", "one JSON request, unknown fields refused"], 0.95, 0.35, VIOLET, w=1.8),
        card("time", "Civil time", ["IANA zones with historic offsets", "DST gaps and folds: errors, not guesses"], 0.95, 2.35, TEAL, w=1.8),
        card("eng", "Engine::execute_json", ["Rust · deterministic", "same answer on every surface"], 3.4, 1.35, INK, w=1.8, sub="one entry point"),
        card("eph", "Ephemeris", ["analytic (no files) or JPL SPK", "checked against DE440s and Swiss Ephemeris"], 3.4, 0.0, SLATE, w=1.8),
        card("rules", "Rule packs (YAML)", ["versioned, named methods", "unknown method = ESH-METHOD"], 3.4, 2.75, GOLD, w=1.8),
        card("env", "Result envelope", ["calculated: positions, cusps", "derived: tradition_rule + method id", "interpretations: always empty", "provenance + config_hash"], 5.95, 0.55, GREEN, w=2.0),
        card("out", "What you get", ["Studio app · PNG / SVG", "calendar CSV / ICS · graphs", "golden-tested, arc-second checks"], 5.95, 2.55, VIOLET, w=2.0),
    ],
    flows=[
        Flow(1, "ask", "time", "A birth or event in local civil time is resolved to UTC first, or refused with a stable code", packet="local"),
        Flow(2, "time", "eng", "The engine receives an exact instant and place", packet="UTC"),
        Flow(3, "eng", "eph", "Positions come from a measured provider: the calculated layer", packet="λ, β"),
        Flow(4, "eng", "rules", "Nakshatras, vargas, daśās, muhūrta, KP: each from a named, versioned rule", packet="rule"),
        Flow(5, "eng", "env", "Everything returns in one envelope, with every assumption recorded", packet="JSON"),
        Flow(6, "env", "out", "Surfaces only render: the studio, files and images never compute a factor of their own", packet="render"),
    ],
)]
