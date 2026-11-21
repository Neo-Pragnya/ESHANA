// Browser journeys for the ESHANA website: real Chrome, real time, real pointer events.
//
//   node acceptance/site/journeys.mjs <website/dist> [--base /ESHANA/] [--shots <dir>]
//
// Serves the built site and checks what a visitor sees: the home hero runs the universe (stars, the 27-nakshatra
// ecliptic plexus, grahas riding it) and the plexus reacts to the cursor; every page carries the ambient sky; the
// header and doc-section icons are ESHANA sigils (not morph glyphs) and wake on hover; docs pages render; the
// theme toggles; reduced motion gives one still frame; phone width has no horizontal scroll; no console errors.
// Prints one line per journey and a JSON summary; exits 1 on any failure. Needs Node 22+ and Chrome.
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { extname, join, resolve } from "node:path";

const args = process.argv.slice(2);
const dist = args[0] && resolve(args[0]);
const shots = args.includes("--shots") ? resolve(args[args.indexOf("--shots") + 1]) : null;
if (!dist || !existsSync(join(dist, "index.html"))) {
  console.error("usage: journeys.mjs <website/dist> [--shots <dir>]");
  process.exit(2);
}
if (shots) mkdirSync(shots, { recursive: true });
// the site's base path (SITE_BASE at build time); pages and assets live under it
let base = args.includes("--base") ? args[args.indexOf("--base") + 1] : "/";
if (!base.startsWith("/")) base = "/" + base;
if (!base.endsWith("/")) base += "/";
const CHROME = process.env.CHROME || ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"].find(existsSync);
if (!CHROME) { console.error("Chrome not found; set CHROME=/path/to/chrome"); process.exit(2); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".css": "text/css", ".svg": "image/svg+xml", ".png": "image/png", ".json": "application/json", ".webp": "image/webp", ".woff2": "font/woff2" };
const server = createServer((req, res) => {
  let p = decodeURIComponent((req.url || "/").split("?")[0].split("#")[0]);
  if (!p.startsWith(base)) return void res.writeHead(404).end();
  p = "/" + p.slice(base.length);
  let f = join(dist, p);
  if (!f.startsWith(dist)) return void res.writeHead(403).end();
  if (existsSync(f) && statSync(f).isDirectory()) f = join(f, "index.html");
  if (!existsSync(f)) return void res.writeHead(404, { "content-type": TYPES[".html"] }).end(readFileSync(join(dist, "404.html")));
  res.writeHead(200, { "content-type": TYPES[extname(f)] || "application/octet-stream" }).end(readFileSync(f));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;

const port = 9500 + Math.floor(Math.random() * 400);
const chrome = spawn(CHROME, ["--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "site-"))}`,
  "--no-first-run", "--no-default-browser-check", "--disable-gpu", "--use-mock-keychain", "--password-store=basic", "--hide-scrollbars", "about:blank"], { stdio: "ignore" });
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

const open = async (path, settle = 1800) => { await send("Page.navigate", { url: origin + base + path.slice(1) }); await sleep(settle); };
await viewport(1440, 900);

await journey("home hero runs the universe and the plexus answers the cursor", async () => {
  await open("/", 2500);
  const s = await evaluate(`(() => { const c = document.getElementById('universe'); return c && { state: c.dataset.universe, nak: +c.dataset.nakshatras, frames: +(c.dataset.frames || 0), w: c.width, h: c.height }; })()`);
  expect(s, "no #universe canvas");
  expect(s.state === "running", "universe state " + s.state);
  expect(s.nak === 27, "nakshatra nodes " + s.nak);
  expect(s.frames >= 30 && s.w > 0 && s.h > 0, "universe is not animating: " + JSON.stringify(s));
  // sample pixels before and after moving the pointer into the band: the frame must change
  const sample = `(() => { const c = document.getElementById('universe'); const x = c.getContext('2d'); const sw = Math.min(c.width, 480), sh = Math.min(c.height, 320), d = x.getImageData(Math.max(0, (c.width * .72 | 0) - sw / 2), Math.max(0, (c.height / 2 | 0) - sh / 2), sw, sh).data; let lit = 0, sum = 0; for (let i = 0; i < d.length; i += 4 * 7) { if (d[i + 3] > 30) lit++; sum += d[i] + d[i + 1] + d[i + 2]; } return { lit, sum }; })()`;
  const a = await evaluate(sample);
  expect(a.lit > 200, "too few lit pixels: " + a.lit);
  const box = await evaluate(`(() => { const r = document.getElementById('universe').getBoundingClientRect(); return { x: r.left + r.width * 0.72, y: r.top + r.height * 0.5, w: r.width }; })()`);
  for (let i = 0; i < 12; i++) { await mouse(box.x - 160 + i * 25, box.y + Math.sin(i) * 30); await sleep(40); }
  await sleep(300);
  await shot("home-hero");
  const b = await evaluate(sample);
  expect(a.sum !== b.sum, "canvas did not change");
  const front = await evaluate(sample.replace("'universe'", "'universe-front'"));
  expect(front.lit > 20, "the near half of the ring is not drawn in front of the wheel: " + front.lit);
  const pe = await evaluate(`getComputedStyle(document.getElementById('universe-front')).pointerEvents`);
  expect(pe === "none", "front ring would block clicks");
  return `27 nakshatras, ${s.frames}+ frames, ${a.lit} lit samples, ${front.lit} in front`;
});

await journey("the headline is drawn as particles and the logo is a live three-ring gimbal", async () => {
  await open("/", 2500);
  const r = await evaluate(`(() => ({ on: document.querySelectorAll('.hero h1 .mk-particles-on').length, lines: document.querySelectorAll('.hero h1 .pline').length,
    rings: [...document.querySelectorAll('.brand .sg-eshana .gb')].map(g => getComputedStyle(g).animationName),
    letters: document.querySelectorAll('h2[data-letters]').length }))()`);
  expect(r.lines === 4 && r.on === 4, `particle lines ${r.on}/${r.lines}`);
  expect(r.rings.length === 3 && r.rings.every((n) => n && n !== "none"), "gimbal rings: " + r.rings.join(","));
  const a = await evaluate(`getComputedStyle(document.querySelector('.brand .sg-eshana .gb1')).transform`);
  await sleep(700);
  const b = await evaluate(`getComputedStyle(document.querySelector('.brand .sg-eshana .gb1')).transform`);
  expect(a !== b, "gimbal is not moving");
  return `${r.on} particle lines, rings ${r.rings.join(" ")}, ${r.letters} letter headings`;
});

await journey("every page carries the ambient sky", async () => {
  const bad = [];
  for (const p of ["/docs/", "/demos/", "/docs/guides/getting-started/", "/no-such-page/"]) {
    await open(p, 1400);
    const st = await evaluate(`(() => { const c = document.getElementById('sky-ambient'); return c ? c.dataset.universe + ':' + getComputedStyle(c).position : 'missing'; })()`);
    if (!/^(running|still):fixed$/.test(st)) bad.push(`${p} ${st}`);
  }
  expect(!bad.length, bad.join(", "));
  return "4 pages";
});

await journey("icons are ESHANA sigils and wake on hover", async () => {
  await open("/", 1500);
  const r = await evaluate(`(() => ({ morph: document.querySelectorAll('[data-morph]').length, sigils: document.querySelectorAll('svg.sigil').length,
    brand: !!document.querySelector('.brand svg.sigil.sg-eshana'), kinds: [...new Set([...document.querySelectorAll('svg.sigil')].map(s => [...s.classList].find(c => c.startsWith('sg-') && c !== 'sg-live')))] }))()`);
  expect(r.morph === 0, r.morph + " morph glyphs remain");
  expect(r.brand, "header mark is not the ESHANA sigil");
  expect(r.kinds.length >= 5, "too few sigil kinds: " + r.kinds.join(","));
  const pt = await evaluate(`(() => { const s = document.querySelector('.grid-cards svg.sg-yantra'); s.scrollIntoView({ block: 'center' }); const b = s.getBoundingClientRect(); return { x: b.left + b.width / 2, y: b.top + b.height / 2 }; })()`);
  await sleep(400);
  const idle = await evaluate(`getComputedStyle(document.querySelector('.grid-cards svg.sg-yantra .sg-spin')).animationName`);
  await mouse(pt.x, pt.y); await sleep(500);
  const hot = await evaluate(`getComputedStyle(document.querySelector('.grid-cards svg.sg-yantra .sg-spin')).animationName`);
  expect(idle === "none" && hot === "sg-turn", `hover did not wake the sigil (${idle} → ${hot})`);
  await shot("sigils");
  return `${r.sigils} sigils, kinds ${r.kinds.join(" ")}`;
});

await journey("docs render with sidebar sigils and no errors", async () => {
  await open("/docs/guides/getting-started/", 1500);
  const r = await evaluate(`({ h1: document.querySelector('h1')?.textContent?.trim(), side: document.querySelectorAll('aside svg.sigil, nav svg.sigil').length })`);
  expect(r.h1, "no heading");
  expect(r.side >= 4, "sidebar sigils " + r.side);
  return r.h1;
});

await journey("theme toggles and is remembered", async () => {
  await open("/", 1200);
  const before = await evaluate(`document.documentElement.dataset.theme || 'light'`);
  await evaluate(`document.getElementById('theme-toggle').click()`);
  await send("Page.reload"); await sleep(1500);
  const after = await evaluate(`document.documentElement.dataset.theme || 'light'`);
  expect(before !== after, "theme not remembered");
  await shot("home-" + after);
  await evaluate(`document.getElementById('theme-toggle').click()`);
  return `${before} → ${after}`;
});

await journey("reduced motion draws one still frame", async () => {
  await open("/?motion=static", 1500);
  const st = await evaluate(`document.getElementById('universe').dataset.universe`);
  expect(st === "still", "state " + st);
  return "still";
});

await viewport(390, 844, true);
await journey("phone width has no horizontal scroll", async () => {
  const bad = [];
  for (const p of ["/", "/docs/", "/demos/", "/docs/guides/getting-started/"]) {
    await open(p, 1200);
    const w = await evaluate(`[document.documentElement.scrollWidth, window.innerWidth]`);
    if (w[0] > w[1] + 1) bad.push(`${p} ${w[0]}>${w[1]}`);
  }
  await open("/", 1200); await shot("phone-home");
  expect(!bad.length, bad.join(", "));
  return "4 pages fit 390 px";
});

ws.close(); chrome.kill(); server.close();
const failed = results.filter((r) => !r.ok);
console.log(JSON.stringify({ journeys: results.length, failed: failed.length, results }));
console.log(failed.length ? `${failed.length} journey(s) failed` : `all ${results.length} journeys passed`);
process.exit(failed.length ? 1 : 0);
