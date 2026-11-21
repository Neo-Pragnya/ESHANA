/* ESHANA Studio: Auxiliary analysis. Sahams, avakahaḍā chakra, male and female rāśis, mṛtyu bhāga, ghāta chakra,
   vedhasthāna, jīva and deha, śodhana. Every value comes from the engine (val('vedic.aux.…')); the page only lays it
   out, and every number carries a hover explaining the rule that produced it. */

SIGIL.aux = '<rect x="3" y="3" width="18" height="18" rx="2" class="ring"/><path d="M9 3V21M15 3V21M3 9H21M3 15H21" class="ray faint"/>'
  + '<circle cx="6" cy="6" r="1.4" class="core tw"/><circle cx="18" cy="12" r="1.4" class="core tw d2"/><circle cx="12" cy="18" r="1.4" class="core tw d3"/><path d="M6 6L18 12L12 18" class="ray draw dash"/>';

const AUX_TABS = [['sahams', 'Sahams'], ['avakahada', 'Avakahaḍā'], ['gender', 'Male/female'], ['mrityu', 'Mṛtyu bhāga'],
  ['ghata', 'Ghāta'], ['vedha', 'Vedhasthāna'], ['jiva', 'Jīva/deha'], ['shodhana', 'Śodhana']];
const AUX_SRC = { /* where each attribute comes from, for the hover footers */
  sahams: 'Tajika sahams (Tajika Nīlakaṇṭhī), auxiliary.yaml', avakahada: 'almanac avakahaḍā chakra; matching.yaml + auxiliary.yaml',
  gender: 'odd/even signs of the rāśi chart', mrityu: 'mṛtyu-bhāga table reprinted with Phaladīpikā and Jātaka Pārijāta',
  ghata: 'ghātaka chakra (muhūrta almanac table)', vedha: 'gochara vedha table (Phaladīpikā ch. 26)', jiva: 'Kālachakra (BPHS ch. 46)',
  shodhana: 'BPHS ch. 67–69',
};
let AUX_VEDHA = 'natal', AUX_SHOD = 'sun', AUX_SEL = null;
const hov = (el, title, rows, sub) => { el.addEventListener('mousemove', ev => tip(ev, title, rows, sub)); el.addEventListener('mouseleave', untip); return el; };
const gname = id => id === 'lagna' ? 'Lagna' : (G[id] ? G[id].en : id);
const gab = id => id === 'lagna' ? 'As' : (G[id] ? G[id].abbr : id);
const glinkAny = id => id === 'lagna' ? h('a', { href: '#/house/1', class: 'glink' }, dot('lagna'), 'Lagna') : glink(id);
const auxDeg = (lon) => `${SIGN_SKT[signOf(lon)]} ${inSign(lon)}`;

/* a small South Indian square with the sahams written into their signs; rows and cells highlight each other */
function sahamChart(rows) {
  const W = 92, H = 74, svg = s('svg', { viewBox: `-2 -2 ${4 * W + 4} ${4 * H + 4}`, class: 'aux-sq', role: 'img', 'aria-label': 'Sahams on a South Indian chart' });
  const by = {}; rows.forEach(r => (by[r.sign] = by[r.sign] || []).push(r));
  for (const [sg, [c, r]] of Object.entries(SOUTH_CELL)) {
    const sign = +sg, x = c * W, y = r * H, g = s('g', { class: 'aux-cell', 'data-sign': sign }, svg);
    g.style.setProperty('--i', sign);
    s('rect', { x, y, width: W, height: H, class: 'cellbg' }, g);
    const nm = s('text', { x: x + W - 4, y: y + 12, 'text-anchor': 'end', 'font-size': 10, fill: 'var(--muted)' }, g); nm.textContent = SIGN_AB[sign];
    if (LAGNA_SIGN === sign) s('path', { d: `M${x} ${y + 16}L${x + 16} ${y}`, stroke: 'var(--accent)', 'stroke-width': 2 }, g);
    const list = by[sign] || [];
    list.forEach((sr, k) => {
      const col = k % 2, row = Math.floor(k / 2), t = s('text', { x: x + 5 + col * (W / 2 - 2), y: y + 26 + row * 11.5, 'font-size': list.length > 6 ? 8.4 : 9.2, class: 'aux-sn', 'data-id': sr.id }, g);
      t.textContent = sr.name.length > 7 ? sr.name.slice(0, 6) + '.' : sr.name;
    });
    hov(g, `${SIGN_SKT[sign]} · ${list.length} saham${list.length === 1 ? '' : 's'}`, list.map(sr => [sr.name, sr.dms]), houseOf(sign) ? `bhāva ${houseOf(sign)} from the lagna` : '');
    g.addEventListener('mouseenter', () => markSaham(svg.closest('.aux-sahams'), null, sign));
    g.addEventListener('mouseleave', () => markSaham(svg.closest('.aux-sahams'), null, null));
  }
  s('rect', { x: W, y: H, width: 2 * W, height: 2 * H, class: 'cellmid' }, svg);
  const t = s('text', { x: 2 * W, y: 2 * H - 2, 'text-anchor': 'middle', 'font-size': 14, 'font-weight': 800, fill: 'var(--ink)' }, svg); t.textContent = '37 sahams';
  const t2 = s('text', { x: 2 * W, y: 2 * H + 14, 'text-anchor': 'middle', 'font-size': 10, fill: 'var(--muted)' }, svg); t2.textContent = 'hover a sign or a row';
  return svg;
}
function markSaham(root, id, sign) {
  if (!root) return;
  root.querySelectorAll('.aux-cell').forEach(c => c.classList.toggle('hot', sign != null && +c.dataset.sign === sign));
  root.querySelectorAll('tr[data-sign]').forEach(tr => tr.classList.toggle('hot', (id != null && tr.dataset.id === id) || (sign != null && +tr.dataset.sign === sign)));
  root.querySelectorAll('.aux-sn').forEach(t => t.classList.toggle('hot', id != null && t.dataset.id === id));
}

function tabSahams(v) {
  if (!v) return missing('Sahams', 'They need the lagna and the seven grahas.');
  const rows = v.sahams;
  const tb = table(['#', 'Saham', 'Sign', 'Degree', 'Lord', 'Formula'], rows.map((r, i) => [i + 1, h('b', {}, r.name), SIGN_SKT[r.sign], r.dms, glink(r.lord),
    h('span', { class: 'aux-f' }, r.formula, r.reversed_at_night ? h('span', { class: 'chip aux-mini' }, 'night ⇄') : null, r.day_form_inferred ? h('span', { class: 'chip aux-mini warn' }, 'day form inferred') : null)]),
  { rowAttrs: i => ({ 'data-id': rows[i].id, 'data-sign': rows[i].sign, class: 'aux-row' }) });
  tb.querySelectorAll('tr.aux-row').forEach((tr, i) => {
    const r = rows[i];
    hov(tr, `${r.name} saham · ${auxDeg(r.longitude)}`, [['Formula', r.formula], ['Terms', `A ${r.terms.a}, B ${r.terms.b}, C ${r.terms.c}`],
      ['+30° rule', r.plus_30 ? 'applied: C is not on the arc from B to A' : 'not needed'], ['Night reversal', r.reversed_at_night ? 'A and B swapped (night birth)' : 'none'],
      ['Signifies', r.meaning + ' (interpretation, Tajika tradition, paraphrased)']], AUX_SRC.sahams);
    tr.addEventListener('mouseenter', () => markSaham(tr.closest('.aux-sahams'), r.id, r.sign));
    tr.addEventListener('mouseleave', () => markSaham(tr.closest('.aux-sahams'), null, null));
  });
  return h('div', { class: 'aux-sahams' },
    h('div', { class: 'aux-two' }, card('On the chart', sahamChart(rows)),
      card('How they are computed', facts([['Birth', v.birth_in === 'night' ? 'night (A and B swap where the tradition says so)' : 'day'], ['Day or night', v.day_night_rule], ['House cusps', v.cusps], ['Rule', v.rule], ['Pack', v.pack]]),
        h('p', {}, label('tradition_rule'), ' positions; the “signifies” text in each hover is ', label('interpretation'), ' (paraphrased from the Tajika tradition).'))),
    card('All 37 sahams', tb));
}

function tabAvakahada(a) {
  if (!a) return missing('The avakahaḍā chakra', 'It needs the Moon.');
  const cv = a.conventions || {};
  const items = [
    ['Ascendant / lord', a.ascendant ? `${a.ascendant.sign_name} / ${gname(a.ascendant.lord)}` : '—', 'The rising sign and its lord.'],
    ['Sign / lord', `${a.moon_sign.sign_name} / ${gname(a.moon_sign.lord)}`, 'The Moon sign (janma rāśi) and its lord.'],
    ['Nakshatra / pada', `${a.nakshatra.name} / ${a.nakshatra.pada}`, 'The Moon\'s nakshatra and quarter.'],
    ['Yoga / karaṇa', `${a.yoga || '—'} / ${a.karana ? a.karana.name : '—'}${a.karana && a.karana.animal ? ` (${a.karana.animal})` : ''}`, 'Birth pañcāṅga yoga and karaṇa; the karaṇa\'s traditional animal in brackets.'],
    ['Yoni / gaṇa', `${a.yoni.name} / ${a.gana}`, 'Animal (yoni) and temperament (gaṇa) of the Moon\'s nakshatra, the same tables as aṣṭakūṭa matching.'],
    ['Nāḍī / varṇa', `${a.nadi} / ${a.varna}`, 'Nāḍī of the nakshatra; varṇa of the Moon sign (by element).'],
    ['Vaśya / varga', `${a.vashya} / ${a.varga.name}`, `Vaśya group of the Moon sign; varga (${a.varga.animal}) from the first sound of the name syllable.`],
    ['Yuñjā / haṃsaka', `${a.yunja} / ${a.hansak}`, 'Part of the nakshatra circle (pūrva, madhya, para); element (tattva) of the Moon sign.'],
    ['Nāmākṣara', a.naamakshara, 'The syllable of the Moon\'s pada, traditionally the first sound of the birth name.'],
    ['Pāyā sign / nakshatra', `${a.paya.by_sign || '—'} / ${a.paya.by_nakshatra}`, cv.paya],
    ['Sun sign (Western)', a.sunsign_western || '—', cv.sunsign_western],
  ];
  return h('div', {}, card('Avakahaḍā chakra', h('div', { class: 'aux-grid' }, items.map(([k, v2, why], i) => hov(h('div', { class: 'aux-tile', style: `--i:${i}` }, h('small', {}, k), h('b', {}, v2)), k, [['Value', v2], ['Rule', why]], AUX_SRC.avakahada))),
    note(cv.from_moon || ''), h('p', {}, label('tradition_rule'), ' Every item is a table lookup; none is a prediction.')));
}

function tabGender(mf) {
  if (!mf) return missing('Male and female rāśis', 'It needs the positions.');
  const col = (title, list, cls) => card(title, h('div', { class: 'aux-chips ' + cls }, list.map((id, i) => {
    const p = mf.points.find(x => x.id === id);
    return hov(h('span', { class: 'chip aux-pop', style: `--i:${i}` }, glinkAny(id), h('small', {}, ' ' + p.sign_name)), gname(id), [['Sign', `${p.sign_name} (${nth(p.sign + 1)} sign)`], ['Gender', p.gender]], mf.rule);
  })));
  const strip = s('svg', { viewBox: '0 0 600 74', class: 'aux-strip', role: 'img', 'aria-label': 'Twelve signs, male and female' });
  for (let i = 0; i < 12; i++) {
    const x = i * 50, male = i % 2 === 0, g = s('g', { class: 'aux-sg ' + (male ? 'm' : 'f') }, strip); g.style.setProperty('--i', i);
    s('rect', { x: x + 1, y: 4, width: 48, height: 40, rx: 6 }, g);
    const t = s('text', { x: x + 25, y: 20, 'text-anchor': 'middle', 'font-size': 11, 'font-weight': 700 }, g); t.textContent = SIGN_AB[i];
    const here = mf.points.filter(p => p.sign === i);
    const tt = s('text', { x: x + 25, y: 36, 'text-anchor': 'middle', 'font-size': 10 }, g); tt.textContent = here.map(p => gab(p.id)).join(' ');
    const lb = s('text', { x: x + 25, y: 60, 'text-anchor': 'middle', 'font-size': 9.5, fill: 'var(--muted)' }, g); lb.textContent = male ? '♂ odd' : '♀ even';
    hov(g, `${SIGN_SKT[i]} · ${male ? 'male (odd)' : 'female (even)'}`, here.map(p => [gname(p.id), p.gender]));
  }
  return h('div', {}, card('The twelve signs', strip, note(mf.rule)), h('div', { class: 'aux-two' }, col(`In female rāśis (${mf.female.length})`, mf.female, 'f'), col(`In male rāśis (${mf.male.length})`, mf.male, 'm')));
}

function tabMrityu(v) {
  if (!v) return missing('Mṛtyu bhāga', 'It needs the positions.');
  const W = 560, rowH = 30, svg = s('svg', { viewBox: `0 0 ${W + 250} ${v.points.length * rowH + 26}`, class: 'aux-mb', role: 'img', 'aria-label': 'Distance from the mṛtyu bhāga' });
  const X = d => 120 + d / 30 * W;
  [0, 10, 20, 30].forEach(d => { s('line', { x1: X(d), x2: X(d), y1: 0, y2: v.points.length * rowH, class: 'aux-grid-l' }, svg); const t = s('text', { x: X(d), y: v.points.length * rowH + 16, 'text-anchor': 'middle', 'font-size': 10, fill: 'var(--muted)' }, svg); t.textContent = d + '°'; });
  v.points.forEach((p, i) => {
    const y = i * rowH + 15, g = s('g', { class: 'aux-mbrow' + (p.within_1_deg ? ' near' : '') }, svg); g.style.setProperty('--i', i);
    const n = s('text', { x: 112, y: y + 4, 'text-anchor': 'end', 'font-size': 12, 'font-weight': 700 }, g); n.textContent = `${gname(p.id)} · ${SIGN_AB[p.sign]}`;
    s('line', { x1: X(Math.min(p.degree_in_sign, p.mrityu_bhaga_deg)), x2: X(Math.max(p.degree_in_sign, p.mrityu_bhaga_deg)), y1: y, y2: y, class: 'aux-gap' }, g);
    s('path', { d: `M${X(p.mrityu_bhaga_deg)} ${y - 8}l5 8l-5 8l-5 -8z`, class: 'aux-mbmark' }, g);
    s('circle', { cx: X(p.degree_in_sign), cy: y, r: 6, fill: (G[p.id] || G.lagna).dot, class: 'aux-pt' }, g);
    const d = s('text', { x: W + 140, y: y + 4, 'font-size': 12 }, g); d.textContent = p.distance_dms;
    hov(g, `${gname(p.id)} in ${p.sign_name}`, [['Degree in sign', inSign(p.degree_in_sign)], ['Mṛtyu bhāga', `${p.mrityu_bhaga_deg}°`], ['Distance', p.distance_dms], ['Within 1°', p.within_1_deg ? 'yes' : 'no']], v.source);
  });
  return h('div', {}, card('Distance from the mṛtyu bhāga', svg, note('Dot: the point\'s degree in its sign. Diamond: the mṛtyu bhāga degree of that sign for that point. The bar is the distance (printed on the right). ', v.rule)),
    card('Table', table(['Point', 'Sign', 'Degree', 'Mṛtyu bhāga', 'Distance'], v.points.map(p => [glinkAny(p.id), SIGN_SKT[p.sign], inSign(p.degree_in_sign), `${p.mrityu_bhaga_deg}°`, p.within_1_deg ? h('b', { class: 'warn' }, p.distance_dms) : p.distance_dms])), note('Source: ', v.source, '.')));
}

function tabGhata(g) {
  if (!g) return missing('The ghāta chakra', 'It needs the Moon.');
  const hit = new Set(g.birth_coincides_with || []);
  const items = [['Month', g.month, null, 'Lunar month inauspicious for this Moon sign.'], ['Tithi', g.tithis.join(' / '), 'tithi', 'Tithis (counted within the pakṣa).'],
    ['Day', g.weekday_name, 'weekday', 'Weekday.'], ['Nakshatra', g.nakshatra_name, 'nakshatra', 'Nakshatra of the day.'], ['Yoga', g.yoga, 'yoga', 'Pañcāṅga yoga.'],
    ['Karaṇa', `${g.karana}${g.karana_animal ? ` (${g.karana_animal})` : ''}`, 'karana', 'Karaṇa (and its traditional animal).'], ['Prahara', String(g.prahara), null, 'Watch of the day (one of four).'],
    ['Ghāta Chandra', g.moon_sign_name, null, 'The transit Moon sign that is ghāta for this natal Moon.']];
  return h('div', {}, card(`Ghāta chakra for a Moon in ${g.natal_moon_sign_name}`, h('div', { class: 'aux-grid' }, items.map(([k, v2, key, why], i) =>
    hov(h('div', { class: 'aux-tile' + (key && hit.has(key) ? ' hit' : ''), style: `--i:${i}` }, h('small', {}, k), h('b', {}, v2), key && hit.has(key) ? h('span', { class: 'chip aux-mini warn' }, 'the birth has it') : null), k, [['Value', v2], ['Meaning', why], ['At birth', key ? (hit.has(key) ? 'the birth falls on it' : 'not at birth') : null]], AUX_SRC.ghata))),
    note(g.rule), h('p', {}, label('tradition_rule'), ' The table names times to avoid for important undertakings (', label('interpretation'), ', muhūrta tradition).')));
}

function tabVedha(v) {
  if (!v) return missing('Vedhasthāna', 'It needs the Moon.');
  const modes = [['natal', 'Birth grahas']].concat(v.transit ? [['transit', `Transits at ${dateOnly(v.transit.at_utc)}`]] : []);
  const mode = v.transit && AUX_VEDHA === 'transit' ? 'transit' : 'natal', rows = mode === 'transit' ? v.transit.grahas : v.natal;
  const strip = h('div', { class: 'aux-houses' }, Array.from({ length: 12 }, (_, k) => {
    const hno = k + 1, here = rows.filter(r => r.house_from_moon === hno);
    return h('div', { class: 'aux-house', 'data-h': hno, style: `--i:${k}` }, h('small', {}, `${hno}`), h('span', {}, SIGN_AB[(v.natal_moon_sign + k) % 12]), h('b', {}, here.map(r => G[r.graha].abbr).join(' ')));
  }));
  const light = (r) => strip.querySelectorAll('.aux-house').forEach(el => {
    const hno = +el.dataset.h, fav = r && (r.favourable_houses || []).includes(hno), vh = r && r.vedha_house === hno;
    el.classList.toggle('fav', !!fav); el.classList.toggle('vedha', !!vh); el.classList.toggle('at', !!r && r.house_from_moon === hno);
  });
  const tb = table(['Graha', 'House from Moon', 'State', 'Vedha'], rows.map(r => [glink(r.graha), r.house_from_moon,
    h('span', { class: 'chip ' + (r.state === 'auspicious' ? 'ok' : r.state === 'inauspicious' ? 'bad' : 'warn') }, r.state),
    r.obstructed_by.length ? `by ${r.obstructed_by.map(gname).join(', ')} in ${r.vedha_house}` : r.favourable ? `vedha house ${r.vedha_house}: clear` : '—']), { rowAttrs: i => ({ class: 'aux-vrow', 'data-g': rows[i].graha }) });
  tb.querySelectorAll('tr.aux-vrow').forEach((tr, i) => {
    const r = rows[i];
    tr.addEventListener('mouseenter', () => light(r)); tr.addEventListener('mouseleave', () => light(null));
    hov(tr, r.summary, [['Favourable houses', (r.favourable_houses || []).join(', ')], ['This graha', `house ${r.house_from_moon}`], ['Vedha house', r.vedha_house], ['Obstructed by', r.obstructed_by.map(gname).join(', ') || 'none']], AUX_SRC.vedha);
  });
  return h('div', {}, h('div', { class: 'toolbar' }, h('div', { class: 'grp' }, h('b', {}, 'Positions'), tabs(modes, mode, k => { AUX_VEDHA = k; route(); }))),
    card(`Houses from the natal Moon (${v.natal_moon_sign_name})`, strip, note('Hover a row: green = the graha\'s favourable houses, red = the vedha house that obstructs it, ring = where it stands.')),
    card(mode === 'transit' ? `Gochara at ${fmtTime(v.transit.at_utc)}` : 'Birth grahas (as the reference prints it)', tb, note(v.rule)));
}

function tabJiva(j) {
  if (!j) return missing('Jīva and deha', 'It needs the Moon.');
  let acc = 0;
  const seq = h('div', { class: 'aux-seq' }, j.signs.map((sg, i) => {
    acc += j.years[i];
    const role = i === 0 ? 'deha' : i === j.signs.length - 1 ? 'jīva' : '';
    return hov(h('div', { class: 'aux-step' + (role ? ' key' : ''), style: `--i:${i}` }, h('small', {}, role || nth(i + 1)), h('b', {}, SIGN_SKT[sg]), h('span', {}, `${j.years[i]} y`)),
      `${nth(i + 1)} sign: ${SIGN_SKT[sg]}`, [['Daśā years', j.years[i]], ['Running total', `${acc} years`], ['Role', role || 'middle of the sequence']], AUX_SRC.jiva);
  }));
  return h('div', {}, h('div', { class: 'stats' }, stat('Jīva', j.jiva.sign_name, `lord ${gname(j.jiva.lord)}`), stat('Deha', j.deha.sign_name, `lord ${gname(j.deha.lord)}`), stat('Paramāyus', `${j.paramayus_years} years`, `Kālachakra ${j.direction}, ${j.nakshatra} pada ${j.pada}`)),
    card('The nine signs of the Moon\'s pada', seq, note(j.rule), h('p', {}, label('tradition_rule'), ' Paramāyus is the full span of the Kālachakra sequence for this pada, the sum of its signs\' years; it is a structural number of the daśā system, not a lifespan prediction.')));
}

function tabShodhana(v) {
  if (!v) return missing('Śodhana', 'It needs the lagna and the aṣṭakavarga.');
  const ord = v.order, max = Math.max(...ord.map(g => v.by_chart[g].shodhya_pinda));
  const tb = table(['Aṣṭakavarga', 'Rāśi piṇḍa', 'Graha piṇḍa', 'Śodhya piṇḍa', ''], ord.map(g => { const r = v.by_chart[g]; return [glinkAny(g), r.rashi_pinda, r.graha_pinda, h('b', {}, r.shodhya_pinda), bar(r.shodhya_pinda, max, (G[g] || G.lagna).dot)]; }), { rowAttrs: i => ({ class: 'aux-srow', 'data-g': ord[i] }) });
  tb.querySelectorAll('tr.aux-srow').forEach(tr => { const g = tr.dataset.g, r = v.by_chart[g]; hov(tr, `${gname(g)} aṣṭakavarga`, [['Rāśi piṇḍa', `Σ reduced bindus × rāśi multiplier = ${r.rashi_pinda}`], ['Graha piṇḍa', `Σ bindus in each graha's sign × graha multiplier = ${r.graha_pinda}`], ['Śodhya piṇḍa', r.shodhya_pinda]], 'click to see the reduction'); tr.addEventListener('click', () => { AUX_SHOD = g; route(); }); });
  const sel = v.by_chart[AUX_SHOD] ? AUX_SHOD : 'sun', r = v.by_chart[sel];
  const stage = (name, arr, prev, why) => h('tr', {}, h('th', {}, name), arr.map((n, i) => hov(h('td', { class: 'aux-n' + (prev && prev[i] !== n ? ' cut' : ''), style: `--i:${i}` }, String(n)), `${SIGN_SKT[i]} · ${name}`, [['Bindus', n], ['Before', prev ? prev[i] : null], ['Why', why]], AUX_SRC.shodhana)));
  const steps = h('div', { class: 'tbl-wrap' }, h('table', { class: 'aux-steps' }, h('thead', {}, h('tr', {}, h('th', {}, ''), SIGN_AB.map(a => h('th', {}, a)))),
    h('tbody', {}, stage('Bindus', r.bindus, null, 'bhinnāṣṭakavarga'), stage('Trikoṇa', r.after_trikona, r.bindus, 'smallest of each trine triplet subtracted'),
      stage('Ekādhipatya', r.after_ekadhipatya, r.after_trikona, 'pairs with one lord reduced (the lagna counts as an occupant)'),
      h('tr', {}, h('th', {}, '× rāśi'), v.rashi_multiplier.map(m => h('td', { class: 'aux-mul' }, '×' + m))))));
  return h('div', {}, card('Piṇḍas', tb, note(v.rule)),
    card(`Reduction of the ${gname(sel)} aṣṭakavarga`, tabs(ord.map(g => [g, gab(g)]), sel, k => { AUX_SHOD = k; route(); }), steps, note('Highlighted cells changed at that step. Graha multipliers: ', SEVEN.map(g => `${gab(g)} ${v.graha_multiplier[g]}`).join(', '), '.')));
}

page('aux', (arg) => 'Auxiliary analysis' + (arg ? ' · ' + ((AUX_TABS.find(t => t[0] === arg) || [])[1] || '') : ''), 'Analysis', (b, arg) => {
  const any = ['sahams', 'avakahada', 'male_female', 'mrityu_bhaga', 'ghata_chakra', 'vedhasthana', 'jiva_deha', 'shodhana'].some(k => val('vedic.aux.' + k));
  if (!any) return put(b, pageHead('aux', 'Auxiliary analysis'), missing('Auxiliary analysis', 'Compute the chart with the auxiliary feature (features: all).'));
  const cur = AUX_TABS.some(t => t[0] === arg) ? arg : 'sahams';
  const body = {
    sahams: () => tabSahams(val('vedic.aux.sahams')), avakahada: () => tabAvakahada(val('vedic.aux.avakahada')), gender: () => tabGender(val('vedic.aux.male_female')),
    mrityu: () => tabMrityu(val('vedic.aux.mrityu_bhaga')), ghata: () => tabGhata(val('vedic.aux.ghata_chakra')), vedha: () => tabVedha(val('vedic.aux.vedhasthana')),
    jiva: () => tabJiva(val('vedic.aux.jiva_deha')), shodhana: () => tabShodhana(val('vedic.aux.shodhana')),
  }[cur]();
  put(b, pageHead('aux', 'Auxiliary analysis', 'Sahams, the avakahaḍā chakra, male and female rāśis, mṛtyu bhāga, ghāta chakra, vedhasthāna, jīva and deha, and aṣṭakavarga śodhana. Hover any value for its rule.'),
    h('div', { class: 'toolbar aux-tabs' }, tabs(AUX_TABS, cur, k => go('#/aux/' + k))), h('div', { class: 'aux-pane', 'data-tab': cur }, body));
}, { blurb: 'sahams, avakahaḍā, mṛtyu bhāga, vedha, śodhana' });
