# Measured accuracy

Reference: JPL DE440s positions computed **independently** with Skyfield 1.55 as *apparent geocentric ecliptic
longitude and latitude of date* (light-time, aberration, deflection, IAU 2000A nutation). Fixture:
[`positions_de440s.json`](positions_de440s.json), 9 instants: 1950-01-01, 1975-06-15, 1992-05-28, 2000-01-01, 2010-03-21, 2024-02-29, 2026-11-20, 2040-07-04, 2060-12-31. The fixture's own TT is used, so these tables
measure the **provider**, not the time-scale model.

Errors are in arc-seconds of longitude (latitude in the last column). Regenerate with `make accuracy`
(and `ESHANA_SPK=/path/de440s.bsp cargo run -p eshana-ephemeris --example accuracy -- spk $ESHANA_SPK`).

## `analytic` provider (no data files)

| body | max lon err (arcsec) | median lon err (arcsec) | max lat err (arcsec) |
|---|---:|---:|---:|
| sun | 15.8 | 5.9 | 1.1 |
| moon | 2.9 | 1.9 | 7.6 |
| mercury | 27.8 | 4.2 | 1.9 |
| venus | 22.4 | 8.6 | 3.2 |
| mars | 31.5 | 7.2 | 1.6 |
| jupiter | 342.3 | 137.5 | 9.5 |
| saturn | 1025.6 | 286.9 | 36.1 |
| uranus | 103.3 | 51.3 | 2.9 |
| neptune | 44.0 | 21.7 | 1.3 |
| pluto | 49.2 | 21.0 | 6.7 |

Reading it: Sun, Moon and the inner planets are good to a few tens of arc-seconds. **Jupiter and Saturn are only good to
a few arc-minutes** (Saturn's worst case here is about 17′) because a fixed set of Keplerian elements cannot follow the
Jupiter-Saturn mutual perturbations. That is fine for a sign or a nakshatra pada, not for an exact aspect to
Saturn. Use the `spk` provider when it matters. Nothing outside 1800-2100 is accepted.

## `spk` provider (user-supplied `de440s.bsp`)

| body | max lon err (arcsec) | median lon err (arcsec) | max lat err (arcsec) |
|---|---:|---:|---:|
| sun | 0.3 | 0.1 | 0.0 |
| moon | 0.3 | 0.1 | 0.0 |
| mercury | 0.3 | 0.1 | 0.1 |
| venus | 0.3 | 0.1 | 0.0 |
| mars | 0.3 | 0.1 | 0.0 |
| jupiter | 0.3 | 0.1 | 0.0 |
| saturn | 0.3 | 0.1 | 0.2 |
| uranus | 0.3 | 0.1 | 0.0 |
| neptune | 0.3 | 0.1 | 0.0 |
| pluto | 0.3 | 0.1 | 0.0 |

Agreement with the independent pipeline is at the 0.3 arc-second level for every body. The remaining difference is the
4-term nutation series (about 0.5″) and the absence of gravitational light deflection (up to 1.7″ near the Sun);
neither is hidden. The test `spk_provider_agrees_to_two_arcseconds_when_a_kernel_is_available` enforces 2″.

## Other oracles (Swiss Ephemeris, used offline for numbers only)

| Quantity | Fixture | Tolerance enforced | Result |
|---|---|---:|---|
| Ayanamsha (Lahiri, Raman, Krishnamurti, Fagan-Bradley), 1950-2040 | `swiss_oracle.json` | 0.0006° | passes |
| Ascendant and Midheaven, 6 places x 8 instants | `swiss_oracle.json` | 0.01° | passes |
| House cusps: Placidus, Equal, Whole Sign | `swiss_oracle.json` | 0.02° | passes |
| Sidereal Sun and Moon, four ayanamshas | `swiss_oracle.json` | 0.02° | passes |
| Mean lunar node | `swiss_oracle.json` | 0.01° | passes |
| True (osculating) lunar node | `swiss_oracle.json` | 0.06° | passes; worst case about 0.044° |

## The time-scale model is a separate, named source of difference

Skyfield assumes TAI-UTC = 10 s before 1972 and its own forecast for the future; ESHANA uses the Espenak-Meeus
polynomial before 1972 (the real TT-UT, so a UTC-labelled 1950 instant is read as UT) and, after 2027, continues the
37 s leap-second value along the same polynomial's growth. On the fixture: 0.00 s for 1975-2026, -13.1 s for 1950,
+9.5 s for 2040, +27.7 s for 2060. For the Moon (0.55″/s) that is up to 15″ in 2060. Future ΔT is a forecast; the
result's `provenance.time_model` says which model produced it.
