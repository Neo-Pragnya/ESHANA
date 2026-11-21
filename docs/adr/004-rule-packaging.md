---
title: "ADR-004: Rule packaging"
description: "Versioned packs named for traditions; code for algorithms in 0.1, data-driven packs next."
---

# ADR-004: Rule packaging

**Status.** Accepted for 0.1. A data-driven format is planned (`RULE-01`).

## Decision

Rule packs are versioned (`id@version`), named for the tradition and convention (never for the software), and carry a source
statement. In 0.1 they are Rust modules: the tables are small, the algorithms (design-instant solve, Vimśottarī, Placidus) are
code, and the compiler checks the tables' shape. Every derived attribute records its pack. A pack's version changes whenever
its output for some input could change.

## Later

A data format (YAML or JSON) for table-like rules, with a validator, a mandatory `source` and `edition` and worked examples,
so that contributors can add a tradition without Rust. Algorithms stay in code.

## Consequences

Tables are cross-checked by invariants (HD gates partition into centres; 36 channels; every gate in a channel), which caught a
missing channel during development.
