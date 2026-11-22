#!/usr/bin/env python3
"""Build the local review preview in preview/ (git-ignored): studio files for a few charts, the built website, and
preview/index.html linking them. Nothing is published.

    make preview            # build everything
    make preview-serve      # also serve preview/ at http://127.0.0.1:8765/ (the website needs a server)

The studio files are self-contained and open straight from disk (file://).
"""
from __future__ import annotations

import datetime as dt
import html
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "preview"
REPORT = ROOT / ".venv" / "bin" / "eshana-report"
REF = "2026-11-21T00:00:00Z"

CHARTS = [
    ("studio-ramana-maharishi.html", "Sri Ramana Maharishi · 30 Dec 1879 01:00 LMT, Madurai",
     "The chart in the reference-software screenshots, with the same conventions (true node, centre unrefracted "
     "sunrise), so the two can be compared page by page.",
     ["--date", "1879-12-30", "--time", "01:00", "--offset-minutes", "321", "--lat", "9.9333", "--lon", "78.1167",
      "--name", "Sri Ramana Maharishi", "--gender", "male", "--place", "Madurai, India", "--node", "true", "--sunrise", "centre_unrefracted",
      "--reference", REF]),
    ("studio-example.html", "Example Person · 15 May 1990 14:30, New Delhi (with a partner)",
     "The example used throughout the docs, with a partner for compatibility.",
     ["--date", "1990-05-15", "--time", "14:30", "--tz", "Asia/Kolkata", "--lat", "28.6139", "--lon", "77.2090",
      "--name", "Example Person", "--place", "New Delhi, India", "--reference", REF,
      "--partner", "1992-08-20,09:15,Asia/Kolkata,19.076,72.8777"]),
    ("studio-unknown-time.html", "Unknown birth time · 15 May 1990, New Delhi",
     "What the studio does when the time is not known: values that need it are listed as not computed.",
     ["--date", "1990-05-15", "--tz", "Asia/Kolkata", "--lat", "28.6139", "--lon", "77.2090", "--reference", REF]),
]


def run(cmd: list[str], **kw) -> None:
    print("$", " ".join(cmd[:3]), "…")
    subprocess.run(cmd, check=True, **kw)


def main() -> int:
    OUT.mkdir(exist_ok=True)
    for name, _, _, args in CHARTS:
        run([str(REPORT), *args, "-o", str(OUT / name)])
    site = ROOT / "website"
    run(["npm", "run", "build"], cwd=site, env={**__import__("os").environ, "SITE_BASE": "/site/"},
        stdout=subprocess.DEVNULL)
    if (OUT / "site").exists():
        shutil.rmtree(OUT / "site")
    shutil.copytree(site / "dist", OUT / "site")
    built = dt.datetime.now().strftime("%Y-%m-%d %H:%M")
    items = "".join(
        f'<a class="tile" href="{n}"><b>{html.escape(t)}</b><span>{html.escape(d)}</span><code>preview/{n}</code></a>'
        for n, t, d, _ in CHARTS)
    (OUT / "index.html").write_text(f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>ESHANA preview</title>
<style>
:root{{--bg:#0b0d22;--ink:#ece9ff;--muted:#a9a5cc;--card:#14183a;--line:#2a2f60;--gold:#f5c451;--lilac:#8f7bff}}
body{{margin:0;background:radial-gradient(900px 500px at 80% -10%,rgb(143 123 255/18%),transparent 60%),var(--bg);color:var(--ink);font:15px/1.5 Inter,system-ui,sans-serif}}
main{{max-width:980px;margin:0 auto;padding:40px 16px}}h1{{font-size:30px;margin:0 0 6px}}p{{color:var(--muted)}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:14px;margin:18px 0}}
.tile{{display:flex;flex-direction:column;gap:6px;padding:16px;border:1px solid var(--line);border-radius:16px;background:var(--card);color:inherit;text-decoration:none;transition:transform .2s,border-color .2s}}
.tile:hover{{transform:translateY(-3px);border-color:var(--lilac)}}.tile span{{color:var(--muted);font-size:13.5px}}code{{color:var(--gold);font-size:12px}}
</style></head><body><main>
<h1>ESHANA · local preview</h1>
<p>Built {built} from the working tree. Nothing here is published. Rebuild with <code>make preview</code>.</p>
<h2>Chart studio (open directly)</h2><div class="grid">{items}</div>
<h2>Website</h2>
<p>The site needs a local server: run <code>make preview-serve</code>, then open
<a href="http://127.0.0.1:8765/site/" style="color:var(--gold)">http://127.0.0.1:8765/site/</a>.</p>
<h2>Notes</h2><p>What changed and why: <code>Debugging-Updates.md</code>. Progress against the plan: <code>planning/PROGRESS.md</code>.</p>
</main></body></html>
""", encoding="utf-8")
    print(f"preview ready: {OUT / 'index.html'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
