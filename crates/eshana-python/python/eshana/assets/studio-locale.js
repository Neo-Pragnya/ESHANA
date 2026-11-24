/* ESHANA Studio: interface language (APP-04). The catalogue (D.locale) relabels page and group names, the toolbar and
   the not-computed notice. Values, numbers, dates, rule ids and error codes are never translated, so the numerical
   output is identical in every language. A catalogue whose status is not `reviewed` or `reference` says so.
   Every file carries every catalogue (D.locales); the language is a reader setting (S.lang, settings drawer), English
   unless the file was written with --locale or the reader chose another. */
(function applyLocale() {
  const L = (D.locales || {})[S.lang] || D.locale; if (!L) return;
  document.documentElement.lang = L.lang || 'en';
  for (const [id, t] of Object.entries(L.pages || {})) if (PAGES[id] && !PAGES[id].hidden) PAGES[id].title = () => t;
  for (const p of Object.values(PAGES)) if (L.groups && L.groups[p.group]) p.group = L.groups[p.group];
  const ui = L.ui || {};
  for (const id of ['search', 'settings', 'theme', 'print', 'json', 'copy']) { const t = document.querySelector(`#${id} .t`); if (t && ui[id]) t.textContent = ui[id]; }
  const nc = document.getElementById('not-computed');
  if (nc && ui.not_computed) { nc.querySelector('h3').textContent = ui.not_computed; const p = nc.querySelector('p.note'); if (p && ui.not_computed_note) p.textContent = ui.not_computed_note; }
  if (L.status !== 'reviewed' && L.status !== 'reference' && ui.translation_note) {
    const m = document.querySelector('main'); if (m) m.prepend(h('p', { class: 'note locale-draft', role: 'note' }, ui.translation_note));
  }
})();
