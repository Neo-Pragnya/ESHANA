---
title: "Licensing"
description: "What ESHANA ships under, what it depends on, and what it deliberately does not bundle."
---

# Licensing

## ESHANA

ESHANA is an **open-source tool stack published as libraries**: there is no commercial launch
([ADR-011](../adr/011-open-source-distribution.md)). **Apache-2.0** for every crate and the Python package ([LICENSE](https://github.com/Neo-Pragnya/ESHANA/blob/main/LICENSE)).
The permissive core holds *only while every dependency allows it* ([ADR-008](../adr/008-licensing-model.md)).

## Dependencies (0.1)

Rust: `serde`, `serde_json`, `thiserror`, `jiff` and `jiff-tzdb` (bundle the IANA tz database, public domain),
`clap`, `unicode-normalization`; Python binding: `pyo3`. All MIT or Apache-2.0 (or both). `cargo deny` enforces an
allow-list in CI; a copyleft or unknown licence fails the build. See `THIRD_PARTY_NOTICES.md` and `deny.toml`.

## Data

- **IANA tz database**: public domain.
- **JPL SPK kernels** (optional): NASA/JPL data, public domain, **not bundled**; the user downloads them.
- **VSOP87B** (P. Bretagnon and G. Francou, Bureau des longitudes / IMCCE, 1988), the planetary theory's published
  coefficients, truncated by `scripts/vsop87_tables.py` with each source file's SHA-256 recorded: scientific data,
  freely distributed by IMCCE.
- **Standish Keplerian elements** (JPL, published tables, for Pluto) and **Meeus series** (formulae from a published
  book, implemented independently): numerical constants and algorithms, no code copied.
- **Golden fixtures**: positions computed with Skyfield against DE440s; numbers from the Swiss Ephemeris used as an
  offline oracle. Facts (numbers) only; no code or data files from either ship in a package.

## What is not bundled, and why

| Item | Reason | Status |
|---|---|---|
| Swiss Ephemeris | AGPL-3.0: linking it would make the combined work AGPL | optional **separate** `eshana-swisseph` adapter (AGPL-3.0) planned, `PROV-03` (ADR-011) |
| Human Design descriptive text, bodygraph art, third-party tables | copyright in the expression, whatever our licence | not included; the calculation itself is implemented in full and published |
| Third-party interpretation text | rights per source | not included; every rule pack cites its source |

The Nakshatra reference attributes shown in HTML reports come from the author's own Panchang Chakra table and ship as
`eshana/data/nakshatra_reference.json` under the project licence.
