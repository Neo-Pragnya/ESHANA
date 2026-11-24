/* ESHANA Studio: the live engine (ADR-013), the compare workspace and the chart playground.
   A file made with `eshana-report --live` embeds the engine compiled to WebAssembly (#eshana-engine) and recomputes in
   the browser with the same execute_json as the CLI and Python; nothing is fetched. Without it the file is
   report-only: the compare workspace reads the series embedded in the file, and the playground says it needs the
   live engine. No rule lives here: every value shown comes from an engine envelope. */
const LIVE = { present: !!document.getElementById('eshana-engine'), ready: false, error: null, execute: null };
(async () => {
  if (!LIVE.present) return;
  try {
    const b64 = document.getElementById('eshana-engine').textContent.trim();
    const bin = Uint8Array.from(atob(b64), c => c.charCodeAt(0));
    const { instance } = await WebAssembly.instantiate(bin, {});
    const x = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
    LIVE.execute = (req) => {
      const b = enc.encode(JSON.stringify(req)), p = x.eshana_alloc(b.length);
      new Uint8Array(x.memory.buffer, p, b.length).set(b);
      const out = x.eshana_execute(p, b.length), n = new DataView(x.memory.buffer).getUint32(out, true);
      const s = dec.decode(new Uint8Array(x.memory.buffer, out + 4, n).slice());
      x.eshana_free(out, 4 + n); x.eshana_free(p, b.length);
      return JSON.parse(s);
    };
    LIVE.ready = true;
  } catch (e) { LIVE.error = String(e && e.message || e); }
  if (CURRENT === 'compare' || CURRENT === 'playground') route();
})();

/* The natal request, rebuilt from the embedded Vedic envelope: the same instant, place and settings. */
function natalRequest() {
  if (!VED) return null;
  const c = VED.calculation || {}, i = VED.input || {}, pl = i.place || null;
  return { when: { utc: i.instant_utc }, place: pl ? { latitude: pl.latitude_deg, longitude: pl.longitude_deg } : null,
    config: { zodiac: 'sidereal', ayanamsha: c.ayanamsha, provider: c.provider, node_model: c.node_model } };
}
const unavailableRow = (why) => h('p', { class: 'note na' }, 'Unavailable: ' + why);

/* ── Compare workspace: natal, progressed and transit side by side, one shared date ─────────────────────────────────
   One context object; every change bumps its sequence number and every panel is redrawn from it. An answer computed
   for an older sequence is dropped, so a panel never shows a result for a date or setting that is no longer chosen. */
const CMP = { date: null, seq: 0 };
function cmpRows(lons) {
  return GRAHA_ORDER.filter(g => lons[g] != null).map(g => [g, `${SIGN_SKT[signOf(lons[g])]} ${inSign(lons[g])}`]);
}
function lonsFromPositions(ps) { const o = {}; for (const p of ps || []) o[engineId(p.id)] = p.longitude_deg; return o; }
function panel(id, title, stamp, body) {
  return h('section', { class: 'card cmp-panel', 'data-panel': id, 'data-stamp': stamp || '', 'aria-label': title },
    h('h3', {}, title), h('p', { class: 'note cmp-stamp' }, stamp ? 'as of ' + stamp : ''), body);
}
function transitFor(date) {
  if (LIVE.ready) {
    const n = natalRequest();
    const env = LIVE.execute({ op: 'positions', when: { utc: date + 'T00:00:00Z' }, config: n.config });
    return env.kind === 'error' ? { na: env.error.code + ': ' + env.error.message } : { lons: lonsFromPositions(env.positions) };
  }
  const rows = (D.transits && D.transits.rows) || [];
  const r = rows.find(x => x.utc.slice(0, 10) === date);
  if (!r) return { na: rows.length ? `this file holds transits from ${dateOnly(rows[0].utc)} to ${dateOnly(rows[rows.length - 1].utc)}; make it with --live to choose any date` : 'this file has no transit series' };
  const o = {}; for (const [k, v] of Object.entries(r.longitudes || {})) o[engineId(k)] = v; return { lons: o };
}
function progressedFor(date) {
  const n = natalRequest();
  if (!n || !n.place) return { na: 'progressions need the birth place and time' };
  if (LIVE.ready) {
    const env = LIVE.execute({ op: 'progressions', natal: n, at_utc: date + 'T00:00:00Z', angle_method: 'naibod_ra' });
    if (env.kind === 'error') return { na: env.error.code + ': ' + env.error.message };
    return { lons: lonsFromPositions(env.positions), note: `age ${fix(env.calculation.age_years, 3)} years; angles by Naibod (RA)` };
  }
  return { na: 'progressed positions for a chosen date need the live engine (eshana-report --live)' };
}
page('compare', () => 'Compare', 'Time', (b) => {
  if (!VED) return put(b, missing('The compare workspace', 'It needs a Vedic chart.'));
  if (LIVE.present && !LIVE.ready && !LIVE.error) return put(b, note('Starting the engine in this page…'));
  const rows = (D.transits && D.transits.rows) || [];
  if (!CMP.date) CMP.date = (D.reference_utc || (rows[0] && rows[0].utc) || new Date().toISOString()).slice(0, 10);
  const seq = ++CMP.seq;
  const input = h('input', { type: 'date', value: CMP.date, 'aria-label': 'Date for the progressed and transit panels' });
  const shift = (d) => { const t = new Date(CMP.date + 'T00:00:00Z'); t.setUTCDate(t.getUTCDate() + d); CMP.date = t.toISOString().slice(0, 10); route(); };
  input.addEventListener('change', () => { if (input.value) { CMP.date = input.value; route(); } });
  const grid = h('div', { class: 'cmp-grid' });
  const natal = lonsFromPositions(VED.positions);
  const draw = () => {
    if (seq !== CMP.seq) return; // a newer context replaced this one
    const pr = progressedFor(CMP.date), tr = transitFor(CMP.date);
    if (seq !== CMP.seq) return;
    put(grid,
      panel('natal', 'Natal', dateOnly(VED.input && VED.input.instant_utc), table(['Graha', 'Position'], cmpRows(natal))),
      panel('progressed', 'Progressed (a day for a year)', CMP.date, pr.na ? unavailableRow(pr.na) : [table(['Graha', 'Position'], cmpRows(pr.lons)), note(pr.note)]),
      panel('transit', 'Transit', CMP.date, tr.na ? unavailableRow(tr.na) : table(['Graha', 'Position'], cmpRows(tr.lons))));
  };
  put(b, pageHead('transits', 'Compare: natal, progressed and transit', LIVE.ready ? 'Computed in this page by the embedded engine.' : 'Report-only file: panels read the series embedded at generation.'),
    h('div', { class: 'card cmp-bar' }, h('label', {}, 'Date ', input),
      h('button', { class: 'btn', type: 'button', onclick: () => shift(-1), 'aria-label': 'Previous day' }, '‹'),
      h('button', { class: 'btn', type: 'button', onclick: () => shift(1), 'aria-label': 'Next day' }, '›'),
      LIVE.error ? note('The embedded engine could not start: ' + LIVE.error) : null),
    grid);
  draw();
}, { blurb: 'natal, progressed and transit side by side' });

/* ── Playground: place grahas by hand, see what follows (synthetic_chart) ─────────────────────────────────────────
   Keyboard: ↑/↓ choose a graha, ←/→ move it 1°, Shift+←/→ 30°, Ctrl/⌘+Z undo. Notes are kept per graha. */
const PG = { lons: null, asc: null, sel: 0, undo: [], notes: {} };
page('playground', () => 'Playground', 'Systems', (b) => {
  if (!LIVE.present) return put(b, missing('The playground', 'It recomputes as you move grahas, which needs the live engine: make the file with eshana-report --live.'));
  if (!LIVE.ready) return put(b, LIVE.error ? missing('The playground', 'The embedded engine could not start: ' + LIVE.error) : note('Starting the engine in this page…'));
  if (!PG.lons) { PG.lons = {}; for (const g of NINE) if (GR[g]) PG.lons[g] = GR[g].lon; PG.asc = ASC; }
  const ids = Object.keys(PG.lons);
  const bodyId = g => (g === 'rahu' ? 'mean_node' : g);
  const env = LIVE.execute({ op: 'synthetic_chart', ascendant_deg: PG.asc,
    positions: ids.filter(g => g !== 'ketu').map(g => ({ body: bodyId(g), longitude_deg: PG.lons[g], speed_deg_per_day: g === 'rahu' ? -0.05 : 1 })) });
  if (env.kind === 'error') return put(b, missing('The playground', env.error.code + ': ' + env.error.message));
  const syn = env.synthetic, by = syn.bodies || {};
  const move = (g, d) => { PG.undo.push({ ...PG.lons }); if (PG.undo.length > 200) PG.undo.shift(); PG.lons[g] = norm(PG.lons[g] + d); if (g === 'rahu' && 'ketu' in PG.lons) PG.lons.ketu = norm(PG.lons.rahu + 180); route(); };
  const list = h('ul', { class: 'pg-list', role: 'listbox', tabindex: '0', 'aria-label': 'Grahas: arrows choose and move, Ctrl+Z undoes',
    'aria-activedescendant': 'pg-' + ids[PG.sel] },
    ids.map((g, i) => { const v = by[bodyId(g)] || by[g] || {};
      return h('li', { id: 'pg-' + g, role: 'option', 'aria-selected': i === PG.sel ? 'true' : 'false', class: i === PG.sel ? 'sel' : '', onclick: () => { PG.sel = i; route(); } },
        h('b', {}, (G[g] || {}).en || g), ` ${SIGN_SKT[signOf(PG.lons[g])]} ${inSign(PG.lons[g])}`,
        v.nakshatra ? ` · ${v.nakshatra} ${v.pada}` : '', v.dignity ? ` · ${v.dignity}` : '', PG.notes[g] ? ' ✎' : ''); }));
  list.addEventListener('keydown', (e) => {
    const g = ids[PG.sel];
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') { e.preventDefault(); const u = PG.undo.pop(); if (u) { PG.lons = u; route(); } return; }
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); PG.sel = (PG.sel + (e.key === 'ArrowDown' ? 1 : ids.length - 1)) % ids.length; route(); return; }
    if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); move(g, (e.key === 'ArrowRight' ? 1 : -1) * (e.shiftKey ? 30 : 1)); }
  });
  const cur = ids[PG.sel];
  const noteBox = h('textarea', { rows: 3, 'aria-label': `Note for ${(G[cur] || {}).en || cur}`, placeholder: 'Your note for this graha (kept in this page only)' }, PG.notes[cur] || '');
  noteBox.addEventListener('input', () => { PG.notes[cur] = noteBox.value; });
  const yogas = ((syn.derived.find(d => d.id === 'vedic.yogas') || {}).value || {}).yogas || [];
  const present = yogas.filter(y => y.present).map(y => y.name || y.id);
  const na = Object.entries(syn.not_derivable || {}).map(([k, v]) => [k.replace('_', ' '), 'unavailable: ' + v.reason]);
  put(b, pageHead('planets', 'Playground', 'Positions placed by hand. Every value below is derived by the engine from them (labelled synthetic); nothing is read from an ephemeris.'),
    h('div', { class: 'grid2' },
      card('Grahas', list, note('↑/↓ choose · ←/→ move 1° · Shift moves 30° · Ctrl/⌘+Z undo'), h('button', { class: 'btn', type: 'button', onclick: () => { PG.lons = null; PG.undo = []; route(); } }, 'Back to the birth positions')),
      card(`Note: ${(G[cur] || {}).en || cur}`, noteBox)),
    card('Yogas present', present.length ? h('ul', { class: 'list' }, present.map(n => h('li', {}, n))) : note('None.')),
    card('Not derivable from placed positions', facts(na)),
    note('Provenance: ' + (env.provenance.assumptions || []).join('; ')));
  requestAnimationFrame(() => { const l = b.querySelector('.pg-list'); if (l && document.activeElement !== l && CURRENT === 'playground' && PG.focus) l.focus(); });
  list.addEventListener('focus', () => { PG.focus = true; });
}, { blurb: 'move grahas and see what follows' });
