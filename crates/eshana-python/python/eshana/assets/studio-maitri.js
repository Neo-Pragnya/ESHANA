/* ESHANA Studio: Maitri and dignities. The compound (pañcadhā) relationships of the nine grahas, the nakshatra
   spatial matrix (tārās between the lagna and the grahas) and the dignities in the sixteen divisions, all from
   rule vedic.maitri@0.1 (val('vedic.maitri.…')). The page only lays out and colours the engine's values. */

SIGIL.maitri = '<circle cx="7" cy="12" r="3" class="core"/><circle cx="17" cy="12" r="3" class="core soft"/><path d="M10 12H14" class="ray draw"/><circle cx="12" cy="12" r="9.5" class="ring dash spin"/>';

const MT_REL = { great_friend: 'Grt. friend', friend: 'Friend', neutral: 'Neutral', enemy: 'Enemy', great_enemy: 'Grt. enemy' };
const MT_DIG = Object.assign({ exalted: 'Exalted', moolatrikona: 'Mūlatrikoṇa', own: 'Own', debilitated: 'Debilitated' }, MT_REL);
const MT_DIG_AB = { exalted: 'Ex', moolatrikona: 'MT', own: 'Own', debilitated: 'Deb', great_friend: 'GF', friend: 'F', neutral: 'N', enemy: 'E', great_enemy: 'GE' };
const mtName = id => (G[id] || G.lagna).en;
const mtHead = id => GR[id]
  ? h('a', { href: '#/planet/' + id, class: 'mt-h', title: G[id].en }, dot(id), (G[id] || G.lagna).abbr)
  : h('span', { class: 'mt-h', title: mtName(id) }, dot(id), (G[id] || G.lagna).abbr);
const mtHover = (td, title, rows, sub) => {
  td.addEventListener('mousemove', ev => tip(ev, title, rows, sub));
  td.addEventListener('mouseleave', untip);
  return td;
};
/* a square grid: row heads, column heads, one cell per pair; cells animate in along the diagonal */
function mtGrid(rows, cols, cell, cls, colHead) {
  return h('div', { class: 'tbl-wrap' }, h('table', { class: 'mt-grid ' + cls },
    h('thead', {}, h('tr', {}, h('th', {}, ''), cols.map(c => h('th', {}, colHead ? colHead(c) : mtHead(c))))),
    h('tbody', {}, rows.map((r, i) => h('tr', {}, h('th', {}, mtHead(r)), cols.map((c, j) => {
      const td = cell(r, c) || h('td', { class: 'mt-self' }, '·');
      td.style.setProperty('--k', i + j);
      return td;
    }))))));
}
/* the kinds in a grid as chips that filter it: click one to show only those cells, click more to add them, click
   again (or "Show all") to clear. kinds: [[class, text, count]] */
function mtFilter(wrap, kinds) {
  const grid = wrap.querySelector('table'), on = new Set();
  const apply = () => {
    grid.classList.toggle('mt-filtering', on.size > 0);
    grid.querySelectorAll('.mt-c').forEach(td => td.classList.toggle('mt-hit', [...on].some(c => td.classList.contains(c))));
    btns.forEach(b => b.setAttribute('aria-pressed', String(on.has(b.dataset.k))));
    clear.hidden = !on.size;
    shown.textContent = on.size ? `${grid.querySelectorAll('.mt-c.mt-hit').length} shown` : '';
  };
  const btns = kinds.map(([cls, text, n]) => h('button', { type: 'button', class: 'mt-fchip', 'data-k': cls, 'aria-pressed': 'false', title: `Show only ${text} (click more to add; click again to clear)`,
    onclick: () => { if (on.has(cls)) on.delete(cls); else on.add(cls); apply(); } }, h('i', { class: cls }), text + (n != null ? ': ' : ''), n != null ? h('b', {}, n) : null));
  const clear = h('button', { type: 'button', class: 'mt-fclear', hidden: true, onclick: () => { on.clear(); apply(); } }, 'Show all');
  const shown = h('small', { class: 'note' });
  return h('div', { class: 'chips mt-filter', role: 'group', 'aria-label': 'Filter the grid' }, h('b', { class: 'mt-ftitle' }, 'Filter'), ...btns, clear, shown);
}
const mtLegend = items => h('div', { class: 'mt-legend' }, items.map(([cls, text]) => h('span', {}, h('i', { class: cls }), text)));

function mtRelationships(rel) {
  const R = {}; rel.rows.forEach(x => { R[x.from + '>' + x.to] = x; });
  const ids = rel.grahas.filter(g => rel.rows.some(x => x.from === g));
  const cell = (a, b) => {
    const x = R[a + '>' + b]; if (!x) return null;
    const why = mtName(b) + ' is ' + nth(x.temporal_house) + ' from ' + mtName(a) + (x.temporal === 'friend' ? ' (2, 3, 4, 10, 11 or 12: a temporary friend)' : ' (1, 5, 6, 7, 8 or 9: a temporary enemy)');
    const td = h('td', { class: 'mt-c rel-' + x.compound, 'data-from': a, 'data-to': b }, MT_REL[x.compound]);
    return mtHover(td, mtName(a) + ' → ' + mtName(b), [['Natural', x.natural + ' (' + x.natural_source + ')'], ['Temporal', x.temporal], ['Why', why], ['Compound', MT_REL[x.compound]]],
      'natural + temporal = compound (pañcadhā)');
  };
  const count = k => rel.rows.filter(x => x.compound === k).length;
  const grid = mtGrid(ids, ids, cell, 'mt-rel');
  return card('Compound relationships (pañcadhā)',
    h('div', { class: 'chips' }, h('span', { class: 'chip' }, label('tradition_rule'), ' vedic.maitri@0.1')),
    mtFilter(grid, Object.keys(MT_REL).map(k => ['rel-' + k, MT_REL[k], count(k)])),
    note('Row = the graha, column = the graha it regards. Hover a cell for the natural and temporal parts and why. Click a kind above to show only those pairs.'),
    grid,
    note(rel.convention + '. The nodes follow the table in rule pack maitri.yaml; BPHS gives none for them.'));
}

function mtTaras(tm) {
  const T9 = {}; tm.rows.forEach(x => { T9[x.row + '>' + x.column] = x; });
  const nk = id => (tm.nakshatra[id] || {}).name || '—';
  const short = n => n === 'Parama mitra' ? 'P. mitra' : n;
  const cell = (r, c) => {
    const x = T9[r + '>' + c]; if (!x) return null;
    const td = h('td', { class: 'mt-c tara-' + x.nature, 'data-row': r, 'data-col': c }, short(x.name));
    return mtHover(td, mtName(r) + ' from ' + mtName(c), [['Counted from', nk(c)], ['To', nk(r)], ['Count', nth(x.count)], ['Tārā', x.tara + ' · ' + x.name], ['Nature', x.nature]],
      '((count − 1) mod 9) + 1');
  };
  const n = k => tm.rows.filter(x => x.nature === k).length;
  const grid = mtGrid(tm.points, tm.points, cell, 'mt-tara');
  // each tārā by name, too, so one tārā (say Vadha) can be picked out
  grid.querySelectorAll('.mt-c').forEach(td => td.classList.add('tn-' + td.textContent.replace(/\W+/g, '-')));
  const names = [...new Set(tm.rows.map(x => x.name))].sort((a, b2) => (tm.rows.find(x => x.name === a).tara - tm.rows.find(x => x.name === b2).tara));
  return card('Nakshatra spatial matrix (tārās)',
    h('div', { class: 'chips' }, h('span', { class: 'chip' }, label('tradition_rule'), ' vedic.maitri@0.1')),
    mtFilter(grid, [['tara-auspicious', 'Auspicious (Sampat, Kṣema, Sādhaka, Mitra, Parama mitra)', n('auspicious')], ['tara-mixed', 'Janma', n('mixed')], ['tara-inauspicious', 'Inauspicious (Vipat, Pratyari, Vadha)', n('inauspicious')]]
      .concat(names.map(nm => ['tn-' + short(nm).replace(/\W+/g, '-'), nm, tm.rows.filter(x => x.name === nm).length]))),
    note('Each cell is the tārā of the row’s nakshatra counted from the column’s. Hover for the count; click a nature or a tārā above to show only those cells.'),
    grid,
    note(tm.convention + '.'),
    h('p', { class: 'note' }, label('interpretation'), ' In muhūrta and compatibility practice the 3rd, 5th and 7th tārās (Vipat, Pratyari, Vadha) are read as obstructive and the even ones as supportive; Janma is mixed (paraphrased from Muhūrta Chintāmaṇi and common Parāśari teaching).'));
}

function mtDignities(vd) {
  const ids = MT_NINE.filter(g => vd.grahas[g]);
  const name = {}; vd.divisions.forEach(d => { name[d.d] = d.name; });
  const ds = vd.divisions.map(d => d.d);
  const cellOf = (g, d) => vd.grahas[g].find(c => c.d === d);
  const cell = (g, d) => {
    const c = cellOf(g, d); if (!c) return null;
    const td = h('td', { class: 'mt-c dig-' + c.dignity, 'data-g': g, 'data-d': d }, MT_DIG_AB[c.dignity]);
    return mtHover(td, mtName(g) + ' in D' + d + ' (' + name[d] + ')', [['Sign', SIGN_SKT[c.sign] + ' (' + SIGN_AB[c.sign] + ')'], ['Degree in varga', c.degree == null ? 'by sign' : fix(c.degree, 2) + '°'],
      ['Sign lord', mtName(c.lord)], ['Dignity', MT_DIG[c.dignity]], ['Why', c.reason]]);
  };
  const sumCell = g => {
    const s0 = vd.summary[g], n = 16, neu = n - s0.favourable - s0.unfavourable;
    const seg = (cls, v) => v ? h('i', { class: cls, style: 'width:' + (v / n * 100) + '%' }) : null;
    const td = h('td', { class: 'mt-sumcell' }, h('div', { class: 'mt-sum' }, seg('s-dig', s0.dignified), seg('s-fr', s0.friendly), seg('s-neu', neu), seg('s-bad', s0.unfavourable)),
      h('small', {}, s0.dignified + ' dignified · ' + s0.friendly + ' friendly · ' + s0.unfavourable + ' hostile'));
    return mtHover(td, mtName(g) + ' in 16 divisions', Object.keys(MT_DIG).map(k => [MT_DIG[k], (s0.counts[k] || 0) + ' / 16']));
  };
  const head = h('tr', {}, h('th', {}, ''), ds.map(d => h('th', { title: name[d] }, 'D' + d)), h('th', {}, 'Summary'));
  const body = ids.map((g, i) => h('tr', {}, h('th', {}, mtHead(g)), ds.map((d, j) => { const td = cell(g, d); td.style.setProperty('--k', i + j); return td; }), sumCell(g)));
  const best = ids.slice().sort((a, b) => vd.summary[b].favourable - vd.summary[a].favourable)[0];
  const grid = h('div', { class: 'tbl-wrap' }, h('table', { class: 'mt-grid mt-dig' }, h('thead', {}, head), h('tbody', {}, body)));
  const dn = k => ids.reduce((a, g) => a + vd.grahas[g].filter(c => c.dignity === k).length, 0);
  return card('Dignities in the sixteen divisions',
    h('div', { class: 'chips' }, h('span', { class: 'chip' }, label('tradition_rule'), ' vedic.maitri@0.1'),
      best ? h('span', { class: 'chip' }, 'Most favourable placements: ', h('b', {}, mtName(best)), ' (' + vd.summary[best].favourable + ' / 16)') : null),
    mtFilter(grid, Object.keys(MT_DIG).map(k => ['dig-' + k, MT_DIG_AB[k] + ' ' + MT_DIG[k], dn(k)])),
    note('Rows are the grahas, columns the divisions. Hover a cell for the sign, the degree inside the varga, the lord and the reason. Click a dignity above to show only those cells.'),
    grid,
    note(vd.convention + '. Viṃśopaka on the Strength page keeps its own scoring convention.'));
}

const MT_NINE = ['sun', 'moon', 'mars', 'mercury', 'jupiter', 'venus', 'saturn', 'rahu', 'ketu'];

page('maitri', () => 'Maitri and dignities', 'Grahas & bhāvas', (b) => {
  const rel = val('vedic.maitri.relationships'), tm = val('vedic.maitri.tara_matrix'), vd = val('vedic.maitri.varga_dignities');
  if (!rel && !tm && !vd) return put(b, missing('Maitri and dignities', 'Ask for the maitri feature (or features: ["all"]).'));
  put(b, pageHead('maitri', 'Maitri and dignities', 'How the nine grahas regard one another, the tārā each nakshatra makes with the others, and each graha’s dignity in the sixteen divisions.'),
    rel ? mtRelationships(rel) : null, tm ? mtTaras(tm) : null, vd ? mtDignities(vd) : null);
}, { blurb: 'compound relationships, tārā matrix, dignities in the 16 vargas' });
