---
title: "Naming"
description: "The canonical expansion, the discarded alternatives, and what must be checked before publication."
---

# Naming

**ESHANA** is a brand transliteration inspired by *eṣaṇā* (एषणा), "seeking, quest". It is not a literal acronym from
Sanskrit and is distinct from *Īśāna* (ईशान), a name of Śiva.

## Canonical expansion

> **E**xtensible **S**patiotemporal **H**ypersystem for **A**stroinformatics, **N**atural Cycles & **A**ssociations

Use this one expansion everywhere (manifests, docs, site, CLI help). The decision rule from the master plan stands: *one
canonical brand expansion; other phrases describe capabilities; never vary the expansion between artefacts.* A test
(`doctor` prints it; the acceptance suite asserts it) keeps the string from drifting.

The master plan's first draft proposed "Extensible Symbolic Hub for Astronomy, Numerology & Archetypes" (and a
technical subtitle). The project owner chose the expansion above instead: it names the spatial and temporal frame
(*spatiotemporal*), the engineering (*system*), the data science of celestial and calendrical quantities
(*astroinformatics*), the cycles that make up the domain, and the associations the knowledge graph records. The
earlier candidates remain in the [master plan](../design/master-plan.md) as record.

## Tagline

*From celestial positions to connected meaning.* Developer-facing alternative: *Compute precisely. Interpret
transparently.*

## Clearance (decided)

ESHANA is an open-source tool stack, not a commercial product ([ADR-011](../adr/011-open-source-distribution.md)).
For a non-commercial open-source release the owner accepted the residual naming risk instead of a formal trademark
search. Registry checks on 2026-11-20:

| Registry | Name | Result |
|---|---|---|
| PyPI | `eshana` | free (HTTP 404), now the published distribution name |
| crates.io | `eshana` | free (HTTP 404); crates publish later (`BIND-07`) |
| npm | `@neo-pragnya/eshana` | not checked; no npm package yet |

If a prior mark ever forces a change, the distribution is renamed and the import name `eshana` stays.
