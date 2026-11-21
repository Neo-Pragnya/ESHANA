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

`provenance.provider_data` identifies the kernel by file name, the kernel's internal name, size and an FNV-1a hash, so a
result can be tied to the file that produced it (the hash identifies, it does not authenticate).

## Behaviour

- Reads little-endian DAF/SPK with type-2 segments. Anything else: `ESH-PROVIDER`.
- Out-of-span instants: `ESH-RANGE`, naming the kernel's span.
- A missing kernel is `ESH-CONFIG` with the download link; a corrupt one is `ESH-PROVIDER`.
- The kernel is read once per process (the Python `ChartEngine` and the CLI reuse it).
- Nodes: the mean node is the same polynomial as in `analytic`; the true node is osculating from the kernel's own Moon.

## Verifying your setup

```bash
ESHANA_SPK=$ESHANA_SPK cargo test -p eshana-ephemeris --test golden     # 2″ agreement with the independent fixture
ESHANA_SPK=$ESHANA_SPK cargo run -p eshana-ephemeris --example accuracy -- spk "$ESHANA_SPK"
```
