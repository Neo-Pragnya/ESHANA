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
| 2027 onward | a **forecast**: the IERS Bulletin A trend (Vol. XXXIX No. 047, 19 November 2026: UT1−UTC drifting by −0.179 s a year) blended into the Espenak–Meeus curve by 2050; flagged in `time_model` |
| before 1972 | TT = UT + ΔT from the Espenak–Meeus polynomials (NASA *Five Millennium Canon*, −500 to 2150) and the Morrison–Stephenson long-term parabola −20 + 32u² outside them |

ΔT before the telescope era is a model with a large uncertainty (about ±20 s in 1600, ±10 minutes in 500 BCE); the
Moon moves about 0.55″ per second of ΔT, so an ancient Moon carries that uncertainty.

## Calendars and BCE dates

`when.calendar` says how `date` is written:

| Value | Meaning |
|---|---|
| `gregorian` (default) | proleptic Gregorian, as ISO 8601 |
| `julian` | proleptic Julian, for every date |
| `auto` | Julian before 1582-10-15, Gregorian from then: the historical cut-over (Julian 1582-10-04 was followed by Gregorian 1582-10-15; the days between never existed and are refused) |

Years are astronomical: `0000` is 1 BCE and `-0043-03-15` is 15 March 44 BCE. A date before 1582-10-15 given without
a calendar is read as proleptic Gregorian and the assumption says so; a converted date is recorded
(`julian 1500-02-29 = gregorian 1500-03-10`) and `input.calendar` keeps the date as written. Every instant in the
envelope (`instant_utc`) is proleptic Gregorian.

The default `analytic` provider covers 1800–2100. Earlier (or later) dates need a JPL kernel: `de440` covers
1550–2650 and the two `de441` parts reach 13,201 BCE to 17,191 CE; see [Using a JPL kernel](../guides/spk-kernel.md).
Precession and nutation use the IAU 2006 / 4-term models, accurate to arc-seconds within a few centuries of J2000;
the error grows for ancient dates.

## Sidereal time

GMST uses the IAU 1982 polynomial on UT; the equation of the equinoxes (Δψ cos ε) makes it apparent. Checked against
Meeus's worked examples and the published value at J2000.

## Uncertain times

`when.time` omitted means the birth time is unknown; `uncertainty_minutes` makes it approximate. Both become a window,
and every position carries `longitude_interval_deg`. See [Uncertain birth time](../guides/uncertain-birth-time.md).

## Known limits

- Leap seconds are not predicted. A UTC instant after 2027 is read with the forecast ΔT, and the uncertainty grows.
- Historical clock data before the 1970s has regional gaps in any tz database; for old births, verify the zone rule.
- Before standard time, places kept local mean time: use `utc_offset_minutes` (or the zone's LMT, which the tz
  database gives for a city) and the right `calendar`.
