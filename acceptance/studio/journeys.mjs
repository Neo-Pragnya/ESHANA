// Browser journeys for ESHANA Studio (the chart report app): real Chrome, real time, real clicks.
//
//   node acceptance/studio/journeys.mjs <report.html> [--shots <dir>]
//
// Loads a generated studio file and walks what a reader does: every page renders without console errors; the
// Jyotish Chakra (Graha-Nakshatra Mandala) draws 27 nakshatras, 108 padas and a point for every graha at its longitude; the South Indian centre
// sits fully inside the wheel with every graha in the box of its sign; the South and North Indian charts put grahas in
// the right signs and houses and show translucent aspect marks; clicking a graha opens its page with aspects and
// yogas; the Nadi matrix places grahas by element and direction; the command palette jumps; the theme toggles; and
// phone width has no horizontal scroll. Prints one line per journey and a JSON summary; exits 1 on any failure.
// Needs Node 22+ and Chrome (CHROME=… to override).
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";

const args = process.argv.slice(2);
const file = args[0] && resolve(args[0]);
const shots = args.includes("--shots") ? resolve(args[args.indexOf("--shots") + 1]) : null;
if (!file || !existsSync(file)) {
  console.error("usage: journeys.mjs <report.html> [--shots <dir>]");
  process.exit(2);
}
if (shots) mkdirSync(shots, { recursive: true });
const CHROME = process.env.CHROME || ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"].find(existsSync);
if (!CHROME) { console.error("Chrome not found; set CHROME=/path/to/chrome"); process.exit(2); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const server = createServer((req, res) => {
  if ((req.url || "/").split("?")[0] !== "/" + basename(file)) return void res.writeHead(404).end();
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" }).end(readFileSync(file));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const url = `http://127.0.0.1:${server.address().port}/${basename(file)}`;

const port = 9500 + Math.floor(Math.random() * 400);
const chrome = spawn(CHROME, ["--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "studio-"))}`,
  "--no-first-run", "--no-default-browser-check", "--use-mock-keychain", "--password-store=basic", "--hide-scrollbars", "about:blank"], { stdio: "ignore" });
const WATCHDOG_MS = Number(process.env.JOURNEYS_TIMEOUT_MS || 480000);
setTimeout(() => { console.log(`✗ journeys timed out after ${WATCHDOG_MS / 1000}s`); chrome.kill(); process.exit(1); }, WATCHDOG_MS).unref();
let target;
for (let i = 0; i < 100 && !target; i++) { await sleep(150); try { target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page"); } catch {} }
if (!target) { console.log("✗ Chrome did not expose a DevTools target"); process.exit(1); }
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0; const pending = new Map(); let problems = [];
ws.addEventListener("message", (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
  if (msg.method === "Runtime.exceptionThrown") problems.push("exception: " + (msg.params.exceptionDetails.exception?.description || msg.params.exceptionDetails.text).split("\n")[0]);
  if (msg.method === "Runtime.consoleAPICalled" && msg.params.type === "error") problems.push("console.error: " + msg.params.args.map((a) => a.value ?? a.description).join(" ").slice(0, 200));
});
// every DevTools call has a deadline, so a call that never answers fails its journey (named) instead of hanging the run
const CALL_MS = Number(process.env.JOURNEYS_CALL_TIMEOUT_MS || 30000);
const send = (method, params = {}) => new Promise((r, reject) => {
  const i = ++id;
  const timer = setTimeout(() => { pending.delete(i); reject(new Error(`DevTools ${method} did not answer within ${CALL_MS / 1000}s`)); }, CALL_MS);
  pending.set(i, (msg) => { clearTimeout(timer); r(msg); });
  ws.send(JSON.stringify({ id: i, method, params }));
});
const evaluate = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true });
  if (r.result?.exceptionDetails) throw new Error("eval failed: " + (r.result.exceptionDetails.exception?.description || r.result.exceptionDetails.text));
  return r.result?.result?.value;
};
await send("Runtime.enable"); await send("Page.enable");
const viewport = (width, height, mobile = false) => send("Emulation.setDeviceMetricsOverride", { width, height, deviceScaleFactor: 1, mobile });
const go = async (hash, settle = 1400) => { await evaluate(`location.hash = ${JSON.stringify(hash)}`); await sleep(settle); };
const shot = async (name) => { if (!shots) return; const r = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false }); writeFileSync(join(shots, name + ".png"), Buffer.from(r.result.data, "base64")); };
const mouse = (x, y, type = "mouseMoved") => send("Input.dispatchMouseEvent", { type, x, y, button: "left", clickCount: type === "mouseMoved" ? 0 : 1 });
const click = async (x, y) => { await mouse(x, y); await mouse(x, y, "mousePressed"); await mouse(x, y, "mouseReleased"); };

const results = [];
// JOURNEY_ONLY=<text> runs only the journeys whose name contains that text (handy while fixing one)
const ONLY = process.env.JOURNEY_ONLY || "";
async function journey(name, fn) {
  if (ONLY && !name.includes(ONLY)) return;
  const started = Date.now(); problems = [];
  try {
    const detail = (await fn()) || "";
    if (problems.length) throw new Error(problems.slice(0, 3).join(" | "));
    results.push({ name, ok: true, ms: Date.now() - started, detail }); console.log(`✓ ${name}${detail ? ` (${detail})` : ""}`);
  } catch (e) {
    results.push({ name, ok: false, ms: Date.now() - started, detail: String(e.message || e) }); console.log(`✗ ${name}: ${e.message || e}`);
  }
}
const expect = (cond, message) => { if (!cond) throw new Error(message); };

await viewport(1440, 1000);
await send("Page.navigate", { url: url + "#/overview" });
await sleep(2500);
// make the run deterministic: no saved settings from an earlier run
await evaluate(`localStorage.clear(); 1`);
await send("Page.navigate", { url: url + "#/overview" });
await sleep(2500);

const model = await evaluate(`(() => { const d = JSON.parse(document.getElementById('eshana-data').textContent); const v = d.envelopes.vedic;
  const pos = {}; for (const p of v.positions) pos[{mean_node:'rahu',true_node:'rahu',south_node:'ketu'}[p.id] || p.id] = p.longitude_deg;
  const up = (v.derived_attributes.find(a => a.id === 'vedic.upagrahas') || {}).value || {}; for (const k in up) if (up[k] && up[k].longitude_deg != null) pos[k] = up[k].longitude_deg;
  return { pos, asc: v.houses && v.houses.ascendant ? v.houses.ascendant.longitude_deg : null, pages: d.pages }; })()`);
const signOf = (l) => Math.floor(((l % 360) + 360) % 360 / 30);
const SIGN_AB = ["Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi"];

await journey("every page renders without errors", async () => {
  const pages = model.pages.filter((p) => !["planet", "house", "yoga", "point"].includes(p));
  const empty = [];
  for (const p of pages) {
    await go("#/" + p, 900);
    const n = await evaluate(`document.querySelector('section.page#${p.replace(/"/g, "")}:not([hidden]) .body')?.children.length || 0`);
    const err = await evaluate(`/could not be drawn/.test(document.querySelector('section.page#${p}')?.innerText || '')`);
    if (!n || err) empty.push(p);
  }
  expect(!empty.length, "empty or failed pages: " + empty.join(", "));
  return `${pages.length} pages`;
});

await journey("the chakra draws every ring and a point for every graha", async () => {
  await go("#/chakra", 1500);
  await evaluate(`[...document.querySelectorAll('#chakra .tab')].find(b => b.textContent === 'South Indian').click()`);
  await sleep(2000);
  const c = await evaluate(`(() => { const svg = document.querySelector('#chakra svg.wheel'); return { nak: svg.querySelectorAll('.seg[data-kind=nak]').length, pada: svg.querySelectorAll('.seg[data-kind=pada]').length,
    rashi: svg.querySelectorAll('.seg[data-kind=rashi]').length, pts: svg.querySelectorAll('circle.pt').length }; })()`);
  expect(c.nak === 27 && c.pada === 108 && c.rashi === 12, `rings ${JSON.stringify(c)}`);
  expect(c.pts === Object.keys(model.pos).length, `${c.pts} points for ${Object.keys(model.pos).length} grahas`);
  await shot("chakra-south-centre");
  return `27 / 108 / 12 rings, ${c.pts} points`;
});

await journey("the South Indian centre sits inside the wheel with each graha in its sign", async () => {
  const r = await evaluate(`(() => { const svg = document.querySelector('#chakra svg.wheel'); const disc = svg.querySelector('.wheel-g > circle[fill^="url"]').getBoundingClientRect();
    const cx = disc.left + disc.width / 2, cy = disc.top + disc.height / 2, R = disc.width / 2;
    const cells = [...svg.querySelectorAll('.centre .cell')].map(c => { const b = c.querySelector('rect').getBoundingClientRect();
      const corners = [[b.left, b.top], [b.right, b.top], [b.left, b.bottom], [b.right, b.bottom]].map(([x, y]) => Math.hypot(x - cx, y - cy));
      return { sign: +c.dataset.sign, inside: Math.max(...corners) <= R + 1, bodies: [...c.querySelectorAll('.gl')].map(g => g.dataset.body), w: b.width }; });
    return cells; })()`);
  expect(r.length === 12, `${r.length} centre cells`);
  expect(r.every((c) => c.inside), "a centre box crosses the wheel: " + r.filter((c) => !c.inside).map((c) => c.sign).join(","));
  const wrong = [];
  for (const c of r) for (const b of c.bodies) if (b !== "lagna" && signOf(model.pos[b]) !== c.sign) wrong.push(`${b} in ${SIGN_AB[c.sign]}`);
  expect(!wrong.length, "misplaced: " + wrong.join(", "));
  const placed = r.flatMap((c) => c.bodies).filter((b) => b !== "lagna").length;
  expect(placed === Object.keys(model.pos).length, `${placed} grahas placed in the centre`);
  expect(r[0].w >= 70, `centre boxes only ${r[0].w}px wide: too small to read`);
  return `12 boxes inside, ${placed} grahas in the right signs, box ${Math.round(r[0].w)} px`;
});

await journey("hovering a graha point shows its tooltip; clicking a graha opens its page", async () => {
  await go("#/charts", 2000);
  const pos = await evaluate(`(() => { const g = document.querySelector('#charts .chart .gl[data-body=moon]'); g.scrollIntoView({block:'center'}); const b = g.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
  await mouse(pos[0], pos[1]); await sleep(300);
  const tipText = await evaluate(`document.querySelector('.tip.on, .tipdock.on')?.innerText || ''`);
  expect(/Moon/.test(tipText) && /Nakshatra/.test(tipText), "no tooltip: " + tipText.slice(0, 80));
  await click(pos[0], pos[1]); await sleep(1500);
  expect(await evaluate(`location.hash`) === "#/planet/moon", "did not open the Moon page");
  const t = await evaluate(`document.querySelector('#planet').innerText`);
  for (const need of ["Aspects it casts", "Aspects it receives", "Yogas and doshas", "Vargas", "Nakshatra", "Position"]) expect(t.includes(need), `planet page lacks ${need}`);
  const badOrd = t.match(/\b(1|2|3|21|22|23)th\b/);
  expect(!badOrd, "wrong ordinal: " + (badOrd && badOrd[0]));
  await shot("planet-moon");
  return "tooltip, page with aspects, yogas, vargas";
});

await journey("South and North Indian charts place grahas by sign and house, with aspect marks", async () => {
  await go("#/charts", 1500);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Both').click()`);
  await sleep(1800);
  const r = await evaluate(`(() => { const south = document.querySelector('#charts svg.chart.south'), north = document.querySelector('#charts svg.chart.north');
    const cell = (svg) => [...svg.querySelectorAll('.cell')].map(c => ({ sign: +c.dataset.sign, house: +c.dataset.house || null, bodies: [...c.querySelectorAll('.gl')].map(g => g.dataset.body), marks: c.querySelectorAll('.aspmark').length }));
    return { south: cell(south), north: cell(north), marks: south.querySelectorAll('.aspmark').length, opacity: getComputedStyle(south.querySelector('.aspmark')).opacity,
      pisces: (() => { const c = south.querySelector('.cell[data-sign="11"] rect').getBoundingClientRect(), all = [...south.querySelectorAll('.cell rect')].map(r => r.getBoundingClientRect()); return c.left <= Math.min(...all.map(b => b.left)) + 1 && c.top <= Math.min(...all.map(b => b.top)) + 1; })() }; })()`);
  expect(r.pisces, "Meena is not the top-left box of the South Indian chart");
  const wrong = [];
  for (const c of r.south.concat(r.north)) for (const b of c.bodies) if (b !== "lagna" && signOf(model.pos[b]) !== c.sign) wrong.push(`${b}@${SIGN_AB[c.sign]}`);
  expect(!wrong.length, "misplaced: " + wrong.join(", "));
  if (model.asc != null) {
    const h1 = r.north.find((c) => c.house === 1);
    expect(h1 && h1.sign === signOf(model.asc), "North Indian house 1 does not hold the lagna sign");
    expect(h1.bodies.includes("lagna"), "lagna not marked in house 1");
  }
  expect(r.marks > 0, "no aspect marks");
  expect(Number(r.opacity) < 0.9, `aspect marks are not translucent (opacity ${r.opacity})`);
  await shot("charts-both");
  return `${r.marks} aspect marks at opacity ${r.opacity}`;
});

await journey("vargas gallery shows every division", async () => {
  await go("#/vargas", 2000);
  const n = await evaluate(`document.querySelectorAll('#vargas .gallery .mini-chart svg').length`);
  const divs = await evaluate(`[...document.querySelectorAll('#vargas .gallery .mini-chart h3')].map(h => h.innerText.split(' ')[0])`);
  expect(n >= 16, `only ${n} varga charts`);
  expect(divs.includes("D60") && divs.includes("D9") && divs.includes("D5"), "missing divisions: " + divs.join(","));
  return `${n} charts`;
});

await journey("Nadi matrix places grahas by element and links them", async () => {
  await go("#/nadi", 1800);
  const r = await evaluate(`(() => { const svg = document.querySelector('#nadi svg.nadi'); return { pl: svg.querySelectorAll('.pl').length, signs: svg.querySelectorAll('.nsign').length, w: svg.getBoundingClientRect().width,
    over: [...document.querySelectorAll('#nadi .card')].some(c => c.scrollWidth > c.clientWidth + 1) }; })()`);
  expect(r.signs === 12, `${r.signs} signs in the matrix`);
  expect(r.w >= 480, `matrix only ${Math.round(r.w)} px wide`);
  expect(!r.over, "a Nadi card overflows");
  const grahas = Object.keys(model.pos).filter((k) => !['dhuma', 'vyatipata', 'parivesha', 'indrachapa', 'upaketu', 'gulika', 'mandi'].includes(k)).length;
  expect(r.pl >= grahas, `${r.pl} grahas in the matrix (expected ${grahas})`);
  const pos = await evaluate(`(() => { const g = document.querySelector('#nadi svg.nadi .pl[data-body=jupiter]'); const b = g.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
  await click(pos[0], pos[1]); await sleep(700);
  const links = await evaluate(`document.querySelectorAll('#nadi svg.nadi .nlinks path').length`);
  expect(links > 0, "selecting Jupiter drew no links");
  await shot("nadi");
  return `${r.pl} grahas, ${links} links for Jupiter, ${Math.round(r.w)} px`;
});

await journey("strength, aṣṭakavarga and daśā show their tables", async () => {
  await go("#/strength", 1500);
  const s = await evaluate(`document.querySelector('#strength').innerText`);
  expect(/Ṣaḍbala/.test(s) && /Viṃśopaka/.test(s) && /Bhāva bala/.test(s), "strength tables missing");
  await go("#/ashtakavarga", 1200);
  expect(await evaluate(`/337/.test(document.querySelector('#ashtakavarga').innerText)`), "SAV total 337 not shown");
  await go("#/dasha", 1500);
  expect(await evaluate(`document.querySelectorAll('#dasha .timeline svg rect').length > 8`), "no daśā timeline");
  return "ṣaḍbala, viṃśopaka, bhāva bala, SAV 337, timelines";
});

await journey("hovering a graha on the chakra animates where it aspects", async () => {
  await go("#/chakra", 1600);
  const pos = await evaluate(`(() => { const c = document.querySelector('#chakra svg.wheel .pt-hit[data-body=jupiter]'); c.scrollIntoView({ block: 'center' }); const b = c.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
  await mouse(pos[0], pos[1]); await sleep(700);
  const r = await evaluate(`(() => { const svg = document.querySelector('#chakra svg.wheel'); const f = svg.querySelector(':scope > .aflow');
    const anim = f ? getComputedStyle(f.querySelector('.af')).animationName : null;
    return { flow: f ? f.querySelectorAll('path.af').length : 0, dots: f ? f.querySelectorAll('.af-dot animateMotion').length : 0, anim,
      cells: svg.querySelectorAll('.centre .cell.aspected').length, lit: svg.querySelectorAll('.seg.hot[data-kind=rashi]').length }; })()`);
  const expectN = await evaluate(`aspectTargets('jupiter').length`);
  expect(r.flow === expectN && r.flow >= 3, `${r.flow} flow curves for ${expectN} aspects`);
  expect(r.dots === r.flow && r.anim === "af-flow", `curves not animated (${r.dots} moving dots, ${r.anim})`);
  expect(r.cells >= 3, `${r.cells} centre boxes outlined`);
  await shot("chakra-aspect-hover");
  await mouse(5, 5); await sleep(300);
  const gone = await evaluate(`!document.querySelector('#chakra svg.wheel > .aflow') && !document.querySelector('#chakra .cell.aspected')`);
  expect(gone, "the aspect animation did not clear on leaving");
  return `Jupiter: ${r.flow} flowing aspects, ${r.cells} boxes lit`;
});

await journey("chart cells never overlap: ārūḍha padas on top, aspect marks at the bottom, everything inside its rāśi", async () => {
  // every label in every rāśi of the South and North Indian D1 charts, with additional aspects off and on
  const probe = `(() => { const out = { cells: 0, labels: 0, padasTop: 0, padas: 0, bad: [] };
    const name = (e) => e.matches('g.gl') ? 'graha ' + e.dataset.body : e.classList.contains('arudha') ? 'padas' : e.classList.contains('rashimark') ? 'rāśi marks' : e.classList.contains('aspmark') ? 'aspect marks' : 'label ' + e.textContent;
    for (const svg of document.querySelectorAll('#charts svg.chart.south, #charts svg.chart.north')) {
      const kind = svg.classList.contains('north') ? 'North' : 'South';
      for (const cell of svg.querySelectorAll('.cell')) {
        out.cells++;
        const shape = cell.querySelector('rect, polygon'), sr = shape.getBoundingClientRect(), sb = { x: sr.left, y: sr.top, width: sr.width, height: sr.height }, inv = svg.getScreenCTM().inverse(), where = kind + ' ' + (svg.getAttribute('aria-label') || '').split(' ')[0] + ' sign ' + cell.dataset.sign;
        // what is drawn, on screen: a graha row is its text and icon (not the hover halo, which is invisible until hovered)
        const ink = (e) => { const parts = e.matches('g.gl') ? [...e.querySelectorAll('text, image, circle:not(.halo)')] : [e];
          const rs = parts.map(q => q.getBoundingClientRect()).filter(r => r.width > 0 && r.height > 0); if (!rs.length) return { x: 0, y: 0, width: 0, height: 0 };
          const x = Math.min(...rs.map(r => r.left)), y = Math.min(...rs.map(r => r.top)); return { x, y, width: Math.max(...rs.map(r => r.right)) - x, height: Math.max(...rs.map(r => r.bottom)) - y }; };
        const items = [...cell.children].filter(e => e.matches('text, g.gl')).map(e => ({ e, n: name(e), b: ink(e) })).filter(it => it.b.width > 0 && it.b.height > 0);
        out.labels += items.length;
        for (let i = 0; i < items.length; i++) for (let j = i + 1; j < items.length; j++) {
          const a = items[i].b, b = items[j].b, t = 0.75;
          const ox = Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x), oy = Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y);
          if (ox > t && oy > t) out.bad.push(where + ': ' + items[i].n + ' × ' + items[j].n + ' (' + Math.round(ox) + '×' + Math.round(oy) + ' px)');
        }
        for (const it of items) {
          const b = it.b, cx = b.x + b.width / 2, cy = b.y + b.height / 2;
          const inside = kind === 'South' ? b.x >= sb.x - 1 && b.y >= sb.y - 1 && b.x + b.width <= sb.x + sb.width + 1 && b.y + b.height <= sb.y + sb.height + 1
            : [b.x + 1.5, cx, b.x + b.width - 1.5].every(px => shape.isPointInFill(new DOMPoint(px, cy).matrixTransform(inv)));
          if (!inside) out.bad.push(where + ': ' + it.n + ' outside its rāśi');
        }
        const pada = items.find(it => it.n === 'padas');
        if (pada) { out.padas++; const marks = items.filter(it => /marks$/.test(it.n) || it.n.startsWith('graha'));
          if (marks.every(m => pada.b.y + pada.b.height <= m.b.y + 0.75)) out.padasTop++; else out.bad.push(where + ': padas are not above the grahas and aspect marks'); }
      }
    }
    return out; })()`;
  const runs = [];
  for (const extra of [false, true]) {
    await evaluate(`S.layers.arudha = true; S.layers.aspect_marks = true; S.layers.rashi_marks = true; S.layers.extra_aspects = ${extra}; 1`);
    await go("#/overview", 300); await go("#/charts", 1200);
    await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Both').click()`); await sleep(1500);
    runs.push(await evaluate(probe));
  }
  await evaluate(`S.layers.extra_aspects = false; 1`);
  const bad = runs.flatMap(r => r.bad);
  expect(runs[0].cells >= 24, `only ${runs[0].cells} chart cells checked`);
  expect(runs[0].padas > 0, "no ārūḍha padas drawn in the charts");
  expect(!bad.length, `${bad.length} overlap(s): ` + [...new Set(bad)].slice(0, Number(process.env.JOURNEY_DETAIL || 6)).join("; "));
  return `${runs[0].cells} cells, ${runs[0].labels} + ${runs[1].labels} labels, padas on top in ${runs[0].padasTop}/${runs[0].padas} cells, no overlaps (additional aspects off and on)`;
});

await journey("rāśi drishti has its own marks, in the charts and on the chakra", async () => {
  await go("#/charts", 1500);
  const on = await evaluate(`document.querySelectorAll('#charts svg .rashimark').length`);
  expect(on >= 6, `${on} rāśi drishti marks`);
  const lbl = await evaluate(`[...document.querySelectorAll('#charts .tog')].find(l => /rāśi drishti marks/.test(l.textContent)).querySelector('input').click()`);
  void lbl; await sleep(900);
  const off = await evaluate(`document.querySelectorAll('#charts svg .rashimark').length`);
  expect(off === 0, `${off} marks after switching them off`);
  await evaluate(`[...document.querySelectorAll('#charts .tog')].find(l => /rāśi drishti marks/.test(l.textContent)).querySelector('input').click()`); await sleep(900);
  await go("#/chakra", 1500);
  const ch = await evaluate(`document.querySelectorAll('#chakra svg.wheel .rashimark').length`);
  expect(ch >= 6, `${ch} rāśi drishti marks on the chakra`);
  return `${on} in the charts, ${ch} on the chakra, toggle works`;
});

await journey("selecting a segment turns the wheel only when 'turn to selection' is on", async () => {
  await go("#/chakra", 1500);
  const clickRashi = async () => { const p = await evaluate(`(() => { const s = document.querySelector('#chakra svg.wheel .seg[data-kind=rashi][data-i="4"] path'); const b = s.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`); await click(p[0], p[1]); await sleep(900); };
  await evaluate(`(() => { const c = document.getElementById('turn-on-select'); if (!c.checked) c.click(); })()`); await sleep(800);
  await clickRashi();
  const turned = await evaluate(`document.querySelector('#chakra svg.wheel .wheel-g').style.transform`);
  expect(/rotate/.test(turned), "did not turn with the option on: " + turned);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 }); await sleep(500);
  await evaluate(`document.getElementById('turn-on-select').click()`); await sleep(900);
  await clickRashi();
  const still = await evaluate(`({ t: document.querySelector('#chakra svg.wheel .wheel-g').style.transform, hot: document.querySelectorAll('#chakra .seg.hot').length, panel: document.querySelector('#chakra .panel').innerText.slice(0, 40) })`);
  expect(!/rotate\(-?[1-9]/.test(still.t), "turned although the option is off: " + still.t);
  expect(still.hot === 1 && /Simha/.test(still.panel), "selection not shown without turning: " + JSON.stringify(still));
  await evaluate(`document.getElementById('turn-on-select').click()`); await sleep(500);
  return "turns when on, stays put when off";
});

await journey("hover details sit in the panel beside the chakra, not on top of it", async () => {
  await go("#/chakra", 1500);
  const at = "jupiter";
  await evaluate(`document.querySelector('#chakra svg.wheel .pt-hit[data-body=jupiter]').scrollIntoView({ block: 'center' })`);
  await sleep(400);
  const p = await evaluate(`(() => { const r = document.querySelector('#chakra svg.wheel .pt-hit[data-body=jupiter]').getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 }; })()`);
  await mouse(p.x, p.y); await sleep(500);
  const r = await evaluate(`(() => { const d = document.querySelector('#chakra .tipdock'), w = document.querySelector('#chakra svg.wheel').getBoundingClientRect(), dr = d.getBoundingClientRect();
    return { docked: d.classList.contains('on'), text: d.innerText, floating: document.querySelector('.tip.on') != null, beside: dr.left >= w.right - 1 }; })()`);
  expect(r.docked, "hover details did not go to the panel");
  expect(!r.floating, "a floating popup still covers the chakra");
  expect(r.beside, "the hover panel is not beside the chakra");
  expect(r.text.length > 20, "the hover panel is empty");
  await mouse(5, 5); await sleep(300);
  const idle = await evaluate(`!document.querySelector('#chakra .tipdock').classList.contains('on')`);
  expect(idle, "the hover panel stays lit after the pointer leaves");
  return `${at}: details in the side panel`;
});

await journey("dark theme: chart writing has readable contrast", async () => {
  await evaluate(`document.documentElement.dataset.mode = 'dark'; document.documentElement.setAttribute('data-theme', 'dark'); 1`);
  await evaluate(`S.layers.arudha = true; S.layers.extra_aspects = true; 1`);
  await go("#/overview", 300); await go("#/charts", 1200);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Both').click()`); await sleep(1200);
  const r = await evaluate(`(() => {
    const rgb = (c) => { const m = c.match(/[\\d.]+/g); return m ? m.slice(0, 3).map(Number) : null; };
    const lum = ([r, g, b]) => { const f = (v) => { v /= 255; return v <= .03928 ? v / 12.92 : Math.pow((v + .055) / 1.055, 2.4); }; return .2126 * f(r) + .7152 * f(g) + .0722 * f(b); };
    const bg = rgb(getComputedStyle(document.documentElement).getPropertyValue('--panel-solid').trim().replace('#', '').match(/../g).map(h => parseInt(h, 16)).join(','));
    const bad = []; let n = 0;
    // every visible run of text, measured by the element that actually paints it
    const runs = []; for (const svg of document.querySelectorAll('#charts svg.chart')) { const w = document.createTreeWalker(svg, NodeFilter.SHOW_TEXT);
      for (let x = w.nextNode(); x; x = w.nextNode()) if (x.textContent.trim() && !x.parentElement.closest('title')) runs.push(x.parentElement); }
    for (const t of new Set(runs)) {
      const cs = getComputedStyle(t), c = rgb(cs.fill); if (!c) continue;
      let op = 1; for (let e = t; e && e.tagName !== 'svg'; e = e.parentElement) op *= +getComputedStyle(e).opacity;
      const mix = c.map((v, i) => v * op + bg[i] * (1 - op)), L1 = lum(mix), L2 = lum(bg), ratio = (Math.max(L1, L2) + .05) / (Math.min(L1, L2) + .05);
      const need = t.closest('.aspmark') ? 3 : 4.5; n++;
      if (ratio < need) bad.push(t.textContent.trim().slice(0, 12) + ' ' + cs.fill + ' ' + ratio.toFixed(2));
    }
    return { n, bad }; })()`);
  await evaluate(`S.layers.extra_aspects = false; 1`);
  await shot("charts-dark");
  expect(r.n > 100, `only ${r.n} text runs checked`);
  expect(!r.bad.length, `${r.bad.length} low-contrast text run(s): ` + [...new Set(r.bad)].slice(0, 8).join("; "));
  return `${r.n} text runs at 4.5:1 or better (aspect marks 3:1)`;
});

await journey("hovering a transit graha draws a line to where it is now and lights its rāśi, nakshatra and pada", async () => {
  await go("#/chakra", 1500);
  await evaluate(`(() => { const t = [...document.querySelectorAll('#chakra .tog')].find(l => /^transits$/.test(l.textContent.trim())).querySelector('input'); if (!t.checked) t.click(); })()`);
  await sleep(1200);
  const at = await evaluate(`(() => { const m = document.querySelector('#chakra svg.wheel .transit-pl'); if (!m) return null; m.scrollIntoView({ block: 'center' });
    const r = m.getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2, id: m.dataset.body }; })()`);
  expect(at, "no transit grahas on the wheel");
  await sleep(300);
  // conjunct transit grahas overlap on the ring: expect whichever one is on top under the pointer
  const p = await evaluate(`(() => { const r = document.querySelector('#chakra svg.wheel .transit-pl[data-body="${at.id}"]').getBoundingClientRect(), x = r.left + r.width / 2, y = r.top + r.height / 2;
    const top = document.elementFromPoint(x, y)?.closest('.transit-pl'); return { x, y, id: top ? top.dataset.body : null }; })()`);
  expect(p.id, "no transit graha under the pointer");
  at.id = p.id;
  await mouse(p.x, p.y); await sleep(500);
  const r = await evaluate(`(() => { const ray = document.querySelector('#chakra .transit-ray'); const hot = [...document.querySelectorAll('#chakra svg.wheel .seg.hot')].map(e => e.dataset.kind);
    if (!ray) return { ray: false, hot };
    const l = +ray.getAttribute('x1'), len = Math.hypot(+ray.getAttribute('x2') - +ray.getAttribute('x1'), +ray.getAttribute('y2') - +ray.getAttribute('y1'));
    return { ray: true, body: ray.dataset.body, len, hot, tip: (document.querySelector('.tip') || {}).innerText || '' }; })()`);
  await shot("transit-ray");
  expect(r.ray, "no line drawn on hover");
  expect(r.body === at.id, `line belongs to ${r.body}, hovered ${at.id}`);
  expect(r.len > 300, `line too short (${Math.round(r.len)})`);
  for (const k of ["rashi", "nak", "pada"]) expect(r.hot.includes(k), `${k} not lit (lit: ${r.hot.join(",")})`);
  await mouse(5, 5); await sleep(300);
  const gone = await evaluate(`document.querySelectorAll('#chakra .transit-ray').length`);
  expect(gone === 0, "the line stays after the pointer leaves");
  return `${at.id}: line of ${Math.round(r.len)} units, lit ${r.hot.join(", ")}`;
});

await journey("parivartana: each exchanged graha is marked in its partner's sign, and the yoga is listed with both grahas", async () => {
  // exchanges among the seven grahas, worked out from the positions in the file
  const LORD = ["mars", "venus", "mercury", "moon", "sun", "mercury", "venus", "mars", "jupiter", "saturn", "saturn", "jupiter"];
  const seven = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"], pairs = [];
  for (let i = 0; i < 7; i++) for (let j = i + 1; j < 7; j++) { const a = seven[i], b = seven[j];
    if (LORD[signOf(model.pos[a])] === b && LORD[signOf(model.pos[b])] === a) pairs.push([a, b]); }
  await evaluate(`S.layers.extra_aspects = true; S.layers.aspect_marks = true; 1`);
  await go("#/overview", 300); await go("#/charts", 1200);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'South Indian').click()`); await sleep(1200);
  const marks = await evaluate(`[...document.querySelectorAll('#charts svg.chart.south .cell')].flatMap(c => [...c.querySelectorAll('.exchmark')].map(t => ({ sign: +c.dataset.sign, text: t.textContent.trim() })))`);
  await shot("parivartana-marks");
  await evaluate(`S.layers.extra_aspects = false; 1`);
  const want = pairs.flatMap(([a, b]) => [{ g: a, sign: signOf(model.pos[b]) }, { g: b, sign: signOf(model.pos[a]) }]);
  const ab = (g) => ({ sun: "Su", moon: "Mo", mars: "Ma", mercury: "Me", jupiter: "Ju", venus: "Ve", saturn: "Sa" })[g];
  for (const w of want) expect(marks.some(m => m.sign === w.sign && m.text.startsWith(ab(w.g))), `${w.g} not marked in ${SIGN_AB[w.sign]} (marks: ${JSON.stringify(marks)})`);
  // the D1 chart shows each exchange in two signs, the D9 chart none
  expect(marks.length === want.length * 1 || marks.length === want.length * 2, `${marks.length} exchange marks for ${pairs.length} exchange(s)`);
  const yoga = await evaluate(`(() => { const d = JSON.parse(document.getElementById('eshana-data').textContent).envelopes.vedic.derived_attributes.find(a => a.id === 'vedic.yogas');
    return d ? d.value.yogas.filter(y => /parivartana/.test(y.id)).map(y => ({ id: y.id, present: y.present, grahas: y.grahas })) : []; })()`);
  const any = yoga.find(y => y.id === "parivartana");
  expect(any, "no parivartana yoga in the engine output");
  expect(any.present === pairs.length > 0, `parivartana present=${any.present} but ${pairs.length} exchange(s)`);
  for (const [a, b] of pairs) expect(any.grahas.includes(a) && any.grahas.includes(b), `yoga does not name ${a} and ${b}`);
  if (pairs.length) {
    await go("#/yoga/parivartana", 1200);
    const page = await evaluate(`document.querySelector('#yoga')?.innerText || ''`);
    for (const [a] of pairs) expect(new RegExp(({ sun: "Sun", moon: "Moon", mars: "Mars", mercury: "Mercury", jupiter: "Jupiter", venus: "Venus", saturn: "Saturn" })[a]).test(page), `yoga page does not name ${a}`);
  }
  return pairs.length ? `${pairs.map(p => p.join("⇄")).join(", ")}: marked in the partner signs; ${yoga.filter(y => y.present).map(y => y.id).join(", ")}` : "no exchange in this chart; none marked";
});

await journey("transits: pick a date and animate over a chosen range", async () => {
  await go("#/chakra", 1500);
  await evaluate(`(() => { const t = [...document.querySelectorAll('#chakra .tog')].find(l => /^transits$/.test(l.textContent.trim())).querySelector('input'); if (!t.checked) t.click(); })()`);
  await sleep(1500);
  const rg = await evaluate(`({ min: document.getElementById('transit-pick').min, max: document.getElementById('transit-pick').max, n: document.querySelectorAll('#chakra svg.wheel .pl').length })`);
  expect(rg.min && rg.max && rg.min < rg.max, "no date range on the picker: " + JSON.stringify(rg));
  const target = "2027-03-15";
  await evaluate(`(() => { const d = document.getElementById('transit-pick'); d.value = '${target}'; d.dispatchEvent(new Event('change')); })()`); await sleep(500);
  const shown = await evaluate(`document.getElementById('transit-date').textContent`);
  expect(shown === target, `picked ${target}, shows ${shown}`);
  const set = (id, v) => evaluate(`(() => { const d = document.getElementById('${id}'); d.value = '${v}'; d.dispatchEvent(new Event('change')); })()`);
  await set("transit-from", "2027-01-01"); await set("transit-to", "2027-02-01");
  await evaluate(`document.getElementById('transit-play').click()`); await sleep(2500);
  const seen = await evaluate(`document.getElementById('transit-date').textContent`);
  await evaluate(`document.getElementById('transit-play').click()`);
  expect(seen >= "2027-01-01" && seen <= "2027-02-01", `played outside the range: ${seen}`);
  await shot("chakra-transits");
  return `data ${rg.min} → ${rg.max}; jump to ${shown}; loop Jan 2027 at ${seen}`;
});

await journey("yogas: only those present, each with a full reading page", async () => {
  await go("#/yogas", 1400);
  const r = await evaluate(`(() => { const tiles = [...document.querySelectorAll('#yogas a.card.yoga')]; return { n: tiles.length, present: YOGA_PRESENT().length, hrefs: tiles.map(t => t.getAttribute('href')) }; })()`);
  expect(r.n === r.present && r.n > 0, `${r.n} tiles for ${r.present} present`);
  await evaluate(`document.querySelector('#yogas a.card.yoga').click()`); await sleep(1400);
  const hash = await evaluate(`location.hash`);
  expect(hash === r.hrefs[0], `opened ${hash}`);
  const t = await evaluate(`document.querySelector('#yoga').innerText`);
  for (const need of ["How it forms here", "When it is active", "Traditionally associated with", "How it is read", "interpretation"]) expect(t.includes(need), `yoga page lacks "${need}"`);
  const lit = await evaluate(`document.querySelectorAll('#yoga .cell.yoga-src').length`);
  expect(lit >= 1, "the forming grahas are not highlighted");
  await shot("yoga-page");
  return `${r.n} present; ${hash} has formation, strength checks, daśā timing and reading`;
});

await journey("lagnas and points: every point explained on its own page", async () => {
  await go("#/special", 1400);
  const ids = await evaluate(`[...document.querySelectorAll('#special a.card.point')].map(a => a.getAttribute('href'))`);
  expect(ids.length >= 12, `${ids.length} point tiles`);
  const bad = [];
  for (const href of ids) {
    await go(href, 700);
    const t = await evaluate(`document.querySelector('#point').innerText`);
    if (!/Why it matters/.test(t) || !/How to read it/.test(t) || !/In this chart/.test(t) || /could not be drawn/.test(t)) bad.push(href);
  }
  expect(!bad.length, "incomplete point pages: " + bad.join(", "));
  await go("#/point/yogi_point", 1200);
  const el = await evaluate(`document.querySelector('#point').innerText`);
  expect(/the elements around it/.test(el) && /Lord/.test(el) && /Nakshatra/.test(el), "yogi point lacks its sign elements");
  await shot("point-page");
  return `${ids.length} points, each with meaning, reading and chart elements`;
});

await journey("chakra: markers ring, curved names, graha legend", async () => {
  await go("#/chakra", 1600);
  const r = await evaluate(`(() => { const svg = document.querySelector('#chakra svg.wheel'); return { mk: svg.querySelectorAll('.markring .mk-seg').length, pts: svg.querySelectorAll('.markring .mk-pt').length,
    names: svg.querySelectorAll('.nkname textPath').length, legend: document.querySelectorAll('#chakra .legend-card tbody tr').length }; })()`);
  expect(r.mk >= 30, `${r.mk} marker segments`);
  expect(r.pts === 14, `${r.pts} uccha/nīca points (expected 14)`);
  expect(r.names >= 39, `${r.names} curved names (27 nakshatras + 12 rāśis)`);
  expect(r.legend >= 9, `${r.legend} legend rows`);
  return `${r.mk} marker segments, 14 uccha/nīca points, ${r.names} curved names, legend of ${r.legend}`;
});

await journey("aspects: every view draws for D1 and D9", async () => {
  await go("#/aspects", 1400);
  const views = ["On houses", "On house + lord", "On and from grahas", "From each graha to houses", "To Śrīpati cusps (table)", "Graha × graha matrix"];
  const bad = [];
  for (const vg of ["D1 · rāśi", "D9 · navāṃśa"]) {
    await evaluate(`[...document.querySelectorAll('#aspects .tab')].find(b => b.textContent === ${JSON.stringify(vg)}).click()`); await sleep(500);
    for (const v of views) {
      await evaluate(`[...document.querySelectorAll('#aspects .tab')].find(b => b.textContent === ${JSON.stringify(v)}).click()`); await sleep(500);
      const n = await evaluate(`document.querySelectorAll('#aspects .mini-bars, #aspects table').length`);
      if (!n) bad.push(`${vg} / ${v}`);
    }
  }
  expect(!bad.length, "empty views: " + bad.join(", "));
  const bars = await evaluate(`(() => { [...document.querySelectorAll('#aspects .tab')].find(b => b.textContent === 'D1 · rāśi').click(); [...document.querySelectorAll('#aspects .tab')].find(b => b.textContent === 'On houses').click(); return document.querySelectorAll('#aspects .mb-bar').length; })()`);
  expect(bars > 30, `${bars} bars on the houses view`);
  return `12 views, ${bars} bars on houses`;
});

await journey("charts: bhāva chalit as concentric squares and as a circle", async () => {
  await go("#/charts", 1400);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Concentric squares').click()`); await sleep(900);
  const sq = await evaluate(`(() => { const svg = document.querySelector('#charts svg.concentric'); if (!svg) return 0; const sz = [...svg.querySelectorAll('.cc-inner rect')].map(r => r.getAttribute('width') + 'x' + r.getAttribute('height')); return svg.querySelectorAll('.cc-outer').length === 12 && new Set(sz).size === 1 && sz.length === 12 ? 12 : -1; })()`);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Circle').click()`); await sleep(900);
  const ci = await evaluate(`document.querySelectorAll('#charts svg.chalit-circle .cc-bhava').length`);
  await evaluate(`[...document.querySelectorAll('#charts .tab')].find(b => b.textContent === 'Off').click()`); await sleep(500);
  expect(sq === 12 && ci === 12, `inner ring ${sq}, circle bhāvas ${ci}`);
  return "12 inner boxes; 12 Śrīpati sectors";
});

await journey("strength: the ratio column shows a dotted cut-off at 1 (no separate chart)", async () => {
  await go("#/strength", 1400);
  const r = await evaluate(`({ bars: document.querySelectorAll('#strength .ratio-bar').length, cuts: document.querySelectorAll('#strength .ratio-bar .cut').length,
    style: getComputedStyle(document.querySelector('#strength .ratio-bar .cut')).borderLeftStyle, chart: document.querySelectorAll('#strength .ratio-chart').length })`);
  expect(r.bars === 7 && r.cuts === 7 && r.style === "dotted" && r.chart === 0, JSON.stringify(r));
  return "7 ratio cells with a dotted cut-off, no separate chart";
});

await journey("bhāva pages: bhāvāt bhāvam, connections and bearing", async () => {
  await go("#/house/7", 1500);
  const t = await evaluate(`document.querySelector('#house').innerText`);
  for (const need of ["Bhāvāt bhāvam", "The lord’s story", "Connections", "Bearing on this bhāva", "Kāraka"]) expect(t.includes(need), `bhāva page lacks ${need}`);
  const nodes = await evaluate(`document.querySelectorAll('#house .bmap .bm-node').length`);
  expect(nodes >= 3, `${nodes} nodes in the connection map`);
  expect(!/127\\.0\\.0\\.1/.test(t), "a link URL is printed as text");
  return `${nodes} connected nodes`;
});

await journey("progressions: by completed revolutions, with a date and a timeline", async () => {
  await go("#/progressions", 1600);
  const r = await evaluate(`({ rows: document.querySelectorAll('#progressions .revtl .rev-seg').length, inner: document.querySelectorAll('#progressions svg.concentric .cc-outer').length, table: document.querySelectorAll('#progressions table tbody tr').length })`);
  expect(r.rows > 50 && r.inner === 12 && r.table >= 9, JSON.stringify(r));
  const before = await evaluate(`document.querySelector('#progressions .hd-row, #progressions table tbody').innerText`);
  await evaluate(`(() => { const d = document.querySelector('#progressions input[type=date]'); d.value = '1900-01-01'; d.dispatchEvent(new Event('change')); })()`); await sleep(600);
  const after = await evaluate(`document.querySelector('#progressions table tbody').innerText`);
  expect(before !== after, "changing the date did not change the progressed positions");
  return `${r.rows} timeline segments; date picker moves the chart`;
});

await journey("numerology: every school, numbers open their meaning", async () => {
  await go("#/numerology", 1400);
  const counts = {};
  for (const [k, label] of [["all", "All"], ["py", "Pythagorean"], ["ch", "Chaldean"], ["ve", "Indian (Vedic)"], ["lo", "Lo Shu"]]) {
    await evaluate(`[...document.querySelectorAll('#numerology .tab')].find(b => b.textContent === ${JSON.stringify(label)}).click()`); await sleep(500);
    counts[k] = await evaluate(`document.querySelectorAll('#numerology .ntile, #numerology .ls-c').length`);
  }
  expect(counts.all > 20 && counts.py > 12 && counts.ch >= 3 && counts.ve >= 2 && counts.lo === 9, JSON.stringify(counts));
  await evaluate(`[...document.querySelectorAll('#numerology .tab')].find(b => b.textContent === 'All').click()`); await sleep(400);
  await evaluate(`document.querySelector('#numerology .ntile').click()`); await sleep(500);
  const open = await evaluate(`document.querySelector('#numerology .ntile.open .n-detail')?.innerText || ''`);
  expect(open.length > 20, "the opened number has no meaning");
  return JSON.stringify(counts);
});

await journey("Human Design: the standard chart with columns, arrows and properties", async () => {
  await go("#/human-design", 1500);
  const r = await evaluate(`({ rows: document.querySelectorAll('#human-design .hd-row').length, arrows: document.querySelectorAll('#human-design .hd-arrow').length, gates: document.querySelectorAll('#human-design .hdgraph .hd-gate').length,
    centers: document.querySelectorAll('#human-design .hdgraph .hd-center').length, props: document.querySelector('#human-design .hd-props')?.innerText || '' })`);
  expect(r.rows === 26 && r.arrows === 4 && r.gates === 64 && r.centers === 9, JSON.stringify({ ...r, props: undefined }));
  for (const need of ["Strategy", "Signature", "Not-self theme", "Incarnation cross", "Motivation"]) expect(r.props.includes(need), `properties lack ${need}`);
  return "26 activations, 4 arrows, 64 gates, 9 centres, properties";
});

await journey("filters: Uranus, Neptune, Pluto and the upagrahas can be hidden everywhere", async () => {
  await go("#/chakra", 1600);
  const count = () => evaluate(`({ outer: document.querySelectorAll('#chakra .pt-hit[data-body=uranus], #chakra .pt-hit[data-body=neptune], #chakra .pt-hit[data-body=pluto]').length,
    upa: document.querySelectorAll('#chakra .pt-hit[data-body=gulika], #chakra .pt-hit[data-body=mandi], #chakra .pt-hit[data-body=dhuma]').length, legend: document.querySelectorAll('#chakra .legend-card tbody tr').length })`);
  const on = await count();
  expect(on.outer === 3 && on.upa === 3, "defaults do not show both: " + JSON.stringify(on));
  const flip = (label) => evaluate(`[...document.querySelectorAll('#chakra .tog')].find(l => l.textContent.trim() === ${JSON.stringify(label)}).querySelector('input').click()`);
  await flip("Uranus, Neptune, Pluto"); await sleep(900); await flip("upagrahas"); await sleep(900);
  const off = await count();
  expect(off.outer === 0 && off.upa === 0 && off.legend < on.legend, "switched off but still shown: " + JSON.stringify(off));
  await go("#/charts", 1200);
  const inCharts = await evaluate(`document.querySelectorAll('#charts .gl[data-body=uranus], #charts .gl[data-body=gulika]').length`);
  expect(inCharts === 0, `${inCharts} hidden points still in the charts`);
  await go("#/chakra", 1200); await flip("Uranus, Neptune, Pluto"); await sleep(900); await flip("upagrahas"); await sleep(900);
  const back = await count();
  expect(back.outer === 3 && back.upa === 3, "did not come back: " + JSON.stringify(back));
  return `shown ${on.legend} rows, hidden ${off.legend}, restored`;
});

await journey("outer planets aspect the 5th, 7th and 9th; the tab is the Jyotish Chakra", async () => {
  const r = await evaluate(`({ ur: DRISHTI.filter(d => d.from === 'uranus').map(d => d.house_from_graha).sort((a, b) => a - b).join(','), title: PAGES.chakra.title(), mandala: !!document.querySelector('#chakra .mandala-t') || true })`);
  expect(r.ur === "5,7,9", "Uranus aspects " + r.ur);
  await go("#/chakra", 1000);
  const h = await evaluate(`document.querySelector('#chakra h2').textContent + ' | ' + (document.querySelector('#chakra .mandala-t') || {}).textContent`);
  expect(/Jyotish Chakra/.test(h) && /Graha-Nakshatra Mandala/.test(h), h);
  await send("Page.reload"); await sleep(2200); await go("#/aspects", 1200);
  const first = await evaluate(`[...document.querySelectorAll('#aspects .tab[aria-selected=true]')].map(t => t.textContent).join('|')`);
  expect(/Graha × graha matrix/.test(first), "aspects did not open on the matrix: " + first);
  return "Ur 5/7/9; Jyotish Chakra · Graha-Nakshatra Mandala; aspects open on the matrix";
});

await journey("daśā zooms through five levels, deeper levels follow the engine's rule", async () => {
  await go("#/dasha", 1500);
  const rows = await evaluate(`document.querySelectorAll('#dasha .dzoom .dz-row').length`);
  expect(rows === 5, `${rows} levels open on the running chain`);
  // the page's subdivision of an antardaśā must reproduce the engine's pratyantardaśās
  const diff = await evaluate(`(() => { const vim = val('vedic.dasha.vimshottari'); let worst = 0;
    for (const md of vim.mahadashas.slice(1, 4)) for (const ad of md.antardashas.slice(0, 3)) { if (!ad.pratyantardashas) continue;
      const mine = vimChildren(Object.assign({}, ad, { pratyantardashas: undefined }), vim);
      ad.pratyantardashas.forEach((p, i) => { worst = Math.max(worst, Math.abs(T(p.end_utc) - T(mine[i].end_utc)) / 6e4); }); }
    return worst; })()`);
  expect(diff < 2, `subdivision differs from the engine by ${diff.toFixed(2)} minutes`);
  const before = await evaluate(`document.querySelector('#dasha .dz-chain').innerText`);
  await evaluate(`document.querySelectorAll('#dasha .dz-row')[0].querySelectorAll('.dz-bar')[2].dispatchEvent(new MouseEvent('click', { bubbles: true }))`); await sleep(800);
  const after = await evaluate(`({ chain: document.querySelector('#dasha .dz-chain').innerText, rows: document.querySelectorAll('#dasha .dzoom .dz-row').length, bracket: document.querySelectorAll('#dasha .dz-bracket').length })`);
  expect(after.chain !== before && after.rows === 2 && after.bracket === 1, "clicking a mahādaśā did not open it: " + JSON.stringify(after));
  return `5 levels; page rule matches engine within ${diff.toFixed(2)} min; click opens the next level`;
});

await journey("bhāvas: lords in houses with bhāvāt bhāvam; each bhāva read as its own lagna", async () => {
  await go("#/houses", 1400);
  const rows = await evaluate(`[...document.querySelectorAll('#houses .card')].find(c => /Lords in houses/.test(c.innerText)).querySelectorAll('tbody tr').length`);
  expect(rows === 12, `${rows} rows in lords-in-houses`);
  await go("#/house/7", 1500);
  const t = await evaluate(`document.querySelector('#house').textContent`);
  expect(/Bhāva 7 as the lagna/.test(t) && /its loss, expense and release/.test(t) && /what sustains and adds to it/.test(t), "as-lagna analysis missing");
  const rel = await evaluate(`[...document.querySelectorAll('#house .chart .cell')].filter(c => c.querySelector('text') && c.querySelector('text').textContent === '1').map(c => +c.dataset.sign)`);
  return `12 lord rows; bhāva 7 as lagna (renumbered chart starts at sign ${rel.join(',')})`;
});

await journey("Human Design: standard by default, no schematic, symmetric silhouette", async () => {
  await go("#/human-design", 1500);
  const r = await evaluate(`({ tabs: [...document.querySelectorAll('#human-design .tab')].map(t => t.textContent), std: !!document.querySelector('#human-design .hd-std'),
    halves: document.querySelectorAll('#human-design .hd-body path').length, mirrored: !!document.querySelector('#human-design .hd-body path[transform]'), props: !!document.querySelector('#human-design .hd-props h3') })`);
  expect(r.std && !r.tabs.includes("Schematic") && r.halves === 2 && r.mirrored && r.props, JSON.stringify(r));
  return "standard chart, tabs " + r.tabs.join(" / ");
});

await journey("concentric squares are even: inner birth chart, outer band of twelve", async () => {
  await go("#/progressions", 1500);
  const r = await evaluate(`(() => { const svg = document.querySelector('#progressions svg.concentric'); const sz = [...svg.querySelectorAll('.cc-inner rect')].map(x => x.getAttribute('width') + 'x' + x.getAttribute('height'));
    const outerTop = [...svg.querySelectorAll('.cc-outer polygon')].map(p => p.getAttribute('points').split(' ').map(q => q.split(',').map(Number))).filter(pp => pp.some(q => q[1] === 0) && pp.length === 4).map(pp => Math.abs(pp[1][0] - pp[0][0]));
    return { inner: sz.length, same: new Set(sz).size, outer: svg.querySelectorAll('.cc-outer').length, tops: outerTop }; })()`);
  expect(r.inner === 12 && r.same === 1 && r.outer === 12 && r.tops.length === 2 && Math.abs(r.tops[0] - r.tops[1]) < .01, JSON.stringify(r));
  return "12 equal inner boxes; even outer sectors";
});

await journey("maitri page: relationship, tārā and dignity grids fill in; hover explains a cell; a graha head links to its page", async () => {
  await go("#/maitri", 1500);
  const r = await evaluate(`({ rel: document.querySelectorAll('#maitri .mt-rel td.mt-c').length, tara: document.querySelectorAll('#maitri .mt-tara td.mt-c').length,
    dig: document.querySelectorAll('#maitri .mt-dig td.mt-c').length, sums: document.querySelectorAll('#maitri .mt-sum').length, legends: document.querySelectorAll('#maitri .mt-legend').length,
    data: (val('vedic.maitri.relationships') || {}).rows?.length || 0 })`);
  expect(r.rel === r.data && r.rel === 72 && r.tara === 90 && r.dig === 144 && r.sums === 9 && r.legends === 3, JSON.stringify(r));
  const pos = await evaluate(`(() => { const c = document.querySelector('#maitri .mt-rel td[data-from=sun][data-to=jupiter]'); c.scrollIntoView({block:'center'}); const b = c.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
  await mouse(pos[0], pos[1]); await sleep(300);
  const tipText = await evaluate(`document.querySelector('.tip.on, .tipdock.on')?.innerText || ''`);
  expect(/Natural/.test(tipText) && /Temporal/.test(tipText) && /Compound/.test(tipText), "no relationship tooltip: " + tipText.slice(0, 120));
  const dpos = await evaluate(`(() => { const c = document.querySelector('#maitri .mt-dig td[data-g=moon][data-d="9"]'); c.scrollIntoView({block:'center'}); const b = c.getBoundingClientRect(); return [b.left + b.width / 2, b.top + b.height / 2]; })()`);
  await mouse(dpos[0], dpos[1]); await sleep(300);
  const dTip = await evaluate(`document.querySelector('.tip.on, .tipdock.on')?.innerText || ''`);
  expect(/Sign lord/.test(dTip) && /Why/.test(dTip), "no dignity tooltip: " + dTip.slice(0, 120));
  await evaluate(`document.querySelector('#maitri .mt-rel thead a.mt-h[href="#/planet/mars"]').click()`); await sleep(1200);
  expect(await evaluate(`location.hash`) === "#/planet/mars", "graha head did not open the Mars page");
  return `${r.rel} relationships, ${r.tara} tārās, ${r.dig} dignity cells; tooltips and link work`;
});

// ── journeys: maitri (add new journeys for this workstream above this line) ──

await journey("additional aspects (retrograde shadow, parivartana) are off by default and appear everywhere when switched on", async () => {
  await go("#/chakra", 1500);
  const off = await evaluate(`document.querySelectorAll('#chakra .extramark').length`);
  expect(off === 0, `${off} additional marks shown by default`);
  const outerRashi = await evaluate(`((val('vedic.drishti.rashi') || {}).aspects || []).some(r => r.from === 'uranus')`);
  expect(outerRashi, "Uranus has no rāśi drishti");
  const flip = () => evaluate(`[...document.querySelectorAll('#chakra .tog')].find(l => l.textContent.trim() === 'additional aspects').querySelector('input').click()`);
  await flip(); await sleep(1000);
  const on = await evaluate(`({ marks: document.querySelectorAll('#chakra .extramark').length, data: ADDITIONAL.length, kinds: [...new Set(ADDITIONAL.map(d => d.basis))].join(',') })`);
  expect(on.data > 0 && on.marks > 0, "switched on but nothing shown: " + JSON.stringify(on));
  await go("#/aspects", 1200);
  const rows = await evaluate(`[...document.querySelectorAll('#aspects .card')].find(c => /Additional aspects/.test(c.textContent))?.querySelectorAll('tbody tr').length || 0`);
  expect(rows === on.data, `${rows} rows in the additional aspects table for ${on.data}`);
  const who = await evaluate(`ADDITIONAL[0].from`);
  await go("#/planet/" + who, 1200);
  const pl = await evaluate(`/Additional aspects it casts/.test(document.querySelector('#planet').textContent)`);
  expect(pl, "planet page lacks its additional aspects");
  await go("#/chakra", 1200); await flip(); await sleep(800);
  const back = await evaluate(`document.querySelectorAll('#chakra .extramark').length`);
  expect(back === 0, "did not switch off");
  return `${on.data} additional aspects (${on.kinds}); ${on.marks} marks; table and planet page`;
});

await journey("daśā systems: tabs for nine systems, sign-based ones zoom with rāśi labels and a running chain", async () => {
  await go("#/dasha", 1500);
  const tabsTxt = await evaluate(`[...document.querySelectorAll("#dasha .dz-tabs .tab")].map(t => t.textContent)`);
  expect(tabsTxt.length === 9 && tabsTxt[0] === "Vimśottarī" && tabsTxt.includes("Kālachakra"), "tabs: " + tabsTxt.join("|"));
  await evaluate(`document.querySelector("#dasha .dz-chain .btn").click()`); await sleep(600); // back to the running chain
  expect(await evaluate(`document.querySelectorAll("#dasha .dzoom .dz-row").length`) === 5, "Vimśottarī lost its five levels");
  await go("#/dasha/chara", 1500);
  const r = await evaluate(`({ sel: document.querySelector("#dasha .dz-tabs .tab[aria-selected=true]").textContent, rows: document.querySelectorAll("#dasha .dzoom .dz-row").length,
    signBars: document.querySelectorAll("#dasha .dzoom .dz-bar.dz-sign").length, glyph: [...document.querySelectorAll("#dasha .dzoom .dz-name")].some(t => /[♈-♓]/.test(t.textContent)),
    running: document.querySelectorAll("#dasha .dz-running .link").length, conv: /K. N. Rao/.test(document.querySelector("#dasha .dz-conv").textContent),
    houseLink: !!document.querySelector("#dasha .dz-chain a[href^='#/house/']") })`);
  expect(r.sel === "Chara" && r.rows === 2 && r.signBars > 12 && r.glyph && r.running === 2 && r.conv && r.houseLink, JSON.stringify(r));
  const before = await evaluate(`document.querySelector("#dasha .dz-chain").innerText`);
  await evaluate(`document.querySelectorAll("#dasha .dz-row")[0].querySelectorAll(".dz-bar")[1].dispatchEvent(new MouseEvent("click", { bubbles: true }))`); await sleep(800);
  const after = await evaluate(`({ chain: document.querySelector("#dasha .dz-chain").innerText, rows: document.querySelectorAll("#dasha .dzoom .dz-row").length, bracket: document.querySelectorAll("#dasha .dz-bracket").length })`);
  expect(after.chain !== before && after.rows === 2 && after.bracket === 1, "clicking a Chara mahādaśā did not open it: " + JSON.stringify(after));
  // every system renders and its antardaśās tile the mahādaśā above
  const sys = ["yogini", "ashtottari", "shodashottari", "narayana", "navamsha", "shoola", "kalachakra"], seen = [];
  for (const id of sys) {
    await go("#/dasha/" + id, 900);
    const x = await evaluate(`(() => { const v = val("vedic.dasha.${id}"); const m = v.mahadashas.find(m => (m.antardashas || []).length);
      return { rows: document.querySelectorAll("#dasha .dzoom .dz-row").length, conv: !!document.querySelector("#dasha .dz-conv") || "${id}" === "yogini",
        tiles: m ? m.antardashas[m.antardashas.length - 1].end_utc === m.end_utc : false }; })()`);
    expect(x.rows >= 2 && x.conv && x.tiles, id + ": " + JSON.stringify(x));
    seen.push(id + ":" + x.rows);
  }
  // Aṣṭottarī opens a third level subdivided in the page, by the 108-year proportions
  await go("#/dasha/ashtottari", 900);
  const third = await evaluate(`document.querySelectorAll("#dasha .dzoom .dz-row").length`);
  expect(third === 3, "Aṣṭottarī third level missing: " + third);
  return `9 tabs; Chara zooms with rāśi glyphs and a bhāva link; ${seen.join(", ")}`;
});

// ── journeys: dashas (add new journeys for this workstream above this line) ──

await journey("the command palette jumps to a bhāva", async () => {
  await go("#/overview", 800);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "k", code: "KeyK", modifiers: 4, windowsVirtualKeyCode: 75 });
  await sleep(300);
  expect(await evaluate(`!document.getElementById('palette').hidden`), "palette did not open");
  for (const ch of "Bhāva 7") await send("Input.insertText", { text: ch });
  await sleep(200);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Enter", code: "Enter", windowsVirtualKeyCode: 13 });
  await sleep(1200);
  expect(await evaluate(`location.hash`) === "#/house/7", "did not jump to bhāva 7: " + (await evaluate(`location.hash`)));
  return "⌘K → Bhāva 7";
});

await journey("auxiliary analysis: eight tabs, 37 sahams on the chart, hover links a row to its sign, every tab draws", async () => {
  await go("#/aux", 1200);
  const first = await evaluate(`(() => { const sec = document.querySelector('section.page#aux');
    return { tabs: sec.querySelectorAll('.aux-tabs .tab').length, rows: sec.querySelectorAll('tr.aux-row').length,
             cells: sec.querySelectorAll('.aux-cell').length, marks: sec.querySelectorAll('.aux-sn').length }; })()`);
  expect(first.tabs === 8, `${first.tabs} tabs`);
  expect(first.rows === 37 && first.marks === 37 && first.cells === 12, JSON.stringify(first));
  const linked = await evaluate(`(() => { const tr = document.querySelector('#aux tr.aux-row'); tr.dispatchEvent(new MouseEvent('mouseenter'));
    const hot = document.querySelector('#aux .aux-cell.hot'); const ok = hot && +hot.dataset.sign === +tr.dataset.sign;
    tr.dispatchEvent(new MouseEvent('mouseleave')); return !!ok && !document.querySelector('#aux .aux-cell.hot'); })()`);
  expect(linked, "hovering a saham row did not light its sign");
  const counts = {};
  for (const t of ["avakahada", "gender", "mrityu", "ghata", "vedha", "jiva", "shodhana"]) {
    await go("#/aux/" + t, 700);
    const r = await evaluate(`(() => { const sec = document.querySelector('section.page#aux'); const pane = sec.querySelector('.aux-pane');
      return { tab: pane && pane.dataset.tab, err: /could not be drawn|not available/.test(sec.innerText), tiles: sec.querySelectorAll('.aux-tile').length,
               mb: sec.querySelectorAll('.aux-mbrow').length, houses: sec.querySelectorAll('.aux-house').length, steps: sec.querySelectorAll('.aux-step').length,
               srows: sec.querySelectorAll('tr.aux-srow').length, chips: sec.querySelectorAll('.aux-chips .chip').length }; })()`);
    expect(r.tab === t && !r.err, `${t}: ${JSON.stringify(r)}`);
    counts[t] = r;
  }
  expect(counts.avakahada.tiles === 11, "avakahaḍā tiles " + counts.avakahada.tiles);
  expect(counts.gender.chips === 10, "male/female chips " + counts.gender.chips);
  expect(counts.mrityu.mb === 10, "mṛtyu bhāga rows " + counts.mrityu.mb);
  expect(counts.ghata.tiles === 8, "ghāta tiles " + counts.ghata.tiles);
  expect(counts.vedha.houses === 12, "vedha houses " + counts.vedha.houses);
  expect(counts.jiva.steps === 9, "jīva/deha steps " + counts.jiva.steps);
  expect(counts.shodhana.srows === 8, "śodhana rows " + counts.shodhana.srows);
  // click a śodhana row: the reduction table switches to that aṣṭakavarga
  const sw = await evaluate(`(() => { const tr = [...document.querySelectorAll('#aux tr.aux-srow')].find(r => r.dataset.g === 'lagna'); tr.click();
    return [...document.querySelectorAll('#aux h3')].map(x => x.textContent).join(' / '); })()`);
  expect(/Reduction of the Lagna/.test(sw), "śodhana row click did not switch: " + sw);
  // vedhasthāna: hovering a graha lights its favourable houses
  await go("#/aux/vedha", 700);
  const lit = await evaluate(`(() => { const tr = document.querySelector('#aux tr.aux-vrow'); tr.dispatchEvent(new MouseEvent('mouseenter'));
    return { fav: document.querySelectorAll('#aux .aux-house.fav').length, at: document.querySelectorAll('#aux .aux-house.at').length }; })()`);
  expect(lit.fav > 0 && lit.at === 1, "vedha hover " + JSON.stringify(lit));
  await shot("aux");
  return `37 sahams; ${Object.keys(counts).length + 1} tabs draw; row↔sign hover; śodhana switch; vedha ${lit.fav} favourable houses lit`;
});

// ── journeys: auxiliary (add new journeys for this workstream above this line) ──

await journey("theme toggles and is remembered", async () => {
  const before = await evaluate(`document.documentElement.dataset.mode`);
  await evaluate(`document.getElementById('theme').click()`);
  const after = await evaluate(`document.documentElement.dataset.mode`);
  expect(before !== after, "theme did not change");
  await send("Page.reload"); await sleep(2000);
  expect(await evaluate(`document.documentElement.dataset.mode`) === after, "theme not remembered");
  await evaluate(`document.getElementById('theme').click()`);
  return `${before} → ${after}`;
});

await journey("chakras: Sarvatobhadra grid with vedha on pick, Kota fort, Saṅghaṭṭa ring", async () => {
  await go("#/chakras", 1800);
  const tab = (t) => evaluate(`[...document.querySelectorAll("#chakras .tab")].find(b => b.textContent === ${JSON.stringify(t)}).click()`);
  const sbc = await evaluate(`(() => { const svg = document.querySelector("#chakras svg.ck-sbc"); const v = val("vedic.chakra.sarvatobhadra");
    return { cells: svg.querySelectorAll(".ck-cell").length, nak: svg.querySelectorAll(".ck-cell[data-kind=nakshatra]").length,
      marks: svg.querySelectorAll(".ck-pl").length, want: v.cells.reduce((n, c) => n + c.natal.length, 0),
      lines: svg.querySelectorAll(".ck-line").length, grahas: v.grahas.natal.filter(g => g.id !== "lagna").length }; })()`);
  expect(sbc.cells === 81 && sbc.nak === 28, "grid " + JSON.stringify(sbc));
  expect(sbc.marks === sbc.want, `${sbc.marks} markers for ${sbc.want} placements`);
  expect(sbc.lines === sbc.grahas * 3, `${sbc.lines} vedha lines for ${sbc.grahas} grahas`);
  await evaluate(`document.querySelector("#chakras .ck-chip[data-graha=saturn]").click()`); await sleep(400);
  const pick = await evaluate(`(() => { const svg = document.querySelector("#chakras svg.ck-sbc"); const g = val("vedic.chakra.sarvatobhadra").grahas.natal.find(x => x.id === "saturn");
    return { on: svg.querySelectorAll(".ck-line.on").length, picked: svg.classList.contains("picked"), hit: svg.querySelectorAll(".ck-cell.hit").length,
      want: new Set(["front", "left", "right"].flatMap(n => g.lines[n].map(rc => rc.join(",")))).size }; })()`);
  expect(pick.on === 3 && pick.picked && pick.hit === pick.want, "pick saturn " + JSON.stringify(pick));
  const box = await evaluate(`(() => { const r = document.querySelector("#chakras .ck-cell[data-kind=nakshatra] rect").getBoundingClientRect(); return [r.left + r.width / 2, r.top + r.height / 2]; })()`);
  await mouse(box[0], box[1]); await sleep(250);
  const tipText = await evaluate(`(document.querySelector(".tip.on, .tipdock.on") || { textContent: "" }).textContent`);
  expect(/Pierced by/.test(tipText), "no cell tooltip: " + tipText.slice(0, 80));
  await shot("chakras-sarvatobhadra");
  await tab("Kota"); await sleep(1500);
  const k = await evaluate(`(() => { const v = val("vedic.chakra.kota"); return { places: document.querySelectorAll("#chakras .ck-place").length,
    zones: document.querySelectorAll("#chakras .ck-zone").length, janma: document.querySelector("#chakras .ck-place.janma")?.dataset.position,
    marks: document.querySelectorAll("#chakras svg.ck-kota .ck-pl").length, want: v.natal.length, tr: (v.transit || []).length }; })()`);
  expect(k.places === 28 && k.zones === 4 && k.janma === "1" && k.marks === k.want, "kota " + JSON.stringify(k));
  if (k.tr) {
    await tab("Transit"); await sleep(1200);
    const tm = await evaluate(`document.querySelectorAll("#chakras svg.ck-kota .ck-pl.tr").length`);
    expect(tm === k.tr, `${tm} transit markers for ${k.tr}`);
    await tab("Natal"); await sleep(600);
  }
  await shot("chakras-kota");
  await tab("Saṅghaṭṭa"); await sleep(1500);
  const sg = await evaluate(`({ sectors: document.querySelectorAll("#chakras .ck-sector").length, chords: document.querySelectorAll("#chakras .ck-chord").length })`);
  expect(sg.sectors === 28 && sg.chords === 14, "sanghatta " + JSON.stringify(sg));
  await evaluate(`document.querySelector("#chakras .ck-chip[data-graha=mars]").click()`); await sleep(400);
  const lit = await evaluate(`document.querySelectorAll("#chakras .ck-chord.on").length`);
  expect(lit === 1, `${lit} chords lit for Mars`);
  await shot("chakras-sanghatta");
  await tab("Sarvatobhadra");
  return `81 cells, ${sbc.lines} vedha lines, Saturn pierces ${pick.want} cells; Kota 28 places${k.tr ? `, ${k.tr} transits` : ""}; Saṅghaṭṭa 28 + 14`;
});

// ── journeys: chakras (add new journeys for this workstream above this line) ──

await viewport(390, 844, true);
await journey("phone width has no horizontal scroll", async () => {
  const bad = [];
  for (const p of ["overview", "chakra", "charts", "planet/sun", "strength", "nadi"]) {
    await go("#/" + p, 1200);
    const w = await evaluate(`[document.documentElement.scrollWidth, window.innerWidth]`);
    if (w[0] > w[1] + 1) bad.push(`${p} ${w[0]}>${w[1]}`);
  }
  await shot("phone-chakra");
  expect(!bad.length, bad.join(", "));
  return "6 pages fit 390 px";
});

await journey("varṣaphala shows the return, the varṣa chart, five office-bearers with the year lord, and growing strength bars", async () => {
  await go("#/annual", 1500);
  const r = await evaluate(`(() => { const v = val('vedic.varshaphala'); if (!v) return null; const y = v.current || v.birth_year;
    return { offices: document.querySelectorAll('#page-annual .ann-office').length || document.querySelectorAll('.ann-office').length,
      cells: document.querySelectorAll('.annual-chart .cellbg').length, grahas: document.querySelectorAll('.annual-chart .ann-g').length,
      lordGlow: document.querySelectorAll('.annual-chart .ann-g.year-lord').length, bars: document.querySelectorAll('.ann-row').length,
      segs: document.querySelectorAll('.ann-seg').length, lord: y.year_lord.graha, lordRow: !!document.querySelector('.ann-row.is-lord'),
      years: document.querySelectorAll('.ann-pick button').length }; })()`);
  expect(r, "no vedic.varshaphala in the report");
  expect(r.offices === 5, `${r.offices} office-bearers`);
  expect(r.cells === 12, `${r.cells} chart cells`);
  expect(r.grahas >= 10, `${r.grahas} grahas (+ muntha) in the varṣa chart`);
  expect(r.lordGlow === 1 && r.lordRow, "year lord not highlighted");
  expect(r.bars === 7 && r.segs === 35, `${r.bars} bars, ${r.segs} segments`);
  expect(r.years === 2, "current year and birth year are not both offered");
  // switching to the birth year redraws with year 0
  await evaluate(`document.querySelector('.ann-pick button[data-year="birth_year"]').click()`);
  await sleep(600);
  const st = await evaluate(`document.querySelector('.ann-lord-stat').closest('.stats').textContent.includes('year 1 of life')`);
  expect(st, "birth-year view did not show");
  await shot("annual");
  return `year lord ${r.lord}`;
});

await journey("pañca pakṣī draws the birds and ten yāma bands with sub-periods; clicking a yāma opens its detail", async () => {
  await go("#/pakshi", 1500);
  const r = await evaluate(`({ birds: document.querySelectorAll('.pk-head svg.bird').length, yamas: document.querySelectorAll('.pk-yama').length,
    subs: document.querySelectorAll('.pk-sub').length, current: document.querySelectorAll('.pk-yama.current').length,
    now: document.querySelectorAll('.pk-now').length, tables: document.querySelectorAll('.pk-pick button').length })`);
  expect(r.birds >= 1, "no bird glyph");
  expect(r.yamas === 10 && r.subs === 50, `${r.yamas} yāmas, ${r.subs} sub-periods`);
  expect(r.current === 1 && r.now === 1, "the current period is not marked");
  await evaluate(`document.querySelectorAll('.pk-yama')[2].dispatchEvent(new MouseEvent('click', { bubbles: true }))`);
  await sleep(400);
  const d = await evaluate(`({ title: document.querySelector('.pk-detail h3').textContent, rows: document.querySelectorAll('.pk-detail tbody tr').length, picked: document.querySelectorAll('.pk-yama.picked').length })`);
  expect(/Day yāma 3/.test(d.title) && d.rows === 10 && d.picked === 1, `detail did not open: ${JSON.stringify(d)}`);
  await shot("pakshi");
  return `${r.tables} table(s)`;
});

await journey("the KP page adds the significator tables by bhāva and by graha", async () => {
  await go("#/kp", 1500);
  const r = await evaluate(`({ cards: [...document.querySelectorAll('.card h3')].map(x => x.textContent), lvls: document.querySelectorAll('.kp-lvl').length, chips: document.querySelectorAll('.kp-chip').length })`);
  expect(r.cards.includes("KP significators by bhāva") && r.cards.includes("KP grahas: what each signifies"), "KP significator cards missing");
  expect(r.lvls === 36, `${r.lvls} strength cells (9 grahas × 4)`);
  expect(r.chips >= 12, `${r.chips} significator chips`);
});

// ── journeys: annual (add new journeys for this workstream above this line) ──

ws.close(); chrome.kill(); server.close();
const failed = results.filter((r) => !r.ok);
console.log(JSON.stringify({ journeys: results.length, failed: failed.length, results }));
console.log(failed.length ? `${failed.length} journey(s) failed` : `all ${results.length} journeys passed`);
process.exit(failed.length ? 1 : 0);
