# ESHANA

**Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations**
*From celestial positions to connected meaning.*

A Rust-first computation layer for celestial positions, civil time, and the symbolic traditions built on them. It
keeps three things apart, in the data and not just in the docs:

| Layer | Example | Produced by |
|---|---|---|
| **Calculated** | ecliptic longitude, speed, Ascendant | verifiable code, tested against JPL DE440s and the Swiss Ephemeris |
| **Derived by a named system** | nakshatra and pada, aspect, divisional sign, Human Design gate, name number | a versioned **rule pack**, labelled `tradition_rule` |
| **Interpretive** | character, meaning, compatibility | **never the engine**: `interpretations` is always empty |

Every result carries **provenance**: engine, provider, tz-database and rule versions, and every assumption made on your
behalf. ESHANA does not claim astrology, numerology or Human Design have predictive validity; it makes their rules
computable and auditable.

```console
$ eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 9 --dasha
```
```python
from eshana import ChartEngine, Context

ctx = Context.from_local_birth(date="1992-05-28", time="20:05:00", timezone="Asia/Kolkata",
                               latitude=17.385, longitude=78.487)
chart = ChartEngine().vedic_chart(ctx, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
chart.position("moon")["sign"]["name"]          # 'Meena'
chart.attribute("vedic.moon.nakshatra")["name"] # 'Revati'
chart.provenance["assumptions"]                 # everything chosen for you
chart.to_html("chart.html")                     # one self-contained HTML report
```
```console
$ pip install eshana
$ eshana-report --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 -o chart.html
```

![ESHANA system overview](docs/architecture/svg/01-system-overview.svg)

## What works in 0.1 (everything below is tested; see `planning/TRACKING.md`)

- **Civil time**: IANA zones with historical offsets; DST folds and gaps are *errors with distinct codes*, never guesses.
- **Positions**: ten bodies and the lunar node (mean or osculating true), apparent geocentric, speed and retrograde.
  Two providers: `analytic` (no data files; arc-minute class) and `spk` (reads a JPL kernel you supply; **0.3″** against an
  independent pipeline). Measured, not promised: [`benchmarks/golden/accuracy.md`](benchmarks/golden/accuracy.md).
- **Vedic**: rāśi, nakshatra/pada, whole-sign bhāva, D1/D3/D4/D7/D9/D10/D12, Vimśottarī daśā, four ayanāṃśas.
- **Western**: tropical, whole-sign/equal/Placidus houses (refused where undefined), Ptolemaic aspects, applying state.
- **Human Design**: the design instant by solving the **88° solar arc** (not "88 days"), gates, lines, channels, centres, type,
  authority, profile.
- **Numerology**: Pythagorean and Chaldean as separate schemas; explicit normalisation; life path.
- **Uncertain birth time**: intervals and stability flags; the Ascendant and houses are never invented.
- **Method comparison** (`eshana compare`) and a **symbolic graph** (JSON, Mermaid, GraphML) with no causal edges.
- **Vedic features from YAML rule packs**: vargas D1–D60, Vimśottarī to five levels and Yoginī daśā, strength and
  dignity, graha and rāśi drishti, aṣṭakavarga, Jaimini kārakas and ārūḍhas, KP sub-lords and significators, 18 yogas
  and 6 doshas, Sāḍe Sātī ([guide](docs/guides/vedic-features.md)); **pañcāṅga** with rise/set, horā, choghaḍiyā and
  special lagnas ([guide](docs/guides/panchanga.md)); **aṣṭakūṭa matching** ([guide](docs/guides/matching.md)).
- **ESHANA Studio** (`eshana-report`, `Chart.to_html()`): one offline HTML file that works as a multi-page app. It
  has the Panchang Chakra with a South or North Indian chart in its centre, a page per graha and per bhāva, aspects
  in virūpas, ṣaḍbala and viṃśopaka, 19 vargas with degrees, a Nadi direction matrix, daśā timelines and a BodyGraph
  ([guide](docs/guides/html-report.md), [example](examples/reports/example-birth-chart.html)).
- **Birth-chart accuracy suite**: 44 reference charts against the Swiss Ephemeris, the Lahiri definition and a recorded
  ProKerala chart, PyJHora special lagnas, Swiss rise/set and ingresses, plus an independent re-derivation of every
  rule; about 1,300 checks
  ([guide](docs/guides/accuracy-suite.md), [results](benchmarks/accuracy/birth-chart-accuracy.md)).

## Guarantees you can rely on

Sidereal without an ayanāṃśa is refused. A repeated local time is refused. Placidus beyond the polar circle is refused. An
unknown birth time gets intervals, not noon. The same request returns the same bytes. A chart writes nothing to disk. Each is
a scenario in the release gate (`make acceptance`: 98 scenarios, 22 capabilities).

## Install

```bash
pip install eshana                                        # Python package + eshana-report (wheels: macOS, Linux)
make install && cargo build --release -p eshana-cli       # the Rust CLI, from source
```

Other platforms install from the sdist, which needs a Rust toolchain. See [ways to use ESHANA](docs/guides/ways-to-use.md).

Docs: <https://neo-pragnya.github.io/ESHANA/> · Start with [`docs/index.md`](docs/index.md) and
[`docs/concepts/epistemic-model.md`](docs/concepts/epistemic-model.md).

## Repository map

| Path | What |
|---|---|
| `crates/` | `eshana-core`, `-time`, `-ephemeris`, `-engine`, `-cli`, `-python` |
| `docs/` | concepts, guides, reference, ADRs; also the source of the website |
| `website/` | Astro site (interactive landing, demos); built from `docs/` |
| `acceptance/` | Gherkin capability scenarios and the release gate |
| `benchmarks/` | golden fixtures, measured provider accuracy and the birth-chart accuracy report |
| `accuracy/` | the birth-chart accuracy suite and its reference data (Swiss oracle, published values) |
| `examples/reports/` | example HTML chart reports generated by the real engine |
| `diagrams/` | architecture diagrams as code (`make diagrams`) |
| `schemas/` | the JSON Schema of the result envelope |
| `planning/` | `backlog.json` and the generated `TRACKING.md`; `NEXT-STEPS.md` |
| `ESHANA_Project_Vision_and_Master_Plan.md` | the original design record |

## Status and honesty

Version 0.2.0. ESHANA is an open-source tool stack published as libraries
([ADR-011](docs/adr/011-open-source-distribution.md)), licensed Apache-2.0. The `analytic` provider is arc-minute class
for Jupiter and Saturn; use `spk` when that matters. The Swiss Ephemeris (AGPL) is used only as an offline test oracle and
is never bundled; an optional, separately licensed adapter is planned. Next steps: [`docs/next-steps.md`](docs/next-steps.md).
