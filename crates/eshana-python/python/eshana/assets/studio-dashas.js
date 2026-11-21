/* ESHANA Studio: daśā pages (Vimśottarī zoom, Yoginī, and the extended systems from vedic.dashas.*). */

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
/* ── daśā zoom: five levels, each a thinner row that opens out of the period chosen above it ──────────────────── */
const VIM_ORDER = ['ketu', 'venus', 'sun', 'moon', 'mars', 'rahu', 'jupiter', 'saturn', 'mercury'];
const VIM_YEARS = { ketu: 7, venus: 20, sun: 6, moon: 10, mars: 7, rahu: 18, jupiter: 16, saturn: 19, mercury: 17 };
const DASHA_LEVELS = ['Mahādaśā', 'Antardaśā', 'Pratyantardaśā', 'Sūkṣma daśā', 'Prāṇa daśā'];
const cap = s0 => s0.charAt(0).toUpperCase() + s0.slice(1);
/* sub-periods of a Vimśottarī period: from the engine where it gave them (levels 1–3), otherwise by the same rule
   (each lord's share is its years out of 120, in order from the period's own lord), clipped at birth */
function vimChildren(p, vim) {
  const kids = p.antardashas || p.pratyantardashas || p.sookshmas || p.sub;
  if (kids && kids.length) return kids;
  const lord = (p.lord || '').toLowerCase(); if (!VIM_YEARS[lord]) return [];
  const ms = (vim.year_days || 365.25) * 864e5, end = T(p.end_utc), full = (p.years || (end - T(p.start_utc)) / ms) * ms, start = end - full;
  const birth = T(VIM_BIRTH(vim)), out = [];
  let t = start; const k0 = VIM_ORDER.indexOf(lord);
  for (let i = 0; i < 9; i++) {
    const l = VIM_ORDER[(k0 + i) % 9], len = full * VIM_YEARS[l] / 120, a = t, z = t + len; t = z;
    if (z <= birth) continue;
    out.push({ lord: cap(l), start_utc: new Date(Math.max(a, birth)).toISOString(), end_utc: new Date(z).toISOString(), years: len / ms, computed: true, clipped_at_birth: a < birth });
  }
  return out;
}
const VIM_BIRTH = vim => vim.mahadashas[0].start_utc;
let DASHA_PATH = null;
function dashaZoom(vim) {
  const ref = T(REF_UTC || new Date().toISOString());
  if (!DASHA_PATH) { // open on the running chain
    DASHA_PATH = []; let list = vim.mahadashas;
    for (let lvl = 0; lvl < 5 && list.length; lvl++) {
      const i = list.findIndex(p => T(p.start_utc) <= ref && ref < T(p.end_utc)); if (i < 0) break;
      DASHA_PATH.push(i); list = vimChildren(list[i], vim);
    }
  }
  const W = 1000, left = 150, rowH = [30, 24, 19, 15, 12], gap = 34, rows = [];
  let list = vim.mahadashas;
  for (let lvl = 0; lvl < 5 && list && list.length; lvl++) { rows.push(list); const i = DASHA_PATH[lvl]; if (i == null || !list[i]) break; list = vimChildren(list[i], vim); }
  if (rows.length < 5 && DASHA_PATH.length === rows.length && list && list.length) rows.push(list);
  let y = 8; const ys = rows.map((_, k) => { const v = y; y += rowH[k] + gap; return v; });
  const H = y - gap + 26;
  const svg = s('svg', { viewBox: `0 0 ${W} ${H}`, class: 'dzoom', role: 'img', 'aria-label': 'Vimśottarī daśā, five levels' });
  rows.forEach((periods, k) => {
    const t0 = T(periods[0].start_utc), t1 = T(periods[periods.length - 1].end_utc), X = t => left + (t - t0) / (t1 - t0) * (W - left - 4), sel = DASHA_PATH[k];
    const lab = s('text', { x: left - 10, y: ys[k] + rowH[k] / 2 + 4, class: 'dz-lab' }, svg); lab.textContent = DASHA_LEVELS[k];
    const g = s('g', { class: 'dz-row', style: `--k:${k}` }, svg);
    periods.forEach((p, i) => {
      const lord = (p.lord || '').toLowerCase(), col = (G[lord] || G.lagna).dot, a = X(T(p.start_utc)), z = X(T(p.end_utc));
      const r = s('rect', { x: a, y: ys[k], width: Math.max(.8, z - a - .8), height: rowH[k], rx: Math.min(5, rowH[k] / 3), fill: col, class: 'dz-bar' + (i === sel ? ' sel' : sel != null ? ' dim' : '') }, g);
      r.addEventListener('mousemove', ev => tip(ev, `${DASHA_LEVELS[k]}: ${p.lord}`, [['Chain', rows.slice(0, k).map((rr, j) => rr[DASHA_PATH[j]].lord).concat([p.lord]).join(' › ')], ['From', fmtTime(p.start_utc).slice(0, 16)], ['To', fmtTime(p.end_utc).slice(0, 16)],
        ['Length', p.years != null ? (p.years >= 1 ? `${fix(p.years, 2)} years` : `${fix(p.years * 365.25, 1)} days`) : null], ['Source', p.computed ? 'subdivided in the page (Vimśottarī proportions)' : 'engine']], 'click to open its sub-periods'));
      r.addEventListener('mouseleave', untip);
      r.addEventListener('click', () => { DASHA_PATH = DASHA_PATH.slice(0, k).concat([i]); route(); });
      if (z - a > 26) { const t = s('text', { x: (a + z) / 2, y: ys[k] + rowH[k] / 2 + 1, class: 'dz-name', 'font-size': Math.max(8.5, rowH[k] * .5), fill: ink(col) }, g); t.textContent = z - a > 70 ? p.lord : p.lord.slice(0, 2); }
    });
    if (ref > t0 && ref < t1) { const nx = X(ref); s('line', { x1: nx, x2: nx, y1: ys[k] - 4, y2: ys[k] + rowH[k] + 4, class: 'dz-now' }, svg); }
    if (sel != null && rows[k + 1]) { // the zoom bracket: from the chosen period out to the full width of the next row
      const p = periods[sel], a = X(T(p.start_utc)), z = X(T(p.end_utc)), y0 = ys[k] + rowH[k] + 2, y1 = ys[k + 1] - 3;
      s('path', { d: `M${a} ${y0}L${z} ${y0}L${W - 4} ${y1}L${left} ${y1}Z`, class: 'dz-bracket', fill: (G[(p.lord || '').toLowerCase()] || G.lagna).dot }, svg);
    }
    const yr = s('text', { x: left, y: ys[k] + rowH[k] + 12, class: 'dz-date' }, svg); yr.textContent = dateOnly(periods[0].start_utc);
    const yr2 = s('text', { x: W - 4, y: ys[k] + rowH[k] + 12, class: 'dz-date', 'text-anchor': 'end' }, svg); yr2.textContent = dateOnly(periods[periods.length - 1].end_utc);
  });
  const chain = rows.map((rr, j) => DASHA_PATH[j] != null && rr[DASHA_PATH[j]] ? rr[DASHA_PATH[j]] : null).filter(Boolean);
  return h('div', {}, h('div', { class: 'chain dz-chain' }, chain.map((p, j) => h('button', { class: 'link', onclick: () => { DASHA_PATH = DASHA_PATH.slice(0, j); route(); } },
      h('small', {}, DASHA_LEVELS[j]), h('b', {}, glink(p.lord.toLowerCase())), h('small', {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`))),
    h('button', { class: 'btn', onclick: () => { DASHA_PATH = null; route(); } }, 'Back to now')),
    h('div', { class: 'timeline' }, svg),
    note('Click any bar to open its sub-periods below it; the bracket shows the period being zoomed. Levels 1–3 come from the engine; deeper levels are subdivided in the page by the same Vimśottarī proportions (the journeys check the rule against the engine\'s level 3).'));
}

page('dasha', () => 'Daśā', 'Time', (b) => {
  const vim = val('vedic.dasha.vimshottari'), yog = val('vedic.dasha.yogini'), ss = val('vedic.sade_sati');
  if (!vim && !yog) return put(b, missing('Daśā', 'It needs an exact birth time.'));
  const out = [pageHead('dasha', 'Daśā periods', 'Zoom through five levels: click a period to open its sub-periods beneath it. The dashed line marks the reference instant recorded in this file.')];
  if (vim) {
    out.push(card(`Vimśottarī · balance at birth ${fix(vim.balance_at_birth_years, 3)} years of ${vim.starting_lord}`, dashaZoom(vim)));
    if (vim.current) out.push(card(`Current chain at ${fmtTime(vim.current.at_utc)}`, h('div', { class: 'chain' }, vim.current.periods.map(p => h('div', { class: 'link' }, h('small', {}, p.level), h('b', {}, glink(p.lord.toLowerCase())), h('small', {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`))))));
  }
  if (yog) out.push(card(`Yoginī · starts with ${yog.starting_yogini}, balance ${fix(yog.balance_at_birth_years, 3)} years`, timeline(yog.mahadashas, 'yogini', 'yoginī daśā', null),
    yog.current ? h('div', { class: 'chain' }, yog.current.periods.map(p => h('div', { class: 'link' }, h('small', {}, p.level), h('b', {}, p.lord), h('small', {}, `${dateOnly(p.start_utc)} → ${dateOnly(p.end_utc)}`)))) : null));
  if (ss) out.push(card(`Sāḍe Sātī${ss.active_at_reference ? ' · active now' : ''}`, table(['From', 'To', 'Phases'], ss.windows.map(w => [dateOnly(w.start), dateOnly(w.end), w.phases.map(p => `${p.phase} ${dateOnly(p.start)}→${dateOnly(p.end)}`).join(' · ')])), ss.note ? note(ss.note) : null));
  if (vim) out.push(h('details', { class: 'card' }, h('summary', {}, 'Every mahādaśā and antardaśā'), table(['Mahādaśā', 'Antardaśā', 'From', 'To'], vim.mahadashas.flatMap(m => (m.antardashas || []).map(a => [m.lord, a.lord, dateOnly(a.start_utc), dateOnly(a.end_utc)])))));
  put(b, out);
}, { blurb: 'Vimśottarī, Yoginī, Sāḍe Sātī' });

