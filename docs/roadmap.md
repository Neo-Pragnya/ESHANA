---
title: "Roadmap"
description: "What ESHANA 0.1 delivers, what is next, and what is deliberately excluded. Status words are exact."
---

# Roadmap

Status words are used exactly: **proposed** (designed, not built), **implemented** (code exists), **verified**
(a test or a measured comparison proves it), **released** (published). The per-task ledger is
`planning/backlog.json`, rendered to `planning/TRACKING.md`; `make tracking-check` fails if the report is stale.

## 0.1.0: the foundation (this repository)

Verified: time and civil-zone resolution, two ephemeris providers measured against JPL DE440s, ayanāṃśa, houses
and Ascendant against the Swiss Ephemeris as an offline oracle, Vedic and Western rule packs, the Human Design solver
and tables, numerology, uncertainty windows, comparison, graph export, the Python facade, the capability
acceptance gate (74 scenarios). **Not yet released**: nothing is on crates.io or PyPI; names are not reserved
([ADR-007](adr/007-naming-and-scope.md)).

## Next (0.2)

- Name and trademark clearance, then register `eshana` on PyPI and crates.io.
- Event search: transits, ingresses, stations, returns.
- Swiss Ephemeris adapter behind a feature flag, only after the licence decision ([ADR-001](adr/001-ephemeris-provider-and-licensing.md)).
- Node.js (NAPI-RS) and browser (WASM) bindings behind the same JSON entry point.
- More divisional charts (D30, D60 need a documented school choice), dignities and relationships as separate rule packs.
- An MCP server exposing `compute_positions`, `create_vedic_chart`, `compare_charts`, `compute_human_design`,
  `compute_numerology`, `explain_derivation`.
- Real-boundary golden sets: sub-arc-minute HD gate boundaries and dashā boundary instants.

## Later

Synastry, composite, progressions, Jaimini and Tajika as named adapters, I Ching and calendar adapters, a rule
authoring DSL, a hosted API, a backtesting toolkit with look-ahead guards. None before core conformance.

## Excluded on purpose

No unverified prediction of life outcomes, relationships, health or trading returns. No universal
`destiny_score` or `compatibility_score`. No requirement for a hosted LLM or an account. No marketplace of traditions
before conformance.
