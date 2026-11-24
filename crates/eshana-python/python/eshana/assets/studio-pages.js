/* ESHANA Studio, part 3: the pages, the navigation, the command palette and boot. */

/* ── building blocks ──────────────────────────────────────────────────────────────────────────────────────────── */
function facts(rows) { return h('dl', { class: 'facts' }, rows.filter(r => r && r[1] != null && r[1] !== '' && r[1] !== '—').flatMap(([k, v]) => [h('dt', {}, k), h('dd', {}, v)])); }
function table(head, rows, opts = {}) {
  return h('div', { class: 'tbl-wrap' }, h('table', {}, h('thead', {}, h('tr', {}, head.map(c => h('th', {}, c)))),
    h('tbody', {}, rows.map((r, i) => h('tr', Object.assign({}, opts.rowAttrs ? opts.rowAttrs(i) : {}), r.map(c => h('td', c && c.td ? c.td : {}, c && c.nodeType ? c : c && c.td ? c.v : c == null ? '—' : String(c))))))));
}
const card = (title, ...kids) => h('div', { class: 'card' }, title ? h('h3', {}, title) : null, ...kids);
const stat = (label, value, sub, href) => h(href ? 'a' : 'div', { class: 'stat', href }, h('div', { class: 'stat-l' }, label), h('div', { class: 'stat-v' }, value), sub ? h('div', { class: 'stat-s' }, sub) : null);
const dot = id => h('span', { class: 'dot', style: `background:${(G[id] || G.lagna).dot}` });
const glink = id => GR[id] ? h('a', { href: `#/planet/${id}`, class: 'glink' }, dot(id), G[id].en) : h('span', {}, dot(id), (G[id] || G.lagna).en);
const label = ep => h('span', { class: `ep ep-${ep}` }, ep);
const note = (...t) => h('p', { class: 'note' }, ...t);
const bar = (v, max, col) => h('div', { class: 'bar' }, h('i', { style: `width:${Math.max(0, Math.min(100, v / max * 100))}%;${col ? 'background:' + col : ''}` }));
const tabs = (items, cur, onPick) => h('div', { class: 'tabs', role: 'tablist' }, items.map(([k, t]) => h('button', { class: 'tab', role: 'tab', 'aria-selected': String(k === cur), onclick: () => onPick(k) }, t)));
const visToggle = (key, text) => h('label', { class: 'tog' }, h('input', { type: 'checkbox', checked: S[key] !== false, onchange: e => { S[key] = e.target.checked; save(); applyVisibility(); route(); } }), text);
const visGroup = () => h('div', { class: 'grp' }, h('b', {}, 'Include'), visToggle('showOuter', 'Uranus, Neptune, Pluto'), visToggle('showUpa', 'upagrahas'));
const toggle = (key, text, after) => h('label', { class: 'tog' }, h('input', { type: 'checkbox', checked: !!S.layers[key], onchange: e => { S.layers[key] = e.target.checked; save(); after(); } }), text);
/* additional aspects shown under the two switches: graha drishti (and exchange marks) and rāśi drishti */
const extraShown = d => d.drishti === 'rashi' ? !!S.layers.extra_rashi : !!S.layers.extra_aspects;
const pageHead = (sig, title, sub) => h('header', { class: 'phead' }, h('span', { class: 'phead-sigil', html: sigil(sig, 'big') }), h('div', {}, h('h2', {}, title), sub ? h('p', { class: 'lead' }, sub) : null));
const T = iso => Date.parse(iso);
const missing = (what, why) => h('div', { class: 'card empty' }, h('p', {}, `${what} is not available for this chart.`), note(why));

/* ── overview ─────────────────────────────────────────────────────────────────────────────────────────────────── */
page('overview', () => 'Overview', 'Chart', (b) => {
  const inp = (VED || WES || HD || {}).input || {}, subj = D.subject || {}, p = val('vedic.panchanga'), vim = val('vedic.dasha.vimshottari');
  const sb = val('vedic.shadbala'), bg = HD ? (attrsOf(HD)['human_design.bodygraph'] || {}).value : null;
  const strongest = sb ? sb.strongest_by_ratio[0] : null, ypres = YOGAS ? YOGAS.yogas.filter(y => y.present) : [];
  const chain = vim && vim.current ? vim.current.periods.slice(0, 3).map(x => x.lord).join(' › ') : null;
  const tiles = [
    ASC != null && stat('Lagna', `${SIGN_SKT[signOf(ASC)]} ${inSignDM(ASC)}`, `${NAKSHATRA[nakOf(ASC)]} ${padaOf(ASC)}`, '#/house/1'),
    GR.moon && stat('Janma nakshatra', NAKSHATRA[nakOf(GR.moon.lon)], `pada ${padaOf(GR.moon.lon)} · Moon in ${SIGN_SKT[signOf(GR.moon.lon)]}`, '#/planet/moon'),
    GR.sun && stat('Sun', `${SIGN_SKT[signOf(GR.sun.lon)]} ${inSignDM(GR.sun.lon)}`, NAKSHATRA[nakOf(GR.sun.lon)], '#/planet/sun'),
    chain && stat('Current daśā', chain, REF_UTC ? 'at ' + dateOnly(REF_UTC) : '', '#/dasha'),
    p && stat('Tithi · vāra', `${p.tithi.paksha === 'shukla' ? 'Śukla' : 'Kṛṣṇa'} ${p.tithi.name}`, p.vara ? p.vara.name : '', '#/panchanga'),
    strongest && stat('Strongest by ṣaḍbala', G[strongest].en, `${fix(sb.grahas[strongest].ratio)} × required`, '#/strength'),
    YOGAS && stat('Yogas present', String(ypres.length), ypres.slice(0, 3).map(y => y.name.split(' (')[0]).join(', '), '#/yogas'),
    bg && stat('Human Design', `${bg.type}`, `${bg.authority} · ${bg.profile}`, '#/human-design'),
  ];
  const mini = h('div', { class: 'figure' }); if (VED) mini.append(chartSVG(S.chart === 'north' ? 'north' : 'south', 1));
  const mini9 = h('div', { class: 'figure' }); if (VED && DIVS.includes(9)) mini9.append(chartSVG(S.chart === 'north' ? 'north' : 'south', 9));
  const explore = Object.values(PAGES).filter(x => !x.hidden && x.id !== 'overview').map(x => h('a', { class: 'explore', href: '#/' + x.id }, h('span', { html: sigil(x.id) }), h('b', {}, x.title()), h('small', {}, x.blurb || '')));
  put(b, pageHead('overview', subj.name || 'Birth chart', [subj.local, subj.place_label].filter(Boolean).join(' · ')),
    h('div', { class: 'stats' }, tiles),
    h('div', { class: 'grid2' }, card('Rāśi (D1)', mini), card('Navāṃśa (D9)', mini9)),
    card('Birth data', facts([['Instant (UTC)', inp.instant_utc], ['Zone', inp.civil_zone ? `${inp.civil_zone} (${(inp.civil_offset_s / 3600).toFixed(2)} h)` : null],
      ['Latitude, longitude', inp.place ? `${inp.place.latitude_deg.toFixed(4)}°, ${inp.place.longitude_deg.toFixed(4)}°` : null], ['Birth time', inp.birth_time_quality],
      ['Julian day (UT)', inp.jd_ut && inp.jd_ut.toFixed(6)], ['Delta T', inp.delta_t_s && inp.delta_t_s.toFixed(2) + ' s'],
      ['Ayanāṃśa', VED ? `${VED.calculation.ayanamsha} ${dms(VED.calculation.ayanamsha_deg)}` : null], ['Provider', (VED || WES || {}).calculation ? (VED || WES).calculation.provider : null]])),
    h('h3', { class: 'section-t' }, 'Explore'), h('div', { class: 'explore-grid' }, explore));
}, { blurb: 'the essentials at a glance' });

/* ── chakra ───────────────────────────────────────────────────────────────────────────────────────────────────── */
page('chakra', () => 'Jyotish Chakra', 'Chart', (b) => {
  if (!VED) return put(b, missing('The chakra', 'It needs a Vedic chart.'));
  const host = h('div', { class: 'figure wheel-host' }), panel = h('div', { class: 'card panel' });
  // hover details go here, beside the wheel, instead of a popup on top of the chakra
  const idle = '<p class="note">Hover a graha, rāśi, nakshatra or pada: its details appear here, so the aspects on the wheel stay in view.</p>';
  const dock = h('div', { class: 'card tipdock', 'aria-live': 'polite', 'data-idle': idle }, h('div', { class: 'tipdock-t' }, 'Hover details'), h('div', { class: 'tipdock-body' }));
  dock.querySelector('.tipdock-body').innerHTML = idle;
  const side = h('div', { class: 'chakra-side' }, dock, panel);
  const redraw = () => route();
  const transit = D.transits && D.transits.rows && D.transits.rows.length;
  const centre = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.centre === v), onclick: () => { S.centre = v; save(); redraw(); } }, t);
  put(b, pageHead('chakra', 'Jyotish Chakra', 'The Graha-Nakshatra Mandala: every graha placed by its exact degree on the nakshatra, pada (navāṃśa) and rāśi rings.'),
    h('div', { class: 'toolbar' },
      h('div', { class: 'grp' }, h('b', {}, 'Centre'), centre('south', 'South Indian'), centre('north', 'North Indian'), centre('lanes', 'Orbit lanes')), visGroup(),
      h('div', { class: 'grp' }, h('b', {}, 'Rings'), toggle('markers', 'markers', redraw), toggle('ticks', 'degrees', redraw), toggle('nak', 'nakshatras', redraw), toggle('pada', 'padas', redraw), toggle('dres', 'dreshkāṇas', redraw), toggle('houses', 'bhāvas', redraw), toggle('sav', 'SAV', redraw)),
      h('div', { class: 'grp' }, h('b', {}, 'Layers'), toggle('needles', 'needles', redraw), toggle('drishti', 'drishti arcs', redraw), toggle('aspect_marks', 'graha aspects', redraw), toggle('rashi_marks', 'rāśi aspects (◇)', redraw), toggle('rashi_drishti', 'rāśi drishti lines', redraw), toggle('arudha', 'ārūḍha padas', redraw), toggle('west_aspects', 'Western aspects', redraw), transit ? toggle('transits', 'transits', redraw) : null),
      h('div', { class: 'grp' }, h('b', {}, 'Additional'), toggle('extra_aspects', 'additional graha aspects', redraw), toggle('extra_rashi', 'additional rāśi aspects', redraw)),
      h('div', { class: 'grp' }, h('b', {}, 'Turn'), h('button', { class: 'tab', 'aria-selected': String(S.orientation === 'template'), onclick: () => { S.orientation = 'template'; save(); redraw(); } }, 'Mesha at 11'),
        h('button', { class: 'tab', 'aria-selected': String(S.orientation === 'lagna'), disabled: ASC == null, onclick: () => { S.orientation = 'lagna'; save(); redraw(); } }, 'Lagna at top'),
        h('label', { class: 'tog' }, h('input', { type: 'checkbox', id: 'turn-on-select', checked: S.turnOnSelect !== false, onchange: e => { S.turnOnSelect = e.target.checked; save(); if (FOCUS) focusSeg(FOCUS[0], FOCUS[1], panel); } }), 'turn to selection'))),
    transit && S.layers.transits ? transitControls() : null,
    h('h3', { class: 'mandala-t' }, 'Graha-Nakshatra Mandala'), h('div', { class: 'chakra-layout' }, host, side),
    h('div', { class: 'legend' }, h('span', {}, h('i', { style: 'background:#6FAF1F' }), 'pushkara navāṃśa'), h('span', {}, h('i', { style: 'background:#3ED9E4' }), 'vargottama'), h('span', {}, h('i', { style: 'background:#D7141F' }), 'gaṇḍānta'),
      h('span', {}, 'Ju Sa: translucent marks show which grahas aspect a sign (* special aspect); ◇Ju: rāśi drishti'), h('span', {}, 'hover a graha to watch its aspects'), h('span', {}, 'flags: R retrograde · C combust · ↑ exalted · ↓ debilitated · △ mūlatrikoṇa · ◎ own · V vargottama')));
  b.append(grahaLegend());
  renderWheel(host, panel); resetFocus(panel);
}, { blurb: 'nakshatras, padas, navāṃśa and every point' });

/* ── charts ───────────────────────────────────────────────────────────────────────────────────────────────────── */
page('charts', () => 'Charts', 'Chart', (b) => {
  if (!VED) return put(b, missing('Charts', 'They need a Vedic chart.'));
  const re = () => route();
  const style = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.chart === v), onclick: () => { S.chart = v; save(); re(); } }, t);
  const icons = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.icons === v), onclick: () => { S.icons = v; save(); re(); } }, t);
  const kind = S.chart === 'north' ? 'north' : 'south';
  // the D1 charts get full-width rows so grahas, padas and every aspect mark have room; the rest share a grid
  const wide = h('div', { class: 'wide-charts' });
  wide.append(card(kind === 'north' ? 'Rāśi · D1 (North Indian)' : 'Rāśi · D1', h('div', { class: 'figure wide' }, chartSVG(kind, 1))));
  if (S.chart === 'both') wide.append(card('Rāśi · D1 (North Indian)', h('div', { class: 'figure wide' }, chartSVG('north', 1))));
  const grid = h('div', { class: 'grid2' });
  const ov = S.chalit || 'none';
  if (ov !== 'none') {
    const fig = ov === 'circle' ? chalitCircle() : kind === 'south' ? chalitSquare() : null;
    grid.append(card(ov === 'circle' ? 'Rāśi and bhāva chalit · circle' : 'Rāśi and bhāva chalit · concentric squares',
      fig ? h('div', { class: 'figure' }, fig) : note(val('vedic.bhava_chalit') ? 'The concentric overlay is drawn on the South Indian chart; the North Indian chart is house-based, so choose the circle.' : 'The bhāva chalit needs an exact birth time and place.'),
      note('Grahas marked → (square) or ringed (circle) fall in a different bhāva in the chalit than in the rāśi chart.')));
  }
  if (DIVS.includes(9)) grid.append(card('Navāṃśa · D9', h('div', { class: 'figure' }, chartSVG(kind, 9))));
  const bc = val('vedic.bhava_chalit');
  if (bc) {
    const rows = Object.entries(bc.grahas).filter(([g]) => G[g] && GR[g]).map(([g, hh]) => [glink(g), `${SIGN_SKT[signOf(GR[g].lon)]} ${inSignDM(GR[g].lon)}`, houseOf(signOf(GR[g].lon)), hh, hh !== houseOf(signOf(GR[g].lon)) ? h('b', {}, 'shifts') : '']);
    grid.append(card('Bhāva chalit (Śrīpati)', table(['Graha', 'Position', 'Rāśi bhāva', 'Chalit bhāva', ''], rows), note('Bhāva madhyas and sandhis from Porphyry trisection (matches the Swiss Ephemeris Śrīpati houses within 0.2″).')));
  }
  grid.append(card('Sudarshana chakra', h('div', { class: 'figure narrow' }, sudarshana())));
  put(b, pageHead('charts', 'Charts', 'Rāśi, navāṃśa, bhāva chalit and Sudarshana. Click a house for its page, a graha for its own.'),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'Style'), style('south', 'South Indian'), style('north', 'North Indian'), style('both', 'Both')),
      h('div', { class: 'grp' }, h('b', {}, 'Icons'), icons('sphere', 'Spheres'), icons('glyph', 'Glyphs'), icons('badge', 'Badges')), visGroup(),
      h('div', { class: 'grp' }, h('b', {}, 'Bhāva chalit'), ...[['none', 'Off'], ['square', 'Concentric squares'], ['circle', 'Circle']].map(([v, t]) => h('button', { class: 'tab', 'aria-selected': String((S.chalit || 'none') === v), onclick: () => { S.chalit = v; save(); re(); } }, t))),
      h('div', { class: 'grp' }, h('b', {}, 'Show'), toggle('aspect_marks', 'graha aspects', re), toggle('rashi_marks', 'rāśi aspects (◇)', re), toggle('arudha', 'ārūḍha padas', re)),
      h('div', { class: 'grp' }, h('b', {}, 'Additional'), toggle('extra_aspects', 'additional graha aspects', re), toggle('extra_rashi', 'additional rāśi aspects', re))),
    wide, grid, grahaLegend(), note('In each house: graha (higher degree higher), degree, nakshatra; small marks R retrograde, C combust, ↑ exalted, ↓ debilitated, △ mūlatrikoṇa, ◎ own sign, V vargottama. Translucent letters along the bottom of a house are the grahas aspecting it (graha drishti; * special); ◇ letters above them are rāśi drishti. Hover a graha to light up the houses it aspects.'));
}, { blurb: 'rāśi, navāṃśa, bhāva chalit, Sudarshana' });

/* ── vargas ───────────────────────────────────────────────────────────────────────────────────────────────────── */
page('vargas', () => 'Vargas', 'Chart', (b) => {
  if (!VED || DIVS.length < 2) return put(b, missing('Divisional charts', 'Ask for divisions.'));
  const kind = S.chart === 'north' ? 'north' : 'south';
  const gallery = h('div', { class: 'gallery' }, DIVS.map(d => h('a', { class: 'card mini-chart', href: `#/vargas/${d}`, onclick: e => { e.preventDefault(); S.varga = d; save(); route(); } },
    h('h3', {}, `D${d} ${DIV_NAME[d] || ''}`, h('small', {}, DIV_AREA[d] || '')), h('div', { class: 'figure' }, chartSVG(kind, d, { compact: true, noTitle: true })))));
  const d = DIVS.includes(S.varga) ? S.varga : 9;
  const ids = (ASC != null ? ['lagna'] : []).concat(GRAHA_ORDER.filter(g => GR[g]));
  const grid = table(['Body', ...DIVS.map(x => 'D' + x)], ids.map(id => [id === 'lagna' ? 'Lagna' : glink(id), ...DIVS.map(x => { const sg = vargaSign(id, x), d1 = vargaSign(id, 1);
    return sg == null ? '—' : { td: { class: x > 1 && sg === d1 ? 'same' : '', title: vargaDeg(id, x) != null ? dm(vargaDeg(id, x)) : '' }, v: SIGN_AB[sg] }; })]));
  put(b, pageHead('vargas', 'Divisional charts', `${DIVS.length} vargas. Pick one to see it large; the grid shows every body in every varga (highlight: same sign as D1).`),
    h('div', { class: 'grid2' }, card(`D${d} · ${DIV_NAME[d] || ''} (${DIV_AREA[d] || ''})`, h('div', { class: 'figure' }, chartSVG(kind, d))), card('Every body, every varga', grid)),
    gallery);
}, { blurb: 'D1 to D60 side by side' });

/* ── planets and the planet page ──────────────────────────────────────────────────────────────────────────────── */
page('planets', () => 'Grahas', 'Grahas & bhāvas', (b) => {
  if (!VED) return put(b, missing('Grahas', 'They need a Vedic chart.'));
  const sb = (val('vedic.shadbala') || {}).grahas || {};
  const cards = GRAHA_ORDER.filter(g => GR[g]).map(g => {
    const gr = GR[g], st = STRENGTH[g] || {}, svg = s('svg', { viewBox: '-34 -34 68 68', class: 'pcard-icon' }); marker(svg, g, 0, 0, 26, { noLabel: true, passive: true });
    return h('a', { class: 'card pcard', href: '#/planet/' + g }, svg, h('div', {}, h('h3', {}, `${G[g].skt} · ${G[g].en}`),
      h('div', { class: 'mono' }, `${SIGN_SKT[signOf(gr.lon)]} ${inSignDM(gr.lon)} · ${NAKSHATRA[nakOf(gr.lon)]} ${padaOf(gr.lon)}`),
      h('div', { class: 'chips' }, houseOf(signOf(gr.lon)) ? h('span', { class: 'chip' }, 'bhāva ' + houseOf(signOf(gr.lon))) : null, st.dignity ? h('span', { class: 'chip' }, st.dignity) : null,
        gr.retro ? h('span', { class: 'chip' }, 'retrograde') : null, st.combust ? h('span', { class: 'chip bad' }, 'combust') : null, sb[g] ? h('span', { class: 'chip' }, `ṣaḍbala ${fix(sb[g].ratio)}×`) : null)));
  });
  put(b, pageHead('planets', 'Grahas', 'Each graha has its own page: position, nakshatra, vargas, aspects, yogas, strength, lordship, daśā and more.'), h('div', { class: 'pgrid' }, cards),
    card('All grahas', table(['Graha', 'Sidereal', 'Rāśi', 'In sign', 'Nakshatra', 'Pada', 'Navāṃśa', 'Bhāva', 'Declination', '°/day', 'Dignity', 'Avasthā', 'KP sub', 'Markers'],
      (ASC != null ? ['lagna'] : []).concat(GRAHA_ORDER.filter(g => GR[g])).map(id => { const l = lonOf(id), gr = GR[id] || {}, st = STRENGTH[id] || {}, kp = ((val('vedic.kp') || {}).planets || {})[id] || {};
        return [id === 'lagna' ? 'Lagna' : glink(id), fix(l, 4) + '°', SIGN_SKT[signOf(l)], inSign(l), NAKSHATRA[nakOf(l)], padaOf(l), SIGN_SKT[navSignOf(l)], houseOf(signOf(l)), gr.dec != null ? dm(Math.abs(gr.dec)) + (gr.dec < 0 ? ' S' : ' N') : '',
          gr.speed != null ? fix(gr.speed, 4) : '', st.dignity || '', st.avastha || '', kp.sub_lord || '', markers(l).join(', ')]; }))));
}, { blurb: 'a page for every graha' });

page('planet', (id) => G[id] ? `${G[id].en}` : 'Graha', 'Grahas & bhāvas', (b, id) => {
  id = GR[id] ? id : 'sun'; const gr = GR[id], l = gr.lon, st = STRENGTH[id] || {}, sp = SPECIAL, sb = ((val('vedic.shadbala') || {}).grahas || {})[id];
  const icon = s('svg', { viewBox: '-60 -60 120 120', class: 'hero-icon' }); marker(icon, id, 0, 0, 46, { noLabel: true, passive: true });
  const order = GRAHA_ORDER.filter(g => GR[g]), k = order.indexOf(id);
  const nav = h('div', { class: 'pnav' }, h('a', { class: 'btn', href: '#/planet/' + order[(k + order.length - 1) % order.length] }, '← ' + G[order[(k + order.length - 1) % order.length]].en), h('a', { class: 'btn', href: '#/planets' }, 'All grahas'), h('a', { class: 'btn', href: '#/planet/' + order[(k + 1) % order.length] }, G[order[(k + 1) % order.length]].en + ' →'));
  const lord = sp.lordships && sp.lordships.grahas ? sp.lordships.grahas[id] : null;
  const karaka = (() => { const ck = val('vedic.jaimini.chara_karakas'); if (!ck) return null; const r = []; for (const sch of ['eight_karaka_scheme', 'seven_karaka_scheme']) for (const [k2, v] of Object.entries(ck[sch])) if (v.graha === id) r.push(k2 + (sch.startsWith('seven') ? ' (7)' : ' (8)')); return r.join(', '); })();
  const chips = [st.dignity, st.avastha && 'avasthā ' + st.avastha, gr.retro && 'retrograde', st.combust && 'combust', vargaSign(id, 9) === signOf(l) && 'vargottama', lord && lord.functional_nature + ' for this lagna', karaka,
    sp.yogi && sp.yogi.yogi_graha === id && 'yogi', sp.avayogi && sp.avayogi.avayogi_graha === id && 'avayogi', sp.yogi && sp.yogi.duplicate_yogi === id && 'duplicate yogi', AE && AE.graha_band[id] && AE.graha_band[id] + ' (energy)'].filter(Boolean);
  const ref = NAKREF[nakOf(l)] || {};
  const asp = (val('vedic.aspects') || {}).d1 || {};
  const given = Object.entries(asp[id] || {}).map(([to, x]) => [glink(to), nth(x.house), x.fraction || '—', fix(x.virupa, 1)]);
  const received = Object.entries(asp).filter(([from, row]) => from !== id && row[id]).map(([from, row]) => [glink(from), nth(row[id].house), row[id].fraction || '—', fix(row[id].virupa, 1)]);
  const signsAspected = DRISHTI.filter(d => d.from === id).map(d => [nth(d.house_from_graha), SIGN_SKT[d.sign], d.aspects_bhava || '—', d.aspects_grahas.map(x => G[x].abbr).join(' ') || '—', d.kind]);
  const yog = YOGAS ? YOGAS.yogas.concat(YOGAS.doshas).filter(y => y.present && (y.grahas || []).includes(id)) : [];
  const vim = val('vedic.dasha.vimshottari');
  const mahas = vim ? vim.mahadashas.filter(m => m.lord.toLowerCase() === id).map(m => [dateOnly(m.start_utc), dateOnly(m.end_utc), fix(m.years, 2) + ' y']) : [];
  const vargaRows = DIVS.map(d => ['D' + d + ' ' + (DIV_NAME[d] || ''), SIGN_SKT[vargaSign(id, d)], vargaDeg(id, d) != null ? dm(vargaDeg(id, d)) : '', vargaSign(id, d) === signOf(l) && d > 1 ? '= D1' : '']);
  const vimsh = (val('vedic.vimshopaka') || {})[id];
  const av = val('vedic.ashtakavarga'), bav = av && av.bhinnashtakavarga[id];
  const kp = ((val('vedic.kp') || {}).planets || {})[id], kps = ((val('vedic.kp') || {}).significators_planet_view || {})[id];
  const tr = D.transits && D.transits.rows ? D.transits.rows[D.transits.reference_index || 0] : null, trl = tr ? tr.longitudes[id === 'rahu' ? Object.keys(tr.longitudes).find(x => /node/.test(x) && !/south/.test(x)) : id] : null;
  const links = nadiLinks(id);
  put(b, nav, h('header', { class: 'phero' }, icon, h('div', {}, h('h2', {}, `${G[id].skt} · ${G[id].en}`), h('div', { class: 'phero-pos' }, `${SIGN_SKT[signOf(l)]} ${inSign(l)} · ${NAKSHATRA[nakOf(l)]} pada ${padaOf(l)}`),
    h('div', { class: 'chips' }, chips.map(c => h('span', { class: 'chip' }, c))))),
    h('div', { class: 'grid3' },
      card('Position', facts([['Sidereal', `${fix(l, 4)}° · ${SIGN_EN[signOf(l)]} ${inSign(l)}`], ['Tropical', gr.trop != null ? `${fix(gr.trop, 4)}° · ${SIGN_EN[signOf(gr.trop)]}` : null],
        ['Latitude', gr.lat != null ? fix(gr.lat, 4) + '°' : null], ['Declination', gr.dec != null ? dms(Math.abs(gr.dec)) + (gr.dec < 0 ? ' S' : ' N') : null], ['Right ascension', gr.ra != null ? fix(gr.ra / 15, 4) + ' h' : null],
        ['Speed', gr.speed != null ? `${fix(gr.speed, 4)}°/day` : null], ['Motion', sb ? sb.chesta_state : (gr.retro ? 'retrograde' : 'direct')], ['Distance', gr.dist ? fix(gr.dist, 5) + ' AU' : null],
        ['Bhāva', houseOf(signOf(l))], ['Bhāva chalit', (val('vedic.bhava_chalit') || { grahas: {} }).grahas[id]], ['Markers', markers(l).join(', ') || null]])),
      card(`Nakshatra · ${NAKSHATRA[nakOf(l)]}`, facts([['Pada', `${padaOf(l)} (navāṃśa ${SIGN_SKT[navSignOf(l)]})`], ['Lord', G[NAK_LORD[nakOf(l) % 9]].en], ...['gana', 'yoni', 'nadi', 'dosha', 'tattva', 'guna', 'purpose', 'nature'].map(x => [x[0].toUpperCase() + x.slice(1), ref[x]])]),
        note('Nakshatra attributes: reference table (tradition_rule).')),
      lord ? card('Lordship', facts([['Owns bhāvas', lord.houses_owned.join(', ') || 'none (node)'], ['Placed in', lord.placed_in_house], ['Functional nature', lord.functional_nature]]), note(sp.lordships.rule)) : null),
    h('div', { class: 'grid2' },
      card('Aspects it casts', given.length ? table(['To', 'House', 'Graha drishti', 'Virūpa'], given) : note('none on grahas'), signsAspected.length ? table(['House', 'Sign', 'Bhāva', 'Grahas there', 'Kind'], signsAspected) : null),
      card('Aspects it receives', received.length ? table(['From', 'House', 'Graha drishti', 'Virūpa'], received) : note('none'))),
    S.layers.extra_aspects || S.layers.extra_rashi ? card('Additional aspects it casts', extraTable(ADDITIONAL.filter(d => d.from === id && extraShown(d))), note(EXTRA_NOTE)) : null,
    h('div', { class: 'grid2' },
      card('Yogas and doshas it takes part in', yog.length ? h('ul', { class: 'list' }, yog.map(y => h('li', {}, h('b', {}, y.name), ' — ', y.rule))) : note('none present')),
      card('Vargas', table(['Varga', 'Sign', 'Degree', ''], vargaRows))),
    sb ? card('Ṣaḍbala', h('div', { class: 'grid3' },
      facts([['Sthāna', fix(sb.sthana.total, 1)], ['Dig', fix(sb.dig, 1)], ['Kāla', fix(sb.kala.total, 1)], ['Ceṣṭā', fix(sb.chesta, 1)], ['Naisargika', fix(sb.naisargika, 1)], ['Dṛk', fix(sb.drik, 1)]]),
      facts([['Total', `${fix(sb.total_virupa, 1)} virūpa = ${fix(sb.rupas, 2)} rūpa`], ['Required', `${sb.required_rupas} rūpa`], ['Ratio', fix(sb.ratio, 2)]]),
      h('div', {}, bar(sb.ratio, 2), note('ratio of strength to the required minimum')))) : null,
    h('div', { class: 'grid3' },
      vimsh ? card('Viṃśopaka', facts(Object.entries(vimsh).map(([k2, v]) => [k2, `${fix(v.score, 1)} / 20`]))) : null,
      bav ? card('Aṣṭakavarga', h('div', { class: 'bav' }, bav.bindus.map((v, i) => h('span', { class: 'bv' + (i === signOf(l) ? ' here' : ''), title: SIGN_SKT[i] }, h('small', {}, SIGN_AB[i]), v))), note(`${bav.total} bindus; ${bav.bindus[signOf(l)]} in its own sign`)) : null,
      kp ? card('KP', facts([['Sign lord', kp.sign_lord], ['Star lord', kp.star_lord], ['Sub lord', kp.sub_lord], ['Sub-sub lord', kp.sub_sub_lord], kps ? ['Signifies', `A ${kps.A_house_of_star_lord || '—'} · B ${kps.B_house_occupied} · C ${kps.C_houses_owned_by_star_lord.join(',') || '—'} · D ${kps.D_houses_owned.join(',') || '—'}`] : null])) : null),
    h('div', { class: 'grid3' },
      mahas.length ? card('Vimśottarī mahādaśā', table(['From', 'To', 'Length'], mahas)) : null,
      trl != null ? card('Transit now', facts([['At', dateOnly(tr.utc)], ['Position', `${SIGN_SKT[signOf(trl)]} ${inSignDM(trl)}`], ['Over natal bhāva', houseOf(signOf(trl))], ['From natal Moon', GR.moon ? (signOf(trl) - signOf(GR.moon.lon) + 12) % 12 + 1 : null]])) : null,
      card('Nadi links', links.length ? h('ul', { class: 'list' }, links.map(x => h('li', {}, glink(x.to), ' — ', x.kind, x.via ? ` (${x.via})` : ''))) : note('none'), note('report.nadi@0.1: trines, 2nd/12th and 7th from its sign; retrograde also from the previous sign.'))));
}, { hidden: true, section: 'planet' });

/* ── houses and the house page ────────────────────────────────────────────────────────────────────────────────── */
page('houses', () => 'Bhāvas', 'Grahas & bhāvas', (b) => {
  if (LAGNA_SIGN == null) return put(b, missing('Bhāvas', 'They need an exact birth time and place.'));
  const bb = val('vedic.bhava_bala'), sav = val('vedic.ashtakavarga');
  const cards = Array.from({ length: 12 }, (_, i) => { const sign = (LAGNA_SIGN + i) % 12, occ = GRAHA_ORDER.filter(g => GR[g] && signOf(GR[g].lon) === sign), ae = AE ? AE.houses[i + 1] : null;
    return h('a', { class: 'card hcard', href: '#/house/' + (i + 1) }, h('div', { class: 'hnum' }, i + 1), h('div', {}, h('h3', {}, `${SIGN_SKT[sign]} ${GLYPH[sign]}`),
      h('div', { class: 'note' }, ae ? ae.dimension : ''), h('div', { class: 'chips' }, occ.map(g => h('span', { class: 'chip' }, dot(g), G[g].abbr)),
        bb ? h('span', { class: 'chip' }, `bala ${fix(bb.bhavas[i].rupas, 1)}`) : null, sav ? h('span', { class: 'chip' }, `SAV ${sav.sarvashtakavarga.bindus[sign]}`) : null))); });
  put(b, pageHead('houses', 'Bhāvas', 'Twelve houses from the lagna, where each lord sits, and how the houses connect. Each has its own page.'), h('div', { class: 'hgrid' }, cards), lordsAnalysis(), lordshipsByVarga());
}, { blurb: 'a page for every bhāva' });

/* ── lordships in D1, D9 and D10: "lord of 1 in 3", with the exchanges between house lords (engine: special points) ── */
const PARIV_KIND = { maha: 'Mahā (between good houses)', khala: 'Khala (involves the 3rd)', dainya: 'Dainya (involves the 6th, 8th or 12th)' };
function lordshipsByVarga() {
  const all = (SPECIAL || {}).lordships_by_varga; if (!all) return null;
  const host = h('div', {}), cur = { d: S.lordVarga || 'd1' };
  const draw = () => {
    const v = all[cur.d]; if (!v) return put(host, note('Not available for this division.'));
    const exH = new Set(v.exchanges.flatMap(e => e.houses));
    const rows = v.houses.map(x => [h('b', {}, x.house), `${SIGN_SKT[x.sign]} ${GLYPH[x.sign]}`, glink(x.lord),
      x.lord_in_house == null ? '—' : h('span', { class: exH.has(x.house) && exH.has(x.lord_in_house) ? 'lv-ex' : '' }, `in ${x.lord_in_house}`, x.lord_sign != null ? ` (${SIGN_SKT[x.lord_sign]})` : ''),
      `lord of ${x.house} in ${x.lord_in_house ?? '—'}`]);
    put(host, tabs([['d1', 'D1 rāśi'], ['d9', 'D9 navāṃśa'], ['d10', 'D10 daśāṃśa']], cur.d, k => { cur.d = k; S.lordVarga = k; save(); draw(); }),
      h('p', { class: 'note' }, `Houses counted from the ${cur.d.toUpperCase()} lagna, ${SIGN_SKT[v.lagna_sign]}.`),
      h('div', { class: 'grid2' },
        table(['Bhāva', 'Sign', 'Lord', 'Lord sits', 'Read as'], rows),
        h('div', {}, h('h4', {}, 'Exchanges (parivartana)'),
          v.exchanges.length ? table(['Grahas', 'Signs', 'Houses', 'Kind'], v.exchanges.map(e => [h('span', {}, glink(e.grahas[0]), ' ⇄ ', glink(e.grahas[1])),
            `${SIGN_SKT[e.signs[0]]} ⇄ ${SIGN_SKT[e.signs[1]]}`, `${e.houses[0]} ⇄ ${e.houses[1]}`, PARIV_KIND[e.kind] || e.kind])) : note('No exchange between house lords in this division.'),
          note('In an exchange each graha sits in the other\'s sign, so the two houses are read as linked; the kind follows Phaladeepika.'))));
  };
  draw();
  return card('Lordships by division (D1, D9, D10)', host, h('p', { class: 'note' }, label('tradition_rule'), ' Each bhāva\'s lord and the bhāva it occupies, in the rāśi, navāṃśa and daśāṃśa charts.'));
}

/* ── bhāva depth: bhāvāt bhāvam, the lord's story, occupants, aspects, kāraka, connections, bearing ───────────── */
const KARAKA = { 1: ['sun'], 2: ['jupiter'], 3: ['mars'], 4: ['moon', 'mercury'], 5: ['jupiter'], 6: ['mars', 'saturn'], 7: ['venus'], 8: ['saturn'], 9: ['jupiter', 'sun'], 10: ['sun', 'mercury', 'jupiter', 'saturn'], 11: ['jupiter'], 12: ['saturn'] };
const houseFrom = (from, to) => (to - from + 12) % 12 + 1;
function bhavaDepth(n) {
  const sign = (LAGNA_SIGN + n - 1) % 12, lord = SIGN_LORD[sign], gl = GR[lord], sp = SPECIAL, lords = (sp.lordships || {}).grahas || {};
  const sb = (val('vedic.shadbala') || {}).grahas || {}, asp = val('vedic.aspects') || {}, onH = ((asp.houses || {}).d1 || {}), avim = (asp.d1 || {});
  const occ = GRAHA_ORDER.filter(g => GR[g] && G[g] && signOf(GR[g].lon) === sign);
  const bb = (2 * n - 2) % 12 + 1, bbSign = (LAGNA_SIGN + bb - 1) % 12, bbOcc = GRAHA_ORDER.filter(g => GR[g] && G[g] && signOf(GR[g].lon) === bbSign);
  const lordHouse = gl ? houseOf(signOf(gl.lon)) : null, lordFromHere = gl ? houseFrom(sign, signOf(gl.lon)) : null, st = STRENGTH[lord] || {};
  const lordYogas = typeof YOGA_PRESENT === 'function' ? YOGA_PRESENT().filter(y => (y.grahas || []).includes(lord)) : [];
  const recv = gl ? Object.entries(avim).filter(([f, row]) => f !== lord && row[lord] && row[lord].virupa >= 30).map(([f, row]) => [f, row[lord].virupa]) : [];
  const onMadhya = NINE.map(g => [g, onH[g] && onH[g][String(n)] ? onH[g][String(n)].virupa : 0]).filter(([, v]) => v >= 15);
  const ben = x => NAT_BENEFIC.includes(x), sum = arr => arr.reduce((a, [, v]) => a + v, 0);
  const karakas = KARAKA[n] || [];
  // factors usually weighed for a bhāva, evaluated here (each is a fact; the tally is a reading aid)
  const F = [];
  const add = (ok, text) => F.push({ ok, text });
  if (sb[lord]) add(sb[lord].ratio >= 1, `the lord ${G[lord].en} has ${sb[lord].ratio >= 1 ? 'enough' : 'less than its required'} ṣaḍbala (ratio ${fix(sb[lord].ratio, 2)})`);
  if (st.dignity) add(!['debilitated'].includes(st.dignity) && st.dignity !== 'enemy' && st.dignity !== 'great_enemy', `the lord is ${st.dignity.replace(/_/g, ' ')}${st.combust ? ' and combust' : ''}`);
  if (lordHouse) add(![6, 8, 12].includes(lordHouse), `the lord sits in bhāva ${lordHouse} from the lagna${[1, 4, 7, 10].includes(lordHouse) ? ' (a kendra)' : [5, 9].includes(lordHouse) ? ' (a trikoṇa)' : [6, 8, 12].includes(lordHouse) ? ' (a dusthāna)' : ''}`);
  if (lordFromHere) add(![6, 8, 12].includes(lordFromHere), `the lord is ${nth(lordFromHere)} from this bhāva`);
  occ.filter(g => NINE.includes(g)).forEach(g => add(ben(g) || /benefic/.test((lords[g] || {}).functional_nature || ''), `${G[g].en} occupies it (${ben(g) ? 'natural benefic' : 'natural malefic'}${lords[g] ? ', functional ' + lords[g].functional_nature : ''})`));
  const bsum = sum(onMadhya.filter(([g]) => ben(g))), msum = sum(onMadhya.filter(([g]) => !ben(g)));
  if (bsum || msum) add(bsum >= msum, `aspects on the madhya: ${Math.round(bsum)} virūpa from benefics, ${Math.round(msum)} from malefics`);
  const sav = val('vedic.ashtakavarga'); if (sav) add(sav.sarvashtakavarga.bindus[sign] >= 28, `${sav.sarvashtakavarga.bindus[sign]} SAV bindus (28 is the usual average)`);
  karakas.forEach(k => { if (GR[k] && houseOf(signOf(GR[k].lon)) === n) add(false, `the kāraka ${G[k].en} sits in its own bhāva (kāraka bhāva nāśa, often read as a strain)`); });
  const good = F.filter(f => f.ok).length, bad = F.length - good;
  return { sign, lord, gl, occ, bb, bbSign, bbOcc, lordHouse, lordFromHere, st, lordYogas, recv, onMadhya, karakas, F, good, bad };
}
/* a small connection map: the bhāva in the centre, linked to its lord, occupants, aspecting grahas, kāraka and bhāvāt bhāvam */
function bhavaMap(n, x) {
  const W = 520, H = 360, cx = W / 2, cy = H / 2;
  const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'bmap', role: 'img', 'aria-label': `How bhāva ${n} connects` });
  const nodes = [], seen = new Set();
  const addNode = (id, label, kind, colour, href) => { if (seen.has(id)) { nodes.find(o => o.id === id).kinds.push(kind); return; } seen.add(id); nodes.push({ id, label, kinds: [kind], colour, href }); };
  addNode(x.lord, G[x.lord].en, 'lord', G[x.lord].dot, '#/planet/' + x.lord);
  x.occ.forEach(g => addNode(g, G[g].en, 'occupies', G[g].dot, '#/planet/' + g));
  x.onMadhya.forEach(([g]) => addNode(g, G[g].en, 'aspects', G[g].dot, '#/planet/' + g));
  x.karakas.forEach(g => addNode(g, G[g].en, 'kāraka', G[g].dot, '#/planet/' + g));
  addNode('bb', `Bhāva ${x.bb}`, 'bhāvāt bhāvam', 'var(--accent2)', '#/house/' + x.bb);
  nodes.forEach((o, i) => {
    const a = -Math.PI / 2 + i * 2 * Math.PI / nodes.length, ox = cx + Math.cos(a) * 140, oy = cy + Math.sin(a) * 120;
    const ln = s('line', { x1: cx, y1: cy, x2: ox, y2: oy, class: 'bm-edge ' + o.kinds[0].replace(/\W/g, ''), stroke: o.colour }, svg);
    ln.style.setProperty('--d', (i * 120) + 'ms');
    const [mx, my] = [(cx + ox) / 2, (cy + oy) / 2]; const lab = s('text', { x: mx, y: my - 4, class: 'bm-elab' }, svg); lab.textContent = o.kinds.join(' · ');
    const a2 = s('a', { href: o.href }, svg);
    s('circle', { cx: ox, cy: oy, r: 26, fill: o.colour, class: 'bm-node' }, a2);
    const t = s('text', { x: ox, y: oy + 4, class: 'bm-nlab' }, a2); t.textContent = o.label.length > 9 ? o.label.slice(0, 8) + '…' : o.label;
  });
  s('circle', { cx, cy, r: 46, class: 'bm-halo' }, svg); s('circle', { cx, cy, r: 38, class: 'bm-core' }, svg);
  const t = s('text', { x: cx, y: cy - 2, class: 'bm-clab' }, svg); t.textContent = `Bhāva ${n}`;
  const t2 = s('text', { x: cx, y: cy + 14, class: 'bm-csub' }, svg); t2.textContent = SIGN_SKT[x.sign];
  return svg;
}
/* ── bhāvāt bhāvam across the chart, and each bhāva read as its own lagna ───────────────────────────────────────
   house topics and the meaning of each house counted from another follow standard Parāśari teaching; the reading
   sentences are assembled from these and labelled interpretation */
const BHAVA_TOPIC = { 1: 'self, body and direction', 2: 'wealth, family and speech', 3: 'courage, effort and siblings', 4: 'home, mother and inner happiness', 5: 'children, intellect and creativity',
  6: 'debts, illness, rivals and service', 7: 'spouse and partnerships', 8: 'longevity, the hidden and sudden change', 9: 'fortune, dharma, father and teachers', 10: 'career, status and action',
  11: 'gains, friends and wishes fulfilled', 12: 'losses, expenses, retreat and liberation' };
const REL_MEANING = { 1: 'its very nature', 2: 'what sustains and adds to it (its gains)', 3: 'the effort and initiative behind it', 4: 'its foundation and comfort', 5: 'its fruition and creativity',
  6: 'what obstructs or competes with it', 7: 'its counterpart and partners', 8: 'its hidden side, endings and transformation', 9: 'its fortune and blessings', 10: 'its action and visible status',
  11: 'its fulfilment and gains', 12: 'its loss, expense and release' };
const VRY = { 6: ['Harṣa', 'the 6th lord in a dusthāna: victory over rivals, health and happiness won through adversity'], 8: ['Sarala', 'the 8th lord in a dusthāna: resilience, fearlessness and lasting resources'],
  12: ['Vimala', 'the 12th lord in a dusthāna: thrift, independence and good conduct'] };
function lordOfHouse(n) { return SIGN_LORD[(LAGNA_SIGN + n - 1) % 12]; }
function placedIn(g) { return GR[g] ? houseOf(signOf(GR[g].lon)) : null; }
function lordRow(n) {
  const L = lordOfHouse(n), k = placedIn(L), st = STRENGTH[L] || {}, r = k ? houseFrom(n, k) : null, bb = (2 * n - 2) % 12 + 1, bbL = lordOfHouse(bb), bbK = placedIn(bbL);
  const flags = [], read = [];
  if (k === n) flags.push(['own bhāva', 'ok']);
  if (k && [1, 4, 7, 10].includes(k)) flags.push(['kendra', 'ok']); else if (k && [5, 9].includes(k)) flags.push(['trikoṇa', 'ok']); else if (k && [6, 8, 12].includes(k)) flags.push(['dusthāna', 'warn']);
  if (k && k !== n && lordOfHouse(k) !== L && placedIn(lordOfHouse(k)) === n) flags.push([`exchange with ${k}`, 'ok']);
  if ([6, 8, 12].includes(n) && [6, 8, 12].includes(k)) flags.push([`Vipareeta Rāja yoga (${VRY[n][0]})`, 'ok']);
  if (st.dignity === 'debilitated') flags.push(['lord debilitated', 'warn']); else if (st.dignity === 'exalted') flags.push(['lord exalted', 'ok']);
  if (st.combust) flags.push(['lord combust', 'warn']);
  read.push(`The matters of bhāva ${n} (${BHAVA_TOPIC[n]}) are carried into bhāva ${k} (${BHAVA_TOPIC[k] || '—'}).`);
  if (r) read.push(`Counted from its own house the lord is ${nth(r)}: ${REL_MEANING[r]} of bhāva ${n}.`);
  if ([6, 8, 12].includes(n) && [6, 8, 12].includes(k)) read.push(`This forms ${VRY[n][0]} Vipareeta Rāja yoga: ${VRY[n][1]}.`);
  else if (k && [6, 8, 12].includes(k) && ![6, 8, 12].includes(n)) read.push(`A lord in a dusthāna usually brings effort or strain to the bhāva it rules.`);
  return { n, L, k, r, bb, bbL, bbK, flags, read };
}
function lordsAnalysis() {
  if (LAGNA_SIGN == null) return null;
  const rows = Array.from({ length: 12 }, (_, i) => lordRow(i + 1));
  return card('Lords in houses and bhāvāt bhāvam', table(['Bhāva', 'Lord', 'Placed in', 'From its own house', 'Bhāvāt bhāvam', 'Notes and reading'], rows.map(x => [
    h('a', { href: '#/house/' + x.n }, h('b', {}, String(x.n)), ` ${BHAVA_TOPIC[x.n].split(',')[0]}`), glink(x.L),
    x.k ? `bhāva ${x.k} · ${SIGN_AB[signOf(GR[x.L].lon)]} ${inSignDM(GR[x.L].lon)}${(STRENGTH[x.L] || {}).dignity ? ' · ' + STRENGTH[x.L].dignity : ''}` : '—',
    x.r ? `${nth(x.r)} — ${REL_MEANING[x.r]}` : '—',
    h('span', {}, h('a', { href: '#/house/' + x.bb }, `bhāva ${x.bb}`), x.bbK ? `: lord ${G[x.bbL].abbr} in ${x.bbK}` : ''),
    h('div', {}, h('span', { class: 'badges' }, x.flags.map(([t, c]) => h('span', { class: 'badge ' + (c === 'ok' ? 'b-up' : 'b-down') }, t))), h('span', { class: 'rd' }, x.read.join(' ')))])),
    h('p', { class: 'note' }, label('interpretation'), ' House topics and relative meanings follow standard Parāśari teaching; bhāvāt bhāvam is the house as far from a bhāva as it is from the lagna (2nd from 2nd = 3rd, and so on). The flags are computed; the sentences are a reading aid.'));
}
/* the chart read from one bhāva: that bhāva becomes the lagna, and each house from it says something about it */
function asLagna(n) {
  const base = (LAGNA_SIGN + n - 1) % 12;
  const rows = Array.from({ length: 12 }, (_, i) => {
    const rel = i + 1, sign = (base + i) % 12, abs = houseOf(sign), occ = GRAHA_ORDER.filter(g => GR[g] && signOf(GR[g].lon) === sign), L = SIGN_LORD[sign], st = STRENGTH[L] || {};
    const ben = occ.filter(g => NAT_BENEFIC.includes(g)).length, mal = occ.filter(g => NAT_MALEFIC.includes(g)).length;
    return [h('b', {}, nth(rel)), h('a', { href: '#/house/' + abs }, `bhāva ${abs} · ${SIGN_AB[sign]}`), REL_MEANING[rel], occ.length ? h('span', {}, occ.map(g => h('span', { class: 'chip' }, dot(g), G[g].abbr))) : '—',
      h('span', {}, glink(L), ` in ${placedIn(L) ? nth(houseFrom(abs, placedIn(L))) + ' from it' : '—'}${st.dignity ? ', ' + st.dignity : ''}`),
      ben || mal ? h('span', { class: ben >= mal ? 'ok' : 'warn' }, ben >= mal ? 'supports' : 'strains') : '—'];
  });
  const chart = chartSVG(S.chart === 'north' ? 'north' : 'south', 1, { lagSign: base });
  return [h('h3', { class: 'section-t' }, `Bhāva ${n} as the lagna`),
    h('div', {}, card(`The chart counted from bhāva ${n}`, h('div', { class: 'figure narrow' }, chart), note(`House numbers here count from bhāva ${n} (${SIGN_SKT[base]}): its 2nd shows what sustains it, its 12th what drains it, its 6th what opposes it, and so on.`)),
      card('What each house from it says', table(['From it', 'Bhāva', 'Says about bhāva ' + n, 'Occupants', 'Lord', ''], rows),
        h('p', { class: 'note' }, label('interpretation'), ' Occupants supporting or straining are counted from natural benefics and malefics; read with the lord\'s strength.')))];
}

function bhavaDepthCards(n) {
  const x = bhavaDepth(n), ae = AE ? AE.houses[n] : null, bbAe = AE ? AE.houses[x.bb] : null;
  const vim = typeof activation === 'function' ? activation([x.lord, ...x.occ.filter(g => NINE.includes(g))]) : null;
  return [
    h('h3', { class: 'section-t' }, 'How this bhāva works in the chart'),
    h('div', { class: 'grid2' },
      card('Connections', h('div', { class: 'figure' }, bhavaMap(n, x)), note('The bhāva in the centre; around it its lord, occupants, grahas aspecting its madhya (15 virūpas or more), kāraka and bhāvāt bhāvam. Click any to open it.')),
      card('The lord’s story', facts([['Lord', glink(x.lord)], ['Placed in', x.gl ? `bhāva ${x.lordHouse} from the lagna · ${nth(x.lordFromHere)} from this bhāva · ${SIGN_SKT[signOf(x.gl.lon)]} ${inSignDM(x.gl.lon)}` : null],
        ['Dignity', [x.st.dignity, x.st.combust && 'combust', x.gl && x.gl.retro && 'retrograde'].filter(Boolean).join(', ') || null],
        ['Navāṃśa', x.gl ? `${SIGN_SKT[vargaSign(x.lord, 9)]}${vargaSign(x.lord, 9) === signOf(x.gl.lon) ? ' (vargottama)' : ''}` : null],
        ['Also rules', (((SPECIAL.lordships || {}).grahas || {})[x.lord] || { houses_owned: [] }).houses_owned.filter(k => k !== n).map(k => 'bhāva ' + k).join(', ') || 'only this bhāva'],
        ['Aspected by', x.recv.length ? x.recv.map(([f, v]) => `${G[f].abbr} ${Math.round(v)}`).join(', ') + ' virūpa' : 'none of 30 virūpas or more'],
        ['Yogas it forms', x.lordYogas.length ? h('span', {}, x.lordYogas.map(y => h('a', { class: 'chip', href: '#/yoga/' + y.id }, y.name))) : 'none present']]))),
    h('div', { class: 'grid3' },
      card(`Bhāvāt bhāvam · bhāva ${x.bb}`, facts([['Why', `bhāva ${n} counted ${n} from itself lands on bhāva ${x.bb}: its matters reflect and reinforce this one`], ['Sign', SIGN_SKT[x.bbSign]], ['Lord', glink(SIGN_LORD[x.bbSign])],
        ['Occupants', x.bbOcc.length ? x.bbOcc.map(g => G[g].abbr).join(', ') : 'empty'], ['Dimension', bbAe ? bbAe.dimension : null]]), h('a', { class: 'btn', href: '#/house/' + x.bb }, `Open bhāva ${x.bb} →`)),
      card('Occupants and what they bring', x.occ.length ? table(['Graha', 'Dignity', 'Rules', 'Nature'], x.occ.map(g => { const lo = ((SPECIAL.lordships || {}).grahas || {})[g]; return [glink(g), (STRENGTH[g] || {}).dignity || '—', lo && lo.houses_owned.length ? h('span', {}, lo.houses_owned.flatMap((k, i) => (i ? [', '] : []).concat([h('a', { href: '#/house/' + k }, String(k))]))) : '—', lo ? lo.functional_nature : (NAT_BENEFIC.includes(g) ? 'natural benefic' : 'natural malefic')]; })) : note('No graha occupies it; its lord and aspects carry it.')),
      card('Kāraka', facts(x.karakas.map(k => [G[k].en, GR[k] ? `bhāva ${houseOf(signOf(GR[k].lon))}, ${(STRENGTH[k] || {}).dignity || ''}${houseOf(signOf(GR[k].lon)) === n ? ' · in its own bhāva (kāraka bhāva nāśa)' : ''}` : '—'])), note('Natural significators of the bhāva (traditional table).'))),
    card('Bearing on this bhāva', h('div', { class: 'bearing' },
      h('div', { class: 'meter' }, h('i', { class: 'good', style: `flex:${x.good || .01}` }), h('i', { class: 'bad', style: `flex:${x.bad || .01}` })),
      h('p', {}, h('b', {}, `${x.good} supportive`), ' and ', h('b', {}, `${x.bad} challenging`), ' factors.'),
      h('ul', { class: 'list factors' }, x.F.map(f => h('li', { class: f.ok ? 'ok' : 'warn' }, (f.ok ? '✓ ' : '✗ ') + f.text))),
      ae ? h('p', {}, `These bear on the bhāva’s significations — ${ae.significations.split(',').slice(0, 8).join(',').trim()} — `,
        x.good > x.bad ? 'which the tradition reads as supported, ' : x.good < x.bad ? 'which the tradition reads as needing effort, ' : 'with a mixed reading, ',
        `most noticeably in the periods of ${[x.lord, ...x.occ.filter(g => NINE.includes(g))].map(g => G[g].en).filter((v, i, a) => a.indexOf(v) === i).join(', ')}.`) : null,
      vim && vim.md.length ? table(['Mahādaśā', 'From', 'To', ''], vim.md) : null,
      h('p', { class: 'note' }, label('interpretation'), ' The factors are computed; weighing them is a reading aid in the Parāśari manner, not a prediction.')))];
}

page('house', (n) => `Bhāva ${n || 1}`, 'Grahas & bhāvas', (b, n) => {
  n = Math.max(1, Math.min(12, +n || 1));
  if (LAGNA_SIGN == null) return put(b, missing('Bhāvas', 'They need an exact birth time and place.'));
  const sign = (LAGNA_SIGN + n - 1) % 12, lord = SIGN_LORD[sign], occ = GRAHA_ORDER.filter(g => GR[g] && signOf(GR[g].lon) === sign);
  const bb = val('vedic.bhava_bala'), row = bb ? bb.bhavas[n - 1] : null, sav = val('vedic.ashtakavarga'), kp = ((val('vedic.kp') || {}).cusps || {})[n], ae = AE ? AE.houses[n] : null;
  const ar = val('vedic.jaimini.arudha_padas'), pada = ar ? Object.values(ar).find(x => x.house === n) : null;
  const nav = h('div', { class: 'pnav' }, h('a', { class: 'btn', href: '#/house/' + ((n + 10) % 12 + 1) }, '← ' + ((n + 10) % 12 + 1)), h('a', { class: 'btn', href: '#/houses' }, 'All bhāvas'), h('a', { class: 'btn', href: '#/house/' + (n % 12 + 1) }, (n % 12 + 1) + ' →'));
  put(b, nav, pageHead('houses', `Bhāva ${n} · ${SIGN_SKT[sign]}`, ae ? `${ae.dimension} — ${ae.question}` : ''),
    h('div', { class: 'grid3' },
      card('Sign and lord', facts([['Sign', `${SIGN_SKT[sign]} · ${SIGN_EN[sign]} (${ELEMENT[sign % 4]}, ${MODALITY[sign % 3]})`], ['Lord', glink(lord)], ['Lord placed in', GR[lord] ? `bhāva ${houseOf(signOf(GR[lord].lon))} · ${SIGN_SKT[signOf(GR[lord].lon)]}` : null],
        ['Ārūḍha pada', pada ? `${SIGN_SKT[pada.pada_sign]} (bhāva ${pada.pada_house})` : null], ['SAV bindus', sav ? sav.sarvashtakavarga.bindus[sign] : null]])),
      card('Occupants', occ.length ? h('ul', { class: 'list' }, occ.map(g => h('li', {}, glink(g), ` ${inSignDM(GR[g].lon)} · ${NAKSHATRA[nakOf(GR[g].lon)]} ${padaOf(GR[g].lon)}`))) : note('empty')),
      card('Aspected by', (ASPECTED[sign] || []).length ? h('ul', { class: 'list' }, ASPECTED[sign].map(d => h('li', {}, glink(d.from), ` (${nth(d.house_from_graha)}${d.kind === 'special' ? ', special' : ''})`))) : note('no graha drishti'))),
    h('div', { class: 'grid3' },
      row ? card('Bhāva bala', facts([['Lord’s ṣaḍbala', fix(row.adhipati, 1)], ['Dig', `${fix(row.dig, 1)} (${row.sign_nature})`], ['Drishti', fix(row.drishti, 1)], ['Total', `${fix(row.total_virupa, 1)} virūpa = ${fix(row.rupas, 2)} rūpa`], ['Madhya', `${SIGN_SKT[signOf(row.madhya)]} ${inSignDM(row.madhya)}`]])) : null,
      kp ? card('KP cusp', facts([['Cusp', `${SIGN_SKT[signOf(kp.cusp)]} ${inSign(kp.cusp)}`], ['Star · sub · sub-sub', `${kp.lords.star_lord} · ${kp.lords.sub_lord} · ${kp.lords.sub_sub_lord}`],
        ['A in star of occupants', kp.A_in_star_of_occupants.join(', ') || '—'], ['B occupants', kp.B_occupants.join(', ') || '—'], ['C in star of lord', kp.C_in_star_of_lord.join(', ') || '—'], ['D lord', kp.D_lord]])) : null,
      ae ? card('Reference (AstroEnergetics)', facts([['Names', ae.names], ['Kārakas', ae.karakas], ['Body', ae.body], ['Quadrant', ae.quadrant], ['Trikoṇa', ae.trikona]]), note('From the author’s AstroEnergetics framework: reference, not computed.')) : null),
    ae ? card('Significations (reference)', h('p', {}, ae.significations)) : null, ...bhavaDepthCards(n), ...asLagna(n));
}, { hidden: true, section: 'house' });

/* ── aspects ──────────────────────────────────────────────────────────────────────────────────────────────────── */
/* ── aspects: on houses, house + lord, on and from grahas, to Śrīpati cusps, for D1 and D9 ─────────────────────── */
let ASP_VIEW = 'matrix', ASP_VARGA = 'd1';
/* a small bar chart: one bar per label, values in virūpas (0–60) shown as percent of a full aspect */
function barChart(title, labels, values, opts = {}) {
  const W = 300, H = 150, left = 30, bottom = 22, top = 10, w = (W - left - 6) / labels.length;
  const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'mini-bars', role: 'img', 'aria-label': title });
  for (const p of [0, 25, 50, 75, 100]) {
    const y = H - bottom - (H - bottom - top) * p / 100;
    s('line', { x1: left, x2: W - 4, y1: y, y2: y, class: 'mb-grid' }, svg);
    const t = s('text', { x: left - 4, y: y + 3, class: 'mb-tick' }, svg); t.textContent = p + '%';
  }
  labels.forEach((lb, i) => {
    const v = values[i], pct = v == null ? 0 : Math.max(0, Math.min(100, v / 60 * 100)), bh = (H - bottom - top) * pct / 100, x = left + i * w + w * .18;
    const col = opts.colour ? opts.colour(i) : '#3fb950';
    if (v != null && pct > 0) {
      const r = s('rect', { x, y: H - bottom - bh, width: w * .64, height: bh, rx: 2, fill: col, class: 'mb-bar' }, svg);
      r.style.setProperty('--d', (i * 35) + 'ms');
      const tt = s('title', {}, r); tt.textContent = `${opts.tipFrom ? opts.tipFrom(i) : lb}: ${Math.round(pct)}% (${fix(v, 1)} virūpa)`;
    }
    const t = s('text', { x: x + w * .32, y: H - 7, class: 'mb-lab' }, svg); t.textContent = lb;
  });
  return h('div', { class: 'mini' }, h('div', { class: 'mini-t' }, title), svg);
}
/* additional aspects as a table: who casts, from which sign and why, onto which sign, bhāva and grahas */
function extraTable(list) {
  if (!list.length) return note('No additional aspects: no graha (other than the nodes) is retrograde and no two grahas are in parivartana.');
  return table(['Graha', 'Why', 'Cast from', 'Drishti', 'House from there', 'Sign', 'Bhāva', 'Grahas there'], list.map(d => [glink(d.from),
    h('span', { class: 'extramark' }, EXTRA_MARK[d.basis] + ' ' + EXTRA_NAME[d.basis] + (d.partner ? ` with ${G[d.partner].en}` : '')), SIGN_SKT[d.from_sign], d.drishti === 'rashi' ? 'rāśi' : `graha${d.kind === 'special' ? ' (special)' : ''}`,
    nth(d.house_from_graha), SIGN_SKT[d.sign], d.aspects_bhava || '—', (d.aspects_grahas || []).filter(g => G[g]).map(g => G[g].abbr).join(' ') || '—']));
}
const EXTRA_NOTE = 'Additional aspects (vedic.drishti.additional): a retrograde graha, other than Rāhu and Ketu, also casts its graha and rāśi drishti from the sign before it (its shadow, ʳ); two grahas in parivartana, each in the other’s sign, also cast their aspects as if exchanged (⇄). They are conventions some practitioners use; ESHANA shows them only when this layer is on.';
page('aspects', () => 'Aspects', 'Grahas & bhāvas', (b) => {
  const a = val('vedic.aspects'); if (!a) return put(b, missing('Aspects', 'Ask for the aspects feature.'));
  const vg = a.houses && a.houses[ASP_VARGA] ? ASP_VARGA : 'd1', m = a[vg] || {}, hs = (a.houses || {})[vg], hl = (a.house_and_lord || {})[vg];
  const from9 = NINE.filter(g => m[g] || GR[g]), colour = i => G[from9[i]].dot, abbr = from9.map(g => G[g].abbr);
  const onHouse = k => from9.map(g => hs && hs[g] && hs[g][String(k)] ? hs[g][String(k)].virupa : null);
  const onLagna = () => from9.map(g => hs && hs[g] && hs[g].lagna ? hs[g].lagna.virupa : null);
  const onGraha = t => from9.map(g => g === t ? null : (m[g] && m[g][t] ? m[g][t].virupa : null));
  const fromGraha = f => from9.map(t => t === f ? null : (m[f] && m[f][t] ? m[f][t].virupa : null));
  const toHouses = f => Array.from({ length: 12 }, (_, k) => hs && hs[f] && hs[f][String(k + 1)] ? hs[f][String(k + 1)].virupa : null);
  const lagnaSign = vg === 'd9' ? vargaSign('lagna', 9) : LAGNA_SIGN;
  const hname = k => lagnaSign == null ? `House ${k}` : `House ${k} · ${SIGN_SKT[(lagnaSign + k - 1) % 12]}`;
  const views = [['matrix', 'Graha × graha matrix'], ['houses', 'On houses'], ['houselord', 'On house + lord'], ['planets', 'On and from grahas'], ['from', 'From each graha to houses'], ['cusps', 'To Śrīpati cusps (table)']];
  let body;
  const needHouses = note('Aspects on houses need an exact birth time and place.');
  if (ASP_VIEW === 'houses') body = hs ? h('div', { class: 'minis' }, [barChart('Aspects on the lagna', abbr, onLagna(), { colour })].concat(Array.from({ length: 12 }, (_, k) => barChart(hname(k + 1), abbr, onHouse(k + 1), { colour })))) : needHouses;
  else if (ASP_VIEW === 'houselord') body = hl ? h('div', { class: 'minis' }, Array.from({ length: 12 }, (_, k) => barChart(hname(k + 1) + ' + its lord', abbr, from9.map(g => hl[g] ? hl[g][String(k + 1)] : null), { colour }))) : needHouses;
  else if (ASP_VIEW === 'planets') body = h('div', { class: 'minis' }, (hs ? [barChart('Aspects on the lagna', abbr, onLagna(), { colour }), barChart('Aspects from the lagna', abbr, from9.map(() => null), {})] : [])
    .concat(...from9.filter(g => GR[g]).map(g => [barChart(`Aspects on ${G[g].en}`, abbr, onGraha(g), { colour }), barChart(`Aspects from ${G[g].en}`, abbr, fromGraha(g), { colour: () => G[g].dot })])));
  else if (ASP_VIEW === 'from') body = hs ? h('div', { class: 'minis' }, from9.filter(g => GR[g]).map(g => barChart(`From ${G[g].en} to the houses`, Array.from({ length: 12 }, (_, k) => String(k + 1)), toHouses(g), { colour: () => G[g].dot, tipFrom: i => hname(i + 1) }))) : needHouses;
  else if (ASP_VIEW === 'cusps') body = hs ? table(['House', 'Madhya', ...abbr], Array.from({ length: 12 }, (_, k) => {
    const md = ((a.house_madhya || {})[vg] || [])[k];
    return [String(k + 1), md != null ? `${SIGN_AB[signOf(md)]} ${inSignDM(md)}` : '—', ...from9.map(g => { const v = hs[g] && hs[g][String(k + 1)] ? hs[g][String(k + 1)].virupa : null; return v ? { td: { class: 'asp-cell', style: `--v:${Math.min(1, v / 60)}` }, v: String(Math.round(v)) } : '–'; })];
  })) : needHouses;
  else {
    const mat = (mm) => { const ids = NINE.filter(g => mm[g] || GR[g]); return table(['', ...ids.map(g => G[g].abbr)], ids.map(from => [glink(from), ...ids.map(to => {
      const x = (mm[from] || {})[to]; if (from === to || !x) return ''; return { td: { class: 'asp-cell', style: `--v:${Math.min(1, x.virupa / 60)}`, title: `${G[from].en} → ${G[to].en}: ${nth(x.house)} house, ${x.fraction || 'no graha drishti'}, ${fix(x.virupa, 1)} virūpa` }, v: `${x.fraction || '·'} (${Math.round(x.virupa)})` }; })])); };
    body = mat(m);
  }
  put(b, pageHead('aspects', 'Aspects', 'Sputa drishti as a percent of a full aspect (60 virūpas), for the rāśi chart and the navāṃśa, from every perspective.'),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'Chart'), tabs([['d1', 'D1 · rāśi'], ['d9', 'D9 · navāṃśa']], vg, k => { ASP_VARGA = k; route(); })),
      h('div', { class: 'grp' }, h('b', {}, 'View'), tabs(views, ASP_VIEW, k => { ASP_VIEW = k; route(); }))),
    card(null, body),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'Additional'), toggle('extra_aspects', 'additional graha aspects (retrograde shadow, parivartana)', () => route()), toggle('extra_rashi', 'additional rāśi aspects', () => route()))),
    S.layers.extra_aspects || S.layers.extra_rashi ? card('Additional aspects', extraTable(ADDITIONAL.filter(extraShown)), note(EXTRA_NOTE)) : null,
    note(a.note), a.house_conventions ? note(a.house_conventions) : null,
    note('Rāhu and Ketu have no sputa drishti in Bṛhat Parāśara Horā Śāstra, so their bars are empty; some programs assign them aspects by other conventions.'));
}, { blurb: 'every aspect, D1 and D9, as charts' });

/* ── strength ─────────────────────────────────────────────────────────────────────────────────────────────────── */
/* ṣaḍbala ratio bars with a dotted cut-off at 1.0: below it a graha is short of its required strength */
function ratioChart(sb) {
  const gs = SEVEN.filter(g => sb.grahas[g]), W = 640, rowH = 34, left = 92, right = 70, top = 26, max = 2;
  const H = top + gs.length * rowH + 30, x = v => left + Math.min(v, max) / max * (W - left - right);
  const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'ratio-chart', role: 'img', 'aria-label': 'Ṣaḍbala ratio to the required minimum, with the cut-off at 1' });
  for (const t of [0, .5, 1, 1.5, 2]) { s('line', { x1: x(t), y1: top - 6, x2: x(t), y2: H - 24, class: 'rc-grid' }, svg); const tx = s('text', { x: x(t), y: H - 8, class: 'rc-tick' }, svg); tx.textContent = t.toFixed(1); }
  gs.forEach((g, i) => {
    const r = sb.grahas[g].ratio, y = top + i * rowH, below = r < 1;
    const lab = s('text', { x: left - 10, y: y + rowH / 2 + 4, class: 'rc-lab' }, svg); lab.textContent = G[g].en;
    const bar = s('rect', { x: left, y: y + 7, width: Math.max(2, x(r) - left), height: rowH - 14, rx: 6, class: 'rc-bar' + (below ? ' below' : ''), fill: below ? '#d0563f' : G[g].dot }, svg);
    bar.style.setProperty('--w', (x(r) - left).toFixed(1) + 'px');
    const v = s('text', { x: x(r) + 8, y: y + rowH / 2 + 4, class: 'rc-val' + (below ? ' below' : '') }, svg); v.textContent = fix(r, 2) + (below ? ' below cut-off' : '');
    const t = s('title', {}, bar); t.textContent = `${G[g].en}: ${fix(sb.grahas[g].rupas, 2)} rūpa of ${sb.grahas[g].required_rupas} required = ${fix(r, 2)}`;
  });
  s('line', { x1: x(1), y1: top - 12, x2: x(1), y2: H - 24, class: 'rc-cut' }, svg);
  const c = s('text', { x: x(1) + 6, y: top - 12, class: 'rc-cutlab' }, svg); c.textContent = 'cut-off 1.0 (required minimum)';
  return svg;
}
page('strength', () => 'Strength', 'Strength', (b) => {
  const sb = val('vedic.shadbala'), bb = val('vedic.bhava_bala'), vs = val('vedic.vimshopaka'), bal = SPECIAL.balance;
  if (!sb && !vs) return put(b, missing('Strength', 'Ask for the shadbala and vimshopaka features.'));
  const out = [pageHead('strength', 'Strength', 'Ṣaḍbala, bhāva bala, viṃśopaka and the balance of elements.')];
  if (sb) {
    const rows = SEVEN.filter(g => sb.grahas[g]).map(g => { const x = sb.grahas[g]; return [glink(g), fix(x.sthana.total, 1), fix(x.dig, 1), fix(x.kala.total, 1), fix(x.chesta, 1), fix(x.naisargika, 1), fix(x.drik, 1), h('b', {}, fix(x.total_virupa, 0)), fix(x.rupas, 2), x.required_rupas, h('div', { class: 'barcell' }, h('div', { class: 'bar ratio-bar' + (x.ratio < 1 ? ' below' : ''), title: x.ratio < 1 ? 'below the cut-off of 1.0' : 'at or above the cut-off of 1.0' }, h('i', { style: `width:${Math.min(100, x.ratio / 2 * 100)}%` }), h('span', { class: 'cut' })), h('b', { class: x.ratio < 1 ? 'warn' : 'ok' }, fix(x.ratio, 2)))]; });
    out.push(card('Ṣaḍbala (virūpas)', table(['Graha', 'Sthāna', 'Dig', 'Kāla', 'Ceṣṭā', 'Naisargika', 'Dṛk', 'Total', 'Rūpas', 'Required', 'Ratio'], rows),
      note('Kāla bala uses: vāra ', sb.sky.vara_lord || '—', ', horā ', sb.sky.hora_lord || '—', ', abda ', sb.sky.abda_lord || '—', ', māsa ', sb.sky.masa_lord || '—', '. Conventions: rule pack vedic.bala.')));
    const parts = SEVEN.filter(g => sb.grahas[g]).map(g => { const x = sb.grahas[g]; return [glink(g), ...['uchcha', 'saptavargaja', 'ojayugma', 'kendradi', 'drekkana'].map(k => fix(x.sthana[k], 1)), ...['natonnata', 'paksha', 'tribhaga', 'abda', 'masa', 'vara', 'hora', 'ayana'].map(k => fix(x.kala[k], 1)), x.chesta_state]; });
    out.push(h('details', { class: 'card' }, h('summary', {}, 'Every component'), table(['Graha', 'Uchcha', 'Saptavargaja', 'Ojayugma', 'Kendrādi', 'Drekkāṇa', 'Natonnata', 'Pakṣa', 'Tribhāga', 'Abda', 'Māsa', 'Vāra', 'Horā', 'Ayana', 'Motion'], parts)));
  }
  if (vs) out.push(card('Viṃśopaka (out of 20)', table(['Graha', 'Ṣaḍvarga', 'Saptavarga', 'Daśavarga', 'Ṣoḍaśavarga'], SEVEN.filter(g => vs[g]).map(g => [glink(g), ...['shadvarga', 'saptavarga', 'dashavarga', 'shodashavarga'].map(k => h('div', { class: 'barcell' }, bar(vs[g][k].score, 20), fix(vs[g][k].score, 1)))]))));
  if (bb) out.push(card('Bhāva bala', table(['Bhāva', 'Sign', 'Lord', 'Lord’s ṣaḍbala', 'Dig', 'Drishti', 'Total', 'Rūpas'], bb.bhavas.map(x => [h('a', { href: '#/house/' + x.bhava }, x.bhava), SIGN_SKT[x.sign], glink(x.lord), fix(x.adhipati, 0), fix(x.dig, 0), fix(x.drishti, 1), fix(x.total_virupa, 0), h('div', { class: 'barcell' }, bar(x.rupas, 12), fix(x.rupas, 2))]))));
  if (sb && vs) {
    const score = { exalted: 5, moolatrikona: 4, own: 4, friend: 3, neutral: 2, enemy: 1, debilitated: 0 };
    out.push(card('Auspiciousness (at a glance)', table(['Graha', 'Ṣaḍbala ratio', 'Viṃśopaka (16)', 'Dignity', 'Good vargas', 'Avasthā'], SEVEN.filter(g => sb.grahas[g]).map(g => {
      const good = vs[g].shodashavarga.vargas ? Object.values(vs[g].shodashavarga.vargas).filter(v => ['own', 'great_friend', 'friend'].includes(v.dignity)).length : 0;
      return [glink(g), bar(sb.grahas[g].ratio, 2, 'var(--good)'), bar(vs[g].shodashavarga.score, 20, 'var(--good)'), bar(score[(STRENGTH[g] || {}).dignity] ?? 2, 5, 'var(--good)'), `${good} / 16`, (STRENGTH[g] || {}).avastha]; })),
      note('Bars put the computed values side by side; they are not a verdict.')));
  }
  if (bal) out.push(h('div', { class: 'grid2' }, card('Elements (7 grahas + lagna)', h('div', { class: 'elbars' }, Object.entries(bal.elements).map(([k, v]) => h('div', { class: 'elb el-' + k }, h('i', { style: `height:${v * 22}px` }), h('b', {}, v), h('small', {}, k))))),
    card('Element × modality', table(['', 'Fire', 'Earth', 'Air', 'Water'], ['movable', 'fixed', 'dual'].map(m => [m, ...['fire', 'earth', 'air', 'water'].map(e => (bal.grid[`${e}_${m}`] || []).map(x => x === 'lagna' ? 'As' : G[x].abbr).join(' '))])))));
  put(b, out);
}, { blurb: 'ṣaḍbala, bhāva bala, viṃśopaka' });

/* ── aṣṭakavarga ──────────────────────────────────────────────────────────────────────────────────────────────── */
let PRASTARA = 'sun';
page('ashtakavarga', () => 'Aṣṭakavarga', 'Strength', (b) => {
  const a = val('vedic.ashtakavarga'); if (!a) return put(b, missing('Aṣṭakavarga', 'It needs an exact birth time and place.'));
  const heat = (v, max) => ({ td: { class: 'heat', style: `--h:${Math.min(1, v / max)}` }, v: String(v) });
  const head = SIGN_SKT.map((n, i) => LAGNA_SIGN != null ? `${n.slice(0, 4)} (${houseOf(i)})` : n.slice(0, 4));
  const rows = SEVEN.map(g => [glink(g), ...a.bhinnashtakavarga[g].bindus.map(v => heat(v, 8)), a.bhinnashtakavarga[g].total]);
  rows.push([h('b', {}, 'Sarva'), ...a.sarvashtakavarga.bindus.map(v => heat(v, 40)), a.sarvashtakavarga.total]);
  const contrib = a.bhinnashtakavarga[PRASTARA].contributions, cs = Object.keys(contrib);
  const pr = table(['Contributor', ...SIGN_AB], cs.map(c => [c === 'lagna' ? 'Lagna' : glink(c), ...Array.from({ length: 12 }, (_, i) => contrib[c].includes(i) ? { td: { class: 'bindu' }, v: '●' } : '')]).concat([[h('b', {}, 'Total'), ...a.bhinnashtakavarga[PRASTARA].bindus.map(v => h('b', {}, v))]]));
  put(b, pageHead('ashtakavarga', 'Aṣṭakavarga', 'Samudāya (sarva) and bhinna bindus by sign; prastāra shows who contributed each bindu.'),
    card('Samudāya and bhinna aṣṭakavarga', table(['', ...head, 'Total'], rows), note('Totals are fixed by the tables: 48, 49, 39, 54, 56, 52, 39; Sarva 337.')),
    card('Prastāra aṣṭakavarga', tabs(SEVEN.map(g => [g, G[g].en]), PRASTARA, g => { PRASTARA = g; route(); }), pr));
}, { blurb: 'bindus by sign and contributor' });

/* ── pañcāṅga ─────────────────────────────────────────────────────────────────────────────────────────────────── */
page('panchanga', () => 'Pañcāṅga', 'Time', (b) => {
  const p = val('vedic.panchanga'); if (!p) return put(b, missing('The pañcāṅga', 'It needs a place.'));
  const lim = (t, v, e) => h('div', { class: 'stat' }, h('div', { class: 'stat-l' }, t), h('div', { class: 'stat-v' }, v), h('div', { class: 'stat-s' }, e ? 'ends ' + fmtTime(e.local).slice(0, 16) : ''));
  const hm = x => x ? fmtTime(x.local).slice(11, 16) : '—';
  put(b, pageHead('panchanga', 'Pañcāṅga of the birth moment', `Sunrise convention: ${p.sunrise_convention}.`),
    h('div', { class: 'stats' }, lim('Tithi', `${p.tithi.paksha === 'shukla' ? 'Śukla' : 'Kṛṣṇa'} ${p.tithi.name}`, p.tithi.ends), lim('Vāra', p.vara ? p.vara.name : '—'), lim('Nakshatra', `${p.nakshatra.name} ${p.nakshatra.pada}`, p.nakshatra.ends), lim('Yoga', p.yoga.name, p.yoga.ends), lim('Karaṇa', p.karana.name, p.karana.ends)),
    h('div', { class: 'grid3' }, card('Sun and Moon', facts([['Sunrise', hm(p.sunrise)], ['Sunset', hm(p.sunset)], ['Moonrise', p.moonrise ? hm(p.moonrise) : 'none that day'], ['Moonset', p.moonset ? hm(p.moonset) : 'none that day'], ['Day length', p.day_length_hours && fix(p.day_length_hours, 2) + ' h']])),
      card('Kālas', facts([['Rāhu kālam', p.rahu_kalam && `${hm(p.rahu_kalam.start)}–${hm(p.rahu_kalam.end)}`], ['Yamagaṇḍam', p.yamagandam && `${hm(p.yamagandam.start)}–${hm(p.yamagandam.end)}`], ['Gulika kālam', p.gulika_kalam && `${hm(p.gulika_kalam.start)}–${hm(p.gulika_kalam.end)}`]])),
      card('Special lagnas', facts(['bhava_lagna', 'hora_lagna', 'ghati_lagna'].filter(k => p.special_lagnas && p.special_lagnas[k]).map(k => [k.replace('_', ' '), `${SIGN_SKT[p.special_lagnas[k].sign]} ${inSign(p.special_lagnas[k].longitude_deg)}`]).concat([['Sun taken at', p.special_lagnas && p.special_lagnas.sun_taken_at]])))),
    p.horas ? h('div', { class: 'grid2' }, card('Horās', table(['#', 'Lord', 'From', 'To'], p.horas.map(x => [x.current ? h('b', {}, x.n + ' ◀') : x.n, glink(x.lord), hm(x.start), hm(x.end)]))),
      card('Choghaḍiyās', table(['#', 'Name', 'Quality', 'From', 'To'], p.choghadiya.map(x => [x.current ? h('b', {}, x.n + ' ◀') : x.n, `${x.name} (${x.period})`, x.quality, hm(x.start), hm(x.end)])))) : null);
}, { blurb: 'tithi, vāra, nakshatra, yoga, karaṇa' });

/* ── lagnas and special points: a tile per point, each opening its own page ─────────────────────────────────── */
const KARAKA_ORDER = ['atmakaraka', 'amatyakaraka', 'bhratrikaraka', 'matrikaraka', 'pitrikaraka', 'putrakaraka', 'gnatikaraka', 'darakaraka'];
/* every point this chart has: its value, the sign(s) it marks, an exact longitude or a graha where there is one */
function chartPoints() {
  const sp = SPECIAL, sl = (val('vedic.panchanga') || {}).special_lagnas || {}, ck = val('vedic.jaimini.chara_karakas'), ar = val('vedic.jaimini.arudha_padas'), out = [];
  const add = (id, value, signs, extra = {}) => { if (value != null) out.push(Object.assign({ id, value, signs: (signs || []).filter(x => x != null) }, extra)); };
  const at = lon => `${SIGN_SKT[signOf(lon)]} ${inSign(lon)}`;
  if (ASC != null) add('lagna', at(ASC), [signOf(ASC)], { lon: ASC });
  for (const k of ['bhava_lagna', 'hora_lagna', 'ghati_lagna']) if (sl[k]) add(k, at(sl[k].longitude_deg), [sl[k].sign], { lon: sl[k].longitude_deg, rows: [['Sun taken at', sl.sun_taken_at]] });
  if (sp.indu_lagna) add('indu_lagna', SIGN_SKT[sp.indu_lagna.sign], [sp.indu_lagna.sign], { rows: [['Kalā sum', sp.indu_lagna.kalas_sum], ['9th lord from the lagna', glink(sp.indu_lagna.ninth_lord_from_lagna)], ['9th lord from the Moon', glink(sp.indu_lagna.ninth_lord_from_moon)]] });
  if (ar && ar.A1_arudha_lagna) add('arudha_lagna', SIGN_SKT[ar.A1_arudha_lagna.pada_sign], [ar.A1_arudha_lagna.pada_sign], { rows: [['Bhāva from the lagna', ar.A1_arudha_lagna.pada_house], ['Lagna lord', glink(ar.A1_arudha_lagna.lord)], ['Lord in', SIGN_SKT[ar.A1_arudha_lagna.lord_sign]]] });
  if (ar && ar.A12_upapada) add('upapada', SIGN_SKT[ar.A12_upapada.pada_sign], [ar.A12_upapada.pada_sign], { rows: [['Bhāva from the lagna', ar.A12_upapada.pada_house], ['12th lord', glink(ar.A12_upapada.lord)], ['Lord in', SIGN_SKT[ar.A12_upapada.lord_sign]]] });
  if (sp.yogi) {
    add('yogi_point', at(sp.yogi.point), [sp.yogi.sign], { lon: sp.yogi.point });
    add('yogi', G[sp.yogi.yogi_graha].en, [GR[sp.yogi.yogi_graha] && signOf(GR[sp.yogi.yogi_graha].lon)], { graha: sp.yogi.yogi_graha });
    add('duplicate_yogi', G[sp.yogi.duplicate_yogi].en, [GR[sp.yogi.duplicate_yogi] && signOf(GR[sp.yogi.duplicate_yogi].lon)], { graha: sp.yogi.duplicate_yogi });
  }
  if (sp.avayogi) add('avayogi', G[sp.avayogi.avayogi_graha].en, [GR[sp.avayogi.avayogi_graha] && signOf(GR[sp.avayogi.avayogi_graha].lon)], { graha: sp.avayogi.avayogi_graha, rows: [['Avayogi point', at(sp.avayogi.point)], ['Its nakshatra', sp.avayogi.nakshatra]] });
  if (sp.dagdha_rashis) add('dagdha_rashis', sp.dagdha_rashis.signs.map(x => SIGN_SKT[x]).join(', ') || 'none', sp.dagdha_rashis.signs, { rows: [['Tithi', `${sp.dagdha_rashis.tithi} (${sp.dagdha_rashis.tithi_in_paksha} in the pakṣa)`]] });
  if (sp.navamsha_64th_from_moon) { const a = sp.navamsha_64th_from_lagna || {}, m = sp.navamsha_64th_from_moon; add('navamsha_64th', `${a.sign != null ? SIGN_SKT[a.sign] : '—'} / ${SIGN_SKT[m.sign]}`, [a.sign, m.sign], { rows: [['From the lagna', a.sign != null ? `${SIGN_SKT[a.sign]}, lord ${G[a.lord] ? G[a.lord].en : a.lord || '—'}` : '—'], ['From the Moon', `${SIGN_SKT[m.sign]}, lord ${G[m.lord] ? G[m.lord].en : m.lord || '—'}`]] }); }
  if (sp.drekkana_22nd_from_moon) { const a = sp.drekkana_22nd_from_lagna || {}, m = sp.drekkana_22nd_from_moon; add('drekkana_22nd', `${a.sign != null ? SIGN_SKT[a.sign] : '—'} / ${SIGN_SKT[m.sign]}`, [a.sign, m.sign], { rows: [['From the lagna', a.sign != null ? `${SIGN_SKT[a.sign]}, khara lord ${G[a.lord] ? G[a.lord].en : '—'}` : '—'], ['From the Moon', `${SIGN_SKT[m.sign]}, khara lord ${G[m.lord] ? G[m.lord].en : '—'}`]] }); }
  if (sp.graha_yuddha) add('graha_yuddha', sp.graha_yuddha.length ? sp.graha_yuddha.map(w => w.grahas.map(g => G[g] ? G[g].abbr : g).join(' × ')).join(', ') : 'none', sp.graha_yuddha.flatMap(w => w.grahas.map(g => GR[g] && signOf(GR[g].lon))));
  if (sp.kartari) { const ks = Object.entries(sp.kartari); add('kartari', ks.length ? `${ks.length} hemmed` : 'none', ks.map(([k]) => k === 'lagna' ? LAGNA_SIGN : GR[k] && signOf(GR[k].lon))); }
  if (ck) { const ak = ck.eight_karaka_scheme && ck.eight_karaka_scheme.atmakaraka; add('chara_karakas', ak ? `ātmakāraka ${G[ak.graha].en}` : 'computed', ak ? [signOf(GR[ak.graha].lon)] : [], { graha: ak && ak.graha }); }
  if (ck && ck.karakamsa_sign != null) add('karakamsa', SIGN_SKT[ck.karakamsa_sign], [ck.karakamsa_sign]);
  if (ar) add('arudha_padas', `${Object.keys(ar).length} padas`, []);
  const ups = UPAGRAHA.filter(u => GR_ALL[u]);
  if (ups.length) add('upagrahas', `${ups.length} points`, ups.map(u => signOf(GR_ALL[u].lon)), { rows: ups.map(u => [G[u].en + ` (${G[u].abbr})`, `${SIGN_SKT[signOf(GR_ALL[u].lon)]} ${inSignDM(GR_ALL[u].lon)} · bhāva ${houseOf(signOf(GR_ALL[u].lon)) || '—'}`]) });
  if (sp.lordships) { const gl = Object.values(sp.lordships.grahas || {}); add('functional_nature', `${gl.filter(x => /benefic/.test(x.functional_nature)).length} benefic · ${gl.filter(x => /malefic/.test(x.functional_nature)).length} malefic`, []); }
  return out;
}
/* everything about one sign in this chart: lord and where it sits, bhāva, occupants, aspects, SAV */
function signElements(sign) {
  const lord = SIGN_LORD[sign], lg = GR[lord], sav = val('vedic.ashtakavarga'), dag = (SPECIAL.dagdha_rashis || { signs: [] }).signs.includes(sign);
  return facts([['Sign', `${GLYPH[sign]} ${SIGN_SKT[sign]} · ${SIGN_EN[sign]} (${ELEMENT[sign % 4]}, ${MODALITY[sign % 3]})`], ['Bhāva from the lagna', houseOf(sign)],
    ['Lord', lg ? h('span', {}, glink(lord), ` in ${SIGN_SKT[signOf(lg.lon)]} ${inSignDM(lg.lon)}, bhāva ${houseOf(signOf(lg.lon)) || '—'}, ${(STRENGTH[lord] || {}).dignity || ''}`) : G[lord].en],
    ['Grahas in it', h('span', {}, grahasIn(sign * 30, sign * 30 + 30).filter(g => G[g]).map(g => h('span', { class: 'chip' }, dot(g), `${G[g].abbr} ${inSignDM(GR[g].lon)}`)))],
    ['Graha drishti on it', (ASPECTED[sign] || []).map(d => `${G[d.from].abbr} (${nth(d.house_from_graha)}${d.kind === 'special' ? ', special' : ''})`).join(', ') || 'none'],
    ['Rāśi drishti on it', (RASHI_ASPECTED[sign] || []).map(r => G[r.from].abbr).join(', ') || 'none'],
    ['SAV bindus', sav ? sav.sarvashtakavarga.bindus[sign] : null], ['Dagdha', dag ? 'yes (burnt sign by tithi)' : null]]);
}
page('special', () => 'Lagnas & points', 'Systems', (b) => {
  const pts = chartPoints(); if (!pts.length) return put(b, missing('Special points', 'Ask for the special_points and jaimini features.'));
  const ck = val('vedic.jaimini.chara_karakas'), ar = val('vedic.jaimini.arudha_padas'), sp = SPECIAL;
  const tile = p => { const r = READ.points[p.id] || {}; return h('a', { class: 'card point', href: '#/point/' + p.id }, h('div', { class: 'stat-l' }, r.name || p.id.replace(/_/g, ' ')),
    h('div', { class: 'stat-v' }, p.graha && G[p.graha] ? h('span', {}, dot(p.graha), p.value) : p.value), h('p', { class: 'note' }, r.why || ''), h('span', { class: 'more' }, 'What it means →')); };
  const kar = sch => table(['Kāraka', 'Graha', 'Degrees in sign', 'Navāṃśa'], Object.entries(ck[sch]).sort((a, bb) => KARAKA_ORDER.indexOf(a[0]) - KARAKA_ORDER.indexOf(bb[0])).map(([k, v]) => [k, glink(v.graha), fix(v.degrees_in_sign, 3), v.navamsa_sign != null ? SIGN_SKT[v.navamsa_sign] : '—']));
  put(b, pageHead('special', 'Lagnas and special points', 'Each point with its value in this chart and why it matters. Open one for what it is, how it is read and everything around it in the chart.'),
    h('div', { class: 'ptgrid' }, pts.map(tile)),
    ck ? h('div', { class: 'grid2' }, card('Chara kārakas · 8 (with Rahu)', kar('eight_karaka_scheme')), card('Chara kārakas · 7', kar('seven_karaka_scheme'), note(`Kārakāṃśa: ${ck.karakamsa_sign != null ? SIGN_SKT[ck.karakamsa_sign] : '—'}`))) : null,
    ar ? card('Ārūḍha padas', table(['Pada', 'Of bhāva', 'Lord', 'Lord in', 'Pada sign', 'Pada bhāva'], Object.entries(ar).sort((a, bb) => a[1].house - bb[1].house).map(([k, v]) => [k.replace('_', ' '), v.house, glink(v.lord), SIGN_SKT[v.lord_sign], SIGN_SKT[v.pada_sign], v.pada_house]))) : null,
    sp.lordships ? card('Lordships and functional nature', table(['Graha', 'Owns', 'Placed in', 'Nature'], SEVEN.map(g => [glink(g), sp.lordships.grahas[g].houses_owned.join(', '), sp.lordships.grahas[g].placed_in_house, sp.lordships.grahas[g].functional_nature])), note(sp.lordships.rule)) : null);
}, { blurb: 'special lagnas, yogi, Jaimini, each explained' });

page('point', (id) => ((READ.points || {})[id] || {}).name || 'Point', 'Systems', (b, id) => {
  const pts = chartPoints(), p = pts.find(x => x.id === id) || pts[0];
  if (!p) return put(b, missing('Special points', 'Ask for the special_points feature.'));
  const r = READ.points[p.id] || {}, k = pts.indexOf(p), sp = SPECIAL;
  const nav = h('div', { class: 'pnav' }, k > 0 ? h('a', { class: 'btn', href: '#/point/' + pts[k - 1].id }, '← ' + ((READ.points[pts[k - 1].id] || {}).name || pts[k - 1].id)) : h('span', {}),
    h('a', { class: 'btn', href: '#/special' }, 'All points'), k < pts.length - 1 ? h('a', { class: 'btn', href: '#/point/' + pts[k + 1].id }, ((READ.points[pts[k + 1].id] || {}).name || pts[k + 1].id) + ' →') : h('span', {}));
  const lonRows = p.lon != null ? [['Longitude', `${fix(p.lon, 4)}° · ${SIGN_SKT[signOf(p.lon)]} ${inSign(p.lon)}`], ['Nakshatra', `${NAKSHATRA[nakOf(p.lon)]} pada ${padaOf(p.lon)} (lord ${G[NAK_LORD[nakOf(p.lon) % 9]].en})`], ['Navāṃśa', SIGN_SKT[navSignOf(p.lon)]]] : [];
  const gRows = p.graha && GR[p.graha] ? [['Graha', glink(p.graha)], ['Position', `${SIGN_SKT[signOf(GR[p.graha].lon)]} ${inSignDM(GR[p.graha].lon)}`], ['Bhāva', houseOf(signOf(GR[p.graha].lon))], ['Dignity', (STRENGTH[p.graha] || {}).dignity],
    ['Ṣaḍbala ratio', (((val('vedic.shadbala') || {}).grahas || {})[p.graha] || {}).ratio != null ? fix(val('vedic.shadbala').grahas[p.graha].ratio, 2) : null], ['Functional nature', sp.lordships && sp.lordships.grahas[p.graha] ? sp.lordships.grahas[p.graha].functional_nature : null]] : [];
  let extra = null;
  if (p.id === 'kartari' && sp.kartari) extra = table(['Hemmed', 'Kind', '2nd from it', '12th from it'], Object.entries(sp.kartari).map(([k2, v]) => [k2 === 'lagna' ? 'Lagna' : glink(k2), v.kind.replace('_', ' '), (v.second || []).map(g => G[g] ? G[g].abbr : g).join(' '), (v.twelfth || []).map(g => G[g] ? G[g].abbr : g).join(' ')]));
  if (p.id === 'graha_yuddha') extra = sp.graha_yuddha && sp.graha_yuddha.length ? table(['Grahas', 'Separation', 'Winner'], sp.graha_yuddha.map(w => [w.grahas.map(g => G[g].en).join(' × '), w.separation_deg != null ? fix(w.separation_deg, 3) + '°' : '—', (() => { const [a, z] = w.grahas; return GR[a] && GR[z] && GR[a].lat != null ? h('span', {}, glink(GR[a].lat >= GR[z].lat ? a : z), ' (further north)') : '—'; })()])) : note('No two of Mars, Mercury, Jupiter, Venus and Saturn are within 1° in this chart.');
  if (p.id === 'chara_karakas' && val('vedic.jaimini.chara_karakas')) { const ck = val('vedic.jaimini.chara_karakas'); extra = table(['Kāraka', 'Graha', 'Degrees in sign'], Object.entries(ck.eight_karaka_scheme).sort((a, bb) => KARAKA_ORDER.indexOf(a[0]) - KARAKA_ORDER.indexOf(bb[0])).map(([k2, v]) => [k2, glink(v.graha), fix(v.degrees_in_sign, 3)])); }
  if (p.id === 'arudha_padas' && val('vedic.jaimini.arudha_padas')) extra = table(['Pada', 'Of bhāva', 'Pada sign', 'Pada bhāva'], Object.entries(val('vedic.jaimini.arudha_padas')).sort((a, bb) => a[1].house - bb[1].house).map(([k2, v]) => [k2.replace('_', ' '), v.house, SIGN_SKT[v.pada_sign], v.pada_house]));
  if (p.id === 'functional_nature' && sp.lordships) extra = h('div', {}, table(['Graha', 'Owns', 'Placed in', 'Nature'], SEVEN.map(g => [glink(g), sp.lordships.grahas[g].houses_owned.join(', '), sp.lordships.grahas[g].placed_in_house, sp.lordships.grahas[g].functional_nature])), note(sp.lordships.rule));
  const signs = [...new Set(p.signs)].slice(0, 3);
  const chart = chartSVG(S.chart === 'north' ? 'north' : 'south', 1);
  signs.forEach(sg => $$(`.cell[data-sign="${sg}"]`, chart).forEach(c => c.classList.add('yoga-src')));
  put(b, nav, pageHead('special', r.name || p.id, r.what || ''),
    h('div', { class: 'chips' }, h('span', { class: 'chip' }, 'In this chart: ', h('b', {}, p.value)), h('span', { class: 'chip' }, label('tradition_rule'), ' computed by the engine')),
    h('div', { class: 'grid2' },
      card('In this chart', facts([['Value', p.value], ...lonRows, ...gRows, ...(p.rows || [])]), extra),
      card('Where it falls', h('div', { class: 'figure' }, chart), note(signs.length ? 'Highlighted: the sign(s) this point marks.' : 'This point is a table, not a single sign.'))),
    signs.length ? h('div', { class: 'grid' + Math.min(3, Math.max(2, signs.length)) }, signs.map(sg => card(`${SIGN_SKT[sg]}: the elements around it`, signElements(sg)))) : null,
    h('div', { class: 'grid3 reading' },
      r.why ? card('Why it matters', h('p', {}, r.why)) : null,
      r.read ? card('How to read it', h('ul', { class: 'list' }, r.read.map(x => h('li', {}, x)))) : null,
      r.what ? card('What it is', h('p', {}, r.what), r.sources ? note('Sources: ' + r.sources.join(', ') + '.') : null) : null),
    readingLabel());
}, { hidden: true, section: 'point' });

/* ── yogas: only those present, each opening its own page ───────────────────────────────────────────────────── */
const READ = D.reading || { yogas: {}, points: {}, meta: {} };
const DEBIL = { sun: 6, moon: 7, mars: 3, mercury: 11, jupiter: 9, venus: 5, saturn: 0 };
const NAT_BENEFIC = ['jupiter', 'venus', 'mercury', 'moon'], NAT_MALEFIC = ['sun', 'mars', 'saturn', 'rahu', 'ketu'];
const YOGA_ALL = () => YOGAS ? YOGAS.yogas.map(y => Object.assign({ group: 'yoga' }, y)).concat(YOGAS.doshas.map(y => Object.assign({ group: 'dosha' }, y))) : [];
const YOGA_PRESENT = () => YOGA_ALL().filter(y => y.present);
const readingLabel = () => h('p', { class: 'note' }, label(READ.meta.epistemic || 'interpretation'), ' ', READ.meta.attribution || '');
page('yogas', () => 'Yogas', 'Systems', (b) => {
  if (!YOGAS) return put(b, missing('Yogas', 'Ask for the yogas feature.'));
  const present = YOGA_PRESENT(), absent = YOGA_ALL().filter(y => !y.present);
  const tile = y => h('a', { class: 'card yoga on ' + y.group, href: '#/yoga/' + y.id },
    h('div', { class: 'yh' }, h('b', {}, y.name), h('span', { class: 'ystate' }, y.group === 'dosha' ? 'dosha' : y.category.replace(/_/g, ' '))),
    h('p', {}, (READ.yogas[y.id] || {}).summary || y.rule),
    h('div', { class: 'chips' }, (y.grahas || []).map(g => GR[g] ? h('span', { class: 'chip' }, dot(g), `${G[g].abbr} ${SIGN_AB[signOf(GR[g].lon)]} ${inSignDM(GR[g].lon)}`) : null)),
    h('span', { class: 'more' }, 'Read it →'));
  const ys = present.filter(y => y.group === 'yoga'), ds = present.filter(y => y.group === 'dosha');
  put(b, pageHead('yogas', 'Yogas and doshas in this chart', `${ys.length} yoga${ys.length === 1 ? '' : 's'} and ${ds.length} dosha${ds.length === 1 ? '' : 's'} hold here. Open one for how it forms in this chart, how strong it is, when it is active and what the tradition says about it.`),
    ys.length ? h('div', {}, h('h3', { class: 'section-t' }, 'Yogas'), h('div', { class: 'ygrid' }, ys.map(tile))) : note('No yoga in the rule pack holds for this chart.'),
    ds.length ? h('div', {}, h('h3', { class: 'section-t' }, 'Doshas'), h('div', { class: 'ygrid' }, ds.map(tile))) : null,
    h('details', { class: 'card' }, h('summary', {}, `Also checked, not present here (${absent.length})`),
      h('p', { class: 'note' }, absent.map(y => y.name + (y.present === null || y.evaluable === false ? ' (not evaluable)' : '')).join(' · ')),
      note('Rule pack vedic.yogas: every rule is stated in words; ESHANA computes whether it holds.')));
}, { blurb: 'the yogas and doshas this chart holds' });

/* when a set of grahas is active in Vimśottarī: their mahādaśās, and their antardaśās in the running mahādaśā */
function activation(ids) {
  const vim = val('vedic.dasha.vimshottari'); if (!vim) return null;
  const now = T(REF_UTC || new Date().toISOString()), lords = new Set(ids.map(x => x.toLowerCase()));
  const when = (a, z) => T(a) <= now && now < T(z) ? h('b', {}, 'running') : T(a) > now ? 'ahead' : 'past';
  const md = vim.mahadashas.filter(m => lords.has(m.lord.toLowerCase())).map(m => [glink(m.lord.toLowerCase()), dateOnly(m.start_utc), dateOnly(m.end_utc), when(m.start_utc, m.end_utc)]);
  const cur = vim.mahadashas.find(m => T(m.start_utc) <= now && now < T(m.end_utc));
  const ad = cur ? (cur.antardashas || []).filter(x => lords.has(x.lord.toLowerCase())).map(x => [`${cur.lord} › ${x.lord}`, dateOnly(x.start_utc), dateOnly(x.end_utc), when(x.start_utc, x.end_utc)]) : [];
  return { md, ad, cur };
}
/* checks practitioners apply to the grahas of a yoga, evaluated on this chart */
function grahaChecks(g) {
  const gr = GR[g], st = STRENGTH[g] || {}, sb = ((val('vedic.shadbala') || {}).grahas || {})[g], asp = (val('vedic.aspects') || {}).d1 || {};
  const hs = houseOf(signOf(gr.lon)), d9 = vargaSign(g, 9);
  const from = Object.entries(asp).filter(([f, row]) => f !== g && row[g] && row[g].virupa >= 30).map(([f]) => f);
  const ben = from.filter(f => NAT_BENEFIC.includes(f)), mal = from.filter(f => NAT_MALEFIC.includes(f));
  const ok = (cond, yes, no) => h('span', { class: cond ? 'ok' : 'warn' }, cond ? '✓ ' + yes : '✗ ' + no);
  return [glink(g), `${SIGN_SKT[signOf(gr.lon)]} ${inSignDM(gr.lon)}`, hs || '—', st.dignity || '—',
    sb ? ok(sb.ratio >= 1, `ṣaḍbala ${fix(sb.ratio, 2)}`, `ṣaḍbala ${fix(sb.ratio, 2)}`) : '—',
    ok(!st.combust, 'not combust', 'combust'),
    d9 == null ? '—' : DEBIL[g] != null ? ok(d9 !== DEBIL[g], `D9 ${SIGN_AB[d9]}${d9 === signOf(gr.lon) ? ' (vargottama)' : ''}`, `D9 ${SIGN_AB[d9]} (debilitated)`) : `D9 ${SIGN_AB[d9]}`,
    hs ? ok([1, 4, 5, 7, 9, 10].includes(hs), 'kendra/trikoṇa', [6, 8, 12].includes(hs) ? 'dusthāna' : 'other house') : '—',
    h('span', {}, ben.length ? h('span', { class: 'ok' }, '+' + ben.map(x => G[x].abbr).join(' ')) : '', ben.length && mal.length ? ' ' : '',
      mal.length ? h('span', { class: 'warn' }, '−' + mal.map(x => G[x].abbr).join(' ')) : '', !ben.length && !mal.length ? 'none ≥ 30 virūpa' : '')];
}
page('yoga', (id) => (YOGA_ALL().find(y => y.id === id) || {}).name || 'Yoga', 'Systems', (b, id) => {
  if (!YOGAS) return put(b, missing('Yogas', 'Ask for the yogas feature.'));
  const list = YOGA_PRESENT(), y = YOGA_ALL().find(x => x.id === id) || list[0];
  if (!y) return put(b, h('a', { class: 'btn', href: '#/yogas' }, '← Yogas'), note('No yoga holds in this chart.'));
  const ref = READ.yogas[y.id] || {}, k = list.findIndex(x => x.id === y.id), gs = (y.grahas || []).filter(g => GR[g]);
  const nav = h('div', { class: 'pnav' }, k > 0 ? h('a', { class: 'btn', href: '#/yoga/' + list[k - 1].id }, '← ' + list[k - 1].name) : h('span', {}),
    h('a', { class: 'btn', href: '#/yogas' }, 'All yogas'), k >= 0 && k < list.length - 1 ? h('a', { class: 'btn', href: '#/yoga/' + list[k + 1].id }, list[k + 1].name + ' →') : h('span', {}));
  const chart = chartSVG(S.chart === 'north' ? 'north' : 'south', 1);
  gs.forEach(g => $$(`.cell[data-sign="${signOf(GR[g].lon)}"]`, chart).forEach(c => c.classList.add('yoga-src')));
  const act = activation(gs);
  put(b, nav, pageHead('yogas', y.name, ref.summary || ''),
    h('div', { class: 'chips' }, h('span', { class: 'chip ' + (y.present ? 'ok' : 'warn') }, y.present ? 'present in this chart' : 'not present'), h('span', { class: 'chip' }, y.group),
      h('span', { class: 'chip' }, y.category.replace(/_/g, ' ')), h('span', { class: 'chip' }, label('tradition_rule'), ' vedic.yogas')),
    h('div', { class: 'grid2' },
      card('How it forms here', facts([['Rule', y.rule], ['Grahas', gs.length ? h('span', {}, gs.map(g => h('span', { class: 'chip' }, dot(g), G[g].en))) : '—']]),
        gs.length ? h('div', { class: 'checks' }, gs.map(g => { const c = grahaChecks(g);
          return card(null, h('h4', {}, c[0]), facts(['Position', 'Bhāva', 'Dignity', 'Strength', 'Combustion', 'Navāṃśa', 'House', 'Aspected by (+ benefic, − malefic)'].map((k2, i) => [k2, c[i + 1]]))); })) : null,
        note('The usual checks for judging a yoga (strength, combustion, navāṃśa dignity, house, benefic or malefic influence), evaluated on this chart. They inform a reading; they are not a verdict.')),
      card('In the chart', h('div', { class: 'figure' }, chart), note('Highlighted: the signs of the grahas forming it. Hover a graha to see what it aspects.'))),
    act ? card('When it is active (Vimśottarī)', !gs.length ? note('This yoga is not formed by particular grahas (it is a pattern of the whole chart), so it is not tied to the periods of one graha.') : h('div', { class: 'grid2' },
      act.md.length ? table(['Mahādaśā', 'From', 'To', ''], act.md) : note('No mahādaśā of these grahas in the computed span.'),
      act.ad.length ? table([`Antardaśā in ${act.cur.lord}`, 'From', 'To', ''], act.ad) : note(act.cur ? `No antardaśā of these grahas in the running ${act.cur.lord} mahādaśā.` : '')),
      note('A yoga is usually read as giving its results in the periods of the grahas that form it.')) : null,
    h('div', { class: 'grid3 reading' },
      ref.effects ? card('Traditionally associated with', h('ul', { class: 'list' }, ref.effects.map(x => h('li', {}, x)))) : null,
      ref.read ? card('How it is read', h('ul', { class: 'list' }, ref.read.map(x => h('li', {}, x)))) : null,
      ref.cancel ? card('What weakens or cancels it', h('ul', { class: 'list' }, ref.cancel.map(x => h('li', {}, x)))) : null),
    ref.sources ? note('Sources: ' + ref.sources.join(', ') + '.') : null, readingLabel());
}, { hidden: true, section: 'yoga' });

/* ── KP ───────────────────────────────────────────────────────────────────────────────────────────────────────── */
page('kp', () => 'KP', 'Systems', (b) => {
  const k = val('vedic.kp'); if (!k) return put(b, missing('KP', 'It needs an exact birth time and place.'));
  const cusps = Object.entries(k.cusps).map(([hh, c]) => [h('a', { href: '#/house/' + hh }, hh), `${SIGN_SKT[signOf(c.cusp)]} ${inSign(c.cusp)}`, c.lords.sign_lord, c.lords.star_lord, c.lords.sub_lord, c.lords.sub_sub_lord, c.A_in_star_of_occupants.join(', '), c.B_occupants.join(', '), c.C_in_star_of_lord.join(', '), c.D_lord]);
  const pl = Object.entries(k.planets).sort((a, bb) => GRAHA_ORDER.indexOf(a[0]) - GRAHA_ORDER.indexOf(bb[0])).map(([g, l]) => { const sv = k.significators_planet_view[g]; return [glink(g), l.sign_lord, l.star_lord, l.sub_lord, l.sub_sub_lord, sv.A_house_of_star_lord, sv.B_house_occupied, sv.C_houses_owned_by_star_lord.join(', '), sv.D_houses_owned.join(', ')]; });
  put(b, pageHead('kp', 'KP (Krishnamurti Paddhati)', k.note || 'Placidus cusps, Vimśottarī sub-division of every nakshatra.'),
    card('Cusps and house significators', table(['Cusp', 'Position', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A', 'B', 'C', 'D'], cusps)),
    card('Planets and planet-view significators', table(['Graha', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A', 'B', 'C', 'D'], pl)),
    k.ruling_planets ? card('Ruling planets at the chart instant', facts(Object.entries(k.ruling_planets).map(([kk, v]) => [kk.replaceAll('_', ' '), v]))) : null,
    kpSignificators(k.significator_table));
  /* the significator table (vedic.kp significator_table, built by the annual module from the same KP value) */
  function kpSignificators(t) {
    if (!t) return null;
    const ab = g => g ? (G[g] || G.lagna).abbr : '—';
    const chips = list => h('span', { class: 'chips' }, (list || []).map(g => h('a', { class: 'chip kp-chip', href: '#/planet/' + g, title: (G[g] || {}).en || g }, dot(g), ab(g))));
    const hov = (el, title, rows) => { el.addEventListener('mousemove', ev => tip(ev, title, rows, 'tradition_rule · KP significators')); el.addEventListener('mouseleave', untip); return el; };
    const houses = t.houses.map(r => [hov(h('a', { href: '#/house/' + r.house }, nth(r.house)), `Bhāva ${r.house}`, [['Cusp', `${SIGN_SKT[r.sign]} ${inSign(r.cusp)}`], ['Sign · star · sub · sub-sub', [r.sign_lord, r.star_lord, r.sub_lord, r.sub_sub_lord].map(ab).join(' · ')], ['Significators', (t.significators_by_house[r.house] || []).map(ab).join(' ')]]),
      `${SIGN_AB[r.sign]} ${inSignDM(r.cusp)}`, ab(r.star_lord), ab(r.sub_lord), ab(r.sub_sub_lord), chips(r.in_star_of_occupants), chips(r.occupants), chips(r.in_star_of_owner), chips([r.owner])]);
    const grahas = t.grahas.map(r => [glink(r.graha), ab(r.star_lord), ab(r.sub_lord), ab(r.sub_sub_lord),
      ...['very_strong', 'strong', 'normal', 'weak'].map(k => hov(h('span', { class: 'kp-lvl kp-' + k }, (r[k] || []).join(', ') || '—'), `${(G[r.graha] || {}).en || r.graha}: ${k.replace('_', ' ')}`, [['Houses', (r[k] || []).join(', ') || 'none'], ['Rule', t.levels[k]]]))]);
    const eq = (t.equal_cusps_from_ascendant || []).map(r => [nth(r.house), `${SIGN_AB[signOf(r.cusp)]} ${inSignDM(r.cusp)}`, ab(r.star_lord), ab(r.sub_lord), ab(r.sub_sub_lord)]);
    return [
      card('KP significators by bhāva', table(['Bhāva', 'Cusp', 'Star', 'Sub', 'Sub-sub', 'In stars of occupants', 'Occupants', 'In star of owner', 'Owner'], houses),
        note(label('tradition_rule'), ' Strongest first: grahas in the stars of the occupants, the occupants, grahas in the star of the owner, the owner. Hover a bhāva for its full list.')),
      card('KP grahas: what each signifies', table(['Graha', 'Star', 'Sub', 'Sub-sub', 'Very strong', 'Strong', 'Normal', 'Weak'], grahas),
        note(label('tradition_rule'), ` Very strong: ${t.levels.very_strong}; strong: ${t.levels.strong}; normal: ${t.levels.normal}; weak: ${t.levels.weak}. ${t.note}.`)),
      eq.length ? h('details', { class: 'card kp-equal' }, h('summary', {}, 'Equal cusps from the Ascendant (an alternative some programs print)'), table(['Bhāva', 'Cusp', 'Star', 'Sub', 'Sub-sub'], eq)) : null,
    ];
  }
}, { blurb: 'sub-lords and significators' });

/* ── KP horary (a kp_horary envelope: the number fixes the Ascendant; the moment and place are those of judgement) ── */
page('horary', () => 'KP horary', 'Systems', (b) => {
  const k = ENV.kp_horary; if (!k) return put(b, missing('KP horary', 'Give a horary number (1–249) when the report is made: eshana-report --kp-number N [--kp-at UTC].'));
  const n = k.number, kp = k.kp;
  const cusps = Object.entries(kp.cusps).map(([hh, c]) => [hh, `${SIGN_SKT[signOf(c.cusp)]} ${inSign(c.cusp)}`, c.lords.sign_lord, c.lords.star_lord, c.lords.sub_lord, c.lords.sub_sub_lord, c.B_occupants.join(', ') || '—']);
  const pl = Object.entries(kp.planets).sort((a, bb) => GRAHA_ORDER.indexOf(a[0]) - GRAHA_ORDER.indexOf(bb[0])).map(([g, l]) => [g, l.star_lord, l.sub_lord, l.sub_sub_lord, kp.significators_planet_view[g].B_house_occupied]);
  put(b, pageHead('kp', 'KP horary', k.calculation.rule),
    card('Number and moment', facts([['Number', `${n.number} · ${n.sign} ${n.start_in_sign}–${n.end_in_sign}`], ['Star · sub', `${n.star_lord} · ${n.sub_lord}`],
      ['Judgement (UTC)', k.input.judgement_utc], ['Place', `${fix(k.input.place.latitude_deg, 4)}°, ${fix(k.input.place.longitude_deg, 4)}°`],
      ['Ayanāṃśa', k.calculation.ayanamsha], ['Cusp occupancy', `${k.calculation.method.id} (${kp.occupancy})`]])),
    card('Horary cusps', table(['Cusp', 'Position', 'Sign', 'Star', 'Sub', 'Sub-sub', 'Occupants'], cusps)),
    card('Planets at judgement', table(['Graha', 'Star', 'Sub', 'Sub-sub', 'House'], pl)),
    kp.ruling_planets_at_judgement ? card('Ruling planets at judgement', facts(Object.entries(kp.ruling_planets_at_judgement).map(([kk, v]) => [kk.replaceAll('_', ' '), typeof v === 'object' ? JSON.stringify(v) : v]))) : null,
    (k.notes || []).length ? note(k.notes.join(' ')) : null);
}, { blurb: 'number, cusps and sub-lords at judgement' });

/* ── Personal calendar (a personal_calendar envelope): a day grid of real buttons; arrows move, Enter or Space opens the day ── */
let CAL_DAY = 0;
page('calendar', () => 'Calendar', 'Time', (b) => {
  const c = ENV.personal_calendar; if (!c) return put(b, missing('The personal calendar', 'Ask for it when the report is made: eshana-report --calendar-days N [--calendar-profile ID] [--event-lat φ --event-lon λ --event-tz ZONE].'));
  const days = c.days || [], NA = v => v && v.status === 'unavailable';
  const fmtT = t => (t || '').slice(11, 16), dayLabel = d => d.date.slice(5);
  const detail = h('div', { class: 'card cal-detail', role: 'region', 'aria-live': 'polite', 'aria-label': 'Day detail' });
  const show = (i) => {
    CAL_DAY = i; const d = days[i]; if (!d) return;
    const bind = Object.entries(d.transit_bindus || {}).map(([g, v]) => [g, NA(v) ? `unavailable: ${v.reason}` : `${SIGN_SKT[v.transit_sign]} · BAV ${v.bav} · SAV ${v.sav}`]);
    put(detail, h('h3', {}, `${d.date}`),
      facts([['Sunrise', NA(d.sunrise_local) ? `unavailable: ${d.sunrise_local.reason}` : fmtT(d.sunrise_local)], ...bind]),
      d.windows.length ? table(['From', 'To', 'Minutes'], d.windows.map(w => [fmtT(w.start_local), fmtT(w.end_local), Math.round(w.duration_minutes)]))
        : note('No window passes on this day.'));
    grid.querySelectorAll('button').forEach((x, k) => { x.setAttribute('aria-pressed', k === i ? 'true' : 'false'); x.tabIndex = k === i ? 0 : -1; });
  };
  const grid = h('div', { class: 'cal-grid', role: 'group', 'aria-label': 'Days; use the arrow keys, Enter opens a day' },
    days.map((d, i) => h('button', { type: 'button', class: 'cal-day' + (d.windows.length ? ' has' : ''), 'data-i': i,
      'aria-label': `${d.date}: ${d.windows.length} window${d.windows.length === 1 ? '' : 's'}`, onclick: () => show(i),
      onkeydown: (ev) => {
        const step = { ArrowRight: 1, ArrowLeft: -1, ArrowDown: 7, ArrowUp: -7 }[ev.key];
        if (step) { ev.preventDefault(); const j = Math.max(0, Math.min(days.length - 1, i + step)); show(j); grid.children[j].focus(); }
        else if (ev.key === 'Enter' || ev.key === ' ') { ev.preventDefault(); show(i); detail.focus && detail.setAttribute('tabindex', '-1'); }
      } }, h('b', {}, dayLabel(d)), h('span', {}, d.windows.length ? `${d.windows.length}` : '·'))));
  const prof = c.profile || {};
  put(b, pageHead('panchanga', 'Personal calendar', `${prof.title || 'constraints'}${prof.status === 'experimental' ? ' (experimental)' : ''}. Event place ${fix(c.input.event_place.latitude_deg, 2)}°, ${fix(c.input.event_place.longitude_deg, 2)}°; birth place kept apart.`),
    card('Days', grid, note('Numbers are passing windows starting that day. Times are local to the event place.')), detail,
    (prof.missing || []).length ? note('Not covered by this profile: ' + prof.missing.join('; ') + '.') : null);
  show(Math.min(CAL_DAY, Math.max(0, days.length - 1)));
}, { blurb: 'windows by day, with tārā and candra bala' });

/* ── Nadi ─────────────────────────────────────────────────────────────────────────────────────────────────────── */
let NADI_SEL = null;
page('nadi', () => 'Nadi', 'Systems', (b) => {
  if (!VED) return put(b, missing('Nadi', 'It needs a Vedic chart.'));
  const host = h('div', { class: 'figure narrow' }), side = h('div', {});
  const draw = () => {
    put(host, nadiMatrix(NADI_SEL, g => { NADI_SEL = g === NADI_SEL ? null : g; draw(); }));
    const groups = ['Fire', 'Earth', 'Air', 'Water'].map(el => { const q = NADI_Q[el], ids = NINE.filter(g => GR[g] && ELEMENT[signOf(GR[g].lon) % 4] === el);
      return [h('span', {}, h('i', { class: 'sw', style: `background:${q.c}` }), `${q.dir} · ${el}`), q.trikona, q.houses, h('span', { class: 'chips' }, ids.map(g => h('a', { class: 'chip', href: '#', onclick: e => { e.preventDefault(); NADI_SEL = g; draw(); } }, dot(g), `${G[g].abbr} ${inSignDM(GR[g].lon)}`)))]; });
    const links = NADI_SEL ? nadiLinks(NADI_SEL) : [];
    put(side, card('Directions', table(['Direction', 'Trikoṇa', 'Houses', 'Grahas (degree)'], groups)),
      card(NADI_SEL ? `${G[NADI_SEL].en}: links` : 'Pick a graha', NADI_SEL ? (links.length ? h('ul', { class: 'list' }, links.map(l => h('li', {}, glink(l.to), ` — ${l.kind}${l.via ? ' (' + l.via + ')' : ''}`))) : note('no links')) : note('Click a graha in the matrix to see the grahas it connects with.'),
        note('report.nadi@0.1: grahas in trine (same element, same direction), in the next and previous signs, and opposite are linked; a retrograde graha also acts from the previous sign. Shown for reading; ESHANA does not interpret the links.')));
  };
  draw();
  put(b, pageHead('nadi', 'Nadi direction matrix', 'The zodiac as a spiral through the four directions in three rounds; each graha sits in its sign with its degree.'), h('div', { class: 'grid2' }, host, side));
}, { blurb: 'directions, trikoṇas and links' });

/* ── energetics (the author's framework, as reference) ────────────────────────────────────────────────────────── */
page('energetics', () => 'Energetics', 'Systems', (b) => {
  if (!AE || !VED) return put(b, missing('Energetics', 'It needs the reference data and a Vedic chart.'));
  const band = AE.energy_bands, bands = Object.entries(AE.graha_band);
  const planets = table(['Graha', 'Energy', 'Means', 'Temperature', 'Bhāva', 'Body (bhāva reference)'], bands.filter(([g]) => GR[g]).map(([g, bd]) => { const hh = houseOf(signOf(GR[g].lon)); return [glink(g), h('span', { class: 'chip', style: `border-color:${band[bd].c}` }, bd), band[bd].t, AE.temperature_k[g] ? AE.temperature_k[g] + ' K' : '—', hh, hh ? AE.houses[hh].zodiac_body || AE.houses[hh].body : '—']; }));
  const quad = Object.entries(AE.quadrants).map(([k, q]) => { const occ = NINE.filter(g => GR[g] && houseOf(signOf(GR[g].lon)) && q.houses.includes(houseOf(signOf(GR[g].lon))));
    return h('div', { class: 'card quad', style: `--q:${q.colour}` }, h('h3', {}, q.name), h('div', { class: 'note' }, `bhāvas ${q.houses.join(', ')} · ${q.lean}`), h('div', { class: 'chips' }, occ.map(g => h('span', { class: 'chip' }, dot(g), G[g].abbr))), h('b', { class: 'qn' }, occ.length)); });
  const tri = Object.entries(AE.trikonas).map(([k, t]) => { const occ = NINE.filter(g => GR[g] && t.houses.includes(houseOf(signOf(GR[g].lon)))); return [t.name, t.houses.join(', '), occ.map(g => G[g].abbr).join(' ') || '—', occ.length]; });
  const houses = Array.from({ length: 12 }, (_, i) => { const n = i + 1, a = AE.houses[n], sign = (LAGNA_SIGN + i) % 12, occ = GRAHA_ORDER.filter(g => GR[g] && signOf(GR[g].lon) === sign);
    return [h('a', { href: '#/house/' + n }, n), a.dimension, a.zodiac_body || '—', occ.map(g => G[g].abbr).join(' ') || '—']; });
  put(b, pageHead('energetics', 'AstroEnergetics', 'The author’s framework: houses as dimensions of life and regions of the body, grahas as givers, sustainers or restrainers of energy. Reference for reading, not computed.'),
    card('Grahas: energy bands', planets), h('div', { class: 'qgrid' }, quad), h('div', { class: 'grid2' }, card('Trikoṇas', table(['Trikoṇa', 'Bhāvas', 'Grahas', 'n'], tri)), card('Bhāvas: dimension and body', table(['Bhāva', 'Dimension', 'Body', 'Grahas'], houses))));
}, { blurb: 'dimensions, body regions, energy bands' });

/* ── transits, progressions ───────────────────────────────────────────────────────────────────────────────────── */
page('transits', () => 'Transits', 'Time', (b) => {
  const tr = D.transits; if (!tr || !tr.rows || !tr.rows.length) return put(b, missing('Transits', 'The file was made without a transit span.'));
  const ref = tr.rows[tr.reference_index || 0], moonSign = GR.moon ? signOf(GR.moon.lon) : null;
  const rows = Object.entries(ref.longitudes).sort((a, bb) => GRAHA_ORDER.indexOf(engineId(a[0])) - GRAHA_ORDER.indexOf(engineId(bb[0]))).map(([k, l]) => { const id = engineId(k);
    return [glink(id), `${SIGN_SKT[signOf(l)]} ${inSign(l)}`, `${NAKSHATRA[nakOf(l)]} ${padaOf(l)}`, houseOf(signOf(l)), moonSign == null ? '—' : (signOf(l) - moonSign + 12) % 12 + 1, GR[id] ? `${SIGN_SKT[signOf(GR[id].lon)]} ${inSignDM(GR[id].lon)}` : '—']; });
  put(b, pageHead('transits', 'Transits', `Positions at ${fmtTime(ref.utc)} against the natal chart. Turn on the transits layer on the chakra and press play to move through ${dateOnly(tr.rows[0].utc)} – ${dateOnly(tr.rows[tr.rows.length - 1].utc)}.`),
    card('At the reference instant', table(['Graha', 'Transit', 'Nakshatra', 'Bhāva from lagna', 'From natal Moon', 'Natal'], rows)), h('a', { class: 'btn primary', href: '#/chakra', onclick: () => { S.layers.transits = true; save(); } }, 'Animate on the chakra →'));
}, { blurb: 'today against the birth chart' });

/* ── progression by completed revolutions (report.revolutions@0.1) ───────────────────────────────────────────────
   each completed sidereal revolution after birth moves a graha one sign on, in its direction of motion */
const REVS = D.revolutions || null;
const REV_IDS = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn', 'rahu', 'ketu'];
const REV_ASPECTS = { mars: [4, 7, 8], jupiter: [5, 7, 9], saturn: [3, 7, 10], rahu: [5, 7, 9], ketu: [5, 7, 9] };
let REV_T = null, REV_VIEW = 'square', REV_PLAY = null, REV_STEP = 30;
function revKey(id) { if (!REVS) return null; if (id === 'rahu' || id === 'ketu') return Object.keys(REVS.bodies).find(k => /node/.test(k) && !/south/.test(k)) || null; return REVS.bodies[id] ? id : null; }
function revState(id, t) {
  const k = revKey(id); if (!k || !GR[id]) return null;
  const b = REVS.bodies[k]; let n = 0; while (n < b.returns.length && T(b.returns[n]) <= t) n++;
  const natal = signOf(GR[id].lon), sign = ((natal + b.direction * n) % 12 + 12) % 12;
  return { n, sign, lon: sign * 30 + norm(GR[id].lon) % 30, prev: n ? b.returns[n - 1] : null, next: b.returns[n] || null, dir: b.direction };
}
const revAspects = (id, sign) => (REV_ASPECTS[id] || [7]).map(hh => (sign + hh - 1) % 12);
function revSquare(t) {
  const by = {}, asp = {};
  for (const id of REV_IDS) { const st = revState(id, t); if (!st) continue; (by[st.sign] = by[st.sign] || []).push([id, st]); for (const x of revAspects(id, st.sign)) (asp[x] = asp[x] || []).push(id); }
  const outer = sign => (by[sign] || []).map(([id, st]) => ({ text: 'P' + G[id].abbr, colour: G[id].dot === '#E6E9F0' ? 'var(--ink)' : G[id].dot, strong: st.n > 0,
    title: `Progressed ${G[id].en}: ${SIGN_SKT[st.sign]} after ${st.n} revolution(s)` }))
    .concat((asp[sign] || []).length ? [{ text: '⟶' + asp[sign].map(id => G[id].abbr).join(''), colour: 'var(--muted)', title: 'Signs the progressed grahas aspect' }] : []);
  return concentricSouth(natalItems, outer, { title: 'Inner: birth chart', sub: `Outer: progressed · ${dateOnly(new Date(t).toISOString())}`, label: 'Birth chart inside, progressed positions outside' });
}
function revCircle(t, focus) {
  const svg = s('svg', { viewBox: '-310 -310 620 620', role: 'img', 'aria-label': 'Natal and progressed positions as concentric circles', class: 'chalit-circle revcircle' });
  const P = (lon, r) => { const a = Math.PI + norm(lon - (ASC != null ? ASC : 0)) * DEG; return [r * Math.cos(a), -r * Math.sin(a)]; };
  const sector = (l0, l1, r0, r1) => { const [a, b] = P(l0, r1), [c, d] = P(l1, r1), [e, f] = P(l1, r0), [g, k] = P(l0, r0); return `M${a.toFixed(1)} ${b.toFixed(1)}A${r1} ${r1} 0 0 0 ${c.toFixed(1)} ${d.toFixed(1)}L${e.toFixed(1)} ${f.toFixed(1)}A${r0} ${r0} 0 0 1 ${g.toFixed(1)} ${k.toFixed(1)}Z`; };
  for (let i = 0; i < 12; i++) { s('path', { d: sector(i * 30, i * 30 + 30, 232, 268), fill: G[SIGN_LORD[i]].band, stroke: 'var(--line)' }, svg); const [x, y] = P(i * 30 + 15, 250); const tx = s('text', { x, y, class: 'lbl', 'font-size': 16, fill: '#1b1b1b' }, svg); tx.textContent = GLYPH[i]; }
  s('circle', { r: 160, fill: 'none', stroke: 'var(--line)', 'stroke-dasharray': '3 5' }, svg);
  for (const id of REV_IDS) {
    if (!GR[id]) continue; const st = revState(id, t);
    const [nx, ny] = P(GR[id].lon, 288); marker(svg, id, nx, ny, 11, { passive: true });
    if (!st) continue;
    const [px, py] = P(st.lon, 196); const mk = marker(svg, id, px, py, 13, { passive: true });
    if (st.n) { const [ax, ay] = P(GR[id].lon, 230), [bx, by] = P(st.lon, 210); s('path', { d: `M${ax.toFixed(1)} ${ay.toFixed(1)}Q0 0 ${bx.toFixed(1)} ${by.toFixed(1)}`, class: 'rev-move', stroke: G[id].dot }, svg); }
    if (!focus || focus === id) for (const a of revAspects(id, st.sign)) { const [tx, ty] = P(a * 30 + 15, 226); s('line', { x1: px, y1: py, x2: tx, y2: ty, class: 'rev-asp', stroke: G[id].dot }, svg); }
    mk.addEventListener('mousemove', ev => tip(ev, `Progressed ${G[id].en}`, [['Sign', SIGN_SKT[st.sign]], ['Revolutions', st.n], ['Aspects', revAspects(id, st.sign).map(x => SIGN_AB[x]).join(' ')], ['Next move', st.next ? dateOnly(st.next) : '—']]));
    mk.addEventListener('mouseleave', untip);
  }
  const t1 = s('text', { x: 0, y: -4, 'text-anchor': 'middle', 'font-size': 13, 'font-weight': 800, fill: 'var(--ink)' }, svg); t1.textContent = 'outer: natal · inner: progressed';
  const t2 = s('text', { x: 0, y: 14, 'text-anchor': 'middle', 'font-size': 11, fill: 'var(--muted)' }, svg); t2.textContent = dateOnly(new Date(t).toISOString());
  return svg;
}
function revTimeline(t, onPick) {
  const t0 = T(REVS.birth_utc), t1 = t0 + REVS.years * 365.25 * 864e5, W = 1000, lane = 24, top = 18, H = top + REV_IDS.length * lane + 22;
  const X = v => 90 + (v - t0) / (t1 - t0) * (W - 100);
  const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'revtl', role: 'img', 'aria-label': 'Progressed sign of each graha across life' });
  REV_IDS.forEach((id, i) => {
    const k = revKey(id); if (!k || !GR[id]) return; const b = REVS.bodies[k], y = top + i * lane;
    const lab = s('text', { x: 82, y: y + 15, class: 'rc-lab' }, svg); lab.textContent = G[id].en;
    const cuts = [t0, ...b.returns.map(T).filter(v => v < t1), t1];
    for (let j = 0; j < cuts.length - 1; j++) {
      const sign = ((signOf(GR[id].lon) + b.direction * j) % 12 + 12) % 12, w = X(cuts[j + 1]) - X(cuts[j]);
      const r = s('rect', { x: X(cuts[j]), y: y + 3, width: Math.max(.6, w), height: lane - 6, fill: PADA_FILL[ELEMENT[sign % 4]], class: 'rev-seg' }, svg);
      const tt = s('title', {}, r); tt.textContent = `${G[id].en} progressed in ${SIGN_SKT[sign]} from ${dateOnly(new Date(cuts[j]).toISOString())}`;
      if (w > 26) { const tx = s('text', { x: X(cuts[j]) + w / 2, y: y + 16, class: 'rev-sl' }, svg); tx.textContent = SIGN_AB[sign]; }
    }
  });
  for (let a = 0; a <= REVS.years; a += 10) { const x = X(t0 + a * 365.25 * 864e5); const tx = s('text', { x, y: H - 6, class: 'rc-tick' }, svg); tx.textContent = `age ${a}`; s('line', { x1: x, x2: x, y1: top, y2: H - 18, class: 'rc-grid' }, svg); }
  s('line', { x1: X(t), x2: X(t), y1: top - 6, y2: H - 16, class: 'rev-now' }, svg);
  svg.addEventListener('click', ev => { const r = svg.getBoundingClientRect(), vx = (ev.clientX - r.left) / r.width * W; onPick(t0 + Math.max(0, Math.min(1, (vx - 90) / (W - 100))) * (t1 - t0)); });
  return svg;
}
function revolutionsSection() {
  if (!REVS) return null;
  const t0 = T(REVS.birth_utc), t1 = t0 + REVS.years * 365.25 * 864e5, ref = T(REF_UTC || new Date().toISOString());
  if (REV_T == null) REV_T = Math.min(t1, Math.max(t0, ref));
  const fig = h('div', { class: 'figure' }), tl = h('div', { class: 'timeline' }), tbl = h('div', {}), dateIn = h('input', { type: 'date', min: dateOnly(REVS.birth_utc), max: dateOnly(new Date(t1).toISOString()) });
  const draw = () => {
    put(fig, REV_VIEW === 'circle' ? revCircle(REV_T) : revSquare(REV_T));
    put(tl, revTimeline(REV_T, v => { REV_T = v; draw(); }));
    dateIn.value = dateOnly(new Date(REV_T).toISOString());
    put(tbl, table(['Graha', 'Natal', 'Revolutions', 'Progressed sign', 'Bhāva (natal lagna)', 'Moved on', 'Moves next'], REV_IDS.filter(id => GR[id]).map(id => { const st = revState(id, REV_T); return st ? [glink(id), SIGN_SKT[signOf(GR[id].lon)], st.n, h('b', {}, SIGN_SKT[st.sign]), houseOf(st.sign) || '—', st.prev ? dateOnly(st.prev) : 'birth', st.next ? dateOnly(st.next) : '—'] : [glink(id), SIGN_SKT[signOf(GR[id].lon)], '—', '—', '—', '—', '—']; })));
  };
  dateIn.addEventListener('change', () => { const v = T(dateIn.value + 'T00:00:00Z'); if (!isNaN(v)) { REV_T = Math.min(t1, Math.max(t0, v)); draw(); } });
  const play = h('button', { class: 'btn', id: 'rev-play', onclick: () => {
    if (REV_PLAY) { cancelAnimationFrame(REV_PLAY); REV_PLAY = null; play.textContent = '▶ Play'; return; }
    play.textContent = '❚❚ Pause'; let last = 0;
    const loop = ts => { if (ts - last > 120) { last = ts; REV_T += REV_STEP * 864e5; if (REV_T > t1) REV_T = t0; draw(); } REV_PLAY = requestAnimationFrame(loop); };
    REV_PLAY = requestAnimationFrame(loop);
  } }, '▶ Play');
  const speed = h('select', { onchange: e => { REV_STEP = +e.target.value; } }, [[7, 'a week'], [30, 'a month'], [91, 'a quarter'], [365, 'a year']].map(([v, t]) => h('option', { value: v, selected: REV_STEP === v }, `${t} per tick`)));
  draw();
  return h('div', {}, h('h3', { class: 'section-t' }, 'Progression by completed revolutions'),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'As of'), dateIn, h('button', { class: 'tab', onclick: () => { REV_T = Math.min(t1, Math.max(t0, ref)); draw(); } }, 'Reference date'), play, speed),
      h('div', { class: 'grp' }, h('b', {}, 'Show'), ...[['square', 'Concentric squares'], ['circle', 'Circle']].map(([v, t]) => h('button', { class: 'tab', 'aria-selected': String(REV_VIEW === v), onclick: e => { REV_VIEW = v; $$('.tab', e.target.parentNode).forEach(x => x.setAttribute('aria-selected', String(x === e.target))); draw(); } }, t)))),
    h('div', { class: 'grid2' }, card('Natal and progressed', fig, note('Inner ring (P…): progressed positions; ⟶ marks the signs they aspect (7th; Mars 4th and 8th, Jupiter 5th and 9th, Saturn 3rd and 10th, nodes 5th and 9th).')), card('Where each graha has moved', tbl)),
    card('Across life', tl, note('Each lane shows the sign a graha is progressed into, from birth; it moves on at every completed revolution. Click the timeline to jump to that date.')),
    note(`${REVS.rule}: ${REVS.convention}. Revolutions are measured against the sidereal zodiac from the birth position, from an ephemeris every ${REVS.step_days} days.`));
}

page('progressions', () => 'Progressions', 'Time', (b) => {
  const pr = D.progressions; if (!pr || !pr.rows || !pr.rows.length) return put(b, missing('Progressions', 'They need an exact birth time.'), revolutionsSection());
  const ids = Object.keys(pr.rows[0].longitudes).filter(k => ['sun', 'moon', 'mercury', 'venus', 'mars'].includes(k));
  const W = 1000, H = 220, svg = s('svg', { viewBox: `0 0 ${W} ${H + 24}`, role: 'img', 'aria-label': 'Secondary progressions by age' });
  for (let sg = 0; sg <= 12; sg++) s('line', { x1: 0, x2: W, y1: H - sg / 12 * H, y2: H - sg / 12 * H, stroke: 'var(--line)' }, svg);
  for (const id of ids) { let d = '', prev = null; pr.rows.forEach((r, i) => { const x = i / (pr.rows.length - 1) * W, l = r.longitudes[id], y = H - l / 360 * H; d += (prev != null && Math.abs(l - prev) > 180 ? 'M' : (i ? 'L' : 'M')) + `${x.toFixed(1)} ${y.toFixed(1)}`; prev = l; }); s('path', { d, fill: 'none', stroke: G[id].dot, 'stroke-width': 2.2, class: 'flowline' }, svg); }
  for (let a = 0; a < pr.rows.length; a += 10) { const t = s('text', { x: a / (pr.rows.length - 1) * W, y: H + 16, 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `age ${a}`; }
  const every = pr.rows.filter((_, i) => i % 5 === 0);
  put(b, pageHead('progressions', 'Progressions', 'Two ways to progress the chart: by completed revolutions (each return moves a graha one sign on) and secondary progressions (a day for a year).'), revolutionsSection(), h('h3', { class: 'section-t' }, 'Secondary progressions (a day for a year, tropical)'), card('', h('div', { class: 'timeline' }, svg), h('div', { class: 'legend' }, ids.map(id => h('span', {}, h('i', { style: `background:${G[id].dot}` }), G[id].en)))),
    card('Every 5 years', table(['Age', ...ids.map(id => G[id].en)], every.map((r, k) => [k * 5, ...ids.map(id => `${SIGN_EN[signOf(r.longitudes[id])].slice(0, 3)} ${inSignDM(r.longitudes[id])}`)]))));
}, { blurb: 'a day for a year' });

/* ── other traditions ─────────────────────────────────────────────────────────────────────────────────────────── */
page('western', () => 'Western', 'Traditions', (b) => {
  if (!WES) return put(b, missing('The Western chart', 'It was not requested.'));
  const wgr = grahasOf(WES), asp = WES.derived_attributes.filter(a => a.id.includes('.aspect.')), bodies = GRAHA_ORDER.filter(g => wgr[g] && !['rahu', 'ketu'].includes(g));
  const grid = table(['', ...bodies.map(g => G[g].glyph)], bodies.map((g1, i) => [glink(g1), ...bodies.map((g2, j) => { if (j >= i) return ''; const a = asp.find(x => { const ids = x.derived_from.map(engineId); return ids.includes(g1) && ids.includes(g2); });
    return a ? { td: { style: `color:${ASPECT_COLOUR[a.value.aspect]};font-weight:700;text-align:center` }, v: { conjunction: '☌', opposition: '☍', square: '□', trine: '△', sextile: '⚹' }[a.value.aspect] + ' ' + a.value.orb_deg.toFixed(1) } : ''; })]));
  put(b, pageHead('western', 'Western chart (tropical)', `Houses ${WES.calculation.house_system}; Ptolemaic aspects.`), h('div', { class: 'grid2' }, h('div', { class: 'figure' }, westernWheel()), card('Aspect grid', grid)),
    card('Positions', table(['Body', 'Longitude', 'Sign', 'In sign', 'House', 'R', 'Latitude'], GRAHA_ORDER.filter(g => wgr[g]).map(g => [glink(g), fix(wgr[g].lon, 4) + '°', SIGN_EN[signOf(wgr[g].lon)], inSign(wgr[g].lon),
      (WA[`western.${g === 'rahu' ? 'mean_node' : g === 'ketu' ? 'south_node' : g}.house`] || { value: {} }).value.house || '—', wgr[g].retro ? 'R' : '', fix(wgr[g].lat, 4)]))));
}, { blurb: 'tropical wheel and aspect grid' });

page('human-design', () => 'Human Design', 'Traditions', (b) => {
  if (!HD) return put(b, missing('Human Design', 'It needs an exact birth time.'));
  const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value || {}, pers = HD.positions.personality.activations, des = HD.positions.design.activations;
  const byBody = {}; for (const a of pers) byBody[a.body] = { p: a }; for (const a of des) (byBody[a.body] = byBody[a.body] || {}).d = a;
  put(b, pageHead('human_design', 'Human Design', 'Design instant by the 88° solar arc.'), h('div', { class: 'stats' }, ['type', 'authority', 'profile', 'definition'].map(k => stat(k, bg[k] || '—'))),
    tabs([['standard', 'Standard chart'], ['detailed', 'Detailed BodyGraph']], S.hd === 'detailed' ? 'detailed' : 'standard', v => { S.hd = v; save(); route(); }),
    S.hd !== 'detailed' ? card(null, hdChart()) : null,
    h('div', { class: 'grid2' }, S.hd !== 'detailed' ? null : h('div', { class: 'figure narrow' }, bodygraph()), h('div', {}, card('', facts([['Personality (UTC)', HD.positions.personality.instant_utc], ['Design (UTC)', HD.positions.design.instant_utc], ['Channels', (bg.channels || []).join(', ')], ['Defined', (bg.defined_centers || []).join(', ')], ['Undefined', (bg.undefined_centers || []).join(', ')]])),
      card('Activations', table(['Body', 'Personality', 'Design'], Object.entries(byBody).map(([k, v]) => [k.replace('_', ' '), v.p ? `${v.p.gate}.${v.p.line}` : '', v.d ? h('b', { class: 'red' }, `${v.d.gate}.${v.d.line}`) : '']))), note('Black: personality · red: design · striped: both.'))));
}, { blurb: 'type, authority and BodyGraph', section: 'human-design' });

/* ── numerology: every school, every number, each one explained ─────────────────────────────────────────────── */
let NUM_SCHOOL = 'all', NUM_OPEN = null, LOSHU_DC = false;
page('numerology', () => 'Numerology', 'Traditions', (b) => {
  if (!NUM) return put(b, missing('Numerology', 'It needs a name or a date.'));
  const A = attrsOf(NUM), P = (A['numerology.pythagorean.extended'] || {}).value, C = (A['numerology.chaldean.extended'] || {}).value,
    V = (A['numerology.vedic'] || {}).value, L = (A['numerology.lo_shu'] || {}).value, NR = (READ.numerology || {});
  const meaning = n => (NR.numbers || {})[String(n)] || null;
  const dobStr = (NUM.input || {}).date_of_birth, refStr = (REF_UTC || new Date().toISOString()).slice(0, 10);
  const age = dobStr ? Math.floor((T(refStr) - T(dobStr)) / (365.25 * 864e5)) : null;
  // one number tile: value, role, karmic debt; click to open what it means
  const tile = (id, title, v, opts = {}) => {
    if (v == null) return null;
    const n = typeof v === 'object' ? (v.reduced != null ? v.reduced : v.number) : v, comp = typeof v === 'object' ? v.compound : null, debt = typeof v === 'object' ? v.karmic_debt : null;
    const m = meaning(n), open = NUM_OPEN === id;
    return h('button', { class: 'ntile' + (open ? ' open' : '') + (opts.small ? ' small' : ''), 'aria-expanded': String(open), onclick: () => { NUM_OPEN = open ? null : id; route(); } },
      h('span', { class: 'n-num' }, String(n)), h('span', { class: 'n-role' }, title),
      comp != null && comp !== n ? h('span', { class: 'n-comp' }, `${comp}/${n}`) : null,
      debt ? h('span', { class: 'badge b-down' }, `karmic debt ${debt}`) : null,
      opts.planet ? h('span', { class: 'badge b-soft' }, opts.planet) : null,
      open ? h('div', { class: 'n-detail' },
        h('p', {}, (NR.roles || {})[opts.role || id] || ''),
        m ? h('p', {}, h('b', {}, `${n}: `), m.key) : null,
        comp != null && (NR.compound || {})[String(comp)] ? h('p', { class: 'n-compound' }, h('b', {}, `Compound ${comp}: `), NR.compound[String(comp)], h('span', { class: 'note' }, ' (Chaldean compound reading, after Cheiro)')) : null,
        m ? h('p', { class: 'note' }, `Strengths: ${m.strengths.join(', ')} · Challenges: ${m.challenges.join(', ')} · Planet: ${m.planet}`) : null,
        debt ? h('p', { class: 'note' }, (NR.karmic_debt || {})[String(debt)] || '') : null) : null);
  };
  const show = sch => NUM_SCHOOL === 'all' || NUM_SCHOOL === sch;
  const out = [pageHead('numerology', 'Numerology', 'Every number in each school, computed by stated rules. Click a number for what it is said to describe.'),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'School'), tabs([['all', 'All'], ['pythagorean', 'Pythagorean'], ['chaldean', 'Chaldean'], ['vedic', 'Indian (Vedic)'], ['loshu', 'Lo Shu']], NUM_SCHOOL, k => { NUM_SCHOOL = k; route(); })))];
  if (P && show('pythagorean')) {
    const core = [['life_path', 'Life path'], ['expression', 'Expression'], ['soul_urge', 'Soul urge'], ['personality', 'Personality'], ['birthday', 'Birthday'], ['maturity', 'Maturity'], ['attitude', 'Attitude'], ['balance', 'Balance'], ['rational_thought', 'Rational thought']];
    out.push(h('h3', { class: 'section-t' }, 'Pythagorean · core numbers'), h('div', { class: 'ntiles' }, core.map(([k, t]) => tile(k, t, P[k])).filter(Boolean),
      P.subconscious_self != null ? tile('subconscious_self', 'Subconscious self', P.subconscious_self) : null,
      (P.hidden_passion || []).length ? tile('hidden_passion', 'Hidden passion', P.hidden_passion[0]) : null,
      P.cornerstone ? tile('cornerstone', `Cornerstone (${P.cornerstone.letter})`, P.cornerstone.value) : null,
      P.capstone ? tile('capstone', `Capstone (${P.capstone.letter})`, P.capstone.value) : null,
      P.first_vowel ? tile('first_vowel', `First vowel (${P.first_vowel.letter})`, P.first_vowel.value) : null,
      P.bridge_life_path_expression != null ? tile('bridge_life_path_expression', 'Bridge: path ↔ expression', P.bridge_life_path_expression) : null,
      P.bridge_soul_personality != null ? tile('bridge_soul_personality', 'Bridge: soul ↔ personality', P.bridge_soul_personality) : null,
      P.growth ? tile('growth', 'Growth (first name)', P.growth) : null,
      P.intensity && P.intensity.numbers.length ? tile('intensity', `Intensity (${P.intensity.numbers.join(', ')})`, P.intensity.numbers[0]) : null));
    // letter transits and essence (name-based timing): now, and year by year
    if (P.transits_by_age) {
      const tr = P.transits_at_reference, L = x => x ? `${x.letter} (${x.value})` : '—';
      if (tr) out.push(h('h3', { class: 'section-t' }, `Letter transits now (age ${tr.age})`), h('div', { class: 'ntiles' },
        tr.physical ? tile('transit_physical', `Physical · ${tr.physical.letter}`, tr.physical.value, { small: true }) : null,
        tr.mental ? tile('transit_mental', `Mental · ${tr.mental.letter}`, tr.mental.value, { small: true }) : null,
        tr.spiritual ? tile('transit_spiritual', `Spiritual · ${tr.spiritual.letter}`, tr.spiritual.value, { small: true }) : null,
        tile('essence', 'Essence', tr.essence)));
      out.push(h('details', { class: 'card ntransits' }, h('summary', {}, 'Letter transits and essence, year by year (ages 0–99)'),
        table(['Age', 'Physical (first name)', 'Mental (middle names)', 'Spiritual (last name)', 'Essence'],
          P.transits_by_age.map(y => [tr && y.age === tr.age ? h('b', {}, `${y.age} · now`) : String(y.age), L(y.physical), L(y.mental), L(y.spiritual), h('b', {}, String(y.essence))])),
        note(P.conventions && P.conventions.transits ? P.conventions.transits : '')));
    }
    if (P.cycles_at_reference) { const c = P.cycles_at_reference;
      out.push(h('h3', { class: 'section-t' }, `Now · ${c.date}${age != null ? ` (age ${age})` : ''}`), h('div', { class: 'ntiles' }, tile('personal_year', 'Personal year', c.personal_year), tile('personal_month', 'Personal month', c.personal_month), tile('personal_day', 'Personal day', c.personal_day), tile('universal_year', 'Universal year', c.universal_year))); }
    if (P.pinnacles) {
      const W = 1000, H = 150, maxAge = 90, X = a => 104 + Math.min(a, maxAge) / maxAge * (W - 124);
      const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'revtl', role: 'img', 'aria-label': 'Pinnacles, challenges and life cycles by age' });
      [['Pinnacles', P.pinnacles, '#8f7bff'], ['Challenges', P.challenges, '#e0564f'], ['Life cycles', P.life_cycles, '#2bb3c0']].forEach(([lab, arr, col], r) => {
        const y = 14 + r * 40; const t = s('text', { x: 96, y: y + 20, class: 'rc-lab' }, svg); t.textContent = lab;
        arr.forEach((p, i) => { const x0 = X(p.from_age), x1 = X(Math.min(p.to_age, maxAge) + 1); const rr = s('rect', { x: x0, y, width: Math.max(2, x1 - x0 - 3), height: 30, rx: 8, fill: col, opacity: .25 + .15 * (i % 2), class: 'rev-seg' }, svg);
          const tt = s('title', {}, rr); tt.textContent = `${lab.slice(0, -1)} ${i + 1}: ${p.number}, ages ${p.from_age}–${p.to_age >= 120 ? 'on' : p.to_age}`;
          const n = s('text', { x: (x0 + x1) / 2, y: y + 20, class: 'rev-sl', 'font-size': 14 }, svg); n.textContent = p.number; });
      });
      for (let a = 0; a <= maxAge; a += 10) { const tx = s('text', { x: X(a), y: H - 4, class: 'rc-tick' }, svg); tx.textContent = a; }
      if (age != null) s('line', { x1: X(age), x2: X(age), y1: 6, y2: H - 16, class: 'rev-now' }, svg);
      out.push(card('Pinnacles, challenges and life cycles', h('div', { class: 'timeline' }, svg), note(P.conventions.pinnacles + '. ' + (NR.roles || {}).pinnacles + ' ' + (NR.roles || {}).challenges)));
    }
    if (P.inclusion) {
      const bars = arr => h('div', { class: 'incl' }, arr.map((v, i) => h('div', { class: 'incl-c', title: `${i + 1}: ${v}` }, h('i', { style: `height:${Math.min(100, v * 18)}%` }), h('b', {}, i + 1), h('small', {}, v))));
      out.push(h('div', { class: 'grid3' },
        card('Inclusion (how often each value appears)', bars(P.inclusion), note((NR.roles || {}).inclusion || '')),
        card('Karmic lessons', P.karmic_lessons.length ? h('ul', { class: 'list' }, P.karmic_lessons.map(k => h('li', {}, h('b', {}, k), ' — ', (NR.lessons || {})[String(k)] || ''))) : note('None: every value appears in the name.'),
          ['life_path', 'expression', 'soul_urge', 'personality', 'birthday'].filter(k => P[k] && P[k].karmic_debt).map(k => h('p', { class: 'note' }, h('b', {}, `Karmic debt ${P[k].karmic_debt} in the ${k.replace('_', ' ')}: `), (NR.karmic_debt || {})[String(P[k].karmic_debt)] || ''))),
        card('Planes of expression', facts(Object.entries(P.planes_of_expression || {}).map(([k, v]) => [k, h('span', {}, String(v), ' ', bar(v, Math.max(1, ...Object.values(P.planes_of_expression))))])), note((NR.roles || {}).planes_of_expression || ''))));
    }
  }
  if (C && show('chaldean')) out.push(h('h3', { class: 'section-t' }, 'Chaldean'), h('div', { class: 'ntiles' }, tile('ch_name', 'Name number', C.name, { role: 'chaldean_name' }), tile('ch_psychic', 'Psychic', C.psychic, { role: 'psychic' }), tile('ch_destiny', 'Destiny', C.destiny, { role: 'destiny' }),
    ...(C.words || []).map((w, i) => tile('ch_w' + i, w.word, w.number, { role: 'chaldean_name', small: true }))));
  if (V && show('vedic')) {
    const rel = x => x ? h('span', { class: 'badge ' + (x === 'friend' ? 'b-up' : x === 'enemy' ? 'b-down' : 'b-soft') }, x) : null;
    const nums = o => h('div', { class: 'badges' }, ...['friendly', 'neutral', 'enemy'].map(k => h('span', { class: 'badge ' + (k === 'friendly' ? 'b-up' : k === 'enemy' ? 'b-down' : 'b-soft') }, `${k}: ${o[k].join(' ') || '—'}`)));
    const pl = p => G[p] ? G[p].en : p;
    out.push(h('h3', { class: 'section-t' }, 'Indian (Vedic) numerology'), h('div', { class: 'ntiles' },
      V.mulank ? tile('mulank', 'Mulank (driver)', V.mulank.number, { planet: pl(V.mulank.planet) }) : null,
      V.bhagyank ? tile('bhagyank', 'Bhagyank (conductor)', V.bhagyank.number, { planet: pl(V.bhagyank.planet) }) : null,
      V.name_number ? tile('name_number', 'Name number', { compound: V.name_number.compound, reduced: V.name_number.number }, { planet: pl(V.name_number.planet) }) : null),
      h('div', { class: 'grid3' },
        V.mulank ? card(`Mulank ${V.mulank.number} · ${pl(V.mulank.planet)}`, nums(V.mulank.numbers), V.mulank_bhagyank ? h('p', {}, 'Mulank and bhagyank: ', rel(V.mulank_bhagyank)) : null) : null,
        V.bhagyank ? card(`Bhagyank ${V.bhagyank.number} · ${pl(V.bhagyank.planet)}`, nums(V.bhagyank.numbers)) : null,
        V.name_number ? card(`Name number ${V.name_number.number} · ${pl(V.name_number.planet)}`, facts([['With mulank', rel(V.name_number.with_mulank)], ['With bhagyank', rel(V.name_number.with_bhagyank)]])) : null),
      note(V.conventions.friendship + '. ' + V.conventions.planets + '.'));
  }
  if (L && show('loshu')) {
    const counts = LOSHU_DC ? L.counts_with_driver_conductor : L.counts, arrows = LOSHU_DC ? L.arrows_with_driver_conductor : L.arrows;
    const grid = h('div', { class: 'loshu' }, L.layout.flat().map((k, i) => { const c = counts[k - 1]; return h('div', { class: 'ls-c' + (c ? ' on' : ''), style: `--i:${i}`, title: `${k}: ${c ? c + ' time(s)' : 'missing'}` }, h('b', {}, c ? String(k).repeat(Math.min(c, 4)) : ''), h('small', {}, String(k))); }));
    out.push(h('h3', { class: 'section-t' }, 'Lo Shu grid'), h('div', { class: 'grid2' },
      card(`Birth date ${L.digits}`, h('div', { class: 'toolbar' }, h('label', { class: 'tog' }, h('input', { type: 'checkbox', checked: LOSHU_DC, onchange: e => { LOSHU_DC = e.target.checked; route(); } }), `add driver ${L.driver} and conductor ${L.conductor}`)), grid,
        facts([['Missing', L.missing.join(', ') || 'none'], ['Repeated', L.repeated.join(', ') || 'none']]), note(L.conventions)),
      card('Arrows (lines of the grid)', table(['Line', 'Numbers', 'State', 'Said to show'], arrows.map(a => [a.name, a.numbers.join('-'), h('span', { class: 'badge ' + (a.state === 'complete' ? 'b-up' : a.state === 'missing' ? 'b-down' : 'b-soft') }, a.state), (NR.arrows || {})[a.name] || ''])), note((NR.roles || {}).lo_shu || ''))));
    if (L.kua) { const K = L.kua, KR = NR.kua || {};
      out.push(card('Kua number (Eight Mansions)', K.number != null ? h('div', { class: 'stat kua-mine' }, h('b', {}, String(K.number)), h('span', {}, ` ${K.gender} · ${K.group} group`)) : null, facts([['Man', h('span', { class: K.gender === 'male' ? 'kua-sel' : '' }, h('b', {}, String(K.male)), ` · ${K.male_group} group`)], ['Woman', h('span', { class: K.gender === 'female' ? 'kua-sel' : '' }, h('b', {}, String(K.female)), ` · ${K.female_group} group`)], ['Solar year', String(K.solar_year)]]), K.number == null ? note('Give the gender (--gender male|female) to pick one.') : null,
        KR.about ? h('p', {}, label('interpretation'), ' ', KR.about) : null,
        KR.groups ? note(`East group: ${KR.groups.east} West group: ${KR.groups.west}`) : null, note(K.conventions))); }
  }
  const nameAttr = A['numerology.name'];
  if (nameAttr && (show('pythagorean') || show('chaldean'))) out.push(h('details', { class: 'card' }, h('summary', {}, `Letter by letter (${nameAttr.value.school})`), table(['Letter', 'Value', 'Vowel'], nameAttr.value.letters.map(x => [x.letter, x.value, x.vowel ? '✓' : '']))));
  out.push(h('p', { class: 'note' }, label('tradition_rule'), ' Numbers are computed by the named rules (numerology.extended@0.1, numerology.vedic@0.1, numerology.lo_shu@0.1). ', label('interpretation'), ' What they are said to describe is paraphrased from ', (NR.sources || []).join(', '), '.'));
  put(b, ...out);
}, { blurb: 'every number, every school, explained' });

page('matching', () => 'Matching', 'Traditions', (b) => {
  if (!MATCH) return put(b, missing('Compatibility', 'Give a partner (--partner) to compute it.'));
  const k = MATCH.ashtakoota;
  put(b, pageHead('matching', 'Compatibility (Ashtakoota)', `Total ${k.total} of ${k.max}.`), h('div', { class: 'stats' }, k.kootas.map(x => h('div', { class: 'stat' }, h('div', { class: 'stat-l' }, x.koota.replace('_', ' ')), h('div', { class: 'stat-v' }, `${x.score}/${x.max}`), bar(x.score, x.max)))),
    card('Kūṭas', table(['Kūṭa', 'A', 'B', 'Score', 'Max', 'Rule'], k.kootas.map(x => [x.koota.replace('_', ' '), typeof x.a === 'object' ? JSON.stringify(x.a) : x.a, typeof x.b === 'object' ? JSON.stringify(x.b) : x.b, x.score, x.max, x.note]))),
    card('', facts([['A Moon', `${k.a_moon.nakshatra} ${k.a_moon.pada} · ${SIGN_SKT[k.a_moon.sign]}`], ['B Moon', `${k.b_moon.nakshatra} ${k.b_moon.pada} · ${SIGN_SKT[k.b_moon.sign]}`],
      ['Manglik A', (MATCH.manglik.a || []).filter(x => x.present).map(x => x.id).join(', ') || 'none'], ['Manglik B', (MATCH.manglik.b || []).filter(x => x.present).map(x => x.id).join(', ') || 'none'], ['Rule pack', k.rule_version]])));
}, { blurb: 'aṣṭakūṭa out of 36' });

page('data', () => 'Data', 'Data', (b) => {
  const raw = JSON.stringify(D, null, 1);
  put(b, pageHead('data', 'Provenance and data', 'Every setting, source and assumption behind the numbers, and the raw envelopes.'),
    h('div', { class: 'grid3' }, Object.entries(ENV).filter(([, e]) => e && e.provenance).map(([n, e]) => card(n,
      facts([...Object.entries(e.calculation || {}).map(([k, v]) => [k.replaceAll('_', ' '), v == null ? '—' : String(v)]), ...Object.entries(e.provenance).filter(([k]) => !['assumptions', 'rulesets'].includes(k)).map(([k, v]) => [k.replaceAll('_', ' '), String(v)]), ['rule packs', (e.provenance.rulesets || []).join(', ')]]),
      (e.provenance.assumptions || []).length ? h('details', {}, h('summary', {}, `${e.provenance.assumptions.length} assumptions`), h('ul', {}, e.provenance.assumptions.map(a => h('li', {}, a)))) : null,
      (e.warnings || []).length ? h('ul', { class: 'warn' }, e.warnings.map(w => h('li', {}, typeof w === 'string' ? w : `${w.code}: ${w.message}`))) : null))),
    D.errors ? card('Not computed', h('ul', { class: 'warn' }, Object.entries(D.errors).map(([k, v]) => h('li', {}, h('b', {}, k), ': ', v)))) : null,
    h('details', { class: 'card' }, h('summary', {}, `Raw data (${raw.length.toLocaleString()} characters)`), h('pre', { class: 'raw' }, raw)));
}, { blurb: 'provenance and raw JSON' });

/* ── navigation, palette, drawer, boot ────────────────────────────────────────────────────────────────────────── */
function buildNav() {
  const nav = $('nav.side'); if (!nav) return;
  const groups = {};
  for (const p of Object.values(PAGES)) if (!p.hidden) (groups[p.group] = groups[p.group] || []).push(p);
  put(nav, Object.entries(groups).map(([g, list]) => h('div', { class: 'navgrp' }, h('div', { class: 'navg' }, g), list.map(p => h('a', { href: '#/' + p.id, 'data-page': p.section || p.id, class: 'navlink' }, h('span', { html: sigil(p.id) }), h('span', { class: 'navt' }, p.title()))))));
}
function palette() {
  const dlg = $('#palette'); if (!dlg) return;
  const items = Object.values(PAGES).filter(p => !p.hidden).map(p => ({ t: p.title(), h: '#/' + p.id, k: 'page' }))
    .concat(GRAHA_ORDER.filter(g => GR[g]).map(g => ({ t: `${G[g].en} · ${G[g].skt}`, h: '#/planet/' + g, k: 'graha' })))
    .concat(LAGNA_SIGN == null ? [] : Array.from({ length: 12 }, (_, i) => ({ t: `Bhāva ${i + 1} · ${SIGN_SKT[(LAGNA_SIGN + i) % 12]}`, h: '#/house/' + (i + 1), k: 'bhāva' })))
    .concat(YOGA_PRESENT().map(y => ({ t: y.name, h: '#/yoga/' + y.id, k: y.group })))
    .concat(chartPoints().map(x => ({ t: (READ.points[x.id] || {}).name || x.id, h: '#/point/' + x.id, k: 'point' })));
  const input = $('input', dlg), list = $('ul', dlg); let sel = 0;
  const draw = () => { const q = input.value.toLowerCase(); const hits = items.filter(x => x.t.toLowerCase().includes(q)).slice(0, 12); sel = Math.min(sel, hits.length - 1);
    put(list, hits.map((x, i) => h('li', { class: i === sel ? 'on' : '', onclick: () => { close(); go(x.h); } }, h('span', {}, x.t), h('small', {}, x.k)))); return hits; };
  const open = () => { dlg.hidden = false; input.value = ''; sel = 0; draw(); input.focus(); };
  const close = () => { dlg.hidden = true; };
  input.addEventListener('input', () => { sel = 0; draw(); });
  input.addEventListener('keydown', e => { const hits = draw(); if (e.key === 'ArrowDown') { sel = Math.min(hits.length - 1, sel + 1); draw(); e.preventDefault(); } else if (e.key === 'ArrowUp') { sel = Math.max(0, sel - 1); draw(); e.preventDefault(); } else if (e.key === 'Enter' && hits[sel]) { close(); go(hits[sel].h); } else if (e.key === 'Escape') close(); });
  dlg.addEventListener('click', e => { if (e.target === dlg) close(); });
  document.addEventListener('keydown', e => { if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); dlg.hidden ? open() : close(); } });
  $('#search').addEventListener('click', open);
}
function drawer() {
  const dr = $('#drawer');
  const radio = (name, v, t, key) => h('label', {}, h('input', { type: 'radio', name, checked: S[key] === v, onchange: () => { S[key] = v; save(); route(); } }), t);
  put(dr, h('h2', {}, 'Show and style'), note('Choices are kept in this browser only.'),
    h('fieldset', {}, h('legend', {}, 'Planet icons'), radio('ic', 'sphere', 'Textured spheres', 'icons'), radio('ic', 'glyph', 'Astronomical glyphs', 'icons'), radio('ic', 'badge', 'Lettered badges', 'icons')),
    h('fieldset', {}, h('legend', {}, 'Charts'), radio('ch', 'south', 'South Indian', 'chart'), radio('ch', 'north', 'North Indian', 'chart'), radio('ch', 'both', 'Both', 'chart')),
    h('fieldset', {}, h('legend', {}, 'Chakra centre'), radio('ce', 'south', 'South Indian boxes', 'centre'), radio('ce', 'north', 'North Indian', 'centre'), radio('ce', 'lanes', 'Orbit lanes', 'centre')),
    h('fieldset', {}, h('legend', {}, 'Include in charts'), h('label', {}, h('input', { type: 'checkbox', checked: S.showOuter !== false, onchange: e => { S.showOuter = e.target.checked; save(); applyVisibility(); route(); } }), 'Uranus, Neptune, Pluto'),
      h('label', {}, h('input', { type: 'checkbox', checked: S.showUpa !== false, onchange: e => { S.showUpa = e.target.checked; save(); applyVisibility(); route(); } }), 'Upagrahas (Dhūma … Māndi)')),
    h('fieldset', {}, h('legend', {}, 'Motion'), h('label', {}, h('input', { type: 'checkbox', checked: S.animate, onchange: e => { S.animate = e.target.checked; save(); route(); } }), 'Animate charts'),
      h('label', {}, h('input', { type: 'checkbox', checked: S.sky, onchange: e => { S.sky = e.target.checked; save(); location.reload(); } }), 'Living sky background')),
    h('fieldset', {}, h('legend', {}, 'Interface language'),
      ...Object.values(D.locales || {}).map(c => h('label', {}, h('input', { type: 'radio', name: 'lang', checked: S.lang === c.lang,
        onchange: () => { S.lang = c.lang; save(); location.reload(); } }), c.name + (c.status === 'draft' ? ' (draft)' : ''))),
      note('Labels only; values, numbers and dates are the same in every language.')),
    h('button', { class: 'btn', onclick: () => { const lang = S.lang; S = JSON.parse(JSON.stringify(DEFAULTS)); save(); if (S.lang !== lang) return location.reload(); route(); drawer(); } }, 'Reset to defaults'), ' ',
    h('button', { class: 'btn primary', onclick: () => dr.classList.remove('open') }, 'Done'));
}
