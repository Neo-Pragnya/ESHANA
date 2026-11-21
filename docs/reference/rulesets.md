---
title: "Rule packs"
description: "The versioned rule packs, their sources, and what changes a version."
---

# Rule packs

A rule pack is a named, versioned set of derivations for one tradition and convention. Every derived value in a result names its
pack as `id@version`.

| Pack | Covers | Source and status |
|---|---|---|
| `vedic.parashari@0.1` | rāśi, nakshatra/pada, bhāva, D1/D3/D4/D7/D9/D10/D12, Vimśottarī | Parāśara conventions as widely published; implemented from the standard tables; unit-tested with worked values |
| `western.basic@0.1` | signs, houses placement, Ptolemaic aspects, applying/separating | Ptolemaic aspects; orbs are a tradition-level choice |
| `human_design.basic@0.1` | design instant, wheel, lines, channels, centres, type, authority, profile, definition | public structure; mathematics independent; **rights review pending (ADR-009)** |
| `numerology.pythagorean@0.1` | letter table, reductions, life path, personal year | as commonly published |
| `numerology.chaldean@0.1` | letter table, reductions | as commonly published |
| `ayanamsha-<id>@0.1` | Lahiri, Raman, Krishnamurti, Fagan–Bradley anchors | J2000 anchors from the Swiss Ephemeris reference values; Lahiri also from the 1956 CRC value |

## When does a version change?

When the output for some input could differ: a corrected table, a new convention default, a changed tie-break. Adding a
division that used to be `ESH-RULES` does not change the pack's version for the divisions it already served.

## Authoring a pack (planned)

Today packs are Rust modules (`crates/eshana-engine/src/{vedic,hd,numerology}.rs`) with their tables in code. A data-driven
format with a validator and a source-citation requirement is planned ([ADR-004](../adr/004-rule-packaging.md), task
`RULE-01`). A contribution that adds a tradition must include: tradition id, source and edition, the convention in prose,
worked examples from the source, boundary tests, and a version.
