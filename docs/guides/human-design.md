---
title: "Human Design"
description: "The design instant, gate and line activations, and the derived BodyGraph attributes, with the rights position stated."
---

# Human Design

```bash
eshana human-design --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487
```

## What is computed

1. **Personality** instant: the birth instant.
2. **Design** instant: the moment the Sun's apparent longitude was exactly **88°** less than at birth, solved numerically.
   It is **not** "88 days earlier": for this birth it is 89.76 days (the Sun's apparent speed varies through the year). The result reports
   `design_arc_deg_achieved` (88.0000000001) so the solve is auditable.
3. **13 activations per side**: Sun, Earth, Moon, north node, south node, Mercury–Pluto, each mapped to
   `gate`, `line`, `color` and the owning `center`.
4. **BodyGraph** (`human_design.bodygraph`): active gates, defined channels, defined and undefined centres, `type`,
   `authority`, `profile` (personality Sun line / design Sun line) and `definition`.

```json
{ "type": "generator", "authority": "emotional", "profile": "2/5", "definition": "triple_split",
  "channels": ["9-52", "10-20", "37-40"], "defined_centers": ["throat", "g", "heart", "solar_plexus", "sacral", "root"] }
```

## Settings that matter

- **Node**: the true node is the default here (the usual convention) and is recorded as an assumption. Pass
  `--node mean` to change it.
- **Zodiac**: tropical only; a sidereal request is `ESH-CONFIG`.
- **Exact time required**: gate lines change within minutes. An unknown or approximate time is `ESH-TIME-UNCERTAIN`.
- **Boundaries**: a longitude within a few arc-minutes of a gate, line or colour edge can fall either side at the
  `analytic` provider's accuracy. Use `--provider spk` and treat a value near an edge as uncertain.

## What is deliberately not included

Gate numbers are I Ching hexagram numbers placed on the ecliptic starting at 2°00′ Aquarius, and the channel and centre
structure is publicly described. ESHANA implements that **structure and the mathematics** from first principles. It does
**not** include branded terminology beyond field names, descriptive text, bodygraph artwork or any third-party table.
Whether the arrangement or the nomenclature is protectable in a given jurisdiction is an open question tracked as
[ADR-009](../adr/009-human-design-rights.md); the result's assumptions say so every time. Take advice before a commercial
launch of this feature.

## Validation

Structural invariants are tested: the wheel is a permutation of 1–64 with 6 lines each, the 36 channels are unique and join
different centres, the centres partition the 64 gates, every gate is in a channel, type/authority/definition rules, and the
solver returns 88° exactly. Against an external reference chart, **not yet verified**: that is task `HD-04` and needs a
reference with documented conventions.
