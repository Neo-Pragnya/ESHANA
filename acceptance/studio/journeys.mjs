// Browser journeys for ESHANA Studio (the chart report app): real Chrome, real time, real clicks.
//
//   node acceptance/studio/journeys.mjs <report.html> [--shots <dir>]
//
// Loads a generated studio file and walks what a reader does: every page renders without console errors; the
// Panchang chakra draws 27 nakshatras, 108 padas and a point for every graha at its longitude; the South Indian centre
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
const WATCHDOG_MS = Number(process.env.JOURNEYS_TIMEOUT_MS || 240000);
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
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
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
async function journey(name, fn) {
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
  return { pos, asc: v.houses && v.houses.ascendant ? v.houses.ascendant.longitude_deg : null, pages: d.pages }; })()`);
const signOf = (l) => Math.floor(((l % 360) + 360) % 360 / 30);
const SIGN_AB = ["Ar", "Ta", "Ge", "Cn", "Le", "Vi", "Li", "Sc", "Sg", "Cp", "Aq", "Pi"];

await journey("every page renders without errors", async () => {
  const pages = model.pages.filter((p) => !["planet", "house"].includes(p));
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
  const tipText = await evaluate(`document.querySelector('.tip.on')?.innerText || ''`);
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
  expect(r.pl >= Object.keys(model.pos).length, `${r.pl} grahas in the matrix`);
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

ws.close(); chrome.kill(); server.close();
const failed = results.filter((r) => !r.ok);
console.log(JSON.stringify({ journeys: results.length, failed: failed.length, results }));
console.log(failed.length ? `${failed.length} journey(s) failed` : `all ${results.length} journeys passed`);
process.exit(failed.length ? 1 : 0);
