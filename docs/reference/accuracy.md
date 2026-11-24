---
title: "Accuracy"
description: "Measured error of each provider and each derived quantity, with the method and the caveats."
---

# Accuracy

> [!TIP]
> Whole birth charts (positions, houses, nakshatra, vargas, daśā) are checked by the
> [birth-chart accuracy suite](../guides/accuracy-suite.md) against the Swiss Ephemeris, the Lahiri definition and
> a recorded ProKerala chart. Results: [birth-chart-accuracy.md](../../benchmarks/accuracy/birth-chart-accuracy.md).

All numbers below are **measured**, not promised; the full tables are in `benchmarks/golden/accuracy.md` and the tests that
enforce them are named.

## Positions vs JPL DE440s (independent pipeline)

Fixture: 9 instants from 1950 to 2060, apparent geocentric ecliptic of date, computed with Skyfield 1.55.

| Body | `analytic` max longitude error | `spk` max longitude error |
|---|---:|---:|
| Sun | 15.8″ | 0.3″ |
| Moon | 9.6″ | 0.3″ |
| Mercury | 27.8″ | 0.3″ |
| Venus | 22.4″ | 0.3″ |
| Mars | 31.5″ | 0.3″ |
| Jupiter | **342″** | 0.3″ |
| Saturn | **1026″ (17′)** | 0.3″ |
| Uranus | 103″ | 0.3″ |
| Neptune | 44″ | 0.3″ |
| Pluto | 49″ | 0.3″ |

Enforced by `crates/eshana-ephemeris/tests/golden.rs`: analytic within 60″ (inner), 600″ (Jupiter), 1800″ (Saturn), 200″ (outer);
`spk` within 2″.

## Derived quantities vs the Swiss Ephemeris (offline oracle)

| Quantity | Tolerance enforced |
|---|---:|
| Ayanāṃśa, four models, 1950–2040 | 0.0006° |
| Ascendant, Midheaven | 0.01° |
| House cusps (Placidus, Equal, Whole Sign), 6 places × 8 instants | 0.02° |
| Sidereal Sun and Moon, four ayanāṃśas | 0.02° |
| Mean node | 0.01° |
| True node (osculating) | 0.06° |

## Event times vs the Swiss Ephemeris (offline oracle)

`benchmarks/golden/events_oracle.json` (from `benchmarks/make_event_oracle.py`) holds the stations of Mercury to Saturn
in 2024-2026, tropical and Lahiri-sidereal, from the Swiss Ephemeris' own analytical speeds, and three longitude
crossings that include a retrograde loop. `crates/eshana-engine/tests/events_oracle.rs` checks them. Every station
and crossing is found, with the same direction. The solver refines to 1 s; the differences below come from the
positions.

| Measured worst, 2024-2026 | spk (JPL DE440s) | analytic |
|---|---:|---:|
| Station longitude | 0.35″ | 288″ (Saturn), 51″ (Venus) |
| Station time, Mercury / Venus / Mars, tropical | 0.003 / 0.006 / 0.013 h | 0.04 / 0.24 / 0.10 h |
| Station time, Jupiter / Saturn, tropical | 0.054 / 0.122 h | 1.3 / 2.0 h |
| Station time, any body, Lahiri sidereal | 0.0095 h | 2.0 h |
| Crossing, as time × speed | 0.2″ | within each body's position error above |

Reading it:

- **Where a planet stations is well determined; when is not.** Speed is flat at a station, so a speed difference of
  1e-5 °/day moves a slow planet's station by minutes (Δt = Δspeed / acceleration) while its longitude agrees to a
  fraction of an arc-second. Compare station *longitudes* when it matters.
- With the kernel, tropical stations are 5-16× further from the oracle than sidereal ones. Tropical speed of date
  carries the nutation rate, and ESHANA's 4-term nutation series is not the oracle's full one; in the sidereal frame
  the two largely cancel.
- The analytic provider's Venus is 51″ from the oracle at its stations, more than the 22.4″ worst case of the nine
  position instants above. Use `spk` for timing work on any body.
- Stations are computed in the frame requested: a sidereal station is not the tropical one, by up to an hour for Saturn.

## What these numbers do and do not say

- They say the *geometry* is right to the stated precision. They say nothing about the tradition applied to it.
- The `analytic` provider is not suitable for an exact Saturn or Jupiter aspect, or for a body within ~0.01° of a boundary.
- **Time scale**: before 1972 and after 2027 the UTC→TT model differs from other libraries'; see
  [Time](../concepts/time.md). That is a separate source of difference from the provider's.
- Only nine instants are in the position fixture; the Swiss oracle has eight and six places. Wider sampling and a
  randomised differential test are tracked as `ACC-03`.

Reproduce: `make accuracy`; with a kernel, `ESHANA_SPK=… make golden`.
