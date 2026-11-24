// Chart gallery for the website (CHK-22): opens real reports in headless Chrome, captures each studio page as a WebP
// screenshot into examples/gallery/, and writes website/src/data/gallery.json (title, caption, image, live link).
// Usage: node scripts/gallery.mjs  (after `make reports`; needs Node 22+ and Chrome; CHROME=… to override)
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repo = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(repo, "examples", "gallery");
mkdirSync(outDir, { recursive: true });
const CHROME = process.env.CHROME || ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"].find(existsSync);
if (!CHROME) { console.error("Chrome not found; set CHROME=/path/to/chrome"); process.exit(2); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// [report file, hash, image name, title, caption, setup script run before the capture]
const R = "reference-ramana.html", E = "example-birth-chart.html", A = "example-studio-app.html";
const SHOTS = [
  [R, "#/chakra", "jyotish-chakra", "Jyotish Chakra", "The Graha-Nakshatra Mandala: rāśis, nakshatras, padas and every graha, with the D1 chart in the centre. Hover details sit beside the wheel."],
  [R, "#/charts", "charts", "South and North Indian charts", "Ārūḍha padas at the top of each rāśi, aspect marks at the bottom; crowded North Indian houses fit their shape.", "S.layers.extra_aspects = true; S.chart = 'both';"],
  [R, "#/vargas", "vargas", "Divisional charts", "D1 to D60 side by side."],
  [R, "#/aspects", "aspects", "Aspects", "Graha drishti, rāśi drishti and sputa drishti in virūpas, on grahas, houses and cusps."],
  [R, "#/houses", "bhavas", "Bhāvas", "Every house, where its lord sits, and lordships in D1, D9 and D10 with the exchanges between lords."],
  [R, "#/maitri", "maitri", "Maitri and dignities", "Compound relationships of the nine grahas, the nakshatra tārā matrix and dignities in 16 divisions."],
  [R, "#/strength", "strength", "Strength", "Ṣaḍbala with the cut-off at the required minimum, bhāva bala and viṃśopaka."],
  [R, "#/ashtakavarga", "ashtakavarga", "Aṣṭakavarga", "Bhinna and sarva aṣṭakavarga."],
  [R, "#/dasha/chara", "dasha", "Daśā", "Nine daśā systems; the zoom opens five levels deep. Shown: Chara daśā."],
  [R, "#/yoga/parivartana", "yoga-parivartana", "Yogas", "Each yoga present, with how it forms, strength checks, timing and its reading. Shown: the Jupiter–Saturn parivartana."],
  [R, "#/aux/sahams", "sahams", "Sahams", "37 sahams on the chart, each with an info box: formula as computed, placement and meaning."],
  [R, "#/chakras/sbc", "sarvatobhadra", "Sarvatobhadra", "The 9×9 chakra framed by nakshatra, pada-sound and pada bands, with vedha lines and borders."],
  [R, "#/chakras/kota", "kota", "Kota chakra", "The fort: Bāhya, Prākāra, Madhya and Stambha, with the paths in and out."],
  [R, "#/annual", "varshaphala", "Varṣaphala", "The annual chart, muntha, office-bearers and the five-fold strengths."],
  [R, "#/pakshi", "pancha-pakshi", "Pañca pakṣī", "The five birds through the day and the night, the current yāma, and the sub-periods."],
  [R, "#/kp", "kp", "KP", "Sub-lords, cusps and significator tables by bhāva and by graha."],
  [R, "#/panchanga", "panchanga", "Pañcāṅga", "Tithi, vāra, nakshatra, yoga and karaṇa at birth."],
  [E, "#/human-design", "human-design", "Human Design", "The standard BodyGraph with gates, channels and properties."],
  [E, "#/numerology", "numerology", "Numerology", "Every school's numbers, the Lo Shu grid and the Kua number."],
  [A, "#/calendar", "calendar", "Personal calendar", "Muhūrta windows for 31 days with tārā and candra bala from the birth chart; export as CSV or ICS."],
  [A, "#/horary", "kp-horary", "KP horary", "Number 100 fixes the Ascendant; Placidus cusps, sub-lords and significators at the moment of judgement."],
  [A, "#/compare", "compare", "Compare workspace", "Natal, progressed and transit positions side by side, recomputed offline by the embedded engine."],
  [A, "#/playground", "playground", "Playground", "Place grahas by keyboard and see what follows; factors that need a real moment are listed as unavailable."],
];

const reports = join(repo, "examples", "reports");
const server = createServer((req, res) => {
  const f = join(reports, basename((req.url || "/").split("?")[0]));
  if (!existsSync(f)) return void res.writeHead(404).end();
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" }).end(readFileSync(f));
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}/`;
const port = 9100 + Math.floor(Math.random() * 300);
const chrome = spawn(CHROME, ["--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "gallery-"))}`, "--no-first-run",
  "--no-default-browser-check", "--hide-scrollbars", "about:blank"], { stdio: "ignore" });
let target;
for (let i = 0; i < 100 && !target; i++) { await sleep(150); try { target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === "page"); } catch {} }
if (!target) { console.error("Chrome did not start"); process.exit(1); }
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0; const pending = new Map();
ws.addEventListener("message", (m) => { const msg = JSON.parse(m.data); if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); } });
const send = (method, params = {}) => new Promise((r, reject) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params }));
  setTimeout(() => { if (pending.has(i)) { pending.delete(i); reject(new Error(`${method} timed out`)); } }, 60000); });
const evaluate = async (expression) => (await send("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true })).result?.result?.value;

await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 1440, height: 1000, deviceScaleFactor: 1, mobile: false });
const items = []; let loaded = null;
for (const [file, hash, name, title, caption, setup] of SHOTS) {
  if (loaded !== file) { await send("Page.navigate", { url: base + file + "#/overview" }); await sleep(2500); await evaluate("localStorage.clear(); 1"); loaded = file; }
  if (setup) await evaluate(setup + " 1");
  await evaluate(`location.hash = '#/overview'`); await sleep(300);
  await evaluate(`location.hash = ${JSON.stringify(hash)}`); await sleep(2600);
  const shot = await send("Page.captureScreenshot", { format: "webp", quality: 78, captureBeyondViewport: false });
  writeFileSync(join(outDir, `${name}.webp`), Buffer.from(shot.result.data, "base64"));
  items.push({ name, title, caption, image: `gallery/${name}.webp`, link: `reports/${file}${hash}`, chart: file === R ? "Sri Ramana Maharishi, 30 Dec 1879" : "Example Person, 15 May 1990" });
  console.log("captured", name);
}
writeFileSync(join(repo, "website", "src", "data", "gallery.json"), JSON.stringify({ generated_from: "real reports in examples/reports", items }, null, 2) + "\n");
ws.close(); chrome.kill(); server.close();
console.log(`wrote ${items.length} images to examples/gallery and website/src/data/gallery.json`);
