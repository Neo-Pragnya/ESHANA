/* ESHANA chart studio, part 3: every section, the settings drawer, and boot. */
function table(head, rows, opts = {}) {
  return h('div', { class: 'tbl-wrap' }, h('table', {}, h('thead', {}, h('tr', {}, head.map(c => h('th', {}, c)))),
    h('tbody', {}, rows.map((r, i) => h('tr', Object.assign({}, opts.rowAttrs ? opts.rowAttrs(i) : {}), r.map(c => h('td', typeof c === 'object' && c && c.td ? c.td : {}, c && c.nodeType ? c : c && c.td ? c.v : c == null ? '—' : String(c))))))));
}
const dot = id => h('span', { style: `display:inline-block;width:10px;height:10px;border-radius:50%;margin-right:6px;background:${(G[id] || G.lagna).dot};border:1px solid rgba(0,0,0,.35)` });
const nameCell = id => h('span', {}, dot(id), (G[id] || {}).en || id);
const label = ep => h('span', { class: `ep ep-${ep}` }, ep);
function put(b, ...kids) { b.replaceChildren(...kids.flat(Infinity).filter(x => x != null && x !== false)); }
function body(id) { const sec = document.getElementById(id); if (!sec) return null; sec.hidden = false; return sec.querySelector('.body'); }
function hide(id) { const sec = document.getElementById(id); if (sec) sec.hidden = true; }
const REF_UTC = D.reference_utc;

/* ── summary ──────────────────────────────────────────────────────────────────────────────────────────────────── */
function renderSummary() {
  const b = body('summary'); if (!b) return;
  const inp = (VED || WES || HD || {}).input || {}, subj = D.subject || {}, chips = [];
  const add = (k, v) => v && chips.push(h('span', { class: 'chip' }, k + ': ', h('b', {}, v)));
  if (VED) { add('Lagna', ASC != null ? `${SIGN_SKT[signOf(ASC)]} ${inSign(ASC)}` : null); add('Moon', GR.moon && `${SIGN_SKT[signOf(GR.moon.lon)]} · ${NAKSHATRA[nakOf(GR.moon.lon)]} ${padaOf(GR.moon.lon)}`);
    add('Sun', GR.sun && SIGN_SKT[signOf(GR.sun.lon)]); const p = val('vedic.panchanga'); if (p) { add('Tithi', `${p.tithi.paksha} ${p.tithi.name}`); add('Vāra', p.vara && p.vara.name); add('Yoga', p.yoga.name); add('Karaṇa', p.karana.name); } }
  if (HD) { const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value; if (bg) add('Human Design', `${bg.type} ${bg.profile}`); }
  const place = inp.place;
  put(b, h('div', { class: 'chips' }, chips), h('div', { class: 'grid3' },
    h('div', { class: 'mini' }, facts([['Name', subj.name], ['Place', subj.place_label], ['Local time', subj.local], ['Instant (UTC)', inp.instant_utc],
      ['Zone', inp.civil_zone ? `${inp.civil_zone} (${(inp.civil_offset_s / 3600).toFixed(2)} h)` : null], ['Latitude, longitude', place ? `${place.latitude_deg.toFixed(4)}°, ${place.longitude_deg.toFixed(4)}°` : null]])),
    h('div', { class: 'mini' }, facts([['Birth time', inp.birth_time_quality], ['Julian day UT', inp.jd_ut && inp.jd_ut.toFixed(6)], ['Delta T (TT - UT)', inp.delta_t_s && inp.delta_t_s.toFixed(2) + ' s'],
      ['Ayanāṃśa', VED ? `${VED.calculation.ayanamsha} ${dms(VED.calculation.ayanamsha_deg)}` : null], ['Provider', (VED || WES || {}).calculation ? (VED || WES).calculation.provider : null],
      ['"Current" means', REF_UTC ? fmtTime(REF_UTC) : null]]))));
}

/* ── wheel section with toolbar and panel ─────────────────────────────────────────────────────────────────────── */
function renderChakra() {
  const b = body('chakra'); if (!b || !VED) { hide('chakra'); return; }
  const L = S.layers, tog = (k, t) => h('label', {}, h('input', { type: 'checkbox', checked: L[k], onchange: e => { L[k] = e.target.checked; save(); renderChakra(); renderCharts(); } }), t);
  const host = h('div', { class: 'figure' }); PANEL = h('div', { class: 'mini panel' });
  const transit = D.transits && D.transits.rows && D.transits.rows.length;
  put(b, 
    h('div', { class: 'toolbar no-print' },
      h('div', { class: 'grp' }, h('b', {}, 'Rings'), tog('ticks', 'degrees'), tog('nak', 'nakshatras'), tog('pada', 'padas / navāṃśa'), tog('dres', 'dreshkāṇas'), tog('houses', 'bhāvas'), tog('sav', 'SAV bindus')),
      h('div', { class: 'grp' }, h('b', {}, 'Layers'), tog('needles', 'exact-degree needles'), tog('drishti', 'graha drishti'), tog('rashi_drishti', 'rāśi drishti'), tog('west_aspects', 'Western aspects'), transit ? tog('transits', 'transits') : null),
      h('div', { class: 'grp' }, h('b', {}, 'Wheel'), h('label', {}, h('input', { type: 'radio', name: 'orient', checked: S.orientation === 'template', onchange: () => { S.orientation = 'template'; save(); renderChakra(); } }), 'Mesha at 11 o\'clock'),
        h('label', {}, h('input', { type: 'radio', name: 'orient', checked: S.orientation === 'lagna', disabled: ASC == null, onchange: () => { S.orientation = 'lagna'; save(); renderChakra(); } }), 'lagna at top'))),
    transit && L.transits ? h('div', { class: 'slider no-print' }, h('button', { class: 'btn', onclick: e => playTransit(e.target) }, '▶ Play transits'),
      h('input', { type: 'range', id: 'transit-slider', min: 0, max: D.transits.rows.length - 1, value: transitIdx, oninput: e => { transitIdx = +e.target.value; drawTransit(); } }), h('span', { id: 'transit-date', class: 'mono' })) : null,
    h('div', { class: 'grid2' }, host, PANEL),
    h('div', { class: 'legend' }, h('span', {}, h('i', { style: 'background:#6FAF1F' }), 'pushkara navāṃśa'), h('span', {}, h('i', { style: 'background:#3ED9E4' }), 'vargottama'), h('span', {}, h('i', { style: 'background:#D7141F' }), 'gaṇḍānta'),
      h('span', {}, '— solid drishti: full (7th) · dashed: special'), h('span', {}, 'markers: report.markers@0.1')));
  renderWheel(host); resetFocus();
}

/* ── South / North Indian charts with varga selector ──────────────────────────────────────────────────────────── */
function renderCharts() {
  const b = body('charts'); if (!b || !VED) { hide('charts'); return; }
  const host1 = h('div', { class: 'figure' }), host2 = h('div', { class: 'figure' });
  const style = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.chart === v), onclick: () => { S.chart = v; save(); renderCharts(); } }, t);
  const icons = (v, t) => h('button', { class: 'tab', 'aria-selected': String(S.icons === v), onclick: () => { S.icons = v; save(); renderAllCharts(); } }, t);
  put(b, h('div', { class: 'toolbar no-print' }, h('div', { class: 'tabs' }, style('south', 'South Indian'), style('north', 'North Indian'), style('both', 'Both')),
    h('div', { class: 'tabs' }, icons('sphere', 'Planet spheres'), icons('glyph', 'Glyphs'), icons('badge', 'Badges')),
    h('label', {}, h('input', { type: 'checkbox', checked: S.layers.chart_drishti, onchange: e => { S.layers.chart_drishti = e.target.checked; save(); renderCharts(); } }), 'drishti arrows'),
    h('label', {}, h('input', { type: 'checkbox', checked: S.layers.arudha, onchange: e => { S.layers.arudha = e.target.checked; save(); renderCharts(); } }), 'ārūḍha padas'),
    h('label', {}, 'Varga ', h('select', { onchange: e => { S.varga = +e.target.value; save(); renderCharts(); } }, DIVS.map(d => h('option', { value: d, selected: d === S.varga }, `D${d} · ${DIV_NAME[d] || ''}`))))),
    h('div', { class: 'grid3' }, host1, host2));
  const d = DIVS.includes(S.varga) ? S.varga : 1;
  if (S.chart === 'north') { northChart(host1, 1); if (d !== 1) northChart(host2, d); }
  else if (S.chart === 'both') { southChart(host1, d); northChart(host2, d); }
  else { southChart(host1, 1); if (d !== 1) southChart(host2, d); }
  if (ASC == null) b.append(h('p', { class: 'note' }, 'The North Indian chart and the lagna need an exact birth time and place; they are omitted rather than guessed.'));
}
function renderAllCharts() { renderChakra(); renderCharts(); }

/* ── planets, nakshatra ruler, vargas ─────────────────────────────────────────────────────────────────────────── */
function renderPlanets() {
  const b = body('planets'); if (!b || !VED) { hide('planets'); return; }
  const st = val('vedic.strength') || {}, kp = (val('vedic.kp') || {}).planets || {};
  const ids = (ASC != null ? ['lagna'] : []).concat(GRAHA_ORDER.filter(g => GR[g]));
  const rows = ids.map(id => {
    const l = id === 'lagna' ? ASC : GR[id].lon, gr = GR[id] || {}, x = st[id] || {}, k = kp[id] || {};
    return [nameCell(id), l.toFixed(4) + '°', SIGN_SKT[signOf(l)], inSign(l), NAKSHATRA[nakOf(l)], padaOf(l), G[NAK_LORD[nakOf(l) % 9]].en, SIGN_SKT[navSignOf(l)],
      LAGNA_SIGN == null ? '—' : (signOf(l) - LAGNA_SIGN + 12) % 12 + 1, gr.retro ? 'R' : '', gr.speed != null ? gr.speed.toFixed(4) : '', x.dignity || '', x.avastha || '',
      x.combust ? 'combust' : '', k.sub_lord || '', markers(l).join(', ')];
  });
  put(b, table(['Graha', 'Sidereal', 'Rāśi', 'In sign', 'Nakshatra', 'Pada', 'Nak. lord', 'Navāṃśa', 'Bhāva', 'R', '°/day', 'Dignity', 'Avasthā', 'Combust', 'KP sub', 'Markers'], rows,
    { rowAttrs: i => ({ 'data-body': ids[i] }) }));
  const houses = VED.houses; if (houses && houses.cusps_deg) b.append(h('h3', {}, `Bhāva cusps (${houses.system})`),
    table(['Bhāva', 'Cusp', 'Rāśi', 'In sign', 'Lord'], houses.cusps_deg.map((c, i) => [i + 1, c.toFixed(4) + '°', SIGN_SKT[signOf(c)], inSign(c), G[SIGN_LORD[signOf(c)]].en])),
    h('p', { class: 'note' }, `Midheaven ${houses.midheaven.longitude_deg.toFixed(4)}° · local apparent sidereal time ${houses.local_apparent_sidereal_time_deg.toFixed(4)}°`));
}
function renderNakshatraTable() {
  const b = body('nakshatra'); if (!b || !VED) { hide('nakshatra'); return; }
  const rows = NAKSHATRA.map((n, i) => {
    const here = GRAHA_ORDER.filter(g => GR[g] && nakOf(GR[g].lon) === i), r = NAKREF[i] || {};
    const pts = h('span', {}, here.map(g => h('span', { class: 'chip' }, dot(g), `${G[g].abbr} p${padaOf(GR[g].lon)}`)), ASC != null && nakOf(ASC) === i ? h('span', { class: 'chip' }, 'Lagna p' + padaOf(ASC)) : null);
    return [i + 1, n, `${SIGN_SKT[Math.floor(i * NAKS / 30)]} ${inSign(i * NAKS)}`, G[NAK_LORD[i % 9]].en, pts, r.gana, r.yoni, r.nadi, r.dosha, r.tattva, r.guna, r.purpose, r.direction];
  });
  put(b, table(['#', 'Nakshatra', 'Starts', 'Lord', 'Grahas here', 'Gana', 'Yoni', 'Nadi', 'Dosha', 'Tattva', 'Guna', 'Purpose', 'Direction'], rows,
    { rowAttrs: i => ({ class: GRAHA_ORDER.some(g => GR[g] && nakOf(GR[g].lon) === i) || (ASC != null && nakOf(ASC) === i) ? 'has' : '', style: 'cursor:pointer', onclick: () => { document.getElementById('chakra').scrollIntoView({ behavior: 'smooth' }); focus('nak', i); } }) }),
    h('p', { class: 'note' }, 'Attribute columns: the Nakshatra Ruler table of the Panchang Chakra template (tradition_rule, for reference). Click a row to turn the wheel to it.'));
}
function renderVargas() {
  const b = body('vargas'); if (!b || !VED || DIVS.length < 2) { hide('vargas'); return; }
  const ids = (ASC != null ? ['lagna'] : []).concat(GRAHA_ORDER.filter(g => GR[g]));
  const rows = ids.map(id => [nameCell(id), ...DIVS.map(d => { const sg = vargaSign(id, d); const d1 = vargaSign(id, 1);
    return sg == null ? '—' : { td: { style: d > 1 && sg === d1 ? 'background:color-mix(in srgb,#3ED9E4 30%,transparent);font-weight:700' : '' }, v: SIGN_SKT[sg] }; })]);
  put(b, table(['Body', ...DIVS.map(d => `D${d}`)], rows), h('p', { class: 'note' }, 'Highlighted: the same sign as in D1. Every varga sign comes from the engine (vedic.parashari@0.2).'));
}

/* ── panchanga ────────────────────────────────────────────────────────────────────────────────────────────────── */
function renderPanchanga() {
  const b = body('panchanga'); const p = val('vedic.panchanga'); if (!b || !p) { hide('panchanga'); return; }
  const lim = (t, v, e) => h('div', { class: 'mini' }, h('h3', {}, t), h('div', { style: 'font-size:18px;font-weight:700' }, v), h('div', { class: 'note' }, e ? 'ends ' + fmtTime(e.local) : ''));
  const sl = p.special_lagnas || {};
  put(b, h('div', { class: 'grid3' },
    lim('Tithi', `${p.tithi.paksha === 'shukla' ? 'Śukla' : 'Kṛṣṇa'} ${p.tithi.name} (${p.tithi.index})`, p.tithi.ends), lim('Vāra', p.vara ? `${p.vara.name} · ${p.vara.english}` : '—'),
    lim('Nakshatra', `${p.nakshatra.name} ${p.nakshatra.pada}`, p.nakshatra.ends), lim('Yoga', p.yoga.name, p.yoga.ends), lim('Karaṇa', p.karana.name, p.karana.ends)),
    h('div', { class: 'grid3' }, h('div', { class: 'mini' }, facts([['Sunrise', p.sunrise && fmtTime(p.sunrise.local)], ['Sunset', p.sunset && fmtTime(p.sunset.local)], ['Moonrise', p.moonrise ? fmtTime(p.moonrise.local) : 'none that day'],
      ['Moonset', p.moonset ? fmtTime(p.moonset.local) : 'none that day'], ['Day length', p.day_length_hours && `${p.day_length_hours.toFixed(2)} h`], ['Convention', p.sunrise_convention]])),
      h('div', { class: 'mini' }, facts([['Rāhu kālam', p.rahu_kalam && `${fmtTime(p.rahu_kalam.start.local).slice(11, 16)}–${fmtTime(p.rahu_kalam.end.local).slice(11, 16)}`],
        ['Yamagaṇḍam', p.yamagandam && `${fmtTime(p.yamagandam.start.local).slice(11, 16)}–${fmtTime(p.yamagandam.end.local).slice(11, 16)}`],
        ['Gulika kālam', p.gulika_kalam && `${fmtTime(p.gulika_kalam.start.local).slice(11, 16)}–${fmtTime(p.gulika_kalam.end.local).slice(11, 16)}`]])),
      h('div', { class: 'mini' }, facts([['Bhāva lagna', sl.bhava_lagna && `${SIGN_SKT[sl.bhava_lagna.sign]} ${inSign(sl.bhava_lagna.longitude_deg)}`], ['Horā lagna', sl.hora_lagna && `${SIGN_SKT[sl.hora_lagna.sign]} ${inSign(sl.hora_lagna.longitude_deg)}`],
        ['Ghaṭī lagna', sl.ghati_lagna && `${SIGN_SKT[sl.ghati_lagna.sign]} ${inSign(sl.ghati_lagna.longitude_deg)}`], ['Sun taken at', sl.sun_taken_at]]))),
    p.horas ? h('details', {}, h('summary', {}, 'Horās (24) and Choghaḍiyās (16) of the birth day'), h('div', { class: 'grid3' },
      table(['#', 'Horā lord', 'From', 'To'], p.horas.map(x => [x.current ? h('b', {}, `${x.n} ◀ birth`) : x.n, G[x.lord].en, fmtTime(x.start.local).slice(11, 19), fmtTime(x.end.local).slice(11, 19)])),
      table(['#', 'Choghaḍiyā', 'Quality', 'From', 'To'], p.choghadiya.map(x => [x.current ? h('b', {}, `${x.n} ◀ birth`) : x.n, `${x.name} (${x.period})`, x.quality, fmtTime(x.start.local).slice(11, 19), fmtTime(x.end.local).slice(11, 19)])))) : null);
}

/* ── dasha: timelines with drill-down to five levels ──────────────────────────────────────────────────────────── */
const T = iso => Date.parse(iso);
function timeline(periods, labelKey, sub, onPick) {
  const W = 1000, H = 54, t0 = T(periods[0].start_utc), t1 = T(periods[periods.length - 1].end_utc), x = t => (t - t0) / (t1 - t0) * W;
  const svg = s('svg', { viewBox: `0 0 ${W} ${H + 22}`, role: 'img', 'aria-label': 'Period timeline' });
  periods.forEach((p, i) => {
    const lord = (p.lord || '').toLowerCase(), col = (G[lord] || G.lagna).dot, a = x(T(p.start_utc)), b = x(T(p.end_utc));
    const r = s('rect', { x: a, y: 4, width: Math.max(1, b - a - 1), height: H - 8, rx: 6, fill: col, stroke: 'rgba(0,0,0,.25)' }, svg);
    r.addEventListener('mousemove', ev => tip(ev, `${p[labelKey] || p.lord}`, [['From', dateOnly(p.start_utc)], ['To', dateOnly(p.end_utc)], ['Years', p.years && p.years.toFixed(2)]], sub));
    r.addEventListener('mouseleave', untip); r.addEventListener('click', () => onPick && onPick(p, i));
    if (b - a > 40) { const t = s('text', { class: 'lbl', x: (a + b) / 2, y: H / 2, 'font-size': 13, 'font-weight': 700, fill: ink(col.length === 7 ? col : '#888888') }, svg); t.textContent = (p[labelKey] || p.lord).slice(0, b - a > 90 ? 12 : 3); }
    const yr = s('text', { x: a + 2, y: H + 16, 'font-size': 11, fill: 'var(--muted)' }, svg); yr.textContent = b - a > 30 ? p.start_utc.slice(0, 4) : '';
  });
  if (REF_UTC && T(REF_UTC) > t0 && T(REF_UTC) < t1) { const nx = x(T(REF_UTC)); s('line', { x1: nx, x2: nx, y1: 0, y2: H, stroke: 'var(--accent)', 'stroke-width': 3 }, svg); const t = s('text', { x: nx + 4, y: 12, 'font-size': 12, 'font-weight': 800, fill: 'var(--accent)' }, svg); t.textContent = 'NOW'; }
  return h('div', { class: 'timeline' }, svg);
}
function renderDasha() {
  const b = body('dasha'); const vim = val('vedic.dasha.vimshottari'), yog = val('vedic.dasha.yogini'); if (!b || (!vim && !yog)) { hide('dasha'); return; }
  const out = [];
  if (vim) {
    const drill = h('div', {});
    const pick = (p) => { const kids = p.antardashas || p.pratyantardashas || []; drill.replaceChildren(h('h3', {}, `${p.lord}: sub-periods`), kids.length ? timeline(kids, 'lord', p.lord, pick) : h('p', { class: 'note' }, 'Deepest level shown; the current chain below goes to five levels.')); };
    out.push(h('h3', {}, `Vimśottarī · balance at birth ${vim.balance_at_birth_years.toFixed(3)} years of ${vim.starting_lord} · ${vim.levels} levels`), timeline(vim.mahadashas, 'lord', 'mahādaśā', pick), drill);
    if (vim.current) out.push(h('h3', {}, `Current at ${fmtTime(vim.current.at_utc)}`), table(['Level', 'Lord', 'From', 'To'], vim.current.periods.map(p => [p.level, nameCell(p.lord.toLowerCase()), dateOnly(p.start_utc), dateOnly(p.end_utc)])));
    out.push(h('details', {}, h('summary', {}, 'All mahādaśās and antardaśās'), table(['Mahādaśā', 'Antardaśā', 'From', 'To'], vim.mahadashas.flatMap(m => (m.antardashas || []).map(a => [m.lord, a.lord, dateOnly(a.start_utc), dateOnly(a.end_utc)])))));
  }
  if (yog) {
    out.push(h('h3', {}, `Yoginī · starts with ${yog.starting_yogini}, balance ${yog.balance_at_birth_years.toFixed(3)} years`), timeline(yog.mahadashas, 'yogini', 'yoginī daśā', null));
    if (yog.current) out.push(table(['Level', 'Yoginī', 'From', 'To'], yog.current.periods.map(p => [p.level, p.lord, dateOnly(p.start_utc), dateOnly(p.end_utc)])));
  }
  const ss = val('vedic.sade_sati');
  if (ss) out.push(h('h3', {}, `Sāḍe Sātī (Saturn over the 12th, 1st and 2nd from the natal Moon)${ss.active_at_reference ? ' · active now' : ''}`),
    table(['From', 'To', 'Phases'], ss.windows.map(w => [dateOnly(w.start), dateOnly(w.end), w.phases.map(p => `${p.phase} ${dateOnly(p.start)}→${dateOnly(p.end)}`).join(' · ')])),
    ss.note ? h('p', { class: 'note' }, `Searched ${dateOnly(ss.range[0])} – ${dateOnly(ss.range[1])}: ${ss.note}.`) : null);
  put(b, ...out, h('p', { class: 'note' }, label('tradition_rule'), ` Year length ${vim ? vim.year_days : ''} days. Click a bar to open its sub-periods.`));
}

/* ── strength, relationships, ashtakavarga ────────────────────────────────────────────────────────────────────── */
function renderStrength() {
  const b = body('strength'); const st = val('vedic.strength'); if (!b || !st) { hide('strength'); return; }
  const ids = GRAHA_ORDER.filter(g => st[g]);
  const bar = v => v == null ? '—' : h('div', { style: 'display:flex;gap:6px;align-items:center' }, h('div', { class: 'bar', style: 'width:90px' }, h('i', { style: `width:${Math.max(0, Math.min(100, v / 60 * 100))}%` })), v.toFixed(1));
  const out = [table(['Graha', 'Dignity', 'Avasthā', 'Retrograde', 'Combust', 'Dig bala', 'Naisargika bala'], ids.map(g => [nameCell(g), st[g].dignity, st[g].avastha, st[g].retrograde ? 'yes' : '', st[g].combust ? 'yes' : '', bar(st[g].dig_bala_virupa), bar(st[g].naisargika_bala_virupa)]))];
  const rel = val('vedic.relationships');
  if (rel) { const seven = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn'], col = { great_friend: '#2f7d32', friend: '#6fbf73', neutral: '#999', enemy: '#e07b6e', great_enemy: '#b4322a' };
    out.push(h('h3', {}, 'Pañcadhā maitrī (natural + temporary)'), table(['', ...seven.map(g => G[g].abbr)], seven.map(a => [nameCell(a), ...seven.map(bb => a === bb ? '—' : { td: { style: `color:${col[rel[a][bb].combined]};font-weight:700` }, v: rel[a][bb].combined.replace('_', ' ') })]))); }
  put(b, ...out);
}
function renderAshtakavarga() {
  const b = body('ashtakavarga'); const a = val('vedic.ashtakavarga'); if (!b || !a) { hide('ashtakavarga'); return; }
  const heat = (v, max) => ({ td: { class: 'heat', style: `background:hsl(${Math.round(Math.min(1, v / max) * 120)} 65% 50% / .35)` }, v: String(v) });
  const seven = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn'];
  const rows = seven.map(g => [nameCell(g), ...a.bhinnashtakavarga[g].bindus.map(v => heat(v, 8)), a.bhinnashtakavarga[g].total]);
  rows.push([h('b', {}, 'Sarva'), ...a.sarvashtakavarga.bindus.map(v => heat(v, 40)), a.sarvashtakavarga.total]);
  put(b, table(['', ...SIGN_SKT.map((n, i) => LAGNA_SIGN != null ? `${n.slice(0, 4)} (${(i - LAGNA_SIGN + 12) % 12 + 1})` : n.slice(0, 4)), 'Total'], rows),
    h('p', { class: 'note' }, 'Columns are signs (bhāva from the lagna in brackets). Totals are fixed by the tables: 48, 49, 39, 54, 56, 52, 39; Sarva 337. Turn on "SAV bindus" on the wheel to see them by sign.'));
}

/* ── KP, Jaimini, yogas ───────────────────────────────────────────────────────────────────────────────────────── */
function renderKP() {
  const b = body('kp'); const k = val('vedic.kp'); if (!b || !k) { hide('kp'); return; }
  const cusps = Object.entries(k.cusps).map(([hh, c]) => [hh, `${SIGN_SKT[signOf(c.cusp)]} ${inSign(c.cusp)}`, c.lords.sign_lord, c.lords.star_lord, c.lords.sub_lord, c.lords.sub_sub_lord,
    c.A_in_star_of_occupants.join(', '), c.B_occupants.join(', '), c.C_in_star_of_lord.join(', '), c.D_lord]);
  const pl = Object.entries(k.planets).sort((a, b) => GRAHA_ORDER.indexOf(a[0]) - GRAHA_ORDER.indexOf(b[0])).map(([g, l]) => { const sv = k.significators_planet_view[g]; return [nameCell(g), l.sign_lord, l.star_lord, l.sub_lord, l.sub_sub_lord, sv.A_house_of_star_lord, sv.B_house_occupied, sv.C_houses_owned_by_star_lord.join(', '), sv.D_houses_owned.join(', ')]; });
  const rp = k.ruling_planets;
  put(b, h('h3', {}, 'Cusps (Placidus) and house significators'), table(['Cusp', 'Position', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A: in star of occupants', 'B: occupants', 'C: in star of lord', 'D: lord'], cusps),
    h('h3', {}, 'Planets and planet-view significators'), table(['Graha', 'Sign', 'Star', 'Sub', 'Sub-sub', 'A', 'B', 'C', 'D'], pl),
    rp ? h('div', { class: 'mini' }, h('h3', {}, 'Ruling planets at the chart instant'), facts(Object.entries(rp).map(([kk, v]) => [kk.replaceAll('_', ' '), v]))) : null,
    k.note ? h('p', { class: 'note' }, k.note) : null);
}
function renderJaimini() {
  const b = body('jaimini'); const ck = val('vedic.jaimini.chara_karakas'); if (!b || !ck) { hide('jaimini'); return; }
  const ORDER = ['atmakaraka', 'amatyakaraka', 'bhratrikaraka', 'matrikaraka', 'pitrikaraka', 'putrakaraka', 'gnatikaraka', 'darakaraka'];
  const kar = sch => table(['Kāraka', 'Graha', 'Degrees in sign', 'Navāṃśa'], Object.entries(ck[sch]).sort((a, b) => ORDER.indexOf(a[0]) - ORDER.indexOf(b[0])).map(([k, v]) => [k, nameCell(v.graha), v.degrees_in_sign.toFixed(3), v.navamsa_sign != null ? SIGN_SKT[v.navamsa_sign] : '—']));
  const ar = val('vedic.jaimini.arudha_padas');
  put(b, h('div', { class: 'grid3' }, h('div', {}, h('h3', {}, 'Chara kārakas · 8 (with Rahu)'), kar('eight_karaka_scheme')), h('div', {}, h('h3', {}, 'Chara kārakas · 7'), kar('seven_karaka_scheme'),
    h('p', { class: 'note' }, `Kārakāṃśa: ${ck.karakamsa_sign != null ? SIGN_SKT[ck.karakamsa_sign] : '—'} (${ck.karakamsa_note})`))),
    ar ? h('div', {}, h('h3', {}, 'Ārūḍha padas'), table(['Pada', 'Of bhāva', 'Lord', 'Lord in', 'Pada sign', 'Pada bhāva', '10th-from exception'],
      Object.entries(ar).sort((a, b) => a[1].house - b[1].house).map(([k, v]) => [k.replace('_', ' '), v.house, G[v.lord].en, SIGN_SKT[v.lord_sign], SIGN_SKT[v.pada_sign], v.pada_house, v.tenth_from_exception ? 'yes' : '']))) : null);
}
function renderYogas() {
  const b = body('yogas'); const y = val('vedic.yogas'); if (!b || !y) { hide('yogas'); return; }
  const list = (arr) => table(['', 'Name', 'Rule', 'Grahas'], arr.map(x => [x.present === true ? h('b', { style: 'color:var(--good)' }, '✓ present') : x.present === false ? h('span', { class: 'note' }, 'absent') : h('span', { class: 'note' }, 'not evaluable'),
    x.name, x.rule, (x.grahas || []).map(g => G[g] ? G[g].abbr : g).join(', ')]), { rowAttrs: i => ({ class: arr[i].present ? 'has' : '' }) });
  put(b, h('div', { class: 'tabs' }, h('span', { class: 'chip good' }, `${y.yogas.filter(x => x.present).length} yogas present`), h('span', { class: 'chip bad' }, `${y.doshas.filter(x => x.present).length} doshas present`)),
    h('h3', {}, 'Yogas'), list(y.yogas), h('h3', {}, 'Doshas'), list(y.doshas),
    h('p', { class: 'note' }, 'Each entry states the condition checked (rule pack vedic.yogas@0.2). Whether a condition holds is a derived fact; ESHANA does not say what it means.'));
}

/* ── transits and progressions ────────────────────────────────────────────────────────────────────────────────── */
function renderTransits() {
  const b = body('transits'); const tr = D.transits; if (!b || !tr || !tr.rows || !tr.rows.length) { hide('transits'); return; }
  const ref = tr.rows[tr.reference_index || 0], moonSign = GR.moon ? signOf(GR.moon.lon) : null;
  const rows = Object.entries(ref.longitudes).sort((a, b) => GRAHA_ORDER.indexOf(engineId(a[0])) - GRAHA_ORDER.indexOf(engineId(b[0]))).map(([k, l]) => { const id = engineId(k); return [nameCell(id), `${SIGN_SKT[signOf(l)]} ${inSign(l)}`, `${NAKSHATRA[nakOf(l)]} ${padaOf(l)}`,
    LAGNA_SIGN == null ? '—' : (signOf(l) - LAGNA_SIGN + 12) % 12 + 1, moonSign == null ? '—' : (signOf(l) - moonSign + 12) % 12 + 1, GR[id] ? `${SIGN_SKT[signOf(GR[id].lon)]} ${inSign(GR[id].lon)}` : '—']; });
  put(b, h('p', { class: 'lead' }, `Positions at ${fmtTime(ref.utc)} against the natal chart. Turn on "transits" on the wheel and press play to move through ${dateOnly(tr.rows[0].utc)} – ${dateOnly(tr.rows[tr.rows.length - 1].utc)}.`),
    table(['Graha', 'Transit', 'Nakshatra', 'Bhāva from lagna', 'From natal Moon', 'Natal'], rows));
}
function renderProgressions() {
  const b = body('progressions'); const pr = D.progressions; if (!b || !pr || !pr.rows || !pr.rows.length) { hide('progressions'); return; }
  const ids = Object.keys(pr.rows[0].longitudes).filter(k => ['sun', 'moon', 'mercury', 'venus', 'mars'].includes(k));
  const W = 1000, H = 220, svg = s('svg', { viewBox: `0 0 ${W} ${H + 24}`, role: 'img', 'aria-label': 'Secondary progressions by age' });
  for (let sg = 0; sg <= 12; sg++) s('line', { x1: 0, x2: W, y1: H - sg / 12 * H, y2: H - sg / 12 * H, stroke: 'var(--line)' }, svg);
  for (const id of ids) {
    let d = '', prev = null;
    pr.rows.forEach((r, i) => { const x = i / (pr.rows.length - 1) * W, l = r.longitudes[id], y = H - l / 360 * H; d += (prev != null && Math.abs(l - prev) > 180 ? 'M' : (i ? 'L' : 'M')) + `${x.toFixed(1)} ${y.toFixed(1)}`; prev = l; });
    s('path', { d, fill: 'none', stroke: G[id].dot, 'stroke-width': 2.2 }, svg);
  }
  for (let a = 0; a < pr.rows.length; a += 10) { const t = s('text', { x: a / (pr.rows.length - 1) * W, y: H + 16, 'font-size': 11, fill: 'var(--muted)' }, svg); t.textContent = `age ${a}`; }
  const every = pr.rows.filter((_, i) => i % 5 === 0);
  put(b, h('p', { class: 'lead' }, 'Secondary progressions (a day after birth for each year of life), tropical. Lines climb through the signs; a jump back is a pass through 0° Aries.'),
    h('div', { class: 'timeline' }, svg), h('div', { class: 'legend' }, ids.map(id => h('span', {}, h('i', { style: `background:${G[id].dot};height:6px` }), G[id].en))),
    h('details', {}, h('summary', {}, 'Every 5 years'), table(['Age', ...ids.map(id => G[id].en)], every.map((r, k) => [k * 5, ...ids.map(id => `${SIGN_EN[signOf(r.longitudes[id])].slice(0, 3)} ${inSign(r.longitudes[id])}`)]))));
}

/* ── Western ──────────────────────────────────────────────────────────────────────────────────────────────────── */
function renderWestern() {
  const b = body('western'); if (!b || !WES) { hide('western'); return; }
  const wgr = grahasOf(WES), asc = WES.houses && WES.houses.ascendant ? WES.houses.ascendant.longitude_deg : 0, ang = l => norm(270 - (l - asc));
  const svg = s('svg', { viewBox: '-440 -440 880 880', role: 'img', 'aria-label': 'Western chart wheel' });
  s('circle', { r: 425, fill: 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 2 }, svg);
  for (let i = 0; i < 12; i++) { s('path', { d: arc(355, 425, ang(i * 30 + 30), ang(i * 30 + 30) + 30), fill: PADA_FILL[ELEMENT[i % 4]], stroke: '#fff' }, svg);
    const [x, y] = pt(390, ang(i * 30 + 15)); const t = s('text', { class: 'lbl', x, y, 'font-size': 30, fill: '#1b1b1b' }, svg); t.textContent = GLYPH[i]; }
  const cusps = (WES.houses || {}).cusps_deg || [];
  cusps.forEach((c, k) => { const [x0, y0] = pt(125, ang(c)), [x1, y1] = pt(355, ang(c)); s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: 'var(--ink)', 'stroke-width': [0, 3, 6, 9].includes(k) ? 3 : 1 }, svg);
    const nx = cusps[(k + 1) % 12], mid = c + norm(nx - c) / 2, [x, y] = pt(140, ang(mid)); const t = s('text', { class: 'lbl', x, y, 'font-size': 14, fill: 'var(--muted)' }, svg); t.textContent = k + 1; });
  s('circle', { r: 125, fill: 'none', stroke: 'var(--line)' }, svg);
  const asp = WES.derived_attributes.filter(a => a.id.includes('.aspect.')), ag = s('g', { class: 'asp' }, svg);
  for (const a of asp) { const [b1, b2] = a.derived_from.map(engineId); if (!wgr[b1] || !wgr[b2]) continue; const [x0, y0] = pt(122, ang(wgr[b1].lon)), [x1, y1] = pt(122, ang(wgr[b2].lon));
    const ln = s('line', { x1: x0, y1: y0, x2: x1, y2: y1, stroke: ASPECT_COLOUR[a.value.aspect] }, ag); ln.addEventListener('mousemove', ev => tip(ev, `${G[b1].en} ${a.value.aspect} ${G[b2].en}`, [['Orb', a.value.orb_deg.toFixed(2) + '°'], ['', a.value.applying ? 'applying' : 'separating']])); ln.addEventListener('mouseleave', untip); }
  const items = GRAHA_ORDER.filter(g => wgr[g]).map(g => ({ g, a: ang(wgr[g].lon) })).sort((x, y) => x.a - y.a), last = [null, null, null];
  for (const it of items) { let lane = 0; for (; lane < 3; lane++) if (last[lane] == null || norm(it.a - last[lane]) >= 9) break; lane = Math.min(lane, 2); last[lane] = it.a; const [x, y] = pt(320 - lane * 44, it.a); marker(svg, it.g, x, y, 17, { retro: wgr[it.g].retro && !['rahu', 'ketu'].includes(it.g) }); }
  const bodies = GRAHA_ORDER.filter(g => wgr[g] && !['rahu', 'ketu'].includes(g));
  const grid = table(['', ...bodies.map(g => G[g].glyph)], bodies.map((g1, i) => [nameCell(g1), ...bodies.map((g2, j) => { if (j >= i) return ''; const a = asp.find(x => { const ids = x.derived_from.map(engineId); return ids.includes(g1) && ids.includes(g2); });
    return a ? { td: { style: `color:${ASPECT_COLOUR[a.value.aspect]};font-weight:700;text-align:center` }, v: { conjunction: '☌', opposition: '☍', square: '□', trine: '△', sextile: '⚹' }[a.value.aspect] + ' ' + a.value.orb_deg.toFixed(1) } : ''; })]));
  put(b, h('div', { class: 'grid2' }, h('div', { class: 'figure' }, svg), h('div', {}, h('h3', {}, 'Aspect grid'), grid,
    h('div', { class: 'legend' }, Object.entries(ASPECT_COLOUR).map(([k, c]) => h('span', {}, h('i', { style: `background:${c}` }), k))))),
    table(['Body', 'Longitude', 'Sign', 'In sign', 'House', 'R', 'Latitude'], GRAHA_ORDER.filter(g => wgr[g]).map(g => [nameCell(g), wgr[g].lon.toFixed(4) + '°', SIGN_EN[signOf(wgr[g].lon)], inSign(wgr[g].lon),
      (WA[`western.${g === 'rahu' ? 'mean_node' : g === 'ketu' ? 'south_node' : g}.house`] || { value: {} }).value.house || '—', wgr[g].retro ? 'R' : '', (wgr[g].lat || 0).toFixed(4)])));
}

/* ── Human Design: detailed or schematic BodyGraph ────────────────────────────────────────────────────────────── */
const HD_CENTER_GATES = { head: [64, 61, 63], ajna: [47, 24, 4, 17, 43, 11], throat: [62, 23, 56, 35, 12, 45, 33, 8, 31, 20, 16], g: [1, 13, 25, 46, 2, 15, 10, 7],
  heart: [21, 40, 26, 51], sacral: [5, 14, 29, 59, 9, 3, 42, 27, 34], spleen: [48, 57, 44, 50, 32, 28, 18], solar_plexus: [36, 22, 37, 6, 49, 55, 30], root: [53, 60, 52, 19, 39, 41, 58, 38, 54] };
const HD_CHANNELS = [[1, 8], [2, 14], [3, 60], [4, 63], [5, 15], [6, 59], [7, 31], [9, 52], [10, 20], [10, 34], [10, 57], [11, 56], [12, 22], [13, 33], [16, 48], [17, 62], [18, 58], [19, 49], [20, 34], [20, 57], [21, 45], [23, 43], [24, 61], [25, 51], [26, 44], [27, 50], [28, 38], [29, 46], [30, 41], [32, 54], [34, 57], [35, 36], [37, 40], [39, 55], [42, 53], [47, 64]];
const HD_POS = { head: [200, 50], ajna: [200, 135], throat: [200, 235], g: [200, 340], heart: [275, 380], sacral: [200, 470], spleen: [75, 440], solar_plexus: [325, 440], root: [200, 575] };
const HD_COL = { head: '#F5D547', ajna: '#7FB77E', throat: '#B88A5A', g: '#F5D547', heart: '#D9534F', sacral: '#D9534F', spleen: '#B88A5A', solar_plexus: '#B88A5A', root: '#B88A5A' };
function renderHD() {
  const b = body('human_design'); if (!b || !HD) { hide('human_design'); return; }
  const bg = (attrsOf(HD)['human_design.bodygraph'] || {}).value || {}, defined = new Set(bg.defined_centers || []), chans = new Set(bg.channels || []);
  const pers = HD.positions.personality.activations, des = HD.positions.design.activations;
  const gp = new Set(pers.map(a => a.gate)), gd = new Set(des.map(a => a.gate));
  const center = {}; for (const c in HD_CENTER_GATES) for (const g of HD_CENTER_GATES[c]) center[g] = c;
  const svg = s('svg', { viewBox: '0 0 400 640', role: 'img', 'aria-label': 'Human Design BodyGraph' });
  const gatePos = {};
  for (const c in HD_CENTER_GATES) { const [cx, cy] = HD_POS[c], gs = HD_CENTER_GATES[c], r = c === 'heart' ? 26 : 42;
    gs.forEach((g, k) => { const a = (k + .5) / gs.length * 2 * Math.PI - Math.PI / 2; gatePos[g] = [cx + Math.cos(a) * r * .85, cy + Math.sin(a) * r * .85]; }); }
  const detailed = S.hd === 'detailed';
  for (const [a, c] of HD_CHANNELS) {
    const key = `${Math.min(a, c)}-${Math.max(a, c)}`, on = chans.has(key) || chans.has(`${a}-${c}`) || chans.has(`${c}-${a}`);
    if (!detailed && !on) continue;
    const [x0, y0] = detailed ? gatePos[a] : HD_POS[center[a]], [x1, y1] = detailed ? gatePos[c] : HD_POS[center[c]];
    const half = (g, xa, ya, xb, yb) => { const col = gp.has(g) && gd.has(g) ? 'url(#both)' : gp.has(g) ? '#111' : gd.has(g) ? '#c8102e' : 'var(--line)';
      s('line', { x1: xa, y1: ya, x2: (xa + xb) / 2, y2: (ya + yb) / 2, stroke: col, 'stroke-width': on ? 7 : 4, 'stroke-linecap': 'round' }, svg); };
    half(a, x0, y0, x1, y1); half(c, x1, y1, x0, y0);
  }
  const defs = s('defs', {}, svg), pat = s('pattern', { id: 'both', width: 6, height: 6, patternUnits: 'userSpaceOnUse', patternTransform: 'rotate(45)' }, defs);
  s('rect', { width: 3, height: 6, fill: '#111' }, pat); s('rect', { x: 3, width: 3, height: 6, fill: '#c8102e' }, pat);
  const shapes = { head: 'up', ajna: 'down', throat: 'sq', g: 'dia', heart: 'up', sacral: 'sq', spleen: 'right', solar_plexus: 'left', root: 'sq' };
  for (const c in HD_POS) { const [x, y] = HD_POS[c], r = c === 'heart' ? 26 : 42, sh = shapes[c]; let d;
    if (sh === 'sq') d = `M${x - r} ${y - r}h${2 * r}v${2 * r}h${-2 * r}Z`; else if (sh === 'dia') d = `M${x} ${y - r - 8}L${x + r + 8} ${y}L${x} ${y + r + 8}L${x - r - 8} ${y}Z`;
    else if (sh === 'up') d = `M${x} ${y - r}L${x + r} ${y + r}L${x - r} ${y + r}Z`; else if (sh === 'down') d = `M${x - r} ${y - r}L${x + r} ${y - r}L${x} ${y + r}Z`;
    else if (sh === 'right') d = `M${x - r} ${y - r}L${x + r} ${y}L${x - r} ${y + r}Z`; else d = `M${x + r} ${y - r}L${x - r} ${y}L${x + r} ${y + r}Z`;
    const p = s('path', { d, fill: defined.has(c) ? HD_COL[c] : 'var(--panel)', stroke: 'var(--ink)', 'stroke-width': 2 }, svg);
    p.addEventListener('mousemove', ev => tip(ev, c.replace('_', ' '), [['State', defined.has(c) ? 'defined' : 'undefined'], ['Gates', HD_CENTER_GATES[c].join(', ')]])); p.addEventListener('mouseleave', untip);
    if (!detailed) { const t = s('text', { class: 'lbl', x, y, 'font-size': 11, fill: defined.has(c) ? '#1b1b1b' : 'var(--muted)' }, svg); t.textContent = c.replace('_', ' '); }
  }
  if (detailed) for (const g in gatePos) { const [x, y] = gatePos[g], act = gp.has(+g) || gd.has(+g);
    s('circle', { cx: x, cy: y, r: 7.5, fill: act ? (gp.has(+g) && gd.has(+g) ? 'url(#both)' : gp.has(+g) ? '#111' : '#c8102e') : 'var(--panel)', stroke: 'var(--muted)' }, svg);
    const t = s('text', { class: 'lbl', x, y, 'font-size': 7.5, 'font-weight': 700, fill: act ? '#fff' : 'var(--ink)' }, svg); t.textContent = g; }
  const pick = v => h('button', { class: 'tab', 'aria-selected': String(S.hd === v), onclick: () => { S.hd = v; save(); renderHD(); } }, v === 'detailed' ? 'Detailed (all gates and channels)' : 'Schematic (centres and defined channels)');
  const byBody = {}; for (const a of pers) byBody[a.body] = { p: a }; for (const a of des) (byBody[a.body] = byBody[a.body] || {}).d = a;
  put(b, h('div', { class: 'chips' }, ['type', 'authority', 'profile', 'definition'].map(k => h('span', { class: 'chip' }, k + ': ', h('b', {}, bg[k] || '—')))),
    h('div', { class: 'tabs no-print' }, pick('detailed'), pick('schematic')),
    h('div', { class: 'grid2' }, h('div', { class: 'figure', style: 'max-width:420px' }, svg), h('div', {},
      facts([['Personality (UTC)', HD.positions.personality.instant_utc], ['Design (UTC)', HD.positions.design.instant_utc], ['Channels', (bg.channels || []).join(', ')], ['Defined', (bg.defined_centers || []).join(', ')], ['Undefined', (bg.undefined_centers || []).join(', ')]]),
      table(['Body', 'Personality', 'Design'], Object.entries(byBody).map(([k, v]) => [k.replace('_', ' '), v.p ? `${v.p.gate}.${v.p.line}` : '', v.d ? h('b', { style: 'color:#c8102e' }, `${v.d.gate}.${v.d.line}`) : ''])),
      h('p', { class: 'note' }, 'Black: personality · red: design · striped: both. The drawing is schematic; positions of gates on a centre are for reading, not a reproduction of any branded chart.'))));
}

/* ── numerology, matching, other, provenance, raw ─────────────────────────────────────────────────────────────── */
function renderNumerology() {
  const b = body('numerology'); if (!b || !NUM) { hide('numerology'); return; }
  put(b, h('div', { class: 'grid3' }, NUM.derived_attributes.map(a => h('div', { class: 'mini' }, h('h3', {}, a.id), facts(Object.entries(a.value).filter(([k]) => k !== 'letters').map(([k, v]) =>
    [k.replaceAll('_', ' '), v && typeof v === 'object' && 'reduced' in v ? `${v.reduced} (${v.compound})` : typeof v === 'object' ? JSON.stringify(v) : String(v)])), h('p', { class: 'note' }, label(a.epistemic), ' ', a.rule_version)))));
}
function renderMatching() {
  const b = body('matching'); if (!b || !MATCH) { hide('matching'); return; }
  const k = MATCH.ashtakoota;
  put(b, h('div', { class: 'chips' }, h('span', { class: 'chip' }, 'Total: ', h('b', {}, `${k.total} / ${k.max}`)), k.nadi_dosha ? h('span', { class: 'chip bad' }, 'nadi dosha') : null, k.bhakoot_dosha ? h('span', { class: 'chip bad' }, 'bhakoot dosha') : null),
    table(['Kūṭa', 'A', 'B', 'Score', 'Max', 'Rule'], k.kootas.map(x => [x.koota.replace('_', ' '), typeof x.a === 'object' ? JSON.stringify(x.a) : x.a, typeof x.b === 'object' ? JSON.stringify(x.b) : x.b, x.score, x.max, x.note])),
    facts([['A Moon', `${k.a_moon.nakshatra} ${k.a_moon.pada} · ${SIGN_SKT[k.a_moon.sign]}`], ['B Moon', `${k.b_moon.nakshatra} ${k.b_moon.pada} · ${SIGN_SKT[k.b_moon.sign]}`],
      ['Manglik A', (MATCH.manglik.a || []).filter(x => x.present).map(x => x.id).join(', ') || 'none'], ['Manglik B', (MATCH.manglik.b || []).filter(x => x.present).map(x => x.id).join(', ') || 'none'], ['Rule pack', k.rule_version]]));
}
function renderOther() {
  const b = body('attributes'); if (!b) return;
  const shown = /^(vedic\.[a-z_]+\.(rashi|nakshatra|bhava|d\d+)|vedic\.(lagna)\..*|vedic\.(dasha|strength|relationships|drishti|ashtakavarga|jaimini|kp|yogas|panchanga|sade_sati).*|western\..*|human_design\.bodygraph|numerology\..*)$/;
  const rows = []; for (const [name, env] of Object.entries(ENV)) for (const a of (env && env.derived_attributes) || []) if (!shown.test(a.id)) rows.push([name, a.id, JSON.stringify(a.value).slice(0, 200), a.epistemic, a.rule_version]);
  if (!rows.length) { hide('attributes'); return; } put(b, table(['Envelope', 'Id', 'Value', 'Label', 'Rule'], rows));
}
function renderProvenance() {
  const b = body('provenance'); if (!b) return;
  put(b, h('div', { class: 'grid3' }, Object.entries(ENV).filter(([, e]) => e && e.provenance).map(([n, e]) => h('div', { class: 'mini' }, h('h3', {}, n),
    facts([...Object.entries(e.calculation || {}).map(([k, v]) => [k.replaceAll('_', ' '), v == null ? '—' : String(v)]), ...Object.entries(e.provenance).filter(([k]) => !['assumptions', 'rulesets'].includes(k)).map(([k, v]) => [k.replaceAll('_', ' '), String(v)]),
      ['rule packs', (e.provenance.rulesets || []).join(', ')]]),
    (e.provenance.assumptions || []).length ? h('details', {}, h('summary', {}, `${e.provenance.assumptions.length} assumptions`), h('ul', {}, e.provenance.assumptions.map(a => h('li', {}, a)))) : null,
    (e.warnings || []).length ? h('ul', { class: 'warn' }, e.warnings.map(w => h('li', {}, typeof w === 'string' ? w : `${w.code}: ${w.message}`))) : null))));
}
function renderRaw() {
  const b = body('raw'); if (!b) return;
  const raw = JSON.stringify(D, null, 1);
  put(b, h('details', {}, h('summary', {}, `Show JSON (${raw.length.toLocaleString()} characters)`), h('pre', { class: 'raw' }, raw)));
}

/* ── settings drawer ──────────────────────────────────────────────────────────────────────────────────────────── */
const SECTIONS = ['summary', 'chakra', 'charts', 'planets', 'nakshatra', 'vargas', 'panchanga', 'dasha', 'strength', 'ashtakavarga', 'kp', 'jaimini', 'yogas', 'transits', 'progressions', 'western', 'human_design', 'numerology', 'matching', 'attributes', 'provenance', 'raw'];
function renderDrawer() {
  const dr = $('#drawer'); if (!dr) return;
  S.hidden = S.hidden || {};
  const sec = SECTIONS.filter(id => document.getElementById(id)).map(id => h('label', {}, h('input', { type: 'checkbox', checked: !S.hidden[id], onchange: e => { S.hidden[id] = !e.target.checked; save(); applyHidden(); } }),
    document.getElementById(id).querySelector('h2').textContent));
  const radio = (name, v, t, key) => h('label', {}, h('input', { type: 'radio', name, checked: S[key] === v, onchange: () => { S[key] = v; save(); renderAll(); } }), t);
  dr.replaceChildren(h('h2', {}, 'Show and style'), h('p', { class: 'note' }, 'Choices are kept in this browser only.'),
    h('fieldset', {}, h('legend', {}, 'Sections'), sec),
    h('fieldset', {}, h('legend', {}, 'Planet icons'), radio('ic', 'sphere', 'Textured spheres', 'icons'), radio('ic', 'glyph', 'Astronomical glyphs', 'icons'), radio('ic', 'badge', 'Lettered badges', 'icons')),
    h('fieldset', {}, h('legend', {}, 'Charts'), radio('ch', 'south', 'South Indian', 'chart'), radio('ch', 'north', 'North Indian', 'chart'), radio('ch', 'both', 'Both side by side', 'chart')),
    h('fieldset', {}, h('legend', {}, 'BodyGraph'), radio('hd', 'detailed', 'Detailed', 'hd'), radio('hd', 'schematic', 'Schematic', 'hd')),
    h('fieldset', {}, h('legend', {}, 'Motion'), h('label', {}, h('input', { type: 'checkbox', checked: S.animate, onchange: e => { S.animate = e.target.checked; save(); renderAll(); } }), 'Animate charts')),
    h('button', { class: 'btn', onclick: () => { S = JSON.parse(JSON.stringify(DEFAULTS)); save(); renderAll(); renderDrawer(); applyHidden(); } }, 'Reset to defaults'), ' ',
    h('button', { class: 'btn primary', onclick: () => dr.classList.remove('open') }, 'Done'));
}
function applyHidden() { for (const id of SECTIONS) { const el = document.getElementById(id); if (el && el.dataset.empty !== '1') el.hidden = !!(S.hidden || {})[id]; } }
function renderAll() {
  const steps = [renderSummary, renderChakra, renderCharts, renderPlanets, renderNakshatraTable, renderVargas, renderPanchanga, renderDasha, renderStrength, renderAshtakavarga, renderKP, renderJaimini,
    renderYogas, renderTransits, renderProgressions, renderWestern, renderHD, renderNumerology, renderMatching, renderOther, renderProvenance, renderRaw];
  for (const f of steps) { try { f(); } catch (e) { console.error(f.name, e); } }
  for (const id of SECTIONS) { const el = document.getElementById(id); if (el && el.hidden) el.dataset.empty = '1'; }
  applyHidden();
  $$('nav.toc a').forEach(a => { const t = document.getElementById(a.getAttribute('href').slice(1)); a.hidden = !t || t.hidden; });
}

/* ── boot ─────────────────────────────────────────────────────────────────────────────────────────────────────── */
(function boot() {
  const root = document.documentElement;
  try { const t = localStorage.getItem('eshana-theme'); if (t) root.setAttribute('data-theme', t); } catch (e) { /* ignore */ }
  $('#theme').addEventListener('click', () => { const cur = root.getAttribute('data-theme') || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    const nx = cur === 'dark' ? 'light' : 'dark'; root.setAttribute('data-theme', nx); try { localStorage.setItem('eshana-theme', nx); } catch (e) { /* ignore */ } });
  $('#print').addEventListener('click', () => { $$('details').forEach(d => d.open = true); window.print(); });
  $('#settings').addEventListener('click', () => { renderDrawer(); $('#drawer').classList.add('open'); });
  $('#json').addEventListener('click', () => { const a = h('a', { href: URL.createObjectURL(new Blob([JSON.stringify(D, null, 1)], { type: 'application/json' })), download: 'eshana-chart.json' }); document.body.append(a); a.click(); a.remove(); });
  $('#svg').addEventListener('click', () => { const svg = $('#chakra svg'); if (!svg) return; const src = new XMLSerializer().serializeToString(svg);
    const a = h('a', { href: URL.createObjectURL(new Blob([src], { type: 'image/svg+xml' })), download: 'eshana-chakra.svg' }); document.body.append(a); a.click(); a.remove(); });
  $('#copy').addEventListener('click', async () => {
    const lines = [`Chart data computed by ESHANA (${(VED || WES || {}).provenance ? (VED || WES).provenance.engine_version : ''}). Values are calculated or tradition rules, not interpretations.`];
    if (VED) { lines.push(`Lagna: ${ASC != null ? SIGN_SKT[signOf(ASC)] + ' ' + inSign(ASC) : 'unknown'}`); for (const g of GRAHA_ORDER) if (GR[g]) lines.push(`${G[g].en}: ${SIGN_SKT[signOf(GR[g].lon)]} ${inSign(GR[g].lon)}, ${NAKSHATRA[nakOf(GR[g].lon)]} pada ${padaOf(GR[g].lon)}${GR[g].retro ? ', retrograde' : ''}`); }
    const vim = val('vedic.dasha.vimshottari'); if (vim && vim.current) lines.push('Current Vimśottarī: ' + vim.current.periods.map(p => `${p.level} ${p.lord}`).join(' / '));
    try { await navigator.clipboard.writeText(lines.join('\n')); $('#copy').textContent = '✓ Copied'; } catch (e) { $('#copy').textContent = 'Copy blocked'; }
  });
  renderAll();
  // deep links: #focus=rashi-4, #focus=nak-20, #focus=pada-81, #graha=moon, #theme=dark
  const q = new URLSearchParams(location.hash.slice(1));
  if (q.get('theme')) root.setAttribute('data-theme', q.get('theme'));
  const f = (q.get('focus') || '').match(/^(rashi|nak|pada|dres)-(\d+)$/);
  if (f && VED) focus(f[1], +f[2]);
  const g = q.get('graha'); if (g && GR[g]) showPanel(grahaDetail(g), [['⟲ Chakra', resetFocus]]);
})();
