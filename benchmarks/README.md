# Benchmarks and golden data

| File | What | Produced by |
|---|---|---|
| `golden/positions_de440s.json` | apparent geocentric ecliptic longitude, latitude and distance of date for ten bodies at 9 instants, from JPL DE440s | **Skyfield 1.55**: `earth.at(t).observe(body).apparent().ecliptic_latlon(epoch='date')` |
| `golden/swiss_oracle.json` | ayanāṃśa (4 models), sidereal Sun and Moon, mean and true node, ASC/MC and cusps for Placidus, Equal, Whole Sign at 6 places, at 8 instants | **pyswisseph 2.10.3** (AGPL) as an *offline oracle*; numbers only |
| `golden/accuracy.md` | the measured-accuracy report | `make accuracy` and the `spk` run |

## Regenerating

```bash
uv venv refenv && uv pip install --python refenv/bin/python skyfield pyswisseph numpy
# positions: load de440s.bsp (https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp) with skyfield,
# for each instant compute the apparent ecliptic_latlon(epoch='date') of each body from earth, and dump
# {utc, jd_tt, delta_t_s, bodies: {name: {lon, lat, dist_au}}}.
# oracle: swisseph.get_ayanamsa_ut, calc_ut(..., FLG_MOSEPH | FLG_SIDEREAL), houses_ex for 'P', 'E', 'W'.
```

Skyfield's `ts.utc()` uses its own ΔT; the fixture stores `jd_tt` so tests can isolate the provider from ESHANA's time model.
Do not loosen a tolerance to make a test pass: investigate, or document the cause here and in `reference/accuracy.md`.
