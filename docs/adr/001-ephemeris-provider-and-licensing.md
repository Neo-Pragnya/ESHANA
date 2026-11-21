---
title: "ADR-001: Ephemeris provider and licensing"
description: "An abstract provider, two licence-clean implementations, and the Swiss Ephemeris kept out of the core."
---

# ADR-001: Ephemeris provider and licensing

**Status.** Accepted (trait, `analytic`, `spk`). **Open**: whether and how to ship a Swiss Ephemeris adapter.

> [!NOTE]
> **Amended by [ADR-011](011-open-source-distribution.md) (2026-11-20).** ESHANA is an open-source tool stack published as libraries, with no commercial launch. The open question here is closed there.

## Context

Accurate positions need an ephemeris. The Swiss Ephemeris is the de-facto reference but is AGPL or paid. NASA/JPL ephemerides are
public domain but large and need a reader. A library that is Apache-2.0 must not link AGPL code by default.

## Decision

1. Positions come through the `EphemerisProvider` trait with one frame for every provider.
2. Ship two providers that carry **no licence entanglement**: `analytic` (published elements and series, in source, no
   data files) and `spk` (reads a JPL kernel the *user* supplies).
3. Use the Swiss Ephemeris only as an **offline test oracle** (numbers, never code or data files).
4. A Swiss adapter, if built, is a separate crate behind a Cargo feature, off by default, with its AGPL/commercial
   consequences documented, after a written legal decision.

## Consequences

- Out of the box, accuracy is arc-minutes (`analytic`); arc-seconds need a 32 MB download (`spk`). Both are measured
  ([Accuracy](../reference/accuracy.md)).
- Switching providers changes digits, not definitions.
- No AGPL code in any published artefact; `cargo deny` enforces the allow-list.

## Open items

`PROV-03` legal review for a Swiss adapter; `PROV-04` an SPK type-3 (Chebyshev with velocity) reader for the lunar kernels if needed.
