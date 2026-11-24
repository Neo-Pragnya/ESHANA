/* ESHANA Studio: download any chart as an image (PNG or SVG), exactly as it is drawn now.
   Every chart drawing (an <svg role="img"> at least 160 px wide) gets a small bar in its top-right corner. The export
   is a snapshot of the live drawing: the current layers, filters, varga, rotation, transit date, theme and language, with
   every CSS variable and style resolved into the file, so it looks the same outside the studio. The PNG is rendered at
   print resolution (at least 1600 px wide) on the page's background and carries a caption: the person, the birth
   moment and place, the chart's title, and "ESHANA" with the engine version. Nothing is sent anywhere. */
const EXPORT_PROPS = ['fill', 'fill-opacity', 'fill-rule', 'stroke', 'stroke-width', 'stroke-opacity', 'stroke-dasharray', 'stroke-dashoffset',
  'stroke-linecap', 'stroke-linejoin', 'opacity', 'font-family', 'font-size', 'font-weight', 'font-style', 'letter-spacing', 'text-anchor',
  'dominant-baseline', 'alignment-baseline', 'baseline-shift', 'text-decoration', 'visibility', 'display', 'filter', 'mix-blend-mode',
  'paint-order', 'stop-color', 'stop-opacity', 'clip-path', 'mask', 'marker-start', 'marker-mid', 'marker-end', 'vector-effect'];
const EXPORT_SKIP = /\b(pcard-icon|hero-icon|bird)\b/;

function exportLabel(svg) {
  const card = svg.closest('.card, .figure, .timeline');
  const head = card && (card.querySelector('h2, h3, h4, .card-title') || card.closest('.card')?.querySelector('h2, h3, h4'));
  const sec = svg.closest('section.page');
  const aria = (svg.getAttribute('aria-label') || '').split(/[:;(]/)[0].trim();
  return (head && head.textContent.trim()) || aria || (sec && sec.querySelector('h1')?.textContent.trim()) || 'chart';
}
/* button labels in the reader's interface language (ui.download_* in the locale catalogue), English otherwise */
const EXPORT_TEXT = { download_group: 'Download {label} as an image', download_title: 'Download “{label}” as {kind}, as shown now', download_aria: 'Download {label} as {kind}', download_error: 'Could not create the image: ' };
function exportText(key, vars = {}) {
  const L = (D.locales || {})[S.lang] || D.locale || {}, t = (L.ui && L.ui[key]) || EXPORT_TEXT[key];
  return t.replace(/\{(\w+)\}/g, (_, k) => vars[k] != null ? vars[k] : '');
}
function exportSlug(t) { return String(t || '').normalize('NFKD').replace(/[̀-ͯ]/g, '').replace(/[^A-Za-z0-9]+/g, '-').replace(/^-|-$/g, '').toLowerCase().slice(0, 48) || 'chart'; }
function exportName(svg, ext) { const who = D.subject && D.subject.name; return (who ? exportSlug(who) + '-' : '') + exportSlug(exportLabel(svg)) + '.' + ext; }

/* the drawing as a standalone SVG document string, styles resolved; width/height in px when given */
function exportSVGText(svg, w, h) {
  const clone = svg.cloneNode(true), src = [svg, ...svg.querySelectorAll('*')], dst = [clone, ...clone.querySelectorAll('*')];
  src.forEach((el, i) => {
    const cs = getComputedStyle(el), out = dst[i], st = [];
    for (const p of EXPORT_PROPS) { const v = cs.getPropertyValue(p); if (v !== '') st.push(`${p}:${v}`); }
    // CSS-driven transforms (the turning wheel); attribute transforms are copied with the element already
    if (!el.hasAttribute('transform') && cs.transform && cs.transform !== 'none') st.push(`transform:${cs.transform}`, `transform-origin:${cs.transformOrigin}`, `transform-box:${cs.transformBox}`);
    out.setAttribute('style', st.join(';'));
    for (const a of [...out.attributes]) if (/var\(/.test(a.value) && a.name !== 'style') out.removeAttribute(a.name);
    out.removeAttribute('tabindex');
  });
  // a url(#id) that points outside the drawing is copied in, so the file stands alone
  const need = new Set(), have = new Set([...clone.querySelectorAll('[id]')].map(e => e.id));
  const scan = (root) => { for (const e of [root, ...root.querySelectorAll('*')]) for (const a of e.attributes) for (const m of a.value.matchAll(/url\(["']?#([^"')]+)["']?\)|^#([\w-]+)$/g)) { const id = m[1] || m[2]; if (id && !have.has(id)) need.add(id); } };
  scan(clone);
  if (need.size) { const defs = s('defs', {}); for (const id of need) { const e = document.getElementById(id); if (e) { defs.appendChild(e.cloneNode(true)); have.add(id); } } clone.prepend(defs); }
  clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg'); clone.setAttribute('xmlns:xlink', 'http://www.w3.org/1999/xlink');
  if (w) { clone.setAttribute('width', w); clone.setAttribute('height', h); }
  return '<?xml version="1.0" encoding="UTF-8"?>\n' + new XMLSerializer().serializeToString(clone);
}

/* the background behind the drawing: the page colour, then any card or figure colour on top of it */
function exportBackground(svg) {
  const layers = []; for (let e = svg.parentElement; e; e = e.parentElement) { const c = getComputedStyle(e).backgroundColor; if (c && c !== 'transparent' && !/rgba\(.*,\s*0\)$/.test(c)) layers.unshift(c); }
  const page = getComputedStyle(document.documentElement).getPropertyValue('--bg').trim() || (document.documentElement.dataset.mode === 'light' ? '#ffffff' : '#0b0d22');
  return [page, ...layers];
}

function exportCaption(svg) {
  const sj = D.subject || {}, v = ((Object.values(D.envelopes || {})[0] || {}).provenance || {}).engine_version || '';
  return { title: [sj.name, exportLabel(svg)].filter(Boolean).join(' · '), sub: [sj.local, sj.place_label].filter(Boolean).join(' · '), mark: 'ESHANA' + (v ? ' ' + v : '') };
}

/* a PNG blob of the drawing with its caption */
async function exportPNG(svg) {
  const r = svg.getBoundingClientRect(), vb = svg.viewBox && svg.viewBox.baseVal;
  const aspect = vb && vb.width ? vb.height / vb.width : (r.height / r.width || 1);
  const W = Math.round(Math.min(4000, Math.max(1600, r.width * 2))), Hd = Math.round(W * aspect);
  const pad = Math.round(W * .03), capH = Math.round(W * .05), CW = W + 2 * pad, CH = Hd + 2 * pad + capH;
  const img = new Image(); img.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(exportSVGText(svg, W, Hd)); await img.decode();
  const c = document.createElement('canvas'); c.width = CW; c.height = CH; const x = c.getContext('2d');
  for (const bg of exportBackground(svg)) { x.fillStyle = bg; x.fillRect(0, 0, CW, CH); }
  x.drawImage(img, pad, pad, W, Hd);
  const cap = exportCaption(svg), ink = getComputedStyle(document.body).color, muted = getComputedStyle(document.documentElement).getPropertyValue('--muted').trim() || ink;
  const font = getComputedStyle(document.body).fontFamily, y0 = pad + Hd + capH * .5;
  x.textBaseline = 'middle'; x.fillStyle = ink; x.font = `700 ${Math.round(capH * .34)}px ${font}`; x.fillText(cap.title, pad, y0 - capH * .05, W * .7);
  x.fillStyle = muted; x.font = `500 ${Math.round(capH * .24)}px ${font}`; x.fillText(cap.sub, pad, y0 + capH * .32, W * .7);
  x.textAlign = 'right'; x.font = `700 ${Math.round(capH * .26)}px ${font}`; x.fillText(cap.mark, pad + W, y0 + capH * .1);
  return await new Promise((ok, no) => c.toBlob(b => b ? ok(b) : no(new Error('the browser could not encode the image')), 'image/png'));
}

function exportSave(blob, name) {
  const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = name; document.body.append(a); a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1500);
}
async function exportDownload(svg, kind, btn) {
  const was = btn.textContent; btn.disabled = true; btn.textContent = '…';
  try {
    const blob = kind === 'png' ? await exportPNG(svg) : new Blob([exportSVGText(svg)], { type: 'image/svg+xml' });
    exportSave(blob, exportName(svg, kind));
  } catch (e) { console.warn('image export failed', e); btn.title = exportText('download_error') + e.message; }
  finally { btn.disabled = false; btn.textContent = was; }
}

/* the bar: one per drawing, placed over its top-right corner, kept in place when the layout changes */
function placeImgBar(bar) {
  const svg = bar._svg, host = bar.parentElement; if (!svg.isConnected || !host) return bar.remove();
  const hr = host.getBoundingClientRect(), r = svg.getBoundingClientRect();
  bar.style.top = Math.max(0, r.top - hr.top + 6) + 'px'; bar.style.right = Math.max(0, hr.right - r.right + 6) + 'px';
}
function scanImgBars() {
  $$('.imgbar').forEach(b => { if (!b._svg || !b._svg.isConnected) b.remove(); else if (!b._strip) placeImgBar(b); });
  for (const svg of $$('section.page:not([hidden]) svg[role="img"]')) {
    if (svg._imgbar || EXPORT_SKIP.test(svg.getAttribute('class') || '') || svg.closest('.imgbar, .mini-chart')) continue;
    if (svg.getBoundingClientRect().width < 160) continue;
    // above the drawing, outside it, when it sits in a figure box; a round wheel keeps the bar over its empty corner
    const host = svg.parentElement, strip = host.classList.contains('figure') && !svg.classList.contains('wheel');
    if (!strip && getComputedStyle(host).position === 'static') host.classList.add('imghost');
    const label = exportLabel(svg);
    const bar = h('div', { class: 'imgbar' + (strip ? ' imgstrip' : ''), role: 'group', 'aria-label': exportText('download_group', { label }) },
      ...['png', 'svg'].map(k => h('button', { type: 'button', class: 'imgbtn', 'data-kind': k, title: exportText('download_title', { label, kind: k.toUpperCase() }),
        'aria-label': exportText('download_aria', { label, kind: k.toUpperCase() }), onclick: (ev) => { ev.stopPropagation(); exportDownload(svg, k, ev.currentTarget); } }, '⤓ ' + k.toUpperCase())));
    bar._svg = svg; svg._imgbar = bar; bar._strip = strip;
    if (strip) host.insertBefore(bar, svg); else { host.append(bar); placeImgBar(bar); }
  }
}
let imgScanQueued = false;
const queueImgScan = () => { if (imgScanQueued) return; imgScanQueued = true; requestAnimationFrame(() => { imgScanQueued = false; scanImgBars(); }); };
new MutationObserver(muts => { if (muts.some(m => [...m.addedNodes, ...m.removedNodes].some(n => n.nodeType === 1 && !(n.classList && n.classList.contains('imgbar'))))) queueImgScan(); })
  .observe(document.querySelector('main') || document.body, { childList: true, subtree: true });
addEventListener('resize', queueImgScan); addEventListener('hashchange', () => setTimeout(queueImgScan, 50));
window.eshanaExport = { svgText: exportSVGText, png: exportPNG, name: exportName, scan: scanImgBars };
