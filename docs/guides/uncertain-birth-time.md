---
title: "Uncertain birth time"
description: "What ESHANA does when the birth time is unknown or approximate: intervals, stability flags, and omission instead of invention."
---

# Uncertain birth time

Many real birth records have no time, or "around 8 pm". The wrong answer is to pick noon. ESHANA models the uncertainty.

| Input | Quality | Window |
|---|---|---|
| `time` omitted | `date_only` | the whole local day (±12 h around local noon) |
| `time` + `uncertainty_minutes` | `approximate` | ± that many minutes |
| `time` | `exact` | none |

```bash
eshana chart vedic --date 1992-05-28 --tz Asia/Kolkata                       # date only
eshana chart western --date 1992-05-28 --time 20:05 --tz Asia/Kolkata \
        --lat 17.385 --lon 78.487 --uncertainty-minutes 5                      # approximate
```

## What changes

- Every position carries `longitude_interval_deg`: `[at window start, at window end]`.
- Rāśi and nakshatra attributes carry `stable_within_uncertainty`. When `false`, a `candidates` list names the possible
  values rather than choosing one. For the example birth, the Moon could be in Meena or Meṣa on that date.
- **Ascendant and houses are omitted** when the window is wider than 30 minutes (the Ascendant moves more than 7° in
  that time) and a `warnings` entry with code `ESH-TIME-UNCERTAIN` says why. At ±5 minutes they are kept.
- **Daśā is withheld** for any non-exact time: its dates depend on the Moon's exact longitude.
- **Human Design is refused** (`ESH-TIME-UNCERTAIN`): gate lines move within minutes.

## Assumption

The interval is taken between the window's end points. For the Moon and the Sun this is exact unless a station occurs
inside the window, which cannot happen within a day for those two. For a retrograde station of a planet inside a
date-only window, the true extent can exceed the end points; the assumption is stated in `provenance.assumptions`.
