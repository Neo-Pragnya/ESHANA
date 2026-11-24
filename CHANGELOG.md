# Changelog

All notable changes. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow SemVer. `schema_version` of the
result envelope is versioned separately (see `docs/concepts/provenance.md`).

## [Unreleased]

## [0.5.0] - 2026-11-24

The roadmap waves (planning/WAVES.md): time and ephemeris foundations, events and asteroids, rule packs and
traditions, outputs and bindings, and quality gates.

### Added
- Calendars and BCE dates (TIME-03): `when.calendar` = `gregorian` (default, proleptic), `julian` or `auto` (the
  1582 cut-over; the ten skipped days are refused); signed astronomical years (`-0043-03-15`); the date as written is
  kept in `input.calendar` and a converted date is recorded as an assumption.
- ΔT across five millennia (TIME-05): the full Espenak–Meeus set (−500 to 2150) and the long-term parabola outside
  it; after 2027 a forecast from IERS Bulletin A (Vol. XXXIX No. 047) blended into the long-term curve by 2050.
- Kernel catalogue (PROV-06): `eshana kernel list | verify | path`, NAIF's published MD5 for DE440s, DE440, DE441
  (both parts) and DE442; a catalogue name works as `--spk`; `scripts/fetch_kernel.py` downloads and checks one.
- SPK type-3 segments, and kernels over 256 MB read on demand (PROV-04).
- `rise_set` operation and `eshana rise-set` (AST-08): rise, upper transit and set of any body over up to 366 local
  days, with polar days flagged, and civil, nautical and astronomical twilight; within 30 s of an independent oracle
  at four places including Tromsø.
- Topocentric positions (AST-08): `config.topocentric` / `--topocentric` apply the parallax for the place and its
  altitude; `calculation.positions_reference` records it.
- True (star-anchored) ayanāṃśas (AYA-02): `true_citra`, `true_revati`, `true_pushya` and `true_mula` hold Spica,
  ζ Psc, δ Cnc and λ Sco at their anchors, from Hipparcos positions and proper motions with aberration and light
  deflection; within 0.62″ of an independent oracle at 160 instants over 1800–2200.
- House systems (AST-03): Koch, Regiomontanus, Campanus, Alcabitius, Topocentric (Polich–Page), Morinus and
  Meridian, each within 0.01° of an independent oracle across seven latitudes and twelve sidereal times; Porphyry and
  Śrīpati now listed in `eshana methods`.

- `returns` operation and `eshana returns` (WES-03, AST-06): every return of the Sun, the Moon or any body to its
  natal longitude in a range, tropical or sidereal, with a chart at each and optional relocation; within a minute
  (Sun) and 30 s (Moon) of an independent oracle.
- Solar arc directions (WES-05): the progressions result carries every natal point moved by the solar arc and the
  directed-to-natal aspects within 1°.
- `gochara` operation and `eshana gochara` (VED-06): sign segments with the house from the natal Moon and lagna,
  retrograde intervals and combustion intervals from the vedic-core orbs.
- `eclipses` operation and `eshana eclipses` (AST-06): solar (partial, annular, total, hybrid) and lunar (penumbral,
  partial, total) eclipses from the provider's own Sun and Moon; every eclipse of 2000–2040 matches an independent
  oracle in type and within 2 minutes (plus the faint penumbral eclipse of 2016-08-18 that NASA's canon lists).
- Optional bodies (AST-07): Ceres, Pallas, Juno, Vesta (JPL's `sb441-n16` kernel, in the catalogue) and Chiron (a JPL
  Horizons kernel, `scripts/fetch_kernel.py horizons:2060`), within 1″ of Horizons; several kernels at once
  (`--spk de440,sb441-n16`); SPK type 21.

- Western essential dignities (WES-04): `features: ["dignities"]` on a western chart; domicile, exaltation,
  Dorothean triplicity by sect, Egyptian terms, Chaldean faces, detriment and fall, scores, peregrine planets and
  mutual receptions, from the new `western.dignities` pack (the terms reach the classical totals 79, 82, 76, 66, 57).
- `synastry` operation and `eshana synastry` (WES-02): inter-aspects, house overlays both ways, a midpoint composite
  (cusp midpoints) and a Davison chart (midpoint in time and space).
- Gematria and abjad (NUM-04): numerology schools `hebrew_gematria` and `abjad` for names written in Hebrew or Arabic
  letters (nothing is transliterated); the basmala gives the classic 786.
- Varga schools (VED-03, VED-09): `d30_parivritti`, `d60_parivritti` and `d150_parivritti` beside the Parāśari
  defaults.
- User rule packs (RULE-02): yoga packs given with a chart request (`user_rule_packs`, `--rule-pack FILE`), checked by
  the same grammar and reported under their own id.
- Lal Kitab (LK-01, **draft**): `features: ["lal_kitab"]`: the Lal Kitab chart and pakka ghar, pending the owner's
  review of the table.

- Graph exports (GRA-03, GRA-04): `eshana graph … --format svg | drawio | chitkala`: a self-contained SVG laid out by
  node type, a diagrams.net file, and Chitkala's import schema for interactive exploration.
- JavaScript (BIND-03, BIND-04): the `eshana` package in `bindings/js`, the engine as WebAssembly for browsers and
  Node 18+, with TypeScript types and a parity test against the CLI (ADR-014).
- MCP server (BIND-05): `eshana-mcp` serves eleven tools over stdio (charts, positions, Human
  Design, numerology, pañcāṅga, eclipses, rise and set, comparisons, `explain_derivation` and raw requests) so an assistant can compute charts locally
  (`claude mcp add eshana -- eshana-mcp`); refusals come back with their `ESH-*` code.
- East Indian chart style (REP-06): `--chart-style east` and an East Indian button on Charts and Vargas.
- Playground drag (REP-07): drag a graha round the wheel; the engine recomputes on release and Ctrl/⌘+Z undoes.

- Fuzzing (QA-06): `fuzz/` holds cargo-fuzz targets for the time parsers, the raw request entry point and chart
  requests with fuzzed fields; `make fuzz`, and five minutes per target weekly in CI.
- Benchmarks (PERF-01): `make perf` times cold start, one full chart and batches of 100 and 1,000 charts against fixed
  budgets and, on a machine with a recorded baseline, a 1.5× regression threshold; budgets run in CI.
- Supply chain (SEC-03): a CycloneDX SBOM of every crate in the wheel (`make sbom`), and an `attest` workflow that
  checks the files PyPI serves, attests provenance and the SBOM, and creates the GitHub release with checksums.
- Cookbook (DOC-04): four notebooks (first chart, comparing methods, sky events, agent recipes) that the test suite
  runs cell by cell, and a cookbook guide.

### Changed
- `vedic.core` split (VED-05): dignities, relationships, drishti and strength are now packs of their own
  (`vedic.dignity`, `vedic.relationships`, `vedic.drishti`, `vedic.strength`), values unchanged; attributes cite them.
- The `analytic` provider (0.2.0) uses VSOP87B for the Sun and Mercury to Neptune: worst error against JPL DE440s
  from 17′ (Saturn) and 16″ (Sun) to under 3″ for every planet and 0.3″ for the Sun. Sunrise, returns, ingresses and
  every planetary position are more accurate with no data files; the accuracy tolerances are tightened to match.

### Fixed
- Dates before 1582 now round-trip: the Julian-day inverse used the Julian calendar before 1582 while the forward
  conversion was proleptic Gregorian.
- ΔT before 1800 no longer extrapolates the 1800–1860 polynomial (the year 100 gave a nonsensical instant).

### Changed
- The download buttons' tooltips and screen-reader labels follow the interface language (Hindi strings pending the
  owner's review, listed in `pending_review`).
- README: the release-pipeline diagram is three readable rows (gates, ship, publish) and shows the platform wheels and
  the install checks; every Mermaid diagram is checked to render.

## [0.4.4] - 2026-11-24

### Added
- Charts page: the chakra's layers on the South and North Indian charts. Drishti arrows (drawn in), rāśi drishti
  lines, Western aspects, transits as a Gochara chart that follows the date slider and play, the hover animation of a
  graha's aspects between boxes, and grahas that animate in (switch: Motion).
- Vargas page: pick any varga (menu or gallery) to see it full width with its own aspects: graha and rāśi marks,
  arrows, lines, additional aspects and the hover animation, from the new engine attribute `vedic.drishti.vargas`
  (the rāśi-chart rules cast between the signs the grahas occupy in that varga). Same style, icon and layer switches
  as Charts.
- Aspects page: a multi-select of D1 and every varga in the file; the chosen charts are shown one under another.
  The engine's `vedic.aspects` now covers every requested division (`divisions`, `dN`), with houses from each varga's
  lagna.
- Hover animation on the Jyotish Chakra, Charts and Vargas: graha drishti (bold dashed line, round marker, the
  house count) and rāśi drishti (dotted line, ◇ marker) are drawn differently, and the **Hover animation** switches
  turn each off on its own, or both for no animation.
- Maitri and dignities: the count chips (Great friend, each tārā, Exalted, Own…) filter their grid; several can be on
  at once, and Show all clears them.
- The example reports carry a 14-day personal calendar, so the Calendar page is filled in the docs.

### Changed
- The PNG and SVG download buttons sit above a chart, outside the drawing (round wheels keep the corner).

## [0.4.3] - 2026-11-24

### Added
- PyPI project links: Changelog and Issues, next to Homepage, Repository and Documentation.
- Linux wheels (x86_64 and aarch64, manylinux2014) on PyPI alongside the macOS arm64 wheel, so `pip install eshana` on
  Linux no longer needs a Rust compiler.

### Fixed
- CI installs the `wasm32-unknown-unknown` target from `rust-toolchain.toml`; the browser journeys, the acceptance
  gate and the wheel scan build the in-browser engine and failed on runners without it.

## [0.4.2] - 2026-11-24

### Changed
- README for GitHub: light and dark heroes (`<picture>`), an animated muhūrta-composition diagram drawn from a real
  run (`assets/readme/src/muhurta_svg.py`), Mermaid diagrams (crate architecture, the `eshana-report new` sequence,
  the release pipeline), GitHub alerts, formulas in LaTeX, a Python cookbook whose outputs were run, an FAQ, the full
  23-view gallery, and reference tables (operations, CLI commands, method families, error codes) generated from the
  engine by `scripts/readme_sections.py` and checked by `make docs-check`.
- The PyPI description follows it, with GitHub-only syntax converted (Mermaid → a link, alerts → bold notes, math →
  code).

## [0.4.1] - 2026-11-24

### Changed
- README rewritten: an animated hero, animated terminal walkthroughs made from real output, an animated "how it works"
  flow, a gallery linking to the live studio, measured accuracy and guarantees. The PyPI description is generated from
  it (`scripts/pypi_readme.py`, checked by `make docs-check`) with images served from the docs site.
- The website publishes the README images and a new example, "The studio as an app" (live engine, calendar, KP
  horary, partner), with gallery views of the calendar, KP horary, compare workspace and playground.
- The docs-site workflow also runs on release tags and on demand, and the publishing guide ends with checking the
  site deploy.

### Fixed
- Form controls follow the theme: in the dark theme the date pickers and selects on the Compare and Progressions
  pages had pale text on a white field (contrast 1.1:1). A browser journey now checks every control on every page in
  both themes.

## [0.4.0] - 2026-11-24

### Added
- Download any chart as an image (`APP-08`): every chart drawing in the studio has PNG and SVG buttons; the export is
  a snapshot of the current state (layers, filters, varga, rotation, theme, language). The PNG is at least 1600 px wide
  with a caption; the SVG has every style resolved so it stands alone.

### Fixed
- Chart text hierarchy (`REP-20`): grahas stay the largest, bold text in every chart cell; ārūḍha padas and aspect marks
  are secondary (at most 0.64 of the graha size, lighter weight). Crowded signs move grahas into two columns before
  shrinking them. Flags are raised superscripts in the Nadi quadrants too (`Mo V` → Mo<sup>V</sup>), and a crowded Nadi
  quadrant uses two columns when that gives larger text.

- Guided chart creation (`APP-07`): `eshana-report new` opens a form on `127.0.0.1` (random token, host and origin
  checked, no external assets), waits, refuses invalid details with the engine's error code while keeping what was
  typed, writes one chart to a new file (never overwriting), optionally saves a profile revision (`--library`), shows
  the chart, then stops and prints the path. `--timeout` (exit 3) and Ctrl-C (exit 130) write nothing.
- Event search (`EVT-01`, `EVT-02`): one solver for every event search: samples, bisection to a stated tolerance,
  angles as signed separations so 359.99° → 0.01° is a crossing, and the extremum between samples searched so a
  retrograde loop or a touch inside one step is not missed.
- `stations` (`eshana stations`, `ChartEngine.stations`): stations in the requested frame and the retrograde
  intervals they bound, flagged when clipped at the range. Checked against the Swiss Ephemeris (offline oracle).
- `crossings` (`eshana crossings`, `ChartEngine.crossings`): when a body reaches a point or an aspect to it, with
  direction and residual; touches that stop within an orb are reported apart; `truncated` past `max_results`.
- Envelope kinds `stations` and `crossings` (schema adds values only).
- Method registry (`BASE-02`): `methods` (`eshana methods`, `ChartEngine.methods`) lists every method family and id;
  `info` names the families. Error codes `ESH-METHOD` (unknown method id), `ESH-RANGE-INVALID` (empty or reversed
  range) and `ESH-LIMIT` (past a documented work limit). Schema `$defs` for the unavailable, interval, method and
  configuration-hash shapes; `calculation.config_hash` on charts and event searches.

- Dated Aṣṭakavarga (`EVT-03`): `ashtakavarga_transits` (`eshana ashtakavarga-transits`,
  `ChartEngine.ashtakavarga_transits`): per graha, `[start, end)` segments that change exactly at sign ingresses,
  each naming the natal BAV and SAV cell it read. Nodes are typed unavailable; no table is invented for them.
- Muhūrta windows (`MUH-02`, `MUH-03`): `muhurta_windows` (`eshana muhurta`, `ChartEngine.muhurta_windows`) cuts a
  range at every boundary of the factors named, evaluates each piece and merges neighbours whose values and verdicts
  are equal; an empty result names the constraints that removed the candidates. Profiles are versioned data
  (`muhurta-profiles.yaml`); purpose presets are labelled experimental and list what they lack.
- Personal calendar (`MUH-04`): `personal_calendar` (`eshana calendar`, `ChartEngine.personal_calendar`) adds tārā and
  candra bala from the natal Moon, day summaries in the event zone, and CSV and ICS exports. The studio gains a
  keyboard-driven calendar page (`eshana-report --calendar-days N`).
- `eshana_time::zone_offset_seconds`: a zone's offset at an instant (DST and historic offsets).
- Supplementary matching (`MAT-03` to `MAT-06`): `matching` gains a `supplementary` section from its own pack
  (`matching-supplementary.yaml`): Rajju, Vedha, Mahendra and Strī-dīrgha; Maṅgala doṣa per reference point with
  every cancellation and its scope; each partner's grahas in the other's Aṣṭakavarga; cross-chart sign aspects. The
  36-point Aṣṭakūṭa and the manglik flags are unchanged byte for byte.
- Pair calendar (`MAT-07`): `pair_calendar` (`eshana pair-calendar`, `ChartEngine.pair_calendar`).
- Conditional daśās (`DASH-01`): Dvādaśottarī, Pañcottarī, Śatābdikā, Caturaśīti-sama, Dvisaptati-sama, Ṣaṣṭihāyanī
  and Ṣaṭtriṃśat-sama as `vedic.dasha.*` (feature `dashas_extended`), each with its condition reported in
  `applicability` and never applied silently; studio tabs for all sixteen systems.
- Ayanāṃśas (`METH-01`): Yukteshwar, J. N. Bhasin, Ushā–Shashi, Sassanian, De Luce and Djwhal Khul (opt in), each
  with its epoch, frame and source in `info.ayanamsha_definitions`; all ten models match the Swiss Ephemeris mean
  ayanāṃśa (offline oracle) within 0.05″ from 1800 to 2100.
- Further vargas (`METH-02`): `varga_methods` on Vedic charts (`--varga-methods`), family `varga_method`: cyclic D2,
  D3, D81, D108 and D144 (D12 of D12); each result names its method and school.
- Secondary progressions (`METH-03`): `progressions` (`eshana progress`, `ChartEngine.progressions`) with explicit
  fractional age and year length; angles only under a named `progression_angles` method.
- `info.operations` lists every operation; `scripts/check_capabilities.py` (`make capabilities-check`, also run by the
  Python tests) checks the reference docs against the installed wheel in both directions: operations, error codes,
  CLI commands, and `docs/reference/methods.md`, which it generates from the method registry.
- Browser engine (`APP-01`, ADR-013, proposed): `crates/eshana-wasm` compiles `execute_json` to WebAssembly (`make wasm`);
  `eshana-report --live` embeds it so pages recompute offline. `acceptance/studio/wasm_parity.mjs` checks every
  operation family against the native CLI (equal structure, floats within 1e-9).
- Profile library (`APP-02`): `eshana.profiles.ProfileLibrary`, revisioned, with lossless export/import and replay
  that checks the stored birth UTC (local mean time, war time and double summer time tested).
- Studio: a compare workspace (`APP-03`) and a keyboard playground (`APP-06`) over the new `synthetic_chart`
  operation (`APP-05`, `ChartEngine.synthetic_chart`: factors from hand-placed positions, no ephemeris, labelled
  synthetic); interface catalogues (`APP-04`, `--locale en|hi`, Hindi reviewed) that never change a value. Every studio file carries
  every catalogue; the language is a reader setting in the settings drawer (kept in the browser), English by default.
- Research models (`RES-01` to `RES-04`): `docs/design/research-models.md` specifies every score before it is built;
  `research_window_score` (`research.window_score@0.1`, experimental) orders passing muhūrta windows with
  contributions, sensitivity and coverage; a birth-time evidence evaluator is specified (not built); `eshana.content`
  keeps attributed text beside envelopes, keyed by the engine's factor ids, with naming adapters.
- KP horary (`KP-03` to `KP-06`): `kp_table` (`eshana kp-table`) lists the 249 numbers, built in integer arc-seconds
  and checked against an independent construction; `kp_horary` (`eshana kp-horary`, `ChartEngine.kp_horary`) fixes
  the Ascendant at the start of the number's arc and takes Placidus cusps for the sidereal time at which it rises at
  the place of judgement (checked against Swiss Ephemeris `houses_armc`, offline, within 0.07″). The studio gains a
  KP horary page (`eshana-report --kp-number N --kp-at UTC`).
- KP cusp occupancy is a named method: `kp_cusp_occupancy` = `cusp_start` (default, unchanged) or `cusp_midpoint`
  (opt in), on charts (`--kp-occupancy`) and horary. `vedic.kp` names `occupancy_method`.

### Changed
- `stations` and `crossings` (unreleased) refuse a reversed range with `ESH-RANGE-INVALID` and a span past 200 years
  with `ESH-LIMIT` instead of `ESH-INPUT`.
- `ingresses` uses the shared bisection with the same 30 halvings: results are unchanged.

## [0.3.3] - 2026-11-22

Review round 8. Schema 0.3.3 adds fields only.

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
