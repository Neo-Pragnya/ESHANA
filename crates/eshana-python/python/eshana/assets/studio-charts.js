/* ESHANA Studio, part 2: charts. The Panchang Chakra (with a South Indian, North Indian or orbit-lane centre), South
   and North Indian charts for any varga with degrees, nakshatras, flags and translucent aspect marks, the Sudarshana
   chakra, the Nadi direction matrix, the Western wheel and the BodyGraph. */

const RINGS_LANES = { tick: [532, 548], nak: [472, 532], pada: [436, 472], dres: [404, 436], rashi: [330, 404], house: [292, 330], lanes: [252, 210, 168, 126] };
// with a chart in the centre the rings are slimmer, so the inner disc (and the boxes inside it) can be large enough to read
const RINGS_CENTRE = { tick: [536, 550], nak: [494, 536], pada: [466, 494], dres: [442, 466], rashi: [398, 442], house: [374, 398], lanes: [252, 210, 168, 126] };
let RINGS = RINGS_LANES;
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
const ASPECTED = (() => { const m = {}; for (const d of DRISHTI) (m[d.sign] = m[d.sign] || []).push(d); return m; })();
function aspectMarks(parent, sign, x, y, anchor = 'start', size = 10) {
  if (!S.layers.aspect_marks) return;
  const list = (ASPECTED[sign] || []).filter(d => G[d.from]);
  if (!list.length) return;
  const t = s('text', { x, y, 'text-anchor': anchor, 'font-size': size, class: 'aspmark' }, parent);
  list.forEach((d, i) => { const ts = s('tspan', { fill: G[d.from].dot, 'font-weight': d.kind === 'special' ? 800 : 600 }, t); ts.textContent = (i ? ' ' : '') + G[d.from].abbr + (d.kind === 'special' ? '*' : ''); });
  const title = s('title', {}, t); title.textContent = 'Aspected by ' + list.map(d => `${G[d.from].en} (${nth(d.house_from_graha)}${d.kind === 'special' ? ', special' : ''})`).join(', ');
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

/* ── the Panchang Chakra ──────────────────────────────────────────────────────────────────────────────────────── */
let WHEEL = null, transitIdx = 0, playing = null, FOCUS = null;
function renderWheel(host, panel) {
  host.replaceChildren();
  RINGS = S.centre === 'lanes' ? RINGS_LANES : RINGS_CENTRE;
  const svg = s('svg', { viewBox: '-640 -640 1280 1280', role: 'img', 'aria-label': 'Panchang chakra with the grahas placed by sidereal longitude', class: 'wheel' }, host);
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
  if (L.ticks) for (let d = 0; d < 360; d++) {
    const a = angOf(d), len = d % 10 === 0 ? 16 : d % 5 === 0 ? 10 : 5, [x0, y0] = pt(RINGS.tick[0], a), [x1, y1] = pt(RINGS.tick[0] + len, a);
    s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--muted)', 'stroke-width': d % 10 === 0 ? 1.4 : .7 }, rot);
  }
  if (L.nak) for (let i = 0; i < 27; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'nak', 'data-i': i }, rot), lord = NAK_LORD[i % 9], fill = G[lord].nak;
    s('path', { d: arc(RINGS.nak[0], RINGS.nak[1], angOf(i * NAKS), angOf(i * NAKS) + NAKS), fill }, g);
    const mid = angOf((i + .5) * NAKS), [x, y] = pt((RINGS.nak[0] + RINGS.nak[1]) / 2, mid), r = mid <= 180 ? mid - 90 : mid + 90;
    const t = s('text', { class: 'lbl', x, y, 'font-size': 14, 'font-weight': 600, fill: ink(fill), transform: `rotate(${r.toFixed(1)} ${x.toFixed(1)} ${y.toFixed(1)})` }, g); t.textContent = NAK_SHORT[NAKSHATRA[i]] || NAKSHATRA[i];
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
    const [x, y] = pt(wide ? (RINGS.rashi[0] + rashiOuter) / 2 + 12 : (RINGS.rashi[0] + rashiOuter) / 2 + 6, angOf(i * 30 + (wide ? 15 : 9))); const t = s('text', { class: 'lbl', x, y, 'font-size': wide ? 34 : 26, fill: '#1b1b1b' }, g); t.textContent = GLYPH[i];
    const [x2, y2] = wide ? pt(RINGS.rashi[0] + 16, angOf(i * 30 + 15)) : pt((RINGS.rashi[0] + rashiOuter) / 2, angOf(i * 30 + 20)); const t2 = s('text', { class: 'lbl', x: x2, y: y2, 'font-size': 14, 'font-weight': 700, fill: '#1b1b1b' }, g); t2.textContent = SIGN_SKT[i];
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
  const pointsG = s('g', {}, inner);
  for (const g of GRAHA_ORDER) {
    if (!GR[g]) continue; const a = angOf(GR[g].lon), [tx, ty] = pt(rashiOuter + (L.pada ? 36 : 0), a);
    s('circle', { cx: tx, cy: ty, r: 5, fill: G[g].dot, stroke: '#111', 'stroke-width': 1.2, class: 'pt' }, pointsG);
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
    const pg = s('g', {}, inner);
    for (const it of items) {
      const p = pos[it.g], gr = GR[it.g], [x, y] = pt(p.r, p.a);
      const mk = marker(pg, it.g, x, y, 21, { retro: gr.retro && !['rahu', 'ketu'].includes(it.g) });
      mk.dataset.a = p.a; mk.dataset.r = p.r;
      mk.addEventListener('mousemove', ev => placeTip(ev, it.g)); mk.addEventListener('mouseleave', () => { untip(); hot([]); });
      mk.addEventListener('mouseenter', () => hot([['nak', nakOf(gr.lon)], ['pada', Math.floor(norm(gr.lon) / NAV)], ['rashi', signOf(gr.lon)], ['dres', Math.floor(norm(gr.lon) / 10)]]));
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
    WHEEL = { svg, rot, tg: s('g', {}, rot) }; drawTransit();
  } else WHEEL = { svg, rot };
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
  if (WHEEL) WHEEL.rot.style.transform = `rotate(${-mid}deg)`;
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
  const row = rows[Math.max(0, Math.min(rows.length - 1, transitIdx))]; WHEEL.tg.replaceChildren();
  for (const k in row.longitudes) {
    const id = engineId(k); if (!G[id]) continue; const a = angOf(row.longitudes[k]), [x, y] = pt(598, a);
    const mk = marker(WHEEL.tg, id, x, y, 15, { passive: true });
    mk.addEventListener('mousemove', ev => tip(ev, `Transit ${G[id].en}`, [['Date', dateOnly(row.utc)], ['Sidereal', `${SIGN_SKT[signOf(row.longitudes[k])]} ${inSign(row.longitudes[k])}`], ['Over natal bhāva', houseOf(signOf(row.longitudes[k]))]]));
    mk.addEventListener('mouseleave', untip);
  }
  const lab = $('#transit-date'); if (lab) lab.textContent = dateOnly(row.utc);
  const sl = $('#transit-slider'); if (sl) sl.value = transitIdx;
}
function playTransit(btn) {
  if (playing) { cancelAnimationFrame(playing); playing = null; btn.textContent = '▶ Play transits'; return; }
  btn.textContent = '❚❚ Pause'; let last = 0;
  const loop = t => { if (t - last > 90) { last = t; transitIdx = (transitIdx + 1) % D.transits.rows.length; drawTransit(); } playing = requestAnimationFrame(loop); };
  playing = requestAnimationFrame(loop);
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
  if (!opts.compact) marker(g, id, x + 8, y - 4, 7.5, { noLabel: true, passive: true });
  const deg = vargaDeg(id, d), lon = d === 1 ? lonOf(id) : null;
  const t = s('text', { x: x + (opts.compact ? 0 : 19), y, 'font-size': opts.fs || 12.5, 'font-weight': 700, fill: (G[id] || G.lagna).dot === '#E6E9F0' ? 'var(--ink)' : (G[id] || G.lagna).dot, class: 'gtxt' }, g);
  t.textContent = (G[id] || G.lagna).abbr;
  const f = flags(id, d); if (f) { const fs = s('tspan', { 'font-size': (opts.fs || 12.5) * .72, dy: -4, fill: 'var(--accent)' }, t); fs.textContent = f; }
  const t2 = s('tspan', { 'font-size': (opts.fs || 12.5) * .8, 'font-weight': 500, fill: 'var(--muted)', dy: f ? 4 : 0 }, t);
  t2.textContent = ' ' + (deg != null ? dm(deg) : '') + (lon != null && !opts.compact ? ' ' + NAK_AB[nakOf(lon)] : '');
  g.addEventListener('mousemove', ev => placeTip(ev, id, d)); g.addEventListener('mouseleave', untip);
  if (GR[id]) g.addEventListener('click', ev => { ev.stopPropagation(); go('#/planet/' + id); });
  return g;
}
function southGrid(svg, d, x0, y0, W, opts = {}) {
  const H = W * (opts.compact ? 1 : .86), pl = placements(d), lag = vargaSign('lagna', d), ar = d === 1 ? arudhaMarks() : {};
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg, x = x0 + c * W, y = y0 + r * H, g = s('g', { class: 'cell', 'data-sign': sign }, svg);
    s('rect', { x, y, width: W, height: H, class: 'cellbg' }, g);
    const small = opts.compact ? Math.max(10, W * .11) : 11;
    const nm = s('text', { x: x + W - 5, y: y + small + 2, 'text-anchor': 'end', 'font-size': small, fill: 'var(--muted)' }, g); nm.textContent = opts.compact ? SIGN_AB[sign] : SIGN_SKT[sign];
    if (lag != null) { const hs = s('text', { x: x + 5, y: y + small + 2, 'font-size': small, 'font-weight': 700, fill: 'var(--accent2)' }, g); hs.textContent = (sign - lag + 12) % 12 + 1; }
    if (lag === sign) s('path', { d: `M${x} ${y + 20}L${x + 20} ${y}`, stroke: 'var(--accent)', 'stroke-width': 2 }, g);
    const list = (lag === sign ? ['lagna'] : []).concat(pl[sign] || []);
    const fs = opts.compact ? Math.max(10.5, Math.min(18, W * .15 * (list.length > 4 ? .8 : 1))) : 12.5, lineH = opts.compact ? fs * 1.22 : 17, top = y + (opts.compact ? small * 2.4 : 32);
    list.forEach((gid, k) => grahaLine(g, gid, x + 6, top + k * lineH, d, { compact: opts.compact, fs }));
    if (d === 1) aspectMarks(g, sign, x + 6, y + H - 6, 'start', opts.compact ? Math.max(9, W * .1) : 10.5);
    if (ar[sign]) { const at = s('text', { x: x + W - 5, y: y + H - 6, 'text-anchor': 'end', 'font-size': 9.5, fill: '#a78bfa', 'font-weight': 700 }, g); at.textContent = ar[sign].join(' '); }
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
    const n = s('text', { x: cx, y: cy - (big ? S0 * .085 : S0 * .045), 'text-anchor': 'middle', 'font-size': opts.compact ? 10 : 12, 'font-weight': 800, fill: 'var(--accent)' }, g); n.textContent = sign + 1;
    const list = (hIdx === 0 ? ['lagna'] : []).concat(pl[sign] || []);
    const fs = opts.compact ? 9.5 : 11.5, lineH = opts.compact ? 11 : 14;
    list.forEach((gid, k) => { const tw = big ? S0 * .1 : S0 * .06; grahaLine(g, gid, cx - tw, cy - (big ? S0 * .04 : S0 * .005) + k * lineH, d, { compact: true, fs }); });
    if (d === 1) aspectMarks(g, sign, cx, cy + (big ? S0 * .13 : S0 * .085), 'middle', opts.compact ? 8.5 : 10);
    if (ar[sign]) { const at = s('text', { x: cx, y: cy + (big ? S0 * .16 : S0 * .11), 'text-anchor': 'middle', 'font-size': 9, fill: '#a78bfa', 'font-weight': 700 }, g); at.textContent = ar[sign].join(' '); }
    g.addEventListener('click', () => { if (d === 1) go('#/house/' + (hIdx + 1)); });
  });
}
function chartSVG(kind, d, opts = {}) {
  const svg = s('svg', { viewBox: kind === 'north' ? '-4 -4 448 448' : '-4 -4 568 491', role: 'img', 'aria-label': `D${d} ${kind === 'north' ? 'North' : 'South'} Indian chart`, class: 'chart ' + kind });
  if (kind === 'north') { northGrid(svg, d, 0, 0, 440, opts); const t = s('text', { x: 220, y: 436, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `D${d} · ${DIV_NAME[d] || ''} · North Indian`; }
  else southGrid(svg, d, 0, 0, 140, opts);
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
