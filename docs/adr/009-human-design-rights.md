---
title: "ADR-009: Human Design nomenclature and mapping rights"
description: "Structure and mathematics implemented from first principles. Closed by ADR-011: open source, no commercial launch."
---

# ADR-009: Human Design nomenclature and mapping rights

**Status.** **Open.**

> [!NOTE]
> **Amended by [ADR-011](011-open-source-distribution.md) (2026-11-20).** ESHANA is an open-source tool stack published as libraries, with no commercial launch. The open question here is closed there.

## Context

Human Design uses a gate wheel, channels, centres and a branded vocabulary, and has associated copyrighted text and artwork.
The calculation (a solar-arc root-solve and a table lookup) is mathematics; the *expression* and some nomenclature may be
protected. Gate numbers are I Ching hexagram numbers placed on the ecliptic.

## Decision (interim)

Implement only the **structure and the mathematics**, independently: the 88° solve, the 64-gate wheel from 2°00′ Aquarius,
the channels and centres as pairs and lists of integers, and the usual type/authority/profile/definition derivation. Include
**no** descriptive text, artwork or third-party tables. Label everything `tradition_rule` and record the review as an assumption
in every Human Design result.

## Required before commercial use

Counsel's opinion on nomenclature and arrangement in the target jurisdictions; a decision on whether field names such as
`type` and `authority` are acceptable or should be neutral; confirmation that no stored table is copied from a protected
source. Tracked as `HD-05`.
