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
const toggle = (key, text, after) => h('label', { class: 'tog' }, h('input', { type: 'checkbox', checked: !!S.layers[key], onchange: e => { S.layers[key] = e.target.checked; save(); after(); } }), text);
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
page('chakra', () => 'Panchang chakra', 'Chart', (b) => {
  if (!VED) return put(b, missing('The chakra', 'It needs a Vedic chart.'));
  const host = h('div', { class: 'figure wheel-host' }), panel = h('div', { class: 'card panel' });
  const redraw = () => route();
  const transit = D.transits && D.transits.rows && D.transits.rows.length;
  const centre = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.centre === v), onclick: () => { S.centre = v; save(); redraw(); } }, t);
  put(b, pageHead('chakra', 'Panchang chakra', 'Every graha placed by its exact degree on the nakshatra, pada (navāṃśa) and rāśi rings.'),
    h('div', { class: 'toolbar' },
      h('div', { class: 'grp' }, h('b', {}, 'Centre'), centre('south', 'South Indian'), centre('north', 'North Indian'), centre('lanes', 'Orbit lanes')),
      h('div', { class: 'grp' }, h('b', {}, 'Rings'), toggle('ticks', 'degrees', redraw), toggle('nak', 'nakshatras', redraw), toggle('pada', 'padas', redraw), toggle('dres', 'dreshkāṇas', redraw), toggle('houses', 'bhāvas', redraw), toggle('sav', 'SAV', redraw)),
      h('div', { class: 'grp' }, h('b', {}, 'Layers'), toggle('needles', 'needles', redraw), toggle('drishti', 'drishti arcs', redraw), toggle('aspect_marks', 'aspect marks', redraw), toggle('rashi_drishti', 'rāśi drishti', redraw), toggle('west_aspects', 'Western aspects', redraw), transit ? toggle('transits', 'transits', redraw) : null),
      h('div', { class: 'grp' }, h('b', {}, 'Turn'), h('button', { class: 'tab', 'aria-selected': String(S.orientation === 'template'), onclick: () => { S.orientation = 'template'; save(); redraw(); } }, 'Mesha at 11'),
        h('button', { class: 'tab', 'aria-selected': String(S.orientation === 'lagna'), disabled: ASC == null, onclick: () => { S.orientation = 'lagna'; save(); redraw(); } }, 'Lagna at top'))),
    transit && S.layers.transits ? h('div', { class: 'slider' }, h('button', { class: 'btn', onclick: e => playTransit(e.target) }, '▶ Play transits'),
      h('input', { type: 'range', id: 'transit-slider', min: 0, max: D.transits.rows.length - 1, value: transitIdx, oninput: e => { transitIdx = +e.target.value; drawTransit(); } }), h('span', { id: 'transit-date', class: 'mono' })) : null,
    h('div', { class: 'chakra-layout' }, host, panel),
    h('div', { class: 'legend' }, h('span', {}, h('i', { style: 'background:#6FAF1F' }), 'pushkara navāṃśa'), h('span', {}, h('i', { style: 'background:#3ED9E4' }), 'vargottama'), h('span', {}, h('i', { style: 'background:#D7141F' }), 'gaṇḍānta'),
      h('span', {}, 'Ju Sa: translucent marks show which grahas aspect a sign (* special aspect)'), h('span', {}, 'flags: R retrograde · C combust · ↑ exalted · ↓ debilitated · ◎ own · V vargottama')));
  renderWheel(host, panel); resetFocus(panel);
}, { blurb: 'nakshatras, padas, navāṃśa and every point' });

/* ── charts ───────────────────────────────────────────────────────────────────────────────────────────────────── */
page('charts', () => 'Charts', 'Chart', (b) => {
  if (!VED) return put(b, missing('Charts', 'They need a Vedic chart.'));
  const re = () => route();
  const style = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.chart === v), onclick: () => { S.chart = v; save(); re(); } }, t);
  const icons = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.icons === v), onclick: () => { S.icons = v; save(); re(); } }, t);
  const kind = S.chart === 'north' ? 'north' : 'south';
  const grid = h('div', { class: 'grid2' });
  grid.append(card('Rāśi · D1', h('div', { class: 'figure' }, chartSVG(kind, 1))));
  if (S.chart === 'both') grid.append(card('Rāśi · D1 (North Indian)', h('div', { class: 'figure' }, chartSVG('north', 1))));
  if (DIVS.includes(9)) grid.append(card('Navāṃśa · D9', h('div', { class: 'figure' }, chartSVG(kind, 9))));
  const bc = val('vedic.bhava_chalit');
  if (bc) {
    const rows = Object.entries(bc.grahas).filter(([g]) => G[g]).map(([g, hh]) => [glink(g), `${SIGN_SKT[signOf(GR[g].lon)]} ${inSignDM(GR[g].lon)}`, houseOf(signOf(GR[g].lon)), hh, hh !== houseOf(signOf(GR[g].lon)) ? h('b', {}, 'shifts') : '']);
    grid.append(card('Bhāva chalit (Śrīpati)', table(['Graha', 'Position', 'Rāśi bhāva', 'Chalit bhāva', ''], rows), note('Bhāva madhyas and sandhis from Porphyry trisection (matches the Swiss Ephemeris Śrīpati houses within 0.2″).')));
  }
  grid.append(card('Sudarshana chakra', h('div', { class: 'figure narrow' }, sudarshana())));
  put(b, pageHead('charts', 'Charts', 'Rāśi, navāṃśa, bhāva chalit and Sudarshana. Click a house for its page, a graha for its own.'),
    h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'Style'), style('south', 'South Indian'), style('north', 'North Indian'), style('both', 'Both')),
      h('div', { class: 'grp' }, h('b', {}, 'Icons'), icons('sphere', 'Spheres'), icons('glyph', 'Glyphs'), icons('badge', 'Badges')),
      h('div', { class: 'grp' }, h('b', {}, 'Show'), toggle('aspect_marks', 'aspect marks', re), toggle('arudha', 'ārūḍha padas', re))),
    grid, note('In each house: graha, degree, nakshatra; small marks R retrograde, C combust, ↑ exalted, ↓ debilitated, ◎ own sign, V vargottama. Translucent letters along the bottom of a house are the grahas aspecting it.'));
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
  put(b, pageHead('houses', 'Bhāvas', 'Twelve houses from the lagna. Each has its own page.'), h('div', { class: 'hgrid' }, cards));
}, { blurb: 'a page for every bhāva' });

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
    ae ? card('Significations (reference)', h('p', {}, ae.significations)) : null);
}, { hidden: true, section: 'house' });

/* ── aspects ──────────────────────────────────────────────────────────────────────────────────────────────────── */
page('aspects', () => 'Aspects', 'Grahas & bhāvas', (b) => {
  const a = val('vedic.aspects'); if (!a) return put(b, missing('Aspects', 'Ask for the aspects feature.'));
  const mat = (m) => { const ids = NINE.filter(g => m[g] || GR[g]); return table(['', ...ids.map(g => G[g].abbr)], ids.map(from => [glink(from), ...ids.map(to => {
    const x = (m[from] || {})[to]; if (from === to || !x) return ''; return { td: { class: 'asp-cell', style: `--v:${Math.min(1, x.virupa / 60)}`, title: `${G[from].en} → ${G[to].en}: ${nth(x.house)} house, ${x.fraction || 'no graha drishti'}, ${fix(x.virupa, 1)} virūpa` }, v: `${x.fraction || '·'} (${Math.round(x.virupa)})` }; })])); };
  put(b, pageHead('aspects', 'Aspects between grahas', 'Rows aspect columns. Fraction: whole-sign graha drishti; number: sputa drishti in virūpas (0–60).'),
    card('D1 · rāśi', mat(a.d1)), card('D9 · navāṃśa', mat(a.d9)), note(a.note));
}, { blurb: 'who aspects whom, and how strongly' });

/* ── strength ─────────────────────────────────────────────────────────────────────────────────────────────────── */
page('strength', () => 'Strength', 'Strength', (b) => {
  const sb = val('vedic.shadbala'), bb = val('vedic.bhava_bala'), vs = val('vedic.vimshopaka'), bal = SPECIAL.balance;
  if (!sb && !vs) return put(b, missing('Strength', 'Ask for the shadbala and vimshopaka features.'));
  const out = [pageHead('strength', 'Strength', 'Ṣaḍbala, bhāva bala, viṃśopaka and the balance of elements.')];
  if (sb) {
    const rows = SEVEN.filter(g => sb.grahas[g]).map(g => { const x = sb.grahas[g]; return [glink(g), fix(x.sthana.total, 1), fix(x.dig, 1), fix(x.kala.total, 1), fix(x.chesta, 1), fix(x.naisargika, 1), fix(x.drik, 1), h('b', {}, fix(x.total_virupa, 0)), fix(x.rupas, 2), x.required_rupas, h('div', { class: 'barcell' }, bar(x.ratio, 2, x.ratio >= 1 ? 'var(--good)' : 'var(--bad)'), fix(x.ratio, 2))]; });
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

/* ── daśā ─────────────────────────────────────────────────────────────────────────────────────────────────────── */
function timeline(periods, labelKey, sub, onPick) {
  const W = 1000, H = 54, t0 = T(periods[0].start_utc), t1 = T(periods[periods.length - 1].end_utc), x = t => (t - t0) / (t1 - t0) * W;
  const svg = s('svg', { viewBox: `0 0 ${W} ${H + 22}`, role: 'img', 'aria-label': 'Period timeline' });
  periods.forEach((p, i) => {
    const lord = (p.lord || '').toLowerCase(), col = (G[lord] || G.lagna).dot, a = x(T(p.start_utc)), bb = x(T(p.end_utc));
    const r = s('rect', { x: a, y: 4, width: Math.max(1, bb - a - 1), height: H - 8, rx: 6, fill: col, stroke: 'rgba(0,0,0,.25)', class: 'tl' }, svg);
    r.style.setProperty('--i', i);
    r.addEventListener('mousemove', ev => tip(ev, `${p[labelKey] || p.lord}`, [['From', dateOnly(p.start_utc)], ['To', dateOnly(p.end_utc)], ['Years', p.years && p.years.toFixed(2)]], sub));
    r.addEventListener('mouseleave', untip); r.addEventListener('click', () => onPick && onPick(p, i));
    if (bb - a > 40) { const t = s('text', { class: 'lbl', x: (a + bb) / 2, y: H / 2, 'font-size': 13, 'font-weight': 700, fill: ink(col) }, svg); t.textContent = (p[labelKey] || p.lord).slice(0, bb - a > 90 ? 12 : 3); }
    if (bb - a > 30) { const yr = s('text', { x: a + 2, y: H + 16, 'font-size': 11, fill: 'var(--muted)' }, svg); yr.textContent = p.start_utc.slice(0, 4); }
  });
  if (REF_UTC && T(REF_UTC) > t0 && T(REF_UTC) < t1) { const nx = x(T(REF_UTC)); s('line', { x1: nx, x2: nx, y1: 0, y2: H, stroke: 'var(--accent)', 'stroke-width': 3, class: 'now' }, svg); const t = s('text', { x: nx + 4, y: 12, 'font-size': 12, 'font-weight': 800, fill: 'var(--accent)' }, svg); t.textContent = 'NOW'; }
  return h('div', { class: 'timeline' }, svg);
}
page('dasha', () => 'Daśā', 'Time', (b) => {
  const vim = val('vedic.dasha.vimshottari'), yog = val('vedic.dasha.yogini'), ss = val('vedic.sade_sati');
  if (!vim && !yog) return put(b, missing('Daśā', 'It needs an exact birth time.'));
  const out = [pageHead('dasha', 'Daśā periods', 'Click a period to open its sub-periods. NOW marks the reference instant recorded in this file.')];
  if (vim) {
    const drill = h('div', {});
    const pick = (p) => { const kids = p.antardashas || p.pratyantardashas || []; put(drill, h('h4', {}, `${p.lord}: sub-periods`), kids.length ? timeline(kids, 'lord', p.lord, pick) : note('Deepest level here; the current chain goes to five levels.')); };
    out.push(card(`Vimśottarī · balance at birth ${fix(vim.balance_at_birth_years, 3)} years of ${vim.starting_lord}`, timeline(vim.mahadashas, 'lord', 'mahādaśā', pick), drill));
    if (vim.current) out.push(card(`Current chain at ${fmtTime(vim.current.at_utc)}`, h('div', { class: 'chain' }, vim.current.periods.map(p => h('div', { class: 'link' }, h('small', {}, p.level), h('b', {}, glink(p.lord.toLowerCase())), h('small', {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`))))));
  }
  if (yog) out.push(card(`Yoginī · starts with ${yog.starting_yogini}, balance ${fix(yog.balance_at_birth_years, 3)} years`, timeline(yog.mahadashas, 'yogini', 'yoginī daśā', null),
    yog.current ? h('div', { class: 'chain' }, yog.current.periods.map(p => h('div', { class: 'link' }, h('small', {}, p.level), h('b', {}, p.lord), h('small', {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`)))) : null));
  if (ss) out.push(card(`Sāḍe Sātī${ss.active_at_reference ? ' · active now' : ''}`, table(['From', 'To', 'Phases'], ss.windows.map(w => [dateOnly(w.start), dateOnly(w.end), w.phases.map(p => `${p.phase} ${dateOnly(p.start)}→${dateOnly(p.end)}`).join(' · ')])), ss.note ? note(ss.note) : null));
  if (vim) out.push(h('details', { class: 'card' }, h('summary', {}, 'Every mahādaśā and antardaśā'), table(['Mahādaśā', 'Antardaśā', 'From', 'To'], vim.mahadashas.flatMap(m => (m.antardashas || []).map(a => [m.lord, a.lord, dateOnly(a.start_utc), dateOnly(a.end_utc)])))));
  put(b, out);
}, { blurb: 'Vimśottarī, Yoginī, Sāḍe Sātī' });

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

/* ── special points, lagnas and Jaimini ───────────────────────────────────────────────────────────────────────── */
page('special', () => 'Lagnas & points', 'Systems', (b) => {
  const sp = SPECIAL, p = val('vedic.panchanga') || {}, ck = val('vedic.jaimini.chara_karakas'), ar = val('vedic.jaimini.arudha_padas');
  if (!Object.keys(sp).length && !ck) return put(b, missing('Special points', 'Ask for the special_points and jaimini features.'));
  const sg = x => x == null ? '—' : SIGN_SKT[x];
  const ORDER = ['atmakaraka', 'amatyakaraka', 'bhratrikaraka', 'matrikaraka', 'pitrikaraka', 'putrakaraka', 'gnatikaraka', 'darakaraka'];
  const kar = sch => table(['Kāraka', 'Graha', 'Degrees in sign', 'Navāṃśa'], Object.entries(ck[sch]).sort((a, bb) => ORDER.indexOf(a[0]) - ORDER.indexOf(bb[0])).map(([k, v]) => [k, glink(v.graha), fix(v.degrees_in_sign, 3), v.navamsa_sign != null ? SIGN_SKT[v.navamsa_sign] : '—']));
  put(b, pageHead('special', 'Lagnas and special points', 'Special lagnas, yogi and avayogi, dagdha rāśis, Jaimini kārakas and ārūḍhas, lordships.'),
    h('div', { class: 'grid3' },
      card('Lagnas', facts([['Lagna', ASC != null ? `${SIGN_SKT[signOf(ASC)]} ${inSign(ASC)}` : null],
        ...['bhava_lagna', 'hora_lagna', 'ghati_lagna'].filter(k => p.special_lagnas && p.special_lagnas[k]).map(k => [k.replace('_', ' '), `${SIGN_SKT[p.special_lagnas[k].sign]} ${inSign(p.special_lagnas[k].longitude_deg)}`]),
        ['Indu lagna', sp.indu_lagna ? sg(sp.indu_lagna.sign) : null], ['Ārūḍha lagna', ar ? sg(ar.A1_arudha_lagna.pada_sign) : null], ['Upapada', ar ? sg(ar.A12_upapada.pada_sign) : null]])),
      card('Yogi and dagdha', facts([['Yogi point', sp.yogi ? `${SIGN_SKT[sp.yogi.sign]} ${inSign(sp.yogi.point)}` : null], ['Yogi', sp.yogi ? glink(sp.yogi.yogi_graha) : null], ['Duplicate yogi', sp.yogi ? glink(sp.yogi.duplicate_yogi) : null],
        ['Avayogi', sp.avayogi ? glink(sp.avayogi.avayogi_graha) : null], ['Dagdha rāśis', sp.dagdha_rashis ? sp.dagdha_rashis.signs.map(x => SIGN_SKT[x]).join(', ') || 'none' : null],
        ['64th navāṃśa (lagna / Moon)', sp.navamsha_64th_from_moon ? `${sg((sp.navamsha_64th_from_lagna || {}).sign)} / ${sg(sp.navamsha_64th_from_moon.sign)}` : null],
        ['22nd drekkāṇa (lagna / Moon)', sp.drekkana_22nd_from_moon ? `${sg((sp.drekkana_22nd_from_lagna || {}).sign)} / ${sg(sp.drekkana_22nd_from_moon.sign)}` : null]])),
      card('Conditions', facts([['Graha yuddha', sp.graha_yuddha && sp.graha_yuddha.length ? sp.graha_yuddha.map(w => w.grahas.join(' × ')).join(', ') : 'none'],
        ['Kartari', sp.kartari ? Object.entries(sp.kartari).map(([k, v]) => `${k === 'lagna' ? 'Lagna' : G[k].en}: ${v.kind.replace('_', ' ')}`).join('; ') || 'none' : null]]))),
    ck ? h('div', { class: 'grid2' }, card('Chara kārakas · 8 (with Rahu)', kar('eight_karaka_scheme')), card('Chara kārakas · 7', kar('seven_karaka_scheme'), note(`Kārakāṃśa: ${sg(ck.karakamsa_sign)}`))) : null,
    ar ? card('Ārūḍha padas', table(['Pada', 'Of bhāva', 'Lord', 'Lord in', 'Pada sign', 'Pada bhāva'], Object.entries(ar).sort((a, bb) => a[1].house - bb[1].house).map(([k, v]) => [k.replace('_', ' '), v.house, glink(v.lord), SIGN_SKT[v.lord_sign], SIGN_SKT[v.pada_sign], v.pada_house]))) : null,
    sp.lordships ? card('Lordships and functional nature', table(['Graha', 'Owns', 'Placed in', 'Nature'], SEVEN.map(g => [glink(g), sp.lordships.grahas[g].houses_owned.join(', '), sp.lordships.grahas[g].placed_in_house, sp.lordships.grahas[g].functional_nature])), note(sp.lordships.rule)) : null);
}, { blurb: 'special lagnas, yogi, Jaimini' });

/* ── yogas ────────────────────────────────────────────────────────────────────────────────────────────────────── */
page('yogas', () => 'Yogas', 'Systems', (b) => {
  if (!YOGAS) return put(b, missing('Yogas', 'Ask for the yogas feature.'));
  const list = arr => h('div', { class: 'ygrid' }, arr.map(x => h('div', { class: 'card yoga ' + (x.present ? 'on' : x.present === false ? 'off' : 'na') }, h('div', { class: 'yh' }, h('b', {}, x.name), h('span', { class: 'ystate' }, x.present ? 'present' : x.present === false ? 'absent' : 'not evaluable')),
    h('p', {}, x.rule), h('div', { class: 'chips' }, (x.grahas || []).map(g => G[g] ? h('a', { class: 'chip', href: '#/planet/' + g }, dot(g), G[g].abbr) : null)))));
  const onlyPresent = !!S.yogaPresent;
  put(b, pageHead('yogas', 'Yogas and doshas', 'Each entry states the rule checked (rule pack vedic.yogas). Whether it holds is a derived fact; ESHANA does not say what it means.'),
    h('div', { class: 'toolbar' }, h('label', { class: 'tog' }, h('input', { type: 'checkbox', checked: onlyPresent, onchange: e => { S.yogaPresent = e.target.checked; save(); route(); } }), 'only those present')),
    h('h3', { class: 'section-t' }, 'Yogas'), list(YOGAS.yogas.filter(x => !onlyPresent || x.present)), h('h3', { class: 'section-t' }, 'Doshas'), list(YOGAS.doshas.filter(x => !onlyPresent || x.present)));
}, { blurb: 'rules that hold in this chart' });

/* ── KP ───────────────────────────────────────────────────────────────────────────────────────────────────────── */
page('kp', () => 'KP', 'Systems', (b) => {
  const k = val('vedic.kp'); if (!k) return put(b, missing('KP', 'It needs an exact birth time and place.'));
  const cusps = Object.entries(k.cusps).map(([hh, c]) => [h('a', { href: '#/house/' + hh }, hh), `${SIGN_SKT[signOf(c.cusp)]} ${inSign(c.cusp)}`, c.lords.sign_lord, c.lords.star_lord, c.lords.sub_lord, c.lords.sub_sub_lord, c.A_in_star_of_occupants.join(', '), c.B_occupants.join(', '), c.C_in_star_of_lord.join(', '), c.D_lord]);
  const pl = Object.entries(k.planets).sort((a, bb) => GRAHA_ORDER.indexOf(a[0]) - GRAHA_ORDER.indexOf(bb[0])).map(([g, l]) => { const sv = k.significators_planet_view[g]; return [glink(g), l.sign_lord, l.star_lord, l.sub_lord, l.sub_sub_lord, sv.A_house_of_star_lord, sv.B_house_occupied, sv.C_houses_owned_by_star_lord.join(', '), sv.D_houses_owned.join(', ')]; });
  put(b, pageHead('kp', 'KP (Krishnamurti Paddhati)', k.note || 'Placidus cusps, Vimśottarī sub-division of every nakshatra.'),
    card('Cusps and house significators', table(['Cusp', 'Position', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A', 'B', 'C', 'D'], cusps)),
    card('Planets and planet-view significators', table(['Graha', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A', 'B', 'C', 'D'], pl)),
    k.ruling_planets ? card('Ruling planets at the chart instant', facts(Object.entries(k.ruling_planets).map(([kk, v]) => [kk.replaceAll('_', ' '), v]))) : null);
}, { blurb: 'sub-lords and significators' });

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

page('progressions', () => 'Progressions', 'Time', (b) => {
  const pr = D.progressions; if (!pr || !pr.rows || !pr.rows.length) return put(b, missing('Progressions', 'They need an exact birth time.'));
  const ids = Object.keys(pr.rows[0].longitudes).filter(k => ['sun', 'moon', 'mercury', 'venus', 'mars'].includes(k));
  const W = 1000, H = 220, svg = s('svg', { viewBox: `0 0 ${W} ${H + 24}`, role: 'img', 'aria-label': 'Secondary progressions by age' });
  for (let sg = 0; sg <= 12; sg++) s('line', { x1: 0, x2: W, y1: H - sg / 12 * H, y2: H - sg / 12 * H, stroke: 'var(--line)' }, svg);
  for (const id of ids) { let d = '', prev = null; pr.rows.forEach((r, i) => { const x = i / (pr.rows.length - 1) * W, l = r.longitudes[id], y = H - l / 360 * H; d += (prev != null && Math.abs(l - prev) > 180 ? 'M' : (i ? 'L' : 'M')) + `${x.toFixed(1)} ${y.toFixed(1)}`; prev = l; }); s('path', { d, fill: 'none', stroke: G[id].dot, 'stroke-width': 2.2, class: 'flowline' }, svg); }
  for (let a = 0; a < pr.rows.length; a += 10) { const t = s('text', { x: a / (pr.rows.length - 1) * W, y: H + 16, 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `age ${a}`; }
  const every = pr.rows.filter((_, i) => i % 5 === 0);
  put(b, pageHead('progressions', 'Secondary progressions', 'A day after birth for each year of life (tropical).'), card('', h('div', { class: 'timeline' }, svg), h('div', { class: 'legend' }, ids.map(id => h('span', {}, h('i', { style: `background:${G[id].dot}` }), G[id].en)))),
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
    tabs([['detailed', 'Detailed BodyGraph'], ['schematic', 'Schematic']], S.hd, v => { S.hd = v; save(); route(); }),
    h('div', { class: 'grid2' }, h('div', { class: 'figure narrow' }, bodygraph()), h('div', {}, card('', facts([['Personality (UTC)', HD.positions.personality.instant_utc], ['Design (UTC)', HD.positions.design.instant_utc], ['Channels', (bg.channels || []).join(', ')], ['Defined', (bg.defined_centers || []).join(', ')], ['Undefined', (bg.undefined_centers || []).join(', ')]])),
      card('Activations', table(['Body', 'Personality', 'Design'], Object.entries(byBody).map(([k, v]) => [k.replace('_', ' '), v.p ? `${v.p.gate}.${v.p.line}` : '', v.d ? h('b', { class: 'red' }, `${v.d.gate}.${v.d.line}`) : '']))), note('Black: personality · red: design · striped: both.'))));
}, { blurb: 'type, authority and BodyGraph', section: 'human-design' });

page('numerology', () => 'Numerology', 'Traditions', (b) => {
  if (!NUM) return put(b, missing('Numerology', 'It needs a name or a date.'));
  put(b, pageHead('numerology', 'Numerology', 'Each school is its own rule pack.'), h('div', { class: 'grid3' }, NUM.derived_attributes.map(a => card(a.id, facts(Object.entries(a.value).filter(([k]) => k !== 'letters').map(([k, v]) =>
    [k.replaceAll('_', ' '), v && typeof v === 'object' && 'reduced' in v ? `${v.reduced} (${v.compound})` : typeof v === 'object' ? JSON.stringify(v) : String(v)])), note(label(a.epistemic), ' ', a.rule_version)))));
}, { blurb: 'name and birth numbers' });

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
    .concat(LAGNA_SIGN == null ? [] : Array.from({ length: 12 }, (_, i) => ({ t: `Bhāva ${i + 1} · ${SIGN_SKT[(LAGNA_SIGN + i) % 12]}`, h: '#/house/' + (i + 1), k: 'bhāva' })));
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
    h('fieldset', {}, h('legend', {}, 'Motion'), h('label', {}, h('input', { type: 'checkbox', checked: S.animate, onchange: e => { S.animate = e.target.checked; save(); route(); } }), 'Animate charts'),
      h('label', {}, h('input', { type: 'checkbox', checked: S.sky, onchange: e => { S.sky = e.target.checked; save(); location.reload(); } }), 'Living sky background')),
    h('button', { class: 'btn', onclick: () => { S = JSON.parse(JSON.stringify(DEFAULTS)); save(); route(); drawer(); } }, 'Reset to defaults'), ' ',
    h('button', { class: 'btn primary', onclick: () => dr.classList.remove('open') }, 'Done'));
}
(function boot() {
  const root = document.documentElement;
  const setMode = m => { root.dataset.mode = m; root.setAttribute('data-theme', m); };
  try { const t = localStorage.getItem('eshana-theme'); setMode(t || (matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark')); } catch (e) { setMode('dark'); }
  if (D.theme === 'light' || D.theme === 'dark') setMode(D.theme);
  $('#theme').addEventListener('click', () => { const nx = root.dataset.mode === 'dark' ? 'light' : 'dark'; setMode(nx); try { localStorage.setItem('eshana-theme', nx); } catch (e) { /* ignore */ } });
  $('#print').addEventListener('click', () => { $$('section.page').forEach(x => { x.hidden = false; }); $$('details').forEach(d => { d.open = true; }); window.print(); route(); });
  $('#settings').addEventListener('click', () => { drawer(); $('#drawer').classList.add('open'); });
  $('#json').addEventListener('click', () => { const a = h('a', { href: URL.createObjectURL(new Blob([JSON.stringify(D, null, 1)], { type: 'application/json' })), download: 'eshana-chart.json' }); document.body.append(a); a.click(); a.remove(); });
  $('#copy').addEventListener('click', async () => {
    const lines = [`Chart data computed by ESHANA. Values are calculated or tradition rules, not interpretations.`];
    if (VED) { lines.push(`Lagna: ${ASC != null ? SIGN_SKT[signOf(ASC)] + ' ' + inSign(ASC) : 'unknown'}`); for (const g of GRAHA_ORDER) if (GR[g]) lines.push(`${G[g].en}: ${SIGN_SKT[signOf(GR[g].lon)]} ${inSign(GR[g].lon)}, ${NAKSHATRA[nakOf(GR[g].lon)]} pada ${padaOf(GR[g].lon)}${GR[g].retro ? ', retrograde' : ''}`); }
    const vim = val('vedic.dasha.vimshottari'); if (vim && vim.current) lines.push('Current Vimśottarī: ' + vim.current.periods.map(p => `${p.level} ${p.lord}`).join(' / '));
    try { await navigator.clipboard.writeText(lines.join('\n')); $('#copy').textContent = '✓ Copied'; } catch (e) { $('#copy').textContent = 'Copy blocked'; }
  });
  $('#menu').addEventListener('click', () => document.body.classList.toggle('nav-open'));
  $$('nav.side').forEach(n => n.addEventListener('click', e => { if (e.target.closest('a')) document.body.classList.remove('nav-open'); }));
  document.addEventListener('keydown', e => {
    if (e.target.matches('input, textarea') || e.metaKey || e.ctrlKey) return;
    const list = Object.values(PAGES).filter(p => !p.hidden).map(p => p.id), i = list.indexOf(CURRENT);
    if (e.key === ']' && i >= 0) go('#/' + list[(i + 1) % list.length]); if (e.key === '[' && i >= 0) go('#/' + list[(i + list.length - 1) % list.length]);
  });
  buildNav(); palette(); startSky(); route();
})();
