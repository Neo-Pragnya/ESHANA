# Changelog

All notable changes. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow SemVer. `schema_version` of the
result envelope is versioned separately (see `docs/concepts/provenance.md`).

## [Unreleased]

## [0.1.0] - 2026-11-21

First implementation of the master plan's Sprint 0 and the MVP: see `planning/TRACKING.md` for the task-level state.

### Added
- Rust workspace: `eshana-core`, `eshana-time`, `eshana-ephemeris`, `eshana-engine`, `eshana-cli`, `eshana-python`.
- Civil time via IANA tz (historical offsets, fold and gap errors), leap seconds, ΔT, TT, JD, sidereal time.
- Providers: `analytic` (Standish elements + Meeus lunar series) and `spk` (JPL type-2 kernels, user supplied).
  Measured against DE440s: `spk` 0.3″ max; `analytic` 16″ Sun, 10″ Moon, 342″ Jupiter, 1026″ Saturn.
- Ayanāṃśa (Lahiri, Raman, Krishnamurti, Fagan–Bradley); Ascendant, Midheaven, whole-sign, equal and Placidus houses; mean and
  osculating true node; Ptolemaic aspects with applying state.
- Vedic rule pack `vedic.parashari@0.1`: rāśi, nakshatra/pada, bhāva, D1/D3/D4/D7/D9/D10/D12, Vimśottarī daśā.
- Human Design rule pack `human_design.basic@0.1`: the 88° solar-arc design instant, wheel, channels, centres, type, authority.
- Numerology packs `numerology.pythagorean@0.1` and `numerology.chaldean@0.1`.
- Birth-time uncertainty windows, method comparison, symbolic graph (JSON, Mermaid, GraphML).
- `eshana` CLI and the `eshana` Python package over one JSON entry point; JSON Schema for the envelope.
- 103 Rust tests, 21 Python tests, 74 acceptance scenarios (17 capabilities), golden fixtures and an accuracy report.
- Docs (concepts, guides, reference, eleven ADRs), an Astro site with demos, tracking ledger, CI workflows.
- `eshana-report` and `Chart.to_html()`: one self-contained HTML file per chart (Panchang chakra, South and North
  Indian charts, vargas, daśā, nakshatra reference, Western wheel with aspects, BodyGraph, numerology, provenance,
  raw JSON), also from saved CLI JSON (`--from-json`), and inline in Jupyter.
- Birth-chart accuracy suite (`accuracy/`): 44 reference charts against the Swiss Ephemeris (offline oracle), the Lahiri
  definition, a recorded ProKerala chart and an independent re-derivation of every rule; about 970 checks.
- Architecture diagrams as code (`diagrams/`, `make diagrams`), release workflow with trusted publishing.
- ADR-011: ESHANA is an open-source tool stack published as libraries.

### Fixed
- Sidereal whole-sign house cusps started one sign early when the sidereal Ascendant was less than the ayanāṃśa past
  a sign boundary (the bhāva numbers were already right). Found by the accuracy suite.
- Ketu now carries its tropical longitude.
- The last antardaśā of each mahādaśā now ends exactly at the mahādaśā end (was off by up to 1 ms).

### Known limits
- `analytic` is arc-minute class for Jupiter and Saturn. D30 and D60 are not implemented. No event search yet.
- Leap seconds after 2027 are a forecast. Proleptic Gregorian only.
