"""Self-contained HTML chart reports.

``render_html`` turns engine envelopes into ONE HTML file: inline CSS, inline SVG, the raw envelopes embedded as JSON,
no network access, light/dark/print layouts. ``birth_report`` runs the engine for every system you ask for and renders
the result. The visual language (palette, graha colours, rashi/nakshatra/pada rings) follows the Panchang Chakra
template in the author's Astrology repository.

Doctrine: the renderer formats what the engine computed. The only values it adds are *chart markers* (pushkara
navamsha, vargottama, gandanta) that follow fixed published rules from that template; they are labelled
``report.markers@0.1`` wherever they appear. Nothing here interprets a chart.

Command line::

    eshana-report --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
        --name "Example Person" -o chart.html
    eshana chart vedic ... > vedic.json && eshana-report --from-json vedic.json -o chart.html
"""

from __future__ import annotations

import argparse
import datetime as _dt
import html
import json
import math
import sys
from importlib import resources
from typing import Any, Dict, Iterable, List, Optional, Sequence, Tuple

__all__ = ["birth_report", "render_html", "envelopes_for", "main", "SECTIONS", "chakra_svg", "south_indian_svg",
           "north_indian_svg", "western_wheel_svg", "bodygraph_svg", "markers"]

MARKER_RULES = "report.markers@0.1"
# ── tradition tables (names only; every number comes from the engine) ─────────────────────────────────────────────
SIGN_EN = ["Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius", "Capricorn",
           "Aquarius", "Pisces"]
SIGN_SKT = ["Mesha", "Vrishabha", "Mithuna", "Karka", "Simha", "Kanya", "Tula", "Vrischika", "Dhanu", "Makara",
            "Kumbha", "Meena"]
GLYPH = ["♈", "♉", "♊", "♋", "♌", "♍", "♎", "♏", "♐", "♑", "♒", "♓"]
SIGN_LORD = ["Ma", "Ve", "Me", "Mo", "Su", "Me", "Ve", "Ma", "Ju", "Sa", "Sa", "Ju"]
ELEMENT = ["Fire", "Earth", "Air", "Water"]
NAKSHATRA = ["Ashwini", "Bharani", "Krittika", "Rohini", "Mrigashirsha", "Ardra", "Punarvasu", "Pushya", "Ashlesha",
             "Magha", "Purva Phalguni", "Uttara Phalguni", "Hasta", "Chitra", "Swati", "Vishakha", "Anuradha",
             "Jyeshtha", "Mula", "Purva Ashadha", "Uttara Ashadha", "Shravana", "Dhanishtha", "Shatabhisha",
             "Purva Bhadrapada", "Uttara Bhadrapada", "Revati"]
NAK_SHORT = {"Mrigashirsha": "Mrigashira", "Purva Phalguni": "P. Phalguni", "Uttara Phalguni": "U. Phalguni",
             "Purva Ashadha": "P. Ashadha", "Uttara Ashadha": "U. Ashadha", "Shatabhisha": "Shatabhisha",
             "Purva Bhadrapada": "P. Bhadra", "Uttara Bhadrapada": "U. Bhadra"}
NAK_LORD = ["Ke", "Ve", "Su", "Mo", "Ma", "Ra", "Ju", "Sa", "Me"]

# Graha palette from the Panchang Chakra template (dot = marker, band = rashi band, nak = nakshatra ring)
GRAHA = {
    "Su": ("Surya", "Sun", "#F39A1E", "#F4B31B", "#F4B31B"),
    "Mo": ("Chandra", "Moon", "#FFFFFF", "#FFFFFF", "#FFFFFF"),
    "Ju": ("Guru", "Jupiter", "#FFE81A", "#FFEE1E", "#FFE21C"),
    "Ra": ("Rahu", "Rahu", "#6B4FA8", "#B6A8DA", "#B6A8DA"),
    "Me": ("Budha", "Mercury", "#C4D84B", "#CCDB57", "#A9C79A"),
    "Ve": ("Shukra", "Venus", "#3FE0E0", "#52E5E2", "#E39479"),
    "Ke": ("Ketu", "Ketu", "#9C9C9C", "#DCDCDC", "#DCDCDC"),
    "Sa": ("Shani", "Saturn", "#7E99C9", "#C8D5EA", "#8DA2D0"),
    "Ma": ("Mangala", "Mars", "#D3133F", "#F2A6AD", "#E0583B"),
}
PADA_FILL = {"Fire": "#FFE24D", "Earth": "#A6EDD5", "Air": "#C5EBA0", "Water": "#6FE2EE"}
OUTER = {"Ur": "#5B8DEF", "Ne": "#3AA6A0", "Pl": "#8B5E3C"}

# engine body id -> (abbreviation, graha key or outer key, display name)
BODY = {
    "sun": ("Su", "Su", "Sun"), "moon": ("Mo", "Mo", "Moon"), "mercury": ("Me", "Me", "Mercury"),
    "venus": ("Ve", "Ve", "Venus"), "mars": ("Ma", "Ma", "Mars"), "jupiter": ("Ju", "Ju", "Jupiter"),
    "saturn": ("Sa", "Sa", "Saturn"), "mean_node": ("Ra", "Ra", "Rahu (mean node)"),
    "true_node": ("Ra", "Ra", "Rahu (true node)"), "south_node": ("Ke", "Ke", "Ketu"),
    "uranus": ("Ur", "Ur", "Uranus"), "neptune": ("Ne", "Ne", "Neptune"), "pluto": ("Pl", "Pl", "Pluto"),
}
LORD_NAME = {"Su": "Sun", "Mo": "Moon", "Ma": "Mars", "Me": "Mercury", "Ju": "Jupiter", "Ve": "Venus", "Sa": "Saturn",
             "Ra": "Rahu", "Ke": "Ketu"}

HD_CENTER_GATES = {
    "head": [64, 61, 63], "ajna": [47, 24, 4, 17, 43, 11],
    "throat": [62, 23, 56, 35, 12, 45, 33, 8, 31, 20, 16], "g": [1, 13, 25, 46, 2, 15, 10, 7],
    "heart": [21, 40, 26, 51], "sacral": [5, 14, 29, 59, 9, 3, 42, 27, 34],
    "spleen": [48, 57, 44, 50, 32, 28, 18], "solar_plexus": [36, 22, 37, 6, 49, 55, 30],
    "root": [53, 60, 52, 19, 39, 41, 58, 38, 54],
}
GATE_CENTER = {g: c for c, gs in HD_CENTER_GATES.items() for g in gs}

NAV = 10.0 / 3.0
NAKS = 40.0 / 3.0
PUSHKARA_POS = {"Fire": (6, 8), "Earth": (2, 4), "Air": (5, 7), "Water": (0, 2)}
GANDANTA_PADAS = (107, 0, 35, 36, 71, 72)


def _esc(x: Any) -> str:
    return html.escape(str(x), quote=True)


def _dms(x: float) -> str:
    x = x % 360.0
    d = int(x)
    m_f = (x - d) * 60
    m = int(m_f)
    s = int(round((m_f - m) * 60))
    if s == 60:
        m, s = m + 1, 0
    if m == 60:
        d, m = d + 1, 0
    return f"{d}°{m:02d}′{s:02d}″"


def _deg_in_sign(lon: float) -> str:
    return _dms(lon % 30.0)


def _nak_reference() -> List[Dict[str, str]]:
    try:
        raw = resources.files("eshana").joinpath("data/nakshatra_reference.json").read_text(encoding="utf-8")
        return json.loads(raw)["rows"]
    except Exception:  # pragma: no cover - data file is part of the wheel
        return []


def markers(lon: float) -> List[str]:
    """Chart markers for a sidereal longitude, by the fixed rules of the Panchang Chakra template."""
    i = int(math.floor((lon % 360.0) / NAV + 1e-9)) % 108
    rashi, pos, nsign = i // 9, i % 9, i % 12
    out = []
    if pos in PUSHKARA_POS[ELEMENT[rashi % 4]]:
        out.append("pushkara navamsha")
    if nsign == rashi:
        out.append("vargottama")
    if i in GANDANTA_PADAS:
        out.append("gandanta")
    return out


# ── envelope helpers ───────────────────────────────────────────────────────────────────────────────────────────────
def _attrs(env: Optional[Dict[str, Any]]) -> Dict[str, Dict[str, Any]]:
    return {a["id"]: a for a in (env or {}).get("derived_attributes", [])}


def _bodies(env: Dict[str, Any]) -> List[Dict[str, Any]]:
    return [p for p in env.get("positions", []) if p.get("id") in BODY]


def _colour(key: str) -> str:
    return GRAHA[key][2] if key in GRAHA else OUTER.get(key, "#888")


def _lagna(env: Dict[str, Any]) -> Optional[float]:
    h = env.get("houses") or {}
    a = h.get("ascendant") or {}
    return a.get("longitude_deg")


# ── SVG primitives ─────────────────────────────────────────────────────────────────────────────────────────────────
def _pt(r: float, a: float) -> Tuple[float, float]:
    return r * math.sin(math.radians(a)), -r * math.cos(math.radians(a))


def _arc(r0: float, r1: float, a0: float, a1: float) -> str:
    large = 1 if (a1 - a0) % 360 > 180 else 0
    x0, y0 = _pt(r1, a0)
    x1, y1 = _pt(r1, a1)
    x2, y2 = _pt(r0, a1)
    x3, y3 = _pt(r0, a0)
    return (f"M{x0:.2f} {y0:.2f}A{r1} {r1} 0 {large} 1 {x1:.2f} {y1:.2f}L{x2:.2f} {y2:.2f}"
            f"A{r0} {r0} 0 {large} 0 {x3:.2f} {y3:.2f}Z")


def _ink(hex_: str) -> str:
    n = int(hex_[1:], 16)
    c = [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255]
    c = [v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4 for v in c]
    return "#ffffff" if 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2] < 0.42 else "#1b1b1b"


def _lanes(items: List[Tuple[float, Any]], min_gap: float = 7.0, lanes: int = 4) -> List[Tuple[float, Any, int]]:
    """Give each (angle, item) a lane so markers closer than ``min_gap`` degrees do not overlap."""
    out: List[Tuple[float, Any, int]] = []
    last: List[Optional[float]] = [None] * lanes
    for a, it in sorted(items, key=lambda t: t[0]):
        lane = 0
        for k in range(lanes):
            if last[k] is None or (a - last[k]) % 360 >= min_gap:
                lane = k
                break
        else:
            lane = len(out) % lanes
        last[lane] = a
        out.append((a, it, lane))
    return out


def chakra_svg(env: Dict[str, Any], title: str = "Rāśi chakra") -> str:
    """The Panchang Chakra wheel with the chart's grahas and lagna placed on it (sidereal longitudes)."""
    scr = lambda lon: (lon - 30.0) % 360.0  # noqa: E731 - Aries starts at 11 o'clock, clockwise (template)
    parts = ['<svg class="chakra" viewBox="-540 -540 1080 1080" role="img" '
             f'aria-label="{_esc(title)}: rashi, nakshatra and pada rings with planet positions">']
    for i in range(27):  # nakshatra ring
        lord = NAK_LORD[i % 9]
        fill = GRAHA[lord][4]
        a0, a1 = scr(i * NAKS), scr((i + 1) * NAKS)
        mid = scr((i + 0.5) * NAKS)
        parts.append(f'<path class="seg" d="{_arc(470, 525, a0, a0 + NAKS)}" fill="{fill}"><title>{NAKSHATRA[i]} '
                     f'(lord {LORD_NAME[lord]})</title></path>')
        x, y = _pt(497.5, mid)
        rot = mid if mid <= 180 else mid - 180
        parts.append(f'<text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="14" fill="{_ink(fill)}" '
                     f'transform="rotate({rot - 90:.1f} {x:.1f} {y:.1f})">{_esc(NAK_SHORT.get(NAKSHATRA[i], NAKSHATRA[i]))}</text>')
        del a1
    for i in range(108):  # pada ring
        fill = PADA_FILL[ELEMENT[(i % 12) % 4]]
        a0 = scr(i * NAV)
        parts.append(f'<path class="seg" d="{_arc(440, 470, a0, a0 + NAV)}" fill="{fill}"><title>'
                     f'{NAKSHATRA[i // 4]} pada {i % 4 + 1} · navamsha {SIGN_EN[i % 12]}</title></path>')
    for s in range(12):  # rashi band
        fill = GRAHA[SIGN_LORD[s]][3]
        a0 = scr(s * 30.0)
        parts.append(f'<path class="seg" d="{_arc(350, 440, a0, a0 + 30)}" fill="{fill}"><title>{SIGN_SKT[s]} / '
                     f'{SIGN_EN[s]} · lord {LORD_NAME[SIGN_LORD[s]]}</title></path>')
        x, y = _pt(408, scr(s * 30 + 15))
        parts.append(f'<text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="34" fill="#1b1b1b">{GLYPH[s]}&#xFE0E;</text>')
        x, y = _pt(372, scr(s * 30 + 15))
        parts.append(f'<text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="15" font-weight="700" fill="#1b1b1b">'
                     f'{SIGN_SKT[s]}</text>')
    parts.append('<circle r="350" fill="var(--disc)" stroke="var(--line)"/>')
    for s in range(12):
        x0, y0 = _pt(110, scr(s * 30))
        x1, y1 = _pt(350, scr(s * 30))
        parts.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="var(--line)"/>')
    asc = _lagna(env)
    if asc is not None:
        x0, y0 = _pt(110, scr(asc))
        x1, y1 = _pt(530, scr(asc))
        lx, ly = _pt(318, scr(asc))
        parts.append(f'<g data-body="lagna"><line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" '
                     f'stroke="var(--accent)" stroke-width="5"/><circle cx="{lx:.1f}" cy="{ly:.1f}" r="22" '
                     f'fill="var(--accent)"/><text class="lbl" x="{lx:.1f}" y="{ly:.1f}" font-size="16" '
                     f'font-weight="800" fill="#fff">Asc</text><title>Lagna {_dms(asc)} sidereal</title></g>')
    items = [(scr(p["longitude_deg"]), p) for p in _bodies(env)]
    for a, p, lane in _lanes(items):
        abbr, key, name = BODY[p["id"]]
        r = 300 - lane * 52
        x, y = _pt(r, a)
        tx, ty = _pt(348, a)
        fill = _colour(key)
        retro = " ℞" if p.get("retrograde") else ""
        parts.append(
            f'<g class="pl" data-body="{_esc(p["id"])}"><line x1="{x:.1f}" y1="{y:.1f}" x2="{tx:.1f}" y2="{ty:.1f}" '
            f'stroke="#555" stroke-width="1.5" stroke-dasharray="3 3"/><circle cx="{x:.1f}" cy="{y:.1f}" r="23" '
            f'fill="{fill}" stroke="#222" stroke-width="1.5"/><text class="lbl" x="{x:.1f}" y="{y:.1f}" '
            f'font-size="16" font-weight="800" fill="{_ink(fill)}">{abbr}</text>'
            f'<title>{_esc(name)} {_deg_in_sign(p["longitude_deg"])} {SIGN_SKT[int(p["longitude_deg"] // 30) % 12]}'
            f'{retro}</title></g>')
    parts.append(f'<text class="lbl" y="-14" font-size="26" font-weight="800" fill="var(--ink)">{_esc(title)}</text>'
                 '<text class="lbl" y="20" font-size="15" fill="var(--muted)">nakshatra · pada · rāśi</text></svg>')
    return "".join(parts)


def western_wheel_svg(env: Dict[str, Any]) -> str:
    """A classic tropical wheel: Ascendant at 9 o'clock, zodiac counter-clockwise, house cusps, aspect lines."""
    asc = _lagna(env) or 0.0
    ang = lambda lon: (270.0 - (lon - asc)) % 360.0  # noqa: E731
    out = ['<svg class="wheel" viewBox="-430 -430 860 860" role="img" aria-label="Western chart wheel">',
           '<circle r="420" fill="var(--panel)" stroke="var(--ink)" stroke-width="2"/>']
    for s in range(12):
        a0 = ang(s * 30.0 + 30)
        fill = PADA_FILL[ELEMENT[s % 4]]
        out.append(f'<path d="{_arc(350, 420, a0, a0 + 30)}" fill="{fill}" stroke="#fff" stroke-width="1.5">'
                   f'<title>{SIGN_EN[s]}</title></path>')
        x, y = _pt(385, ang(s * 30 + 15))
        out.append(f'<text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="30" fill="#1b1b1b">{GLYPH[s]}&#xFE0E;</text>')
    cusps = (env.get("houses") or {}).get("cusps_deg") or []
    for k, c in enumerate(cusps):
        x0, y0 = _pt(120, ang(c))
        x1, y1 = _pt(350, ang(c))
        w = 3 if k in (0, 3, 6, 9) else 1
        out.append(f'<line x1="{x0:.1f}" y1="{y0:.1f}" x2="{x1:.1f}" y2="{y1:.1f}" stroke="var(--ink)" '
                   f'stroke-width="{w}"/>')
        nxt = cusps[(k + 1) % 12]
        mid = c + ((nxt - c) % 360) / 2
        x, y = _pt(135, ang(mid))
        out.append(f'<text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="15" fill="var(--muted)">{k + 1}</text>')
    out.append('<circle r="120" fill="none" stroke="var(--line)"/>')
    pos = {p["id"]: p["longitude_deg"] for p in _bodies(env)}
    colours = {"conjunction": "#777", "opposition": "#C8102E", "square": "#C8102E", "trine": "#2563EB",
               "sextile": "#2563EB"}
    for a in env.get("derived_attributes", []):
        if ".aspect." in a["id"]:
            b1, b2 = a.get("derived_from", [None, None])[:2]
            if b1 in pos and b2 in pos:
                x0, y0 = _pt(118, ang(pos[b1]))
                x1, y1 = _pt(118, ang(pos[b2]))
                v = a["value"]
                out.append(f'<line class="asp" data-body="{_esc(b1)} {_esc(b2)}" x1="{x0:.1f}" y1="{y0:.1f}" '
                           f'x2="{x1:.1f}" y2="{y1:.1f}" stroke="{colours.get(v["aspect"], "#999")}" '
                           f'stroke-width="1.6" opacity=".75"><title>{_esc(b1)} {v["aspect"]} {_esc(b2)} · orb '
                           f'{v["orb_deg"]:.2f}°</title></line>')
    for a, p, lane in _lanes([(ang(p["longitude_deg"]), p) for p in _bodies(env)], min_gap=8):
        abbr, key, name = BODY[p["id"]]
        r = 318 - lane * 44
        x, y = _pt(r, a)
        fill = _colour(key)
        out.append(f'<g class="pl" data-body="{_esc(p["id"])}"><circle cx="{x:.1f}" cy="{y:.1f}" r="19" '
                   f'fill="{fill}" stroke="#222"/><text class="lbl" x="{x:.1f}" y="{y:.1f}" font-size="14" '
                   f'font-weight="800" fill="{_ink(fill)}">{abbr}</text><title>{_esc(name)} '
                   f'{_deg_in_sign(p["longitude_deg"])} {SIGN_EN[int(p["longitude_deg"] // 30) % 12]}</title></g>')
    if _lagna(env) is not None:
        x, y = _pt(440, 270)
        out.append(f'<text class="lbl" x="{x + 30:.1f}" y="{y - 14:.1f}" font-size="15" font-weight="800" '
                   f'fill="var(--accent)">ASC</text>')
    out.append("</svg>")
    return "".join(out)


SOUTH_CELL = {11: (0, 0), 0: (1, 0), 1: (2, 0), 2: (3, 0), 3: (3, 1), 4: (3, 2), 5: (3, 3), 6: (2, 3), 7: (1, 3),
              8: (0, 3), 9: (0, 2), 10: (0, 1)}


def south_indian_svg(placements: Dict[int, List[Tuple[str, str, str]]], lagna_sign: Optional[int], title: str) -> str:
    """South Indian chart: signs fixed, Pisces top-left. ``placements`` maps sign -> [(body id, abbr, note)]."""
    W, H = 120, 96
    out = [f'<svg class="sic" viewBox="-2 -2 {4 * W + 4} {4 * H + 4}" role="img" aria-label="{_esc(title)}">']
    for s, (c, r) in SOUTH_CELL.items():
        x, y = c * W, r * H
        out.append(f'<rect x="{x}" y="{y}" width="{W}" height="{H}" fill="var(--panel)" stroke="var(--ink)" '
                   f'stroke-width="1.4"><title>{SIGN_SKT[s]} / {SIGN_EN[s]}</title></rect>')
        out.append(f'<text x="{x + W - 6}" y="{y + 15}" text-anchor="end" font-size="11" fill="var(--muted)">'
                   f'{SIGN_SKT[s]}</text>')
        if lagna_sign == s:
            out.append(f'<line x1="{x}" y1="{y + 22}" x2="{x + 22}" y2="{y}" stroke="var(--accent)" stroke-width="2"/>'
                       f'<text x="{x + 4}" y="{y + 15}" font-size="11" font-weight="800" fill="var(--accent)">As</text>')
        for k, (bid, abbr, note) in enumerate(placements.get(s, [])):
            col, row = k % 2, k // 2
            tx, ty = x + 8 + col * 56, y + 34 + row * 19
            out.append(f'<text data-body="{_esc(bid)}" x="{tx}" y="{ty}" font-size="13" font-weight="700" '
                       f'fill="var(--ink)">{_esc(abbr)}<tspan font-size="10" font-weight="400" fill="var(--muted)"> '
                       f'{_esc(note)}</tspan></text>')
    out.append(f'<rect x="{W}" y="{H}" width="{2 * W}" height="{2 * H}" fill="var(--chip)" stroke="var(--ink)" '
               f'stroke-width="1.4"/><text x="{2 * W}" y="{2 * H}" text-anchor="middle" font-size="17" '
               f'font-weight="800" fill="var(--ink)">{_esc(title)}</text></svg>')
    return "".join(out)


def north_indian_svg(placements: Dict[int, List[Tuple[str, str, str]]], lagna_sign: int, title: str) -> str:
    """North Indian chart: houses fixed, house 1 at the top diamond, signs numbered 1-12 in each house."""
    S = 400.0
    A, B, C, D = (0, 0), (S, 0), (S, S), (0, S)
    T, R, Bm, L, O = (S / 2, 0), (S, S / 2), (S / 2, S), (0, S / 2), (S / 2, S / 2)
    P1, P2, P3, P4 = (S / 4, S / 4), (3 * S / 4, S / 4), (3 * S / 4, 3 * S / 4), (S / 4, 3 * S / 4)
    houses = [[T, P2, O, P1], [A, T, P1], [A, P1, L], [L, P1, O, P4], [L, P4, D], [D, P4, Bm], [Bm, P4, O, P3],
              [Bm, P3, C], [C, P3, R], [R, P3, O, P2], [R, P2, B], [B, P2, T]]
    out = [f'<svg class="nic" viewBox="-2 -2 {S + 4} {S + 4}" role="img" aria-label="{_esc(title)}">',
           f'<rect width="{S}" height="{S}" fill="var(--panel)" stroke="var(--ink)" stroke-width="1.6"/>']
    for h, poly in enumerate(houses):
        sign = (lagna_sign + h) % 12
        pts = " ".join(f"{x},{y}" for x, y in poly)
        cx = sum(p[0] for p in poly) / len(poly)
        cy = sum(p[1] for p in poly) / len(poly)
        fill = "var(--chip)" if h == 0 else "none"
        out.append(f'<polygon points="{pts}" fill="{fill}" stroke="var(--ink)" stroke-width="1.4">'
                   f'<title>House {h + 1}: {SIGN_SKT[sign]}</title></polygon>')
        big = len(poly) == 4
        out.append(f'<text x="{cx:.1f}" y="{cy - (30 if big else 14):.1f}" text-anchor="middle" font-size="12" '
                   f'fill="var(--accent)" font-weight="700">{sign + 1}</text>')
        items = placements.get(sign, [])
        for k, (bid, abbr, note) in enumerate(items):
            per = 2 if big else 1
            col, row = k % per, k // per
            tx = cx + (col - (per - 1) / 2) * 44
            ty = cy - (8 if big else 0) + row * 15
            out.append(f'<text data-body="{_esc(bid)}" x="{tx:.1f}" y="{ty:.1f}" text-anchor="middle" '
                       f'font-size="12" font-weight="700" fill="var(--ink)">{_esc(abbr)}<tspan font-size="9" '
                       f'fill="var(--muted)"> {_esc(note)}</tspan></text>')
    out.append(f'<text x="{S / 2}" y="{S - 6}" text-anchor="middle" font-size="11" fill="var(--muted)">'
               f'{_esc(title)}</text></svg>')
    return "".join(out)


def bodygraph_svg(hd: Dict[str, Any]) -> str:
    """A schematic BodyGraph: nine centres, defined ones filled, defined channels drawn between them."""
    bg = _attrs(hd).get("human_design.bodygraph", {}).get("value", {})
    defined = set(bg.get("defined_centers", []))
    pos = {"head": (150, 35), "ajna": (150, 100), "throat": (150, 175), "g": (150, 250), "heart": (212, 278),
           "sacral": (150, 340), "spleen": (55, 320), "solar_plexus": (245, 320), "root": (150, 410)}
    colour = {"head": "#F5D547", "ajna": "#7FB77E", "throat": "#B88A5A", "g": "#F5D547", "heart": "#D9534F",
              "sacral": "#D9534F", "spleen": "#B88A5A", "solar_plexus": "#B88A5A", "root": "#B88A5A"}
    out = ['<svg class="bodygraph" viewBox="0 0 300 450" role="img" aria-label="Human Design BodyGraph">']
    for ch in bg.get("channels", []):
        a, b = (int(x) for x in ch.split("-"))
        ca, cb = GATE_CENTER.get(a), GATE_CENTER.get(b)
        if ca and cb:
            (x0, y0), (x1, y1) = pos[ca], pos[cb]
            out.append(f'<line x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}" stroke="var(--ink)" stroke-width="6" '
                       f'stroke-linecap="round"><title>Channel {ch}</title></line>')
    shapes = {"head": "tri-up", "ajna": "tri-down", "throat": "sq", "g": "dia", "heart": "tri-up", "sacral": "sq",
              "spleen": "tri-right", "solar_plexus": "tri-left", "root": "sq"}
    for c, (x, y) in pos.items():
        r = 26 if c != "heart" else 18
        sh = shapes[c]
        if sh == "sq":
            d = f"M{x - r} {y - r}h{2 * r}v{2 * r}h{-2 * r}Z"
        elif sh == "dia":
            d = f"M{x} {y - r - 4}L{x + r + 4} {y}L{x} {y + r + 4}L{x - r - 4} {y}Z"
        elif sh == "tri-up":
            d = f"M{x} {y - r}L{x + r} {y + r}L{x - r} {y + r}Z"
        elif sh == "tri-down":
            d = f"M{x - r} {y - r}L{x + r} {y - r}L{x} {y + r}Z"
        elif sh == "tri-right":
            d = f"M{x - r} {y - r}L{x + r} {y}L{x - r} {y + r}Z"
        else:
            d = f"M{x + r} {y - r}L{x - r} {y}L{x + r} {y + r}Z"
        fill = colour[c] if c in defined else "var(--panel)"
        out.append(f'<path d="{d}" fill="{fill}" stroke="var(--ink)" stroke-width="2"><title>{c.replace("_", " ")}: '
                   f'{"defined" if c in defined else "undefined"}</title></path>')
        out.append(f'<text class="lbl" x="{x}" y="{y + (8 if sh == "tri-up" else 0)}" font-size="9" '
                   f'fill="{"#1b1b1b" if c in defined else "var(--muted)"}">{c.replace("_", " ")}</text>')
    out.append("</svg>")
    return "".join(out)


# ── HTML sections ───────────────────────────────────────────────────────────────────────────────────────────────────
# ── the chart studio page ───────────────────────────────────────────────────────────────────────────────────────
SECTION_TITLES = {
    "overview": "Overview",
    "chakra": "Jyotish Chakra (Graha-Nakshatra Mandala)",
    "charts": "Charts (South and North Indian)",
    "vargas": "Divisional charts (vargas)",
    "planets": "Grahas",
    "planet": "Graha",
    "houses": "Bhāvas",
    "house": "Bhāva",
    "aspects": "Aspects",
    "strength": "Strength (ṣaḍbala, bhāva bala, viṃśopaka)",
    "ashtakavarga": "Aṣṭakavarga",
    "dasha": "Daśā periods (Vimśottarī daśā, Yoginī daśā, Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla, Kālachakra; Sāḍe Sātī)",
    "panchanga": "Pañcāṅga",
    "special": "Lagnas and special points",
    "point": "Lagna or special point",
    "maitri": "Maitri and dignities",
    "yogas": "Yogas and doshas",
    "yoga": "Yoga",
    "kp": "KP (Krishnamurti Paddhati)",
    "nadi": "Nadi direction matrix",
    "aux": "Auxiliary analysis (sahams, avakahaḍā, mṛtyu bhāga, ghāta, vedhasthāna, jīva/deha, śodhana)",
    "chakras": "Chakras (Sarvatobhadra, Kota, Saṅghaṭṭa)",
    "annual": "Varṣaphala (annual chart)",
    "pakshi": "Pañca pakṣī",
    "energetics": "AstroEnergetics",
    "transits": "Transits",
    "progressions": "Progressions",
    "western": "Western chart (tropical)",
    "human-design": "Human Design",
    "numerology": "Numerology",
    "matching": "Compatibility (Ashtakoota)",
    "data": "Provenance and data",
}
SECTIONS = tuple(SECTION_TITLES)
ASSETS = ("studio-core.js", "studio-charts.js", "studio-pages.js", "studio-dashas.js", "studio-maitri.js", "studio-analysis.js",
          "studio-chakras.js", "studio-annual.js", "studio-boot.js")


def _asset(name: str) -> str:
    return resources.files("eshana").joinpath(f"assets/{name}").read_text(encoding="utf-8")


def _reading_reference() -> Optional[Dict[str, Any]]:
    """Reading notes for yogas, doshas and special points (interpretation, attributed); see
    scripts/build_reading_reference.py."""
    try:
        return json.loads(resources.files("eshana").joinpath("data/reading_reference.json").read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return None


def _energetics() -> Optional[Dict[str, Any]]:
    try:
        return json.loads(resources.files("eshana").joinpath("data/astro_energetics.json").read_text(encoding="utf-8"))
    except Exception:  # pragma: no cover - data file is part of the wheel
        return None


def _kind(env: Dict[str, Any]) -> Optional[str]:
    kind = env.get("kind")
    system = (env.get("calculation") or {}).get("system")
    if kind == "computed_chart" and system in ("vedic", "western"):
        return system
    if kind in ("human_design", "matching"):
        return kind
    if env.get("derived_attributes") and str(env["derived_attributes"][0].get("id", "")).startswith("numerology"):
        return "numerology"
    return None


# The ESHANA mark: a three-ring gimbal around a bindu. Each ring turns about its own axis (vertical, horizontal,
# diagonal) while the whole gimbal precesses; styles in studio.css (.gimbal).
BRAND = ('<svg class="sigil gimbal" viewBox="0 0 24 24" aria-hidden="true"><g class="gb-precess">'
         '<g class="gb gb1"><circle cx="12" cy="12" r="10.6" class="ring"/><circle cx="22.6" cy="12" r="1.3" class="sat"/></g>'
         '<g class="gb gb2"><circle cx="12" cy="12" r="7.9" class="ring"/><circle cx="12" cy="4.1" r="1.1" class="sat"/></g>'
         '<g class="gb gb3"><circle cx="12" cy="12" r="5.2" class="ring"/><circle cx="17.2" cy="12" r="0.9" class="sat"/></g>'
         '</g><circle class="core" cx="12" cy="12" r="2.4"/></svg>')


def render_html(envelopes: Dict[str, Dict[str, Any]], title: str = "ESHANA chart report",
                subject: Optional[Dict[str, Any]] = None, sections: Optional[Iterable[str]] = None,
                theme: str = "auto", transits: Optional[Dict[str, Any]] = None,
                progressions: Optional[Dict[str, Any]] = None, reference_utc: Optional[str] = None,
                errors: Optional[Dict[str, str]] = None, defaults: Optional[Dict[str, Any]] = None,
                revolutions: Optional[Dict[str, Any]] = None) -> str:
    """Render engine envelopes as ESHANA Studio: one self-contained, multi-page, animated HTML app.

    ``envelopes`` maps labels to envelopes; their kinds are recognised from the envelopes themselves. ``transits`` and
    ``progressions`` are ``ephemeris`` envelopes. ``sections`` limits the app to pages named in :data:`SECTIONS`;
    ``defaults`` presets viewer settings (icons, chart style, chakra centre, layers). Everything renders in the browser
    from the embedded data; nothing is fetched.
    """
    want = list(sections or SECTIONS)
    unknown = set(want) - set(SECTIONS)
    if unknown:
        raise ValueError(f"unknown sections: {', '.join(sorted(unknown))}; choose from {', '.join(SECTIONS)}")
    if "planets" in want and "planet" not in want:
        want.append("planet")
    if "houses" in want and "house" not in want:
        want.append("house")
    if "yogas" in want and "yoga" not in want:
        want.append("yoga")
    if "special" in want and "point" not in want:
        want.append("point")
    by_kind: Dict[str, Dict[str, Any]] = {}
    for env in envelopes.values():
        k = _kind(env)
        if k and k not in by_kind:
            by_kind[k] = env
    data = {
        "envelopes": by_kind, "subject": subject or {}, "title": title, "theme": theme if theme in ("light", "dark") else None,
        "reference_utc": reference_utc, "transits": transits, "progressions": progressions, "revolutions": revolutions,
        "nakshatra_reference": _nak_reference(), "energetics": _energetics(), "reading": _reading_reference(), "errors": errors or None,
        "defaults": defaults or {}, "pages": want, "generator": "eshana.report",
    }
    first = next(iter(by_kind.values()), {})
    version = (first.get("provenance") or {}).get("engine_version", "")
    data_json = json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("</", "<\\/")
    shells = "".join(
        f'<section class="page" id="{sid}" hidden aria-labelledby="h-{sid}"><h2 class="sr" id="h-{sid}">{_esc(SECTION_TITLES[sid])}</h2>'
        f'<div class="body"><noscript><p class="note">ESHANA Studio draws this page in the browser; enable JavaScript. '
        f'The data is embedded in the file.</p></noscript></div></section>'
        for sid in SECTIONS if sid in want)
    err_html = ""
    if errors:
        items = "".join(f"<li><b>{_esc(k)}</b>: {_esc(v)}</li>" for k, v in errors.items())
        err_html = (f'<div class="card" id="not-computed"><h3>Not computed</h3><p class="note">These were refused by the '
                    f'engine, so they are left out rather than guessed.</p><ul class="warn">{items}</ul></div>')
    subj = subject or {}
    who = subj.get("name") and (f'{subj["name"]} ({subj["gender"]})' if subj.get("gender") else subj["name"])
    sub = " · ".join(str(x) for x in (who, subj.get("local"), subj.get("place_label")) if x)
    generated = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    js = "\n".join(_asset(n) for n in ASSETS)
    js = js.replace("const sigil = (name, cls = '') => `<svg class=\"sigil ${cls}\" viewBox=\"0 0 24 24\" aria-hidden=\"true\">${SIGIL[name] || SIGIL.overview}</svg>`;",
                    "const sigil = (name, cls = '') => `<svg class=\"sigil ${cls}\" viewBox=\"0 0 24 24\" aria-hidden=\"true\">${SIGIL[name] || SIGIL[String(name).replace('-', '_')] || SIGIL.overview}</svg>`;")
    theme_attr = f' data-theme="{theme}" data-mode="{theme}"' if theme in ("light", "dark") else ' data-mode="dark"'
    return f"""<!doctype html>
<html lang="en"{theme_attr}>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="generator" content="ESHANA {_esc(version)}">
<title>{_esc(title)}</title>
<style>{_asset("studio.css")}
.sr{{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}}</style>
</head>
<body>
<canvas id="sky" aria-hidden="true"></canvas>
<div class="app">
<header class="topbar">
<button class="btn" id="menu" type="button" aria-label="Menu">☰</button>
<a class="brand" href="#/overview">{BRAND}ESHANA</a>
<span class="who">{_esc(title)}{(' · ' + _esc(sub)) if sub else ''}</span>
<div class="tools">
<button class="btn" id="search" type="button" title="Jump to a page, graha or bhāva">⌕ <span class="t">Jump</span> <kbd>⌘K</kbd></button>
<button class="btn" id="settings" type="button">⚙ <span class="t">Show and style</span></button>
<button class="btn" id="theme" type="button">◐ <span class="t">Theme</span></button>
<button class="btn" id="print" type="button">⎙ <span class="t">Print</span></button>
<button class="btn" id="json" type="button">⤓ <span class="t">JSON</span></button>
<button class="btn" id="copy" type="button">⧉ <span class="t">Copy summary</span></button>
</div>
</header>
<nav class="side" aria-label="Pages"></nav>
<main>
{err_html}{shells}
</main>
<footer>Generated {generated} by ESHANA {_esc(version)} (Extensible Spatiotemporal Hypersystem for Astroinformatics,
Natural Cycles &amp; Associations). Open source, Apache-2.0. Every value comes from the engine and is labelled calculated
or tradition_rule; chart markers follow {MARKER_RULES}, Nadi links report.nadi@0.1. Self-contained; no network requests.</footer>
</div>
<div id="palette" hidden role="dialog" aria-label="Jump to"><div class="box"><input placeholder="Jump to a page, graha or bhāva…" aria-label="Search"><ul></ul></div></div>
<aside class="drawer" id="drawer" aria-label="Show and style"></aside>
<script type="application/json" id="eshana-data">{data_json}</script>
<script>{js}</script>
</body>
</html>
"""


# ── running the engine ─────────────────────────────────────────────────────────────────────────────────────────────
ALL_DIVISIONS = (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60)


def envelopes_for(ctx: Any, *, name: Optional[str] = None, ayanamsha: str = "lahiri", houses: str = "whole_sign",
                  western_houses: str = "placidus", node_model: str = "mean", divisions: Sequence[int] = ALL_DIVISIONS,
                  dasha: bool = True, dasha_levels: int = 3, dasha_systems: Sequence[str] = ("vimshottari", "yogini"),
                  features: Sequence[str] = ("all",), reference_utc: Optional[str] = None,
                  systems: Sequence[str] = ("vedic", "western", "human_design", "numerology"),
                  provider: str = "analytic", spk_path: Optional[str] = None, numerology_school: str = "pythagorean",
                  sunrise: Optional[str] = None, partner: Any = None, gender: Optional[str] = None) -> Dict[str, Dict[str, Any]]:
    """Run the engine for each requested system. A system that cannot be computed becomes an ``_errors`` entry."""
    from . import ChartEngine, EshanaError

    eng = ChartEngine(provider=provider, spk_path=spk_path)
    out: Dict[str, Dict[str, Any]] = {}
    errors: Dict[str, str] = {}

    def run(label: str, fn: Any) -> None:
        try:
            r = fn()
            out[label] = r.raw if hasattr(r, "raw") else r
        except EshanaError as e:
            errors[label] = f"{e.code}: {e.message}"

    if "vedic" in systems:
        run("vedic", lambda: eng.vedic_chart(ctx, ayanamsha=ayanamsha, houses=houses, node_model=node_model,
                                             divisions=list(divisions), dasha=dasha, dasha_levels=dasha_levels,
                                             dasha_systems=list(dasha_systems), features=list(features),
                                             reference_utc=reference_utc, sunrise=sunrise))
        if "vedic" in errors and features:
            # A feature may be refused for this input (for example no place); keep the chart without features.
            errors.pop("vedic")
            run("vedic", lambda: eng.vedic_chart(ctx, ayanamsha=ayanamsha, houses=houses, node_model=node_model,
                                                 divisions=list(divisions), dasha=dasha, dasha_levels=dasha_levels,
                                                 dasha_systems=list(dasha_systems), reference_utc=reference_utc))
    if "western" in systems:
        run("western", lambda: eng.western_chart(ctx, houses=western_houses, node_model=node_model, aspects=True))
    if "human_design" in systems:
        run("human_design", lambda: eng.human_design(ctx))
    if "numerology" in systems and (name or ctx.when.get("date")):
        run("numerology", lambda: eng.numerology(name=name, date_of_birth=ctx.when.get("date"), school=numerology_school,
                                                 extended=True, reference_date=(reference_utc or "")[:10] or None,
                                                 gender=gender))
    if partner is not None:
        run("matching", lambda: eng.matching(ctx, partner, ayanamsha=ayanamsha))
    if errors:
        out["_errors"] = {"kind": "report_errors", "errors": errors}
    return out


def _series(eng: Any, start: str, step: float, count: int, zodiac: str, ayanamsha: Optional[str]) -> Optional[Dict[str, Any]]:
    from . import EshanaError
    try:
        return eng.ephemeris(start, step, count, zodiac=zodiac, ayanamsha=ayanamsha)
    except EshanaError:
        return None


def _instant(x: str) -> _dt.datetime:
    """A date (YYYY-MM-DD) or a UTC instant as a naive UTC datetime."""
    x = x.strip().rstrip("Z")
    return _dt.datetime.strptime(x[:19], "%Y-%m-%dT%H:%M:%S") if "T" in x else _dt.datetime.strptime(x[:10], "%Y-%m-%d")


def _transit_series(eng: Any, ref: str, ayanamsha: Optional[str], days: Optional[int], start: Optional[str],
                    end: Optional[str], step: float) -> Optional[Dict[str, Any]]:
    ref_dt = _instant(ref)
    if start or end:
        s0 = _instant(start) if start else _instant(end) - _dt.timedelta(days=365)
        s1 = _instant(end) if end else s0 + _dt.timedelta(days=365)
    elif days:
        s0, s1 = ref_dt - _dt.timedelta(days=days / 2), ref_dt + _dt.timedelta(days=days / 2)
    else:
        s0, s1 = ref_dt - _dt.timedelta(days=365), ref_dt + _dt.timedelta(days=730)
    if s1 <= s0:
        raise ValueError("the transit range must end after it starts")
    span = (s1 - s0).total_seconds() / 86400
    step = max(float(step or 1.0), span / 6000)
    out = _series(eng, s0.strftime("%Y-%m-%dT%H:%M:%SZ"), step, int(span / step) + 1, "sidereal", ayanamsha)
    if out:
        k = round((ref_dt - s0).total_seconds() / 86400 / step)
        out["reference_index"] = k if 0 <= k < len(out.get("rows", [])) else 0
        out["range"] = {"start": s0.strftime("%Y-%m-%d"), "end": s1.strftime("%Y-%m-%d"), "step_days": step}
    return out


def _revolutions(eng: Any, birth_utc: str, ayanamsha: Optional[str], years: int, step: float = 5.0) -> Optional[Dict[str, Any]]:
    """Return dates of every completed sidereal revolution of each body after birth (report.revolutions@0.1).

    Each body's motion is unwrapped from a sidereal ephemeris every ``step`` days; a revolution completes the first
    time the travel since birth passes another 360 degrees in the body's mean direction (backwards for the nodes), so
    retrograde loops are not counted twice. Dates are interpolated within the step.
    """
    t0 = _instant(birth_utc)
    count = int(years * 365.25 / step) + 1
    ser = _series(eng, t0.strftime("%Y-%m-%dT%H:%M:%SZ"), step, count, "sidereal", ayanamsha)
    if not ser or not ser.get("rows"):
        return None
    rows = ser["rows"]
    bodies: Dict[str, Any] = {}
    for key in rows[0]["longitudes"]:
        natal = rows[0]["longitudes"][key]
        cum, prev, done, out = 0.0, natal, 0, []
        direction = -1 if "node" in key else 1
        for i in range(1, len(rows)):
            lon = rows[i]["longitudes"].get(key)
            if lon is None:
                continue
            d = (lon - prev + 540.0) % 360.0 - 180.0
            before, cum, prev = cum, cum + d, lon
            while direction * cum >= 360.0 * (done + 1):
                target = direction * 360.0 * (done + 1)
                frac = (target - before) / (cum - before) if cum != before else 1.0
                when = t0 + _dt.timedelta(days=step * (i - 1 + max(0.0, min(1.0, frac))))
                out.append(when.strftime("%Y-%m-%dT%H:%MZ"))
                done += 1
        bodies[key] = {"natal": natal, "direction": direction, "returns": out}
    return {"rule": "report.revolutions@0.1", "birth_utc": birth_utc, "years": years, "step_days": step,
            "convention": "each completed sidereal revolution after birth advances the body one sign in its direction "
                          "of motion (backwards for the nodes); the degree within the sign is kept",
            "bodies": bodies}


def birth_report(ctx: Any, *, name: Optional[str] = None, gender: Optional[str] = None, place_label: Optional[str] = None,
                 title: Optional[str] = None, sections: Optional[Iterable[str]] = None, theme: str = "auto",
                 reference_utc: Optional[str] = None, transit_days: Optional[int] = None,
                 transit_start: Optional[str] = None, transit_end: Optional[str] = None,
                 transit_step_days: float = 1.0, progression_years: int = 90,
                 defaults: Optional[Dict[str, Any]] = None, **kw: Any) -> str:
    """Compute every requested system for ``ctx`` (an :class:`eshana.Context`) and return the chart studio HTML.

    ``reference_utc`` is the instant used for "current" daśā periods and the transit table (default: now, recorded
    in the file). The transit series embedded for the chakra's date picker and animation runs from ``transit_start``
    to ``transit_end`` (dates or UTC instants) every ``transit_step_days``. Without them it runs from a year before
    the reference to two years after; ``transit_days`` instead centres a span of that many days on the reference, and
    ``transit_days=0`` leaves transits out. The step grows if the range would exceed 6,000 rows.
    ``progression_years`` sets the span of secondary progressions. Other keyword arguments go to :func:`envelopes_for`.
    """
    from . import ChartEngine

    ref = reference_utc or _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    envs = envelopes_for(ctx, name=name, gender=gender, reference_utc=ref, **kw)
    errs = envs.pop("_errors", None)
    eng = ChartEngine(provider=kw.get("provider", "analytic"), spk_path=kw.get("spk_path"))
    transits = progressions = revolutions = None
    ayan = kw.get("ayanamsha", "lahiri")
    if transit_days != 0 and "vedic" in envs:
        transits = _transit_series(eng, ref, ayan, transit_days, transit_start, transit_end, transit_step_days)
    birth_utc = next((e.get("input", {}).get("instant_utc") for e in envs.values() if e.get("input")), None)
    if progression_years > 0 and birth_utc and ctx.when.get("time"):
        progressions = _series(eng, birth_utc[:19] + "Z", 1.0, progression_years + 1, "tropical", None)
        if "vedic" in envs:
            revolutions = _revolutions(eng, birth_utc[:19] + "Z", ayan, max(progression_years, 1))
    local = " ".join(str(ctx.when.get(k)) for k in ("date", "time") if ctx.when.get(k)) or ctx.when.get("utc", "")
    if ctx.when.get("timezone"):
        local += f' ({ctx.when["timezone"]})'
    return render_html(envs, title=title or (f"{name} · birth chart" if name else "Birth chart"),
                       subject={"name": name, "gender": gender, "place_label": place_label, "local": local}, sections=sections,
                       theme=theme, transits=transits, progressions=progressions, reference_utc=ref,
                       errors=(errs or {}).get("errors"), defaults=defaults, revolutions=revolutions)


def _parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="eshana-report", description="Write a self-contained, interactive HTML chart studio.")
    p.add_argument("-o", "--output", default="-", help="output .html file (default: stdout)")
    p.add_argument("--from-json", nargs="+", metavar="FILE", help="render envelopes saved from the eshana CLI instead")
    p.add_argument("--date", help="local date YYYY-MM-DD")
    p.add_argument("--time", help="local time HH:MM[:SS]; omit for an unknown birth time")
    p.add_argument("--tz", help="IANA time zone, e.g. Asia/Kolkata")
    p.add_argument("--offset-minutes", type=int, help="fixed UTC offset in minutes east, when no zone is known")
    p.add_argument("--utc", help="UTC instant instead of a local time, e.g. 2000-01-01T12:00:00Z")
    p.add_argument("--fold", choices=["reject", "earliest", "latest"])
    p.add_argument("--uncertainty-minutes", type=float)
    p.add_argument("--lat", type=float)
    p.add_argument("--lon", type=float)
    p.add_argument("--name", help="name (display and numerology)")
    p.add_argument("--gender", choices=["male", "female"], help="gender (shown with the name; picks the Kua number)")
    p.add_argument("--place", dest="place_label", help="place label shown in the report")
    p.add_argument("--title")
    p.add_argument("--ayanamsha", default="lahiri", choices=["lahiri", "raman", "krishnamurti", "fagan_bradley"])
    p.add_argument("--houses", default="whole_sign", choices=["whole_sign", "equal", "placidus"], help="Vedic houses")
    p.add_argument("--western-houses", default="placidus", choices=["whole_sign", "equal", "placidus"])
    p.add_argument("--node", default="mean", choices=["mean", "true"])
    p.add_argument("--divisions", default=",".join(str(d) for d in ALL_DIVISIONS))
    p.add_argument("--features", default="all", help="Vedic features (comma list) or all, or none")
    p.add_argument("--dasha-levels", type=int, default=3)
    p.add_argument("--dasha-systems", default="vimshottari,yogini")
    p.add_argument("--no-dasha", action="store_true")
    p.add_argument("--reference", help="instant for 'current' periods and transits, UTC (default: now)")
    p.add_argument("--transit-days", type=int, default=None,
                   help="centre a transit span of N days on the reference (0 = no transits)")
    p.add_argument("--transit-start", help="first date of the transit series, YYYY-MM-DD (default: a year before the reference)")
    p.add_argument("--transit-end", help="last date of the transit series, YYYY-MM-DD (default: two years after the reference)")
    p.add_argument("--transit-step-days", type=float, default=1.0, help="days between transit frames (default 1)")
    p.add_argument("--progression-years", type=int, default=90, help="span of secondary progressions (0 = none)")
    p.add_argument("--sunrise", choices=["upper_limb_refracted", "centre_refracted", "centre_unrefracted"])
    p.add_argument("--systems", default="vedic,western,human_design,numerology")
    p.add_argument("--school", default="pythagorean", choices=["pythagorean", "chaldean"])
    p.add_argument("--partner", help="second person for matching: date,time,tz,lat,lon")
    p.add_argument("--provider", default="analytic", choices=["analytic", "spk"])
    p.add_argument("--spk", help="JPL SPK kernel for --provider spk")
    p.add_argument("--sections", help=f"comma list from: {','.join(SECTIONS)}")
    p.add_argument("--icons", choices=["sphere", "glyph", "badge"], help="default planet icon style")
    p.add_argument("--centre", choices=["nadi", "south", "north", "lanes"], help="default centre of the Jyotish Chakra (Graha-Nakshatra Mandala)")
    p.add_argument("--chart-style", choices=["south", "north", "both"], help="default chart style")
    p.add_argument("--theme", default="auto", choices=["auto", "light", "dark"])
    return p


def main(argv: Optional[Sequence[str]] = None) -> int:
    from . import Context, EshanaError

    a = _parser().parse_args(argv)
    sections = a.sections.split(",") if a.sections else None
    defaults = {k: v for k, v in {"icons": a.icons, "chart": a.chart_style, "centre": a.centre}.items() if v}
    try:
        if a.from_json:
            envs = {}
            for f in a.from_json:
                with open(f, encoding="utf-8") as fh:
                    env = json.load(fh)
                if env.get("kind") == "error":
                    print(f"eshana-report: {f} is an error envelope: {env['error']['code']}", file=sys.stderr)
                    return 2
                envs[f.rsplit("/", 1)[-1].rsplit(".", 1)[0]] = env
            page = render_html(envs, title=a.title or "ESHANA chart report",
                               subject={"name": a.name, "place_label": a.place_label}, sections=sections,
                               theme=a.theme, reference_utc=a.reference, defaults=defaults)
        else:
            if a.utc:
                ctx = Context.from_utc(a.utc, a.lat, a.lon)
            elif a.date:
                ctx = Context.from_local_birth(a.date, a.time, a.tz, a.lat, a.lon, a.offset_minutes, a.fold,
                                               a.uncertainty_minutes)
            else:
                print("eshana-report: give --date (with --tz or --offset-minutes) or --utc, or --from-json",
                      file=sys.stderr)
                return 2
            partner = None
            if a.partner:
                f = [x.strip() for x in a.partner.split(",")]
                partner = Context.from_local_birth(f[0], f[1] or None, f[2] or None,
                                                   float(f[3]) if len(f) > 3 and f[3] else None,
                                                   float(f[4]) if len(f) > 4 and f[4] else None)
            feats = [] if a.features == "none" else [x for x in a.features.split(",") if x]
            page = birth_report(ctx, name=a.name, gender=a.gender, place_label=a.place_label, title=a.title, sections=sections,
                                theme=a.theme, reference_utc=a.reference, transit_days=a.transit_days,
                                transit_start=a.transit_start, transit_end=a.transit_end,
                                transit_step_days=a.transit_step_days,
                                progression_years=a.progression_years, defaults=defaults,
                                ayanamsha=a.ayanamsha, houses=a.houses, western_houses=a.western_houses,
                                node_model=a.node, divisions=[int(x) for x in a.divisions.split(",") if x],
                                dasha=not a.no_dasha, dasha_levels=a.dasha_levels,
                                dasha_systems=[x for x in a.dasha_systems.split(",") if x], features=feats,
                                systems=a.systems.split(","), provider=a.provider, spk_path=a.spk,
                                numerology_school=a.school, sunrise=a.sunrise, partner=partner)
    except (EshanaError, ValueError) as e:
        print(f"eshana-report: {e}", file=sys.stderr)
        return 2
    if a.output == "-":
        sys.stdout.write(page)
    else:
        with open(a.output, "w", encoding="utf-8") as fh:
            fh.write(page)
        print(f"wrote {a.output} ({len(page):,} bytes)", file=sys.stderr)
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
