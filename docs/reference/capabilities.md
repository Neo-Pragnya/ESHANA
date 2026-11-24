---
title: "Capabilities"
description: "Everything ESHANA computes and shows today, and what is planned, as one checklist."
---

# Capabilities

**Verified** means computed by the engine, covered by tests, and checked against an outside reference where one exists
([accuracy suite](../guides/accuracy-suite.md)). **Planned** items have a tracker id in `planning/backlog.json`.

## Charts and display

| Capability | State |
|---|---|
| Natal chart, Vedic (sidereal) and Western (tropical) | verified |
| Multi-page studio app: graha, bhāva, yoga and point pages, ⌘K palette, settings drawer, starfield, deep links | verified (HTML studio, browser journeys) |
| South Indian and North Indian charts, side by side, any varga, degrees and nakshatras, translucent aspect marks, ◇ rāśi drishti marks | verified (HTML studio) |
| South or North Indian chart in the centre of the Jyotish Chakra (Graha-Nakshatra Mandala) | verified (browser journeys) |
| Hover a graha to animate its aspects; optional turn to selection | verified (browser journeys) |
| Nadi direction matrix with degrees and links | verified (HTML studio, `report.nadi@0.1`) |
| AstroEnergetics: houses as body, dimension and question; quadrants, trikoṇas, energy bands | verified (HTML studio) |
| Yoga pages: present yogas only, formation, strength checks, daśā timing, attributed reading notes | verified (browser journeys) |
| Point pages for every lagna and special point: meaning, reading, value, sign elements | verified (browser journeys) |
| Chakra markers ring (vargottama, pushkara, gaṇḍānta, mūlatrikoṇa, uccha/nīca), curved names, graha legend | verified (browser journeys) |
| Bhāva chalit over the rāśi chart: concentric squares or circle | verified (browser journeys) |
| Bhāva depth: bhāvāt bhāvam, lord's story, kāraka, connection map, bearing on significations | verified (browser journeys) |
| Aspects on houses, house + lord, on and from grahas, to Śrīpati cusps, D1 and D9, as percent charts | verified (browser journeys); Jupiter and node conventions differ from one reference program (`CHK-23`) |
| Progression by completed revolutions with overlay, date picker and life timeline | verified (browser journeys) |
| Human Design standard chart: columns, BodyGraph over a silhouette, variables, properties | verified (browser journeys) |
| Numerology: Pythagorean, Chaldean, Indian and Lo Shu numbers with meanings and filters | verified (unit tests, browser journeys) |
| Jyotish Chakra wheel: nakshatra, pada/navāṃśa, dreshkāṇa, rāśi and bhāva rings; exact-degree needles; drill-down | verified (HTML studio) |
| Layers: graha drishti, rāśi drishti, Western aspects, ārūḍha padas, SAV bindus, transits | verified (HTML studio) |
| Planet icons: textured spheres, glyphs or badges | verified (HTML studio) |
| Transit overlay with play, a date picker, a from/to range and speed; `--transit-start`, `--transit-end` | verified (browser journeys) |
| Hover tooltips with sign, degree, nakshatra and pada | verified |
| Show/hide sections; light and dark; print and PDF; SVG and JSON export; copy a text summary | verified |
| Rotate the wheel to the lagna | verified |
| East Indian chart style | planned (`REP-06`) |
| Drag planets in a playground | planned (`REP-07`) |
| Languages other than English | planned (`REP-08`) |

## Positions, time and houses

| Capability | State |
|---|---|
| Ten planets and the node (mean or true), speed, retrograde | verified |
| Ayanāṃśas: Lahiri, Raman, Krishnamurti, Fagan–Bradley | verified |
| Yukteshwar, J. N. Bhasin, Ushā–Shashi, Sassanian, De Luce, Djwhal Khul (opt in), each with epoch, frame and source | verified (Swiss Ephemeris mean ayanāṃśa at eight epochs 1800–2100, within 0.05″) |
| True Chitrapakṣa, true Revatī and other star-based ayanāṃśas | planned (`AYA-02`) |
| Whole-sign, equal and Placidus houses (Placidus refused in the polar circles) | verified |
| Sripati (bhāva chalit) and Porphyry houses | verified (Swiss within 0.2″) |
| Declination and right ascension for every body | verified (published reference chart) |
| Civil time with IANA zones, DST folds and gaps as errors; uncertain birth time | verified |

## Vedic

| Capability | State |
|---|---|
| Rāśi, nakshatra, pada, nakshatra lord, bhāva | verified |
| Vargas D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60, with degree in varga | verified |
| D11 (rudrāṃśa) | planned (`VED-09`) |
| D150 (nāḍyāṃśa) and other non-Parāśara vargas | planned (`VED-09`) |
| Vimśottarī daśā to five levels with the current chain | verified |
| Yoginī daśā | verified |
| Chara daśā, Nārāyaṇa daśā | verified (see below) |
| Conditional daśās: Dvādaśottarī, Pañcottarī, Śatābdikā, Caturaśīti-sama, Dvisaptati-sama, Ṣaṣṭihāyanī, Ṣaṭtriṃśat-sama, each reporting its condition | implemented (hand-worked starts; a published worked example per system is pending, `DASH-01`) |
| Pañcāṅga: tithi, vāra, nakshatra, yoga, karaṇa with end times; sunrise, sunset, moonrise, moonset | verified |
| Horā, choghaḍiyā, Rāhu kālam, Yamagaṇḍam, Gulika kālam | verified |
| Tārā bala, Candra bala | verified |
| Bhāva, Horā and Ghaṭī lagnas | verified (PyJHora within 2′) |
| Dignity, avasthā, combustion, five-fold relationships | verified |
| Dig bala and naisargika bala | verified |
| Full ṣaḍbala (sthāna, dig, kāla, ceṣṭā, naisargika, dṛk) with required rūpas and ratio | implemented (`VED-11`); totals within ~60 virūpas of reference software, reconciliation `VED-14` |
| Iṣṭa and kaṣṭa phala | planned (`VED-14`) |
| Bhāva bala | verified (unit tests) |
| Viṃśopaka bala: ṣaḍvarga, saptavarga, daśavarga, ṣoḍaśavarga | verified (weights validated) |
| Sputa drishti in virūpas (D1 and D9 matrices) | verified (unit tests) |
| Yogi, avayogi, duplicate yogi, dagdha rāśis, Indu lagna, 64th navāṃśa, 22nd dreshkāṇa, kartari, graha yuddha, functional nature | verified (published reference chart) |
| Graha drishti and rāśi drishti (outer planets included: 5th, 7th, 9th and rāśi drishti) | verified |
| Additional aspects: retrograde shadow and parivartana ([guide](../guides/aspects.md)) | verified (unit tests, browser journeys) |
| Bhinna- and sarvāṣṭakavarga | verified |
| Aṣṭakavarga transit strength by date | planned (`VED-12`) |
| Chara kārakas (7 and 8), kārakāṃśa, ārūḍha padas A1–A12 | verified |
| KP: sub and sub-sub lords, cusps, significators (house and planet view), ruling planets | verified |
| KP horary (praśna by number): the 1–249 table, cusps from the number at the moment and place of judgement | verified (independent table construction; cusps vs Swiss Ephemeris `houses_armc` within 0.07″) |
| KP cusp occupancy as a named method: `cusp_start` (default) or `cusp_midpoint` (opt in) | verified |
| Method registry (`eshana methods`) and the `ESH-METHOD`, `ESH-RANGE-INVALID`, `ESH-LIMIT` codes | verified |
| Yogas (18) and doshas (Manglik, Kāla Sarpa, Grahaṇa, Shrapit) from a YAML pack | verified |
| Sāḍe Sātī windows with phases | verified (Saturn ingresses vs Swiss) |
| Aṣṭakūṭa matching with Manglik flags | verified |
| Rajju and vedha doshas, matching calendar | planned (`MAT-02`) |
| Muhūrta search | planned (`MUH-01`) |
| Upagrahas: Dhūma, Vyatīpāta, Pariveṣa, Indracāpa, Upaketu, Gulika, Māndi | verified (published reference chart) |
| Maitri: natural, temporal and compound relationships of the nine grahas, nakshatra tārā matrix, dignities in 16 divisions ([guide](../guides/maitri.md)) | verified (published reference chart, 305/306 cells) |
| More daśās: Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla ([guide](../guides/dashas.md)) | verified (published reference chart, to the day or ±1 day) |
| Kālachakra daśā (BPHS pada tables) | implemented; the reference's timing convention differs (xfail) |
| Sahams (37), avakahaḍā, male/female rāśis, mṛtyu bhāga, ghāta chakra, vedhasthāna, jīva/deha, śodhana piṇḍas ([guide](../guides/auxiliary.md)) | verified (published reference chart) |
| Sarvatobhadra with vedha, Kota and Saṅghaṭṭa chakras, natal and transit ([guide](../guides/chakras.md)) | verified (placements and layout vs published reference chart) |
| Varṣaphala: muntha, pañcādhikārīs, pañcavargīya bala, year lord ([guide](../guides/annual.md)) | verified (strengths, muntha, office-bearers); year-lord score differs (xfail) |
| Pañca pakṣī: birth bird, ruling and dying birds, day and night yāma tables with sub-periods, ±3 days | implemented (birds verified; first-yāma activity convention differs) |
| Parivartana yoga and its Mahā, Khala and Dainya kinds | verified (unit test; Ramana chart: Jupiter–Saturn, Dainya) |
| KP significator tables by bhāva and graha, equal-cusp lords | verified (published reference chart) |
| Lal Kitab | planned (`LK-01`) |

## Western, Human Design, numerology, time series

| Capability | State |
|---|---|
| Western wheel, Placidus houses, Ptolemaic aspects with orb and phase, aspect grid | verified |
| Secondary progressions (a day for a year) | verified |
| Secondary progressions as an engine operation: explicit fractional age and year length, angles by solar arc, Naibod or quotidian | verified (method identities) |
| Further vargas by method id: cyclic D2, D3, D81, D108 and D144 (D12 of D12) | verified (every boundary) |
| Solar arc and other progressions, returns | planned (`WES-05`) |
| Human Design: design instant, gates and lines, channels, centres, type, authority, profile; detailed or schematic BodyGraph | verified |
| Numerology: Pythagorean and Chaldean, extended numbers, Indian (mulank, bhagyank), Lo Shu | verified |
| Numerology: Kua by gender, Chaldean compounds 10–52, growth, intensity, letter transits and essence by year | verified (engine tests, studio journeys) |
| Ephemeris tables at any step; sign ingresses including retrograde re-entries | verified |
| Stations and retrograde intervals (clipped at the range), crossings of a point or aspect with retrograde loops and touches | verified (Swiss Ephemeris oracle, [accuracy](accuracy.md)) |
| Transit Aṣṭakavarga as dated `[start, end)` segments, each naming the natal BAV and SAV cell read; nodes unavailable | verified (hand-computed cells, ingress boundaries) |
| Muhūrta windows: factor timelines composed by partition, evaluation and merging, versioned profiles, empty results explained | verified (profiles: stable definitions; purpose presets experimental pending review) |
| Personal calendar: tārā and candra bala, birth and event places apart, DST and polar days, JSON, CSV and ICS, keyboard day grid in the studio | verified |
| Supplementary matching beside the 36 points: Rajju, Vedha, Mahendra, Strī-dīrgha, Maṅgala doṣa with a cancellation trace, pair Aṣṭakavarga, cross-chart aspects | verified (36-point output unchanged byte-for-byte) |
| Pair calendar: windows that suit both people and the event, explained per window | verified |
| Studio as a local application: live engine (WebAssembly), compare workspace, keyboard playground, profile library, interface catalogues | verified (ADR-013 accepted; Hindi catalogue reviewed; language is a reader setting, English by default) |
| Guided chart creation: `eshana-report new` opens a local form, waits, refuses invalid details with the engine's code, writes the chart and stops | verified (Python tests, browser journey) |
| Download any chart drawing as PNG (captioned, ≥ 1600 px) or standalone SVG, as currently shown | verified (browser journeys) |
| Eclipses, returns as a search | planned (`AST-06`) |
