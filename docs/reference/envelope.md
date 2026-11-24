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
| `positions` | charts | per body: `id`, `longitude_deg` (in the chosen zodiac), `tropical_longitude_deg`, `latitude_deg`, `right_ascension_deg` and `declination_deg` (true obliquity of date; since schema 0.3.0), `distance_au`, `speed_deg_per_day`, `retrograde`, `sign`, `epistemic: calculated`, optional `longitude_interval_deg` |
| `houses` | charts with a place and an exact time | `system`, `ascendant`, `midheaven`, `cusps_deg`, `local_apparent_sidereal_time_deg`; `null` otherwise |
| `derived_attributes` | charts, numerology, HD | `id`, `tradition_id`, `rule_version`, `epistemic`, `value`, `derived_from` |
| `interpretations` | charts | **always empty** from the engine |
| `warnings` | charts | `{code, message}`, e.g. `ESH-TIME-UNCERTAIN` |
| `provenance` | charts | see [Provenance](../concepts/provenance.md) |

Human Design replaces `positions` by `{personality, design}`, each with `instant_utc`, `jd_ut` and 13 `activations`
(`body`, `longitude_deg`, `gate`, `line`, `color`, `center`).

## Shared shapes (since the capability expansion)

Newer operations reuse four shapes, defined under `$defs` in the schema:

- **Unavailable** — `{"status": "unavailable", "reason": "…", "missing": ["…"]}`. A factor that cannot be computed (no
  reliable Ascendant, no sunrise at a polar latitude, a node with no Aṣṭakavarga table) is reported this way, never as
  zero.
- **Interval** — `{"start_utc", "end_utc", "bounds": "[start, end)"}`. `end_utc` is `null` when the interval is still
  open where the searched range ends.
- **Method** — `{"family", "id"}`. Every family and id is listed by the `methods` operation (`eshana methods`); an id
  the engine does not implement is refused with `ESH-METHOD`, never replaced by a default.
- **Configuration hash** — `calculation.config_hash`, `fnv1a64:` and 16 hex digits over the normalised settings (keys
  sorted). Two results with the same hash were computed under the same settings.

## Compatibility promise

Within `0.x`, fields may be added without bumping `schema_version`'s minor only when they are optional and ignorable. A
removed or renamed field bumps it. Consumers should ignore unknown keys and pin `schema_version`.

Schema 0.3.2 adds the analysis attributes (`vedic.maitri.*`, `vedic.dasha.*` for seven more systems, `vedic.aux.*`, `vedic.chakra.*`, `vedic.varshaphala`, `vedic.panch_pakshi`), `vedic.jaimini.arudha_padas_d9`, `lordships_by_varga` in the special points, `significator_table` in `vedic.kp` and `kua` in `numerology.lo_shu`.

Schema 0.3.3 (round 8) adds fields only: the numerology request takes `gender` (`male` or `female`), echoed in `input`; `numerology.lo_shu.kua` gains `gender`, `number` and `group` when it is given; `numerology.pythagorean.extended` gains `growth`, `intensity`, `transits_by_age` and `transits_at_reference` (each year with `essence`).

Schema 0.4.0 adds fields and values only: envelope kinds `methods`, `stations`, `crossings`, `ashtakavarga_transits`, `muhurta_windows`, `personal_calendar`, `pair_calendar`, `progressions`, `synthetic_chart`, `research_window_score`, `kp_table` and `kp_horary`; `calculation.config_hash` on charts and event searches; the chart request's optional `kp_cusp_occupancy` and `varga_methods`; six more ayanāṃśa ids; error codes `ESH-METHOD`, `ESH-RANGE-INVALID` and `ESH-LIMIT`. Existing fields and defaults are unchanged.

In eshana 0.4.4, schema 0.4.0 gains optional fields only, so `schema_version` stays: the `vedic.drishti.vargas` attribute (graha, rāśi and additional drishti in each requested varga) and, in `vedic.aspects`, `divisions` and a `dN` matrix, `houses`, `house_and_lord` and `house_madhya` entry for every requested varga besides D9.

Schema 0.3.1 adds `tone` (1–6) and `base` (1–5) to every Human Design activation, next to `gate`, `line` and `color`. It also adds the `vedic.upagrahas` and `vedic.drishti.additional` attributes and `houses`, `house_and_lord` and `house_madhya` in `vedic.aspects`.
