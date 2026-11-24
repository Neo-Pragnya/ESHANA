---
title: "Request format"
description: "The JSON request every surface sends to the engine. Unknown fields are refused."
---

# Request format

A request is one JSON object with an `op`. Unknown fields are an `ESH-INPUT` error (a typo must not silently change
meaning).

```json
{
  "op": "chart",
  "system": "vedic",
  "when": { "date": "1992-05-28", "time": "20:05:00", "timezone": "Asia/Kolkata" },
  "place": { "latitude": 17.385, "longitude": 78.487 },
  "config": { "zodiac": "sidereal", "ayanamsha": "lahiri", "house_system": "whole_sign", "node_model": "mean" },
  "divisions": [1, 9],
  "dasha": true
}
```

## `op` values

| `op` | Body |
|---|---|
| `positions` | a chart request without `system` |
| `chart` | a chart request with `system`: `vedic` or `western` |
| `human_design` | a chart request (exact time required) |
| `numerology` | `name`, `date_of_birth`, `school` (`pythagorean`, `chaldean`; `hebrew_gematria` or `abjad` with the name in Hebrew or Arabic letters), `keep_master_numbers`, `y_policy`, `life_path_method`, `reference_year` |
| `compare` | `base` (a chart request), `a` and `b` (config overrides) |
| `graph` | `chart` (a chart request with `system`), `format` |
| `panchanga` | `chart` (a chart request with `place`, sidereal), `natal_moon_deg` |
| `matching` | `a`, `b` (chart requests) |
| `ephemeris` | `start_utc`, `step_days`, `count` (≤ 20 000), `bodies`, `config`, `spk_path` |
| `ingresses` | `body`, `start_utc`, `end_utc`, `config`, `spk_path` |
| `stations` | `body`, `start_utc` (included), `end_utc` (excluded), `tolerance_seconds` (default 1), `config`, `spk_path` |
| `crossings` | `body`, `point_deg`, `aspects` (default `[0]`), `start_utc`, `end_utc`, `tolerance_seconds`, `touch_orb_deg` (default 0.1), `max_results` (default 10 000), `config`, `spk_path` |
| `rulesets` | none |
| `ashtakavarga_transits` | `natal` (a chart request: exact time, place, sidereal settings), `start_utc`, `end_utc` (at most 50 years), `bodies` (default Sun–Saturn; a node is answered as unavailable) |
| `muhurta_windows` | `place` (the event place), `start_utc`, `end_utc` (at most 62 days), `timezone` or `utc_offset_minutes`, `profile`, `disable`, `constraints` (ids or `{factor, include \| exclude}`), `show`, `sunrise`, `config` (ayanāṃśa) |
| `personal_calendar` | `natal` (a chart request: the birth), `muhurta` (as above: the event), `format` (`json`, `csv`, `ics`) |
| `pair_calendar` | `a`, `b` (chart requests), `muhurta` (event place, range, constraints; default profile `pair_general`), `format` |
| `progressions` | `natal` (a chart request), `at_utc`, `angle_method` (`solar_arc_longitude`, `naibod_ra`, `quotidian`; without it angles are unavailable), `year_days` (default 365.24219); the result also carries `solar_arc_directions` (every natal point + the solar arc, and directed-to-natal aspects within 1°) |
| `synthetic_chart` | `positions` (`[{body, longitude_deg, speed_deg_per_day?}]`, sidereal), `ascendant_deg`; no ephemeris; labelled synthetic |
| `research_window_score` | `windows` (as a muhūrta or calendar result gives them); experimental model `research.window_score@0.1`, specified in `docs/design/research-models.md` |
| `kp_table` | none; the 1–249 KP horary numbers |
| `kp_horary` | `number` (1–249), `chart` (moment and place of judgement, sidereal settings, optional `kp_cusp_occupancy`) |
| `rise_set` | `place` (with optional `altitude_m`), `date` (first local day), `timezone` or `utc_offset_minutes`, `days` (1–366, default 1), `bodies` (default Sun and Moon), `sunrise` (convention), `twilight` (default true), `config`, `spk_path` |
| `returns` | `natal` (a chart request with an exact time; its zodiac settings apply), `body` (default `sun`), `start_utc`, `end_utc` (≤ 120 years; Moon ≤ 10), `place` (relocated return), `charts` (default true) |
| `gochara` | `natal` (sidereal settings), `start_utc`, `end_utc` (≤ 30 years), `bodies` (default the nine grahas) |
| `synastry` | `a`, `b` (chart requests; `system` from `a`, default western), `orb_scale`, `composite` (default true), `davison` (default true) |
| `eclipses` | `start_utc`, `end_utc` (≤ 200 years), `kinds` (`solar`, `lunar`; default both), `config`, `spk_path` |
| `methods` | none; returns every method family and id (`kind: methods`) |
| `info` | none |

## Chart request fields

| Field | Type | Notes |
|---|---|---|
| `when.utc` | string | alternative to `date`/`time`; not both |
| `when.date`, `when.time`, `when.timezone`, `when.utc_offset_minutes`, `when.fold`, `when.uncertainty_minutes` | | see [Time](../concepts/time.md) |
| `when.calendar` | string | `gregorian` (default, proleptic), `julian` or `auto` (Julian before 1582-10-15); signed years for BCE |
| `place.latitude`, `place.longitude`, `place.altitude_m` | number | degrees |
| `config.provider`, `.zodiac`, `.ayanamsha`, `.house_system`, `.node_model` | string | see [CLI](cli.md) |
| `config.topocentric` | bool | positions as seen from `place` (parallax applied; up to ~1° for the Moon); `calculation.positions_reference` says which |
| `system` | string | `vedic` or `western` |
| `bodies` | array of ids | `sun`, `moon`, `mercury` … `pluto`, `mean_node`, `true_node`; optional `ceres`, `pallas`, `juno`, `vesta` (kernel `sb441-n16`) and `chiron` (a JPL Horizons kernel) with `--spk de440,…` |
| `divisions`, `dasha`, `year_convention` | | Vedic |
| `features` | array | Vedic feature names (`all` for every one, including `lal_kitab`); a western chart takes only `dignities` (essential dignities, pack `western.dignities`) |
| `varga_methods` | array | further vargas by method id: `d2_parivritti`, `d3_parivritti`, `d30_parivritti`, `d60_parivritti`, `d81_navanavamsha`, `d108_ashtottaramsha_cyclic`, `d144_dvadasha_dvadashamsha`, `d150_parivritti` |
| `user_rule_packs` | array of YAML text | user yoga packs in the `yogas.yaml` format (Vedic), checked by the same grammar and reported as `user.yogas.<id>` |
| `aspects`, `orb_scale` | | Western |
| `spk_path` | string | kernel for the `spk` provider: a path, or a catalogue name such as `de440` (see `eshana kernel list`) |
| `assume_defaults` | bool | let the engine default a missing ayanāṃśa to Lahiri and **record** that; off by default |

From Python: `eshana.execute(request_dict)` returns the envelope dict (or raises `EshanaError`). From the shell:
`eshana request req.json`.
