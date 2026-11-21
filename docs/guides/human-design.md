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
   `gate`, `line`, `color`, `tone`, `base` (since schema 0.3.1) and the owning `center`.
4. **BodyGraph** (`human_design.bodygraph`): active gates, defined channels, defined and undefined centres, `type`,
   `authority`, `profile` (personality Sun line / design Sun line) and `definition`.

```json
{ "type": "generator", "authority": "emotional", "profile": "2/5", "definition": "triple_split",
  "channels": ["9-52", "10-20", "37-40"], "defined_centers": ["throat", "g", "heart", "solar_plexus", "sacral", "root"] }
```

## In ESHANA Studio

The **Standard chart** view lays the chart out as Human Design practitioners usually read it:
- the Design activations in a column on the left and the Personality activations on the right, each as gate.line;
- the BodyGraph between them, over a silhouette, with each channel half coloured by its gate (red Design, black
  Personality, striped both);
- the four variable arrows at the head;
- a properties panel: type, strategy, inner authority, definition, profile, incarnation cross (angle and the
  Sun/Earth gates), signature, not-self theme, determination, senses, motivation, perspective and environment.

The geometry is ESHANA's own. The studio derives strategy, signature and not-self theme from the type, and the
variables from the colour and tone of the Sun and nodes, using small labelled tables. **Detailed** and **Schematic**
views remain.

## Settings that matter

- **Node**: the true node is the default here (the usual convention) and is recorded as an assumption. Pass
  `--node mean` to change it.
- **Zodiac**: tropical only; a sidereal request is `ESH-CONFIG`.
- **Exact time required**: gate lines change within minutes. An unknown or approximate time is `ESH-TIME-UNCERTAIN`.
- **Boundaries**: a longitude within a few arc-minutes of a gate, line or colour edge can fall either side at the
  `analytic` provider's accuracy. Use `--provider spk` and treat a value near an edge as uncertain.

## What is deliberately not included

Gate numbers are I Ching hexagram numbers placed on the ecliptic starting at 2°00′ Aquarius, and the channel and centre
structure is publicly described. The engine implements that **structure and the mathematics** from first principles and returns no descriptive text.
Since 0.3.1 the studio also shows the system's conventional labels: strategy, signature, not-self theme and the
variable names, in small labelled tables. It does not show descriptive readings, third-party artwork or the
incarnation-cross name table.
Whether the arrangement or the nomenclature is protectable in a given jurisdiction is an open question tracked as
[ADR-009](../adr/009-human-design-rights.md); the result's assumptions say so every time. Take advice before a commercial
launch of this feature.

## Validation

Structural invariants are tested: the wheel is a permutation of 1–64 with 6 lines each, the 36 channels are unique and join
different centres, the centres partition the 64 gates, every gate is in a channel, type/authority/definition rules, and the
solver returns 88° exactly. Against an external reference chart, **not yet verified**: that is task `HD-04` and needs a
reference with documented conventions.
