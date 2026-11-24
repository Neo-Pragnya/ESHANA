#!/usr/bin/env python3
"""Animated SVG of how muhūrta windows are composed, drawn from a real `eshana muhurta` run.

Each constraint's blocked spans drop in row by row, the day is cut at every boundary, and the passing windows light up.
All spans, times and minutes come from the engine's envelope (timeline pieces, `removed_minutes`, `windows`); nothing
is drawn by hand. Pure CSS keyframes, so it plays in a GitHub README <img>; reduced motion shows the final frame.

    python assets/readme/src/muhurta_svg.py target/release/eshana assets/readme/muhurta-windows.svg
"""
from __future__ import annotations

import html
import json
import subprocess
import sys
from datetime import datetime

REQ = ["muhurta", "--lat", "13.08", "--lon", "80.27", "--tz", "Asia/Kolkata", "--start", "2026-12-01T00:30:00Z",
       "--end", "2026-12-02T00:30:00Z", "--profile", "general_auspicious"]
ROWS = [("no_rahu_kalam", "Rāhu kālam"), ("no_yamagandam", "Yamagaṇḍam"), ("no_gulika_kalam", "Gulika kālam"),
        ("no_rikta_tithi", "Rikta tithi"), ("no_amavasya", "Amāvāsyā"), ("no_panchaka", "Pañcaka"),
        ("no_vishti_karana", "Viṣṭi karaṇa")]
W, X0, X1, TOP, RH = 1200, 190, 1160, 112, 34
CYCLE = 16.0  # seconds per loop
GOLD, RED, GREEN, INK, MUTED, LINE = "#f5c451", "#ef5350", "#4cd18b", "#eef0ff", "#9aa1c7", "#2a3170"


def t(s: str) -> datetime:
    return datetime.fromisoformat(s.replace("Z", "+00:00"))


def main() -> int:
    exe, out = sys.argv[1], sys.argv[2]
    env = json.loads(subprocess.run([exe, *REQ], check=True, capture_output=True, text=True).stdout)
    t0, t1 = t(env["timeline"][0]["start_utc"]), t(env["timeline"][-1]["end_utc"])
    span = (t1 - t0).total_seconds()
    x = lambda s: X0 + (t(s) - t0).total_seconds() / span * (X1 - X0)  # noqa: E731
    local = lambda s: s[11:16]  # noqa: E731
    H = TOP + (len(ROWS) + 1) * RH + 150
    k = 0
    keys: list[str] = []

    def step(at: float, dur: float = .6) -> str:
        """A keyframe that keeps an element hidden until `at` seconds into the cycle, then shows it to the end."""
        nonlocal k
        k += 1
        a, b = at / CYCLE * 100, min(99.0, (at + dur) / CYCLE * 100)
        keys.append(f"@keyframes k{k}{{0%,{a:.2f}%{{opacity:0;transform:translateY(-6px)}}{b:.2f}%,100%{{opacity:1;transform:none}}}}")
        return f"k{k}"

    o = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="{W}" height="{H}" role="img" aria-labelledby="t d">',
         '<title id="t">How muhūrta windows are composed</title>',
         f'<desc id="d">A real run: Chennai, {local(env["timeline"][0]["start_local"])} on {env["timeline"][0]["start_local"][:10]} to the next sunrise, '
         f'profile {env["profile"]["id"]}. Each rule removes its spans; what remains, cut at every boundary, are the windows.</desc>']
    body = []
    body.append(f'<rect width="{W}" height="{H}" rx="20" fill="#0b0f2e"/>')
    body.append(f'<text x="40" y="48" font-size="24" font-weight="800" fill="{INK}">🪔 How muhūrta windows are composed</text>')
    body.append(f'<text x="40" y="76" font-size="14" fill="{MUTED}">A real run: Chennai, {env["timeline"][0]["start_local"][:10]}, profile '
                f'<tspan fill="{GOLD}" font-weight="700">{env["profile"]["id"]}</tspan> (v{env["profile"]["version"]}, {env["profile"]["status"]}). '
                f'Every rule removes its spans; what remains, cut at every boundary, are the windows.</text>')
    # time axis: every 3 hours, local
    for h in range(0, 25, 3):
        sec = h * 3600
        xx = X0 + sec / span * (X1 - X0)
        lab = f"{(6 + h) % 24:02d}:00"
        body.append(f'<line x1="{xx:.1f}" y1="{TOP - 14}" x2="{xx:.1f}" y2="{TOP + (len(ROWS) + 1) * RH}" stroke="{LINE}" stroke-dasharray="2 5"/>')
        body.append(f'<text x="{xx:.1f}" y="{TOP - 20}" font-size="11.5" fill="{MUTED}" text-anchor="middle">{lab}</text>')
    removed = env["removed_minutes"]
    for i, (cid, name) in enumerate(ROWS):
        y = TOP + i * RH
        body.append(f'<text x="{X0 - 14}" y="{y + 21}" font-size="14" font-weight="600" fill="{INK}" text-anchor="end">{name}</text>')
        body.append(f'<rect x="{X0}" y="{y + 6}" width="{X1 - X0}" height="{RH - 12}" rx="5" fill="#141a45"/>')
        pieces = [p for p in env["timeline"] if any(f["constraint"] == cid for f in p["failed"])]
        anim = step(1.0 + i * 1.0)
        if not pieces:
            body.append(f'<text style="animation:{anim} {CYCLE}s ease-out infinite" x="{X0 + 12}" y="{y + 21}" font-size="12" fill="{MUTED}" font-style="italic">none in this range</text>')
        for p in pieces:
            a, b = x(p["start_utc"]), x(p["end_utc"])
            lab = f'{local(p["start_local"])}–{local(p["end_local"])}'
            inside = b - a > len(lab) * 6.6 + 8  # a narrow span puts its label just after it
            tx = f'<text x="{(a + b) / 2:.1f}" y="{y + 21}" font-size="11" font-weight="700" fill="#fff" text-anchor="middle">{lab}</text>' if inside else \
                 f'<text x="{b + 6:.1f}" y="{y + 21}" font-size="11" font-weight="700" fill="{RED}">{lab}</text>'
            body.append(f'<g style="animation:{anim} {CYCLE}s ease-out infinite"><rect x="{a:.1f}" y="{y + 6}" width="{b - a:.1f}" height="{RH - 12}" rx="5" fill="{RED}" fill-opacity=".82"/>{tx}</g>')
        mins = removed.get(cid, 0)
        if mins:
            body.append(f'<text style="animation:{anim} {CYCLE}s ease-out infinite" x="{X0 - 14}" y="{y + 33}" font-size="10" fill="{RED}" text-anchor="end">−{mins:.0f} min</text>')
    # the cuts: a sweep, then a line at every boundary of the partition
    cut_at = 1.0 + len(ROWS) * 1.0 + .4
    yb0, yb1 = TOP - 6, TOP + (len(ROWS) + 1) * RH
    sweep = step(cut_at, .2)
    body.append(f'<g style="animation:{sweep} {CYCLE}s linear infinite">')
    for p in env["timeline"][1:]:
        xx = x(p["start_utc"])
        body.append(f'<line x1="{xx:.1f}" y1="{yb0}" x2="{xx:.1f}" y2="{yb1}" stroke="{GOLD}" stroke-width="1.4" stroke-dasharray="4 3"/>')
    body.append('</g>')
    # windows
    yw = TOP + len(ROWS) * RH + 8
    body.append(f'<text x="{X0 - 14}" y="{yw + 23}" font-size="15" font-weight="800" fill="{GREEN}" text-anchor="end">✨ Windows</text>')
    body.append(f'<rect x="{X0}" y="{yw + 6}" width="{X1 - X0}" height="{RH - 6}" rx="6" fill="#10301f" fill-opacity=".6"/>')
    for j, wdw in enumerate(env["windows"]):
        a, b = x(wdw["start_utc"]), x(wdw["end_utc"])
        an = step(cut_at + 1.0 + j * .45)
        body.append(f'<g style="animation:{an} {CYCLE}s ease-out infinite"><rect x="{a + 1:.1f}" y="{yw + 6}" width="{b - a - 2:.1f}" height="{RH - 6}" rx="6" fill="{GREEN}"/>'
                    + (f'<text x="{(a + b) / 2:.1f}" y="{yw + 25}" font-size="11" font-weight="800" fill="#062a17" text-anchor="middle">{round(wdw["duration_minutes"])}m</text>' if b - a > 26 else '')
                    + '</g>')
    # legend / facts
    total = sum(w["duration_minutes"] for w in env["windows"])
    yl = yw + RH + 34
    facts = [f"🧮 {len(env['timeline'])} pieces after cutting at every boundary",
             f"✅ {len(env['windows'])} windows · {total / 60:.1f} h pass",
             f"🚫 {sum(removed.values()) / 60:.1f} h removed, each by a named rule",
             "🔁 intervals are [start, end) in UTC; local times are presentation"]
    for i, f in enumerate(facts):
        body.append(f'<text x="{40 + (i % 2) * 560}" y="{yl + (i // 2) * 26}" font-size="14" fill="{INK}">{html.escape(f)}</text>')
    body.append(f'<text x="40" y="{H - 18}" font-size="11.5" fill="{MUTED}">Generated by assets/readme/src/muhurta_svg.py from `eshana {" ".join(REQ)}`</text>')
    style = ("<style>text{font-family:Inter,'Segoe UI',system-ui,-apple-system,sans-serif}" + "".join(keys)
             + "@media (prefers-reduced-motion:reduce){*{animation:none!important}}</style>")
    o.append(style)
    o.extend(body)
    o.append("</svg>")
    open(out, "w", encoding="utf-8").write("\n".join(o))
    print(f"wrote {out}: {len(env['timeline'])} pieces, {len(env['windows'])} windows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
