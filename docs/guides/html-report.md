---
title: "ESHANA Studio (HTML chart app)"
description: "One self-contained, offline HTML app per chart: Panchang chakra with a South or North Indian centre, graha and bhāva pages, aspects in virūpas, ṣaḍbala, vargas to D60, Nadi matrix, daśā, Western, Human Design and provenance."
---

# ESHANA Studio (HTML chart app)

`eshana-report` turns a birth request into **one interactive HTML file**, ESHANA Studio: a multi-page app. The file holds every chart and table, the raw engine
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
| `--divisions` | all nineteen | vargas to include (D1 … D60) |
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
| `--icons`, `--chart-style`, `--centre` | `sphere`, `south`, `south` | default planet icons, chart style and chakra centre (viewers can change them) |
| `--sections` | all | any of `overview,chakra,charts,vargas,planets,planet,houses,house,aspects,strength,ashtakavarga,dasha,panchanga,special,yogas,kp,nadi,energetics,transits,progressions,western,human-design,numerology,matching,data` |
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
chart.to_html("vedic.html", sections=["overview", "chakra", "charts", "planets", "dasha"])
```

In Jupyter a `Chart` renders itself: put `chart` on the last line of a cell.

The SVG builders are public too, for apps that want only a picture: `chakra_svg(env)`, `south_indian_svg(...)`,
`north_indian_svg(...)`, `western_wheel_svg(env)` and `bodygraph_svg(hd_env)` in `eshana.report`.

## What is in the file: ESHANA Studio

The file is a small app, not a long page. A side rail lists the pages; each page opens with its own animation over a
slowly turning starfield, and the address bar keeps the page (`#/chakra`, `#/planet/moon`, `#/house/7`), so the back
button and bookmarks work. **⌘K** (or **Jump**) opens a palette that finds any page, graha or bhāva. **Show and style**
holds the viewer's choices (planet icons, chart style, chakra centre, layers, animation), kept in that browser only.

| Page | What you see | Engine data |
|---|---|---|
| **Overview** | the person, the moment and the place; lagna, Moon, tithi, vāra, yoga, karaṇa, current daśā chain; a small chakra and both charts; cards that lead into every page | `input`, `vedic.panchanga`, `vedic.dasha.*` |
| **Panchang chakra** | every point placed by exact degree on rings for degrees, 27 nakshatras, 108 padas (navāṃśa), 36 dreshkāṇas, 12 rāśis, bhāvas and SAV. The centre shows a **South Indian** chart, a **North Indian** chart or the orbit lanes. Layers: needles, drishti arcs, translucent **aspect marks**, rāśi drishti, Western aspects and animated transits. Turn the wheel to Mesha or the lagna; click a rāśi, nakshatra or pada to drill down | positions, `vedic.*` |
| **Charts** | South Indian, North Indian or both, for D1, D9 and the Sripati bhāva chalit. Each graha shows its degree and nakshatra. Aspects appear as **translucent graha marks** (for example `Ju*` and `Ve*` in the house they aspect), with ārūḍha padas in the corners. Clicking a house opens its bhāva page; clicking a graha opens its graha page | `vedic.*.dN`, `vedic.drishti.graha`, `vedic.bhava_chalit` |
| **Vargas** | a gallery of all 19 divisional charts (D1–D60, including D5, D6 and D8), each graha with its degree inside the varga, and the body × division grid | `vedic.*.dN` |
| **Grahas** and each **graha page** | one page per graha. It shows position (sidereal, tropical, latitude, **declination**, right ascension, speed, distance), nakshatra and its attributes, and lordship with functional nature. Aspects it casts and receives come in virūpas (sputa drishti, D1 and D9) and as rāśi drishti. Then the yogas and doshas it takes part in, its ṣaḍbala components, viṃśopaka, avasthā, KP lords, every varga with degree, and its Nadi links. Arrows step to the previous and next graha | positions, `vedic.aspects`, `vedic.shadbala`, `vedic.vimshopaka`, `vedic.yogas`, `vedic.kp` |
| **Bhāvas** and each **bhāva page** | the house as cusp and Sripati madhya, its lord, the grahas in it, the aspects falling on it, bhāva bala, SAV, the KP cusp, and its AstroEnergetics row (significations, body, quadrant, trikoṇa) | `vedic.bhava_chalit`, `vedic.bhava_bala`, `vedic.kp` |
| **Aspects** | the full graha × graha sputa drishti matrix in virūpas for D1 and D9, with graha drishti fractions and rāśi drishti | `vedic.aspects`, `vedic.drishti.*` |
| **Strength** | ṣaḍbala by component (sthāna, dig, kāla, ceṣṭā, naisargika, dṛk) with totals in virūpas and rūpas, the required minimum and the ratio; every component in detail; viṃśopaka in four schemes; bhāva bala | `vedic.shadbala`, `vedic.vimshopaka`, `vedic.bhava_bala` |
| **Aṣṭakavarga** | bhinna matrices and the sarva row as a heat map, plus prastāraka (who gives each bindu) | `vedic.ashtakavarga` |
| **Daśā** | Vimśottarī and Yoginī timelines with a NOW marker, drill-down to five levels, Sāḍe Sātī | `vedic.dasha.*`, `vedic.sade_sati` |
| **Pañcāṅga** | the five limbs with end times, rise and set, horā, choghaḍiyā and the inauspicious periods | `vedic.panchanga` |
| **Lagnas and points** | Bhāva, Horā, Ghaṭī and Indu lagna; yogi, avayogi and duplicate yogi; dagdha rāśis; 64th navāṃśa and 22nd dreshkāṇa from Moon and lagna; kartari; graha yuddha | `vedic.special_points`, `vedic.panchanga` |
| **Yogas**, **KP** | each rule with the grahas involved; KP cusps, sub and sub-sub lords, significators, ruling planets | `vedic.yogas`, `vedic.kp` |
| **Nadi** | the [direction matrix](nadi.md): signs as a spiral through North, East, South and West in three rounds. Each graha sits in its sign with its degree; click one to see its links | positions |
| **Energetics** | the twelve houses as body, dimension and question; quadrants, trikoṇas, energy bands and temperatures ([guide](nadi.md#astroenergetics)) | positions, reference table |
| Transits, Progressions, Western, Human Design, Numerology, Compatibility | as before; compatibility appears when a partner is given | all envelopes |
| **Provenance and data** | versions, assumptions, rule packs, refusals (**Not computed**), the raw envelopes | all envelopes |

**Copy summary** puts a plain-text list of positions and the current daśā on the clipboard. **JSON** saves the data,
and **Print** gives a clean layout of the current page.

Choose the defaults for a file with `--icons`, `--chart-style south|north|both` and `--centre south|north|lanes`. Pick
pages with `--sections` (for example `--sections overview,chakra,charts,planets,dasha`). The graha and bhāva pages come
with Grahas and Bhāvas.

### Tested in a real browser

`make studio-journeys` drives the generated file in headless Chrome ([`acceptance/studio/journeys.mjs`](../../acceptance/studio/journeys.mjs)),
with no extra dependency. The checks:

- every page renders without console errors;
- the chakra draws 27, 108 and 12 segments and one point per graha;
- the South Indian centre sits wholly inside the wheel, with every graha in the box of its sign;
- the South and North charts place grahas correctly and show translucent aspect marks;
- clicking a graha opens its page with aspects, yogas and vargas;
- all 19 vargas are drawn;
- the Nadi matrix holds every graha and draws links;
- the command palette jumps to a bhāva;
- the theme is remembered;
- nothing scrolls sideways at phone width.

CI runs it on every pull request.

### Planet icons

*Textured spheres* render each graha as a small shaded planet: banded Jupiter, ringed Saturn, cratered Moon and
Mercury, and Rahu and Ketu as dark shadow discs with a halo and a tail. *Glyphs* (☉ ☽ ♂ …) and *badges* (Su, Mo…)
are the alternatives. They are drawn in the browser, with no image files.

### Chart markers

Pushkara navāṃśa, vargottama and gaṇḍānta follow the fixed rules printed in the Panchang Chakra template. They are the
only values the page computes itself, and they are labelled `report.markers@0.1`.

## Privacy

A report contains birth data. ESHANA writes it only where you tell it to and sends it nowhere. Leave out `--name` and
`--place`, or use `--sections` to leave out the overview, when you share a chart publicly. See
[privacy and ethics](../concepts/privacy-and-ethics.md).
