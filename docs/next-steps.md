---
title: "Next steps"
description: "What is done, what was decided, and the ordered list of what to build next."
---

# Next steps

State on 2026-11-21. The ledger of every task is [`planning/TRACKING.md`](../planning/TRACKING.md), generated from
`planning/backlog.json`. This page is the readable order of work.

## Done

**0.2.0 (2026-11-21)**
- **Chart studio** ([guide](guides/html-report.md)): the Panchang Chakra with every point placed by exact degree; layers
  for drishti, rāśi drishti, Western aspects, ārūḍhas, SAV and animated transits; South and North Indian charts for
  every varga; sphere, glyph or badge icons; detailed or schematic BodyGraph; viewer-chosen sections.
- **Vedic features** ([guide](guides/vedic-features.md)), **pañcāṅga** ([guide](guides/panchanga.md)) and
  **matching** ([guide](guides/matching.md)) from **YAML rule packs** ([ADR-012](adr/012-yaml-rule-packs.md)).
- Accuracy suite extended to about 1,300 checks: pañcāṅga, rise/set and ingresses vs Swiss, special lagnas vs PyJHora.
- The full checklist of what is done and planned: [capabilities](reference/capabilities.md).

**0.1.0 (2026-11-20/21)**: engine, CLI, Python package, first reports, accuracy suite, open-source decision
([ADR-011](adr/011-open-source-distribution.md)), PyPI release, first CI run (all green), merged to `main`,
GitHub Pages enabled.

## Next, in order

1. **Wheels for every platform.** Add the GitHub publisher on PyPI (trusted publishing, workflow `release.yml`,
   environment `pypi`) and push a version tag. Linux, Intel macOS and Windows users then install without Rust.
2. **Full ṣaḍbala** (`VED-11`) and **aṣṭakavarga transit strength** (`VED-12`).
3. **Conditional daśās** (`VED-10`): Chara, Nārāyaṇa.
4. **Muhūrta search** (`MUH-01`), **varṣaphala** (`VAR-01`), **KP horary** (`KP-02`), **Rajju/vedha** (`MAT-02`).
5. **More ayanāṃśas** (`AYA-02`) and **upagrahas** (`VED-13`).
6. **Studio extras**: East Indian chart (`REP-06`), a chart playground (`REP-07`), languages (`REP-08`).
7. **User rule packs at run time** (`RULE-02`).
8. **Bindings**: MCP server (`BIND-05`), Node and WASM (`BIND-03`, `BIND-04`), crates.io (`BIND-07`).
9. **More published references** (`ACC-04`).

## Decisions still open

None. The rule-pack format was settled as YAML (ADR-012), and the studio now offers a detailed and a schematic
BodyGraph.
