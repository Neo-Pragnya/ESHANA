---
title: "Rule packs"
description: "The versioned rule packs, the YAML tables behind them, the condition language, and what changes a version."
---

# Rule packs

A rule pack is a named, versioned set of derivations for one tradition and convention. Every derived value in a result
names its pack as `id@version`. Tables and conditions are **YAML** ([ADR-012](../adr/012-yaml-rule-packs.md)) in
`crates/eshana-engine/rulesets/`; `eshana rulesets` prints them verbatim.

| Pack | Covers | Data |
|---|---|---|
| `vedic.parashari@0.3` | rāśi, nakshatra/pada, bhāva, vargas D1–D60 (D1, D2, D3, D4, D5, D6, D7, D8, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45, D60) with the degree in each varga, Vimśottarī (five levels), Yoginī | algorithms in code (`vedic.rs`) |
| `vedic.core@0.3` | sign lords, dignity (exaltation, debilitation, moolatrikoṇa, own), natural and five-fold relationships, graha drishti, combustion, dig bala, naisargika bala, avasthā; Jaimini kārakas, ārūḍhas, rāśi drishti; KP; upagrahas (Sun-based, Gulika, Māndi); outer planets aspect 5th, 7th and 9th (a modern convention); optional additional aspects from a retrograde graha's shadow and from parivartana; natural and functional benefics, yogi and avayogi offsets, dagdha rāśis by tithi, Indu lagna kalās | `vedic-core.yaml` |
| `vedic.bala@0.3` | ṣaḍbala (six components with named conventions), bhāva bala, viṃśopaka (four schemes, each summing to 20), sputa drishti in virūpas ([guide](../guides/strength.md)) | `bala.yaml` |
| `vedic.ashtakavarga@0.2` | bhinna- and sarvāṣṭakavarga | `ashtakavarga.yaml` |
| `vedic.panchanga@0.2` | tithi, vāra, nakṣatra, yoga, karaṇa; horā, choghaḍiyā, Rāhu kālam, Yamagaṇḍam, Gulika kālam; Tārā and Candra bala | `panchanga.yaml` |
| `vedic.yogas@0.2` | 18 yogas and 6 doshas as declarative conditions | `yogas.yaml` |
| `vedic.matching.ashtakoota@0.2` | aṣṭakūṭa (8 kūṭas, 36 points) | `matching.yaml` |
| `western.basic@0.1` | signs, house placement, Ptolemaic aspects | code |
| `human_design.basic@0.1` | design instant, wheel, channels, centres, type, authority, profile, definition | code |
| `numerology.pythagorean@0.1`, `numerology.chaldean@0.1` | letter tables, reductions, life path, personal year | code |
| `numerology.extended@0.1` | every Pythagorean and Chaldean number: core, karmic, planes, pinnacles, challenges, cycles ([guide](../guides/numerology.md)) | code (`numerology_ext.rs`) |
| `numerology.vedic@0.1`, `numerology.lo_shu@0.1` | mulank, bhagyank, name number, planets and number friendships; the Lo Shu grid and arrows | code (`numerology_ext.rs`) |
| `ayanamsha-<id>@0.1` | Lahiri, Raman, Krishnamurti, Fagan–Bradley | code |
| `report.markers@0.1` | pushkara navāṃśa, vargottama, gaṇḍānta markers in HTML reports | report renderer |
| `report.markers@0.2` | the chakra's markers ring: also pushkara bhāga degrees, mūlatrikoṇa ranges, uccha and nīca degrees | report renderer |
| `report.revolutions@0.1` | progression by completed revolutions: each sidereal return after birth moves a graha one sign on (the nodes backwards) | `eshana.report` |
| `report.nadi@0.1` | Nadi links: conjunct, trine, 2nd, 12th, 7th; retrograde reach ([guide](../guides/nadi.md)) | report renderer |

## Conventions named in the packs

| Choice | This pack set | Alternatives (would be another pack) |
|---|---|---|
| Lords of Scorpio and Aquarius | Mars and Saturn only | co-lordship of Ketu and Rahu |
| Rahu and Ketu aspects | 5th, 7th, 9th | 7th only |
| Chara kārakas | both the 7-kāraka and 8-kāraka (with Rahu) schemes are returned | — |
| Ārūḍha exception | pada in the house or its 7th moves to the 10th from there | other exceptions |
| Vashya and gana scoring | the tables written in `matching.yaml` | regional tables |
| Manglik | from lagna, Moon and Venus; cancellations not applied | cancellation rules |
| Sunrise | upper limb with refraction by default; `centre_refracted` and `centre_unrefracted` on request | — |
| Special lagnas | Sun at birth by default (`special_lagna_sun: sunrise` for the other reading) | — |

## The condition language (yogas and doshas)

All houses are whole-sign counts from a reference (`lagna`, `moon` or any graha):

```yaml
- id: gaja_kesari
  name: Gaja Kesari
  category: lunar
  rule: "Jupiter in a kendra (1, 4, 7, 10) from the Moon"
  when: {in_house: {graha: jupiter, from: moon, houses: [1, 4, 7, 10]}}
```

Conditions: `in_house`, `dignity`, `same_sign`, `occupied` (with `min` and `exclude`), `empty`, `lords_conjoined`,
`lord_in_house`, `lord_dignity`, `between_nodes`, `all_of`, `any_of`, `not`, and `for_each_graha` (with `$g`).
A result reports `present` (true, false, or null when it cannot be evaluated, for example a lagna rule without a birth
time) and the grahas that satisfied it. It never states what a yoga means.

## When does a version change?

When the output for some input could differ: a corrected table, a new convention default, a changed tie-break. Every
pack's fixed facts are checked at load and in CI (`packs::validate`): table sizes, aṣṭakavarga totals, a symmetric
yoni matrix, kūṭa maxima summing to 36.

## Adding or changing a pack

Edit the YAML, bump its `version`, and add worked examples to the tests. A new tradition needs: tradition id, source and
edition, the convention in prose, worked examples, boundary tests and a version. Loading your own packs at run time is
planned (`RULE-02`).
