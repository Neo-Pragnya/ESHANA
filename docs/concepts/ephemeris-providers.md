---
title: "Ephemeris providers"
description: "The analytic and SPK providers, their measured accuracy, and why the Swiss Ephemeris is not bundled."
---

# Ephemeris providers

A provider turns a TT Julian day and a body into an apparent geocentric longitude, latitude and distance. The trait is
`EphemerisProvider`; the contract is the single frame described in [Architecture](architecture.md).

| Provider | Data | Accuracy (measured vs JPL DE440s) | Range | Licence footprint |
|---|---|---|---|---|
| `analytic` (default) | JPL's published Keplerian elements (Standish, Table 1) and the Meeus ch. 47 lunar series, in the source | Sun 16″, Moon 10″, Mercury–Mars ≤ 32″, Uranus–Pluto ≤ 103″, **Jupiter 342″, Saturn 1026″** (worst case, 1950–2060) | 1800–2100 | none: no data files |
| `spk` | a JPL SPK kernel you supply (`de440s.bsp`) | **≤ 0.7″ for every body** against an independent pipeline | the kernel's span (1849–2150 for de440s) | none bundled; NASA/JPL data is public domain, you download it |

Full tables, the method and the time-scale caveat: [Accuracy](../reference/accuracy.md).

## Choosing

- Charts at sign, nakshatra or house resolution, demos, tests: `analytic`. It needs nothing.
- Aspects to Saturn or Jupiter, boundary cases, anything near a sign cusp: `spk`.
- Both return the same frame, so results differ in digits, never in definition.

## Using the SPK provider

```bash
curl -O https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp   # ~32 MB
export ESHANA_SPK=$PWD/de440s.bsp
eshana positions --utc 2000-01-01T12:00:00Z --provider spk
```

The reader supports little-endian DAF/SPK with type 2 (Chebyshev position) segments, which covers the planetary DE
kernels. A big-endian or non-type-2 file is a clear `ESH-PROVIDER` error. See the [SPK guide](../guides/spk-kernel.md).

## The nodes

The **mean node** is a mean element (Meeus), identical in both providers. The **true node** is the *osculating* node of the
Moon's own orbit, computed from the provider's Moon, so it is as good as the Moon. It matches the Swiss Ephemeris
within 0.06° (worst case measured 0.044°).

## Why not the Swiss Ephemeris?

The Swiss Ephemeris is excellent and is the de-facto reference, but it is dual-licensed AGPL or a paid professional
licence. Linking it into an Apache-2.0 library would make the combined work AGPL. So it is **not** a dependency:

- It is used as an **offline test oracle** (`pyswisseph`, numbers only, in `benchmarks/golden/swiss_oracle.json`); nothing
  of it ships.
- An adapter is planned **behind a feature flag**, off by default, after the licence decision in
  [ADR-001](../adr/001-ephemeris-provider-and-licensing.md) and [Licensing](licensing.md).

## Adding a provider

Implement `supports`, `info` (with an honest accuracy statement and range) and `apparent`; `position` (speed) and
`check_range` come for free. Add a golden test against the DE440s fixture with a tolerance you can defend.
