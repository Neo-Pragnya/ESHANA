# Changelog

All notable changes. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow SemVer. `schema_version` of the
result envelope is versioned separately (see `docs/concepts/provenance.md`).

## [Unreleased]

## [0.3.0] - 2026-11-21

### Added
- **ESHANA Studio**: `eshana-report` now writes a multi-page, hash-routed app with 24 pages. There is one page per
  graha and per bhāva, a ⌘K command palette, a settings drawer, a starfield and animated sigil icons. Pages animate
  in and the address keeps the page.
- The Panchang chakra can hold a **South Indian** or **North Indian** chart in its centre, or the orbit lanes
  (`--centre`).
- South and North Indian charts are redrawn with degrees, nakshatras and **translucent aspect marks** (for example
  `Ju*` in the house Jupiter aspects).
- **Graha pages**: declination, aspects cast and received in virūpas, yogas, ṣaḍbala, viṃśopaka, vargas with
  degrees, Nadi links.
- **Nadi direction matrix** with degrees and links (`report.nadi@0.1`); **AstroEnergetics** page.
- **Strength** (`vedic.bala@0.3`, `bala.yaml`): ṣaḍbala with named conventions, bhāva bala, viṃśopaka (four
  schemes), sputa drishti matrices for D1 and D9.
- **Special points**: yogi, avayogi, duplicate yogi, dagdha rāśis, Indu lagna, 64th navāṃśa, 22nd dreshkāṇa, kartari,
  graha yuddha, functional nature.
- Vargas **D5, D6, D8** (19 in all) and `degree_in_varga`.
- **Sripati** (bhāva chalit) and **Porphyry** houses; declination and right ascension for every body.
- Accuracy: a published reference-software chart (positions, declinations, special lagnas, special points). D5, D6
  and D8 are re-derived independently.
- Real-Chrome journeys for the studio (`make studio-journeys`) and the website (`make site-journeys`), run in CI.
- Website: a universe hero (parallax stars, nebulae, a 27-nakshatra ecliptic plexus that answers the cursor, grahas
  riding the band, shooting stars), an ambient sky on every page, and ESHANA's own sigil icons.

### Changed
- Rule packs: `vedic.parashari@0.3`, `vedic.core@0.3`. The Sun's required rūpas are 6.5.
- Report section ids are now page ids (`overview`, `chakra`, `charts`, `planets`, ...); `summary` became `overview`.

### Known gaps
- Ṣaḍbala totals are within about 60 virūpas of reference software (`VED-14`). Iṣṭa and kaṣṭa phala are not yet
  computed.

## [0.2.0] - 2026-11-21

### Added
- **Data-driven rule packs in YAML** (ADR-012): `vedic-core`, `ashtakavarga`, `panchanga`, `matching`, `yogas`, read
  with `serde_norway`, validated at load; `eshana rulesets` prints them.
- **Vedic features** (`--features`, `features=[...]`): strength (dignity, avasthā, combustion, dig and naisargika
  bala), pañcadhā maitrī, graha and rāśi drishti, bhinna- and sarvāṣṭakavarga, Jaimini chara kārakas, kārakāṃśa and
  ārūḍha padas, KP sub and sub-sub lords with significators and ruling planets, 18 yogas and 6 doshas, Sāḍe Sātī.
- Vargas D2, D16, D20, D24, D27, D30, D40, D45 and D60.
- Vimśottarī daśā to five levels with the current chain at `reference_utc`; Yoginī daśā.
- **Pañcāṅga** (`eshana panchanga`): the five limbs with end times; sunrise, sunset, moonrise and moonset under a
  stated convention; horā, choghaḍiyā, Rāhu kālam, Yamagaṇḍam, Gulika kālam; Bhāva, Horā and Ghaṭī lagnas; Tārā and
  Candra bala.
- **Aṣṭakūṭa matching** (`eshana matching`) with Manglik flags.
- **Ephemeris tables and ingress search** (`eshana ephemeris`, `eshana ingresses`).
- **Chart studio**: `eshana-report` is now an interactive page. It includes the Panchang Chakra wheel with every point
  placed by exact degree; nakshatra, pada/navāṃśa and dreshkāṇa rings; and drill-down. Layers cover drishti, rāśi
  drishti, Western aspects, ārūḍhas, SAV and animated transits. South and North Indian charts are available for any
  varga, with textured-sphere, glyph or badge planet icons. Further sections: daśā timelines, pañcāṅga, strength,
  aṣṭakavarga, KP, Jaimini, yogas, progressions, a detailed or schematic BodyGraph and compatibility. Viewers choose
  sections and styles.
- Accuracy suite: pañcāṅga limbs and rise/set vs Swiss; ingresses vs Swiss; special lagnas vs PyJHora; KP,
  aṣṭakavarga, aṣṭakūṭa and vargas to D60 re-derived (about 1,300 checks).
- Docs: Vedic features, pañcāṅga, matching, capabilities checklist, ADR-012.

### Fixed
- Combustion and dig bala used a signed angle.
- A pañcāṅga at the end of the provider range failed instead of reporting no event.

### Changed
- `vedic.parashari` is now `@0.2` (more vargas; Vimśottarī periods carry `years`). Envelope `schema_version` 0.2.0
  adds the kinds `panchanga`, `matching`, `ephemeris`, `ingresses` and `rulesets`.

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
