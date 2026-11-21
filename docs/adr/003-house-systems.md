---
title: "ADR-003: House system handling"
description: "Explicit enum, honest limits, no silent extrapolation."
---

# ADR-003: House system handling

**Status.** Accepted.

## Decision

`HouseSystem` is an explicit enum (`whole_sign`, `equal`, `placidus` in 0.1) echoed in every result. A system that is
undefined for the inputs returns `ESH-LATITUDE`; it is never approximated or swapped. Ascendant and houses need an exact time
(window ≤ 30 minutes) and are omitted with a warning otherwise. Further systems (Koch, Regiomontanus, Campanus, Porphyry)
are added one at a time with golden tests against the oracle.

## Consequences

A service that wants a fallback must choose it explicitly and say so. Placidus is validated at six places including 51.5°N
and 64.1°N; the polar refusal is an acceptance scenario with a legitimate twin (whole sign at the same latitude works).
