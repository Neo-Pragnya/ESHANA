---
title: "Western chart"
description: "Tropical positions, houses with honest limits, and aspects with orb and applying state."
---

# Western chart

```bash
eshana chart western --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 \
                     --houses placidus --aspects --orb-scale 0.75
```

- **Zodiac** is tropical unless you ask for a sidereal Western variant (`--zodiac sidereal --ayanamsha fagan_bradley`).
- **Houses**: `whole_sign`, `equal`, `placidus`. Placidus is refused (`ESH-LATITUDE`) at and beyond the polar circle and
  wherever the semi-arc iteration has no solution. The same request with `whole_sign` works, so a service can fall back
  explicitly and say so.
- **Aspects** are Ptolemaic with conventional orbs (8/4/6/6/8°), scalable. Each carries `separation_deg`, `orb_deg` (distance
  from exact), and `applying` (true if tightening). The id is the alphabetically ordered pair, e.g.
  `western.aspect.moon-sun`.
- Angles (`ascendant`, `midheaven`) are in `houses`; `cusps_deg[0]` is house 1.

## Validation

Ascendant, Midheaven and every cusp of the three systems are compared with the Swiss Ephemeris at 6 places and 8 instants
(tolerance 0.01° for the angles, 0.02° for cusps). Sidereal Sun and Moon match to 0.02° for four ayanāṃśas.

## Not here yet

Synastry, composite, progressions, returns and dignities ([roadmap](../roadmap.md)). Dignity and interpretive
traditions will be separate rule packs, never mixed into measurements.
