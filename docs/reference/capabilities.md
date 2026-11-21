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
| South Indian and North Indian charts, side by side, any varga | verified (HTML studio) |
| Panchang chakra wheel: nakshatra, pada/navāṃśa, dreshkāṇa, rāśi and bhāva rings; exact-degree needles; drill-down | verified (HTML studio) |
| Layers: graha drishti, rāśi drishti, Western aspects, ārūḍha padas, SAV bindus, transits | verified (HTML studio) |
| Planet icons: textured spheres, glyphs or badges | verified (HTML studio) |
| Transit overlay with play and a date slider | verified (HTML studio) |
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
| Yukteshwar, true Chitrapakṣa, true Revatī and other star-based ayanāṃśas | planned (`AYA-02`) |
| Whole-sign, equal and Placidus houses (Placidus refused in the polar circles) | verified |
| Civil time with IANA zones, DST folds and gaps as errors; uncertain birth time | verified |

## Vedic

| Capability | State |
|---|---|
| Rāśi, nakshatra, pada, nakshatra lord, bhāva | verified |
| Vargas D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60 | verified |
| D150 (nāḍyāṃśa) and other non-Parāśara vargas | planned (`VED-09`) |
| Vimśottarī daśā to five levels with the current chain | verified |
| Yoginī daśā | verified |
| Chara daśā, Nārāyaṇa daśā, other conditional daśās | planned (`VED-10`) |
| Pañcāṅga: tithi, vāra, nakshatra, yoga, karaṇa with end times; sunrise, sunset, moonrise, moonset | verified |
| Horā, choghaḍiyā, Rāhu kālam, Yamagaṇḍam, Gulika kālam | verified |
| Tārā bala, Candra bala | verified |
| Bhāva, Horā and Ghaṭī lagnas | verified (PyJHora within 2′) |
| Dignity, avasthā, combustion, five-fold relationships | verified |
| Dig bala and naisargika bala | verified |
| Full ṣaḍbala (sthāna, kāla, ceṣṭā, dṛk bala) | planned (`VED-11`) |
| Graha drishti and rāśi drishti | verified |
| Bhinna- and sarvāṣṭakavarga | verified |
| Aṣṭakavarga transit strength by date | planned (`VED-12`) |
| Chara kārakas (7 and 8), kārakāṃśa, ārūḍha padas A1–A12 | verified |
| KP: sub and sub-sub lords, cusps, significators (house and planet view), ruling planets | verified |
| KP horary (praśna by number) | planned (`KP-02`) |
| Yogas (18) and doshas (Manglik, Kāla Sarpa, Grahaṇa, Shrapit) from a YAML pack | verified |
| Sāḍe Sātī windows with phases | verified (Saturn ingresses vs Swiss) |
| Aṣṭakūṭa matching with Manglik flags | verified |
| Rajju and vedha doshas, matching calendar | planned (`MAT-02`) |
| Muhūrta search | planned (`MUH-01`) |
| Varṣaphala (annual chart) | planned (`VAR-01`) |
| Upagrahas (Gulika, Māndi) | planned (`VED-13`) |
| Lal Kitab | planned (`LK-01`) |

## Western, Human Design, numerology, time series

| Capability | State |
|---|---|
| Western wheel, Placidus houses, Ptolemaic aspects with orb and phase, aspect grid | verified |
| Secondary progressions (a day for a year) | verified |
| Solar arc and other progressions, returns | planned (`WES-05`) |
| Human Design: design instant, gates and lines, channels, centres, type, authority, profile; detailed or schematic BodyGraph | verified |
| Numerology: Pythagorean and Chaldean | verified |
| Ephemeris tables at any step; sign ingresses including retrograde re-entries | verified |
| Stations, eclipses, returns | planned (`AST-06`) |
