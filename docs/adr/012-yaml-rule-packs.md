---
title: "ADR-012: Data-driven rule packs are YAML"
description: "Tradition tables and condition rules live in versioned YAML packs, read by the engine through serde_norway; code applies them."
---

# ADR-012: Data-driven rule packs are YAML

**Status.** Accepted (owner decision, 2026-11-21). Amends [ADR-004](004-rule-packaging.md) (`RULE-01`).

## Context

ADR-004 left the format of data-driven rule packs open. Tradition data (dignities, aṣṭakavarga tables, pañcāṅga
names and day divisions, aṣṭakūṭa tables, yoga conditions) changes by school, needs citations and review, and is read
by people who do not write Rust.

## Decision

1. Rule packs are **YAML** files in `crates/eshana-engine/rulesets/`, one per concern: `vedic-core.yaml`,
   `ashtakavarga.yaml`, `panchanga.yaml`, `matching.yaml`, `yogas.yaml`. Each has an `id`, a `version`, and the
   `conventions` it follows in prose.
2. The engine compiles them in with `include_str!`, parses them once with **serde_norway** (the maintained fork of
   `serde_yaml`; `serde_yaml` is deprecated and `serde_yml` has an unsoundness advisory), and validates fixed facts
   at load and in tests: table sizes, aṣṭakavarga totals (48, 49, 39, 54, 56, 52, 39; 337), a symmetric yoni matrix,
   kūṭa maxima summing to 36.
3. Conditions (yogas, doshas) use a small declarative language: `in_house`, `dignity`, `same_sign`, `occupied`,
   `empty`, `lords_conjoined`, `lord_in_house`, `lord_dignity`, `between_nodes`, `all_of`, `any_of`, `not` and
   `for_each_graha`. Each entry carries the rule in words. A yoga's presence is a derived fact; its meaning is not
   in the pack.
4. Algorithms stay in code: the wheel arithmetic, nakshatra and varga mappings, daśā nesting, KP subdivision, rise and
   set searches. A pack supplies tables and conditions; it does not run code.
5. `eshana rulesets` (and `ChartEngine.rulesets()`) print the packs verbatim, so a result can be audited against the
   exact tables that produced it.

## Consequences

- Changing a table is a reviewed data change with a version bump of that pack, recorded in every result's provenance.
- A different school is a different pack (for example another vashya table), never a silent edit.
- Loading user-supplied packs at run time is the next step (`RULE-02`), with the same validator.
