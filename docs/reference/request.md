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
| `numerology` | `name`, `date_of_birth`, `school`, `keep_master_numbers`, `y_policy`, `life_path_method`, `reference_year` |
| `compare` | `base` (a chart request), `a` and `b` (config overrides) |
| `graph` | `chart` (a chart request with `system`), `format` |
| `info` | none |

## Chart request fields

| Field | Type | Notes |
|---|---|---|
| `when.utc` | string | alternative to `date`/`time`; not both |
| `when.date`, `when.time`, `when.timezone`, `when.utc_offset_minutes`, `when.fold`, `when.uncertainty_minutes` | | see [Time](../concepts/time.md) |
| `place.latitude`, `place.longitude`, `place.altitude_m` | number | degrees |
| `config.provider`, `.zodiac`, `.ayanamsha`, `.house_system`, `.node_model` | string | see [CLI](cli.md) |
| `system` | string | `vedic` or `western` |
| `bodies` | array of ids | `sun`, `moon`, `mercury` … `pluto`, `mean_node`, `true_node` |
| `divisions`, `dasha`, `year_convention` | | Vedic |
| `aspects`, `orb_scale` | | Western |
| `spk_path` | string | kernel for the `spk` provider |
| `assume_defaults` | bool | let the engine default a missing ayanāṃśa to Lahiri and **record** that; off by default |

From Python: `eshana.execute(request_dict)` returns the envelope dict (or raises `EshanaError`). From the shell:
`eshana request req.json`.
