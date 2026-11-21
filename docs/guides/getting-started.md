---
title: "Getting started"
description: "Install, make a first chart, and read the result. About five minutes."
---

# Getting started

ESHANA is not published to a registry yet ([Roadmap](../roadmap.md)). Build from source:

```bash
git clone https://github.com/Neo-Pragnya/ESHANA && cd ESHANA
make install            # Rust toolchain check + a .venv with maturin and test tools
cargo build --release -p eshana-cli
export PATH="$PWD/target/release:$PATH"
eshana --version
```

For Python: `cd crates/eshana-python && maturin develop --uv` (or `make wheel` for a wheel in `dist/`).

## A first chart

```bash
eshana chart vedic --date 1992-05-28 --time 20:05:00 --tz Asia/Kolkata \
                   --lat 17.385 --lon 78.487 --divisions 1,9 --dasha
```

The output is one JSON envelope. The parts to look at:

```json
{
  "kind": "computed_chart",
  "input": { "instant_utc": "1992-05-28T14:35:00Z", "civil_offset_s": 19800, "birth_time_quality": "exact" },
  "calculation": { "zodiac": "sidereal", "ayanamsha": "lahiri", "ayanamsha_deg": 23.751007, "house_system": "whole_sign" },
  "positions": [ { "id": "moon", "longitude_deg": 358.79, "sign": { "name": "Meena", "dms": "28°47′27″" },
                  "epistemic": "calculated", "speed_deg_per_day": 12.97, "retrograde": false } ],
  "houses": { "ascendant": { "sign": { "name": "Dhanu", "dms": "2°25′39″" } } },
  "derived_attributes": [ { "id": "vedic.moon.nakshatra", "rule_version": "vedic.parashari@0.1",
                            "epistemic": "tradition_rule", "value": { "name": "Revati", "pada": 4, "lord": "Mercury" } } ],
  "interpretations": [],
  "provenance": { "rulesets": ["ayanamsha-lahiri@0.1", "vedic.parashari@0.1"], "assumptions": ["…"] }
}
```

Three things to notice:

1. The Moon is **calculated** (a position); *Revati, pada 4* is **derived by a rule** and names it.
2. `interpretations` is empty. ESHANA tells you what the rule says, never what it means.
3. `provenance.assumptions` lists every choice made for you, including that Lahiri was defaulted by `chart vedic`.

## The same moment, a different lens

```bash
eshana chart western --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --houses placidus --aspects
eshana compare vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --a ayanamsha=lahiri --b ayanamsha=fagan_bradley
```

`compare` lists what changed and says why: differences are consequences of the configuration, not defects.

## When it refuses

ESHANA prefers a clear refusal to a plausible guess.

```text
$ eshana positions --utc 2000-01-01T12:00:00Z --zodiac sidereal
ESH-CONFIG: invalid configuration: sidereal zodiac requires an explicit ayanamsha (...); ESHANA never picks one silently

$ eshana positions --date 2024-11-03 --time 01:30 --tz America/New_York
ESH-TIME-AMBIGUOUS: ... occurs twice (...); pass fold=earliest or fold=latest
```

Exit status is 2 and stdout holds an error envelope. `eshana explain ESH-LATITUDE` explains any code; the
[error reference](../reference/errors.md) lists them all.

## Next

[Python](python.md) · [Vedic](vedic.md) · [Western](western.md) · [Human Design](human-design.md) ·
[Numerology](numerology.md) · [Uncertain birth time](uncertain-birth-time.md)
