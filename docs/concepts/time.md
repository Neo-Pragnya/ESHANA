---
title: "Time"
description: "From a civil moment to the time scales an ephemeris needs, and every place it can go wrong."
---

# Time

Most wrong charts are wrong in the time step, not the astronomy.

## Civil to UTC

- Local input needs an IANA zone (`Asia/Kolkata`) or an explicit `utc_offset_minutes`. **ESHANA never assumes a zone.**
- The bundled tz database applies the zone's *historical* rules: LMT before standardisation, wartime changes, DST. The
  database version is in `provenance.tzdb_version`.
- **A repeated local time** (clocks go back) is `ESH-TIME-AMBIGUOUS` unless `fold` says `earliest` or `latest`.
- **A skipped local time** (clocks go forward) is `ESH-TIME-NONEXISTENT`. There is no policy to pick: the moment did not
  happen, so the caller must say what was meant.

## UTC, UT1, TT

| Step | Model |
|---|---|
| UTC treated as UT1 | within 0.9 s; effect ≤ 0.001° for the Moon, ≤ 0.004° for the Ascendant; recorded as an assumption |
| 1972-01-01 to 2026-12-31 | TT = UTC + (TAI−UTC) + 32.184 s from the leap-second table (37 s since 2017) |
| 2027 onward | the 2027 value continued along the Espenak–Meeus ΔT growth: a **forecast**, flagged in `time_model` |
| before 1972 | TT = UT + ΔT from the Espenak–Meeus polynomials (1800–1972) |

Julian day is computed from the proleptic Gregorian calendar. The Julian/Gregorian cut-over is not modelled: input is
proleptic Gregorian, and the providers' ranges (1800–2100 and the kernel span) stay well clear of 1582.

## Sidereal time

GMST uses the IAU 1982 polynomial on UT; the equation of the equinoxes (Δψ cos ε) makes it apparent. Checked against
Meeus's worked examples and the published value at J2000.

## Uncertain times

`when.time` omitted means the birth time is unknown; `uncertainty_minutes` makes it approximate. Both become a window,
and every position carries `longitude_interval_deg`. See [Uncertain birth time](../guides/uncertain-birth-time.md).

## Known limits

- Leap seconds are not predicted. A UTC instant after 2027 is read with the forecast ΔT, and the uncertainty grows.
- Historical clock data before the 1970s has regional gaps in any tz database; for old births, verify the zone rule.
- Julian-calendar dates before 1582 must be converted by the caller.
