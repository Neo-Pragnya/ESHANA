---
title: "Next steps"
description: "What is done, what was decided, and the ordered list of what to build next."
---

# Next steps

State on 2026-11-22. The ledger of every task is [`planning/TRACKING.md`](../planning/TRACKING.md), generated from
`planning/backlog.json`. This page is the readable order of work.

## Done

**0.3.3 (2026-11-22): review round 8**
- Readable charts: graha text larger and bolder than the marks, full-width D1 charts, the higher degree written higher, mūlatrikoṇa △.
- The South Indian centre of the Jyotish Chakra shows the four Nadi directions in its middle square.
- Gender with the name (Kua for the person); growth, intensity, letter transits and essence in name numerology.
- Separate filters for rāśi aspects, ārūḍha padas, additional graha aspects and additional rāśi aspects.

**0.3.2 (2026-11-22): review rounds 5 and 6**
- Five analysis categories: maitri and dignities in 16 divisions; seven more daśā systems; sahams, avakahaḍā, mṛtyu bhāga, ghāta, vedhasthāna, jīva/deha, śodhana; Sarvatobhadra, Kota and Saṅghaṭṭa; varṣaphala, pañca pakṣī and KP tables — each compared with a published reference chart.
- Charts with padas on top and aspects at the bottom (no overlaps, tested); parivartana marks and yogas; transit hover line; docked hover details; dark-theme contrast.
- Lordships in D1/D9/D10, pada navāṃśa, Kua and compound numbers, Yoginī past 120 years, a website Gallery.

**0.3.1 (2026-11-21): review rounds 1–4**
- Upagrahas; switches to hide the outer planets and upagrahas; outer-planet graha and rāśi drishti; optional additional aspects (retrograde shadow, parivartana) — see the [Aspects guide](guides/aspects.md).
- Jyotish Chakra (Graha-Nakshatra Mandala); daśā five-level zoom; bhāvāt bhāvam and each bhāva as its own lagna; even concentric squares.
- **Aspects:** percent charts from every view, for D1 and D9.
- **Chakra:** markers ring, curved names and a graha legend.
- **Charts and bhāvas:** chalit overlays (square and circle); bhāva depth (bhāvāt bhāvam, connections, bearing).
- **Strength and progressions:** a strength cut-off line; progression by completed revolutions.
- **Numerology and Human Design:** numerology in every school; the standard Human Design chart.
- **Website and tooling:** PyPI links on the website; local preview (`make preview`) and
  [progress tracker](../planning/PROGRESS.md).

**0.3.0 (2026-11-21)**
- **ESHANA Studio** ([guide](guides/html-report.md)) is now a multi-page app: graha and bhāva pages, a ⌘K palette and a
  starfield. The Panchang chakra can hold a South or North Indian chart in its centre. Charts show translucent aspect
  marks, there is a gallery of 19 vargas, and a [Nadi direction matrix](guides/nadi.md) and AstroEnergetics page.
- **Strength and special points** ([guide](guides/strength.md)): ṣaḍbala, bhāva bala, viṃśopaka, sputa drishti
  in virūpas, yogi/avayogi, dagdha, Indu lagna, Sripati bhāva chalit, declination; vargas D5, D6, D8.
- A published reference-software chart joins the accuracy suite. Real-Chrome journeys test the studio and the website
  in CI. The website has its own universe hero and ESHANA sigils.

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
2. **Reconcile ṣaḍbala totals** with reference software and add iṣṭa/kaṣṭa phala (`VED-14`); **aṣṭakavarga transit strength** (`VED-12`).
3. **Conditional daśās** (`VED-10`): Chara, Nārāyaṇa.
4. **Muhūrta search** (`MUH-01`), **varṣaphala** (`VAR-01`), **KP horary** (`KP-02`), **Rajju/vedha** (`MAT-02`).
5. **More ayanāṃśas** (`AYA-02`), **upagrahas** (`VED-13`) and **D11 and other vargas** (`VED-09`).
6. **Studio extras**: East Indian chart (`REP-06`), a chart playground (`REP-07`), languages (`REP-08`).
7. **User rule packs at run time** (`RULE-02`).
8. **Bindings**: MCP server (`BIND-05`), Node and WASM (`BIND-03`, `BIND-04`), crates.io (`BIND-07`).
9. **More published references** (`ACC-04`).

## Planned studio sections (reference chart library, 0.5)

The reference-software screenshots add about twenty charts. They fit into six new pages, so the studio stays easy to
move around in, rather than one long page. Each will have its own sigil, deep link, tooltips and journeys, and a
gallery entry on the website (`CHK-22`).

| Page (nav group) | Contents | Tasks |
|---|---|---|
| **Lordships** (Grahas & bhāvas) | "lord of 1 in 3" for D1, D9 and D10 with sign exchanges; pada navāṃśa chart; upagrahas (Dhūma … Māndi) on the charts | `CHK-05`, `CHK-06`, `CHK-07` |
| **Chakras** (Systems) | Sarvatobhadra with vedha, Kota, Saṅghaṭṭa, Ghāta, Avakahaḍā | `CHK-12`, `CHK-14`…`CHK-17` |
| **Annual** (Time) | Varṣaphala: year lord, muntha, pañcādhikārīs, harṣa and hudda strengths; the 37 Sahams | `CHK-11`, `CHK-13` (with `VAR-01`) |
| **Pañca pakṣī** (Time) | ruling and dying birds, the five activities through day and night | `CHK-09` |
| **Sensitive points** (Strength) | mṛtyu bhāga, jīva and deha, śodhana piṇḍa, male and female rāśis | `CHK-10`, `CHK-18`…`CHK-20` |
| KP (existing) and Transits (existing) | bhāva significator table, strong / normal / weak significators; vedhasthāna from the Moon | `CHK-08`, `CHK-21` |

Open differences found while matching the screenshots:
- Jupiter's special aspects and the node aspects in one reference program (`CHK-23`);
- Human Design cross names and exaltation and detriment arrows (`CHK-24`).

## Decisions still open

None. The rule-pack format was settled as YAML (ADR-012), and the studio now offers a detailed and a schematic
BodyGraph.
