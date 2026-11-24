"""Animated terminal walkthroughs for the README (rendered by the living-diagrams skill's term_flow.py).

Every line of output below was captured from a real run of eshana 0.4.0 (2026-11-24); only the URL token and port of
`eshana-report new` are as they happened to be. Regenerate when the commands or their output change:

    python <Skills>/living-diagrams/scripts/term_flow.py assets/readme/src/walkthroughs.py --out assets/readme
"""
SPECS = [
    {
        "id": "walkthrough-studio",
        "title": "🪐 The studio in three commands",
        "subtitle": "Install from PyPI, write a chart studio, or open a form and let ESHANA write it for you.",
        "theme_name": "night",
        "rows": 13,
        "steps": [
            {"title": "Install", "text": "One abi3 wheel for CPython 3.9 and later; no other dependencies."},
            {"title": "A chart studio", "text": "One offline HTML file; --live embeds the engine for recomputing."},
            {"title": "Or fill in a form", "text": "A local form waits for the birth details, writes the chart, then stops."},
        ],
        "session": [
            {"cmd": "pip install eshana", "step": 0, "out": [{"text": "Successfully installed eshana-0.4.0", "tone": "ok"}]},
            {"cmd": "eshana-report --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 --name \"Sri Ramana Maharishi\" --live -o ramana.html",
             "step": 1, "out": [{"text": "wrote ramana.html (6,984,061 bytes)", "tone": "ok"}]},
            {"cmd": "eshana-report new -o charts/", "step": 2,
             "out": ["eshana-report new: waiting for birth details at http://127.0.0.1:57316/WKmT…/",
                     {"text": "  (Ctrl-C to stop)", "tone": "muted"},
                     "# the form opens in your browser; you enter the birth details",
                     {"text": "charts/asha-rao-1990-05-15.html", "tone": "ok"}]},
        ],
    },
    {
        "id": "walkthrough-engine",
        "title": "🧭 One engine, every question",
        "subtitle": "The eshana CLI prints one JSON envelope per question; jq picks out a few lines.",
        "theme_name": "night",
        "rows": 15,
        "steps": [
            {"title": "A Vedic chart", "text": "Sidereal positions with sign and degree, under the ayanāṃśa you name."},
            {"title": "Derived by a rule", "text": "Nakshatra and pada come from a versioned rule pack."},
            {"title": "Muhūrta windows", "text": "Exact [start, end) intervals; Abhijit skips Wednesday."},
            {"title": "KP horary", "text": "Number 100 fixes the Ascendant; cusps and sub-lords follow."},
            {"title": "Never a guess", "text": "A clock time that did not exist is an error with a stable code."},
        ],
        "session": [
            {"cmd": "eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 --ayanamsha lahiri | jq -r '.positions[:5][] | \"\\(.id)\\t\\(.sign.name) \\(.sign.dms)\"'",
             "step": 0, "out": ["sun      Dhanu 15°35′41″", "moon     Mithuna 28°19′42″", "mercury  Vrishchika 23°06′31″", "venus    Vrishchika 0°28′39″", "mars     Mesha 21°58′40″"]},
            {"cmd": "… | jq -r '.derived_attributes[] | select(.id==\"vedic.moon.nakshatra\") | .value | \"\\(.name), pada \\(.pada)\"'",
             "step": 1, "out": [{"text": "Punarvasu, pada 3", "tone": "accent"}]},
            {"cmd": "eshana muhurta --lat 13.08 --lon 80.27 --tz Asia/Kolkata --start 2026-12-01T00:00:00Z --end 2026-12-04T00:00:00Z --profile abhijit | jq -r '.windows[] | …'",
             "step": 2, "out": ["2026-12-01T11:34  46 min  Mangalavara", "2026-12-03T11:35  46 min  Guruvara"]},
            {"cmd": "eshana kp-horary --number 100 --utc 2026-11-24T06:00:00Z --lat 13.08 --lon 80.27 | jq -r '…'",
             "step": 3, "out": ["Ascendant Simha 24°00′00″", "cusp 1: star venus, sub saturn"]},
            {"cmd": "eshana chart vedic --date 2021-03-14 --time 02:30 --tz America/New_York --lat 40.7 --lon -74 | jq -r '.error | …'",
             "step": 4, "out": [{"text": "ESH-TIME-NONEXISTENT: non-existent local time: 2021-03-14T02:30:00 in America/New_York was skipped by a clock change", "tone": "warn"}]},
        ],
    },
]
