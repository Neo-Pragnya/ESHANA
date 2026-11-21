/* ESHANA chart studio, part 1: data, constants, helpers, planet icons.
   Everything shown is read from the engine envelopes embedded in the page (#eshana-data). The page computes
   nothing astrological itself, except the chart markers of report.markers@0.1 (pushkara, vargottama, gandanta),
   which are fixed published rules and are labelled where they appear. */
'use strict';
const D = JSON.parse(document.getElementById('eshana-data').textContent);
const ENV = D.envelopes || {};
const NS = 'http://www.w3.org/2000/svg';
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
function h(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag);
  for (const k in attrs) {
    const v = attrs[k];
    if (v == null || v === false) continue;
    if (k === 'class') e.className = v; else if (k === 'html') e.innerHTML = v;
    else if (k.startsWith('on')) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v === true ? '' : v);
  }
  for (const c of kids.flat()) if (c != null && c !== false) e.append(c.nodeType ? c : document.createTextNode(String(c)));
  return e;
}
function s(tag, attrs = {}, parent) {
  const e = document.createElementNS(NS, tag);
  for (const k in attrs) if (attrs[k] != null) e.setAttribute(k, attrs[k]);
  if (parent) parent.appendChild(e);
  return e;
}
const esc = x => String(x == null ? '' : x).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const DEG = Math.PI / 180;
const norm = x => ((x % 360) + 360) % 360;
function dms(x) { x = norm(x); let d = Math.floor(x), m = Math.floor((x - d) * 60), sec = Math.round(((x - d) * 60 - m) * 60);
  if (sec === 60) { m++; sec = 0; } if (m === 60) { d++; m = 0; } return `${d}°${String(m).padStart(2, '0')}′${String(sec).padStart(2, '0')}″`; }
const inSign = x => dms(norm(x) % 30);
const fmtTime = iso => iso ? iso.replace('T', ' ').replace(/\.\d+/, '').replace(/([+-]\d\d:\d\d|Z)$/, ' $1') : '—';
const dateOnly = iso => iso ? iso.slice(0, 10) : '—';

/* ── tradition names (labels only) ───────────────────────────────────────────────────────────────────────────── */
const SIGN_EN = ['Aries', 'Taurus', 'Gemini', 'Cancer', 'Leo', 'Virgo', 'Libra', 'Scorpio', 'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces'];
const SIGN_SKT = ['Mesha', 'Vrishabha', 'Mithuna', 'Karka', 'Simha', 'Kanya', 'Tula', 'Vrischika', 'Dhanu', 'Makara', 'Kumbha', 'Meena'];
const GLYPH = ['♈', '♉', '♊', '♋', '♌', '♍', '♎', '♏', '♐', '♑', '♒', '♓'].map(g => g + '\uFE0E');
const SIGN_LORD = ['mars', 'venus', 'mercury', 'moon', 'sun', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'saturn', 'jupiter'];
const ELEMENT = ['Fire', 'Earth', 'Air', 'Water'];
const NAKSHATRA = ['Ashwini', 'Bharani', 'Krittika', 'Rohini', 'Mrigashira', 'Ardra', 'Punarvasu', 'Pushya', 'Ashlesha', 'Magha',
  'Purva Phalguni', 'Uttara Phalguni', 'Hasta', 'Chitra', 'Swati', 'Vishakha', 'Anuradha', 'Jyeshtha', 'Mula', 'Purva Ashadha',
  'Uttara Ashadha', 'Shravana', 'Dhanishta', 'Shatabhisha', 'Purva Bhadrapada', 'Uttara Bhadrapada', 'Revati'];
const NAK_SHORT = { 'Purva Phalguni': 'P. Phalguni', 'Uttara Phalguni': 'U. Phalguni', 'Purva Ashadha': 'P. Ashadha',
  'Uttara Ashadha': 'U. Ashadha', 'Purva Bhadrapada': 'P. Bhadra', 'Uttara Bhadrapada': 'U. Bhadra' };
const NAK_LORD = ['ketu', 'venus', 'sun', 'moon', 'mars', 'rahu', 'jupiter', 'saturn', 'mercury'];
const NAKS = 40 / 3, NAV = 10 / 3;
const PADA_FILL = { Fire: '#FFE24D', Earth: '#A6EDD5', Air: '#C5EBA0', Water: '#6FE2EE' };
/* graha metadata; band/nak colours from the Panchang Chakra template */
const G = {
  sun:     { abbr: 'Su', skt: 'Surya',   en: 'Sun',     glyph: '☉', dot: '#F39A1E', band: '#F4B31B', nak: '#F4B31B' },
  moon:    { abbr: 'Mo', skt: 'Chandra', en: 'Moon',    glyph: '☽', dot: '#E6E9F0', band: '#FFFFFF', nak: '#FFFFFF' },
  mars:    { abbr: 'Ma', skt: 'Mangala', en: 'Mars',    glyph: '♂', dot: '#D3133F', band: '#F2A6AD', nak: '#E0583B' },
  mercury: { abbr: 'Me', skt: 'Budha',   en: 'Mercury', glyph: '☿', dot: '#C4D84B', band: '#CCDB57', nak: '#A9C79A' },
  jupiter: { abbr: 'Ju', skt: 'Guru',    en: 'Jupiter', glyph: '♃', dot: '#FFE81A', band: '#FFEE1E', nak: '#FFE21C' },
  venus:   { abbr: 'Ve', skt: 'Shukra',  en: 'Venus',   glyph: '♀', dot: '#3FE0E0', band: '#52E5E2', nak: '#E39479' },
  saturn:  { abbr: 'Sa', skt: 'Shani',   en: 'Saturn',  glyph: '♄', dot: '#7E99C9', band: '#C8D5EA', nak: '#8DA2D0' },
  rahu:    { abbr: 'Ra', skt: 'Rahu',    en: 'Rahu',    glyph: '☊', dot: '#6B4FA8', band: '#B6A8DA', nak: '#B6A8DA' },
  ketu:    { abbr: 'Ke', skt: 'Ketu',    en: 'Ketu',    glyph: '☋', dot: '#9C9C9C', band: '#DCDCDC', nak: '#DCDCDC' },
  uranus:  { abbr: 'Ur', skt: 'Uranus',  en: 'Uranus',  glyph: '♅', dot: '#7FD3DC' },
  neptune: { abbr: 'Ne', skt: 'Neptune', en: 'Neptune', glyph: '♆', dot: '#4C7CF0' },
  pluto:   { abbr: 'Pl', skt: 'Pluto',   en: 'Pluto',   glyph: '♇', dot: '#CDB39A' },
  lagna:   { abbr: 'As', skt: 'Lagna',   en: 'Ascendant', glyph: 'Asc', dot: '#C8102E' },
};
const GRAHA_ORDER = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn', 'rahu', 'ketu', 'uranus', 'neptune', 'pluto'];
const engineId = id => ({ mean_node: 'rahu', true_node: 'rahu', south_node: 'ketu' }[id] || id);
const ASPECT_COLOUR = { conjunction: '#888', opposition: '#d23c3c', square: '#d23c3c', trine: '#2563eb', sextile: '#16a34a' };
function ink(hex) { const n = parseInt(hex.slice(1), 16); const c = [n >> 16 & 255, n >> 8 & 255, n & 255].map(v => { v /= 255; return v <= .03928 ? v / 12.92 : Math.pow((v + .055) / 1.055, 2.4); });
  return .2126 * c[0] + .7152 * c[1] + .0722 * c[2] < .42 ? '#fff' : '#1b1b1b'; }

/* ── the chart model, read from the envelopes ─────────────────────────────────────────────────────────────────── */
function attrsOf(env) { const m = {}; for (const a of (env && env.derived_attributes) || []) m[a.id] = a; return m; }
const VED = ENV.vedic, WES = ENV.western, HD = ENV.human_design, NUM = ENV.numerology, MATCH = ENV.matching;
const VA = attrsOf(VED), WA = attrsOf(WES);
const val = id => VA[id] ? VA[id].value : null;
function grahasOf(env) {
  const out = {};
  for (const p of (env && env.positions) || []) {
    const id = engineId(p.id); if (!G[id]) continue;
    out[id] = { id, lon: p.longitude_deg, trop: p.tropical_longitude_deg, lat: p.latitude_deg, speed: p.speed_deg_per_day,
      retro: !!p.retrograde, interval: p.longitude_interval_deg };
  }
  return out;
}
const GR = grahasOf(VED);
const ASC = VED && VED.houses && VED.houses.ascendant ? VED.houses.ascendant.longitude_deg : null;
const MC = VED && VED.houses && VED.houses.midheaven ? VED.houses.midheaven.longitude_deg : null;
const LAGNA_SIGN = ASC == null ? null : Math.floor(norm(ASC) / 30);
const signOf = l => Math.floor(norm(l) / 30);
const nakOf = l => Math.floor(norm(l) / NAKS);
const padaOf = l => Math.floor((norm(l) % NAKS) / NAV) + 1;
const navSignOf = l => Math.floor(norm(l) / NAV) % 12;
const dresSignOf = l => (signOf(l) + [0, 4, 8][Math.min(2, Math.floor((norm(l) % 30) / 10))]) % 12;
const PUSHKARA = { Fire: [6, 8], Earth: [2, 4], Air: [5, 7], Water: [0, 2] };
const GANDANTA = [107, 0, 35, 36, 71, 72];
function markers(l) { /* report.markers@0.1: fixed rules printed in the Panchang Chakra template */
  const i = Math.floor(norm(l) / NAV) % 108, r = Math.floor(i / 9), pos = i % 9, out = [];
  if (PUSHKARA[ELEMENT[r % 4]].includes(pos)) out.push('pushkara navamsha');
  if (i % 12 === r) out.push('vargottama');
  if (GANDANTA.includes(i)) out.push('gandanta');
  return out;
}
function vargaSign(id, d) {
  if (d === 1) return id === 'lagna' ? LAGNA_SIGN : (GR[id] ? signOf(GR[id].lon) : null);
  const keys = id === 'rahu' ? ['mean_node', 'true_node'] : id === 'ketu' ? ['south_node'] : [id];
  for (const k of keys) { const a = VA[`vedic.${k}.d${d}`]; if (a) return a.value.index; }
  return null;
}
const DIVS = (() => { const ds = new Set([1]); for (const k in VA) { const m = k.match(/\.d(\d+)$/); if (m) ds.add(+m[1]); } return [...ds].sort((a, b) => a - b); })();
const DIV_NAME = { 1: 'Rāśi', 2: 'Horā', 3: 'Drekkāṇa', 4: 'Chaturthāṃśa', 7: 'Saptāṃśa', 9: 'Navāṃśa', 10: 'Daśāṃśa', 12: 'Dvādaśāṃśa',
  16: 'Ṣoḍaśāṃśa', 20: 'Viṃśāṃśa', 24: 'Chaturviṃśāṃśa', 27: 'Bhāṃśa', 30: 'Triṃśāṃśa', 40: 'Khavedāṃśa', 45: 'Akṣavedāṃśa', 60: 'Ṣaṣṭyāṃśa' };

/* ── settings (per viewer, kept in this browser only) ─────────────────────────────────────────────────────────── */
const DEFAULTS = Object.assign({
  icons: 'sphere', chart: 'south', varga: 1, orientation: 'template', hd: 'detailed', animate: true,
  layers: { nak: true, pada: true, dres: false, houses: true, ticks: true, needles: true, drishti: true, rashi_drishti: false,
    west_aspects: false, transits: false, sav: false, arudha: true, chart_drishti: false },
}, D.defaults || {});
let S = JSON.parse(JSON.stringify(DEFAULTS));
try { const saved = JSON.parse(localStorage.getItem('eshana-studio') || 'null'); if (saved) S = Object.assign(S, saved, { layers: Object.assign(S.layers, saved.layers || {}) }); } catch (e) { /* private mode */ }
function save() { try { localStorage.setItem('eshana-studio', JSON.stringify(S)); } catch (e) { /* ignore */ } }

/* ── tooltip ──────────────────────────────────────────────────────────────────────────────────────────────────── */
const TIP = h('div', { class: 'tip', role: 'tooltip' }); document.body.append(TIP);
function tip(ev, title, rows, sub) {
  TIP.innerHTML = `<h4>${esc(title)}</h4>${sub ? `<div class="note">${esc(sub)}</div>` : ''}<dl>${(rows || []).filter(r => r && r[1] != null && r[1] !== '')
    .map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>`;
  const x = Math.min(ev.clientX + 14, innerWidth - TIP.offsetWidth - 8), y = Math.min(ev.clientY + 14, innerHeight - TIP.offsetHeight - 8);
  TIP.style.left = x + 'px'; TIP.style.top = y + 'px'; TIP.classList.add('on');
}
function untip() { TIP.classList.remove('on'); }

/* ── planet icons: textured spheres (procedural textures as in the Astro-Energetics slide), glyphs or badges ─── */
const ICON = {};
(function buildSpheres() {
  const fr = x => x - Math.floor(x), h3 = (x, y, z) => fr(Math.sin(x * 127.1 + y * 311.7 + z * 74.7) * 43758.5453), sm = t => t * t * (3 - 2 * t);
  function vn(x, y, z) { const xi = Math.floor(x), yi = Math.floor(y), zi = Math.floor(z), xf = sm(x - xi), yf = sm(y - yi), zf = sm(z - zi);
    const L = (a, b, t) => a + (b - a) * t, c = (i, j, k) => h3(xi + i, yi + j, zi + k);
    return L(L(L(c(0, 0, 0), c(1, 0, 0), xf), L(c(0, 1, 0), c(1, 1, 0), xf), yf), L(L(c(0, 0, 1), c(1, 0, 1), xf), L(c(0, 1, 1), c(1, 1, 1), xf), yf), zf); }
  function fbm(x, y, z, o = 4) { let a = 0, sc = .5, f = 1; for (let i = 0; i < o; i++) { a += sc * vn(x * f, y * f, z * f); sc *= .5; f *= 2.03; } return a / (1 - Math.pow(.5, o)); }
  const mix = (a, b, t) => a.map((v, i) => v + (b[i] - v) * t), hx = c => [1, 3, 5].map(i => parseInt(c.slice(i, i + 2), 16));
  const crater = (seed, n) => Array.from({ length: n }, (_, i) => ({ u: h3(i, seed, 1), v: .12 + h3(i, seed, 2) * .76, r: .02 + h3(i, seed, 3) * .06 }));
  const shade = (list, u, v) => { let sh = 0; for (const c of list) { let du = Math.abs(u - c.u); du = Math.min(du, 1 - du) * 2; const d = Math.hypot(du, v - c.v) / c.r; if (d < 1) sh -= .25 * (1 - d); else if (d < 1.25) sh += .18 * (1.25 - d) * 4; } return sh; };
  const CR1 = crater(7, 40), CR2 = crater(11, 30);
  const TEX = {
    sun: (u, v, cx, sx) => mix(hx('#ff7a00'), hx('#fff2a8'), Math.min(1, fbm(cx * 6, v * 12, sx * 6, 5) * 1.35)),
    mercury: (u, v, cx, sx) => mix(hx('#5e5852'), hx('#b3aba3'), fbm(cx * 3, v * 6, sx * 3, 5)).map(x => x * (1 + shade(CR1, u, v))),
    venus: (u, v, cx, sx) => mix(hx('#b5823c'), hx('#f6e1a9'), fbm(cx * 2 + v * 3, v * 9, sx * 2, 5)),
    moon: (u, v, cx, sx) => { let c = mix(hx('#6f6f74'), hx('#d4d4d8'), fbm(cx * 2, v * 4, sx * 2, 5)); if (fbm(cx * 1.2 + 3, v * 2.4, sx * 1.2, 3) > .58) c = c.map(x => x * .7); return c.map(x => x * (1 + shade(CR2, u, v))); },
    mars: (u, v, cx, sx) => { const lat = Math.abs(v - .5) * 2; let c = mix(hx('#7a2a10'), hx('#e07a45'), fbm(cx * 2.5, v * 5, sx * 2.5, 5)); if (fbm(cx * 1.5 + 9, v * 3, sx * 1.5, 3) > .6) c = c.map(x => x * .65); if (lat > .86) c = hx('#f4efe9'); return c; },
    jupiter: (u, v, cx, sx) => { const n = fbm(cx * 2, v * 8, sx * 2, 4), b = Math.sin(v * Math.PI * 15 + n * 2.5), pal = [hx('#7d5a3c'), hx('#d9b48a'), hx('#f2e2c6'), hx('#b07a52')];
      let c = mix(pal[b > 0 ? 1 : 0], pal[b > 0 ? 2 : 3], Math.abs(b)); let du = Math.abs(u - .3); du = Math.min(du, 1 - du); const d = Math.hypot(du * 3.2, (v - .64) * 6); if (d < .32) c = mix(hx('#c0472c'), c, d / .32); return c; },
    saturn: (u, v, cx, sx) => { const n = fbm(cx * 2, v * 6, sx * 2, 3), b = Math.sin(v * Math.PI * 11 + n * 1.5); return mix(hx('#c9ad74'), hx('#efe0b4'), (b + 1) / 2 * .8 + n * .2); },
    uranus: (u, v, cx, sx) => mix(hx('#7fd3dc'), hx('#c8f2f4'), fbm(cx * 2, v * 5, sx * 2, 3) * .6 + Math.sin(v * Math.PI * 6) * .1),
    neptune: (u, v, cx, sx) => { const n = fbm(cx * 2 + v, v * 6, sx * 2, 4); let c = mix(hx('#1d3fae'), hx('#4c7cf0'), n); let du = Math.abs(u - .6); du = Math.min(du, 1 - du); if (Math.hypot(du * 3, (v - .4) * 5) < .25) c = hx('#152a74'); return c; },
    pluto: (u, v, cx, sx) => { const n = fbm(cx * 2.5, v * 5, sx * 2.5, 4); let c = mix(hx('#6e5442'), hx('#cdb39a'), n); let du = Math.abs(u - .5); du = Math.min(du, 1 - du); if (Math.hypot(du * 2.2, (v - .55) * 2.6) < .28) c = hx('#efe3d2'); return c; },
  };
  const R = 48, SZ = 128;
  function sphere(id) {
    let cv; try { cv = document.createElement('canvas'); cv.width = SZ; cv.height = SZ; } catch (e) { return null; }
    const ctx = cv.getContext && cv.getContext('2d'); if (!ctx) return null;
    const cx0 = SZ / 2, cy0 = SZ / 2;
    if (id === 'rahu' || id === 'ketu') { /* shadow planets: an eclipse disc with a halo (Rahu) or a smoky tail (Ketu) */
      if (id === 'ketu') { const g = ctx.createLinearGradient(cx0, cy0, cx0 + R * 1.3, cy0 + R * 1.2); g.addColorStop(0, 'rgba(240,150,90,.85)'); g.addColorStop(1, 'rgba(240,150,90,0)');
        ctx.fillStyle = g; ctx.beginPath(); ctx.moveTo(cx0 - R * .3, cy0 + R * .4); ctx.quadraticCurveTo(cx0 + R * .9, cy0 + R * .1, cx0 + R * 1.3, cy0 + R * 1.25); ctx.quadraticCurveTo(cx0 + R * .4, cy0 + R * .9, cx0 + R * .4, cy0 - R * .3); ctx.fill(); }
      const halo = ctx.createRadialGradient(cx0, cy0, R * .7, cx0, cy0, R * 1.25); halo.addColorStop(0, id === 'rahu' ? 'rgba(170,140,255,.95)' : 'rgba(255,200,140,.8)'); halo.addColorStop(1, 'rgba(0,0,0,0)');
      ctx.fillStyle = halo; ctx.beginPath(); ctx.arc(cx0, cy0, R * 1.25, 0, 7); ctx.fill();
      const disc = ctx.createRadialGradient(cx0 - R * .3, cy0 - R * .3, R * .1, cx0, cy0, R); disc.addColorStop(0, '#2b2340'); disc.addColorStop(1, '#05040a');
      ctx.fillStyle = disc; ctx.beginPath(); ctx.arc(cx0, cy0, R * .82, 0, 7); ctx.fill();
      return cv.toDataURL();
    }
    const fn = TEX[id]; if (!fn) return null;
    const img = ctx.createImageData(SZ, SZ);
    for (let y = 0; y < SZ; y++) for (let x = 0; x < SZ; x++) {
      const nx = (x - cx0) / R, ny = (y - cy0) / R, rr = nx * nx + ny * ny; if (rr > 1) continue;
      const nz = Math.sqrt(1 - rr), lon = Math.atan2(nx, nz) + .6, lat = Math.asin(ny);
      const u = fr(lon / (2 * Math.PI)), v = lat / Math.PI + .5, a = u * Math.PI * 2;
      const col = fn(u, v, Math.cos(a), Math.sin(a));
      const light = id === 'sun' ? 1 : Math.max(.12, nx * -.45 + ny * -.35 + nz * .82);
      const k = (y * SZ + x) * 4; img.data[k] = Math.min(255, col[0] * light); img.data[k + 1] = Math.min(255, col[1] * light); img.data[k + 2] = Math.min(255, col[2] * light);
      img.data[k + 3] = rr > .96 ? 255 * (1 - rr) / .04 : 255;
    }
    ctx.putImageData(img, 0, 0);
    if (id === 'sun') { ctx.globalCompositeOperation = 'destination-over'; const g = ctx.createRadialGradient(cx0, cy0, R * .9, cx0, cy0, R * 1.32); g.addColorStop(0, 'rgba(255,170,60,.9)'); g.addColorStop(1, 'rgba(255,120,0,0)'); ctx.fillStyle = g; ctx.beginPath(); ctx.arc(cx0, cy0, R * 1.32, 0, 7); ctx.fill(); ctx.globalCompositeOperation = 'source-over'; }
    if (id === 'saturn' || id === 'uranus') { ctx.save(); ctx.translate(cx0, cy0); ctx.rotate(id === 'saturn' ? -.35 : 1.2); ctx.strokeStyle = id === 'saturn' ? 'rgba(230,214,170,.95)' : 'rgba(190,240,245,.7)';
      ctx.lineWidth = id === 'saturn' ? 7 : 3; ctx.beginPath(); ctx.ellipse(0, 0, R * 1.32, R * .36, 0, Math.PI * 1.02, Math.PI * 1.98, true); ctx.stroke(); ctx.restore(); }
    return cv.toDataURL();
  }
  for (const id of GRAHA_ORDER) { try { ICON[id] = sphere(id); } catch (e) { ICON[id] = null; } }
})();
/* draw one graha marker into an SVG parent at (x, y) with radius r, in the selected style */
function marker(parent, id, x, y, r, opts = {}) {
  const g = s('g', { class: 'pl', 'data-body': id, transform: `translate(${x.toFixed(1)} ${y.toFixed(1)})` }, parent);
  const meta = G[id] || G.lagna;
  s('circle', { class: 'halo', r: r + 7, fill: 'none', stroke: 'var(--accent)', 'stroke-width': 3 }, g);
  const style = id === 'lagna' ? 'badge' : S.icons;
  if (style === 'sphere' && ICON[id]) {
    s('circle', { r: r + 1.5, fill: 'var(--space)', opacity: .85 }, g);
    s('image', { href: ICON[id], x: -r * 1.33, y: -r * 1.33, width: r * 2.66, height: r * 2.66 }, g);
    const t = s('text', { class: 'lbl', y: r + 11, 'font-size': Math.max(10, r * .62), 'font-weight': 800, fill: 'var(--ink)', stroke: 'var(--panel)', 'stroke-width': 3, 'paint-order': 'stroke' }, g);
    t.textContent = meta.abbr + (opts.retro ? 'ᴿ' : '');
  } else if (style === 'glyph') {
    s('circle', { r, fill: meta.dot, stroke: '#222', 'stroke-width': 1.4 }, g);
    const t = s('text', { class: 'lbl', 'font-size': r * 1.15, fill: ink(meta.dot) }, g); t.textContent = meta.glyph;
    if (opts.retro) { const rt = s('text', { class: 'lbl', x: r * .9, y: -r * .8, 'font-size': r * .7, 'font-weight': 800, fill: 'var(--accent)' }, g); rt.textContent = 'R'; }
  } else {
    s('circle', { r, fill: meta.dot, stroke: '#222', 'stroke-width': 1.4 }, g);
    const t = s('text', { class: 'lbl', 'font-size': r * .8, 'font-weight': 800, fill: ink(meta.dot) }, g); t.textContent = meta.abbr + (opts.retro ? 'ᴿ' : '');
  }
  return g;
}
