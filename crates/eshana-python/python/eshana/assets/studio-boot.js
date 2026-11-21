/* ESHANA Studio: boot. Loads last, after every asset has registered its pages. */
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
  applyVisibility(); buildNav(); palette(); startSky(); route();
})();
