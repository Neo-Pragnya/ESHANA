---
title: "Python"
description: "The Python facade: typed errors, Context, ChartEngine, and the raw execute() escape hatch."
---

# Python

The package is a thin facade over one native function. No rule is implemented in Python, so results are identical to
the CLI's ([parity is a release-gate scenario](../reference/conformance.md)).

```python
from eshana import ChartEngine, Context, EshanaError

ctx = Context.from_local_birth(
    date="1992-05-28", time="20:05:00", timezone="Asia/Kolkata",
    latitude=17.385, longitude=78.487,
)
engine = ChartEngine()                       # provider="analytic"; ChartEngine("spk", spk_path="de440s.bsp") for the kernel
vedic = engine.vedic_chart(ctx, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
western = engine.western_chart(ctx, zodiac="tropical", houses="whole_sign")

assert vedic.provenance and western.provenance
vedic.position("moon")["sign"]["name"]                 # 'Meena'
vedic.attribute("vedic.moon.nakshatra")["name"]        # 'Revati'
vedic.attribute_full("vedic.moon.d9")["rule_version"]  # 'vedic.parashari@0.3'
```

`vedic_chart` **requires** `ayanamsha`. Calling it without one is a Python `TypeError`; passing `zodiac="sidereal"`
to `positions` without one is an `EshanaError` with code `ESH-CONFIG`.

## Errors

```python
try:
    engine.positions(Context.from_utc("1500-01-01T00:00:00Z"))
except EshanaError as e:
    print(e.code, e.message)      # ESH-RANGE outside ephemeris range: ...
```

## API summary

| Call | Returns |
|---|---|
| `Context.from_local_birth(date, time=None, timezone=None, latitude=None, longitude=None, utc_offset_minutes=None, fold=None, uncertainty_minutes=None)` | a context; omit `time` for an unknown birth time |
| `Context.from_utc("2000-01-01T12:00:00Z", latitude=…, longitude=…)` | a context |
| `engine.positions(ctx, zodiac, ayanamsha, node_model, bodies)` | `Chart` |
| `engine.vedic_chart(ctx, ayanamsha, houses, node_model, divisions, dasha, year_convention)` | `Chart` |
| `engine.western_chart(ctx, zodiac, houses, node_model, aspects, orb_scale)` | `Chart` |
| `engine.human_design(ctx, node_model=None)` | dict envelope |
| `engine.numerology(name, date_of_birth, school, keep_master_numbers, y_policy, life_path_method, reference_year)` | dict envelope |
| `engine.compare(ctx, system, a, b, **base_cfg)` | dict envelope |
| `engine.graph(ctx, system, fmt="json"|"mermaid"|"graphml", **cfg)` | dict or text |
| `ChartEngine.info()` | versions, providers, rule packs, error codes |
| `eshana.execute({...})` | any request, raw |

`Chart.raw` is the whole envelope; `Chart.houses` is `None` when the birth time was too uncertain, and `Chart.warnings`
says why.

## Notebooks and AI agents

The envelope is plain JSON, so it drops straight into a DataFrame, a notebook or a tool result. If you hand it to a language
model, hand over the **envelope**, not a request to compute: the model should *explain* attributed outputs, never recompute
a longitude. Keep `epistemic` and `rule_version` in the prompt so the model can say which statements are measurements and
which are a tradition's rules.
