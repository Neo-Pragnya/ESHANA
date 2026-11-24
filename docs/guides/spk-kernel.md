---
title: "Using a JPL kernel"
description: "Download de440s.bsp and switch to arc-second accuracy."
---

# Using a JPL kernel

```bash
curl -O https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp
export ESHANA_SPK="$PWD/de440s.bsp"
eshana chart western --utc 2000-01-01T12:00:00Z --provider spk --lat 51.5 --lon -0.13
```

`de440s.bsp` (32 MB) covers 1849-12-26 to 2150-01-22. `de440.bsp` covers 1550–2650 and is larger. Both are NASA/JPL public
domain data; ESHANA does not bundle them.

## The kernel catalogue

```bash
eshana kernel list                    # every kernel ESHANA knows: size, NAIF's MD5, span, and whether it is present
eshana kernel verify de440.bsp        # MD5 and size against NAIF's published checksum (exit 2 on a mismatch)
eshana kernel path de440              # where a catalogue name is looked up
python scripts/fetch_kernel.py de440  # from the repository: a resumable download, kept only if it verifies
eshana chart western --utc 1600-03-10T12:00:00Z --provider spk --spk de440   # a catalogue name works as --spk
```

| Name | Size | Span | For |
|---|---|---|---|
| `de440s` | 33 MB | 1849 – 2150 | modern charts (the golden tests use it) |
| `de440` | 120 MB | 1550 – 2650 | from the Gregorian reform on |
| `de441-part1` | 1.65 GB | 13,201 BCE – 1969 | ancient dates |
| `de441-part2` | 1.66 GB | 1969 – 17,191 | the far future |
| `de442s`, `de442` | 33 / 120 MB | as DE440 | the 2025 integration |
| `sb441-n16` | 646 MB | 1550 – 2650 | Ceres, Pallas, Juno, Vesta and twelve more asteroids (JPL SSD; type 2); use beside a planetary kernel |

Chiron (and any other small body) comes from JPL Horizons: `python scripts/fetch_kernel.py horizons:2060 --start
1900-01-01 --stop 2100-01-01` saves `horizons-2060.bsp` (SPK type 21); then `--spk de440,/path/to/horizons-2060.bsp
--bodies chiron`. Several kernels are given comma-separated, a planetary one first. ESHANA reads segment types 2, 3
and 21, and both asteroid numbering styles (2000000 + n and Horizons' 20000000 + n).

Catalogue names are looked up in `$ESHANA_KERNEL_DIR`, else `$XDG_CACHE_HOME/eshana/kernels`, else
`~/.cache/eshana/kernels`. The engine never downloads anything: the fetch script is a repository tool, not part of
the package. Kernels over 256 MB (each `de441` part) are read on demand, one Chebyshev record at a time, instead of
into memory.

`provenance.provider_data` identifies the kernel by file name, the kernel's internal name, size and an FNV-1a hash, so a
result can be tied to the file that produced it (the hash identifies, it does not authenticate).

## Behaviour

- Reads little-endian DAF/SPK with type-2 (position) and type-3 (position and velocity) Chebyshev segments and
  type-21 extended modified difference arrays (JPL Horizons). Anything else: `ESH-PROVIDER`.
- Out-of-span instants: `ESH-RANGE`, naming the kernel's span.
- A missing kernel is `ESH-CONFIG` with the download link; a corrupt one is `ESH-PROVIDER`.
- The kernel is read once per process (the Python `ChartEngine` and the CLI reuse it).
- Nodes: the mean node is the same polynomial as in `analytic`; the true node is osculating from the kernel's own Moon.

## Verifying your setup

```bash
ESHANA_SPK=$ESHANA_SPK cargo test -p eshana-ephemeris --test golden     # 2″ agreement with the independent fixture
ESHANA_SPK=$ESHANA_SPK cargo run -p eshana-ephemeris --example accuracy -- spk "$ESHANA_SPK"
```
