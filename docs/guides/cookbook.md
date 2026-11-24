---
title: "Cookbook"
description: "Notebooks and short recipes: a first chart, comparing methods, sky events, batch jobs and agent recipes."
---

# Cookbook

Runnable notebooks live in [`examples/notebooks`](../../examples/notebooks/). The Python tests run every code cell
of every notebook, so they stay correct as the engine changes.

| Notebook | What it shows |
|---|---|
| `01-first-chart.ipynb` | a Vedic chart, an attribute with its rule, provenance, a refusal with its code, the inline report |
| `02-compare-methods.ipynb` | two ayanāṃśas and two house systems side by side, with the values that changed |
| `03-sky-events.ipynb` | eclipses in a year, a solar return, rise and set, gochara from the natal Moon and lagna |
| `04-agent-tools.ipynb` | the MCP tools called in-process; `explain_derivation`; how a refusal reaches an agent |

## Recipes

**Which nakshatra, and by which rule?**

```python
from eshana import ChartEngine, Context
chart = ChartEngine().vedic_chart(Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.209), ayanamsha="lahiri")
chart.attribute("vedic.moon.nakshatra")      # value; the full attribute carries rule_version and derived_from
```

**Every eclipse visible in a decade, as rows**

```python
rows = [(e["max_utc"], e["kind"], e["type"]) for e in ChartEngine().eclipses("2026-01-01T00:00:00Z", "2036-01-01T00:00:00Z")["eclipses"]]
```

**Sunrise for a year at one place** (the pañcāṅga day starts here)

```python
days = ChartEngine().rise_set(28.6139, 77.209, "2027-01-01", "Asia/Kolkata", days=365, bodies=["sun"])["days"]
```

**A thousand charts** (D1, D9 and the daśā) take about 0.75 s on an Apple M3 Ultra with the built-in ephemeris
(measure your own machine with `make perf`); the
[ways to use](ways-to-use.md) page has a CSV-to-HTML batch job.

## Agent recipes

1. Register the server: `claude mcp add eshana -- eshana-mcp` (or any MCP client, see the [MCP guide](mcp.md)).
2. Give the agent a birth with an IANA zone. Without one the tools refuse (`ESH-INPUT`) instead of assuming.
3. Ask it to cite the `rule_version` of every value it uses, and to call `explain_derivation` when asked "why".
4. Keep reading and calculation apart: the envelope's `interpretations` is always empty, so anything interpretive
   is the agent's own text, and the provenance says exactly what was calculated.

In a browser or Node, the same requests go through the [JavaScript package](javascript.md).
