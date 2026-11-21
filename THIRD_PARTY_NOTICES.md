# Third-party notices

ESHANA is Apache-2.0. It depends on the open-source crates below and on published reference data. `deny.toml` enforces the licence
allow-list in CI; regenerate the exact list with `cargo tree --workspace --prefix none --edges normal | sort -u`.

## Direct Rust dependencies (all permissive)

| Crate | Licence | Use |
|---|---|---|
| serde, serde_json | MIT OR Apache-2.0 | (de)serialisation |
| thiserror | MIT OR Apache-2.0 | error types |
| jiff, jiff-tzdb | Unlicense OR MIT | civil time; jiff-tzdb bundles the IANA tz database (public domain). Replaced chrono and chrono-tz, which RustSec marks unmaintained (RUSTSEC-2026-0543, -0555) |
| clap | MIT OR Apache-2.0 | CLI |
| unicode-normalization | MIT OR Apache-2.0 | numerology text normalisation |
| pyo3 | MIT OR Apache-2.0 | Python bindings |
| proptest (dev) | MIT OR Apache-2.0 | property tests |

## Data and algorithms

- **IANA Time Zone Database**: public domain.
- **JPL approximate Keplerian elements** (E. M. Standish, "Keplerian Elements for Approximate Positions of the Major Planets",
  Table 1): published constants of a US-government laboratory; implemented independently.
- **Meeus, *Astronomical Algorithms*** (lunar series, nutation, precession, delta-T polynomial after Espenak and Meeus):
  published algorithms and numerical coefficients, implemented independently; no code copied.
- **JPL SPK/DAF format and DE kernels** (optional, user supplied): NASA/JPL, public domain; no kernel is bundled.

## Test-only references (never shipped)

- **JPL DE440s** positions computed with **Skyfield** (MIT): numbers only, in `benchmarks/golden/positions_de440s.json`.
- **Swiss Ephemeris** via `pyswisseph` (AGPL / commercial), used offline as an oracle for ayanāṃśa, houses, nodes and sidereal
  positions: numbers only, in `benchmarks/golden/swiss_oracle.json`. No Swiss Ephemeris code or data is in any published
  artefact (see `docs/adr/001-ephemeris-provider-and-licensing.md`).
