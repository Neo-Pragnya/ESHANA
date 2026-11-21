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

__all__ = ["birth_report", "render_html", "envelopes_for", "main", "SECTIONS"]

MARKER_RULES = "report.markers@0.1"
SECTIONS = ("summary", "chakra", "vedic", "divisional", "dasha", "nakshatra", "western", "human_design",
            "numerology", "attributes", "provenance", "raw")

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
def _table(head: Sequence[str], rows: Iterable[Sequence[Any]], cls: str = "", row_attrs: Optional[List[str]] = None) -> str:
    th = "".join(f"<th>{_esc(h)}</th>" for h in head)
    body = []
    for i, r in enumerate(rows):
        attr = row_attrs[i] if row_attrs and i < len(row_attrs) else ""
        body.append(f"<tr{attr}>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>")
    return f'<div class="tbl-wrap"><table class="{cls}"><thead><tr>{th}</tr></thead><tbody>{"".join(body)}</tbody></table></div>'


def _facts(pairs: Sequence[Tuple[str, Any]]) -> str:
    return '<dl class="facts">' + "".join(f"<dt>{_esc(k)}</dt><dd>{v}</dd>" for k, v in pairs if v not in (None, "")) + "</dl>"


def _label(epistemic: str) -> str:
    return f'<span class="ep ep-{_esc(epistemic)}">{_esc(epistemic)}</span>'


def _section(sid: str, title: str, body: str, lead: str = "") -> str:
    lead_html = f'<p class="lead">{lead}</p>' if lead else ""
    return f'<section class="card" id="{sid}"><h2>{_esc(title)}</h2>{lead_html}{body}</section>'


def _vedic_placements(env: Dict[str, Any], division: int = 1) -> Tuple[Dict[int, List[Tuple[str, str, str]]], Optional[int]]:
    at = _attrs(env)
    out: Dict[int, List[Tuple[str, str, str]]] = {}
    for p in _bodies(env):
        abbr = BODY[p["id"]][0] + ("℞" if p.get("retrograde") and p["id"] not in ("mean_node", "south_node") else "")
        if division == 1:
            sign = int(p["longitude_deg"] // 30) % 12
            note = f'{int(p["longitude_deg"] % 30)}°'
        else:
            a = at.get(f'vedic.{p["id"]}.d{division}')
            if not a:
                continue
            sign, note = a["value"]["index"], ""
        out.setdefault(sign, []).append((p["id"], abbr, note))
    lagna = None
    if division == 1:
        asc = _lagna(env)
        lagna = int(asc // 30) % 12 if asc is not None else None
    else:
        a = at.get(f"vedic.lagna.d{division}")
        lagna = a["value"]["index"] if a else None
    return out, lagna


def _divisions(env: Dict[str, Any]) -> List[int]:
    ds = set()
    for k in _attrs(env):
        tail = k.rsplit(".", 1)[-1]
        if tail.startswith("d") and tail[1:].isdigit():
            ds.add(int(tail[1:]))
    return sorted(ds)


DIV_NAME = {1: "Rāśi", 2: "Horā", 3: "Drekkāṇa", 4: "Chaturthāṃśa", 7: "Saptāṃśa", 9: "Navāṃśa", 10: "Daśāṃśa",
            12: "Dvādaśāṃśa", 16: "Ṣoḍaśāṃśa", 20: "Viṃśāṃśa", 24: "Chaturviṃśāṃśa", 27: "Bhāṃśa",
            30: "Triṃśāṃśa", 40: "Khavedāṃśa", 45: "Akṣavedāṃśa", 60: "Ṣaṣṭyāṃśa"}


def _vedic_sections(env: Dict[str, Any], used: set) -> Dict[str, str]:
    at = _attrs(env)
    out: Dict[str, str] = {}
    calc = env.get("calculation", {})
    # chakra
    out["chakra"] = _section("chakra", "Panchang chakra", f'<div class="figure">{chakra_svg(env)}</div>',
                             "Sidereal positions on the rāśi, nakshatra and pada rings (palette from the Panchang Chakra "
                             "template). Hover a planet anywhere in the report to find it on every chart.")
    # charts + planet table
    pl, lagna = _vedic_placements(env, 1)
    charts = [f'<div class="figure">{south_indian_svg(pl, lagna, "D1 · Rāśi (South Indian)")}</div>']
    if lagna is not None:
        charts.append(f'<div class="figure">{north_indian_svg(pl, lagna, "D1 · Rāśi (North Indian)")}</div>')
    else:
        charts.append('<p class="note">The North Indian chart needs a lagna, which needs an exact birth time and '
                      'place. It is omitted rather than guessed.</p>')
    rows, attrs = [], []
    asc = _lagna(env)
    if asc is not None:
        nk = at.get("vedic.lagna.nakshatra", {}).get("value", {})
        used.update({"vedic.lagna.rashi", "vedic.lagna.nakshatra"})
        rows.append(["<b>Lagna</b>", f"{asc:.4f}°", f"{SIGN_SKT[int(asc // 30)]}", _deg_in_sign(asc),
                     _esc(nk.get("name", "")), _esc(nk.get("pada", "")), _esc(nk.get("lord", "")), "1", "", "", "",
                     ", ".join(markers(asc))])
        attrs.append(' data-body="lagna"')
    for p in _bodies(env):
        b = p["id"]
        r = at.get(f"vedic.{b}.rashi", {}).get("value", {})
        nk = at.get(f"vedic.{b}.nakshatra", {}).get("value", {})
        bh = at.get(f"vedic.{b}.bhava", {}).get("value", {})
        used.update({f"vedic.{b}.rashi", f"vedic.{b}.nakshatra", f"vedic.{b}.bhava"})
        flags = markers(p["longitude_deg"])
        if not nk.get("stable_within_uncertainty", True):
            flags.append("nakshatra unstable in time window")
        rows.append([f'<i class="dot" style="background:{_colour(BODY[b][1])}"></i>{_esc(BODY[b][2])}',
                     f'{p["longitude_deg"]:.4f}°', _esc(r.get("name", SIGN_SKT[int(p["longitude_deg"] // 30)])),
                     _esc(r.get("dms", _deg_in_sign(p["longitude_deg"]))), _esc(nk.get("name", "")),
                     _esc(nk.get("pada", "")), _esc(nk.get("lord", "")), _esc(bh.get("house", "—")),
                     "℞" if p.get("retrograde") else "", f'{p.get("speed_deg_per_day", 0):+.4f}',
                     f'{p.get("tropical_longitude_deg", 0):.4f}°', _esc(", ".join(flags))])
        attrs.append(f' data-body="{_esc(b)}"')
    table = _table(["Graha", "Sidereal", "Rāśi", "In sign", "Nakshatra", "Pada", "Nak. lord", "Bhāva", "℞",
                    "Speed °/day", "Tropical", f"Markers ({MARKER_RULES})"], rows, "planets", attrs)
    houses = env.get("houses") or {}
    htab = ""
    if houses.get("cusps_deg"):
        hrows = [[str(i + 1), f"{c:.4f}°", SIGN_SKT[int(c // 30) % 12], _deg_in_sign(c)]
                 for i, c in enumerate(houses["cusps_deg"])]
        mc = houses.get("midheaven", {}).get("longitude_deg")
        htab = (f'<h3>Houses ({_esc(houses.get("system", ""))})</h3>' + _table(["Bhāva", "Cusp", "Rāśi", "In sign"], hrows)
                + (f'<p class="note">Midheaven {mc:.4f}° · local apparent sidereal time '
                   f'{houses.get("local_apparent_sidereal_time_deg", 0):.4f}°</p>' if mc is not None else ""))
    out["vedic"] = _section(
        "vedic", "Vedic chart (sidereal)",
        f'<div class="charts">{"".join(charts)}</div>{table}{htab}',
        f'Ayanāṃśa {_esc(calc.get("ayanamsha", "—"))} = {calc.get("ayanamsha_deg", 0):.6f}° · houses '
        f'{_esc(calc.get("house_system", ""))} · node {_esc(calc.get("node_model", ""))} · provider '
        f'{_esc(calc.get("provider", ""))}')
    # divisional charts
    divs = [d for d in _divisions(env) if d != 1]
    if divs:
        tabs, panes, grid_rows = [], [], []
        for k, d in enumerate(divs):
            pl_d, lg_d = _vedic_placements(env, d)
            name = f"D{d} · {DIV_NAME.get(d, '')}"
            tabs.append(f'<button class="tab" data-tab="d{d}" aria-selected="{str(k == 0).lower()}">{_esc(name)}</button>')
            panes.append(f'<div class="pane" data-pane="d{d}"{"" if k == 0 else " hidden"}>'
                         f'{south_indian_svg(pl_d, lg_d, name)}</div>')
        bodies = (["lagna"] if asc is not None else []) + [p["id"] for p in _bodies(env)]
        for b in bodies:
            cells = []
            for d in [1] + divs:
                if d == 1:
                    lon = asc if b == "lagna" else next(p["longitude_deg"] for p in _bodies(env) if p["id"] == b)
                    cells.append(SIGN_SKT[int(lon // 30) % 12])
                else:
                    a = at.get(f"vedic.{b}.d{d}")
                    used.add(f"vedic.{b}.d{d}")
                    cells.append(_esc(a["value"]["rashi"]) if a else "—")
            name = "Lagna" if b == "lagna" else BODY[b][2]
            grid_rows.append([f"<b>{_esc(name)}</b>"] + cells)
        out["divisional"] = _section(
            "divisional", "Divisional charts (vargas)",
            f'<div class="tabs" role="tablist">{"".join(tabs)}</div><div class="figure">{"".join(panes)}</div>'
            + _table(["Body"] + [f"D{d}" for d in [1] + divs], grid_rows, "vargas"),
            "Each varga sign is computed by the engine under vedic.parashari@0.1. A body whose D1 and D9 signs "
            "match is vargottama.")
    # dasha
    das = at.get("vedic.dasha.vimshottari")
    if das:
        used.add("vedic.dasha.vimshottari")
        v = das["value"]
        now = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        blocks = []
        for m in v.get("mahadashas", []):
            cur = m["start_utc"] <= now < m["end_utc"]
            ants = "".join(
                f'<li class="{"cur" if a["start_utc"] <= now < a["end_utc"] else ""}"><b>{_esc(a["lord"])}</b> '
                f'{_esc(a["start_utc"][:10])} → {_esc(a["end_utc"][:10])}'
                f'{" <small>(from birth)</small>" if a.get("clipped_at_birth") else ""}</li>'
                for a in m.get("antardashas", []))
            blocks.append(f'<details{" open" if cur else ""} class="{"cur" if cur else ""}"><summary><b>'
                          f'{_esc(m["lord"])}</b> mahādaśā · {_esc(m["start_utc"][:10])} → {_esc(m["end_utc"][:10])}'
                          f'{" · <em>current</em>" if cur else ""}</summary><ul class="ants">{ants}</ul></details>')
        extra = {k: val for k, val in v.items() if k != "mahadashas"}
        out["dasha"] = _section(
            "dasha", "Vimśottarī daśā", _facts([(k.replace("_", " "), _esc(val)) for k, val in extra.items()])
            + "".join(blocks),
            f'{_label(das["epistemic"])} rule {_esc(das["rule_version"])}. "Current" is marked against the time this '
            f'file was generated ({now}).')
    # nakshatra reference
    ref = _nak_reference()
    if ref:
        cards = []
        for b, label in (("moon", "Janma nakshatra (Moon)"), ("lagna", "Lagna nakshatra")):
            nk = at.get(f"vedic.{b}.nakshatra", {}).get("value")
            if not nk:
                continue
            r = ref[nk["index"]]
            cards.append(f'<div class="mini"><h3>{_esc(label)}: {_esc(nk["name"])} pada {_esc(nk.get("pada", ""))}</h3>'
                         + _facts([(k.replace("_", " ").capitalize(), _esc(r[k])) for k in
                                   ("nature", "gana", "yoni", "gender", "activity", "direction", "behaviour", "dosha",
                                    "nadi", "tattva", "purpose", "guna", "gunas", "gotra", "varna")]) + "</div>")
        if cards:
            out["nakshatra"] = _section(
                "nakshatra", "Nakshatra reference", f'<div class="minis">{"".join(cards)}</div>',
                f'{_label("tradition_rule")} Attributes from the Nakshatra Ruler table of the Panchang Chakra template, '
                'shown beside the computed nakshatra. ESHANA does not compute or interpret them.')
    return out


def _western_section(env: Dict[str, Any], used: set) -> str:
    at = _attrs(env)
    rows, attrs = [], []
    for p in _bodies(env):
        b = p["id"]
        h = at.get(f"western.{b}.house", {}).get("value", {})
        used.update({f"western.{b}.sign", f"western.{b}.house"})
        rows.append([f'<i class="dot" style="background:{_colour(BODY[b][1])}"></i>{_esc(BODY[b][2].split(" (")[0] if b != "mean_node" else "North node")}',
                     f'{p["longitude_deg"]:.4f}°', SIGN_EN[int(p["longitude_deg"] // 30) % 12],
                     _deg_in_sign(p["longitude_deg"]), _esc(h.get("house", "—")), "℞" if p.get("retrograde") else "",
                     f'{p.get("latitude_deg", 0):+.4f}°'])
        attrs.append(f' data-body="{_esc(b)}"')
    asp = []
    for a in env.get("derived_attributes", []):
        if ".aspect." in a["id"]:
            used.add(a["id"])
            v = a["value"]
            b1, b2 = a["derived_from"][:2]
            asp.append([_esc(BODY.get(b1, (b1, b1, b1))[2]), f'<b>{_esc(v["aspect"])}</b>', _esc(BODY.get(b2, (b2, b2, b2))[2]),
                        f'{v["orb_deg"]:.2f}°', "applying" if v.get("applying") else "separating"])
    used.add("western.ascendant.sign")
    calc = env.get("calculation", {})
    houses = env.get("houses") or {}
    hrow = ""
    if houses.get("cusps_deg"):
        hrow = "<h3>House cusps</h3>" + _table(
            ["House", "Cusp", "Sign", "In sign"],
            [[str(i + 1), f"{c:.4f}°", SIGN_EN[int(c // 30) % 12], _deg_in_sign(c)] for i, c in enumerate(houses["cusps_deg"])])
    return _section(
        "western", "Western chart (tropical)",
        f'<div class="charts"><div class="figure">{western_wheel_svg(env)}</div></div>'
        + _table(["Body", "Longitude", "Sign", "In sign", "House", "℞", "Latitude"], rows, "planets", attrs)
        + (("<h3>Aspects</h3>" + _table(["Body", "Aspect", "Body", "Orb", "Phase"], asp)) if asp else "") + hrow,
        f'Zodiac {_esc(calc.get("zodiac", ""))} · houses {_esc(calc.get("house_system", ""))} · '
        f'aspects western.basic@0.1')


def _hd_section(hd: Dict[str, Any], used: set) -> str:
    bg = _attrs(hd).get("human_design.bodygraph", {})
    used.add("human_design.bodygraph")
    v = bg.get("value", {})
    chips = "".join(f'<span class="flag">{_esc(k)}: <b>{_esc(v.get(k, "—"))}</b></span>'
                    for k in ("type", "authority", "profile", "definition"))
    rows = []
    pers = {a["body"]: a for a in hd["positions"]["personality"]["activations"]}
    des = {a["body"]: a for a in hd["positions"]["design"]["activations"]}
    for body in pers:
        p, d = pers[body], des.get(body, {})
        rows.append([_esc(body.replace("_", " ")), f'<b>{p["gate"]}.{p["line"]}</b>', _esc(p["center"].replace("_", " ")),
                     f'<b class="red">{d.get("gate", "")}.{d.get("line", "")}</b>',
                     _esc(str(d.get("center", "")).replace("_", " "))])
    facts = _facts([
        ("Personality (UTC)", _esc(hd["positions"]["personality"].get("instant_utc", ""))),
        ("Design (UTC)", _esc(hd["positions"]["design"].get("instant_utc", ""))),
        ("Defined centres", _esc(", ".join(c.replace("_", " ") for c in v.get("defined_centers", [])))),
        ("Undefined centres", _esc(", ".join(c.replace("_", " ") for c in v.get("undefined_centers", [])))),
        ("Channels", _esc(", ".join(v.get("channels", [])))),
        ("Active gates", _esc(", ".join(str(g) for g in v.get("active_gates", [])))),
    ])
    return _section(
        "human_design", "Human Design",
        f'<div class="flags">{chips}</div><div class="hd"><div class="figure small">{bodygraph_svg(hd)}</div>'
        f'<div>{facts}</div></div>'
        + _table(["Body", "Personality", "Centre", "Design", "Centre"], rows, "hd-acts"),
        f'{_label(bg.get("epistemic", "tradition_rule"))} rule {_esc(bg.get("rule_version", ""))}. The design instant '
        'is when the Sun was 88° of arc earlier. The BodyGraph is schematic.')


def _num_section(num: Dict[str, Any], used: set) -> str:
    cards = []
    letters = ""
    for a in num.get("derived_attributes", []):
        used.add(a["id"])
        v = a["value"]
        pairs = []
        for k, val in v.items():
            if k == "letters":
                letters = _table(["Letter", "Value", "Vowel"],
                                 [[_esc(x["letter"]), str(x["value"]), "yes" if x.get("vowel") else ""] for x in val])
                continue
            if isinstance(val, dict) and "reduced" in val:
                pairs.append((k.replace("_", " "), f'<b>{val["reduced"]}</b> <small>({val.get("compound", "")})</small>'))
            elif isinstance(val, (dict, list)):
                pairs.append((k.replace("_", " "), f"<code>{_esc(json.dumps(val, ensure_ascii=False))}</code>"))
            else:
                pairs.append((k.replace("_", " "), _esc(val)))
        cards.append(f'<div class="mini"><h3>{_esc(a["id"])}</h3>{_facts(pairs)}'
                     f'<p class="note">{_label(a["epistemic"])} {_esc(a["rule_version"])}</p></div>')
    return _section("numerology", "Numerology", f'<div class="minis">{"".join(cards)}</div>'
                    + (f"<details><summary>Letter values</summary>{letters}</details>" if letters else ""),
                    "Each school is a separate rule pack; numbers are shown as reduced (compound).")


def _provenance_section(envs: Dict[str, Dict[str, Any]]) -> str:
    blocks = []
    for name, env in envs.items():
        pv = env.get("provenance", {})
        calc = env.get("calculation", {})
        pairs = [(k.replace("_", " "), _esc(v)) for k, v in calc.items()]
        pairs += [(k.replace("_", " "), _esc(v)) for k, v in pv.items() if k not in ("assumptions", "rulesets")]
        rs = pv.get("rulesets") or []
        if rs:
            pairs.append(("rulesets", _esc(", ".join(r if isinstance(r, str) else json.dumps(r) for r in rs))))
        asm = "".join(f"<li>{_esc(a if isinstance(a, str) else json.dumps(a, ensure_ascii=False))}</li>"
                      for a in pv.get("assumptions", []))
        warns = "".join(f"<li>{_esc(w if isinstance(w, str) else json.dumps(w, ensure_ascii=False))}</li>"
                        for w in env.get("warnings", []))
        blocks.append(f'<div class="mini"><h3>{_esc(name)}</h3>{_facts(pairs)}'
                      + (f"<h4>Assumptions</h4><ul>{asm}</ul>" if asm else "")
                      + (f'<h4>Warnings</h4><ul class="warn">{warns}</ul>' if warns else "") + "</div>")
    return _section("provenance", "Provenance", f'<div class="minis">{"".join(blocks)}</div>',
                    "Every setting, data source and assumption behind the numbers above, as the engine recorded it.")


def _attributes_section(envs: Dict[str, Dict[str, Any]], used: set) -> str:
    rows = []
    for name, env in envs.items():
        for a in env.get("derived_attributes", []):
            if a["id"] in used:
                continue
            rows.append([_esc(name), f'<code>{_esc(a["id"])}</code>',
                         f'<code>{_esc(json.dumps(a["value"], ensure_ascii=False)[:300])}</code>',
                         _label(a["epistemic"]), _esc(a.get("rule_version", ""))])
    if not rows:
        return ""
    return _section("attributes", "Other derived attributes", _table(["Envelope", "Id", "Value", "Label", "Rule"], rows),
                    "Every attribute the engine returned that is not shown in a section above, so nothing is hidden.")


# ── page ─────────────────────────────────────────────────────────────────────────────────────────────────────────────
CSS = r"""
:root{--paper:#f7f5f0;--panel:#fff;--ink:#1f1d1a;--muted:#6b665e;--line:#e3ded3;--accent:#c8102e;--chip:#f1ede4;
--shadow:0 10px 30px rgba(40,30,10,.12);--disc:#fff}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--paper:#16151a;--panel:#211f26;--ink:#f0ece4;
--muted:#a59f95;--line:#37343d;--accent:#ff5c70;--chip:#2c2a31;--shadow:0 10px 30px rgba(0,0,0,.5);--disc:#2a2830}}
:root[data-theme="dark"]{--paper:#16151a;--panel:#211f26;--ink:#f0ece4;--muted:#a59f95;--line:#37343d;--accent:#ff5c70;
--chip:#2c2a31;--shadow:0 10px 30px rgba(0,0,0,.5);--disc:#2a2830}
*{box-sizing:border-box}html,body{margin:0}
body{background:var(--paper);color:var(--ink);font:15px/1.45 "Open Sans","Segoe UI",system-ui,-apple-system,Roboto,sans-serif}
header.top{max-width:1240px;margin:0 auto;padding:22px 16px 6px;display:flex;flex-wrap:wrap;gap:12px;align-items:flex-end;justify-content:space-between}
header.top h1{margin:0;font-style:italic;font-weight:800;font-size:clamp(22px,3vw,34px)}
header.top .sub{color:var(--muted);font-size:14px}
.tools{display:flex;gap:6px;flex-wrap:wrap}
.btn{border:1px solid var(--line);background:var(--panel);color:var(--ink);border-radius:999px;padding:5px 12px;cursor:pointer;font:inherit;font-size:13px;box-shadow:var(--shadow)}
.btn:hover{border-color:var(--ink)}
nav.toc{max-width:1240px;margin:6px auto 0;padding:0 16px;display:flex;flex-wrap:wrap;gap:6px}
nav.toc a{background:var(--chip);color:var(--ink);text-decoration:none;border-radius:999px;padding:3px 10px;font-size:13px}
main{max-width:1240px;margin:0 auto;padding:10px 16px 40px;display:grid;gap:18px}
.card{background:var(--panel);border:1px solid var(--line);border-radius:16px;box-shadow:var(--shadow);padding:16px 18px;min-width:0}
.card h2{margin:0 0 6px;font-style:italic;font-weight:800;font-size:22px}
.card h3{font-size:15px;margin:14px 0 6px}
p.lead,.note{color:var(--muted);font-size:13.5px;margin:0 0 10px}
.charts{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:14px;align-items:start;margin-bottom:12px}
.figure svg{width:100%;height:auto;display:block}
.figure.small{max-width:300px}
.figure .chakra{max-width:760px;margin:0 auto}
.lbl{text-anchor:middle;dominant-baseline:central;pointer-events:none}
.seg{stroke:#fff;stroke-width:1.2}
.tbl-wrap{overflow:auto;border:1px solid var(--line);border-radius:12px;margin:6px 0 10px}
table{border-collapse:collapse;width:100%;font-size:13px}
th,td{padding:6px 9px;text-align:left;vertical-align:top;border-bottom:1px solid var(--line);white-space:nowrap}
th{background:var(--chip);font-size:12.5px;position:sticky;top:0}
tbody tr:hover,tr.hl{background:var(--chip)}
.dot{display:inline-block;width:10px;height:10px;border-radius:50%;border:1px solid rgba(0,0,0,.35);margin-right:6px;vertical-align:middle}
.facts{display:grid;grid-template-columns:minmax(120px,auto) 1fr;gap:4px 12px;margin:0 0 10px;font-size:13.5px}
.facts dt{color:var(--muted)}.facts dd{margin:0;overflow-wrap:anywhere}
.flags{display:flex;flex-wrap:wrap;gap:6px;margin:4px 0 12px}
.flag{border:1px solid var(--line);background:var(--chip);border-radius:999px;padding:3px 10px;font-size:13px}
.minis{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:12px}
.mini{border:1px solid var(--line);border-radius:12px;padding:10px 12px}
.mini h3{margin-top:0}
.hd{display:grid;grid-template-columns:minmax(200px,300px) 1fr;gap:16px;align-items:start}
@media (max-width:720px){.hd{grid-template-columns:1fr}}
.red{color:var(--accent)}
.ep{display:inline-block;font-size:11.5px;font-weight:700;border-radius:999px;padding:1px 8px;background:var(--chip);border:1px solid var(--line)}
.ep-calculated{color:#2563EB}.ep-tradition_rule{color:#7C3AED}
details{border:1px solid var(--line);border-radius:10px;padding:6px 10px;margin:6px 0}
details.cur{border-color:var(--accent)}
summary{cursor:pointer}
ul.ants{columns:2;font-size:13px;margin:6px 0}
@media (max-width:720px){ul.ants{columns:1}}
ul.ants li.cur{color:var(--accent);font-weight:700}
ul.warn li{color:var(--accent)}
.tabs{display:flex;flex-wrap:wrap;gap:6px;margin:4px 0 10px}
.tab{border:1px solid var(--line);background:var(--chip);color:var(--ink);border-radius:999px;padding:3px 11px;cursor:pointer;font:inherit;font-size:13px}
.tab[aria-selected="true"]{background:var(--ink);color:var(--panel)}
.pane{max-width:520px}
[data-body].hl circle,[data-body].hl{filter:drop-shadow(0 0 6px var(--accent))}
line.asp.dim{opacity:.08}
pre.raw{max-height:420px;overflow:auto;background:var(--chip);border-radius:10px;padding:10px;font-size:12px}
footer{max-width:1240px;margin:0 auto 30px;padding:0 16px;color:var(--muted);font-size:12.5px}
code{font-size:12px}
@media print{.tools,nav.toc{display:none}.card{box-shadow:none;break-inside:avoid}body{background:#fff}
details{break-inside:avoid}details>*{display:block}}
@media (prefers-reduced-motion:reduce){*{transition:none!important}}
"""

JS = r"""
(function(){
  var root=document.documentElement;
  try{var t=localStorage.getItem('eshana-theme'); if(t) root.setAttribute('data-theme',t);}catch(e){}
  var tb=document.getElementById('theme');
  if(tb) tb.addEventListener('click',function(){
    var cur=root.getAttribute('data-theme')||(matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light');
    var nx=cur==='dark'?'light':'dark'; root.setAttribute('data-theme',nx);
    try{localStorage.setItem('eshana-theme',nx);}catch(e){}
  });
  var pb=document.getElementById('print'); if(pb) pb.addEventListener('click',function(){
    document.querySelectorAll('details').forEach(function(d){d.open=true}); window.print();});
  var jb=document.getElementById('json'); if(jb) jb.addEventListener('click',function(){
    var raw=document.getElementById('eshana-data').textContent;
    var a=document.createElement('a'); a.href=URL.createObjectURL(new Blob([raw],{type:'application/json'}));
    a.download='eshana-envelopes.json'; document.body.appendChild(a); a.click(); a.remove();});
  function mark(id,on){
    document.querySelectorAll('[data-body]').forEach(function(el){
      var ids=el.getAttribute('data-body').split(' ');
      if(ids.indexOf(id)>=0) el.classList.toggle('hl',on);
      if(el.classList.contains('asp')) el.classList.toggle('dim',on&&ids.indexOf(id)<0);
    });
  }
  document.querySelectorAll('[data-body]').forEach(function(el){
    var id=el.getAttribute('data-body').split(' ')[0];
    if(el.classList.contains('asp')) return;
    el.addEventListener('mouseenter',function(){mark(id,true)});
    el.addEventListener('mouseleave',function(){mark(id,false)});
  });
  document.querySelectorAll('.tabs').forEach(function(tabs){
    var card=tabs.closest('.card');
    tabs.querySelectorAll('.tab').forEach(function(b){b.addEventListener('click',function(){
      tabs.querySelectorAll('.tab').forEach(function(x){x.setAttribute('aria-selected',String(x===b))});
      card.querySelectorAll('.pane').forEach(function(p){p.hidden=p.getAttribute('data-pane')!==b.getAttribute('data-tab')});
    })});
  });
})();
"""


def render_html(envelopes: Dict[str, Dict[str, Any]], title: str = "ESHANA chart report",
                subject: Optional[Dict[str, Any]] = None, sections: Optional[Iterable[str]] = None,
                theme: str = "auto") -> str:
    """Render engine envelopes into one self-contained HTML document.

    ``envelopes`` maps a label to an envelope; kinds are recognised from the envelope itself (``computed_chart`` with
    ``calculation.system`` vedic/western, ``human_design``, ``numerology``). ``subject`` holds display-only details
    such as the person's name and a place label. ``sections`` restricts output to names in :data:`SECTIONS`.
    """
    want = set(sections or SECTIONS)
    unknown = want - set(SECTIONS)
    if unknown:
        raise ValueError(f"unknown sections: {', '.join(sorted(unknown))}; choose from {', '.join(SECTIONS)}")
    vedic = western = hd = num = None
    for env in envelopes.values():
        kind = env.get("kind")
        system = (env.get("calculation") or {}).get("system")
        if kind == "computed_chart" and system == "vedic":
            vedic = env
        elif kind == "computed_chart" and system == "western":
            western = env
        elif kind == "human_design":
            hd = env
        elif env.get("derived_attributes") and str(env["derived_attributes"][0].get("id", "")).startswith("numerology"):
            num = env
    used: set = set()
    parts: Dict[str, str] = {}
    subject = subject or {}
    first = next(iter(envelopes.values()), {})
    inp = first.get("input", {})
    place = inp.get("place") or {}
    summary_pairs = [
        ("Name", _esc(subject.get("name", ""))),
        ("Place", _esc(subject.get("place_label", ""))),
        ("Local date and time", _esc(subject.get("local", ""))),
        ("Instant (UTC)", _esc(inp.get("instant_utc", ""))),
        ("Time zone", _esc(inp.get("civil_zone", "")) + (f' ({inp["civil_offset_s"] / 3600:+.2f} h)' if
                                                         inp.get("civil_offset_s") is not None else "")),
        ("Latitude, longitude", f'{place["latitude_deg"]:.4f}°, {place["longitude_deg"]:.4f}°' if place else ""),
        ("Birth time quality", _esc(inp.get("birth_time_quality", ""))),
        ("Julian day (UT / TT)", f'{inp["jd_ut"]:.6f} / {inp.get("jd_tt", 0):.6f}' if "jd_ut" in inp else ""),
        ("ΔT", f'{inp["delta_t_s"]:.2f} s' if "delta_t_s" in inp else ""),
    ]
    chips = []
    if vedic:
        at = _attrs(vedic)
        for k, lab in (("vedic.lagna.rashi", "Lagna"), ("vedic.moon.rashi", "Moon rāśi"),
                       ("vedic.sun.rashi", "Sun rāśi")):
            if k in at:
                chips.append(f'<span class="flag">{lab}: <b>{_esc(at[k]["value"]["name"])}</b></span>')
        mn = at.get("vedic.moon.nakshatra")
        if mn:
            chips.append(f'<span class="flag">Janma nakshatra: <b>{_esc(mn["value"]["name"])} '
                         f'{_esc(mn["value"].get("pada", ""))}</b></span>')
    if hd:
        bg = _attrs(hd).get("human_design.bodygraph", {}).get("value", {})
        if bg:
            chips.append(f'<span class="flag">HD: <b>{_esc(bg.get("type"))} {_esc(bg.get("profile"))}</b></span>')
    parts["summary"] = _section("summary", "Birth data", f'<div class="flags">{"".join(chips)}</div>' + _facts(summary_pairs),
                                "Computed by ESHANA. Values are labelled calculated or tradition_rule; nothing here is "
                                "an interpretation.")
    if vedic:
        parts.update(_vedic_sections(vedic, used))
    if western:
        parts["western"] = _western_section(western, used)
    if hd:
        parts["human_design"] = _hd_section(hd, used)
    if num:
        parts["numerology"] = _num_section(num, used)
    parts["attributes"] = _attributes_section(envelopes, used)
    parts["provenance"] = _provenance_section(envelopes)
    raw = json.dumps(envelopes, ensure_ascii=False, indent=1)
    parts["raw"] = _section("raw", "Raw envelopes", f'<details><summary>Show JSON ({len(raw):,} characters)</summary>'
                            f'<pre class="raw">{_esc(raw)}</pre></details>',
                            "Exactly what the engine returned. The Export JSON button saves the same data.")
    body = [parts[s] for s in SECTIONS if s in want and parts.get(s)]
    toc_names = {"summary": "Birth data", "chakra": "Chakra", "vedic": "Vedic", "divisional": "Vargas",
                 "dasha": "Daśā", "nakshatra": "Nakshatra", "western": "Western", "human_design": "Human Design",
                 "numerology": "Numerology", "attributes": "Other", "provenance": "Provenance", "raw": "JSON"}
    toc = "".join(f'<a href="#{s}">{toc_names[s]}</a>' for s in SECTIONS if s in want and parts.get(s))
    version = (first.get("provenance") or {}).get("engine_version", "")
    theme_attr = f' data-theme="{theme}"' if theme in ("light", "dark") else ""
    data_json = json.dumps(envelopes, ensure_ascii=False).replace("</", "<\\/")
    generated = _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    sub = " · ".join(x for x in (subject.get("name"), subject.get("local"), subject.get("place_label")) if x)
    return f"""<!doctype html>
<html lang="en"{theme_attr}>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="generator" content="ESHANA {_esc(version)}">
<title>{_esc(title)}</title>
<style>{CSS}</style>
</head>
<body>
<header class="top"><div><h1>{_esc(title)}</h1><div class="sub">{_esc(sub)}</div></div>
<div class="tools"><button class="btn" id="theme" type="button">◐ Theme</button>
<button class="btn" id="print" type="button">⎙ Print / PDF</button>
<button class="btn" id="json" type="button">⤓ Export JSON</button></div></header>
<nav class="toc" aria-label="Sections">{toc}</nav>
<main>
{"".join(body)}
</main>
<footer>Generated {generated} by ESHANA {_esc(version)} (Extensible Spatiotemporal Hypersystem for Astroinformatics,
Natural Cycles &amp; Associations). Open source, Apache-2.0. Chart markers follow {MARKER_RULES}. This file is
self-contained and makes no network requests.</footer>
<script type="application/json" id="eshana-data">{data_json}</script>
<script>{JS}</script>
</body>
</html>
"""


# ── running the engine ─────────────────────────────────────────────────────────────────────────────────────────────
def envelopes_for(ctx: Any, *, name: Optional[str] = None, ayanamsha: str = "lahiri", houses: str = "whole_sign",
                  western_houses: str = "placidus", node_model: str = "mean", divisions: Sequence[int] = (1, 3, 4, 7, 9, 10, 12),
                  dasha: bool = True, systems: Sequence[str] = ("vedic", "western", "human_design", "numerology"),
                  provider: str = "analytic", spk_path: Optional[str] = None,
                  numerology_school: str = "pythagorean") -> Dict[str, Dict[str, Any]]:
    """Run the engine for each requested system. A system that cannot be computed becomes an ``error`` entry."""
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
                                             divisions=list(divisions), dasha=dasha))
    if "western" in systems:
        run("western", lambda: eng.western_chart(ctx, houses=western_houses, node_model=node_model, aspects=True))
    if "human_design" in systems:
        run("human_design", lambda: eng.human_design(ctx))
    if "numerology" in systems and (name or ctx.when.get("date")):
        run("numerology", lambda: eng.numerology(name=name, date_of_birth=ctx.when.get("date"), school=numerology_school))
    if errors:
        out["_errors"] = {"kind": "report_errors", "errors": errors}
    return out


def birth_report(ctx: Any, *, name: Optional[str] = None, place_label: Optional[str] = None,
                 title: Optional[str] = None, sections: Optional[Iterable[str]] = None, theme: str = "auto",
                 **kw: Any) -> str:
    """Compute every requested system for ``ctx`` (an :class:`eshana.Context`) and return the HTML report."""
    envs = envelopes_for(ctx, name=name, **kw)
    errs = envs.pop("_errors", None)
    local = " ".join(str(ctx.when.get(k)) for k in ("date", "time") if ctx.when.get(k)) or ctx.when.get("utc", "")
    if ctx.when.get("timezone"):
        local += f' ({ctx.when["timezone"]})'
    page = render_html(envs, title=title or (f"{name} · birth chart" if name else "Birth chart"),
                       subject={"name": name, "place_label": place_label, "local": local}, sections=sections,
                       theme=theme)
    if errs:
        note = "".join(f"<li><b>{_esc(k)}</b>: {_esc(v)}</li>" for k, v in errs["errors"].items())
        page = page.replace("<main>\n", f'<main>\n<section class="card" id="not-computed"><h2>Not computed</h2>'
                            f'<p class="lead">These systems were refused by the engine, so they are left out rather '
                            f'than guessed.</p><ul class="warn">{note}</ul></section>', 1)
    return page


def _parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="eshana-report", description="Write a self-contained HTML chart report.")
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
    p.add_argument("--place", dest="place_label", help="place label shown in the report")
    p.add_argument("--title")
    p.add_argument("--ayanamsha", default="lahiri", choices=["lahiri", "raman", "krishnamurti", "fagan_bradley"])
    p.add_argument("--houses", default="whole_sign", choices=["whole_sign", "equal", "placidus"],
                   help="Vedic house system")
    p.add_argument("--western-houses", default="placidus", choices=["whole_sign", "equal", "placidus"])
    p.add_argument("--node", default="mean", choices=["mean", "true"])
    p.add_argument("--divisions", default="1,3,4,7,9,10,12")
    p.add_argument("--no-dasha", action="store_true")
    p.add_argument("--systems", default="vedic,western,human_design,numerology")
    p.add_argument("--school", default="pythagorean", choices=["pythagorean", "chaldean"])
    p.add_argument("--provider", default="analytic", choices=["analytic", "spk"])
    p.add_argument("--spk", help="JPL SPK kernel for --provider spk")
    p.add_argument("--sections", help=f"comma list from: {','.join(SECTIONS)}")
    p.add_argument("--theme", default="auto", choices=["auto", "light", "dark"])
    return p


def main(argv: Optional[Sequence[str]] = None) -> int:
    from . import Context, EshanaError

    a = _parser().parse_args(argv)
    sections = a.sections.split(",") if a.sections else None
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
                               subject={"name": a.name, "place_label": a.place_label}, sections=sections, theme=a.theme)
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
            page = birth_report(ctx, name=a.name, place_label=a.place_label, title=a.title, sections=sections,
                                theme=a.theme, ayanamsha=a.ayanamsha, houses=a.houses, western_houses=a.western_houses,
                                node_model=a.node, divisions=[int(x) for x in a.divisions.split(",") if x],
                                dasha=not a.no_dasha, systems=a.systems.split(","), provider=a.provider,
                                spk_path=a.spk, numerology_school=a.school)
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
