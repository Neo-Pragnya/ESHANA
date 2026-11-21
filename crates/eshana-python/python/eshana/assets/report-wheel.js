/* ESHANA chart studio, part 2: the Panchang Chakra wheel and the South/North Indian charts.
   Rings, outside in: degree ticks · 27 nakshatras · 108 padas (coloured by navāṃśa element, labelled with the navāṃśa
   sign) · 36 dreshkāṇas (optional) · 12 rāśis · houses from the lagna (with SAV bindus as a layer) · the grahas,
   each with a needle to the exact degree, so it is visible which nakshatra, pada and navāṃśa a point falls in. */

const RINGS = { tick: [532, 548], nak: [472, 532], pada: [436, 472], dres: [404, 436], rashi: [330, 404], house: [292, 330], lanes: [252, 210, 168, 126] };
function angOf(lon) { return S.orientation === 'lagna' && ASC != null ? norm(lon - ASC) : norm(lon - 30); }
function pt(r, a) { return [r * Math.sin(a * DEG), -r * Math.cos(a * DEG)]; }
function arc(r0, r1, a0, a1) {
  const L = (a1 - a0) > 180 ? 1 : 0, [x0, y0] = pt(r1, a0), [x1, y1] = pt(r1, a1), [x2, y2] = pt(r0, a1), [x3, y3] = pt(r0, a0);
  return `M${x0.toFixed(2)} ${y0.toFixed(2)}A${r1} ${r1} 0 ${L} 1 ${x1.toFixed(2)} ${y1.toFixed(2)}L${x2.toFixed(2)} ${y2.toFixed(2)}A${r0} ${r0} 0 ${L} 0 ${x3.toFixed(2)} ${y3.toFixed(2)}Z`;
}
const NAKREF = D.nakshatra_reference || [];
function grahasIn(lo, hi) { return GRAHA_ORDER.filter(g => GR[g] && norm(GR[g].lon - lo) < hi - lo); }

/* ── detail panel ─────────────────────────────────────────────────────────────────────────────────────────────── */
let PANEL;
function facts(rows) { return h('dl', { class: 'facts' }, rows.filter(r => r && r[1] != null && r[1] !== '').flatMap(([k, v]) => [h('dt', {}, k), h('dd', {}, v)])); }
function grahaDetail(id) {
  const g = GR[id]; if (!g) return h('p', {}, 'Not in this chart.');
  const l = g.lon, nk = nakOf(l), st = (val('vedic.strength') || {})[id] || {}, kp = ((val('vedic.kp') || {}).planets || {})[id] || {};
  const casts = (val('vedic.drishti.graha') || []).filter(x => x.from === id).map(x => `${x.house_from_graha}th → ${SIGN_SKT[x.sign]}${x.aspects_grahas.length ? ' (' + x.aspects_grahas.map(a => G[a].abbr).join(', ') + ')' : ''}`);
  const gets = (val('vedic.drishti.graha') || []).filter(x => x.aspects_grahas.includes(id)).map(x => `${G[x.from].en} (${x.house_from_graha}th)`);
  const ref = NAKREF[nk] || {};
  return h('div', {},
    h('h3', {}, h('span', { style: `display:inline-block;width:14px;height:14px;border-radius:50%;background:${G[id].dot}` }), `${G[id].skt} · ${G[id].en}`),
    facts([
      ['Sidereal', `${l.toFixed(4)}° · ${SIGN_SKT[signOf(l)]} ${inSign(l)}`], ['Tropical', g.trop != null ? `${g.trop.toFixed(4)}° · ${SIGN_EN[signOf(g.trop)]}` : null],
      ['Nakshatra', `${NAKSHATRA[nk]} pada ${padaOf(l)} (lord ${G[NAK_LORD[nk % 9]].en})`], ['Navāṃśa (D9)', SIGN_SKT[navSignOf(l)]], ['Dreshkāṇa (D3)', SIGN_SKT[dresSignOf(l)]],
      ['Bhāva', LAGNA_SIGN == null ? null : String((signOf(l) - LAGNA_SIGN + 12) % 12 + 1)], ['Motion', `${g.speed >= 0 ? '+' : ''}${g.speed.toFixed(4)}°/day${g.retro ? ' · retrograde' : ''}`],
      ['Dignity', st.dignity], ['Avasthā', st.avastha], ['Combust', st.combust == null ? null : st.combust ? 'yes' : 'no'],
      ['Dig bala', st.dig_bala_virupa != null ? st.dig_bala_virupa.toFixed(1) + ' virupa' : null],
      ['KP lords', kp.star_lord ? `sign ${kp.sign_lord} · star ${kp.star_lord} · sub ${kp.sub_lord} · sub-sub ${kp.sub_sub_lord}` : null],
      ['Aspects cast', casts.join('; ')], ['Aspected by', gets.join(', ')], ['Markers (report.markers@0.1)', markers(l).join(', ')],
      ['Gana · yoni · nadi', ref.gana ? `${ref.gana} · ${ref.yoni} · ${ref.nadi}` : null],
    ]));
}
function showPanel(node, crumbs) {
  if (!PANEL) return;
  PANEL.replaceChildren(h('div', { class: 'crumbs' }, (crumbs || []).map(([t, fn]) => h('button', { onclick: fn }, t))), node);
}
function segmentDetail(kind, i) {
  if (kind === 'rashi') {
    const inside = grahasIn(i * 30, i * 30 + 30);
    return h('div', {}, h('h3', {}, `${GLYPH[i]} ${SIGN_SKT[i]} · ${SIGN_EN[i]}`),
      facts([['Span', `${i * 30}° – ${i * 30 + 30}°`], ['Lord', G[SIGN_LORD[i]].en], ['Element', ELEMENT[i % 4]], ['Modality', ['movable', 'fixed', 'dual'][i % 3]],
        ['Bhāva', LAGNA_SIGN == null ? null : String((i - LAGNA_SIGN + 12) % 12 + 1)],
        ['SAV bindus', val('vedic.ashtakavarga') ? String(val('vedic.ashtakavarga').sarvashtakavarga.bindus[i]) : null],
        ['Grahas', inside.map(g => `${G[g].en} ${inSign(GR[g].lon)}`).join(', ') || 'none'],
        ['Nakshatras', NAKSHATRA.filter((_, k) => k * NAKS < i * 30 + 30 && (k + 1) * NAKS > i * 30).join(', ')]]));
  }
  if (kind === 'nak') {
    const ref = NAKREF[i] || {}, inside = grahasIn(i * NAKS, (i + 1) * NAKS);
    return h('div', {}, h('h3', {}, `${NAKSHATRA[i]} (${i + 1})`),
      facts([['Span', `${(i * NAKS).toFixed(2)}° – ${((i + 1) * NAKS).toFixed(2)}° (${SIGN_SKT[Math.floor(i * NAKS / 30)]} ${inSign(i * NAKS)} – ${inSign((i + 1) * NAKS - 1e-9)})`],
        ['Lord', G[NAK_LORD[i % 9]].en], ['Padas → navāṃśa', [0, 1, 2, 3].map(p => `${p + 1}: ${SIGN_SKT[(i * 4 + p) % 12]}`).join(' · ')],
        ['Grahas', inside.map(g => `${G[g].en} pada ${padaOf(GR[g].lon)}`).join(', ') || 'none'],
        ...['nature', 'gana', 'yoni', 'gender', 'nadi', 'dosha', 'tattva', 'purpose', 'guna', 'direction'].map(k => [k[0].toUpperCase() + k.slice(1), ref[k]])]),
      h('p', { class: 'note' }, 'Attributes from the Nakshatra Ruler table (tradition_rule, shown for reference).'));
  }
  if (kind === 'pada') {
    const nk = Math.floor(i / 4), inside = grahasIn(i * NAV, (i + 1) * NAV);
    return h('div', {}, h('h3', {}, `${NAKSHATRA[nk]} pada ${i % 4 + 1}`),
      facts([['Span', `${SIGN_SKT[Math.floor(i * NAV / 30)]} ${inSign(i * NAV)} – ${inSign((i + 1) * NAV - 1e-9)}`], ['Navāṃśa sign', `${SIGN_SKT[i % 12]} (lord ${G[SIGN_LORD[i % 12]].en})`],
        ['Element', ELEMENT[(i % 12) % 4]], ['Markers (report.markers@0.1)', markers(i * NAV + NAV / 2).join(', ') || 'none'], ['Grahas', inside.map(g => `${G[g].en} ${inSign(GR[g].lon)}`).join(', ') || 'none']]));
  }
  const host = Math.floor(i / 3), part = i % 3, sign = (host + [0, 4, 8][part]) % 12;
  return h('div', {}, h('h3', {}, `Dreshkāṇa ${i + 1}`), facts([['Span', `${SIGN_SKT[host]} ${part * 10}° – ${part * 10 + 10}°`], ['D3 sign', `${SIGN_SKT[sign]} (lord ${G[SIGN_LORD[sign]].en})`], ['Grahas', grahasIn(i * 10, i * 10 + 10).map(g => G[g].en).join(', ') || 'none']]));
}

/* ── the wheel ────────────────────────────────────────────────────────────────────────────────────────────────── */
let WHEEL = null, transitIdx = 0, playing = null;
function renderWheel(host) {
  host.replaceChildren();
  const svg = s('svg', { viewBox: '-640 -640 1280 1280', role: 'img', 'aria-label': 'Panchang chakra with the grahas placed by sidereal longitude' }, host);
  const rot = s('g', { class: 'wheel-g' }, svg);
  const L = S.layers;
  const rashiOuter = L.dres ? RINGS.dres[0] : RINGS.pada[0];
  const segHandlers = (g, kind, i, title, rows) => {
    g.addEventListener('mousemove', ev => tip(ev, title, rows()));
    g.addEventListener('mouseleave', untip);
    g.addEventListener('click', () => focus(kind, i));
  };
  if (L.ticks) {
    for (let d = 0; d < 360; d++) {
      const a = angOf(d), long = d % 10 === 0 ? 16 : d % 5 === 0 ? 10 : 5;
      const [x0, y0] = pt(RINGS.tick[0], a), [x1, y1] = pt(RINGS.tick[0] + long, a);
      s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--muted)', 'stroke-width': d % 10 === 0 ? 1.4 : .7 }, rot);
    }
  }
  if (L.nak) for (let i = 0; i < 27; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'nak', 'data-i': i }, rot), lord = NAK_LORD[i % 9], fill = G[lord].nak;
    s('path', { d: arc(RINGS.nak[0], RINGS.nak[1], angOf(i * NAKS), angOf(i * NAKS) + NAKS), fill }, g);
    const mid = angOf((i + .5) * NAKS), [x, y] = pt((RINGS.nak[0] + RINGS.nak[1]) / 2, mid), r = mid <= 180 ? mid - 90 : mid + 90;
    const t = s('text', { class: 'lbl', x, y, 'font-size': 14, 'font-weight': 600, fill: ink(fill), transform: `rotate(${r.toFixed(1)} ${x.toFixed(1)} ${y.toFixed(1)})` }, g);
    t.textContent = NAK_SHORT[NAKSHATRA[i]] || NAKSHATRA[i];
    segHandlers(g, 'nak', i, NAKSHATRA[i], () => [['Lord', G[lord].en], ['Grahas', grahasIn(i * NAKS, (i + 1) * NAKS).map(x => G[x].en).join(', ') || '—'], ['Gana', (NAKREF[i] || {}).gana]]);
  }
  if (L.pada) for (let i = 0; i < 108; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'pada', 'data-i': i }, rot), nav = i % 12, fill = PADA_FILL[ELEMENT[nav % 4]];
    s('path', { d: arc(RINGS.pada[0], RINGS.pada[1], angOf(i * NAV), angOf(i * NAV) + NAV), fill }, g);
    const [x, y] = pt((RINGS.pada[0] + RINGS.pada[1]) / 2, angOf((i + .5) * NAV));
    const t = s('text', { class: 'lbl', x, y, 'font-size': 13, fill: '#1b1b1b' }, g); t.textContent = GLYPH[nav];
    const mk = markers(i * NAV + NAV / 2);
    if (mk.length) { const [mx, my] = pt(RINGS.pada[0] + 4, angOf((i + .5) * NAV)); s('circle', { cx: mx, cy: my, r: 3, fill: mk.includes('gandanta') ? '#D7141F' : mk.includes('vargottama') ? '#3ED9E4' : '#6FAF1F' }, g); }
    segHandlers(g, 'pada', i, `${NAKSHATRA[Math.floor(i / 4)]} pada ${i % 4 + 1}`, () => [['Navāṃśa', SIGN_SKT[nav]], ['Markers', mk.join(', ') || '—'], ['Grahas', grahasIn(i * NAV, (i + 1) * NAV).map(x => G[x].en).join(', ') || '—']]);
  }
  if (L.dres) for (let i = 0; i < 36; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'dres', 'data-i': i }, rot), sign = (Math.floor(i / 3) + [0, 4, 8][i % 3]) % 12, fill = G[SIGN_LORD[sign]].band;
    s('path', { d: arc(RINGS.dres[0], RINGS.dres[1], angOf(i * 10), angOf(i * 10) + 10), fill }, g);
    const [x, y] = pt((RINGS.dres[0] + RINGS.dres[1]) / 2, angOf(i * 10 + 5)); const t = s('text', { class: 'lbl', x, y, 'font-size': 12, fill: '#1b1b1b' }, g); t.textContent = GLYPH[sign];
    segHandlers(g, 'dres', i, `Dreshkāṇa ${i + 1}`, () => [['D3 sign', SIGN_SKT[sign]]]);
  }
  const sav = val('vedic.ashtakavarga');
  for (let i = 0; i < 12; i++) {
    const g = s('g', { class: 'seg', 'data-kind': 'rashi', 'data-i': i }, rot), fill = G[SIGN_LORD[i]].band;
    s('path', { d: arc(RINGS.rashi[0], rashiOuter, angOf(i * 30), angOf(i * 30) + 30), fill }, g);
    const [x, y] = pt((RINGS.rashi[0] + rashiOuter) / 2 + 12, angOf(i * 30 + 15)); const t = s('text', { class: 'lbl', x, y, 'font-size': 34, fill: '#1b1b1b' }, g); t.textContent = GLYPH[i];
    const [x2, y2] = pt(RINGS.rashi[0] + 16, angOf(i * 30 + 15)); const t2 = s('text', { class: 'lbl', x: x2, y: y2, 'font-size': 14, 'font-weight': 700, fill: '#1b1b1b' }, g); t2.textContent = SIGN_SKT[i];
    segHandlers(g, 'rashi', i, `${SIGN_SKT[i]} · ${SIGN_EN[i]}`, () => [['Lord', G[SIGN_LORD[i]].en], ['Grahas', grahasIn(i * 30, i * 30 + 30).map(x => G[x].en).join(', ') || '—'], ['SAV', sav ? sav.sarvashtakavarga.bindus[i] : null]]);
    if (L.houses) {
      const hg = s('g', {}, rot), house = LAGNA_SIGN == null ? null : (i - LAGNA_SIGN + 12) % 12 + 1;
      let hf = 'var(--disc)';
      if (L.sav && sav) { const b = sav.sarvashtakavarga.bindus[i]; const t01 = Math.max(0, Math.min(1, (b - 18) / 20)); hf = `hsl(${Math.round(10 + t01 * 120)} 70% ${S.dark ? 30 : 75}%)`; }
      s('path', { d: arc(RINGS.house[0], RINGS.house[1], angOf(i * 30), angOf(i * 30) + 30), fill: hf, stroke: 'var(--line)' }, hg);
      const [hx, hy] = pt((RINGS.house[0] + RINGS.house[1]) / 2, angOf(i * 30 + 15));
      const ht = s('text', { class: 'lbl', x: hx, y: hy, 'font-size': 14, 'font-weight': 700, fill: house === 1 ? 'var(--accent)' : 'var(--muted)' }, hg);
      ht.textContent = (house ? `${house}` : '') + (L.sav && sav ? ` · ${sav.sarvashtakavarga.bindus[i]}` : '');
    }
  }
  s('circle', { r: L.houses ? RINGS.house[0] : RINGS.rashi[0], fill: 'var(--disc)', stroke: 'var(--line)' }, rot);
  for (let i = 0; i < 12; i++) { const [x0, y0] = pt(96, angOf(i * 30)), [x1, y1] = pt(L.houses ? RINGS.house[0] : RINGS.rashi[0], angOf(i * 30)); s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--line)' }, rot); }
  const inner = s('g', {}, rot);
  // rashi drishti chords between occupied signs and the signs they aspect
  if (L.rashi_drishti) {
    const rd = s('g', { class: 'drishti' }, inner);
    for (const r of ((val('vedic.jaimini.rashi_drishti') || (val('vedic.drishti.rashi') || {}).aspects) || [])) for (const t of r.aspects_signs) {
      const [x0, y0] = pt(118, angOf(r.sign * 30 + 15)), [x1, y1] = pt(118, angOf(t * 30 + 15));
      s('path', { d: `M${x0} ${y0}L${x1} ${y1}`, stroke: G[r.from].dot, 'stroke-width': 1.2, opacity: .55 }, rd);
    }
  }
  // placed grahas, with lanes so close ones do not overlap
  const items = GRAHA_ORDER.filter(g => GR[g]).map(g => ({ g, a: angOf(GR[g].lon) })).sort((x, y) => x.a - y.a);
  const last = [null, null, null, null], pos = {};
  for (const it of items) { let lane = 0; for (; lane < 4; lane++) if (last[lane] == null || norm(it.a - last[lane]) >= 9) break; lane = Math.min(lane, 3); last[lane] = it.a; pos[it.g] = { a: it.a, r: RINGS.lanes[lane] }; }
  if (ASC != null) {
    const a = angOf(ASC), [x0, y0] = pt(96, a), [x1, y1] = pt(RINGS.tick[1] + 12, a);
    s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--accent)', 'stroke-width': 4 }, inner);
    const [lx, ly] = pt(RINGS.house[0] - 20, a), lg = s('g', { class: 'pl', 'data-body': 'lagna' }, inner);
    s('circle', { cx: lx, cy: ly, r: 19, fill: 'var(--accent)' }, lg); const lt = s('text', { class: 'lbl', x: lx, y: ly, 'font-size': 13, 'font-weight': 800, fill: '#fff' }, lg); lt.textContent = 'Asc';
    lg.addEventListener('mousemove', ev => tip(ev, 'Lagna', [['Sidereal', `${SIGN_SKT[signOf(ASC)]} ${inSign(ASC)}`], ['Nakshatra', `${NAKSHATRA[nakOf(ASC)]} pada ${padaOf(ASC)}`], ['Navāṃśa', SIGN_SKT[navSignOf(ASC)]]]));
    lg.addEventListener('mouseleave', untip);
  }
  // graha drishti arcs
  if (L.drishti) {
    const dg = s('g', { class: 'drishti' + (S.animate ? ' drawn' : '') }, inner);
    for (const d of val('vedic.drishti.graha') || []) {
      const p = pos[d.from]; if (!p) continue;
      const [x0, y0] = pt(p.r - 18, p.a), ta = angOf(d.sign * 30 + 15), [x1, y1] = pt(100, ta), [cx, cy] = pt(30, (p.a + ta) / 2 + (Math.abs(p.a - ta) > 180 ? 180 : 0));
      const path = s('path', { d: `M${x0.toFixed(1)} ${y0.toFixed(1)}Q${cx.toFixed(1)} ${cy.toFixed(1)} ${x1.toFixed(1)} ${y1.toFixed(1)}`, stroke: G[d.from].dot, class: d.kind === 'special' ? 'special' : '', 'data-from': d.from, 'marker-end': 'url(#arrow)' }, dg);
      path.style.setProperty('--len', 900);
      path.addEventListener('mousemove', ev => tip(ev, `${G[d.from].en} aspects the ${d.house_from_graha}th`, [['Sign', SIGN_SKT[d.sign]], ['Bhāva', d.aspects_bhava], ['Grahas there', d.aspects_grahas.map(x => G[x].en).join(', ') || '—'], ['Kind', d.kind]]));
      path.addEventListener('mouseleave', untip);
    }
    const defs = s('defs', {}, svg), m = s('marker', { id: 'arrow', viewBox: '0 0 10 10', refX: 8, refY: 5, markerWidth: 6, markerHeight: 6, orient: 'auto-start-reverse' }, defs);
    s('path', { d: 'M0 0L10 5L0 10z', fill: 'var(--muted)' }, m);
  }
  if (L.west_aspects && WES) {
    const wg = s('g', { class: 'asp' }, inner), wgr = grahasOf(WES), ayan = (VED.calculation || {}).ayanamsha_deg || 0;
    for (const a of WES.derived_attributes.filter(x => x.id.includes('.aspect.'))) {
      const [b1, b2] = a.derived_from.map(engineId); if (!wgr[b1] || !wgr[b2]) continue;
      const [x0, y0] = pt(104, angOf(wgr[b1].lon - ayan)), [x1, y1] = pt(104, angOf(wgr[b2].lon - ayan));
      const ln = s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: ASPECT_COLOUR[a.value.aspect] || '#999' }, wg);
      ln.addEventListener('mousemove', ev => tip(ev, `${G[b1].en} ${a.value.aspect} ${G[b2].en}`, [['Orb', a.value.orb_deg.toFixed(2) + '°'], ['Phase', a.value.applying ? 'applying' : 'separating']]));
      ln.addEventListener('mouseleave', untip);
    }
  }
  const pg = s('g', {}, inner);
  for (const it of items) {
    const p = pos[it.g], gr = GR[it.g];
    if (L.needles) { const [x0, y0] = pt(p.r + 24, p.a), [x1, y1] = pt(rashiOuter + (L.pada ? 36 : 0), p.a); s('line', { class: 'needle', x1: x0, y1: y0, x2: x1, y2: y1, stroke: G[it.g].dot, 'stroke-dasharray': '4 3' }, pg);
      const [tx, ty] = pt(rashiOuter + (L.pada ? 36 : 0), p.a); s('circle', { cx: tx, cy: ty, r: 4.5, fill: G[it.g].dot, stroke: '#111' }, pg); }
    const [x, y] = pt(p.r, p.a);
    const m = marker(pg, it.g, x, y, 21, { retro: gr.retro && !['rahu', 'ketu'].includes(it.g) });
    m.dataset.a = p.a; m.dataset.r = p.r;
    m.addEventListener('mousemove', ev => tip(ev, `${G[it.g].skt} · ${G[it.g].en}`, [['Sidereal', `${SIGN_SKT[signOf(gr.lon)]} ${inSign(gr.lon)}`], ['Nakshatra', `${NAKSHATRA[nakOf(gr.lon)]} pada ${padaOf(gr.lon)}`], ['Navāṃśa', SIGN_SKT[navSignOf(gr.lon)]], ['Motion', gr.retro ? 'retrograde' : 'direct']]));
    m.addEventListener('mouseleave', () => { untip(); hot([]); });
    m.addEventListener('mouseenter', () => hot([['nak', nakOf(gr.lon)], ['pada', Math.floor(norm(gr.lon) / NAV)], ['rashi', signOf(gr.lon)], ['dres', Math.floor(norm(gr.lon) / 10)]]));
    m.addEventListener('click', () => { $$('.pl.sel').forEach(x => x.classList.remove('sel')); m.classList.add('sel'); showPanel(grahaDetail(it.g), [['Chakra', resetFocus]]); });
  }
  // transit ring
  if (L.transits && D.transits && D.transits.rows && D.transits.rows.length) {
    s('circle', { r: 598, fill: 'none', stroke: 'var(--line)', 'stroke-dasharray': '3 6' }, rot);
    WHEEL = { svg, rot, tg: s('g', {}, rot) };
    drawTransit();
  } else WHEEL = { svg, rot };
  const c1 = s('text', { class: 'lbl', y: -14, 'font-size': 24, 'font-weight': 800, fill: 'var(--ink)' }, svg); c1.textContent = S.varga === 1 ? 'Rāśi chakra' : 'Rāśi chakra';
  const c2 = s('text', { class: 'lbl', y: 16, 'font-size': 13, fill: 'var(--muted)' }, svg); c2.textContent = `${(VED.calculation || {}).ayanamsha || ''} · ${(VED.calculation || {}).provider || ''}`;
  if (S.animate) animateIn(pg);
}
function hot(list) {
  const svg = WHEEL && WHEEL.svg; if (!svg) return;
  $$('.seg.hot', svg).forEach(x => x.classList.remove('hot'));
  for (const [k, i] of list) { const e = svg.querySelector(`.seg[data-kind="${k}"][data-i="${i}"]`); if (e) e.classList.add('hot'); }
  svg.classList.toggle('dimmed', list.length > 0);
}
function animateIn(pg) {
  const els = $$('.pl', pg); const t0 = performance.now(), dur = 1400;
  const target = els.map(e => ({ e, a: +e.dataset.a, r: +e.dataset.r }));
  function step(t) {
    const k = Math.min(1, (t - t0) / dur), ease = 1 - Math.pow(1 - k, 3);
    for (const { e, a, r } of target) { if (isNaN(a)) continue; const [x, y] = pt(r * (.35 + .65 * ease), a * ease); e.setAttribute('transform', `translate(${x.toFixed(1)} ${y.toFixed(1)})`); }
    if (k < 1) requestAnimationFrame(step);
  }
  requestAnimationFrame(step);
}
let FOCUS = null;
function focus(kind, i) {
  const span = { rashi: 30, nak: NAKS, pada: NAV, dres: 10 }[kind], mid = angOf(i * span + span / 2);
  FOCUS = [kind, i];
  if (WHEEL) WHEEL.rot.style.transform = `rotate(${-mid}deg)`;
  hot([[kind, i]]);
  const crumbs = [['⟲ Chakra', resetFocus]];
  if (kind !== 'rashi') { const r = Math.floor((kind === 'nak' ? i * NAKS : kind === 'pada' ? i * NAV : i * 10) / 30); crumbs.push([SIGN_SKT[r], () => focus('rashi', r)]); }
  if (kind === 'pada') crumbs.push([NAKSHATRA[Math.floor(i / 4)], () => focus('nak', Math.floor(i / 4))]);
  showPanel(segmentDetail(kind, i), crumbs);
}
function resetFocus() { FOCUS = null; if (WHEEL) WHEEL.rot.style.transform = ''; hot([]); showPanel(h('p', { class: 'note' }, 'Hover any ring to see what falls there; click a rāśi, nakshatra or pada to turn the wheel to it, or a graha for its details. Esc returns.'), []); }
document.addEventListener('keydown', e => { if (e.key === 'Escape') resetFocus(); });

/* transit overlay: rows of sidereal longitudes; the slider and play button move through time */
function drawTransit() {
  const rows = D.transits.rows; if (!WHEEL || !WHEEL.tg) return;
  const row = rows[Math.max(0, Math.min(rows.length - 1, transitIdx))]; WHEEL.tg.replaceChildren();
  const lons = row.longitudes;
  for (const k in lons) {
    const id = engineId(k); if (!G[id]) continue; const a = angOf(lons[k]), [x, y] = pt(598, a);
    const m = marker(WHEEL.tg, id, x, y, 15);
    m.addEventListener('mousemove', ev => tip(ev, `Transit ${G[id].en}`, [['Date', dateOnly(row.utc)], ['Sidereal', `${SIGN_SKT[signOf(lons[k])]} ${inSign(lons[k])}`], ['Over natal bhāva', LAGNA_SIGN == null ? null : (signOf(lons[k]) - LAGNA_SIGN + 12) % 12 + 1]]));
    m.addEventListener('mouseleave', untip);
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

/* ── South and North Indian charts (any varga) ────────────────────────────────────────────────────────────────── */
const SOUTH_CELL = { 11: [0, 0], 0: [1, 0], 1: [2, 0], 2: [3, 0], 3: [3, 1], 4: [3, 2], 5: [3, 3], 6: [2, 3], 7: [1, 3], 8: [0, 3], 9: [0, 2], 10: [0, 1] };
function placements(d) {
  const out = {}; for (const g of GRAHA_ORDER) { if (!GR[g]) continue; const sg = vargaSign(g, d); if (sg == null) continue; (out[sg] = out[sg] || []).push(g); }
  return out;
}
function arudhaMarks() {
  const a = val('vedic.jaimini.arudha_padas'); if (!a || !S.layers.arudha) return {};
  const out = {}; for (const k in a) { const label = k.startsWith('A1_') ? 'AL' : k.startsWith('A12') ? 'UL' : k.split('_')[0]; (out[a[k].pada_sign] = out[a[k].pada_sign] || []).push(label); }
  return out;
}
function houseClick(sign, d) {
  const inside = placements(d)[sign] || [], house = LAGNA_SIGN == null ? null : (sign - (d === 1 ? LAGNA_SIGN : vargaSign('lagna', d)) + 12) % 12 + 1;
  const asp = (val('vedic.drishti.graha') || []).filter(x => x.sign === sign).map(x => `${G[x.from].en} (${x.house_from_graha}th)`);
  const kpc = d === 1 && house ? ((val('vedic.kp') || {}).cusps || {})[house] : null;
  showPanel(h('div', {}, h('h3', {}, `${SIGN_SKT[sign]} · D${d}${house ? ' · bhāva ' + house : ''}`),
    facts([['Lord', G[SIGN_LORD[sign]].en], ['Grahas', inside.map(g => G[g].en).join(', ') || 'none'], ['Aspected by (graha drishti)', d === 1 ? asp.join(', ') || 'none' : null],
      ['SAV bindus', d === 1 && val('vedic.ashtakavarga') ? val('vedic.ashtakavarga').sarvashtakavarga.bindus[sign] : null],
      ['KP cusp', kpc ? `${kpc.cusp.toFixed(2)}° · star ${kpc.lords.star_lord} · sub ${kpc.lords.sub_lord}` : null]])), [['⟲ Chakra', resetFocus]]);
}
function southChart(host, d) {
  const W = 130, H = 106, svg = s('svg', { viewBox: `-4 -4 ${4 * W + 8} ${4 * H + 8}`, role: 'img', 'aria-label': `D${d} South Indian chart` }, host);
  const pl = placements(d), lag = d === 1 ? LAGNA_SIGN : vargaSign('lagna', d), ar = d === 1 ? arudhaMarks() : {};
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg, x = c * W, y = r * H, g = s('g', { class: 'seg', style: 'cursor:pointer' }, svg);
    s('rect', { x, y, width: W, height: H, fill: 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 1.4 }, g);
    const t = s('text', { x: x + W - 6, y: y + 14, 'text-anchor': 'end', 'font-size': 11, fill: 'var(--muted)' }, g); t.textContent = SIGN_SKT[sign];
    if (lag === sign) { s('line', { x1: x, y1: y + 24, x2: x + 24, y2: y, stroke: 'var(--accent)', 'stroke-width': 2 }, g); const a = s('text', { x: x + 4, y: y + 14, 'font-size': 11, 'font-weight': 800, fill: 'var(--accent)' }, g); a.textContent = 'As'; }
    (pl[sign] || []).forEach((gid, k) => { const col = k % 3, row = Math.floor(k / 3); const m = marker(g, gid, x + 22 + col * 42, y + 40 + row * 34, 12, { retro: GR[gid].retro && !['rahu', 'ketu'].includes(gid) });
      m.addEventListener('mousemove', ev => tip(ev, G[gid].en, [['In', `${SIGN_SKT[signOf(GR[gid].lon)]} ${inSign(GR[gid].lon)}`], ['Nakshatra', `${NAKSHATRA[nakOf(GR[gid].lon)]} ${padaOf(GR[gid].lon)}`]])); m.addEventListener('mouseleave', untip); });
    if (ar[sign]) { const at = s('text', { x: x + 6, y: y + H - 6, 'font-size': 10.5, fill: '#7c3aed', 'font-weight': 700 }, g); at.textContent = ar[sign].join(' '); }
    g.addEventListener('click', () => houseClick(sign, d));
  }
  s('rect', { x: W, y: H, width: 2 * W, height: 2 * H, fill: 'var(--chip)', stroke: 'var(--ink)', 'stroke-width': 1.4 }, svg);
  const t = s('text', { x: 2 * W, y: 2 * H - 8, 'text-anchor': 'middle', 'font-size': 18, 'font-weight': 800, fill: 'var(--ink)' }, svg); t.textContent = `D${d} · ${DIV_NAME[d] || ''}`;
  const t2 = s('text', { x: 2 * W, y: 2 * H + 16, 'text-anchor': 'middle', 'font-size': 12, fill: 'var(--muted)' }, svg); t2.textContent = 'South Indian';
  if (d === 1 && S.layers.chart_drishti) drishtiArrows(svg, sign => { const [c, r] = SOUTH_CELL[sign]; return [c * W + W / 2, r * H + H / 2]; });
}
function northChart(host, d) {
  const S0 = 420, svg = s('svg', { viewBox: `-4 -4 ${S0 + 8} ${S0 + 8}`, role: 'img', 'aria-label': `D${d} North Indian chart` }, host);
  const lag = d === 1 ? LAGNA_SIGN : vargaSign('lagna', d);
  if (lag == null) { const t = s('text', { x: S0 / 2, y: S0 / 2, 'text-anchor': 'middle', fill: 'var(--muted)' }, svg); t.textContent = 'needs the lagna (exact time and place)'; return; }
  const A = [0, 0], B = [S0, 0], C = [S0, S0], Dd = [0, S0], T = [S0 / 2, 0], R = [S0, S0 / 2], Bm = [S0 / 2, S0], L = [0, S0 / 2], O = [S0 / 2, S0 / 2];
  const P1 = [S0 / 4, S0 / 4], P2 = [3 * S0 / 4, S0 / 4], P3 = [3 * S0 / 4, 3 * S0 / 4], P4 = [S0 / 4, 3 * S0 / 4];
  const H = [[T, P2, O, P1], [A, T, P1], [A, P1, L], [L, P1, O, P4], [L, P4, Dd], [Dd, P4, Bm], [Bm, P4, O, P3], [Bm, P3, C], [C, P3, R], [R, P3, O, P2], [R, P2, B], [B, P2, T]];
  const pl = placements(d), ar = d === 1 ? arudhaMarks() : {}, centre = {};
  s('rect', { width: S0, height: S0, fill: 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 1.6 }, svg);
  H.forEach((poly, hIdx) => {
    const sign = (lag + hIdx) % 12, cx = poly.reduce((a, p) => a + p[0], 0) / poly.length, cy = poly.reduce((a, p) => a + p[1], 0) / poly.length, big = poly.length === 4;
    centre[sign] = [cx, cy];
    const g = s('g', { class: 'seg', style: 'cursor:pointer' }, svg);
    s('polygon', { points: poly.map(p => p.join(',')).join(' '), fill: hIdx === 0 ? 'var(--chip)' : 'transparent', stroke: 'var(--ink)', 'stroke-width': 1.4 }, g);
    const n = s('text', { x: cx, y: cy - (big ? 34 : 16), 'text-anchor': 'middle', 'font-size': 12, 'font-weight': 700, fill: 'var(--accent)' }, g); n.textContent = sign + 1;
    (pl[sign] || []).forEach((gid, k) => { const per = big ? 3 : 2, col = k % per, row = Math.floor(k / per); marker(g, gid, cx + (col - (per - 1) / 2) * 36, cy - (big ? 6 : -2) + row * 30, 11, { retro: GR[gid].retro && !['rahu', 'ketu'].includes(gid) }); });
    if (ar[sign]) { const at = s('text', { x: cx, y: cy + (big ? 46 : 24), 'text-anchor': 'middle', 'font-size': 10, fill: '#7c3aed', 'font-weight': 700 }, g); at.textContent = ar[sign].join(' '); }
    g.addEventListener('click', () => houseClick(sign, d));
  });
  const t = s('text', { x: S0 / 2, y: S0 - 8, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `D${d} · ${DIV_NAME[d] || ''} · North Indian`;
  if (d === 1 && S.layers.chart_drishti) drishtiArrows(svg, sign => centre[sign]);
}
function drishtiArrows(svg, at) {
  const g = s('g', { class: 'drishti' + (S.animate ? ' drawn' : '') }, svg);
  const defs = s('defs', {}, svg), m = s('marker', { id: 'arr2', viewBox: '0 0 10 10', refX: 8, refY: 5, markerWidth: 5, markerHeight: 5, orient: 'auto-start-reverse' }, defs);
  s('path', { d: 'M0 0L10 5L0 10z', fill: 'var(--muted)' }, m);
  for (const d of val('vedic.drishti.graha') || []) {
    if (!d.aspects_grahas.length && d.kind === 'full') continue; // keep the chart readable: special aspects and aspects that land on grahas
    const from = GR[d.from] && at(signOf(GR[d.from].lon)), to = at(d.sign); if (!from || !to) continue;
    const mx = (from[0] + to[0]) / 2 + (to[1] - from[1]) * .15, my = (from[1] + to[1]) / 2 - (to[0] - from[0]) * .15;
    const p = s('path', { d: `M${from[0]} ${from[1]}Q${mx} ${my} ${to[0]} ${to[1]}`, stroke: G[d.from].dot, 'stroke-width': 1.6, class: d.kind === 'special' ? 'special' : '', 'marker-end': 'url(#arr2)', opacity: .7 }, g);
    p.style.setProperty('--len', 600);
  }
}
