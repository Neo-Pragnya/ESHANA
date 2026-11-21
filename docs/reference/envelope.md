---
title: "Result envelope"
description: "The shape of every ESHANA result, and the machine-readable schema that pins it."
---

# Result envelope

Every successful result is one JSON object; `schema_version` pins the contract (currently `0.1.0`). The formal definition
is [`schemas/envelope.schema.json`](https://github.com/Neo-Pragnya/ESHANA/blob/main/schemas/envelope.schema.json), also
printed by `eshana schema`. The Python test suite and the acceptance gate validate real outputs against it.

| Key | Present in | Meaning |
|---|---|---|
| `schema_version` | all | output contract version |
| `kind` | all | `positions`, `computed_chart`, `human_design`, `numerology_analysis`, `method_comparison`, `symbolic_graph`, `symbolic_graph_export`, `info`, `error` |
| `input` | charts | the resolved instant (`instant_utc`, `jd_ut`, `jd_tt`, `delta_t_s`), zone and offset, `birth_time_quality`, `time_window_minutes`, `place` |
| `calculation` | charts | `frame`, `provider`, `zodiac`, `ayanamsha`, `ayanamsha_deg`, `house_system`, `node_model` |
| `positions` | charts | per body: `id`, `longitude_deg` (in the chosen zodiac), `tropical_longitude_deg`, `latitude_deg`, `distance_au`, `speed_deg_per_day`, `retrograde`, `sign`, `epistemic: calculated`, optional `longitude_interval_deg` |
| `houses` | charts with a place and an exact time | `system`, `ascendant`, `midheaven`, `cusps_deg`, `local_apparent_sidereal_time_deg`; `null` otherwise |
| `derived_attributes` | charts, numerology, HD | `id`, `tradition_id`, `rule_version`, `epistemic`, `value`, `derived_from` |
| `interpretations` | charts | **always empty** from the engine |
| `warnings` | charts | `{code, message}`, e.g. `ESH-TIME-UNCERTAIN` |
| `provenance` | charts | see [Provenance](../concepts/provenance.md) |

Human Design replaces `positions` by `{personality, design}`, each with `instant_utc`, `jd_ut` and 13 `activations`
(`body`, `longitude_deg`, `gate`, `line`, `color`, `center`).

## Compatibility promise

Within `0.x`, fields may be added without bumping `schema_version`'s minor only when they are optional and ignorable. A
removed or renamed field bumps it. Consumers should ignore unknown keys and pin `schema_version`.
