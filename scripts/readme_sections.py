#!/usr/bin/env python3
"""Fill the generated sections of README.md from the real engine and the gallery, so the README cannot drift.

  GALLERY    every studio view in website/src/data/gallery.json, linked to the live report on the docs site
  REFERENCE  operations (`info`), CLI commands (`eshana --help`), method families (`methods`), error codes (`doctor`)

    python scripts/readme_sections.py            # rewrite the sections between the markers
    python scripts/readme_sections.py --check    # exit 1 if they are stale (make docs-check)

Uses ESHANA_BIN, or target/release/eshana. Every operation the engine reports must have a description below: a new
operation fails the check until the README says what it does.
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
SITE = "https://neo-pragnya.github.io/ESHANA"
EXE = os.environ.get("ESHANA_BIN") or str(ROOT / "target" / "release" / "eshana")

OPS = {
    "positions": ("🌍", "apparent geocentric positions only"),
    "chart": ("🪐", "a Vedic (sidereal) or Western (tropical) chart, with the requested features"),
    "human_design": ("🧬", "design instant by the 88° solar arc, gates, lines, channels, type, authority, profile"),
    "numerology": ("🔢", "name and birth-date numerology under a named school"),
    "compare": ("🔁", "the same input under two configurations, with a diff of what changed"),
    "graph": ("🕸️", "a symbolic graph of a chart: JSON, Mermaid or GraphML, no causal edges"),
    "panchanga": ("🌅", "tithi, vāra, nakshatra, yoga, karaṇa, sunrise, horā, choghaḍiyā and the kālams"),
    "matching": ("💞", "aṣṭakūṭa matching plus the supplementary rules, each cancellation listed"),
    "ephemeris": ("📈", "longitudes at regular steps (transit animation, progressions)"),
    "ingresses": ("🚪", "sign ingresses of one body between two instants"),
    "stations": ("🔄", "stations and retrograde intervals, clipped flags at the range ends"),
    "crossings": ("🎯", "when a body reaches a point or an aspect to it, with touches reported apart"),
    "rulesets": ("📜", "the built-in YAML rule packs"),
    "methods": ("📐", "every method family and id the engine implements"),
    "ashtakavarga_transits": ("🗓️", "dated aṣṭakavarga: the natal bindus each graha reads, between ingresses"),
    "muhurta_windows": ("🪔", "muhūrta windows composed from factor timelines under a profile"),
    "personal_calendar": ("📅", "windows with tārā and candra bala from a birth; JSON, CSV or ICS"),
    "pair_calendar": ("👫", "windows that suit two people and the event, each explained"),
    "progressions": ("📆", "secondary progressions with fractional age and named angle methods"),
    "kp_table": ("🔢", "the KP 1–249 horary number table"),
    "kp_horary": ("🎯", "a KP horary chart: the number fixes the Ascendant"),
    "synthetic_chart": ("🧪", "positions you place by hand, labelled synthetic (the playground)"),
    "research_window_score": ("📊", "the experimental window score, with coverage and sensitivity"),
    "rise_set": ("🌄", "rise, upper transit and set of any body, and twilight, for local days at a place"),
    "returns": ("🔁", "solar, lunar or any body's returns to its natal longitude, with a chart at each (relocatable)"),
    "gochara": ("🚶", "transits counted from the natal Moon and lagna, with retrograde and combustion intervals"),
    "eclipses": ("🌑", "solar and lunar eclipses: type, time of greatest eclipse, magnitude, γ"),
    "synastry": ("💑", "two charts compared: inter-aspects, house overlays, midpoint composite, Davison chart"),
    "info": ("ℹ️", "versions, operations, method families and ayanāṃśa definitions"),
}


def run(*args: str) -> str:
    return subprocess.run([EXE, *args], check=True, capture_output=True, text=True).stdout


def gallery() -> str:
    items = json.loads((ROOT / "website" / "src" / "data" / "gallery.json").read_text(encoding="utf-8"))["items"]
    rows, per = [], 4
    for i in range(0, len(items), per):
        cells = "".join(
            f'<td width="25%" valign="top"><a href="{SITE}/{it["link"]}"><img src="examples/{it["image"]}" alt="{it["title"]}"/></a>'
            f'<br/><b>{it["title"]}</b><br/><sub>{it["caption"]}</sub></td>' for it in items[i:i + per])
        rows.append(f"<tr>{cells}</tr>")
    return f"<table>\n" + "\n".join(rows) + "\n</table>"


def reference() -> str:
    doctor = json.loads(run("doctor"))
    methods = json.loads(run("methods"))
    ops = doctor["operations"]
    missing = [o for o in ops if o not in OPS]
    if missing:
        raise SystemExit(f"README has no description for operation(s): {', '.join(missing)} (add them to scripts/readme_sections.py)")
    out = [f"<details>\n<summary><b>⚙️ All {len(ops)} operations</b> (the <code>op</code> of a request)</summary>\n",
           "| | `op` | What it answers |", "|---|---|---|"]
    out += [f"| {OPS[o][0]} | `{o}` | {OPS[o][1]} |" for o in ops]
    out.append("\nFields of each: [docs/reference/request.md](docs/reference/request.md).\n</details>\n")
    helptext = run("--help")
    cmds = re.findall(r"^  ([a-z][a-z-]+)\s{2,}(.+)$", helptext.split("Commands:", 1)[1].split("Options:", 1)[0], re.M)
    out += [f"<details>\n<summary><b>🧰 All {len([c for c in cmds if c[0] != 'help'])} CLI commands</b></summary>\n",
            "| Command | What it does |", "|---|---|"]
    out += [f"| `eshana {c}` | {d.strip()} |" for c, d in cmds if c != "help"]
    out.append("\nEvery command prints one JSON envelope; `--compact` for one line. Errors print an error envelope and exit 2.\n</details>\n")
    fams = methods["families"]
    out += [f"<details>\n<summary><b>📐 All {len(fams)} method families</b> (<code>eshana methods</code>)</summary>\n",
            "| Family | Method ids |", "|---|---|"]
    out += [f"| `{f['family']}` | {', '.join('`' + m['id'] + '`' for m in f['methods'])} |" for f in fams]
    out.append("\nAn id that is not listed is refused with `ESH-METHOD`, never replaced by a default.\n</details>\n")
    codes = doctor["error_codes"]
    out += [f"<details>\n<summary><b>🚨 All {len(codes)} error codes</b> (<code>eshana explain CODE</code>)</summary>\n",
            "| Code | Meaning |", "|---|---|"]
    out += [f"| `{c['code']}` | {c['meaning']} |" for c in codes]
    out.append("\nCodes are stable across versions: [docs/reference/errors.md](docs/reference/errors.md).\n</details>")
    return "\n".join(out)


def fill(text: str, name: str, body: str) -> str:
    pat = re.compile(rf"(<!-- {name}:START -->\n).*?(<!-- {name}:END -->)", re.S)
    if not pat.search(text):
        raise SystemExit(f"README.md has no {name} markers")
    return pat.sub(lambda m: m.group(1) + body + "\n" + m.group(2), text)


def main() -> int:
    if not Path(EXE).exists():
        print(f"readme sections: skipped ({EXE} not built; run make build-release)")
        return 0
    old = README.read_text(encoding="utf-8")
    new = fill(fill(old, "GALLERY", gallery()), "REFERENCE", reference())
    if "--check" in sys.argv:
        if new != old:
            print("README.md generated sections are stale: run python scripts/readme_sections.py")
            return 1
        print("readme sections ok")
        return 0
    README.write_text(new, encoding="utf-8")
    print("wrote README.md generated sections")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
