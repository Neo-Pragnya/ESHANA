/* ESHANA Studio: Annual and daily. Varṣaphala (vedic.varshaphala) and pañca pakṣī (vedic.panch_pakshi).
   Every number comes from the engine; this file only draws it. */

SIGIL.annual = '<circle cx="12" cy="12" r="9.5" class="ring"/><path d="M12 2.5A9.5 9.5 0 0 1 21.5 12" class="ray draw"/><g class="spin"><circle cx="12" cy="2.5" r="1.6" class="sat"/></g><circle cx="12" cy="12" r="3" class="core"/>';
SIGIL.pakshi = '<path d="M4 14c3-1 5-4 6-8 1 4 3 6 6 7-3 1-5 3-6 6-1-3-3-5-6-5z" class="ring"/><circle cx="12" cy="9" r="1.2" class="core"/><path d="M12 15v5M9 21h6" class="ray draw"/>';

const ANN_ROLE = {
  muntha_pati: ['Muntha pati', 'Lord of the sign the muntha has reached: the natal lagna moved one sign per completed year.'],
  janma_lagna_pati: ['Janma lagna pati', 'Lord of the natal lagna sign.'],
  varsha_lagna_pati: ['Varṣa lagna pati', 'Lord of the sign rising at the solar return.'],
  trirashi_pati: ['Trirāśi pati', 'From the Tājika trirāśi table, by the varṣa lagna sign and whether the return falls by day or by night.'],
  dinaratri_pati: ['Dinarātri pati', 'Day return: lord of the Sun’s sign; night return: lord of the Moon’s sign.'],
};
const ANN_PARTS = [['griha', 'Gṛha', 30], ['uchcha', 'Uchcha', 20], ['hudda', 'Hudda', 15], ['drekkana', 'Drekkāṇa', 10], ['navamsha', 'Navāṃśa', 5]];
const gname = g => (G[g] || G.lagna).en;
const gab = g => (G[g] || G.lagna).abbr;
const signDeg = l => `${SIGN_SKT[signOf(l)]} ${inSignDM(l)}`;
const hoverable = (el, title, rows, sub) => { el.addEventListener('mousemove', ev => tip(ev, title, rows, sub)); el.addEventListener('mouseleave', untip); return el; };

/* a South Indian chart of any set of sidereal positions (here the varṣa chart) */
function annualSouth(pos, lagnaLon, extra = {}) {
  const W = 120, H = 96, svg = s('svg', { viewBox: `-2 -2 ${W * 4 + 4} ${H * 4 + 4}`, role: 'img', 'aria-label': 'Varṣa chart, South Indian style', class: 'chart south annual-chart' });
  const lag = signOf(lagnaLon), by = {};
  for (const [g, l] of Object.entries(pos)) (by[signOf(l)] = by[signOf(l)] || []).push(g);
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg, x = c * W, y = r * H, g = s('g', { class: 'cell' + (sign === lag ? ' lag' : '') }, svg);
    s('rect', { x, y, width: W, height: H, class: 'cellbg' }, g);
    const nm = s('text', { x: x + W - 5, y: y + 13, 'text-anchor': 'end', 'font-size': 11, fill: 'var(--muted)' }, g); nm.textContent = SIGN_AB[sign];
    const house = (sign - lag + 12) % 12 + 1;
    const hn = s('text', { x: x + 5, y: y + H - 6, 'font-size': 10, fill: 'var(--muted)' }, g); hn.textContent = house;
    if (sign === lag) { const t = s('text', { x: x + 5, y: y + 13, 'font-size': 11, 'font-weight': 700, fill: 'var(--accent)' }, g); t.textContent = `Asc ${inSignDM(lagnaLon)}`; }
    const list = (by[sign] || []).sort((a, b) => GRAHA_ORDER.indexOf(a) - GRAHA_ORDER.indexOf(b));
    if (extra.muntha != null && signOf(extra.muntha) === sign) list.push('muntha');
    list.forEach((id, i) => {
      const col = i % 2, row = Math.floor(i / 2), tx = x + 8 + col * (W / 2 - 4), ty = y + 32 + row * 17;
      const isM = id === 'muntha', colr = isM ? 'var(--accent2)' : (G[id] || G.lagna).dot;
      const t = s('text', { x: tx, y: ty, 'font-size': 13, 'font-weight': 700, fill: colr, class: 'ann-g ' + (extra.lord === id ? 'year-lord' : ''), style: `--i:${i}` }, g);
      t.textContent = isM ? 'Mu' : gab(id);
      const l = isM ? extra.muntha : pos[id];
      hoverable(t, isM ? 'Muntha' : gname(id), [['Position', signDeg(l)], ['Nakshatra', NAKSHATRA[nakOf(l)]], ['Varṣa bhāva', house], extra.lord === id ? ['Role', 'year lord'] : null], 'tradition_rule · varṣa chart');
    });
  }
  return svg;
}

/* pañcavargīya bala as stacked bars that grow in (maximum total 20 = 80 / 4) */
function annualBars(bala, lord) {
  const wrap = h('div', { class: 'ann-bars' });
  SEVEN.filter(g => bala[g]).sort((a, b) => bala[b].total - bala[a].total).forEach((g, i) => {
    const b = bala[g], row = h('div', { class: 'ann-row' + (g === lord ? ' is-lord' : ''), style: `--i:${i}` });
    const track = h('div', { class: 'ann-track' });
    for (const [k, nm, max] of ANN_PARTS) {
      const seg = h('span', { class: 'ann-seg seg-' + k, style: `width:${(b[k] / 80 * 100).toFixed(2)}%` });
      const d = (b.detail || {})[k] || {};
      hoverable(seg, `${gname(g)} · ${nm} bala`, [['Value', `${fix(b[k])} of ${max}`], ['Division lord', d.lord ? gname(d.lord) : null], ['Relationship', d.relation], k === 'uchcha' ? ['From debilitation', fix(d.degrees_from_debilitation, 1) + '°'] : null], 'tradition_rule · Tājika pañcavargīya bala');
      track.append(seg);
    }
    row.append(h('span', { class: 'ann-name' }, glink(g)), track, h('b', { class: 'ann-tot' }, fix(b.total)));
    wrap.append(row);
  });
  const legend = h('div', { class: 'ann-legend' }, ANN_PARTS.map(([k, nm, max]) => h('span', {}, h('i', { class: 'sw seg-' + k }), `${nm} (max ${max})`)));
  return h('div', {}, wrap, legend);
}

let ANN_VIEW = null;
page('annual', () => 'Varṣaphala', 'Time', (body) => {
  const v = val('vedic.varshaphala');
  if (!v) return put(body, missing('Varṣaphala', 'It needs an exact birth time and a place.'));
  const years = [['current', v.current], ['birth_year', v.birth_year]].filter(([, y]) => y);
  if (!ANN_VIEW || !years.some(([k]) => k === ANN_VIEW)) ANN_VIEW = years[0][0];
  const y = (years.find(([k]) => k === ANN_VIEW) || years[0])[1];
  const lord = y.year_lord.graha;
  const pick = h('div', { class: 'seg-toggle ann-pick' }, years.map(([k, yy]) => h('button', { class: 'btn' + (k === ANN_VIEW ? ' primary' : ''), 'data-year': k, onclick: () => { ANN_VIEW = k; route(); } },
    k === 'current' ? `Year ${yy.completed_years + 1} (running now)` : 'Birth year (year 0)')));
  const stats = h('div', { class: 'stats' },
    stat('Solar return', fmtTime(y.return.local).slice(0, 16), y.day_return ? 'by day' : 'by night'),
    stat('Completed years', String(y.completed_years), `year ${y.completed_years + 1} of life`),
    stat('Varṣa lagna', signDeg(y.varsha_lagna.longitude_deg), `lord ${gname(y.varsha_lagna.lord)}`),
    stat('Muntha', signDeg(y.muntha.longitude_deg), `varṣa bhāva ${y.muntha.house_from_varsha_lagna}`),
    h('div', { class: 'stat ann-lord-stat' }, h('div', { class: 'stat-l' }, 'Year lord (varṣeśa)'), h('div', { class: 'stat-v' }, glink(lord)), h('div', { class: 'stat-s' }, `pañcavargīya ${fix(y.year_lord.total)}`)));
  const offices = table(['Office', 'Graha', 'Pañcavargīya', 'Aspects varṣa lagna', 'Varṣa bhāva'], y.office_bearers.map(o => {
    const role = ANN_ROLE[o.role] || [o.role, ''];
    const nm = hoverable(h('span', { class: 'ann-office' }, role[0]), role[0], [['Why', o.why], ['Graha', gname(o.graha)], ['Total', fix(o.total)]], role[1]);
    const g = h('span', { class: o.graha === lord ? 'ann-is-lord' : '' }, glink(o.graha), o.graha === lord ? ' ★' : '');
    return [nm, g, fix(o.total), o.aspects_varsha_lagna ? 'yes' : 'no (2/6/8/12)', o.house_from_varsha_lagna];
  }));
  const chart = annualSouth(y.positions, y.varsha_lagna.longitude_deg, { muntha: y.muntha.longitude_deg, lord });
  const conv = v.conventions || {};
  put(body, pageHead('annual', 'Varṣaphala (Tājika annual chart)', 'The chart for the instant the Sun returns to its natal sidereal longitude, with the muntha, the five office-bearers, the year lord and the pañcavargīya strengths.'),
    pick, stats,
    h('div', { class: 'grid2' },
      card(`Varṣa chart · ${dateOnly(y.return.local)}`, h('div', { class: 'figure narrow' }, chart), note('Hover a graha for its varṣa position. Mu = muntha; the year lord glows.')),
      card('Office-bearers (pañcādhikārīs)', offices, note(label('tradition_rule'), ' ', y.year_lord.why, '.'))),
    card('Pañcavargīya bala', annualBars(y.pancavargiya_bala, lord), note(label('tradition_rule'), ' ', conv.pancavargiya || '')),
    card('How this is computed', facts(Object.entries(conv).map(([k, t]) => [k.replaceAll('_', ' '), t])), note('Rule pack: ', v.rule_pack || 'vedic.annual', '. Year 0 is the birth chart itself.')),
    card('Reading', note(label('interpretation'), ' In Tājika practice (after Nīlakaṇṭha’s Tājika Nīlakaṇṭhī, paraphrased) the year lord colours the year: a strong, well-placed lord is read as an easier year, a weak one as a year that asks for effort; the bhāva the muntha reaches is the area of life the year turns towards. ESHANA computes the factors; it does not judge them.')));
}, { blurb: 'solar return, muntha, year lord' });

/* ── pañca pakṣī ──────────────────────────────────────────────────────────────────────────────────────────────── */
const BIRD = {
  vulture: { en: 'Vulture', ta: 'Kazhugu', c: '#b0805a' },
  owl: { en: 'Owl', ta: 'Ānthai', c: '#8c7bd6' },
  crow: { en: 'Crow', ta: 'Kākam', c: '#5d6b7a' },
  cock: { en: 'Cock', ta: 'Kōḻi', c: '#e0564f' },
  peacock: { en: 'Peacock', ta: 'Mayil', c: '#2fa7a0' },
};
const ACT = { ruling: 'Ruling', eating: 'Eating', walking: 'Walking', sleeping: 'Sleeping', dying: 'Dying' };
/* simple bird glyphs, drawn here: body, head, beak and one mark for each bird */
function birdGlyph(b, size = 56) {
  const c = (BIRD[b] || BIRD.crow).c, svg = s('svg', { viewBox: '0 0 64 64', width: size, height: size, class: 'bird bird-' + b, role: 'img', 'aria-label': (BIRD[b] || {}).en || b });
  const P = (d, cls, fill) => s('path', { d, class: cls, fill: fill || c }, svg);
  if (b === 'peacock') { for (let i = 0; i < 5; i++) { const a = (110 + i * 35) * DEG; s('circle', { cx: 24 + 17 * Math.cos(a), cy: 36 - 17 * Math.sin(a), r: 5, fill: 'none', stroke: c, 'stroke-width': 2.5, class: 'feather', style: `--i:${i}` }, svg); } }
  if (b === 'vulture') P('M8 30 Q20 14 32 28 Q44 14 56 30 Q44 26 32 34 Q20 26 8 30Z', 'wing');
  P('M20 44 Q22 28 36 28 Q48 30 46 44 Q40 52 28 52 Q20 50 20 44Z', 'body');
  s('circle', { cx: 42, cy: 22, r: b === 'owl' ? 10 : 7, fill: c, class: 'head' }, svg);
  if (b === 'owl') {
    for (const ex of [38, 46]) { s('circle', { cx: ex, cy: 21, r: 3, fill: '#fff' }, svg); s('circle', { cx: ex, cy: 21, r: 1.4, fill: '#222', class: 'pupil' }, svg); }
    P('M40 26 L42 30 L44 26Z', 'beak', '#e8b13a');
  } else {
    s('circle', { cx: 44, cy: 20, r: 1.6, fill: '#fff' }, svg);
    P(b === 'vulture' ? 'M48 20 Q56 22 52 27 L48 24Z' : 'M48 20 L56 23 L48 25Z', 'beak', '#e8b13a');
  }
  if (b === 'cock') P('M36 14 Q38 6 41 13 Q43 5 46 13 Q49 8 48 16Z', 'comb', '#c43a33');
  if (b === 'crow') P('M20 44 L6 50 L20 48Z', 'tail');
  s('path', { d: 'M30 52 L28 60 M36 52 L38 60', stroke: c, 'stroke-width': 2.5, fill: 'none', class: 'legs' }, svg);
  return svg;
}
const birdName = b => (BIRD[b] || {}).en || b;
const birdChip = (b, rel) => h('span', { class: 'chip bird-chip rel-' + (rel || 'none') }, birdGlyph(b, 22), birdName(b));
const localHM = st => (st && st.local ? st.local.slice(11, 19) : '—');

/* the day and night as coloured bands: ten yāmas, each split into its five sub-periods, the current one marked */
function pakshiTimeline(tb, onPick) {
  const W = 1000, H = 64, y0 = 22, ys = tb.yamas, t0 = ys[0].start.jd_ut, t1 = ys[ys.length - 1].end.jd_ut, x = t => (t - t0) / (t1 - t0) * W;
  const svg = s('svg', { viewBox: `0 0 ${W} ${y0 + H + 26}`, role: 'img', 'aria-label': 'Pañca pakṣī day and night timeline', class: 'pakshi-tl' });
  const mid = x(ys[5].start.jd_ut);
  s('rect', { x: mid, y: 0, width: W - mid, height: y0 + H + 4, class: 'pk-night' }, svg);
  for (const [lbl, xx] of [['day', 4], ['night', mid + 4]]) { const t = s('text', { x: xx, y: 14, 'font-size': 12, fill: 'var(--muted)' }, svg); t.textContent = lbl; }
  ys.forEach((yy, i) => {
    const a = x(yy.start.jd_ut), b = x(yy.end.jd_ut);
    const g = s('g', { class: 'pk-yama act-' + yy.activity + (yy.current ? ' current' : ''), style: `--i:${i}`, 'data-yama': i }, svg);
    for (const sp of yy.sub_periods) {
      const sa = x(sp.start.jd_ut), sb = x(sp.end.jd_ut);
      s('rect', { x: sa, y: y0 + H * .64, width: Math.max(.5, sb - sa - .5), height: H * .36, class: 'pk-sub act-' + sp.activity }, g);
    }
    s('rect', { x: a + .5, y: y0, width: Math.max(1, b - a - 1), height: H * .6, rx: 5, class: 'pk-band act-' + yy.activity }, g);
    const t = s('text', { x: (a + b) / 2, y: y0 + H * .36, 'text-anchor': 'middle', 'font-size': 12, 'font-weight': 700, class: 'pk-lbl' }, g); t.textContent = ACT[yy.activity] || yy.activity;
    const tt = s('text', { x: a + 2, y: y0 + H + 16, 'font-size': 10, fill: 'var(--muted)' }, g); tt.textContent = localHM(yy.start).slice(0, 5);
    const rows = [['From', localHM(yy.start)], ['To', localHM(yy.end)]].concat(yy.birds.map(bd => [birdName(bd.bird), `${ACT[bd.activity] || bd.activity} (${bd.relation})`]));
    hoverable(g, `${yy.half === 'day' ? 'Day' : 'Night'} yāma ${yy.yama}: ${ACT[yy.activity] || yy.activity}`, rows, 'tradition_rule · pañca pakṣī');
    g.addEventListener('click', () => onPick(i));
    if (yy.current) s('line', { x1: (a + b) / 2, x2: (a + b) / 2, y1: y0 - 6, y2: y0 + H + 2, class: 'now pk-now' }, svg);
  });
  return h('div', { class: 'timeline pk-wrap' }, svg);
}

let PK_TABLE = null, PK_YAMA = null;
page('pakshi', () => 'Pañca pakṣī', 'Time', (body) => {
  const v = val('vedic.panch_pakshi');
  if (!v) return put(body, missing('Pañca pakṣī', 'It needs a place (for sunrise) and the Moon.'));
  const tables = [['reference', v.reference_day_table, 'Today'], ['birth', v.birth_day_table, 'Birth day']].filter(([, t]) => t && t.yamas);
  const bd = v.birth_day || {};
  const tile = (b, lbl, size, cls) => h('div', { class: 'pk-bird ' + (cls || '') }, birdGlyph(b, size), h('div', {}, h('div', { class: 'stat-l' }, lbl), h('div', { class: 'pk-name' }, birdName(b))));
  const self = tile(v.birth_bird, 'Birth bird (self)', 96, 'self');
  self.append(h('div', { class: 'note' }, `${BIRD[v.birth_bird].ta} · Moon in ${v.nakshatra.name}, ${v.paksha === 'krishna' ? 'dark half' : 'bright half'}`));
  hoverable(self, birdName(v.birth_bird), [['Nakshatra', v.nakshatra.name], ['Pakṣa', v.paksha], ['Friends', v.friends.map(birdName).join(', ')], ['Enemies', v.enemies.map(birdName).join(', ')]], (v.conventions || {}).birth_bird);
  const head = h('div', { class: 'pk-head' }, self,
    bd.ruling_bird ? hoverable(tile(bd.ruling_bird, 'Ruling bird of the birth day', 64), 'Ruling bird', [['Bird', birdName(bd.ruling_bird)], ['Night ruler', birdName(bd.night_ruling_bird)]], (v.conventions || {}).birth_day) : null,
    bd.dying_bird ? hoverable(tile(bd.dying_bird, 'Dying bird of the birth day', 64, 'dying'), 'Dying bird', [['Bird', birdName(bd.dying_bird)]], (v.conventions || {}).birth_day) : null);
  const rel = h('div', { class: 'chips pk-rel' }, h('span', { class: 'note' }, 'Friends: '), v.friends.map(b => birdChip(b, 'friend')), h('span', { class: 'note' }, ' Enemies: '), v.enemies.map(b => birdChip(b, 'enemy')));
  const ph = pageHead('pakshi', 'Pañca pakṣī (five birds)', 'The Tamil five-bird system: the birth bird from the Moon’s nakṣatra and the pakṣa, and what each bird does through the five yāmas of the day and of the night.');
  if (!tables.length) return put(body, ph, head, rel, note('No yāma table: the Sun does not rise or set here on that date.'));
  if (!PK_TABLE || !tables.some(([k]) => k === PK_TABLE)) PK_TABLE = tables[0][0];
  const tb = tables.find(([k]) => k === PK_TABLE)[1];
  const pickT = h('div', { class: 'seg-toggle pk-pick' }, tables.map(([k, t, nm]) => h('button', { class: 'btn' + (k === PK_TABLE ? ' primary' : ''), 'data-table': k, onclick: () => { PK_TABLE = k; PK_YAMA = null; route(); } }, `${nm} · ${dateOnly(t.sunrise.local)}`)));
  const cur = tb.yamas.findIndex(y => y.current);
  const detail = h('div', { class: 'pk-detail' });
  const show = i => {
    PK_YAMA = i; const yy = tb.yamas[i];
    put(detail, card(`${yy.half === 'day' ? 'Day' : 'Night'} yāma ${yy.yama} · ${localHM(yy.start)}–${localHM(yy.end)} · ${birdName(v.birth_bird)} ${(ACT[yy.activity] || '').toLowerCase()}`,
      h('div', { class: 'grid2' },
        table(['Bird', 'Activity this yāma', 'Relation'], yy.birds.map(b => [birdChip(b.bird, b.relation), h('span', { class: 'pk-act act-' + b.activity }, ACT[b.activity] || b.activity), b.relation])),
        table(['Sub-period', 'Bird', 'Activity', 'Relation'], yy.sub_periods.map(sp => [`${localHM(sp.start)}–${localHM(sp.end)}`, birdChip(sp.bird, sp.relation), h('span', { class: 'pk-act act-' + sp.activity }, ACT[sp.activity] || sp.activity), sp.relation])))));
    body.querySelectorAll('.pk-yama').forEach(el => el.classList.toggle('picked', +el.dataset.yama === i));
  };
  const legend = h('div', { class: 'ann-legend' }, Object.entries(ACT).map(([k, nm]) => h('span', {}, h('i', { class: 'sw act-' + k }), nm)));
  put(body, ph, head, rel, pickT,
    card(`${tb.vara} · ${tb.paksha === 'krishna' ? 'dark half' : 'bright half'} · sunrise ${localHM(tb.sunrise)}, sunset ${localHM(tb.sunset)}`, pakshiTimeline(tb, show), legend,
      note(label('tradition_rule'), ` Top band: the ${birdName(v.birth_bird)}’s activity in each yāma; lower strip: its sub-periods. Day ruler ${birdName(tb.day_ruling_bird)}, night ruler ${birdName(tb.night_ruling_bird)}, dying bird ${birdName(tb.dying_bird)}. Hover a yāma for every bird’s activity; click it for the sub-periods.`)),
    detail,
    card('How this is computed', facts(Object.entries(v.conventions || {}).map(([k, t]) => [k.replaceAll('_', ' '), t])), note('Source: ', v.source || '', '. Printed tables differ between almanacs; ESHANA names its choices.')),
    card('Reading', note(label('interpretation'), ' In the tradition (paraphrased from Tamil almanac practice) the five activities rank from strongest to weakest as ruling, eating, walking, sleeping and dying; work is begun while one’s own bird rules or eats and put off while it sleeps or dies, above all on the dying bird’s day. ESHANA shows the periods; it does not advise.')));
  show(PK_YAMA != null && PK_YAMA < tb.yamas.length ? PK_YAMA : (cur >= 0 ? cur : 0));
}, { blurb: 'five birds, day and night' });
