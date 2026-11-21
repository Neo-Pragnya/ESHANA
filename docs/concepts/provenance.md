---
title: "Provenance and versioning"
description: "What every result records so it can be replayed, audited and compared."
---

# Provenance and versioning

A result you cannot reproduce is an anecdote. Every ESHANA result carries a `provenance` block:

```json
"provenance": {
  "engine_version": "0.1.0",
  "provider": "analytic",
  "provider_version": "0.1.0",
  "provider_data": "JPL approximate Keplerian elements (Standish, Table 1, 1800-2050) + Meeus ch. 47 lunar series; no data files",
  "provider_accuracy": "Low precision: see benchmarks/golden/accuracy.md ...",
  "tzdb_version": "2026f",
  "time_model": "leap-seconds-1972-2017+37s-to-2027",
  "rulesets": ["ayanamsha-lahiri@0.1", "vedic.parashari@0.1"],
  "assumptions": [
    "ayanamsha not given; defaulted to lahiri because assume_defaults was set",
    "UTC is used as UT1 (differs by under 0.9 s)",
    "civil time resolved with IANA tzdb 2026f"
  ]
}
```

## The versioning contract

| Field | Meaning | Changes when |
|---|---|---|
| `schema_version` | the output contract (the envelope's shape) | fields are added, renamed or removed |
| `engine_version` | the compiled engine | any release |
| `provider` + `provider_version` + `provider_data` | where positions came from; for `spk`, the kernel's file name, internal name, size and an FNV-1a hash | a series, a constant or the data changes |
| `tzdb_version` | the IANA tz database compiled in | the dependency updates |
| `time_model` | which UTC→TT model applied to this instant | the instant crosses 1972 or 2027 |
| `rulesets` | every rule pack and ayanāṃśa anchor applied, as `id@version` | a rule's behaviour changes |
| `assumptions` | everything the engine chose on the caller's behalf | per request |

`schema_version` is independent of `engine_version`: a new release that does not change the shape keeps the same
`schema_version`, so consumers can pin the contract rather than the build.

## Rule versions are behaviour contracts

A rule pack's version changes when its output for the same input could change: a corrected table, a different
convention, a new tie-break. Adding a rule that was not available before does not change the version of existing ones.
Rule packs are named after their tradition and convention (`vedic.parashari`), never after the software.

## Reproducing a result

Replay needs the request, the `engine_version` and, for `spk`, the same kernel. The acceptance suite asserts that the
same request yields the same bytes. For a long-lived archive store the request next to the result; the `input` block
echoes the resolved instant, Julian days and ΔT so the civil-time step can be audited without the tz database.

## Interpretations carry their own version

`interpretations` is always empty in engine output. A downstream interpretation service should add its own
`interpretation_model_version` and must not imply that it is covered by `schema_version` or the rule versions.
