# Changelog

All notable changes. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow SemVer. `schema_version` of the
result envelope is versioned separately (see `docs/concepts/provenance.md`).

## [Unreleased]

### Added (review round 8)
- Charts: graha text larger and bolder than the aspect marks; bigger South Indian cells; full-width D1 charts; light-
  theme graha colours deepened. Inside a rāśi the higher degree is written higher.
- Jyotish Chakra: the South Indian centre (the default) shows the four Nadi directions in its middle square; the
  separate Nadi centre option is gone (saved settings move to South). North and orbit lanes remain.
- Gender (`--gender`, `gender=`): shown with the name, picks the Kua number.
- Numerology: growth, intensity, letter transits (physical, mental, spiritual) and essence by year.
- Filters: separate switches for rāśi aspects, ārūḍha padas, additional graha aspects and additional rāśi aspects.
- Mūlatrikoṇa marked △ in the charts and legends.

## [0.3.2] - 2026-11-22

Review rounds 5 and 6 and the remaining reference-chart items. Schema 0.3.2 adds fields only.

### Added (remaining items)
- **Yoginī daśā** continues one 36-year cycle past the reference date, so a chart of someone born long ago has a
  current period. Daśās whose span has ended open the zoom at birth and say so.
- **Lordships in D1, D9 and D10** with the exchanges between house lords and their kind (`lordships_by_varga` in
  the special points), shown on the Bhāvas page.
- **Pada navāṃśa:** ārūḍha padas worked out in the navāṃśa (`vedic.jaimini.arudha_padas_d9`), shown on the D9 chart.
- **Numerology:** the Kua number for both sexes (`numerology.lo_shu.kua`); Chaldean compound-number readings 10–52.
- **Website:** a Gallery of every studio view captured from real reports (`make gallery`), and the Ramana reference
  report among the example reports.
- `make studio-journeys` also runs every journey on the Ramana reference chart.

### Added (review round 6)
- **Charts:** ārūḍha padas on top of each rāśi and aspect marks at the bottom, wrapped inside the rāśi, in South and
  North Indian charts; crowded North Indian houses fit their triangle or diamond. A journey checks that no label
  overlaps another or leaves its rāśi.
- **Parivartana:** exchanged grahas are also marked in their partner's sign (with additional aspects on); new yogas
  Parivartana, Mahā, Khala and Dainya parivartana (pack `vedic.yogas@0.3`, condition `parivartana`).
- **Jyotish Chakra:** hovering a transit graha draws a line to where it is now; hover details sit in a card beside
  the wheel instead of a popup over it.
- **Dark theme:** chart writing reaches 4.5:1 contrast (aspect marks 3:1), checked by a journey.
- **Kota chakra** redrawn as a fort (Bāhya, Prākāra, Madhya, Stambha) with entry and exit arrows; **Sarvatobhadra**
  framed by nakshatra, pada-sound and pada bands on all four sides, with red/green vedha borders
  (`chakras.yaml` gains `pada_syllables`).
- **Pañca pakṣī:** full day and night tables (five yāmas × five birds), sub-periods, a day picker over the three days
  either side (`vedic.panch_pakshi.nearby_days`) and info boxes.
- **Sahams:** what each of the 37 signifies and how it is read, in info boxes (interpretation, sources named).

### Added (review round 5)
- **Five analysis categories** (features `maitri`, `dashas_extended`, `auxiliary`, `chakras`, `annual`; all included in
  `features: ["all"]`), each with its own rule pack, guide and studio page, and an accuracy test against the
  Ramana Maharishi reference chart:
  - **Maitri** ([guide](docs/guides/maitri.md)): compound relationships for all nine grahas (node friendships named as
    a convention), the nakshatra tārā matrix and dignities in all 16 divisions.
  - **More daśās** ([guide](docs/guides/dashas.md)): Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla and
    Kālachakra, with applicability flags. The Daśā page has a tab per system and the zoom works for sign daśās too.
  - **Auxiliary** ([guide](docs/guides/auxiliary.md)): 37 sahams, avakahaḍā chakra, male/female rāśis, mṛtyu bhāga,
    ghāta chakra, vedhasthāna (natal and transit), jīva/deha and śodhana piṇḍas.
  - **Chakras** ([guide](docs/guides/chakras.md)): Sarvatobhadra with vedha lines, Kota and Saṅghaṭṭa, natal and
    transit layers.
  - **Annual** ([guide](docs/guides/annual.md)): varṣaphala (muntha, office-bearers, pañcavargīya bala, year lord),
    pañca pakṣī and KP significator tables by bhāva and graha.
- Known differences from the reference are recorded as expected failures with reasons (Kālachakra timing, the
  reference's office-bearer score, Kota score, Saṅghaṭṭa ring figure, one D40 boundary).

## [0.3.1] - 2026-11-21

Review rounds 1–4: everything below ships in 0.3.1. Schema 0.3.1 adds fields only.

### Added (review round 4)
- **Rāśi drishti for Uranus, Neptune and Pluto**, matching their graha drishti.
- **Additional aspects** (`vedic.drishti.additional`, off by default in the studio): a retrograde graha's shadow
  casts aspects from the previous sign, and grahas in parivartana cast aspects from their exchanged signs. They
  appear in chakra and chart marks (ʳ and ⇄), hover flows, the legend, an Aspects page table and the graha pages.
- **Docs:** a new guide, [Aspects (drishti)](docs/guides/aspects.md), covering every kind of aspect and convention.

### Added (review round 3)
- **Upagrahas** (`vedic.upagrahas`): Dhūma, Vyatīpāta, Pariveṣa, Indracāpa, Upaketu, Gulika and Māndi. All seven
  signs match the reference software for the Ramana Maharishi chart, which is now an accuracy check. They are shown
  on the Jyotish Chakra and charts and have their own point page.
- **Include switches:** Uranus, Neptune and Pluto, and the upagrahas, can be hidden everywhere (chakra, charts,
  legend, aspects).
- **Outer-planet aspects:** Uranus, Neptune and Pluto aspect the 5th, 7th and 9th (convention named in
  `vedic-core.yaml`).
- **Rename:** the chakra tab is now the **Jyotish Chakra**; the wheel is the **Graha-Nakshatra Mandala**.
- **Daśā:** a five-level zoom. Levels 4 and 5 are subdivided in the page by the engine's Vimśottarī proportions.
- **Bhāvas:**
  - lords in houses with bhāvāt bhāvam, exchanges and Vipareeta Rāja yogas;
  - each bhāva read as its own lagna, with a renumbered chart.
- **Concentric squares** (chalit and progression): the birth chart inside, an even band of twelve sectors outside.

### Changed (review round 3)
- **Aspects:** the page opens on the graha × graha matrix.
- **Strength:** the ratio cut-off sits in the ṣaḍbala table; the separate chart is gone.
- **Human Design:**
  - the standard chart is the default and the diagram has priority;
  - the silhouette is symmetric;
  - the schematic view is removed.

### Added (review round 2)
- **Aspects page:**
  - percent bar charts for D1 and D9: on houses, house + lord, on and from grahas, from each graha to the houses;
  - the virūpa table to the Śrīpati cusps.
  - Engine: `vedic.aspects` gains `houses`, `house_and_lord` and `house_madhya`, and D9 now uses navāṃśa
    longitudes.
- **Chakra:**
  - a markers ring (vargottama, pushkara navāṃśa and bhāga, gaṇḍānta, mūlatrikoṇa, uccha and nīca degrees);
  - nakshatra and rāśi names along the circle;
  - a graha legend with dignity, motion, combustion, speed and aspects.
- **Charts:** bhāva chalit over the rāśi chart, as concentric squares or a circle.
- **Bhāva pages:** bhāvāt bhāvam, the lord's story, occupants, kāraka, an animated connection map, and the bearing
  on the house and its significations.
- **Strength:** a ratio chart with a dotted cut-off at 1.0.
- **Progressions:** by completed revolutions (`report.revolutions@0.1`), with a natal and progressed overlay (square
  or circle), a date picker, play and a life timeline.
- **Numerology:**
  - `extended` numbers for every school (`numerology.extended@0.1`, `numerology.vedic@0.1`, `numerology.lo_shu@0.1`);
  - a filtered page where each number opens its meaning.
- **Human Design:**
  - the standard chart layout with variables and properties;
  - activations gain `tone` and `base` (schema 0.3.1).
- **Website:** PyPI links (navigation, hero, footer), `pip install eshana` in the hero, and an up-to-date status.
- **Tooling:**
  - `make preview` builds a local review folder;
  - `planning/PROGRESS.md` tracks progress against the plan (`make tracking`);
  - the reference chart library (CHK-01…22) is added to the tracker.
- **Journeys:** 25 studio journeys and 8 site journeys. The site suite renders without the GPU and samples a small
  canvas region; that fixed an intermittent headless hang.

### Added
- Studio: **yoga pages**. The yogas page lists only the yogas and doshas that hold. Each one opens a page showing:
  - its formation in this chart and strength checks for the grahas involved;
  - the chart with its signs lit and when it is active in Vimśottarī;
  - attributed reading notes (effects, how it is read, cancellations).
- Studio: **point pages**. Every lagna and special point has a tile and a page: what it is, why it matters, how to
  read it, its value here and the elements around its sign.
- Chakra:
  - hovering a graha animates its aspects (flowing curves, lit signs, outlined boxes);
  - **◇ rāśi drishti marks** in the chakra and the charts;
  - a **turn to selection** switch;
  - **transit date picker, from/to range and speed**.
- `eshana-report --transit-start/--transit-end/--transit-step-days`. By default the transit series runs daily, from a
  year before to two years after the reference.
- Website: a three-ring **gimbal** mark (site, studio and favicon); the headline drawn in **particles**; letter waves
  on headings. The motion kit gains `data-particles-max`.
- Journeys: 6 new studio journeys (17 in all) and a site journey for particles and the gimbal (8 in all). Every
  DevTools call now has a deadline, so a stuck call fails its journey instead of hanging the run.

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
