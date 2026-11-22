---
title: "Vedic features"
description: "Strength and dignity, graha and rāśi drishti, aṣṭakavarga, Jaimini kārakas and ārūḍhas, KP sub-lords and significators, yogas and doshas, Sāḍe Sātī, vargas to D60 and daśā to five levels."
---

# Vedic features

A Vedic chart can carry extra sections, each computed by the engine from a YAML rule pack and labelled
`tradition_rule` with its pack version. Ask for them by name, or all at once:

```bash
eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
  --features all --divisions 1,2,3,4,7,9,10,12,16,20,24,27,30,40,45,60 \
  --dasha --dasha-levels 3 --dasha-systems vimshottari,yogini --reference-utc 2026-11-21T00:00:00Z
```

```python
ALL = [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60]
chart = ChartEngine().vedic_chart(ctx, ayanamsha="lahiri", features=["all"], divisions=ALL,
                                  dasha=True, dasha_levels=3, dasha_systems=["vimshottari", "yogini"],
                                  reference_utc="2026-11-21T00:00:00Z")
chart.attribute("vedic.ashtakavarga")["sarvashtakavarga"]["total"]   # 337
```

| Feature | Attribute id | What it holds | Needs |
|---|---|---|---|
| `strength` | `vedic.strength` | dignity (exalted, debilitated, moolatrikoṇa, own, friend, neutral, enemy), bālādi avasthā, retrograde, combust, dig bala, naisargika bala | time and place for dig bala |
| `relationships` | `vedic.relationships` | pañcadhā maitrī for every pair of the seven grahas: natural, temporary, combined | — |
| `drishti` | `vedic.drishti.graha`, `vedic.drishti.rashi` | graha drishti (7th for all; Mars 4, 8; Jupiter 5, 9; Saturn 3, 10; nodes 5, 9) with aspected signs, bhāvas and grahas; rāśi drishti | — |
| `ashtakavarga` | `vedic.ashtakavarga` | bhinnāṣṭakavarga per graha with each contributor's bindus; sarvāṣṭakavarga by sign | lagna |
| `jaimini` | `vedic.jaimini.chara_karakas`, `…arudha_padas`, `…rashi_drishti` | chara kārakas (7 and 8 schemes), kārakāṃśa, ārūḍha padas A1 (AL) to A12 (UL) | lagna for ārūḍhas |
| `kp` | `vedic.kp` | Placidus cusps with sign, star, sub and sub-sub lords; planets' lords; significators in house view (A, B, C, D) and planet view; ruling planets | time and place |
| `panchanga` | `vedic.panchanga` | see [pañcāṅga](panchanga.md) | place |
| `yogas` | `vedic.yogas` | 22 yogas, including Parivartana yoga and its Mahā, Khala and Dainya kinds, and 6 doshas (Manglik from lagna, Moon and Venus; Kāla Sarpa; Grahaṇa; Shrapit) with the rule in words and the grahas that satisfied it | lagna for lagna rules |
| `sade_sati` | `vedic.sade_sati` | Saturn over the 12th, 1st and 2nd from the natal Moon: every window with its rising, peak and setting phases, retrograde re-entries included, and whether it is active at `reference_utc` | — |

## Daśā

- `dasha_levels` 1–5 nests Vimśottarī: mahādaśā, antardaśā, pratyantardaśā, sūkṣma and prāṇa. Three levels are 729
  periods; deeper levels are returned only as the **current chain**.
- `dasha_systems: [yogini]` adds Yoginī daśā (eight yoginīs, 36-year cycle; the first is (janma nakshatra + 3) mod 8).
- `reference_utc` adds `current`: the five-level chain of periods containing that instant. Without it nothing is
  "current", so a result never depends on the computer's clock.

## Vargas

D1, D2 (horā), D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30 (unequal triṃśāṃśa), D40, D45 and D60, by the
Parāśara starting-sign rules written out in `vedic.rs` and re-derived independently in the accuracy suite.

## What is not here yet

The full ṣaḍbala (only dig and naisargika bala are computed), Chara daśā and other conditional daśās, varṣaphala,
muhūrta search, praśna and Lal Kitab are tracked in [capabilities](../reference/capabilities.md).

Aspects of every kind, including the optional additional aspects (retrograde shadow and parivartana), are explained in [Aspects (drishti)](aspects.md).
