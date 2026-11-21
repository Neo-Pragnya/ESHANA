# eshana

**ESHANA**: *Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations.*
From celestial positions to connected meaning.

ESHANA is an open-source computation layer written in Rust, with a thin Python facade. Its doctrine is
**Compute → Derive → Interpret**, with provenance on every result:

- positions are calculated;
- tradition rules (Vedic, Western, Human Design, numerology) are labelled as rules;
- the engine never emits an interpretation of its own.

```bash
pip install eshana
eshana-report --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 -o chart.html
```

`chart.html` is one self-contained file:

- the Panchang chakra;
- South and North Indian charts;
- divisional charts (D3 to D12);
- Vimśottarī daśā;
- a Western wheel with aspects;
- a Human Design BodyGraph;
- numerology;
- full provenance.

It opens offline and prints cleanly.

```python
from eshana import ChartEngine, Context

ctx = Context.from_local_birth(date="1992-05-28", time="20:05:00", timezone="Asia/Kolkata",
                               latitude=17.385, longitude=78.487)
chart = ChartEngine().vedic_chart(ctx, ayanamsha="lahiri", divisions=[1, 9], dasha=True)
print(chart.position("moon")["sign"]["name"], chart.attribute("vedic.moon.nakshatra")["name"])
print(chart.provenance["assumptions"])
chart.to_html("vedic.html")          # or just `chart` in a Jupyter cell
```

## Accuracy

Accuracy is measured, not promised:

- The optional `spk` provider reads a public-domain JPL DE440s kernel that you supply. It agrees with an independent
  pipeline to 0.3″.
- The default `analytic` provider needs no data files. It is good to tens of arc-seconds for the Sun, Moon and inner
  planets, and to arc-minutes for Jupiter and Saturn.

A suite of about 970 checks compares whole birth charts against:

- the Swiss Ephemeris, as an offline oracle;
- the official definition of the Lahiri ayanāṃśa;
- a recorded ProKerala chart;
- an independent re-implementation of every rule.

## Refusals

ESHANA refuses to guess. It returns an error for:

- a sidereal request without an ayanāṃśa;
- a local time that is ambiguous or that a clock change skipped;
- Placidus houses inside the polar circles.

An unknown birth time gives intervals, and the Ascendant is left out.

## Links

- Docs: <https://neo-pragnya.github.io/ESHANA/docs/>
- HTML reports: <https://neo-pragnya.github.io/ESHANA/docs/guides/html-report/>
- Accuracy suite: <https://neo-pragnya.github.io/ESHANA/docs/guides/accuracy-suite/>
- Source: <https://github.com/Neo-Pragnya/ESHANA>
- Licence: Apache-2.0, open source.

ESHANA is not a prediction service. Astrology, numerology and Human Design are traditions: ESHANA makes their rules
computable and auditable, and makes no claim that they have predictive validity.
