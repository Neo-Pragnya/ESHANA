---
title: "HTML chart reports"
description: "Export one self-contained HTML file with every chart and detail: Panchang chakra, South and North Indian charts, vargas, dasha, Western wheel, Human Design, numerology and provenance."
---

# HTML chart reports

`eshana-report` turns a birth request into **one interactive HTML file**, the chart studio. The file holds every chart and table, the raw engine
output, a light/dark toggle and a print layout. It makes **no network requests**, so you can email it, archive it or
open it offline years later.

Examples, generated from the real engine: [full birth chart](../../examples/reports/example-birth-chart.html) ·
[unknown birth time](../../examples/reports/example-unknown-time.html) ·
[Western + numerology only](../../examples/reports/example-western.html) ·
[rendered from saved CLI JSON](../../examples/reports/example-from-cli.html).

![Report pipeline](../architecture/svg/05-report-pipeline.svg)

## From the command line

`eshana-report` is installed with the Python package (`pip install eshana`).

```bash
eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata \
  --lat 28.6139 --lon 77.2090 --name "Example Person" --place "New Delhi, India" \
  -o chart.html
```

Unknown birth time: leave out `--time`. Positions and signs that stay the same all day are kept. The Ascendant, houses,
the North Indian chart and Human Design are listed under **Not computed**, with the engine's reason, rather than
guessed.

| Option | Default | Meaning |
|---|---|---|
| `--date`, `--time`, `--tz` | none | local civil date and time in an IANA zone; `--offset-minutes` if you only know the offset; `--utc` for an instant |
| `--fold earliest\|latest` | reject | which of a repeated local hour (DST fall-back) you mean |
| `--uncertainty-minutes N` | none | birth-time window ±N; values that could change are flagged |
| `--lat`, `--lon` | none | place; needed for the Ascendant and houses |
| `--name`, `--place`, `--title` | none | display only; `--name` also feeds name numerology |
| `--ayanamsha` | `lahiri` | `lahiri`, `raman`, `krishnamurti`, `fagan_bradley` |
| `--houses` | `whole_sign` | Vedic houses: `whole_sign`, `equal`, `placidus` |
| `--western-houses` | `placidus` | Western houses |
| `--node` | `mean` | `mean` or `true` lunar node |
| `--divisions` | all sixteen | vargas to include (D1 … D60) |
| `--no-dasha` | off | leave out Vimśottarī daśā |
| `--systems` | all four | any of `vedic,western,human_design,numerology` |
| `--school` | `pythagorean` | numerology school, or `chaldean` |
| `--provider`, `--spk` | `analytic` | use `spk` with a JPL kernel for sub-arc-second positions |
| `--features` | `all` | Vedic features (see [Vedic features](vedic-features.md)), or `none` |
| `--dasha-levels`, `--dasha-systems` | `3`, `vimshottari,yogini` | daśā depth and systems |
| `--reference` | now | instant for "current" periods and transits, recorded in the file |
| `--transit-days`, `--progression-years` | `365`, `90` | spans of the transit animation and progressions (0 = none) |
| `--sunrise` | `upper_limb_refracted` | sunrise convention for the pañcāṅga |
| `--partner` | none | second birth (`date,time,tz,lat,lon`) for compatibility |
| `--icons`, `--chart-style` | `sphere`, `south` | default planet icons and chart style (viewers can change them) |
| `--sections` | all | any of `summary,chakra,charts,planets,nakshatra,vargas,panchanga,dasha,strength,ashtakavarga,kp,jaimini,yogas,transits,progressions,western,human_design,numerology,matching,attributes,provenance,raw` |
| `--theme` | `auto` | `light`, `dark` or follow the reader's system |
| `-o` | stdout | output file |

### From JSON saved by the Rust CLI

When you use the Rust binary, or saved envelopes earlier, render them without re-computing:

```bash
eshana chart vedic --date 1996-12-07 --time 10:34 --tz Asia/Kolkata \
  --lat 13.0878 --lon 80.2785 --divisions 9 --dasha > vedic.json
eshana human-design --date 1996-12-07 --time 10:34 --tz Asia/Kolkata > hd.json
eshana-report --from-json vedic.json hd.json --title "Chennai 1996" -o chart.html
```

## From Python

```python
from eshana import Context, ChartEngine
from eshana.report import birth_report, render_html

ctx = Context.from_local_birth("1990-05-15", "14:30", "Asia/Kolkata", 28.6139, 77.2090)

# everything, in one call
html = birth_report(ctx, name="Example Person", place_label="New Delhi", divisions=[1, 9, 10])
open("chart.html", "w").write(html)

# or one chart object
chart = ChartEngine().vedic_chart(ctx, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
chart.to_html("vedic.html", sections=["summary", "chakra", "vedic", "divisional", "dasha"])
```

In Jupyter a `Chart` renders itself: put `chart` on the last line of a cell.

The SVG builders are public too, for apps that want only a picture: `chakra_svg(env)`, `south_indian_svg(...)`,
`north_indian_svg(...)`, `western_wheel_svg(env)` and `bodygraph_svg(hd_env)` in `eshana.report`.

## What is in the file: the chart studio

The page is an interactive studio, drawn in the browser from the embedded engine output. **Show and style** picks
the sections, planet icons, chart style, BodyGraph style and animation. The choices are kept in that browser only.

| Section | What you see | Engine data |
|---|---|---|
| Birth data | name, place, local time, UTC, zone, coordinates, JD, ΔT, ayanāṃśa; chips for lagna, Moon, tithi, vāra, yoga, karaṇa, HD type | `input`, `vedic.panchanga` |
| **Panchang chakra** | the Panchang Chakra wheel with every point placed: degree ticks, 27 nakshatras, 108 padas labelled with their navāṃśa sign, 36 dreshkāṇas (optional), 12 rāśis, bhāvas from the lagna, and each graha with a needle to its exact degree, so you can see which nakshatra, pada and navāṃśa it falls in. Hover any segment for what is there; click a rāśi, nakshatra or pada to turn the wheel to it and drill down (rāśi → nakshatra → pada); click a graha for its full details | positions, `vedic.*` |
| Layers on the wheel | graha drishti (animated arcs; dashed = special aspects), rāśi drishti, Western aspects, SAV bindus by sign, transits (with play and a date slider), pushkara, vargottama and gaṇḍānta markers | `vedic.drishti.*`, `vedic.ashtakavarga`, ephemeris |
| Charts | South Indian, North Indian or both, for D1 and any varga up to D60; ārūḍha padas (AL, UL, A2…) and drishti arrows as layers; click a house for its lord, occupants, aspects, SAV and KP cusp | `vedic.*.dN`, `vedic.jaimini.*` |
| Grahas and bhāvas | sidereal and tropical longitude, rāśi, nakshatra, pada, lord, navāṃśa, bhāva, motion, dignity, avasthā, combustion, KP sub-lord, markers; bhāva cusps | positions, `vedic.strength`, `vedic.kp` |
| Nakshatra ruler | all 27 with the grahas and lagna in each (with pada) and the Nakshatra Ruler attributes; click a row to find it on the wheel | positions, reference table |
| Vargas | body × division grid, with matches to D1 highlighted | `vedic.*.dN` |
| Pañcāṅga | the five limbs with end times, sunrise to moonset, Rāhu kālam and friends, the three special lagnas, and the 24 horās and 16 choghaḍiyās of the birth day | `vedic.panchanga` |
| Daśā | Vimśottarī and Yoginī timelines with a NOW marker; click to drill into antardaśās and deeper; the five-level current chain; Sāḍe Sātī windows | `vedic.dasha.*`, `vedic.sade_sati` |
| Strength | dignity, avasthā, combustion, dig bala and naisargika bala bars; the pañcadhā maitrī matrix | `vedic.strength`, `vedic.relationships` |
| Aṣṭakavarga | the 7 × 12 bhinna matrix and the sarva row as a heat map | `vedic.ashtakavarga` |
| KP | cusps and planets with sign, star, sub and sub-sub lords; A/B/C/D significators; ruling planets | `vedic.kp` |
| Jaimini | chara kārakas in both schemes, kārakāṃśa, ārūḍha padas | `vedic.jaimini.*` |
| Yogas and doshas | each rule in words, present or absent, and the grahas involved | `vedic.yogas` |
| Transits | positions at the reference instant against the natal chart (bhāva from lagna and Moon) | ephemeris |
| Progressions | secondary progressions by age as lines through the zodiac, and a table every five years | ephemeris |
| Western | tropical wheel with houses and aspect lines, an aspect grid, and positions | Western chart |
| Human Design | **detailed** (all 64 gates and 36 channels, personality black, design red, both striped) or **schematic** BodyGraph; activations | Human Design |
| Numerology, Compatibility, Provenance, Raw data | as computed; matching appears when a partner is given | all envelopes |

The **Copy summary** button puts a plain-text list of positions and the current daśā on the clipboard, for notes or for
an assistant. **Chakra SVG** and **Export JSON** save the wheel and the data.

### Planet icons

*Textured spheres* render each graha as a small shaded planet: banded Jupiter, ringed Saturn, cratered Moon and
Mercury, and Rahu and Ketu as dark shadow discs with a halo and a tail. *Glyphs* (☉ ☽ ♂ …) and *badges* (Su, Mo…)
are the alternatives. They are drawn in the browser, with no image files.

### Chart markers

Pushkara navāṃśa, vargottama and gaṇḍānta follow the fixed rules printed in the Panchang Chakra template. They are the
only values the page computes itself, and they are labelled `report.markers@0.1`.

## Privacy

A report contains birth data. ESHANA writes it only where you tell it to and sends it nowhere. Leave out `--name` and
`--place`, or use `--sections` to drop the summary, when you share a chart publicly. See
[privacy and ethics](../concepts/privacy-and-ethics.md).
