/* ESHANA Studio, part 2: charts. The Panchang Chakra (with a South Indian, North Indian or orbit-lane centre), South
   and North Indian charts for any varga with degrees, nakshatras, flags and translucent aspect marks, the Sudarshana
   chakra, the Nadi direction matrix, the Western wheel and the BodyGraph. */

const RINGS_LANES = { mark: [552, 574], tick: [532, 548], nak: [472, 532], pada: [436, 472], dres: [404, 436], rashi: [330, 404], house: [292, 330], lanes: [252, 210, 168, 126] };
// with a chart in the centre the rings are slimmer, so the inner disc (and the boxes inside it) can be large enough to read
const RINGS_CENTRE = { mark: [554, 576], tick: [536, 550], nak: [494, 536], pada: [466, 494], dres: [442, 466], rashi: [398, 442], house: [374, 398], lanes: [252, 210, 168, 126] };
let RINGS = RINGS_LANES, WHEEL_SEQ = 0;
function angOf(lon) { return S.orientation === 'lagna' && ASC != null ? norm(lon - ASC) : norm(lon - 30); }
function pt(r, a) { return [r * Math.sin(a * DEG), -r * Math.cos(a * DEG)]; }
function arc(r0, r1, a0, a1) {
  const L = (a1 - a0) > 180 ? 1 : 0, [x0, y0] = pt(r1, a0), [x1, y1] = pt(r1, a1), [x2, y2] = pt(r0, a1), [x3, y3] = pt(r0, a0);
  return `M${x0.toFixed(2)} ${y0.toFixed(2)}A${r1} ${r1} 0 ${L} 1 ${x1.toFixed(2)} ${y1.toFixed(2)}L${x2.toFixed(2)} ${y2.toFixed(2)}A${r0} ${r0} 0 ${L} 0 ${x3.toFixed(2)} ${y3.toFixed(2)}Z`;
}
const grahasIn = (lo, hi) => GRAHA_ORDER.filter(g => GR[g] && norm(GR[g].lon - lo) < hi - lo);

/* flags shown next to a graha in the charts: R retrograde, C combust, ↑ exalted, ↓ debilitated, ◎ own/moolatrikona, V vargottama */
function flags(id, d = 1) {
  const st = STRENGTH[id] || {}, out = [];
  if (GR[id] && GR[id].retro && !['rahu', 'ketu'].includes(id)) out.push('R');
  if (st.combust) out.push('C');
  if (d === 1) { if (st.dignity === 'exalted') out.push('↑'); else if (st.dignity === 'debilitated') out.push('↓'); else if (st.dignity === 'own' || st.dignity === 'moolatrikona') out.push('◎'); }
  if (GR[id] && vargaSign(id, 9) === signOf(GR[id].lon)) out.push('V');
  return out.join('');
}
/* which grahas aspect each sign (graha drishti, from the engine) */
const ASPECTED = {};
function rebuildAspected() { for (const k of Object.keys(ASPECTED)) delete ASPECTED[k]; for (const d of DRISHTI) (ASPECTED[d.sign] = ASPECTED[d.sign] || []).push(d); }
rebuildAspected();
/* a block of marks inside a chart cell: wrapped to maxW (when given) and anchored at the baseline of its last line,
   so it grows upwards from the bottom of the cell (or at the baseline of its first line when down is set, so it grows
   downwards). Returns the height it takes (0 when there is nothing to show). */
/* parivartana: each graha of an exchange is also read as standing in its partner's sign; shown (with the
   additional aspects) as the graha's abbreviation with ⇄ among that sign's marks */
function exchangesIn(sign) {
  if (!S.layers.extra_aspects) return [];
  const seen = new Set();
  return ADDITIONAL.filter(d => d.basis === 'parivartana' && d.from_sign === sign && G[d.from] && !seen.has(d.from) && seen.add(d.from));
}
function wrapMarks(items, size, maxW) {
  const cw = size * .62, lines = [[]]; let w = 0;
  items.forEach(it => { const iw = (it.text.length + 1) * cw; if (maxW && w + iw > maxW && lines[lines.length - 1].length) { lines.push([]); w = 0; } lines[lines.length - 1].push(it); w += iw; });
  return lines;
}
function markBlock(parent, items, x, y, anchor, size, cls, title, maxW, down) {
  const lines = wrapMarks(items, size, maxW);
  const lh = size * 1.2, t = s('text', { x, y: down ? y : y - (lines.length - 1) * lh, 'text-anchor': anchor, 'font-size': size, class: cls }, parent);
  lines.forEach((ln, li) => ln.forEach((it, k) => {
    const a = { fill: it.fill, 'font-weight': it.weight || 600 }; if (it.italic) a['font-style'] = 'italic'; if (it.cls) a.class = it.cls; if (k === 0 && li > 0) { a.x = x; a.dy = lh; }
    const ts = s('tspan', a, t); ts.textContent = (k && !ln[k - 1].lead ? ' ' : '') + it.text;
  }));
  const tt = s('title', {}, t); tt.textContent = title;
  return lines.length * lh;
}
function aspectItems(sign) {
  if (!S.layers.aspect_marks) return [];
  const list = (ASPECTED[sign] || []).filter(d => G[d.from]);
  const extra = S.layers.extra_aspects ? ADDITIONAL.filter(d => d.drishti === 'graha' && d.sign === sign && G[d.from]) : [];
  return exchangesIn(sign).map(d => ({ text: G[d.from].abbr + '⇄' })).concat(list.map(d => ({ text: G[d.from].abbr + (d.kind === 'special' ? '*' : '') })),
    extra.map(d => ({ text: G[d.from].abbr + EXTRA_MARK[d.basis] })));
}
function rashiItems(sign) {
  if (!S.layers.rashi_marks) return [];
  const list = (RASHI_ASPECTED[sign] || []).filter(r => G[r.from] && GR[r.from]);
  const extra = S.layers.extra_aspects ? ADDITIONAL.filter(d => d.drishti === 'rashi' && d.sign === sign && G[d.from]) : [];
  return list.length || extra.length ? [{ text: '◇', lead: true }].concat(list.map(r => ({ text: G[r.from].abbr })), extra.map(d => ({ text: G[d.from].abbr + EXTRA_MARK[d.basis] }))) : [];
}
function aspectMarks(parent, sign, x, y, anchor = 'start', size = 10, maxW = 0, down = false) {
  if (!S.layers.aspect_marks) return 0;
  const list = (ASPECTED[sign] || []).filter(d => G[d.from]);
  const extra = S.layers.extra_aspects ? ADDITIONAL.filter(d => d.drishti === 'graha' && d.sign === sign && G[d.from]) : [];
  const ex = exchangesIn(sign);
  if (!list.length && !extra.length && !ex.length) return 0;
  const items = ex.map(d => ({ text: G[d.from].abbr + '⇄', fill: G[d.from].dot, weight: 800, cls: 'exchmark' }))
    .concat(list.map(d => ({ text: G[d.from].abbr + (d.kind === 'special' ? '*' : ''), fill: G[d.from].dot, weight: d.kind === 'special' ? 800 : 600 })),
      extra.map(d => ({ text: G[d.from].abbr + EXTRA_MARK[d.basis], fill: G[d.from].dot, cls: 'extramark' })));
  const parts = ex.map(d => `${G[d.from].en} is here by exchange (parivartana with ${G[d.partner] ? G[d.partner].en : d.partner})`)
    .concat(list.length ? ['Aspected by ' + list.map(d => `${G[d.from].en} (${nth(d.house_from_graha)}${d.kind === 'special' ? ', special' : ''})`).join(', ')] : [],
      extra.map(d => `${G[d.from].en} (${nth(d.house_from_graha)} from ${SIGN_SKT[d.from_sign]}, ${EXTRA_NAME[d.basis]})`));
  return markBlock(parent, items, x, y, anchor, size, 'aspmark', parts.join('; '), maxW, down);
}

/* which grahas see each sign by rāśi drishti (sign aspect, from the engine), shown as ◇ marks */
const RASHI_ASPECTED = (() => { const m = {}; for (const r of ((val('vedic.drishti.rashi') || {}).aspects || [])) for (const t of r.aspects_signs) (m[t] = m[t] || []).push(r); return m; })();
function rashiMarks(parent, sign, x, y, anchor = 'start', size = 10, maxW = 0, down = false) {
  if (!S.layers.rashi_marks) return 0;
  const list = (RASHI_ASPECTED[sign] || []).filter(r => G[r.from] && GR[r.from]);
  const extra = S.layers.extra_aspects ? ADDITIONAL.filter(d => d.drishti === 'rashi' && d.sign === sign && G[d.from]) : [];
  if (!list.length && !extra.length) return 0;
  const items = [{ text: '◇', fill: 'var(--muted)', lead: true }]
    .concat(list.map(r => ({ text: G[r.from].abbr, fill: G[r.from].dot, italic: true })))
    .concat(extra.map(d => ({ text: G[d.from].abbr + EXTRA_MARK[d.basis], fill: G[d.from].dot, italic: true, cls: 'extramark' })));
  return markBlock(parent, items, x, y, anchor, size, 'aspmark rashimark', 'Rāśi drishti from ' + list.map(r => `${G[r.from].en} in ${SIGN_SKT[r.sign]}`).join(', '), maxW, down);
}
/* what a graha sees: graha drishti (with the house counted from it) and rāśi drishti */
function aspectTargets(id) {
  const out = DRISHTI.filter(d => d.from === id).map(d => ({ sign: d.sign, kind: d.kind === 'special' ? 'special' : 'graha', label: nth(d.house_from_graha) }));
  for (const r of ((val('vedic.drishti.rashi') || {}).aspects || [])) if (r.from === id) for (const t of r.aspects_signs) out.push({ sign: t, kind: 'rashi', label: 'rāśi' });
  if (S.layers.extra_aspects) for (const d of ADDITIONAL) if (d.from === id) out.push({ sign: d.sign, kind: 'extra', label: (d.drishti === 'rashi' ? 'rāśi' : nth(d.house_from_graha)) + EXTRA_MARK[d.basis] });
  return out;
}
/* hovering a graha animates its aspects: flowing curves to every sign it sees, the signs lit, the boxes outlined */
function aspectFlow(id, svg) {
  svg = svg || (WHEEL && WHEEL.svg); if (!svg) return;
  const old = svg.querySelector(':scope > .aflow'); if (old) old.remove();
  $$('.cell.aspected, .cell.aspect-src', svg).forEach(c => c.classList.remove('aspected', 'aspected-rashi', 'aspected-graha', 'aspect-src'));
  if (!id || !GR[id]) { if (WHEEL && svg === WHEEL.svg) hot(FOCUS ? [FOCUS] : []); return; }
  const tg = aspectTargets(id), src = signOf(GR[id].lon);
  for (const t of tg) $$(`.cell[data-sign="${t.sign}"]`, svg).forEach(c => c.classList.add('aspected', t.kind === 'rashi' ? 'aspected-rashi' : 'aspected-graha'));
  $$(`.cell[data-sign="${src}"]`, svg).forEach(c => c.classList.add('aspect-src'));
  if (!(WHEEL && svg === WHEEL.svg && WHEEL.innerR)) return; // a plain chart: the outlined boxes are the animation
  hot(tg.map(t => ['rashi', t.sign]).concat([['rashi', src]]));
  const g = s('g', { class: 'aflow' }, svg);
  g.style.transform = WHEEL.rot.style.transform; g.style.transformOrigin = '0 0';
  const a0 = angOf(GR[id].lon), [x0, y0] = pt(WHEEL.pointR, a0);
  tg.forEach((t, k) => {
    const ta = angOf(t.sign * 30 + 15), [x1, y1] = pt(WHEEL.innerR + 18, ta), [cx, cy] = pt(WHEEL.innerR * .22, (a0 + ta) / 2 + (k % 2 ? 8 : -8));
    const d = `M${x0.toFixed(1)} ${y0.toFixed(1)}Q${cx.toFixed(1)} ${cy.toFixed(1)} ${x1.toFixed(1)} ${y1.toFixed(1)}`;
    s('path', { d, class: 'af ' + t.kind, stroke: G[id].dot }, g);
    const dot = s('circle', { r: t.kind === 'rashi' ? 5 : 7, fill: G[id].dot, class: 'af-dot' }, g);
    s('animateMotion', { dur: (1.1 + k * .15).toFixed(2) + 's', repeatCount: 'indefinite', path: d }, dot);
    s('path', { d: arc(WHEEL.innerR + 2, WHEEL.innerR + 34, angOf(t.sign * 30) + 1, angOf(t.sign * 30) + 29), class: 'af-target ' + t.kind, fill: G[id].dot }, g);
    const [lx, ly] = pt(WHEEL.innerR + 18, ta); const lt = s('text', { x: lx, y: ly, class: 'lbl af-lbl' }, g); lt.textContent = t.label;
  });
  s('circle', { cx: x0, cy: y0, r: 11, class: 'af-src', stroke: G[id].dot }, g);
}

/* ── detail strings shared by charts and pages ───────────────────────────────────────────────────────────────── */
function placeTip(ev, id, d = 1) {
  if (id === 'lagna') { tip(ev, 'Lagna', [['Sidereal', `${SIGN_SKT[signOf(ASC)]} ${inSign(ASC)}`], ['Nakshatra', `${NAKSHATRA[nakOf(ASC)]} pada ${padaOf(ASC)}`], ['Navāṃśa', SIGN_SKT[navSignOf(ASC)]]]); return; }
  const gr = GR[id]; if (!gr) return;
  const st = STRENGTH[id] || {};
  tip(ev, `${G[id].skt} · ${G[id].en}`, [['Sidereal', `${SIGN_SKT[signOf(gr.lon)]} ${inSign(gr.lon)}`], ['Nakshatra', `${NAKSHATRA[nakOf(gr.lon)]} pada ${padaOf(gr.lon)}`],
    ['Navāṃśa', SIGN_SKT[navSignOf(gr.lon)]], d !== 1 ? ['D' + d, `${SIGN_SKT[vargaSign(id, d)]} ${vargaDeg(id, d) != null ? dm(vargaDeg(id, d)) : ''}`] : null,
    ['Bhāva', houseOf(signOf(gr.lon))], ['Dignity', st.dignity], ['Motion', gr.retro ? 'retrograde' : 'direct']], 'click for everything about it');
}

/* ── the markers ring: notable points around the zodiac (report.markers@0.2) ─────────────────────────────────────
   outer lane: navāṃśa markers per pada (vargottama, pushkara navāṃśa, gaṇḍānta); inner lane: mūlatrikoṇa ranges and
   pushkara bhāga degrees; outside: each graha's uccha (▲) and nīca (▼) degree */
const MARK_COLOUR = { vargottama: '#3ED9E4', 'pushkara navamsha': '#6FAF1F', gandanta: '#D7141F', moola: '#a78bfa', bhaga: '#f5c451' };
function markerRing(parent) {
  const g = s('g', { class: 'markring' }, parent), [r0, r1] = RINGS.mark, mid = (r0 + r1) / 2;
  s('circle', { r: r1, fill: 'none', stroke: 'var(--line)' }, g); s('circle', { r: r0, fill: 'none', stroke: 'var(--line)' }, g);
  const seg = (a, b, ra, rb, colour, title, rows) => {
    const p = s('path', { d: arc(ra, rb, angOf(a), angOf(a) + (b - a)), fill: colour, class: 'mk-seg' }, g);
    p.addEventListener('mousemove', ev => tip(ev, title, rows)); p.addEventListener('mouseleave', untip);
    return p;
  };
  for (let i = 0; i < 108; i++) for (const m of markers(i * NAV + NAV / 2)) {
    const lo = i * NAV, inside = grahasIn(lo, lo + NAV).filter(x => G[x]);
    seg(lo + .15, lo + NAV - .15, mid + 1, r1 - 1, MARK_COLOUR[m], `${m[0].toUpperCase() + m.slice(1)} · ${NAKSHATRA[Math.floor(i / 4)]} pada ${i % 4 + 1}`,
      [['Span', `${SIGN_SKT[Math.floor(lo / 30)]} ${inSignDM(lo)} – ${inSignDM(lo + NAV - 1e-9)}`], ['Navāṃśa', SIGN_SKT[i % 12]], ['Grahas here', inside.map(x => G[x].en).join(', ') || 'none']]);
  }
  for (const [gid, [sg, a, b]] of Object.entries(MOOLA))
    seg(sg * 30 + a, sg * 30 + b, r0 + 1, mid - 1, MARK_COLOUR.moola, `Mūlatrikoṇa of ${G[gid].en}`, [['Range', `${SIGN_SKT[sg]} ${a}° – ${b}°`], [G[gid].en + ' here', GR[gid] && signOf(GR[gid].lon) === sg && norm(GR[gid].lon) % 30 >= a && norm(GR[gid].lon) % 30 < b ? 'yes' : 'no']]);
  PUSHKARA_BHAGA.forEach((d, sg) => seg(sg * 30 + d - .5, sg * 30 + d + .5, r0 + 1, mid - 1, MARK_COLOUR.bhaga, `Pushkara bhāga · ${SIGN_SKT[sg]} ${d}°`,
    [['Grahas within 1°', GRAHA_ORDER.filter(x => GR[x] && Math.abs(((norm(GR[x].lon) - (sg * 30 + d) + 540) % 360) - 180) <= 1).map(x => G[x].en).join(', ') || 'none']]));
  for (const [gid, [sg, d]] of Object.entries(UCCHA)) for (const [kind, off, glyph] of [['uccha (deep exaltation)', 0, '▲'], ['nīca (deep debilitation)', 180, '▼']]) {
    const lon = norm(sg * 30 + d + off), [x, y] = pt(r1 + 13, angOf(lon));
    const t = s('text', { x, y, class: 'lbl mk-pt', fill: G[gid].dot, 'font-size': 13 }, g); t.textContent = glyph;
    const lab = s('text', { x: pt(r1 + 27, angOf(lon))[0], y: pt(r1 + 27, angOf(lon))[1], class: 'lbl mk-pt-l', 'font-size': 9.5 }, g); lab.textContent = G[gid].abbr;
    const dist = GR[gid] ? Math.abs(((norm(GR[gid].lon) - lon + 540) % 360) - 180) : null;
    for (const el of [t, lab]) { el.addEventListener('mousemove', ev => tip(ev, `${G[gid].en}: ${kind}`, [['Point', `${SIGN_SKT[Math.floor(lon / 30)]} ${Math.round(lon % 30)}°`], ['Natal distance', dist != null ? fix(dist, 2) + '°' : null]])); el.addEventListener('mouseleave', untip); }
  }
}

/* ── the graha legend: dignity, motion, combustion, aspects, markers for every graha at a glance ───────────────── */
const DIGNITY_BADGE = { exalted: ['↑ uccha', 'b-up'], debilitated: ['↓ nīca', 'b-down'], moolatrikona: ['◎ mūlatrikoṇa', 'b-moola'], own: ['◎ own sign', 'b-own'] };
function grahaLegend() {
  const rel = val('vedic.relationships') || {}, sb = (val('vedic.shadbala') || {}).grahas || {};
  const relTo = (g) => { const lord = GR[g] ? SIGN_LORD[signOf(GR[g].lon)] : null; const x = lord && lord !== g && rel[g] ? rel[g][lord] : null; return x && x.combined ? x.combined.replace(/_/g, ' ') : null; };
  const rows = GRAHA_ORDER.filter(g => GR[g]).map(g => {
    const gr = GR[g], st = STRENGTH[g] || {}, d = DIGNITY_BADGE[st.dignity], mk = markers(gr.lon), asp = DRISHTI.filter(x => x.from === g);
    const badges = [d ? h('span', { class: 'badge ' + d[1] }, d[0]) : st.dignity ? h('span', { class: 'badge' }, st.dignity.replace(/_/g, ' ')) : relTo(g) ? h('span', { class: 'badge' }, 'in ' + relTo(g) + ' sign') : null,
      gr.retro && !['rahu', 'ketu'].includes(g) ? h('span', { class: 'badge b-retro' }, 'R retrograde') : null, st.combust ? h('span', { class: 'badge b-comb' }, 'C combust') : null,
      vargaSign(g, 9) === signOf(gr.lon) ? h('span', { class: 'badge b-varg' }, 'V vargottama') : null,
      ...mk.filter(m => m !== 'vargottama').map(m => h('span', { class: 'badge', style: `border-color:${MARK_COLOUR[m]}` }, m)),
      st.avastha ? h('span', { class: 'badge b-soft' }, st.avastha) : null];
    const speed = gr.speed != null ? `${fix(gr.speed, 3)}°/d${sb[g] && sb[g].chesta_state ? ' · ' + sb[g].chesta_state.replace(/_/g, ' ') : ''}` : '—';
    const extra = S.layers.extra_aspects ? ADDITIONAL.filter(x => x.from === g && x.drishti === 'graha') : [];
    const aspTxt = asp.map(x => nth(x.house_from_graha) + (x.kind === 'special' ? '*' : '')).join(' ') || '—';
    return [h('a', { href: '#/planet/' + g, class: 'glink' }, dot(g), h('b', {}, G[g].abbr), ' ', G[g].en), `${SIGN_AB[signOf(gr.lon)]} ${inSignDM(gr.lon)}`,
      h('span', { class: 'badges' }, badges.filter(Boolean)), speed,
      extra.length ? h('span', {}, aspTxt, h('span', { class: 'extramark' }, ' + ' + [...new Set(extra.map(x => x.basis))].map(b => `${EXTRA_MARK[b]} ${extra.filter(x => x.basis === b).map(x => SIGN_AB[x.sign]).join(' ')}`).join(' · '))) : aspTxt];
  });
  return h('div', { class: 'card legend-card' }, h('h3', {}, 'Grahas at a glance'), table(['Graha', 'Position', 'Dignity and notable points', 'Speed · motion', 'Aspects (houses from it)'], rows),
    h('div', { class: 'legend' }, h('span', {}, h('i', { style: `background:${MARK_COLOUR.vargottama}` }), 'vargottama'), h('span', {}, h('i', { style: `background:${MARK_COLOUR['pushkara navamsha']}` }), 'pushkara navāṃśa'),
      h('span', {}, h('i', { style: `background:${MARK_COLOUR.gandanta}` }), 'gaṇḍānta'), h('span', {}, h('i', { style: `background:${MARK_COLOUR.moola}` }), 'mūlatrikoṇa range'), h('span', {}, h('i', { style: `background:${MARK_COLOUR.bhaga}` }), 'pushkara bhāga'),
      h('span', {}, '▲ uccha · ▼ nīca points on the outer ring (report.markers@0.2)')));
}

/* ── the Panchang Chakra ──────────────────────────────────────────────────────────────────────────────────────── */
let WHEEL = null, transitIdx = (D.transits && D.transits.reference_index) || 0, playing = null, FOCUS = null;
function renderWheel(host, panel) {
  host.replaceChildren();
  RINGS = S.centre === 'lanes' ? RINGS_LANES : RINGS_CENTRE;
  WHEEL_SEQ++;
  const svg = s('svg', { viewBox: '-640 -640 1280 1280', role: 'img', 'aria-label': 'Graha-Nakshatra Mandala: the grahas placed by sidereal longitude', class: 'wheel' }, host);
  const defs = s('defs', {}, svg);
  const glow = s('radialGradient', { id: 'disc-glow' }, defs); s('stop', { offset: '0%', 'stop-color': 'var(--disc-hi)' }, glow); s('stop', { offset: '100%', 'stop-color': 'var(--disc)' }, glow);
  const m = s('marker', { id: 'arrow', viewBox: '0 0 10 10', refX: 8, refY: 5, markerWidth: 6, markerHeight: 6, orient: 'auto-start-reverse' }, defs); s('path', { d: 'M0 0L10 5L0 10z', fill: 'var(--muted)' }, m);
  const rot = s('g', { class: 'wheel-g' }, svg);
  const L = S.layers;
  const rashiOuter = L.dres ? RINGS.dres[0] : RINGS.pada[0];
  const handlers = (g, kind, i, title, rows) => {
    g.addEventListener('mousemove', ev => tip(ev, title, rows()));
    g.addEventListener('mouseleave', untip);
    g.addEventListener('click', () => focusSeg(kind, i, panel));
  };
  if (L.markers !== false) markerRing(rot);
  if (L.ticks) for (let d = 0; d < 360; d++) {
    const a = angOf(d), len = d % 10 === 0 ? 16 : d % 5 === 0 ? 10 : 5, [x0, y0] = pt(RINGS.tick[0], a), [x1, y1] = pt(RINGS.tick[0] + len, a);
    s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--muted)', 'stroke-width': d % 10 === 0 ? 1.4 : .7 }, rot);
  }
  if (L.nak) for (let i = 0; i < 27; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'nak', 'data-i': i }, rot), lord = NAK_LORD[i % 9], fill = G[lord].nak;
    s('path', { d: arc(RINGS.nak[0], RINGS.nak[1], angOf(i * NAKS), angOf(i * NAKS) + NAKS), fill }, g);
    // the name runs along the ring (upright on both halves), sized to fit its 13°20′ arc
    const rm = (RINGS.nak[0] + RINGS.nak[1]) / 2 - 1, a0 = angOf(i * NAKS) + .6, a1 = a0 + NAKS - 1.2, midA = norm((a0 + a1) / 2), up = midA > 90 && midA < 270;
    const [p0x, p0y] = pt(rm, up ? a1 : a0), [p1x, p1y] = pt(rm, up ? a0 : a1), pid = `nkp-${WHEEL_SEQ}-${i}`;
    s('path', { id: pid, d: `M${p0x.toFixed(2)} ${p0y.toFixed(2)}A${rm} ${rm} 0 0 ${up ? 0 : 1} ${p1x.toFixed(2)} ${p1y.toFixed(2)}`, fill: 'none' }, defs);
    const name = NAK_SHORT[NAKSHATRA[i]] || NAKSHATRA[i], arcLen = rm * (NAKS - 1.2) * DEG, fs = Math.max(9, Math.min(15, arcLen * .94 / (name.length * .56)));
    const t = s('text', { class: 'nkname', 'font-size': fs.toFixed(1), 'font-weight': 650, fill: ink(fill), 'dominant-baseline': 'central' }, g);
    const tp = s('textPath', { href: '#' + pid, startOffset: '50%', 'text-anchor': 'middle' }, t); tp.textContent = name;
    handlers(g, 'nak', i, NAKSHATRA[i], () => [['Lord', G[lord].en], ['Grahas', grahasIn(i * NAKS, (i + 1) * NAKS).map(x => `${G[x].en} p${padaOf(GR[x].lon)}`).join(', ') || '—'], ['Gana', (NAKREF[i] || {}).gana], ['Yoni', (NAKREF[i] || {}).yoni]]);
  }
  if (L.pada) for (let i = 0; i < 108; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'pada', 'data-i': i }, rot), nav = i % 12, fill = PADA_FILL[ELEMENT[nav % 4]];
    s('path', { d: arc(RINGS.pada[0], RINGS.pada[1], angOf(i * NAV), angOf(i * NAV) + NAV), fill }, g);
    const [x, y] = pt((RINGS.pada[0] + RINGS.pada[1]) / 2, angOf((i + .5) * NAV)); const t = s('text', { class: 'lbl', x, y, 'font-size': 13, fill: '#1b1b1b' }, g); t.textContent = GLYPH[nav];
    const mk = markers(i * NAV + NAV / 2);
    if (mk.length) { const [mx, my] = pt(RINGS.pada[0] + 4, angOf((i + .5) * NAV)); s('circle', { cx: mx, cy: my, r: 3, fill: mk.includes('gandanta') ? '#D7141F' : mk.includes('vargottama') ? '#3ED9E4' : '#6FAF1F' }, g); }
    handlers(g, 'pada', i, `${NAKSHATRA[Math.floor(i / 4)]} pada ${i % 4 + 1}`, () => [['Navāṃśa', SIGN_SKT[nav]], ['Markers', mk.join(', ') || '—'], ['Grahas', grahasIn(i * NAV, (i + 1) * NAV).map(x => `${G[x].en} ${inSignDM(GR[x].lon)}`).join(', ') || '—']]);
  }
  if (L.dres) for (let i = 0; i < 36; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'dres', 'data-i': i }, rot), sign = (Math.floor(i / 3) + [0, 4, 8][i % 3]) % 12, fill = G[SIGN_LORD[sign]].band;
    s('path', { d: arc(RINGS.dres[0], RINGS.dres[1], angOf(i * 10), angOf(i * 10) + 10), fill }, g);
    const [x, y] = pt((RINGS.dres[0] + RINGS.dres[1]) / 2, angOf(i * 10 + 5)); const t = s('text', { class: 'lbl', x, y, 'font-size': 12, fill: '#1b1b1b' }, g); t.textContent = GLYPH[sign];
    handlers(g, 'dres', i, `Dreshkāṇa ${i + 1}`, () => [['D3 sign', SIGN_SKT[sign]], ['Grahas', grahasIn(i * 10, i * 10 + 10).map(x => G[x].en).join(', ') || '—']]);
  }
  const sav = val('vedic.ashtakavarga');
  for (let i = 0; i < 12; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'rashi', 'data-i': i }, rot), fill = G[SIGN_LORD[i]].band;
    s('path', { d: arc(RINGS.rashi[0], rashiOuter, angOf(i * 30), angOf(i * 30) + 30), fill }, g);
    const wide = rashiOuter - RINGS.rashi[0] > 60;
    const [x, y] = pt(wide ? (RINGS.rashi[0] + rashiOuter) / 2 + 12 : rashiOuter - 15, angOf(i * 30 + 15)); const t = s('text', { class: 'lbl', x, y, 'font-size': wide ? 34 : 26, fill: '#1b1b1b' }, g); t.textContent = GLYPH[i];
    // the sign's name runs along the inner edge of the ring, upright on both halves
    { const rr = RINGS.rashi[0] + 11, b0 = angOf(i * 30) + 2, b1 = b0 + 26, mA = norm((b0 + b1) / 2), up = mA > 90 && mA < 270;
      const [q0x, q0y] = pt(rr, up ? b1 : b0), [q1x, q1y] = pt(rr, up ? b0 : b1), rid = `rsp-${WHEEL_SEQ}-${i}`;
      s('path', { id: rid, d: `M${q0x.toFixed(2)} ${q0y.toFixed(2)}A${rr} ${rr} 0 0 ${up ? 0 : 1} ${q1x.toFixed(2)} ${q1y.toFixed(2)}`, fill: 'none' }, defs);
      const t2 = s('text', { class: 'nkname', 'font-size': 13.5, 'font-weight': 750, fill: '#1b1b1b', 'dominant-baseline': 'central' }, g);
      const tp2 = s('textPath', { href: '#' + rid, startOffset: '50%', 'text-anchor': 'middle' }, t2); tp2.textContent = SIGN_SKT[i]; }
    handlers(g, 'rashi', i, `${SIGN_SKT[i]} · ${SIGN_EN[i]}`, () => [['Lord', G[SIGN_LORD[i]].en], ['Bhāva', houseOf(i)], ['Grahas', grahasIn(i * 30, i * 30 + 30).map(x => G[x].en).join(', ') || '—'], ['SAV', sav ? sav.sarvashtakavarga.bindus[i] : null]]);
    if (L.houses) {
      const hg = s('g', {}, rot), house = houseOf(i);
      let hf = 'var(--disc)';
      if (L.sav && sav) { const b = sav.sarvashtakavarga.bindus[i]; const t01 = Math.max(0, Math.min(1, (b - 18) / 20)); hf = `hsl(${Math.round(10 + t01 * 120)} 65% 55% / .55)`; }
      s('path', { d: arc(RINGS.house[0], RINGS.house[1], angOf(i * 30), angOf(i * 30) + 30), fill: hf, stroke: 'var(--line)' }, hg);
      const [hx, hy] = pt((RINGS.house[0] + RINGS.house[1]) / 2, angOf(i * 30 + 15));
      const ht = s('text', { class: 'lbl', x: hx, y: hy, 'font-size': 14, 'font-weight': 700, fill: house === 1 ? 'var(--accent)' : 'var(--muted)' }, hg);
      ht.textContent = (house ? `${house}` : '') + (L.sav && sav ? ` · ${sav.sarvashtakavarga.bindus[i]}` : '');
    }
  }
  const innerR = L.houses ? RINGS.house[0] : RINGS.rashi[0];
  s('circle', { r: innerR, fill: 'url(#disc-glow)', stroke: 'var(--line)' }, rot);
  const inner = s('g', {}, rot);
  // exact-degree points on the pada ring, always: the point where each graha falls
  const pointsG = s('g', {}, inner), pointR = rashiOuter + (L.pada ? 36 : 0);
  for (const g of GRAHA_ORDER) {
    if (!GR[g]) continue; const a = angOf(GR[g].lon), [tx, ty] = pt(pointR, a);
    s('circle', { cx: tx, cy: ty, r: 5, fill: G[g].dot, stroke: '#111', 'stroke-width': 1.2, class: 'pt' }, pointsG);
    const hit = s('circle', { cx: tx, cy: ty, r: 13, class: 'pt-hit', 'data-body': g }, pointsG);
    hit.addEventListener('mousemove', ev => placeTip(ev, g)); hit.addEventListener('mouseenter', () => aspectFlow(g));
    hit.addEventListener('mouseleave', () => { untip(); aspectFlow(null); }); hit.addEventListener('click', () => go('#/planet/' + g));
    if (L.needles) { const [x0, y0] = pt(innerR, a); s('line', { class: 'needle', x1: x0, y1: y0, x2: tx, y2: ty, stroke: G[g].dot }, pointsG); }
  }
  if (ASC != null) {
    const a = angOf(ASC), [x0, y0] = pt(S.centre === 'lanes' ? 96 : innerR, a), [x1, y1] = pt(RINGS.tick[1] + 12, a);
    s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--accent)', 'stroke-width': 4 }, inner);
  }
  if (S.centre === 'south' || S.centre === 'north') {
    const box = s('g', { class: 'centre' }, svg); // not rotated with focus: stays readable
    const side = innerR * Math.SQRT2 * .98, cell = side / 4, x0 = -side / 2, y0 = -side / 2;
    if (S.centre === 'south') southGrid(box, 1, x0, y0, cell, { compact: true });
    else northGrid(box, 1, x0, y0, side, { compact: true });
  } else {
    // orbit lanes with planet markers, plus drishti arcs
    const items = GRAHA_ORDER.filter(g => GR[g]).map(g => ({ g, a: angOf(GR[g].lon) })).sort((x, y) => x.a - y.a);
    const last = [null, null, null, null], pos = {};
    for (const it of items) { let lane = 0; for (; lane < 4; lane++) if (last[lane] == null || norm(it.a - last[lane]) >= 9) break; lane = Math.min(lane, 3); last[lane] = it.a; pos[it.g] = { a: it.a, r: RINGS.lanes[lane] }; }
    if (L.drishti) {
      const dg = s('g', { class: 'drishti' + (S.animate ? ' drawn' : '') }, inner);
      for (const d of DRISHTI) {
        const p = pos[d.from]; if (!p) continue;
        const [x0, y0] = pt(p.r - 18, p.a), ta = angOf(d.sign * 30 + 15), [x1, y1] = pt(innerR - 14, ta), [cx, cy] = pt(40, ta);
        const path = s('path', { d: `M${x0.toFixed(1)} ${y0.toFixed(1)}Q${cx.toFixed(1)} ${cy.toFixed(1)} ${x1.toFixed(1)} ${y1.toFixed(1)}`, stroke: G[d.from].dot, class: d.kind === 'special' ? 'special' : '', 'marker-end': 'url(#arrow)' }, dg);
        path.style.setProperty('--len', 900);
        path.addEventListener('mousemove', ev => tip(ev, `${G[d.from].en} aspects the ${nth(d.house_from_graha)}`, [['Sign', SIGN_SKT[d.sign]], ['Bhāva', d.aspects_bhava], ['Grahas there', d.aspects_grahas.map(x => G[x].en).join(', ') || '—'], ['Kind', d.kind]]));
        path.addEventListener('mouseleave', untip);
      }
    }
    if (L.aspect_marks) for (let i = 0; i < 12; i++) { const [x, y] = pt(innerR - 30, angOf(i * 30 + 15)); aspectMarks(inner, i, x, y, 'middle', 13); }
    if (L.rashi_marks) for (let i = 0; i < 12; i++) { const [x, y] = pt(innerR - 52, angOf(i * 30 + 15)); rashiMarks(inner, i, x, y, 'middle', 12); }
    const pg = s('g', {}, inner);
    for (const it of items) {
      const p = pos[it.g], gr = GR[it.g], [x, y] = pt(p.r, p.a);
      const mk = marker(pg, it.g, x, y, 21, { retro: gr.retro && !['rahu', 'ketu'].includes(it.g) });
      mk.dataset.a = p.a; mk.dataset.r = p.r;
      mk.addEventListener('mousemove', ev => placeTip(ev, it.g)); mk.addEventListener('mouseleave', () => { untip(); aspectFlow(null); });
      mk.addEventListener('mouseenter', () => { aspectFlow(it.g); hot([['nak', nakOf(gr.lon)], ['pada', Math.floor(norm(gr.lon) / NAV)], ['rashi', signOf(gr.lon)], ['dres', Math.floor(norm(gr.lon) / 10)]].concat(aspectTargets(it.g).map(t => ['rashi', t.sign]))); });
    }
    if (S.animate) animateIn(pg);
    const c1 = s('text', { class: 'lbl', y: -10, 'font-size': 24, 'font-weight': 800, fill: 'var(--ink)' }, svg); c1.textContent = 'Rāśi chakra';
  }
  if (L.rashi_drishti) {
    const rd = s('g', { class: 'drishti' }, inner);
    for (const r of ((val('vedic.drishti.rashi') || {}).aspects || [])) for (const t of r.aspects_signs) {
      const [x0, y0] = pt(innerR - 6, angOf(r.sign * 30 + 15)), [x1, y1] = pt(innerR - 6, angOf(t * 30 + 15));
      s('path', { d: `M${x0} ${y0}L${x1} ${y1}`, stroke: G[r.from].dot, 'stroke-width': 1.2, opacity: .45 }, rd);
    }
  }
  if (L.west_aspects && WES) {
    const wg = s('g', { class: 'asp' }, inner), wgr = grahasOf(WES), ayan = (VED.calculation || {}).ayanamsha_deg || 0;
    for (const a of WES.derived_attributes.filter(x => x.id.includes('.aspect.'))) {
      const [b1, b2] = a.derived_from.map(engineId); if (!wgr[b1] || !wgr[b2]) continue;
      const [x0, y0] = pt(innerR - 4, angOf(wgr[b1].lon - ayan)), [x1, y1] = pt(innerR - 4, angOf(wgr[b2].lon - ayan));
      const ln = s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: ASPECT_COLOUR[a.value.aspect] || '#999' }, wg);
      ln.addEventListener('mousemove', ev => tip(ev, `${G[b1].en} ${a.value.aspect} ${G[b2].en}`, [['Orb', a.value.orb_deg.toFixed(2) + '°'], ['Phase', a.value.applying ? 'applying' : 'separating']]));
      ln.addEventListener('mouseleave', untip);
    }
  }
  if (L.transits && D.transits && D.transits.rows && D.transits.rows.length) {
    s('circle', { r: 598, fill: 'none', stroke: 'var(--line)', 'stroke-dasharray': '3 6' }, rot);
    WHEEL = { svg, rot, tg: s('g', {}, rot), innerR, pointR }; drawTransit();
  } else WHEEL = { svg, rot, innerR, pointR };
}
function hot(list) {
  const svg = WHEEL && WHEEL.svg; if (!svg) return;
  $$('.seg.hot', svg).forEach(x => x.classList.remove('hot'));
  for (const [k, i] of list) { const e = svg.querySelector(`.seg[data-kind="${k}"][data-i="${i}"]`); if (e) e.classList.add('hot'); }
  svg.classList.toggle('dimmed', list.length > 0);
}
function animateIn(pg) {
  const els = $$('.pl', pg), t0 = performance.now(), dur = 1300;
  const target = els.map(e => ({ e, a: +e.dataset.a, r: +e.dataset.r }));
  (function step(t) {
    const k = Math.min(1, (t - t0) / dur), ease = 1 - Math.pow(1 - k, 3);
    for (const { e, a, r } of target) { if (isNaN(a)) continue; const [x, y] = pt(r * (.35 + .65 * ease), a * ease); e.setAttribute('transform', `translate(${x.toFixed(1)} ${y.toFixed(1)})`); }
    if (k < 1) requestAnimationFrame(step);
  })(t0);
}
function segmentDetail(kind, i) {
  const rows = (pairs) => facts(pairs);
  if (kind === 'rashi') {
    const inside = grahasIn(i * 30, i * 30 + 30);
    return h('div', {}, h('h3', {}, `${GLYPH[i]} ${SIGN_SKT[i]} · ${SIGN_EN[i]}`),
      rows([['Span', `${i * 30}° – ${i * 30 + 30}°`], ['Lord', G[SIGN_LORD[i]].en], ['Element · modality', `${ELEMENT[i % 4]} · ${MODALITY[i % 3]}`], ['Bhāva', houseOf(i)],
        ['SAV bindus', val('vedic.ashtakavarga') ? val('vedic.ashtakavarga').sarvashtakavarga.bindus[i] : null],
        ['Grahas', inside.map(g => `${G[g].en} ${inSignDM(GR[g].lon)}`).join(', ') || 'none'],
        ['Aspected by', (ASPECTED[i] || []).map(d => G[d.from].en).join(', ') || 'none'],
        ['Nakshatras', NAKSHATRA.filter((_, k) => k * NAKS < i * 30 + 30 && (k + 1) * NAKS > i * 30).join(', ')]]),
      houseOf(i) ? h('a', { href: `#/house/${houseOf(i)}`, class: 'btn' }, `Open bhāva ${houseOf(i)} →`) : null);
  }
  if (kind === 'nak') {
    const ref = NAKREF[i] || {}, inside = grahasIn(i * NAKS, (i + 1) * NAKS);
    return h('div', {}, h('h3', {}, `${NAKSHATRA[i]} (${i + 1})`),
      rows([['Span', `${SIGN_SKT[Math.floor(i * NAKS / 30)]} ${inSignDM(i * NAKS)} – ${SIGN_SKT[Math.floor(((i + 1) * NAKS - 1e-9) / 30)]} ${inSignDM((i + 1) * NAKS - 1e-9)}`],
        ['Lord', G[NAK_LORD[i % 9]].en], ['Padas → navāṃśa', [0, 1, 2, 3].map(p => `${p + 1}: ${SIGN_SKT[(i * 4 + p) % 12]}`).join(' · ')],
        ['Grahas', inside.map(g => `${G[g].en} pada ${padaOf(GR[g].lon)}`).join(', ') || 'none'],
        ...['nature', 'gana', 'yoni', 'gender', 'nadi', 'dosha', 'tattva', 'purpose', 'guna', 'direction'].map(k => [k[0].toUpperCase() + k.slice(1), ref[k]])]),
      h('p', { class: 'note' }, 'Attributes from the Nakshatra Ruler table (tradition_rule, shown for reference).'));
  }
  if (kind === 'pada') {
    const nk = Math.floor(i / 4), inside = grahasIn(i * NAV, (i + 1) * NAV);
    return h('div', {}, h('h3', {}, `${NAKSHATRA[nk]} pada ${i % 4 + 1}`),
      rows([['Span', `${SIGN_SKT[Math.floor(i * NAV / 30)]} ${inSignDM(i * NAV)} – ${inSignDM((i + 1) * NAV - 1e-9)}`], ['Navāṃśa sign', `${SIGN_SKT[i % 12]} (lord ${G[SIGN_LORD[i % 12]].en})`],
        ['Element', ELEMENT[(i % 12) % 4]], ['Markers', markers(i * NAV + NAV / 2).join(', ') || 'none'], ['Grahas', inside.map(g => `${G[g].en} ${inSign(GR[g].lon)}`).join(', ') || 'none']]));
  }
  const host = Math.floor(i / 3), part = i % 3, sign = (host + [0, 4, 8][part]) % 12;
  return h('div', {}, h('h3', {}, `Dreshkāṇa ${i + 1}`), rows([['Span', `${SIGN_SKT[host]} ${part * 10}° – ${part * 10 + 10}°`], ['D3 sign', `${SIGN_SKT[sign]} (lord ${G[SIGN_LORD[sign]].en})`], ['Grahas', grahasIn(i * 10, i * 10 + 10).map(g => G[g].en).join(', ') || 'none']]));
}
function focusSeg(kind, i, panel) {
  const span = { rashi: 30, nak: NAKS, pada: NAV, dres: 10 }[kind], mid = angOf(i * span + span / 2);
  FOCUS = [kind, i];
  if (WHEEL) WHEEL.rot.style.transform = S.turnOnSelect === false ? '' : `rotate(${-mid}deg)`;
  hot([[kind, i]]);
  const crumbs = [h('button', { onclick: () => resetFocus(panel) }, '⟲ Chakra')];
  if (kind !== 'rashi') { const r = Math.floor((kind === 'nak' ? i * NAKS : kind === 'pada' ? i * NAV : i * 10) / 30); crumbs.push(h('button', { onclick: () => focusSeg('rashi', r, panel) }, SIGN_SKT[r])); }
  if (kind === 'pada') crumbs.push(h('button', { onclick: () => focusSeg('nak', Math.floor(i / 4), panel) }, NAKSHATRA[Math.floor(i / 4)]));
  if (panel) put(panel, h('div', { class: 'crumbs' }, crumbs), segmentDetail(kind, i));
}
function resetFocus(panel) {
  FOCUS = null; if (WHEEL) WHEEL.rot.style.transform = ''; hot([]);
  if (panel) put(panel, h('p', { class: 'note' }, 'Hover any ring to see what falls there. Click a rāśi, nakshatra or pada to turn the wheel to it and drill in; click a graha to open its page. Esc returns.'));
}
document.addEventListener('keydown', e => { if (e.key === 'Escape' && FOCUS) resetFocus($('#chakra .panel')); });
function drawTransit() {
  const rows = D.transits.rows; if (!WHEEL || !WHEEL.tg) return;
  transitIdx = Math.max(0, Math.min(rows.length - 1, transitIdx));
  const row = rows[transitIdx]; WHEEL.tg.replaceChildren();
  // hovering a transit graha draws a highlighted line from it to the centre at the degree where it is now, and lights
  // the rāśi, nakshatra and pada it is passing through
  const rays = s('g', { class: 'transit-rays' }, WHEEL.tg);
  for (const k in row.longitudes) {
    const id = engineId(k); if (!G[id]) continue; const lon = row.longitudes[k], a = angOf(lon), [x, y] = pt(598, a);
    const mk = marker(WHEEL.tg, id, x, y, 15, { passive: true });
    mk.classList.add('transit-pl');
    mk.addEventListener('mousemove', ev => tip(ev, `Transit ${G[id].en} now`, [['Date', dateOnly(row.utc)], ['Sidereal', `${SIGN_SKT[signOf(lon)]} ${inSign(lon)}`],
      ['Nakshatra', `${NAKSHATRA[nakOf(lon)]} pada ${padaOf(lon)}`], ['Over natal bhāva', houseOf(signOf(lon))]]));
    mk.addEventListener('mouseenter', () => {
      rays.replaceChildren();
      const [x1, y1] = pt(580, a), [x2, y2] = pt(WHEEL.innerR * .25, a);
      s('line', { x1, y1, x2, y2, class: 'transit-ray', stroke: G[id].dot === '#E6E9F0' ? 'var(--ink)' : G[id].dot, 'data-body': id }, rays);
      const dotEnd = s('circle', { cx: x2, cy: y2, r: 5, class: 'transit-ray-end', fill: G[id].dot === '#E6E9F0' ? 'var(--ink)' : G[id].dot }, rays);
      dotEnd.setAttribute('data-body', id);
      hot([['rashi', signOf(lon)], ['nak', nakOf(lon)], ['pada', Math.floor(norm(lon) / NAV)]]);
    });
    mk.addEventListener('mouseleave', () => { untip(); rays.replaceChildren(); hot([]); });
  }
  const lab = $('#transit-date'); if (lab) lab.textContent = dateOnly(row.utc);
  const sl = $('#transit-slider'); if (sl) sl.value = transitIdx;
  const dp = $('#transit-pick'); if (dp && document.activeElement !== dp) dp.value = dateOnly(row.utc);
}
/* transit rows by date: the series is evenly spaced, so a date maps to the nearest row */
function transitIndexOf(dateStr) {
  const rows = D.transits.rows, t0 = Date.parse(rows[0].utc), t1 = Date.parse(rows[rows.length - 1].utc), t = Date.parse(dateStr + 'T00:00:00Z');
  if (isNaN(t)) return null;
  const est = Math.max(0, Math.min(rows.length - 1, Math.round((t - t0) / Math.max(1, t1 - t0) * (rows.length - 1))));
  // prefer the row that falls on the chosen calendar date (rows can sit at any time of day)
  for (const k of [est, est - 1, est + 1, est - 2, est + 2]) if (rows[k] && dateOnly(rows[k].utc) === dateStr) return k;
  return est;
}
/* the animation loops over [TRANSIT_RANGE.from, TRANSIT_RANGE.to] (row indices), `speed` rows per tick */
const TRANSIT_RANGE = { from: 0, to: null, speed: 1 };
function playTransit(btn) {
  if (playing) { cancelAnimationFrame(playing); playing = null; btn.textContent = '▶ Play'; return; }
  btn.textContent = '❚❚ Pause'; let last = 0;
  const n = D.transits.rows.length, lo = Math.max(0, TRANSIT_RANGE.from), hi = Math.min(n - 1, TRANSIT_RANGE.to == null ? n - 1 : TRANSIT_RANGE.to);
  if (transitIdx < lo || transitIdx > hi) transitIdx = lo;
  const loop = t => {
    if (t - last > 90) { last = t; transitIdx += TRANSIT_RANGE.speed; if (transitIdx > hi) transitIdx = lo; drawTransit(); }
    playing = requestAnimationFrame(loop);
  };
  playing = requestAnimationFrame(loop);
}
function stopTransit() { if (playing) { cancelAnimationFrame(playing); playing = null; } }
/* the transit controls: a date to jump to, a from/to range for the animation, speed, and the reference date */
function transitControls() {
  const rows = D.transits.rows, rg = D.transits.range || { start: dateOnly(rows[0].utc), end: dateOnly(rows[rows.length - 1].utc) };
  if (TRANSIT_RANGE.to == null) TRANSIT_RANGE.to = rows.length - 1;
  const lo = rg.start, hi = rg.end;
  const dateIn = (id, value, on) => h('input', { type: 'date', id, min: lo, max: hi, value, onchange: e => on(e.target.value) });
  const ref = D.transits.reference_index || 0;
  const play = h('button', { class: 'btn', id: 'transit-play', onclick: e => playTransit(e.target) }, '▶ Play');
  const speed = h('select', { id: 'transit-speed', onchange: e => { TRANSIT_RANGE.speed = +e.target.value; } },
    [[1, '1 step'], [7, '7 steps'], [30, '30 steps'], [90, '90 steps']].map(([v, t]) => h('option', { value: v, selected: TRANSIT_RANGE.speed === v }, `${t} / tick`)));
  return h('div', { class: 'transit-ctl' },
    h('div', { class: 'slider' }, play,
      h('input', { type: 'range', id: 'transit-slider', min: 0, max: rows.length - 1, value: transitIdx, oninput: e => { stopTransit(); play.textContent = '▶ Play'; transitIdx = +e.target.value; drawTransit(); } }),
      h('span', { id: 'transit-date', class: 'mono' })),
    h('div', { class: 'grp' }, h('b', {}, 'Date'), dateIn('transit-pick', dateOnly(rows[transitIdx].utc), v => { const i = transitIndexOf(v); if (i != null) { stopTransit(); play.textContent = '▶ Play'; transitIdx = i; drawTransit(); } }),
      h('button', { class: 'tab', onclick: () => { stopTransit(); play.textContent = '▶ Play'; transitIdx = ref; drawTransit(); } }, 'Reference date'),
      h('b', {}, 'Animate from'), dateIn('transit-from', dateOnly(rows[TRANSIT_RANGE.from].utc), v => { const i = transitIndexOf(v); if (i != null) TRANSIT_RANGE.from = Math.min(i, TRANSIT_RANGE.to); }),
      h('b', {}, 'to'), dateIn('transit-to', dateOnly(rows[TRANSIT_RANGE.to].utc), v => { const i = transitIndexOf(v); if (i != null) TRANSIT_RANGE.to = Math.max(i, TRANSIT_RANGE.from); }),
      speed),
    note(`Transit data in this file: ${lo} to ${hi}, one frame every ${fix(D.transits.range ? D.transits.range.step_days : 1, 2)} day(s). For other dates, generate the file with --transit-start and --transit-end.`));
}

/* ── South and North Indian charts ────────────────────────────────────────────────────────────────────────────── */
const SOUTH_CELL = { 11: [0, 0], 0: [1, 0], 1: [2, 0], 2: [3, 0], 3: [3, 1], 4: [3, 2], 5: [3, 3], 6: [2, 3], 7: [1, 3], 8: [0, 3], 9: [0, 2], 10: [0, 1] };
function placements(d) {
  const out = {}; for (const g of GRAHA_ORDER) { if (!GR[g]) continue; const sg = vargaSign(g, d); if (sg == null) continue; (out[sg] = out[sg] || []).push(g); }
  for (const k in out) out[k].sort((a, b) => (vargaDeg(a, d) || 0) - (vargaDeg(b, d) || 0));
  return out;
}
function arudhaMarks() {
  const a = val('vedic.jaimini.arudha_padas'); if (!a || !S.layers.arudha) return {};
  const out = {}; for (const k in a) { const label = k.startsWith('A1_') ? 'AL' : k.startsWith('A12') ? 'UL' : k.split('_')[0]; (out[a[k].pada_sign] = out[a[k].pada_sign] || []).push(label); }
  return out;
}
/* one graha line inside a chart cell: icon, abbreviation, degree, nakshatra abbreviation, flags */
function grahaLine(parent, id, x, y, d, opts) {
  const g = s('g', { class: 'gl', 'data-body': id }, parent);
  if (!opts.compact) marker(g, id, x + 8, y - 4, 6.3, { noLabel: true, passive: true });
  const deg = vargaDeg(id, d), lon = d === 1 ? lonOf(id) : null;
  const t = s('text', { x: x + (opts.compact ? 0 : 19), y, 'font-size': opts.fs || 12.5, 'font-weight': 700, fill: (G[id] || G.lagna).dot === '#E6E9F0' ? 'var(--ink)' : (G[id] || G.lagna).dot, class: 'gtxt' }, g);
  t.textContent = (G[id] || G.lagna).abbr;
  const f = flags(id, d); if (f) { const fs = s('tspan', { 'font-size': (opts.fs || 12.5) * .72, dy: -4, fill: 'var(--accent)' }, t); fs.textContent = f; }
  const t2 = s('tspan', { 'font-size': (opts.fs || 12.5) * .8, 'font-weight': 500, fill: 'var(--muted)', dy: f ? 4 : 0 }, t);
  t2.textContent = ' ' + (deg != null ? dm(deg) : '') + (lon != null && !opts.compact ? ' ' + NAK_AB[nakOf(lon)] : '');
  g.addEventListener('mousemove', ev => placeTip(ev, id, d));
  if (d === 1 && GR[id]) { g.addEventListener('mouseenter', () => aspectFlow(id, g.ownerSVGElement)); g.addEventListener('mouseleave', () => { untip(); aspectFlow(null, g.ownerSVGElement); }); }
  else g.addEventListener('mouseleave', untip);
  if (GR[id]) g.addEventListener('click', ev => { ev.stopPropagation(); go(UPAGRAHA.includes(id) ? '#/point/upagrahas' : '#/planet/' + id); });
  return g;
}
function southGrid(svg, d, x0, y0, W, opts = {}) {
  const H = W * (opts.compact ? 1 : .86), pl = placements(d), lag = opts.lagSign != null ? opts.lagSign : vargaSign('lagna', d), ar = d === 1 && opts.lagSign == null ? arudhaMarks() : {};
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg, x = x0 + c * W, y = y0 + r * H, g = s('g', { class: 'cell', 'data-sign': sign }, svg);
    s('rect', { x, y, width: W, height: H, class: 'cellbg' }, g);
    const small = opts.compact ? Math.max(10, W * .11) : 11;
    const nm = s('text', { x: x + W - 5, y: y + small + 2, 'text-anchor': 'end', 'font-size': small, fill: 'var(--muted)' }, g); nm.textContent = opts.compact ? SIGN_AB[sign] : SIGN_SKT[sign];
    if (lag != null) { const hs = s('text', { x: x + 5, y: y + small + 2, 'font-size': small, 'font-weight': 700, fill: 'var(--accent2)' }, g); hs.textContent = (sign - lag + 12) % 12 + 1; }
    if (lag === sign) s('path', { d: `M${x} ${y + 20}L${x + 20} ${y}`, stroke: 'var(--accent)', 'stroke-width': 2 }, g);
    const list = (vargaSign('lagna', d) === sign ? ['lagna'] : []).concat(pl[sign] || []);
    // ārūḍha padas sit at the top, under the house number and sign name
    const ps = opts.compact ? Math.max(9, small * .9) : 9.5, padaY = y + small + 2 + ps * 1.25;
    if (ar[sign]) { const at = s('text', { x: x + 6, y: padaY, 'font-size': ps, fill: '#a78bfa', 'font-weight': 700, class: 'arudha' }, g); at.textContent = ar[sign].join(' ');
      const tt = s('title', {}, at); tt.textContent = 'Ārūḍha padas in this sign: ' + ar[sign].join(', '); }
    // aspect marks at the bottom, wrapped to the cell: graha drishti lowest, rāśi drishti (◇) above it
    const am = opts.compact ? Math.max(9, W * .1) : 10.5, maxW = W - 12;
    let bottom = y + H - 6;
    if (d === 1) { const ah = aspectMarks(g, sign, x + 6, bottom, 'start', am, maxW); bottom -= ah; const rh = rashiMarks(g, sign, x + 6, bottom, 'start', am * .95, maxW); bottom -= rh; }
    // grahas fill the space between: shrink the line height (and drop the icon) when a sign is crowded
    let fs = opts.compact ? Math.max(10.5, Math.min(18, W * .15 * (list.length > 4 ? .8 : 1))) : 12.5, lineH = opts.compact ? fs * 1.22 : 18;
    const top = ar[sign] ? padaY + (opts.compact ? fs * 1.3 : 21) : y + (opts.compact ? small * 2.4 : 32), room = bottom - fs * .9 - top;
    let compact = opts.compact;
    if (list.length > 1 && (list.length - 1) * lineH > room) { lineH = Math.max(9, room / (list.length - 1)); if (lineH < 18) { compact = true; fs = Math.min(fs, lineH * .85); } }
    list.forEach((gid, k) => grahaLine(g, gid, x + 6, top + k * lineH, d, { compact, fs }));
    g.addEventListener('click', () => { const hh = houseOf(sign); if (hh && d === 1) go('#/house/' + hh); });
  }
  s('rect', { x: x0 + W, y: y0 + H, width: 2 * W, height: 2 * H, class: 'cellmid' }, svg);
  if (!opts.noTitle) {
    const t = s('text', { x: x0 + 2 * W, y: y0 + 2 * H - 4, 'text-anchor': 'middle', 'font-size': opts.compact ? Math.max(15, W * .17) : 18, 'font-weight': 800, fill: 'var(--ink)' }, svg); t.textContent = `D${d} · ${DIV_NAME[d] || ''}`;
    const t2 = s('text', { x: x0 + 2 * W, y: y0 + 2 * H + 16, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t2.textContent = opts.sub || (DIV_AREA[d] || '');
  }
}
function northGrid(svg, d, x0, y0, S0, opts = {}) {
  const lag = vargaSign('lagna', d);
  if (lag == null) { const t = s('text', { x: x0 + S0 / 2, y: y0 + S0 / 2, 'text-anchor': 'middle', fill: 'var(--muted)' }, svg); t.textContent = 'needs the lagna (exact time and place)'; return; }
  const P = (x, y) => [x0 + x * S0, y0 + y * S0];
  const A = P(0, 0), B = P(1, 0), C = P(1, 1), Dd = P(0, 1), T = P(.5, 0), R = P(1, .5), Bm = P(.5, 1), L = P(0, .5), O = P(.5, .5), P1 = P(.25, .25), P2 = P(.75, .25), P3 = P(.75, .75), P4 = P(.25, .75);
  const HS = [[T, P2, O, P1], [A, T, P1], [A, P1, L], [L, P1, O, P4], [L, P4, Dd], [Dd, P4, Bm], [Bm, P4, O, P3], [Bm, P3, C], [C, P3, R], [R, P3, O, P2], [R, P2, B], [B, P2, T]];
  const pl = placements(d), ar = d === 1 ? arudhaMarks() : {};
  s('rect', { x: x0, y: y0, width: S0, height: S0, class: 'cellbg' }, svg);
  HS.forEach((poly, hIdx) => {
    const sign = (lag + hIdx) % 12, cx = poly.reduce((a, p) => a + p[0], 0) / poly.length, cy = poly.reduce((a, p) => a + p[1], 0) / poly.length, big = poly.length === 4;
    const g = s('g', { class: 'cell', 'data-sign': sign, 'data-house': hIdx + 1 }, svg);
    s('polygon', { points: poly.map(p => p.join(',')).join(' '), class: hIdx === 0 ? 'cellbg lagna' : 'cellbg', 'stroke-width': 1.3 }, g);
    // one stack, top to bottom: ārūḍha padas, sign number, grahas, rāśi drishti (◇), graha drishti. It is measured first,
    // centred on the house, kept inside its polygon, and shrunk in steps when a crowded house cannot hold it.
    const list = (hIdx === 0 ? ['lagna'] : []).concat(pl[sign] || []), maxW = big ? S0 * .34 : S0 * .2;
    const ys = poly.map(p2 => p2[1]), minY = Math.min(...ys), maxY = Math.max(...ys);
    // horizontal span of the polygon at height y (the houses are triangles and diamonds)
    const spanAt = (y) => { const xs = []; poly.forEach((a, i) => { const b = poly[(i + 1) % poly.length]; if ((a[1] - y) * (b[1] - y) <= 0 && a[1] !== b[1]) xs.push(a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1])); });
      return xs.length ? [Math.min(...xs), Math.max(...xs)] : [cx, cx]; };
    const tw0 = big ? S0 * .1 : S0 * .06;
    const layout = (k) => {
      // a graha row's raised flags (ᴿ, ◎) sit a fixed 4 px up, so rows keep that allowance when the stack shrinks
      const nfs = (opts.compact ? 10 : 12) * k, fs = (opts.compact ? 9.5 : 11.5) * k, lineH = fs * 1.05 + 4, ps = 9 * k;
      const as = (opts.compact ? 8.5 : 10) * k, rs = (opts.compact ? 8 : 9.5) * k;
      const rl = d === 1 ? wrapMarks(rashiItems(sign), rs, maxW) : [[]], al = d === 1 ? wrapMarks(aspectItems(sign), as, maxW) : [[]];
      const rLines = rl[0].length ? rl.length : 0, aLines = al[0].length ? al.length : 0;
      const wOf = (items, size) => items.reduce((a, it) => a + (it.text.length + 1) * size * .62, 0);
      let y = 0; const at = {}, rows = []; // rows: [baseline, size, left, right] relative to cx, for the fit check
      if (ar[sign]) { at.pada = y + ps; const w = ar[sign].join(' ').length * ps * .62; rows.push([at.pada, ps, -w / 2, w / 2]); y = at.pada + 2 * k; }
      at.num = y + nfs; rows.push([at.num, nfs, -nfs * .7, nfs * .7]); y = at.num + 3 * k + 4;
      if (list.length) { at.g0 = y + fs; list.forEach((gid, i) => rows.push([at.g0 + i * lineH, fs, -tw0 * k, -tw0 * k + 9.5 * fs * .62])); y = at.g0 + (list.length - 1) * lineH + 3 * k; }
      if (rLines) { at.r = y + rs; rl.forEach((ln, i) => { const w = wOf(ln, rs); rows.push([at.r + i * rs * 1.2, rs, -w / 2, w / 2]); }); y = at.r + (rLines - 1) * rs * 1.2 + 3 * k; }
      if (aLines) { at.a = y + as; al.forEach((ln, i) => { const w = wOf(ln, as); rows.push([at.a + i * as * 1.2, as, -w / 2, w / 2]); }); y = at.a + (aLines - 1) * as * 1.2 + 3 * k; }
      return { k, nfs, fs, lineH, ps, as, rs, at, rows, h: y };
    };
    // does every row fit the polygon when the stack starts at top? (checked at the top and bottom of each row's text)
    const fits = (L, top) => top >= minY + 4 && top + L.h <= maxY - 4 && L.rows.every(([b, sz, l, r]) => [b - sz * .75, b + sz * .2].every(yy => { const [x1, x2] = spanAt(top + yy); return cx + l >= x1 + 2 && cx + r <= x2 - 2; }));
    let L = null, top = 0;
    for (const k of [1, .9, .8, .7, .62, .55]) {
      const c = layout(k), pref = cy - c.h / 2;
      const cand = []; for (let t = minY + 4; t + c.h <= maxY - 4; t += 2) cand.push(t);
      cand.sort((a, b) => Math.abs(a - pref) - Math.abs(b - pref));
      const t = cand.find(t2 => fits(c, t2));
      if (t != null) { L = c; top = t; break; }
    }
    if (!L) { L = layout(.55); top = Math.max(minY + 4, Math.min(cy - L.h / 2, maxY - 4 - L.h)); }
    const n = s('text', { x: cx, y: top + L.at.num, 'text-anchor': 'middle', 'font-size': L.nfs, 'font-weight': 800, fill: 'var(--accent)' }, g); n.textContent = sign + 1;
    if (ar[sign]) { const t = s('text', { x: cx, y: top + L.at.pada, 'text-anchor': 'middle', 'font-size': L.ps, fill: '#a78bfa', 'font-weight': 700, class: 'arudha' }, g); t.textContent = ar[sign].join(' ');
      const tt = s('title', {}, t); tt.textContent = 'Ārūḍha padas in this sign: ' + ar[sign].join(', '); }
    const tw = tw0 * L.k;
    list.forEach((gid, k) => grahaLine(g, gid, cx - tw, top + L.at.g0 + k * L.lineH, d, { compact: true, fs: L.fs }));
    if (d === 1) {
      if (L.at.r != null) rashiMarks(g, sign, cx, top + L.at.r, 'middle', L.rs, maxW, true);
      if (L.at.a != null) aspectMarks(g, sign, cx, top + L.at.a, 'middle', L.as, maxW, true);
    }
    g.addEventListener('click', () => { if (d === 1) go('#/house/' + (hIdx + 1)); });
  });
}
function chartSVG(kind, d, opts = {}) {
  const svg = s('svg', { viewBox: kind === 'north' ? '-4 -4 448 448' : '-4 -4 568 491', role: 'img', 'aria-label': `D${d} ${kind === 'north' ? 'North' : 'South'} Indian chart`, class: 'chart ' + kind });
  if (kind === 'north') { northGrid(svg, d, 0, 0, 440, opts); const t = s('text', { x: 220, y: 436, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `D${d} · ${DIV_NAME[d] || ''} · North Indian`; }
  else southGrid(svg, d, 0, 0, 140, opts);
  return svg;
}

/* ── two South Indian charts as concentric squares ──────────────────────────────────────────────────────────────
   the inner square holds one chart (twelve boxes around an empty centre); the outer band holds the other, each sign's
   sector widening evenly from its inner box to a quarter of the outer edge (corner signs wrap the corners).
   inner(sign) and outer(sign) return items: { text, colour, title, strong } */
function concentricSouth(inner, outer, opts = {}) {
  const S0 = 600, m = 124, cw = (S0 - 2 * m) / 4, q = S0 / 4;
  const svg = s('svg', { viewBox: `-4 -4 ${S0 + 8} ${S0 + 8}`, role: 'img', 'aria-label': opts.label || 'Concentric South Indian chart', class: 'chart south concentric' });
  s('rect', { x: 0, y: 0, width: S0, height: S0, class: 'cc-frame' }, svg);
  const inn = (c, r) => [m + c * cw, m + r * cw];
  const lines = (parent, items, cx, cy, size) => {
    const per = 2, rows = Math.ceil(items.length / per), lh = size * 1.25, y0 = cy - (rows - 1) * lh / 2;
    for (let r = 0; r < rows; r++) {
      const t = s('text', { x: cx, y: y0 + r * lh, 'text-anchor': 'middle', 'dominant-baseline': 'central', 'font-size': size }, parent);
      items.slice(r * per, r * per + per).forEach((it, k) => {
        const ts = s('tspan', { fill: it.colour, 'font-weight': it.strong ? 850 : 650, 'text-decoration': it.underline ? 'underline' : 'none' }, t);
        ts.textContent = (k ? ' ' : '') + it.text;
        if (it.title) { const tt = s('title', {}, ts); tt.textContent = it.title; }
      });
    }
  };
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg;
    let pts;
    if (r === 0 && c > 0 && c < 3) pts = [[c * q, 0], [(c + 1) * q, 0], inn(c + 1, 0), inn(c, 0)];
    else if (r === 3 && c > 0 && c < 3) pts = [[c * q, S0], [(c + 1) * q, S0], inn(c + 1, 4), inn(c, 4)];
    else if (c === 0 && r > 0 && r < 3) pts = [[0, r * q], [0, (r + 1) * q], inn(0, r + 1), inn(0, r)];
    else if (c === 3 && r > 0 && r < 3) pts = [[S0, r * q], [S0, (r + 1) * q], inn(4, r + 1), inn(4, r)];
    else if (c === 0 && r === 0) pts = [[0, 0], [q, 0], inn(1, 0), inn(0, 0), inn(0, 1), [0, q]];
    else if (c === 3 && r === 0) pts = [[S0, 0], [3 * q, 0], inn(3, 0), inn(4, 0), inn(4, 1), [S0, q]];
    else if (c === 0 && r === 3) pts = [[0, S0], [q, S0], inn(1, 4), inn(0, 4), inn(0, 3), [0, 3 * q]];
    else pts = [[S0, S0], [3 * q, S0], inn(3, 4), inn(4, 4), inn(4, 3), [S0, 3 * q]];
    const og = s('g', { class: 'cell cc-outer', 'data-sign': sign }, svg);
    s('polygon', { points: pts.map(p => p.map(v => v.toFixed(1)).join(',')).join(' '), class: 'cellbg' }, og);
    // label point: the middle of the band in front of the inner box (pulled into the corner for corner signs)
    const [bx, by] = inn(c + .5, r + .5), corner = (c === 0 || c === 3) && (r === 0 || r === 3);
    const ox = c === 0 ? m / 2 : c === 3 ? S0 - m / 2 : bx, oy = r === 0 ? m / 2 : r === 3 ? S0 - m / 2 : by;
    const lx = corner ? (c === 0 ? m * .55 : S0 - m * .55) : ox, ly = corner ? (r === 0 ? m * .55 : S0 - m * .55) : oy;
    lines(og, outer(sign), lx, ly, 15);
    const ig = s('g', { class: 'cell cc-inner', 'data-sign': sign }, svg), [x, y] = inn(c, r);
    s('rect', { x, y, width: cw, height: cw, class: 'cellbg' }, ig);
    const hn = houseOf(sign);
    if (hn) { const ht = s('text', { x: x + 4, y: y + 12, 'font-size': 10, 'font-weight': 800, fill: 'var(--accent2)' }, ig); ht.textContent = hn; }
    const sa = s('text', { x: x + cw - 4, y: y + 12, 'text-anchor': 'end', 'font-size': 10, fill: 'var(--muted)' }, ig); sa.textContent = SIGN_AB[sign];
    lines(ig, inner(sign), x + cw / 2, y + cw / 2 + 4, 14);
    ig.addEventListener('click', () => { if (hn) go('#/house/' + hn); });
  }
  s('rect', { x: m + cw, y: m + cw, width: 2 * cw, height: 2 * cw, class: 'cellmid' }, svg);
  const t1 = s('text', { x: S0 / 2, y: S0 / 2 - 8, 'text-anchor': 'middle', 'font-size': 14, 'font-weight': 800, fill: 'var(--ink)' }, svg); t1.textContent = opts.title || '';
  const t2 = s('text', { x: S0 / 2, y: S0 / 2 + 12, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t2.textContent = opts.sub || '';
  return svg;
}
const natalItems = sign => (LAGNA_SIGN === sign ? [{ text: 'As', colour: G.lagna.dot, strong: true, title: 'Lagna' }] : [])
  .concat(GRAHA_ORDER.filter(g => GR[g] && signOf(GR[g].lon) === sign).map(g => ({ text: G[g].abbr + (GR[g].retro && !['rahu', 'ketu'].includes(g) ? 'ᴿ' : ''), colour: G[g].dot === '#E6E9F0' ? 'var(--ink)' : G[g].dot, title: `${G[g].en} ${SIGN_SKT[sign]} ${inSignDM(GR[g].lon)}` })));

/* ── bhāva chalit over the rāśi chart: concentric South Indian squares, or a circle ─────────────────────────────
   outer: the rāśi chart (signs); inner: the same grahas by their Śrīpati bhāva (from the engine's bhava_chalit) */
function chalitSquare() {
  const bc = val('vedic.bhava_chalit'); if (!bc || LAGNA_SIGN == null) return null;
  const outer = sign => (LAGNA_SIGN === sign ? [{ text: 'As', colour: G.lagna.dot, strong: true }] : []).concat(Object.entries(bc.grahas)
    .filter(([g, hh]) => G[g] && GR[g] && (LAGNA_SIGN + hh - 1) % 12 === sign)
    .map(([g, hh]) => { const shifted = houseOf(signOf(GR[g].lon)) !== hh; return { text: G[g].abbr + (shifted ? '→' : ''), colour: G[g].dot === '#E6E9F0' ? 'var(--ink)' : G[g].dot, strong: shifted,
      title: `${G[g].en}: rāśi bhāva ${houseOf(signOf(GR[g].lon))}, chalit bhāva ${hh}${shifted ? ' (shifts)' : ''}` }; }));
  return concentricSouth(natalItems, outer, { title: 'Inner: rāśi', sub: 'Outer: bhāva (Śrīpati)', label: 'Rāśi chart inside, bhāva chalit outside' });
}
function chalitCircle() {
  const bc = val('vedic.bhava_chalit'); if (!bc || ASC == null) return null;
  const svg = s('svg', { viewBox: '-310 -310 620 620', role: 'img', 'aria-label': 'Rāśi and bhāva chalit as concentric circles', class: 'chalit-circle' });
  // the lagna sits at the left (east), the zodiac runs counter-clockwise
  const P = (lon, r) => { const a = Math.PI + (norm(lon - ASC)) * DEG; return [r * Math.cos(a), -r * Math.sin(a)]; };
  const sector = (l0, l1, r0, r1) => { const span = norm(l1 - l0), L = span > 180 ? 1 : 0, [a, b] = P(l0, r1), [c, d] = P(l1, r1), [e, f] = P(l1, r0), [g, k] = P(l0, r0);
    return `M${a.toFixed(1)} ${b.toFixed(1)}A${r1} ${r1} 0 ${L} 0 ${c.toFixed(1)} ${d.toFixed(1)}L${e.toFixed(1)} ${f.toFixed(1)}A${r0} ${r0} 0 ${L} 1 ${g.toFixed(1)} ${k.toFixed(1)}Z`; };
  for (let i = 0; i < 12; i++) {
    s('path', { d: sector(i * 30, i * 30 + 30, 250, 292), fill: G[SIGN_LORD[i]].band, stroke: 'var(--line)', class: 'cc-sign' }, svg);
    const [x, y] = P(i * 30 + 15, 271); const t = s('text', { x, y, class: 'lbl', 'font-size': 17, fill: '#1b1b1b' }, svg); t.textContent = GLYPH[i];
  }
  const sand = bc.bhava_sandhi, mad = bc.bhava_madhya;
  for (let k = 0; k < 12; k++) {
    const p = s('path', { d: sector(sand[k], sand[(k + 1) % 12], 196, 248), fill: k % 2 ? 'var(--panel)' : 'var(--chip2)', stroke: 'var(--ink)', 'stroke-opacity': .35, class: 'cc-bhava' }, svg);
    const tt = s('title', {}, p); tt.textContent = `Bhāva ${k + 1}: ${SIGN_AB[signOf(sand[k])]} ${inSignDM(sand[k])} → ${SIGN_AB[signOf(sand[(k + 1) % 12])]} ${inSignDM(sand[(k + 1) % 12])}; madhya ${SIGN_AB[signOf(mad[k])]} ${inSignDM(mad[k])}`;
    const [x, y] = P(mad[k], 222); const t = s('text', { x, y, class: 'lbl', 'font-size': 15, 'font-weight': 800, fill: k === 0 ? 'var(--accent)' : 'var(--ink)' }, svg); t.textContent = k + 1;
    const [mx0, my0] = P(mad[k], 196), [mx1, my1] = P(mad[k], 292); s('line', { x1: mx0, y1: my0, x2: mx1, y2: my1, class: 'cc-madhya' }, svg);
  }
  const items = GRAHA_ORDER.filter(g => GR[g] && G[g]).map(g => ({ g, l: GR[g].lon })).sort((a, b) => norm(a.l - ASC) - norm(b.l - ASC));
  let last = -99, lane = 0;
  for (const it of items) {
    const rel = norm(it.l - ASC); lane = rel - last < 7 ? (lane + 1) % 3 : 0; last = rel;
    const r = 168 - lane * 30, [x, y] = P(it.l, r), [nx, ny] = P(it.l, 194), shifted = houseOf(signOf(it.l)) !== bc.grahas[it.g];
    s('line', { x1: x, y1: y, x2: nx, y2: ny, stroke: G[it.g].dot, 'stroke-width': 1.4, opacity: .7 }, svg);
    const mk = marker(svg, it.g, x, y, 12, { passive: true });
    mk.addEventListener('mousemove', ev => tip(ev, `${G[it.g].en}`, [['Position', `${SIGN_SKT[signOf(it.l)]} ${inSignDM(it.l)}`], ['Rāśi bhāva', houseOf(signOf(it.l))], ['Chalit bhāva', bc.grahas[it.g] + (shifted ? ' (shifts)' : '')]]));
    mk.addEventListener('mouseleave', untip); mk.addEventListener('click', () => go('#/planet/' + it.g));
    if (shifted) s('circle', { cx: x, cy: y, r: 17, class: 'cc-shift' }, svg);
  }
  const [ax, ay] = P(ASC, 300), [bx, by] = P(ASC, 140); s('line', { x1: ax, y1: ay, x2: bx, y2: by, stroke: 'var(--accent)', 'stroke-width': 3 }, svg);
  const t = s('text', { x: 0, y: -6, 'text-anchor': 'middle', 'font-size': 14, 'font-weight': 800, fill: 'var(--ink)' }, svg); t.textContent = 'Rāśi · Bhāva chalit';
  const t2 = s('text', { x: 0, y: 12, 'text-anchor': 'middle', 'font-size': 10, fill: 'var(--muted)' }, svg); t2.textContent = 'outer: signs · inner: Śrīpati bhāvas';
  return svg;
}

/* ── Sudarshana chakra: lagna, Moon and Sun charts as three concentric rings ─────────────────────────────────── */
function sudarshana() {
  const svg = s('svg', { viewBox: '-330 -330 660 660', role: 'img', 'aria-label': 'Sudarshana chakra', class: 'sudarshana' });
  const refs = [['Lagna', LAGNA_SIGN, 300, 230], ['Moon', GR.moon ? signOf(GR.moon.lon) : null, 230, 160], ['Sun', GR.sun ? signOf(GR.sun.lon) : null, 160, 90]];
  const pl = placements(1);
  for (const [name, start, r1, r0] of refs) {
    if (start == null) continue;
    for (let hh = 0; hh < 12; hh++) {
      const sign = (start + hh) % 12, a0 = hh * 30 - 15 - 90 + 90, mid = hh * 30;
      s('path', { d: arc(r0, r1, a0, a0 + 30), fill: hh === 0 ? 'var(--chip2)' : 'var(--panel)', stroke: 'var(--line)' }, svg);
      const [nx, ny] = pt(r1 - 12, mid); const n = s('text', { class: 'lbl', x: nx, y: ny, 'font-size': 10, fill: 'var(--accent)' }, svg); n.textContent = sign + 1;
      const list = pl[sign] || [];
      const [x, y] = pt((r0 + r1) / 2 - 4, mid); const t = s('text', { class: 'lbl', x, y, 'font-size': list.length > 2 ? 10 : 12, 'font-weight': 700 }, svg);
      list.forEach((g, k) => { const ts = s('tspan', { fill: G[g].dot === '#E6E9F0' ? 'var(--ink)' : G[g].dot }, t); ts.textContent = (k ? ' ' : '') + G[g].abbr; });
    }
    const [lx, ly] = pt(r1 - 4, -15); const lt = s('text', { x: lx - 4, y: ly + 2, 'font-size': 10, fill: 'var(--muted)', 'text-anchor': 'end' }, svg); lt.textContent = name;
  }
  s('circle', { r: 88, fill: 'var(--disc)', stroke: 'var(--line)' }, svg);
  const t = s('text', { class: 'lbl', y: -6, 'font-size': 15, 'font-weight': 800 }, svg); t.textContent = 'Sudarshana';
  const t2 = s('text', { class: 'lbl', y: 14, 'font-size': 10, fill: 'var(--muted)' }, svg); t2.textContent = 'lagna · Moon · Sun';
  return svg;
}

/* ── the Nadi direction matrix: the zodiac as a spiral of three rounds through the four directions ───────────── */
const NADI_Q = { Fire: { d: [1, -1], dir: 'East', trikona: 'Dharma', houses: '1 · 5 · 9', c: '#ff7a59' }, Earth: { d: [1, 1], dir: 'South', trikona: 'Artha', houses: '2 · 6 · 10', c: '#7cc982' },
  Air: { d: [-1, 1], dir: 'West', trikona: 'Kama', houses: '3 · 7 · 11', c: '#e7c35a' }, Water: { d: [-1, -1], dir: 'North', trikona: 'Moksha', houses: '4 · 8 · 12', c: '#6fa8f5' } };
function nadiLinks(id) { /* report.nadi@0.1: trines (same element and direction), 2nd/12th and 7th from the graha's sign; a retrograde graha also acts from the previous sign */
  const gr = GR[id]; if (!gr) return [];
  const homes = [signOf(gr.lon)]; if (gr.retro && !['rahu', 'ketu'].includes(id)) homes.push((signOf(gr.lon) + 11) % 12);
  const out = [];
  for (const o of NINE) {
    if (o === id || !GR[o]) continue;
    for (const hs of homes) {
      const d = (signOf(GR[o].lon) - hs + 12) % 12 + 1;
      const kind = d === 1 ? 'conjunct' : [5, 9].includes(d) ? 'trine (same direction)' : d === 2 ? '2nd (next)' : d === 12 ? '12th (previous)' : d === 7 ? '7th (opposite)' : null;
      if (kind) { out.push({ to: o, kind, via: hs !== signOf(gr.lon) ? 'retrograde reach' : null }); break; }
    }
  }
  return out;
}
function nadiMatrix(sel, onPick) {
  const C = 330, SZ = [96, 196, 320];
  const svg = s('svg', { viewBox: '0 0 660 700', role: 'img', 'aria-label': 'Nadi direction matrix', class: 'nadi' });
  const labels = [['NORTH', 'Moksha trikona', 160, 18, NADI_Q.Water.c], ['EAST', 'Dharma trikona', 500, 18, NADI_Q.Fire.c], ['WEST', 'Kama trikona', 160, 690, NADI_Q.Air.c], ['SOUTH', 'Artha trikona', 500, 690, NADI_Q.Earth.c]];
  for (const [a, b, x, y, c] of labels) { const t = s('text', { x, y, 'text-anchor': 'middle', 'font-size': 13, 'font-weight': 800, 'letter-spacing': 3, fill: 'var(--ink)' }, svg); t.textContent = a; const t2 = s('text', { x, y: y + (y < 100 ? 15 : -15), 'text-anchor': 'middle', 'font-size': 10.5, 'font-weight': 700, fill: c }, svg); t2.textContent = b.toUpperCase(); }
  const gy = 30; // shift the square down under the labels
  for (const el of ['Fire', 'Earth', 'Air', 'Water']) {
    const [dx, dy] = NADI_Q[el].d;
    [2, 1, 0].forEach(L => { const sz = SZ[L]; s('rect', { x: dx > 0 ? C : C - sz, y: gy + (dy > 0 ? C : C - sz), width: sz, height: sz, rx: 6, fill: NADI_Q[el].c, 'fill-opacity': [.85, .55, .32][L], stroke: 'var(--panel)', 'stroke-width': 2, class: 'nq' }, svg); });
  }
  const linked = sel ? nadiLinks(sel) : [];
  const linkSet = new Set(linked.map(x => x.to));
  const centre = {};
  for (let i = 0; i < 12; i++) {
    const el = ELEMENT[i % 4], [dx, dy] = NADI_Q[el].d, L = Math.floor(i / 4);
    const r = L ? (SZ[L - 1] + SZ[L]) / 2 : SZ[0] / 2;
    const x = C + dx * r, y = gy + C + dy * r;
    centre[i] = [x, y];
    const g = s('g', { class: 'nsign' }, svg);
    const t = s('text', { x, y: y - 22, 'text-anchor': 'middle', 'font-size': 18, fill: '#1b1b1b' }, g); t.textContent = GLYPH[i];
    const t2 = s('text', { x, y: y - 6, 'text-anchor': 'middle', 'font-size': 11.5, 'font-weight': 700, fill: '#1b1b1b' }, g); t2.textContent = SIGN_EN[i] + (houseOf(i) ? ` · ${houseOf(i)}` : '');
    const list = NINE.concat(['uranus', 'neptune', 'pluto']).filter(gid => GR[gid] && signOf(GR[gid].lon) === i);
    list.forEach((gid, k) => {
      const cx = x - (list.length - 1) * 21 + k * 42, cy = y + 18;
      const m = marker(svg, gid, cx, cy, 12, { label: `${G[gid].abbr} ${dm(norm(GR[gid].lon) % 30)}`, passive: true });
      m.classList.toggle('sel', gid === sel); m.classList.toggle('linked', linkSet.has(gid)); if (sel && gid !== sel && !linkSet.has(gid)) m.classList.add('faded');
      m.addEventListener('mousemove', ev => placeTip(ev, gid)); m.addEventListener('mouseleave', untip);
      m.addEventListener('click', () => onPick && onPick(gid));
    });
  }
  if (sel && GR[sel]) {
    const from = centre[signOf(GR[sel].lon)], lg = s('g', { class: 'nlinks' }, svg);
    for (const l of linked) { const to = centre[signOf(GR[l.to].lon)]; s('path', { d: `M${from[0]} ${from[1] + 18}L${to[0]} ${to[1] + 18}`, stroke: G[sel].dot, 'stroke-width': 2.4, 'stroke-dasharray': l.kind.startsWith('trine') ? '' : '6 5', class: 'flow' }, lg); }
  }
  s('circle', { cx: C, cy: gy + C, r: 30, fill: 'var(--panel)', stroke: 'var(--line)', 'stroke-width': 2 }, svg);
  const e = s('text', { x: C, y: gy + C + 4, 'text-anchor': 'middle', 'font-size': 12, 'font-weight': 800 }, svg); e.textContent = 'Ether';
  return svg;
}

/* ── Western wheel ────────────────────────────────────────────────────────────────────────────────────────────── */
function westernWheel() {
  const wgr = grahasOf(WES), asc = WES.houses && WES.houses.ascendant ? WES.houses.ascendant.longitude_deg : 0, ang = l => norm(270 - (l - asc));
  const svg = s('svg', { viewBox: '-440 -440 880 880', role: 'img', 'aria-label': 'Western chart wheel', class: 'wheel' });
  s('circle', { r: 425, fill: 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 2 }, svg);
  for (let i = 0; i < 12; i++) { s('path', { d: arc(355, 425, ang(i * 30 + 30), ang(i * 30 + 30) + 30), fill: PADA_FILL[ELEMENT[i % 4]], stroke: '#fff' }, svg);
    const [x, y] = pt(390, ang(i * 30 + 15)); const t = s('text', { class: 'lbl', x, y, 'font-size': 30, fill: '#1b1b1b' }, svg); t.textContent = GLYPH[i]; }
  const cusps = (WES.houses || {}).cusps_deg || [];
  cusps.forEach((c, k) => { const [x0, y0] = pt(125, ang(c)), [x1, y1] = pt(355, ang(c)); s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--ink)', 'stroke-width': [0, 3, 6, 9].includes(k) ? 3 : 1 }, svg);
    const nx = cusps[(k + 1) % 12], mid = c + norm(nx - c) / 2, [x, y] = pt(140, ang(mid)); const t = s('text', { class: 'lbl', x, y, 'font-size': 14, fill: 'var(--muted)' }, svg); t.textContent = k + 1; });
  s('circle', { r: 125, fill: 'none', stroke: 'var(--line)' }, svg);
  const ag = s('g', { class: 'asp' }, svg);
  for (const a of WES.derived_attributes.filter(x => x.id.includes('.aspect.'))) { const [b1, b2] = a.derived_from.map(engineId); if (!wgr[b1] || !wgr[b2]) continue; const [x0, y0] = pt(122, ang(wgr[b1].lon)), [x1, y1] = pt(122, ang(wgr[b2].lon));
    const ln = s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: ASPECT_COLOUR[a.value.aspect] }, ag); ln.addEventListener('mousemove', ev => tip(ev, `${G[b1].en} ${a.value.aspect} ${G[b2].en}`, [['Orb', a.value.orb_deg.toFixed(2) + '°'], ['', a.value.applying ? 'applying' : 'separating']])); ln.addEventListener('mouseleave', untip); }
  const items = GRAHA_ORDER.filter(g => wgr[g]).map(g => ({ g, a: ang(wgr[g].lon) })).sort((x, y) => x.a - y.a), last = [null, null, null];
  for (const it of items) { let lane = 0; for (; lane < 3; lane++) if (last[lane] == null || norm(it.a - last[lane]) >= 9) break; lane = Math.min(lane, 2); last[lane] = it.a; const [x, y] = pt(320 - lane * 44, it.a); marker(svg, it.g, x, y, 17, { retro: wgr[it.g].retro && !['rahu', 'ketu'].includes(it.g), passive: true }); }
  return svg;
}

/* ── Human Design BodyGraph (detailed or schematic) ───────────────────────────────────────────────────────────── */
const HD_CENTER_GATES = { head: [64, 61, 63], ajna: [47, 24, 4, 17, 43, 11], throat: [62, 23, 56, 35, 12, 45, 33, 8, 31, 20, 16], g: [1, 13, 25, 46, 2, 15, 10, 7],
  heart: [21, 40, 26, 51], sacral: [5, 14, 29, 59, 9, 3, 42, 27, 34], spleen: [48, 57, 44, 50, 32, 28, 18], solar_plexus: [36, 22, 37, 6, 49, 55, 30], root: [53, 60, 52, 19, 39, 41, 58, 38, 54] };
const HD_CHANNELS = [[1, 8], [2, 14], [3, 60], [4, 63], [5, 15], [6, 59], [7, 31], [9, 52], [10, 20], [10, 34], [10, 57], [11, 56], [12, 22], [13, 33], [16, 48], [17, 62], [18, 58], [19, 49], [20, 34], [20, 57], [21, 45], [23, 43], [24, 61], [25, 51], [26, 44], [27, 50], [28, 38], [29, 46], [30, 41], [32, 54], [34, 57], [35, 36], [37, 40], [39, 55], [42, 53], [47, 64]];
const HD_POS = { head: [200, 50], ajna: [200, 135], throat: [200, 235], g: [200, 340], heart: [275, 380], sacral: [200, 470], spleen: [75, 440], solar_plexus: [325, 440], root: [200, 575] };
const HD_COL = { head: '#F5D547', ajna: '#7FB77E', throat: '#B88A5A', g: '#F5D547', heart: '#D9534F', sacral: '#D9534F', spleen: '#B88A5A', solar_plexus: '#B88A5A', root: '#B88A5A' };
/* ── Human Design: the standard chart layout ─────────────────────────────────────────────────────────────────────
   Design column (left) · BodyGraph over a human silhouette · Personality column (right), the four variable arrows
   at the head, and the properties. The geometry is ESHANA's own; it follows the conventional arrangement. */
const HDS_GATE = {
  64: [178, 98], 61: [200, 98], 63: [222, 98],
  47: [178, 142], 24: [200, 142], 4: [222, 142], 17: [183, 168], 11: [217, 168], 43: [200, 194],
  62: [178, 262], 23: [200, 262], 56: [222, 262], 16: [168, 280], 20: [168, 300], 35: [232, 276], 12: [232, 294], 45: [232, 312], 31: [178, 326], 8: [200, 326], 33: [222, 326],
  1: [200, 376], 7: [180, 398], 13: [220, 398], 10: [158, 420], 25: [242, 420], 15: [180, 442], 46: [220, 442], 2: [200, 464],
  21: [292, 446], 51: [276, 462], 26: [272, 484], 40: [306, 484],
  5: [178, 540], 14: [200, 540], 29: [222, 540], 34: [168, 560], 27: [168, 588], 59: [232, 588], 42: [178, 604], 3: [200, 604], 9: [222, 604],
  53: [178, 652], 60: [200, 652], 52: [222, 652], 54: [168, 670], 38: [168, 690], 58: [168, 710], 19: [232, 670], 39: [232, 690], 41: [232, 710],
  48: [50, 528], 57: [66, 543], 44: [84, 558], 50: [104, 576], 32: [84, 596], 28: [66, 612], 18: [50, 628],
  36: [350, 528], 22: [334, 543], 37: [316, 558], 6: [296, 576], 49: [316, 596], 55: [334, 612], 30: [350, 628],
};
const HDS_SHAPE = {
  head: 'M200 28L242 108H158Z', ajna: 'M158 132H242L200 210Z', throat: 'M158 252H242V336H158Z', g: 'M200 360L254 420L200 480L146 420Z',
  heart: 'M296 432L318 494H262Z', sacral: 'M158 528H242V616H158Z', root: 'M158 640H242V722H158Z', spleen: 'M38 512L128 576L38 642Z', solar_plexus: 'M362 512L272 576L362 642Z',
};
const HDS_FILL = { head: '#F5D547', ajna: '#7FB77E', throat: '#B88A5A', g: '#F5D547', heart: '#D9534F', sacral: '#D9534F', spleen: '#B88A5A', solar_plexus: '#B88A5A', root: '#B88A5A' };
const HD_GLYPH = { sun: '☉', earth: '⊕', moon: '☽', north_node: '☊', south_node: '☋', mercury: '☿', venus: '♀', mars: '♂', jupiter: '♃', saturn: '♄', uranus: '♅', neptune: '♆', pluto: '♇' };
const HD_ORDER = ['sun', 'earth', 'north_node', 'south_node', 'moon', 'mercury', 'venus', 'mars', 'jupiter', 'saturn', 'uranus', 'neptune', 'pluto'];
/* tradition tables (human_design.basic): strategy, signature and not-self theme by type; variable names by colour and tone */
const HD_TYPE = {
  generator: ['Generator', 'To respond', 'Satisfaction', 'Frustration'], manifesting_generator: ['Manifesting Generator', 'To respond, then inform', 'Satisfaction', 'Frustration and anger'],
  projector: ['Projector', 'Wait for the invitation', 'Success', 'Bitterness'], manifestor: ['Manifestor', 'To inform', 'Peace', 'Anger'], reflector: ['Reflector', 'Wait a lunar cycle', 'Surprise', 'Disappointment'],
};
const HD_VAR = {
  determination: ['Appetite', 'Taste', 'Thirst', 'Touch', 'Sound', 'Light'], environment: ['Caves', 'Markets', 'Kitchens', 'Mountains', 'Valleys', 'Shores'],
  motivation: ['Fear', 'Hope', 'Desire', 'Need', 'Guilt', 'Innocence'], perspective: ['Survival', 'Possibility', 'Power', 'Wanting', 'Probability', 'Personal'],
  sense: ['Smell', 'Taste', 'Outer vision', 'Inner vision', 'Feeling', 'Touch'],
};
function hdStandard() {
  const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value || {}, defined = new Set(bg.defined_centers || []), chans = new Set(bg.channels || []);
  const pers = HD.positions.personality.activations, des = HD.positions.design.activations, gp = new Set(pers.map(a => a.gate)), gd = new Set(des.map(a => a.gate));
  const svg = s('svg', { viewBox: '0 0 400 750', role: 'img', 'aria-label': 'Human Design BodyGraph', class: 'hdgraph' });
  const defs = s('defs', {}, svg), pat = s('pattern', { id: 'hd-both', width: 6, height: 6, patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(45)' }, defs);
  s('rect', { width: 3, height: 6, fill: '#1b1b1b' }, pat); s('rect', { x: 3, width: 3, height: 6, fill: '#c8102e' }, pat);
  // an original silhouette, drawn as one half and mirrored so both sides (arms, legs) match exactly
  const body = s('g', { class: 'hd-body' }, svg);
  const half = 'M200 8C168 8 150 34 150 68C150 100 166 120 182 126L182 156C150 160 120 166 104 180C88 194 82 224 80 260L66 470C64 492 80 500 90 486L112 300L118 300L118 470C118 520 124 560 132 580L136 744L194 744L198 590L200 590Z';
  s('path', { d: half }, body);
  s('path', { d: half, transform: 'matrix(-1 0 0 1 400 0)' }, body);
  const colour = g => gp.has(g) && gd.has(g) ? 'url(#hd-both)' : gp.has(g) ? '#1b1b1b' : gd.has(g) ? '#c8102e' : null;
  const curve = (a, b) => { const [x0, y0] = HDS_GATE[a], [x1, y1] = HDS_GATE[b], mx = (x0 + x1) / 2, my = (y0 + y1) / 2, side = mx < 200 ? -1 : mx > 200 ? 1 : 0;
    const bow = Math.abs(x1 - x0) > 40 && Math.abs(y1 - y0) > 40 ? 26 * side : 0; return { d: `M${x0} ${y0}Q${mx + bow} ${my} ${x1} ${y1}`, mid: [ (x0 + 2 * (mx + bow) + x1) / 4, (y0 + 2 * my + y1) / 4 ] }; };
  const cg = s('g', { class: 'hd-channels' }, svg);
  for (const [a, b] of HD_CHANNELS) {
    if (!HDS_GATE[a] || !HDS_GATE[b]) continue;
    const { d } = curve(a, b), on = chans.has(`${Math.min(a, b)}-${Math.max(a, b)}`);
    s('path', { d, class: 'hd-ch-out' }, cg); s('path', { d, class: 'hd-ch-in' }, cg);
    // each half takes its gate's colour: split the curve at its midpoint with dash arrays on the full path
    for (const [g, first] of [[a, true], [b, false]]) { const col = colour(g); if (!col) continue;
      const p = s('path', { d: first ? d : curve(b, a).d, class: 'hd-ch-half' + (on ? ' on' : ''), stroke: col }, cg);
      p.setAttribute('pathLength', 100); p.setAttribute('stroke-dasharray', '50 100'); }
    const tt = s('title', {}, cg.lastChild || cg); tt.textContent = `Channel ${a}–${b}${on ? ' (defined)' : ''}`;
  }
  for (const c in HDS_SHAPE) {
    const p = s('path', { d: HDS_SHAPE[c], class: 'hd-center' + (defined.has(c) ? ' on' : ''), fill: defined.has(c) ? HDS_FILL[c] : '#fff' }, svg);
    p.addEventListener('mousemove', ev => tip(ev, c.replace('_', ' '), [['State', defined.has(c) ? 'defined' : 'open (undefined)'], ['Gates', HD_CENTER_GATES[c].join(', ')]])); p.addEventListener('mouseleave', untip);
  }
  for (const g in HDS_GATE) { const [x, y] = HDS_GATE[g], col = colour(+g);
    if (col) s('circle', { cx: x, cy: y, r: 8.6, fill: col, class: 'hd-gate-on' }, svg);
    const t = s('text', { x, y: y + 3, class: 'hd-gate' + (col ? ' on' : '') }, svg); t.textContent = g;
    const who = [gp.has(+g) && 'personality', gd.has(+g) && 'design'].filter(Boolean).join(' and ');
    if (who) { const tt = s('title', {}, t); tt.textContent = `Gate ${g}: ${who}`; }
  }
  return svg;
}
function hdColumn(acts, side) {
  const by = {}; for (const a of acts) by[a.body] = a;
  return h('div', { class: 'hd-col ' + side }, h('div', { class: 'hd-col-t' }, side === 'design' ? 'Design' : 'Personality'),
    HD_ORDER.filter(k => by[k]).map((k, i) => { const a = by[k];
      return h('div', { class: 'hd-row', style: `--i:${i}`, title: `${k.replace('_', ' ')}: gate ${a.gate}, line ${a.line}, colour ${a.color}, tone ${a.tone || '—'}, base ${a.base || '—'}` },
        h('span', { class: 'hd-gl' }, HD_GLYPH[k] || k), h('b', {}, `${a.gate}.${a.line}`)); }));
}
function hdVariables() {
  const P = {}, Dz = {}; for (const a of HD.positions.personality.activations) P[a.body] = a; for (const a of HD.positions.design.activations) Dz[a.body] = a;
  const v = (a, table) => a ? { name: HD_VAR[table][a.color - 1], arrow: a.tone && a.tone <= 3 ? 'left' : 'right', color: a.color, tone: a.tone } : null;
  return { determination: v(Dz.sun, 'determination'), environment: v(Dz.north_node, 'environment'), motivation: v(P.sun, 'motivation'), perspective: v(P.north_node, 'perspective'),
    sense: P.sun && P.sun.tone ? HD_VAR.sense[P.sun.tone - 1] : null, design_sense: Dz.sun && Dz.sun.tone ? HD_VAR.sense[Dz.sun.tone - 1] : null };
}
function hdChart() {
  const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value || {}, V = hdVariables(), ty = HD_TYPE[bg.type] || [bg.type, '—', '—', '—'];
  const P = {}, Dz = {}; for (const a of HD.positions.personality.activations) P[a.body] = a; for (const a of HD.positions.design.activations) Dz[a.body] = a;
  const prof = String(bg.profile || ''), [l1, l2] = prof.split('/').map(Number);
  const angle = !l1 ? '' : (l1 === 4 && l2 === 1) ? 'Juxtaposition Cross' : (l1 >= 5) ? 'Left Angle Cross' : 'Right Angle Cross';
  const cross = P.sun && P.earth && Dz.sun && Dz.earth ? `${angle} (${P.sun.gate}/${P.earth.gate} | ${Dz.sun.gate}/${Dz.earth.gate})` : '—';
  const arrow = (x, label) => x ? h('span', { class: 'hd-arrow ' + x.arrow, title: `${label}: ${x.name} (colour ${x.color}, tone ${x.tone}) · arrow ${x.arrow}` }, x.arrow === 'left' ? '←' : '→') : null;
  const subj = D.subject || {};
  return h('div', { class: 'hd-std' },
    hdColumn(HD.positions.design.activations, 'design'),
    h('div', { class: 'hd-mid' }, h('div', { class: 'hd-arrows' }, h('div', {}, arrow(V.determination, 'Determination'), arrow(V.environment, 'Environment')), h('div', {}, arrow(V.motivation, 'Motivation'), arrow(V.perspective, 'Perspective'))), hdStandard()),
    hdColumn(HD.positions.personality.activations, 'personality'),
    h('div', { class: 'hd-props' }, h('h3', {}, 'Properties'), facts([['Name', subj.name], ['Birth (UTC)', HD.positions.personality.instant_utc], ['Design (UTC)', HD.positions.design.instant_utc],
      ['Type', ty[0]], ['Strategy', ty[1]], ['Inner authority', (bg.authority || '').replace(/_/g, ' ')], ['Definition', (bg.definition || '').replace(/_/g, ' ')], ['Profile', bg.profile],
      ['Incarnation cross', cross], ['Signature', ty[2]], ['Not-self theme', ty[3]],
      ['Determination', V.determination && V.determination.name], ['Sense', V.sense], ['Design sense', V.design_sense], ['Motivation', V.motivation && V.motivation.name],
      ['Perspective', V.perspective && V.perspective.name], ['Environment', V.environment && V.environment.name]]),
      note('Strategy, signature, not-self theme and variables follow the type and the colour and tone of the Sun and nodes (tradition tables). The cross is named by angle and gates; full cross names are a planned table.')));
}

function bodygraph() {
  const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value || {}, defined = new Set(bg.defined_centers || []), chans = new Set(bg.channels || []);
  const pers = HD.positions.personality.activations, des = HD.positions.design.activations, gp = new Set(pers.map(a => a.gate)), gd = new Set(des.map(a => a.gate));
  const center = {}; for (const c in HD_CENTER_GATES) for (const g of HD_CENTER_GATES[c]) center[g] = c;
  const svg = s('svg', { viewBox: '0 0 400 640', role: 'img', 'aria-label': 'Human Design BodyGraph', class: 'bodygraph' });
  const gatePos = {};
  for (const c in HD_CENTER_GATES) { const [cx, cy] = HD_POS[c], gs = HD_CENTER_GATES[c], r = c === 'heart' ? 26 : 42;
    gs.forEach((g, k) => { const a = (k + .5) / gs.length * 2 * Math.PI - Math.PI / 2; gatePos[g] = [cx + Math.cos(a) * r * .85, cy + Math.sin(a) * r * .85]; }); }
  const detailed = S.hd === 'detailed';
  const defs = s('defs', {}, svg), pat = s('pattern', { id: 'both', width: 6, height: 6, patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(45)' }, defs);
  s('rect', { width: 3, height: 6, fill: '#111' }, pat); s('rect', { x: 3, width: 3, height: 6, fill: '#c8102e' }, pat);
  for (const [a, c] of HD_CHANNELS) {
    const on = chans.has(`${Math.min(a, c)}-${Math.max(a, c)}`) || chans.has(`${a}-${c}`) || chans.has(`${c}-${a}`);
    if (!detailed && !on) continue;
    const [x0, y0] = detailed ? gatePos[a] : HD_POS[center[a]], [x1, y1] = detailed ? gatePos[c] : HD_POS[center[c]];
    const half = (g, xa, ya, xb, yb) => { const col = gp.has(g) && gd.has(g) ? 'url(#both)' : gp.has(g) ? '#111' : gd.has(g) ? '#c8102e' : 'var(--line)';
      s('line', { x1: xa, y1: ya, x2: (xa + xb) / 2, y2: (ya + yb) / 2, stroke: col, 'stroke-width': on ? 7 : 4, 'stroke-linecap': 'round' }, svg); };
    half(a, x0, y0, x1, y1); half(c, x1, y1, x0, y0);
  }
  const shapes = { head: 'up', ajna: 'down', throat: 'sq', g: 'dia', heart: 'up', sacral: 'sq', spleen: 'right', solar_plexus: 'left', root: 'sq' };
  for (const c in HD_POS) { const [x, y] = HD_POS[c], r = c === 'heart' ? 26 : 42, sh = shapes[c]; let d;
    if (sh === 'sq') d = `M${x - r} ${y - r}h${2 * r}v${2 * r}h${-2 * r}Z`; else if (sh === 'dia') d = `M${x} ${y - r - 8}L${x + r + 8} ${y}L${x} ${y + r + 8}L${x - r - 8} ${y}Z`;
    else if (sh === 'up') d = `M${x} ${y - r}L${x + r} ${y + r}L${x - r} ${y + r}Z`; else if (sh === 'down') d = `M${x - r} ${y - r}L${x + r} ${y - r}L${x} ${y + r}Z`;
    else if (sh === 'right') d = `M${x - r} ${y - r}L${x + r} ${y}L${x - r} ${y + r}Z`; else d = `M${x + r} ${y - r}L${x - r} ${y}L${x + r} ${y + r}Z`;
    const p = s('path', { d, fill: defined.has(c) ? HD_COL[c] : 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 2 }, svg);
    p.addEventListener('mousemove', ev => tip(ev, c.replace('_', ' '), [['State', defined.has(c) ? 'defined' : 'undefined'], ['Gates', HD_CENTER_GATES[c].join(', ')]])); p.addEventListener('mouseleave', untip);
    if (!detailed) { const t = s('text', { class: 'lbl', x, y, 'font-size': 11, fill: defined.has(c) ? '#1b1b1b' : 'var(--muted)' }, svg); t.textContent = c.replace('_', ' '); } }
  if (detailed) for (const g in gatePos) { const [x, y] = gatePos[g], act = gp.has(+g) || gd.has(+g);
    s('circle', { cx: x, cy: y, r: 7.5, fill: act ? (gp.has(+g) && gd.has(+g) ? 'url(#both)' : gp.has(+g) ? '#111' : '#c8102e') : 'var(--panel)', stroke: 'var(--muted)' }, svg);
    const t = s('text', { class: 'lbl', x, y, 'font-size': 7.5, 'font-weight': 700, fill: act ? '#fff' : 'var(--ink)' }, svg); t.textContent = g; }
  return svg;
}
