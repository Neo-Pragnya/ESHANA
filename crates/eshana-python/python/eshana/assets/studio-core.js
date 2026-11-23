/* ESHANA Studio, part 1: data model, helpers, settings, tooltip, planet icons, sigils, the living sky and the router.
   Every value shown comes from the engine envelopes embedded in the page (#eshana-data). The page itself computes
   only presentation (layout, colours) and the labelled marker rules (report.markers@0.1, report.nadi@0.1). */
'use strict';
const D = JSON.parse(document.getElementById('eshana-data').textContent);
const ENV = D.envelopes || {};
const NS = 'http://www.w3.org/2000/svg';
const $ = (q, r = document) => r.querySelector(q);
const $$ = (q, r = document) => Array.from(r.querySelectorAll(q));
function h(tag, attrs = {}, ...kids) {
  const e = document.createElement(tag);
  for (const k in attrs) {
    const v = attrs[k];
    if (v == null || v === false) continue;
    if (k === 'class') e.className = v; else if (k === 'html') e.innerHTML = v;
    else if (k.startsWith('on')) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v === true ? '' : v);
  }
  for (const c of kids.flat(Infinity)) if (c != null && c !== false) e.append(c.nodeType ? c : document.createTextNode(String(c)));
  return e;
}
function s(tag, attrs = {}, parent) {
  const e = document.createElementNS(NS, tag);
  for (const k in attrs) if (attrs[k] != null) e.setAttribute(k, attrs[k]);
  if (parent) parent.appendChild(e);
  return e;
}
function put(el, ...kids) { el.replaceChildren(...kids.flat(Infinity).filter(x => x != null && x !== false)); return el; }
const esc = x => String(x == null ? '' : x).replace(/[&<>"]/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
const DEG = Math.PI / 180;
const norm = x => ((x % 360) + 360) % 360;
function dms(x) { x = norm(x); let d = Math.floor(x), m = Math.floor((x - d) * 60), sec = Math.round(((x - d) * 60 - m) * 60);
  if (sec === 60) { m++; sec = 0; } if (m === 60) { d++; m = 0; } return d + '°' + String(m).padStart(2, '0') + '′' + String(sec).padStart(2, '0') + '″'; }
const dm = x => { x = norm(x); let d = Math.floor(x), m = Math.round((x - d) * 60); if (m === 60) { d++; m = 0; } return d + '°' + String(m).padStart(2, '0') + '′'; };
const inSign = x => dms(norm(x) % 30);
const inSignDM = x => dm(norm(x) % 30);
const fmtTime = iso => iso ? iso.replace('T', ' ').replace(/\.\d+/, '').replace(/([+-]\d\d:\d\d|Z)$/, ' $1') : '—';
const dateOnly = iso => iso ? iso.slice(0, 10) : '—';
const nth = n => { n = Number(n); const t = n % 100; return n + ((t > 10 && t < 14) ? 'th' : ({ 1: 'st', 2: 'nd', 3: 'rd' }[n % 10] || 'th')); };
const fix = (v, n = 2) => v == null ? '—' : Number(v).toFixed(n);

/* ── tradition names (labels only) ───────────────────────────────────────────────────────────────────────────── */
const SIGN_EN = ['Aries', 'Taurus', 'Gemini', 'Cancer', 'Leo', 'Virgo', 'Libra', 'Scorpio', 'Sagittarius', 'Capricorn', 'Aquarius', 'Pisces'];
const SIGN_SKT = ['Mesha', 'Vrishabha', 'Mithuna', 'Karka', 'Simha', 'Kanya', 'Tula', 'Vrischika', 'Dhanu', 'Makara', 'Kumbha', 'Meena'];
const SIGN_AB = ['Ar', 'Ta', 'Ge', 'Cn', 'Le', 'Vi', 'Li', 'Sc', 'Sg', 'Cp', 'Aq', 'Pi'];
const GLYPH = ['♈', '♉', '♊', '♋', '♌', '♍', '♎', '♏', '♐', '♑', '♒', '♓'].map(g => g + '\uFE0E');
const SIGN_LORD = ['mars', 'venus', 'mercury', 'moon', 'sun', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'saturn', 'jupiter'];
const ELEMENT = ['Fire', 'Earth', 'Air', 'Water'];
const MODALITY = ['Movable', 'Fixed', 'Dual'];
const NAKSHATRA = ['Ashwini', 'Bharani', 'Krittika', 'Rohini', 'Mrigashira', 'Ardra', 'Punarvasu', 'Pushya', 'Ashlesha', 'Magha',
  'Purva Phalguni', 'Uttara Phalguni', 'Hasta', 'Chitra', 'Swati', 'Vishakha', 'Anuradha', 'Jyeshtha', 'Mula', 'Purva Ashadha',
  'Uttara Ashadha', 'Shravana', 'Dhanishta', 'Shatabhisha', 'Purva Bhadrapada', 'Uttara Bhadrapada', 'Revati'];
const NAK_AB = ['Asw', 'Bha', 'Kri', 'Roh', 'Mrg', 'Ard', 'Pun', 'Pus', 'Asl', 'Mag', 'PPh', 'UPh', 'Has', 'Cht', 'Swa', 'Vis', 'Anu', 'Jye', 'Mul', 'PSh', 'USh', 'Shr', 'Dha', 'Sat', 'PBh', 'UBh', 'Rev'];
const NAK_SHORT = { 'Purva Phalguni': 'P. Phalguni', 'Uttara Phalguni': 'U. Phalguni', 'Purva Ashadha': 'P. Ashadha', 'Uttara Ashadha': 'U. Ashadha', 'Purva Bhadrapada': 'P. Bhadra', 'Uttara Bhadrapada': 'U. Bhadra' };
const NAK_LORD = ['ketu', 'venus', 'sun', 'moon', 'mars', 'rahu', 'jupiter', 'saturn', 'mercury'];
const NAKS = 40 / 3, NAV = 10 / 3;
const PADA_FILL = { Fire: '#FFE24D', Earth: '#A6EDD5', Air: '#C5EBA0', Water: '#6FE2EE' };
const G = {
  sun:     { abbr: 'Su', skt: 'Surya',   en: 'Sun',     glyph: '☉', dot: '#F39A1E', band: '#F4B31B', nak: '#F4B31B' },
  moon:    { abbr: 'Mo', skt: 'Chandra', en: 'Moon',    glyph: '☽', dot: '#E6E9F0', band: '#FFFFFF', nak: '#FFFFFF' },
  mars:    { abbr: 'Ma', skt: 'Mangala', en: 'Mars',    glyph: '♂', dot: '#E0314F', band: '#F2A6AD', nak: '#E0583B' },
  mercury: { abbr: 'Me', skt: 'Budha',   en: 'Mercury', glyph: '☿', dot: '#9BCB3B', band: '#CCDB57', nak: '#A9C79A' },
  jupiter: { abbr: 'Ju', skt: 'Guru',    en: 'Jupiter', glyph: '♃', dot: '#F7D21C', band: '#FFEE1E', nak: '#FFE21C' },
  venus:   { abbr: 'Ve', skt: 'Shukra',  en: 'Venus',   glyph: '♀', dot: '#38D6D6', band: '#52E5E2', nak: '#E39479' },
  saturn:  { abbr: 'Sa', skt: 'Shani',   en: 'Saturn',  glyph: '♄', dot: '#7E99C9', band: '#C8D5EA', nak: '#8DA2D0' },
  rahu:    { abbr: 'Ra', skt: 'Rahu',    en: 'Rahu',    glyph: '☊', dot: '#8B6FD8', band: '#B6A8DA', nak: '#B6A8DA' },
  ketu:    { abbr: 'Ke', skt: 'Ketu',    en: 'Ketu',    glyph: '☋', dot: '#A9A9A9', band: '#DCDCDC', nak: '#DCDCDC' },
  uranus:  { abbr: 'Ur', skt: 'Uranus',  en: 'Uranus',  glyph: '♅', dot: '#7FD3DC' },
  neptune: { abbr: 'Ne', skt: 'Neptune', en: 'Neptune', glyph: '♆', dot: '#4C7CF0' },
  pluto:   { abbr: 'Pl', skt: 'Pluto',   en: 'Pluto',   glyph: '♇', dot: '#CDB39A' },
  lagna:   { abbr: 'As', skt: 'Lagna',   en: 'Ascendant', glyph: 'Asc', dot: '#E0314F' },
};
/* upagrahas (aprakāśa and kāla upagrahas), shown like grahas but marked as points; colours are muted on purpose */
Object.assign(G, {
  dhuma:      { abbr: 'Dh', skt: 'Dhūma',     en: 'Dhūma',     glyph: 'Dh', dot: '#8D6E63', upa: true },
  vyatipata:  { abbr: 'Vy', skt: 'Vyatīpāta', en: 'Vyatīpāta', glyph: 'Vy', dot: '#78909C', upa: true },
  parivesha:  { abbr: 'Pv', skt: 'Pariveṣa',  en: 'Pariveṣa',  glyph: 'Pv', dot: '#A1887F', upa: true },
  indrachapa: { abbr: 'Cp', skt: 'Indracāpa', en: 'Indracāpa', glyph: 'Cp', dot: '#9575CD', upa: true },
  upaketu:    { abbr: 'Uk', skt: 'Upaketu',   en: 'Upaketu',   glyph: 'Uk', dot: '#90A4AE', upa: true },
  gulika:     { abbr: 'Gk', skt: 'Gulika',    en: 'Gulika',    glyph: 'Gk', dot: '#6D4C41', upa: true },
  mandi:      { abbr: 'Md', skt: 'Māndi',     en: 'Māndi',     glyph: 'Md', dot: '#5D4037', upa: true },
});
const OUTER = ['uranus', 'neptune', 'pluto'];
const UPAGRAHA = ['dhuma', 'vyatipata', 'parivesha', 'indrachapa', 'upaketu', 'gulika', 'mandi'];
const SEVEN = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn'];
const NINE = SEVEN.concat(['rahu', 'ketu']);
const GRAHA_ORDER = NINE.concat(OUTER, UPAGRAHA);
const engineId = id => ({ mean_node: 'rahu', true_node: 'rahu', south_node: 'ketu' }[id] || id);
const ASPECT_COLOUR = { conjunction: '#9a9a9a', opposition: '#e0564f', square: '#e0564f', trine: '#4f8cff', sextile: '#2fb36d' };
function ink(hex) { if (!/^#[0-9a-f]{6}$/i.test(hex)) return '#1b1b1b'; const n = parseInt(hex.slice(1), 16); const c = [n >> 16 & 255, n >> 8 & 255, n & 255].map(v => { v /= 255; return v <= .03928 ? v / 12.92 : Math.pow((v + .055) / 1.055, 2.4); });
  return .2126 * c[0] + .7152 * c[1] + .0722 * c[2] < .42 ? '#fff' : '#1b1b1b'; }

/* ── the chart model ──────────────────────────────────────────────────────────────────────────────────────────── */
function attrsOf(env) { const m = {}; for (const a of (env && env.derived_attributes) || []) m[a.id] = a; return m; }
const VED = ENV.vedic, WES = ENV.western, HD = ENV.human_design, NUM = ENV.numerology, MATCH = ENV.matching;
const VA = attrsOf(VED), WA = attrsOf(WES);
const val = id => VA[id] ? VA[id].value : null;
function grahasOf(env) {
  const out = {};
  for (const p of (env && env.positions) || []) {
    const id = engineId(p.id); if (!G[id]) continue;
    out[id] = { id, lon: p.longitude_deg, trop: p.tropical_longitude_deg, lat: p.latitude_deg, speed: p.speed_deg_per_day,
      retro: !!p.retrograde, dec: p.declination_deg, ra: p.right_ascension_deg, dist: p.distance_au };
  }
  return out;
}
const GR = grahasOf(VED);
const UPA = val('vedic.upagrahas') || {};
for (const u of UPAGRAHA) if (UPA[u] && UPA[u].longitude_deg != null) GR[u] = { id: u, lon: UPA[u].longitude_deg, retro: false, upa: true };
const GR_ALL = Object.assign({}, GR);
const ASC = VED && VED.houses && VED.houses.ascendant ? VED.houses.ascendant.longitude_deg : null;
const MC = VED && VED.houses && VED.houses.midheaven ? VED.houses.midheaven.longitude_deg : null;
const LAGNA_SIGN = ASC == null ? null : Math.floor(norm(ASC) / 30);
const signOf = l => Math.floor(norm(l) / 30);
const nakOf = l => Math.floor(norm(l) / NAKS);
const padaOf = l => Math.floor((norm(l) % NAKS) / NAV) + 1;
const navSignOf = l => Math.floor(norm(l) / NAV) % 12;
const dresSignOf = l => (signOf(l) + [0, 4, 8][Math.min(2, Math.floor((norm(l) % 30) / 10))]) % 12;
const houseOf = sg => LAGNA_SIGN == null ? null : (sg - LAGNA_SIGN + 12) % 12 + 1;
const lonOf = id => id === 'lagna' ? ASC : (GR[id] ? GR[id].lon : null);
const PUSHKARA = { Fire: [6, 8], Earth: [2, 4], Air: [5, 7], Water: [0, 2] };
const GANDANTA = [107, 0, 35, 36, 71, 72];
/* report.markers@0.2: classical points shown on the chakra's markers ring (fixed tables, labelled in the legend) */
const PUSHKARA_BHAGA = [21, 14, 18, 8, 19, 9, 24, 11, 23, 14, 19, 9]; // the pushkara degree in each sign, Mesha first
const UCCHA = { sun: [0, 10], moon: [1, 3], mars: [9, 28], mercury: [5, 15], jupiter: [3, 5], venus: [11, 27], saturn: [6, 20] }; // sign, degree of deep exaltation
const MOOLA = { sun: [4, 0, 20], moon: [1, 3, 30], mars: [0, 0, 12], mercury: [5, 15, 20], jupiter: [8, 0, 10], venus: [6, 0, 15], saturn: [10, 0, 20] }; // sign, from, to
function markers(l) { /* report.markers@0.1: fixed rules printed in the Panchang Chakra template */
  const i = Math.floor(norm(l) / NAV) % 108, r = Math.floor(i / 9), pos = i % 9, out = [];
  if (PUSHKARA[ELEMENT[r % 4]].includes(pos)) out.push('pushkara navamsha');
  if (i % 12 === r) out.push('vargottama');
  if (GANDANTA.includes(i)) out.push('gandanta');
  return out;
}
function vargaAttr(id, d) {
  const keys = id === 'rahu' ? ['mean_node', 'true_node'] : id === 'ketu' ? ['south_node'] : [id];
  for (const k of keys) { const a = VA[`vedic.${k}.d${d}`]; if (a) return a.value; }
  return null;
}
function vargaSign(id, d) {
  if (d === 1) return id === 'lagna' ? LAGNA_SIGN : (GR[id] ? signOf(GR[id].lon) : null);
  const a = vargaAttr(id, d); return a ? a.index : null;
}
function vargaDeg(id, d) {
  if (d === 1) { const l = lonOf(id); return l == null ? null : norm(l) % 30; }
  const a = vargaAttr(id, d); return a && a.degree_in_varga != null ? a.degree_in_varga : null;
}
const DIVS = (() => { const ds = new Set([1]); for (const k in VA) { const m = k.match(/\.d(\d+)$/); if (m) ds.add(+m[1]); } return [...ds].sort((a, b) => a - b); })();
const DIV_NAME = { 1: 'Rāśi', 2: 'Horā', 3: 'Drekkāṇa', 4: 'Chaturthāṃśa', 5: 'Pañcāṃśa', 6: 'Ṣaṣṭhāṃśa', 7: 'Saptāṃśa', 8: 'Aṣṭāṃśa',
  9: 'Navāṃśa', 10: 'Daśāṃśa', 12: 'Dvādaśāṃśa', 16: 'Ṣoḍaśāṃśa', 20: 'Viṃśāṃśa', 24: 'Chaturviṃśāṃśa', 27: 'Bhāṃśa', 30: 'Triṃśāṃśa', 40: 'Khavedāṃśa', 45: 'Akṣavedāṃśa', 60: 'Ṣaṣṭyāṃśa' };
const DIV_AREA = { 1: 'body, overall', 2: 'wealth', 3: 'siblings, courage', 4: 'property, fortune', 5: 'fame, authority', 6: 'health', 7: 'children', 8: 'longevity, sudden events',
  9: 'spouse, dharma', 10: 'career', 12: 'parents', 16: 'vehicles, comforts', 20: 'spiritual practice', 24: 'learning', 27: 'strength', 30: 'misfortune', 40: 'maternal legacy', 45: 'paternal legacy', 60: 'all areas, past karma' };
const STRENGTH = val('vedic.strength') || {};
const DRISHTI = val('vedic.drishti.graha') || [];
const DRISHTI_ALL = DRISHTI.slice();
/* additional aspects (retrograde shadow, parivartana) from the engine; shown only when the layer is switched on */
const ADDITIONAL_ALL = val('vedic.drishti.additional') || [];
const ADDITIONAL = ADDITIONAL_ALL.slice();
const EXTRA_MARK = { retrograde_shadow: 'ʳ', parivartana: '⇄' };
const EXTRA_NAME = { retrograde_shadow: 'retrograde shadow', parivartana: 'parivartana (exchange)' };
const SPECIAL = val('vedic.special_points') || {};
const YOGAS = val('vedic.yogas');
const NAKREF = D.nakshatra_reference || [];
const AE = D.energetics || null;
const REF_UTC = D.reference_utc;

/* ── settings (per viewer, kept in this browser only) ─────────────────────────────────────────────────────────── */
const DEFAULTS = Object.assign({
  icons: 'sphere', chart: 'south', varga: 1, orientation: 'template', centre: 'south', hd: 'standard', animate: true, sky: true, turnOnSelect: true, showOuter: true, showUpa: true,
  layers: { markers: true, nak: true, pada: true, dres: false, houses: true, ticks: true, needles: true, drishti: true, aspect_marks: true, rashi_marks: true, extra_aspects: false, extra_rashi: false, rashi_drishti: false,
    west_aspects: false, transits: false, sav: false, arudha: true, chart_drishti: false },
}, D.defaults || {});
let S = JSON.parse(JSON.stringify(DEFAULTS));
try { const saved = JSON.parse(localStorage.getItem('eshana-studio-v3') || 'null'); if (saved) S = Object.assign(S, saved, { layers: Object.assign(S.layers, saved.layers || {}) }); } catch (e) { /* private mode */ }
if (!S.hdStandardV1) { S.hd = 'standard'; S.hdStandardV1 = true; }
if (S.centre === 'nadi') S.centre = 'south'; // the Nadi directions now sit inside the South Indian centre
function save() { try { localStorage.setItem('eshana-studio-v3', JSON.stringify(S)); } catch (e) { /* ignore */ } }

/* ── tooltip ──────────────────────────────────────────────────────────────────────────────────────────────────── */
const TIP = h('div', { class: 'tip', role: 'tooltip' }); document.body.append(TIP);
/* a page can offer a docked place for hover details (.tipdock, e.g. beside the chakra) so the popup never covers
   what is being hovered; used on wide screens when the dock is in view, otherwise the floating popup is used */
function tipDock(ev) {
  if (!ev || !ev.target || !ev.target.closest) return null;
  const sec = ev.target.closest('section.page'), dock = sec && sec.querySelector('.tipdock');
  if (!dock) return null; const r = dock.getBoundingClientRect();
  // only when it sits beside what is hovered (to its right) and is in view
  return r.width && r.left > ev.clientX && r.bottom > 40 && r.top < innerHeight - 40 ? dock : null;
}
function tip(ev, title, rows, sub) {
  const html = `<h4>${esc(title)}</h4>${sub ? `<div class="note">${esc(sub)}</div>` : ''}<dl>${(rows || []).filter(r => r && r[1] != null && r[1] !== '')
    .map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>`;
  const dock = tipDock(ev);
  if (dock) { dock.querySelector('.tipdock-body').innerHTML = html; dock.classList.add('on'); TIP.classList.remove('on'); return; }
  TIP.innerHTML = html;
  const x = Math.min(ev.clientX + 14, innerWidth - TIP.offsetWidth - 8), y = Math.min(ev.clientY + 14, innerHeight - TIP.offsetHeight - 8);
  TIP.style.left = x + 'px'; TIP.style.top = y + 'px'; TIP.classList.add('on');
}
function untip() { TIP.classList.remove('on'); document.querySelectorAll('.tipdock.on').forEach(d => { d.classList.remove('on'); d.querySelector('.tipdock-body').innerHTML = d.dataset.idle || ''; }); }

/* ── planet icons: textured spheres, glyphs or badges ────────────────────────────────────────────────────────── */
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
    if (id === 'rahu' || id === 'ketu') {
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
function marker(parent, id, x, y, r, opts = {}) {
  const g = s('g', { class: 'pl', 'data-body': id, transform: `translate(${x.toFixed(1)} ${y.toFixed(1)})`, tabindex: 0, role: 'button', 'aria-label': (G[id] || G.lagna).en }, parent);
  const meta = G[id] || G.lagna;
  s('circle', { class: 'halo', r: r + 7, fill: 'none', stroke: 'var(--accent)', 'stroke-width': 3 }, g);
  const style = id === 'lagna' ? 'badge' : (opts.style || S.icons);
  const label = opts.label != null ? opts.label : meta.abbr + (opts.retro ? 'ᴿ' : '');
  if (style === 'sphere' && ICON[id]) {
    s('circle', { r: r + 1.5, fill: 'var(--space)', opacity: .85 }, g);
    s('image', { href: ICON[id], x: -r * 1.33, y: -r * 1.33, width: r * 2.66, height: r * 2.66 }, g);
    if (!opts.noLabel) { const t = s('text', { class: 'lbl', y: r + 10, 'font-size': Math.max(9, r * .62), 'font-weight': 800, fill: 'var(--ink)', stroke: 'var(--panel)', 'stroke-width': 3, 'paint-order': 'stroke' }, g); t.textContent = label; }
  } else if (style === 'glyph') {
    s('circle', { r, fill: meta.dot, stroke: 'rgba(0,0,0,.45)', 'stroke-width': 1.2 }, g);
    const t = s('text', { class: 'lbl', 'font-size': r * 1.15, fill: ink(meta.dot) }, g); t.textContent = meta.glyph;
    if (opts.retro) { const rt = s('text', { class: 'lbl', x: r * .9, y: -r * .8, 'font-size': r * .7, 'font-weight': 800, fill: 'var(--accent)' }, g); rt.textContent = 'R'; }
  } else {
    s('circle', { r, fill: meta.dot, stroke: 'rgba(0,0,0,.45)', 'stroke-width': 1.2 }, g);
    const t = s('text', { class: 'lbl', 'font-size': r * .8, 'font-weight': 800, fill: ink(meta.dot) }, g); t.textContent = label;
  }
  if (!opts.passive && id !== 'lagna' && GR[id]) {
    g.addEventListener('click', ev => { ev.stopPropagation(); go('#/planet/' + id); });
    g.addEventListener('keydown', ev => { if (ev.key === 'Enter') go('#/planet/' + id); });
  }
  return g;
}

/* ── ESHANA sigils: the studio's own animated icon set (orbits, rays, lattices, phases) ───────────────────────── */
const SIGIL = {
  overview: '<circle class="core" cx="12" cy="12" r="3.2"/><g class="spin"><ellipse cx="12" cy="12" rx="9.5" ry="4" class="ring"/><circle cx="21.5" cy="12" r="1.5" class="sat"/></g><g class="spin rev"><ellipse cx="12" cy="12" rx="4" ry="9.5" class="ring"/><circle cx="12" cy="2.5" r="1.2" class="sat"/></g>',
  chakra: '<circle cx="12" cy="12" r="9.5" class="ring"/><circle cx="12" cy="12" r="5" class="ring"/><g class="spin">' + Array.from({ length: 12 }, (_, i) => { const a = i * 30 * DEG; return `<line x1="${12 + 5 * Math.sin(a)}" y1="${12 - 5 * Math.cos(a)}" x2="${12 + 9.5 * Math.sin(a)}" y2="${12 - 9.5 * Math.cos(a)}" class="ray"/>`; }).join('') + '</g><circle cx="12" cy="12" r="1.6" class="core"/>',
  charts: '<rect x="3" y="3" width="18" height="18" rx="2" class="ring"/><path d="M12 3L21 12L12 21L3 12Z" class="ray draw"/><path d="M3 3L21 21M21 3L3 21" class="ray faint"/>',
  vargas: '<rect x="3" y="3" width="8" height="8" rx="1.5" class="ring"/><rect x="13" y="3" width="8" height="8" rx="1.5" class="ring pulse d1"/><rect x="3" y="13" width="8" height="8" rx="1.5" class="ring pulse d2"/><rect x="13" y="13" width="8" height="8" rx="1.5" class="ring pulse d3"/>',
  planets: '<circle cx="12" cy="12" r="5.5" class="core soft"/><ellipse cx="12" cy="12" rx="10" ry="3.4" class="ring tilt"/><circle cx="19" cy="6" r="1.1" class="star tw"/>',
  houses: '<path d="M3 11L12 3.5L21 11" class="ray draw"/><rect x="5.5" y="11" width="13" height="9.5" rx="1" class="ring"/><circle cx="12" cy="15.5" r="1.8" class="core"/>',
  aspects: '<circle cx="5.5" cy="17" r="2.4" class="core"/><circle cx="18.5" cy="7" r="2.4" class="core soft"/><path d="M7.5 15.5Q10 5 16.5 7.5" class="ray draw dash"/>',
  strength: '<path d="M4 20V14M9 20V9M14 20V11M19 20V4" class="ray grow"/><path d="M3 20.5H21" class="ring"/>',
  ashtakavarga: Array.from({ length: 9 }, (_, i) => `<circle cx="${5 + (i % 3) * 7}" cy="${5 + Math.floor(i / 3) * 7}" r="${[1.4, 2.3, 1.8, 2.6, 1.2, 2, 1.6, 2.4, 1.9][i]}" class="core tw d${i % 4}"/>`).join(''),
  dasha: '<path d="M3 18A9 9 0 0 1 21 18" class="ring"/><path d="M6 18A6 6 0 0 1 18 18" class="ring"/><path d="M12 18L17 9" class="ray hand"/><circle cx="12" cy="18" r="1.5" class="core"/>',
  panchanga: '<circle cx="12" cy="12" r="8.5" class="ring"/><path d="M12 3.5A8.5 8.5 0 0 1 12 20.5A5 8.5 0 0 0 12 3.5Z" class="core phase"/>',
  special: '<path d="M12 2.5L14.6 9.4L22 9.6L16.2 14.1L18.3 21.2L12 17L5.7 21.2L7.8 14.1L2 9.6L9.4 9.4Z" class="ring draw"/><circle cx="12" cy="12.5" r="2" class="core tw"/>',
  yogas: '<path d="M12 3L20.5 18H3.5Z" class="ring draw"/><path d="M12 21L3.5 6H20.5Z" class="ring draw d2"/><circle cx="12" cy="12" r="1.5" class="core"/>',
  kp: '<circle cx="12" cy="12" r="9.5" class="ring"/><path d="M12 2.5V21.5M2.5 12H21.5" class="ray faint"/><path d="M12 12L18.7 5.3" class="ray hand"/><circle cx="12" cy="12" r="2.2" class="core"/>',
  nadi: '<rect x="2.5" y="2.5" width="19" height="19" rx="1.5" class="ring"/><rect x="6" y="6" width="12" height="12" rx="1" class="ring"/><rect x="9.5" y="9.5" width="5" height="5" rx=".6" class="ring"/><path d="M12 12L12 9.5L14.5 9.5L14.5 18L6 18L6 6L21.5 6" class="ray draw"/>',
  energetics: '<path d="M12 21C7 16 5 12 12 3C19 12 17 16 12 21Z" class="ring draw"/><path d="M12 21C9 17 4 15 2.5 9C7 10 10 13 12 21ZM12 21C15 17 20 15 21.5 9C17 10 14 13 12 21Z" class="ring soft"/><circle cx="12" cy="13" r="1.6" class="core tw"/>',
  transits: '<circle cx="17" cy="7" r="2.6" class="core"/><path d="M15 9L4 20" class="ray draw tail"/><circle cx="7" cy="17" r=".9" class="star tw"/><circle cx="10" cy="14" r=".7" class="star tw d2"/>',
  progressions: '<path d="M12 12m0 0a2 2 0 1 1 2 2a4 4 0 1 1-4-4a6 6 0 1 1 6 6a8 8 0 1 1-8-8" class="ring draw"/>',
  western: '<circle cx="12" cy="12" r="9.5" class="ring"/><circle cx="12" cy="12" r="6" class="ring faint"/><path d="M3.5 12H20.5" class="ray"/><circle cx="3.5" cy="12" r="1.5" class="core"/>',
  human_design: '<path d="M12 2.5L15 7H9Z" class="ring"/><path d="M9 9H15L12 13Z" class="ring"/><rect x="9.5" y="14.5" width="5" height="4" class="ring"/><path d="M12 7V9M12 13V14.5" class="ray draw"/>',
  numerology: '<text x="12" y="16.5" class="num">9</text><circle cx="12" cy="12" r="9.5" class="ring dash spin"/>',
  matching: '<circle cx="9" cy="12" r="6" class="ring"/><circle cx="15" cy="12" r="6" class="ring soft"/><path d="M12 7.5A6 6 0 0 1 12 16.5A6 6 0 0 1 12 7.5Z" class="core"/>',
  data: '<path d="M6 3H15L19 7V21H6Z" class="ring"/><path d="M9 11H16M9 14H16M9 17H13" class="ray draw"/>',
};
const sigil = (name, cls = '') => `<svg class="sigil ${cls}" viewBox="0 0 24 24" aria-hidden="true">${SIGIL[name] || SIGIL.overview}</svg>`;

/* ── the living sky: depth-layered stars, twinkle, parallax, a cursor-woven constellation, shooting stars ───────── */
function startSky() {
  const cv = $('#sky'); if (!cv) return;
  const ctx = cv.getContext('2d'); let W = 0, H = 0, DPR = 1;
  const still = matchMedia('(prefers-reduced-motion: reduce)').matches || !S.sky;
  let seed = 7; const rnd = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  const stars = Array.from({ length: 320 }, () => ({ x: rnd(), y: rnd(), z: rnd() * rnd(), r: .3 + rnd() * 1.3, p: rnd() * 6.28, hue: rnd() < .12 ? 35 : rnd() < .2 ? 220 : 0 }));
  const meteors = []; const mouse = { x: -1e4, y: -1e4, tx: .5, ty: .5 }; let par = { x: 0, y: 0 };
  function size() { DPR = Math.min(2, devicePixelRatio || 1); W = innerWidth; H = innerHeight; cv.width = W * DPR; cv.height = H * DPR; ctx.setTransform(DPR, 0, 0, DPR, 0, 0); }
  size(); addEventListener('resize', size);
  addEventListener('pointermove', e => { mouse.x = e.clientX; mouse.y = e.clientY; mouse.tx = e.clientX / W; mouse.ty = e.clientY / H; }, { passive: true });
  function frame(t) {
    const light = document.documentElement.dataset.mode === 'light';
    ctx.clearRect(0, 0, W, H);
    const neb = ctx.createRadialGradient(W * .78, H * .18, 0, W * .78, H * .18, Math.max(W, H) * .7);
    neb.addColorStop(0, light ? 'rgba(242,163,58,.07)' : 'rgba(120,90,255,.16)'); neb.addColorStop(.5, light ? 'rgba(120,90,255,.04)' : 'rgba(40,80,200,.06)'); neb.addColorStop(1, 'rgba(0,0,0,0)');
    ctx.fillStyle = neb; ctx.fillRect(0, 0, W, H);
    par.x += ((mouse.tx - .5) - par.x) * .03; par.y += ((mouse.ty - .5) - par.y) * .03;
    const near = [];
    for (const st of stars) {
      const x = (st.x * W + par.x * 40 * (st.z + .2) + t * .004 * (st.z + .05)) % W, y = st.y * H + par.y * 30 * (st.z + .2);
      const tw = still ? .8 : .55 + .45 * Math.sin(t * .0016 * (1 + st.z * 2) + st.p);
      const a = (light ? .35 : .9) * tw * (.35 + st.z);
      ctx.fillStyle = st.hue ? `hsla(${st.hue},90%,${light ? 40 : 75}%,${a})` : (light ? `rgba(40,40,80,${a})` : `rgba(255,255,255,${a})`);
      ctx.beginPath(); ctx.arc(x, y, st.r * (.6 + st.z), 0, 6.283); ctx.fill();
      if (Math.hypot(x - mouse.x, y - mouse.y) < 140) near.push([x, y]);
    }
    ctx.strokeStyle = light ? 'rgba(120,90,255,.25)' : 'rgba(242,190,110,.35)'; ctx.lineWidth = .7;
    for (let i = 0; i < near.length; i++) for (let j = i + 1; j < near.length; j++) {
      const d = Math.hypot(near[i][0] - near[j][0], near[i][1] - near[j][1]); if (d < 70) { ctx.globalAlpha = 1 - d / 70; ctx.beginPath(); ctx.moveTo(...near[i]); ctx.lineTo(...near[j]); ctx.stroke(); }
    }
    ctx.globalAlpha = 1;
    if (!still && rnd() < .006 && meteors.length < 2) meteors.push({ x: rnd() * W, y: rnd() * H * .4, vx: 6 + rnd() * 5, vy: 2 + rnd() * 2, life: 1 });
    for (const m of meteors) {
      const g = ctx.createLinearGradient(m.x, m.y, m.x - m.vx * 12, m.y - m.vy * 12); g.addColorStop(0, `rgba(255,236,200,${m.life})`); g.addColorStop(1, 'rgba(255,236,200,0)');
      ctx.strokeStyle = g; ctx.lineWidth = 1.6; ctx.beginPath(); ctx.moveTo(m.x, m.y); ctx.lineTo(m.x - m.vx * 12, m.y - m.vy * 12); ctx.stroke();
      m.x += m.vx; m.y += m.vy; m.life -= .012;
    }
    while (meteors.length && meteors[0].life <= 0) meteors.shift();
    if (!still && !document.hidden) requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
  document.addEventListener('visibilitychange', () => { if (!document.hidden && !still) requestAnimationFrame(frame); });
}

/* ── router: one page at a time, animated in; deep links like #/planet/moon or #/house/7 ──────────────────────── */
const PAGES = {};
let CURRENT = null;
function page(id, title, group, render, opts = {}) { PAGES[id] = { id, title, group, render, ...opts }; }
function go(hash) { if (location.hash === hash) route(); else location.hash = hash; }
function route() {
  const [, id, arg] = (location.hash.match(/^#\/([\w-]+)(?:\/([\w-]+))?/) || [null, 'overview', null]);
  const key = PAGES[id] ? id : 'overview';
  const p = PAGES[key];
  const sec = document.getElementById(p.section || key);
  if (!sec) return;
  $$('section.page').forEach(x => { x.hidden = x !== sec; x.classList.remove('enter'); });
  $$('nav.side a').forEach(a => a.classList.toggle('active', a.dataset.page === (p.section || key)));
  const body = sec.querySelector('.body');
  try { p.render(body, arg); } catch (e) { console.error(key, e); put(body, h('p', { class: 'note' }, 'This view could not be drawn: ' + e.message)); }
  void sec.offsetWidth; sec.classList.add('enter');
  $$('.card, .stat, .chip', sec).forEach((el, i) => el.style.setProperty('--i', Math.min(i, 24)));
  CURRENT = key; document.title = `${p.title(arg)} · ${D.title || 'ESHANA'}`;
  window.scrollTo({ top: 0, behavior: S.animate ? 'smooth' : 'auto' });
  untip();
}
addEventListener('hashchange', route);

/* what the charts show: Uranus, Neptune and Pluto, and the upagrahas, can be switched off everywhere at once.
   GR and DRISHTI are rebuilt in place from the full sets, so every page follows the same choice. */
function applyVisibility() {
  for (const k of Object.keys(GR)) delete GR[k];
  for (const [k, v] of Object.entries(GR_ALL)) {
    if (S.showOuter === false && OUTER.includes(k)) continue;
    if (S.showUpa === false && UPAGRAHA.includes(k)) continue;
    GR[k] = v;
  }
  DRISHTI.length = 0; for (const d of DRISHTI_ALL) if (GR[d.from]) DRISHTI.push(d);
  ADDITIONAL.length = 0; for (const d of ADDITIONAL_ALL) if (GR[d.from]) ADDITIONAL.push(d);
  if (typeof rebuildAspected === 'function') rebuildAspected();
}
