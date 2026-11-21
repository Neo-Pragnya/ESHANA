"""ESHANA architecture diagrams, as code. Rebuild with `make diagrams`.

ESHANA is a local library, not an AWS deployment, so these are concept diagrams: cards plus numbered, animated flows.
Every claim here is something the code does today; planned work is labelled "planned".
"""
from archlib import Flow, Group, Node, Spec

BLUE, GREEN, RED, VIOLET, SLATE, AMBER, TEAL = "#3B48CC", "#2F7D32", "#B4322A", "#6D28D9", "#334155", "#B7791F", "#0E7490"


def card(id, title, body, x, y, color, w=1.8, sub="", **kw):
    return Node(id, None, title, sub=sub, x=x, y=y, body=body, color=color, w=w, **kw)


SYSTEM = Spec(
    id="01-system-overview", title="ESHANA: one JSON entry point, three thin shells",
    subtitle="Every surface calls Engine::execute_json, so the CLI, the Python package and the HTML report cannot disagree.",
    cols=9.2, rows=3.7,
    nodes=[
        card("req", "Request (JSON)", ["when + place + settings", "positions · chart · human_design", "numerology · compare · graph"], 0.5, 1.7, SLATE, w=1.9),
        card("time", "eshana-time", ["civil time to UTC with tz rules", "fold / gap refused, never guessed", "leap seconds, delta-T"], 3.0, 0.5, BLUE, w=2.0),
        card("eph", "eshana-ephemeris", ["analytic provider (no files)", "spk provider (your DE440s)", "precession, nutation, aberration"], 3.0, 2.2, BLUE, w=2.0),
        card("rules", "eshana-engine", ["vedic.parashari@0.1", "human_design.basic@0.1", "numerology, houses, graph"], 5.6, 1.3, VIOLET, w=2.0),
        card("env", "Envelope (JSON)", ["calculated values, labelled", "provenance + assumptions", "interpretations: always []"], 5.6, 3.0, GREEN, w=2.0),
        card("cli", "eshana CLI", ["JSON in the terminal"], 8.3, 0.4, TEAL, w=1.6),
        card("py", "eshana (Python)", ["Chart objects", "execute(request)"], 8.3, 1.7, TEAL, w=1.6),
        card("rep", "eshana-report", ["one HTML file, every chart"], 8.3, 3.0, AMBER, w=1.6),
    ],
    flows=[
        Flow(1, "req", "time", "A request names an instant (UTC, or civil time plus zone) and the settings used", packet="request"),
        Flow(2, "time", "eph", "The instant becomes TT and UT time scales and Julian dates", packet="JD"),
        Flow(3, "eph", "rules", "A provider returns apparent ecliptic positions in one frame", packet="lon/lat", sp="r", dp="l"),
        Flow(4, "rules", "env", "Rule packs derive signs, houses, nakshatra, dasha and gates; each value is labelled", packet="derived", sp="b", dp="t"),
        Flow(5, "env", "cli", "The CLI prints the envelope as JSON", packet="JSON", sp="r", dp="l", via=7.1, badge_at=.6),
        Flow(6, "env", "py", "Python returns the same data as a dict or a Chart", packet="dict", sp="r", dp="l", via=7.3, badge_at=.75),
        Flow(7, "env", "rep", "The report renderer turns the envelope into one self-contained HTML file", packet="HTML", sp="r", dp="l", via=7.5, badge_at=.85),
    ],
)

DOCTRINE = Spec(
    id="02-doctrine", title="Compute, Derive, Interpret: the engine stops after Derive",
    subtitle="Each value carries an epistemic label. Interpretation is a separate, opt-in layer that ESHANA does not ship.",
    cols=8.4, rows=2.9,
    nodes=[
        card("c", "1 · Compute", ["positions, houses, time scales", "label: calculated", "checked against JPL DE440s"], 0.9, 0.7, BLUE, w=2.2),
        card("d", "2 · Derive", ["nakshatra, bhava, dasha, gates", "label: tradition_rule", "rule pack and version recorded"], 3.7, 0.7, VIOLET, w=2.2),
        card("i", "3 · Interpret", ["meanings, predictions, advice", "labels: speculative / hypothesis", "NOT in the engine: interpretations = []"], 6.6, 0.7, RED, w=2.4),
        card("u", "Your layer", ["a plugin, notebook or app", "must label what it adds", "can cite rule ids from the envelope"], 6.6, 2.3, SLATE, w=2.4),
        card("g", "Refusals", ["sidereal without ayanamsha", "ambiguous or skipped local time", "polar Placidus houses"], 3.7, 2.3, AMBER, w=2.2),
    ],
    flows=[
        Flow(1, "c", "d", "Computed numbers feed the tradition rules; the rule version travels with every result", packet="calculated"),
        Flow(2, "d", "i", "The engine does not cross this line: interpretations is always an empty list", packet="stop", color=RED, dashed=True),
        Flow(3, "i", "u", "Anything beyond the line is yours, and has to say what kind of claim it is", packet="label", sp="b", dp="t"),
        Flow(4, "d", "g", "When the input cannot support an answer, ESHANA refuses with a stable ESH- error code", packet="ESH-*", sp="b", dp="t", color=AMBER, dashed=True),
    ],
)

ACCURACY = Spec(
    id="03-accuracy-ladder", title="How a birth chart is proven right: five independent layers",
    subtitle="No layer trusts the one below it. A failure in any layer says which kind of disagreement it is.",
    cols=9.4, rows=3.4,
    nodes=[
        card("l1", "1 · Mathematics", ["time scales, precession, nutation", "unit and property tests", "Rust: 100+ tests"], 0.9, 0.6, SLATE, w=1.9),
        card("l2", "2 · JPL DE440s", ["positions vs the kernel", "spk 0.3 arcsec; analytic measured", "golden fixture, Skyfield"], 3.0, 0.6, BLUE, w=1.9),
        card("l3", "3 · Swiss Ephemeris", ["many charts, 1900-2100", "ayanamsha, houses, nodes", "offline oracle, numbers only"], 5.1, 0.6, VIOLET, w=1.9),
        card("l4", "4 · Published sites", ["ProKerala snapshot (recorded)", "classical constants (Lahiri 1956)", "contributed references"], 7.2, 0.6, AMBER, w=1.9),
        card("l5", "5 · Re-derivation", ["nakshatra, pada, dasha, D-charts", "a second implementation in the tests", "invariants on every chart"], 3.9, 2.4, GREEN, w=2.3),
        card("gate", "Release gate", ["PROMOTE / CONDITIONAL / BLOCK", "tolerances live in one file", "report: accuracy.md"], 7.2, 2.4, RED, w=1.9),
    ],
    flows=[
        Flow(1, "l1", "l2", "Mathematics is trusted only after it reproduces the JPL kernel", packet="exact"),
        Flow(2, "l2", "l3", "The same charts are checked against a second, independent engine", packet="cross-check"),
        Flow(3, "l3", "l4", "Then against what real sites and published constants print, with tolerance for rounding", packet="rounded"),
        Flow(4, "l4", "gate", "A disagreement beyond tolerance blocks the release", packet="diff", sp="b", dp="t", color=RED),
        Flow(5, "l2", "l5", "Derived values are recomputed from the longitudes by a separate implementation", packet="re-derive", sp="b", dp="t"),
        Flow(6, "l5", "gate", "Any mismatch is a bug in one of the two implementations, never ignored", packet="mismatch", color=RED, dashed=True),
    ],
)

USAGE = Spec(
    id="04-usage-modes", title="Five ways to use ESHANA, all on the same engine",
    subtitle="Pick the surface that fits; the numbers are identical because they come from one entry point.",
    cols=9.0, rows=3.7,
    nodes=[
        card("me", "You", ["birth data, or just a date"], 0.5, 1.8, SLATE, w=1.4),
        card("cli", "1 · Terminal", ["eshana chart vedic ...", "JSON on stdout, pipe to jq"], 3.0, 0.4, TEAL, w=2.1),
        card("py", "2 · Python library", ["pip install eshana", "Chart objects, execute()"], 3.0, 1.5, TEAL, w=2.1),
        card("rep", "3 · HTML report", ["eshana-report ... -o chart.html", "one file, offline, printable"], 3.0, 2.6, AMBER, w=2.1),
        card("nb", "4 · Notebook or app", ["chart.to_html() in place", "embed the SVG charts anywhere"], 6.2, 1.5, VIOLET, w=2.1),
        card("fut", "5 · Planned", ["Node/WASM, MCP server", "event search (AST-06)"], 6.2, 3.0, SLATE, w=2.1),
        card("eng", "Engine::execute_json", ["one entry point"], 6.2, 0.4, BLUE, w=2.1),
    ],
    flows=[
        Flow(1, "me", "cli", "Quickest: one command, JSON out", packet="args", sp="t", dp="l"),
        Flow(2, "me", "py", "From code: the same requests as a typed API", packet="call"),
        Flow(3, "me", "rep", "To read or share: a complete chart report with every detail", packet="request", sp="b", dp="l"),
        Flow(4, "cli", "eng", "The CLI is a thin shell over the engine", packet="JSON", sp="r", dp="l"),
        Flow(5, "py", "nb", "Chart objects drop into notebooks and apps; to_html() renders the report in place", packet="Chart", sp="r", dp="l"),
        Flow(6, "rep", "fut", "Planned surfaces reuse the same envelope, so nothing about the maths changes", packet="planned", sp="r", dp="l", dashed=True, via=5.6),
    ],
)

REPORT = Spec(
    id="05-report-pipeline", title="From a request to one self-contained HTML chart report",
    subtitle="The renderer only formats what the engine computed. It adds no astrology of its own.",
    cols=9.0, rows=2.9,
    nodes=[
        card("rq", "Request", ["date, time, zone, place", "system, ayanamsha, houses"], 0.5, 1.0, SLATE, w=1.7),
        card("en", "Engine", ["chart, human_design, numerology", "one envelope each"], 2.7, 1.0, BLUE, w=2.0),
        card("rd", "eshana.report", ["South + North Indian charts", "wheel, planets, houses", "D-charts, dasha, nakshatra"], 5.2, 1.0, AMBER, w=2.1),
        card("html", "chart.html", ["inline CSS, SVG and data", "light / dark, print layout", "raw JSON embedded"], 7.8, 1.0, GREEN, w=1.9),
        card("sec", "Sections you choose", ["--sections chart,dasha,hd,numerology", "--theme light|dark|auto"], 5.2, 2.6, SLATE, w=2.4),
    ],
    flows=[
        Flow(1, "rq", "en", "The same request object is used for every system you ask for", packet="request"),
        Flow(2, "en", "rd", "Envelopes go to the renderer unchanged", packet="envelope"),
        Flow(3, "rd", "html", "One file out: no network, no fonts to fetch, safe to email or archive", packet="HTML"),
        Flow(4, "sec", "rd", "You pick sections and theme; a section with no data is omitted, not faked", packet="options", sp="t", dp="b"),
    ],
)

SPECS = [SYSTEM, DOCTRINE, ACCURACY, USAGE, REPORT]
