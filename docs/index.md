---
title: "ESHANA"
description: "Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations. A Rust-first computation layer that keeps measurement, tradition and interpretation apart, with provenance on every result."
---

# ESHANA

**Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations.**
*From celestial positions to connected meaning.*

ESHANA turns an instant, a place, a name or a set of parameters into **deterministic measurements, rule-governed
classifications and typed relationships**, each carrying the settings and the rule version that produced it. It is
infrastructure, not an oracle: it makes traditions computable and auditable, and it makes no claim that astrology,
numerology or Human Design have predictive validity.

```bash
$ eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 9
```

## The doctrine: Compute → Derive → Interpret

| Layer | Example | Status |
|---|---|---|
| **Calculated** | ecliptic longitude, speed, Julian day, Ascendant | verifiable against reference ephemerides; every one is tested |
| **Derived by a named system** | nakshatra and pada, divisional sign, aspect, house, HD gate, a name number | deterministic under an explicit tradition, method and rule version; traditions can disagree |
| **Interpretive** | character, relationship, "what this means" | **never produced by the engine**; `interpretations` is always empty |

Read [The epistemic model](concepts/epistemic-model.md) first; everything else follows from it.

## What is in 0.1

- **Time**: IANA civil time with historical offsets; DST folds and gaps are errors with distinct codes; leap seconds,
  ΔT, TT, Julian day and sidereal time.
- **Positions**: ten bodies and the lunar node, apparent geocentric, with speed and retrograde state. Two providers:
  `analytic` (no data files, arc-second class since 0.5.0) and `spk` (reads a JPL kernel you supply, arc-second class).
- **Vedic**: rāśi, nakshatra and pada, whole-sign bhāva, D1/D3/D4/D7/D9/D10/D12, Vimśottarī daśā with a named year
  convention, four ayanāṃśas.
- **Western**: tropical zodiac, whole-sign, equal and Placidus houses (refused where undefined), Ptolemaic aspects with
  orb and applying/separating state.
- **Human Design**: the design instant as a numerical solve of the 88° solar arc, 13 activations per side, channels,
  centres, type, authority, profile and definition. Rights review is pending ([ADR-009](adr/009-human-design-rights.md)).
- **Numerology**: Pythagorean and Chaldean as separate schemas, explicit normalisation, life path, personal year.
- **Uncertain birth time**: intervals and stability flags; the Ascendant and houses are never invented.
- **Method comparison** and a **symbolic graph** (JSON, Mermaid, GraphML) with relation classes and no causal edges.
- **Bindings**: a Rust CLI and a Python package over one JSON entry point, so the outputs are identical.

## Where to go next

- New here? [Getting started](guides/getting-started.md), then the [Python guide](guides/python.md) and the
  [cookbook](guides/cookbook.md). From JavaScript: [JavaScript guide](guides/javascript.md); from an assistant: [MCP](guides/mcp.md).
- Want a chart to read or share? [HTML chart reports](guides/html-report.md). All the options: [ways to use ESHANA](guides/ways-to-use.md).
- How do we know it is right? [Birth-chart accuracy suite](guides/accuracy-suite.md).
- The big picture in diagrams: [architecture](architecture/diagrams.md). What comes next: [next steps](next-steps.md).
- Want the guarantees? [Calculation conventions](concepts/calculation-conventions.md) and [Provenance](concepts/provenance.md).
- Wondering how accurate it is? [Accuracy](reference/accuracy.md), measured, with the method.
- Contributing a tradition? [Rule packs](reference/rulesets.md) and the [ADRs](adr/index.md).
- What is not done yet? The [Roadmap](roadmap.md), tracked task by task in `planning/TRACKING.md`.
