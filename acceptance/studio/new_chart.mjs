// Browser journey for guided chart creation (`eshana-report new`): real Chrome, real keystrokes, the real command.
//
//   node acceptance/studio/new_chart.mjs <path/to/eshana-report> [--shots <dir>]
//
// Starts `eshana-report new --no-browser` in a fresh directory and reads the form URL it prints. In Chrome: the form
// has a label for every field and fills in the browser's time zone; a submit without a date stays on the form with a
// focused alert and keeps what was typed; a clock time inside a daylight-saving gap comes back with the engine's error
// code; a valid entry lands on the chart studio; then the command exits 0, prints the path of the new file, and the
// server no longer answers. Exits 1 on any failure. Needs Node 22+ and Chrome (CHROME=… to override).
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const args = process.argv.slice(2);
const bin = args[0] && resolve(args[0]);
const shots = args.includes("--shots") ? resolve(args[args.indexOf("--shots") + 1]) : null;
if (!bin || !existsSync(bin)) { console.error("usage: new_chart.mjs <eshana-report> [--shots <dir>]"); process.exit(2); }
if (shots) mkdirSync(shots, { recursive: true });
const CHROME = process.env.CHROME || ["/Applications/Google Chrome.app/Contents/MacOS/Google Chrome", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"].find(existsSync);
if (!CHROME) { console.error("Chrome not found; set CHROME=/path/to/chrome"); process.exit(2); }
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const out = mkdtempSync(join(tmpdir(), "eshana-new-"));
const cmd = spawn(bin, ["new", "--no-browser", "-o", out, "--transit-days", "0", "--progression-years", "0", "--timeout", "240"],
  { stdio: ["ignore", "pipe", "pipe"], env: { ...process.env, TZ: "Asia/Kolkata" } });
let stdout = "", stderr = "";
cmd.stdout.on("data", (d) => { stdout += d; });
cmd.stderr.on("data", (d) => { stderr += d; });
const exited = new Promise((r) => cmd.on("exit", (code) => r(code)));
let formUrl;
for (let i = 0; i < 200 && !formUrl; i++) { await sleep(100); formUrl = (stderr.match(/http:\/\/127\.0\.0\.1:\d+\/[^\s]+\//) || [])[0]; }
if (!formUrl) { console.log("✗ the command did not print a form URL:\n" + stderr); process.exit(1); }

const port = 9100 + Math.floor(Math.random() * 300);
const chrome = spawn(CHROME, ["--headless=new", `--remote-debugging-port=${port}`, `--user-data-dir=${mkdtempSync(join(tmpdir(), "new-chart-"))}`,
  "--no-first-run", "--no-default-browser-check", "--use-mock-keychain", "--password-store=basic", "--hide-scrollbars", "about:blank"],
  { stdio: "ignore", env: { ...process.env, TZ: "Asia/Kolkata" } });
setTimeout(() => { console.log("✗ timed out"); chrome.kill(); cmd.kill(); process.exit(1); }, 240000).unref();
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
});
const send = (method, params = {}) => new Promise((r) => { const i = ++id; pending.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const evaluate = async (expr) => (await send("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true })).result?.result?.value;
const shot = async (name) => { if (!shots) return; const r = await send("Page.captureScreenshot", { format: "png" }); writeFileSync(join(shots, name + ".png"), Buffer.from(r.result.data, "base64")); };
const navigated = async (fn, ms = 60000) => { const before = await evaluate("performance.timeOrigin"); await fn();
  for (let t = 0; t < ms; t += 200) { await sleep(200); const now = await evaluate("document.readyState === 'complete' ? performance.timeOrigin : null"); if (now && now !== before) return; }
  throw new Error("the page did not change"); };
// type into a field the way a person does: focus it, select what is there, type the text
const type = async (fid, text) => {
  await evaluate(`(() => { const e = document.getElementById(${JSON.stringify(fid)}); e.focus(); e.select && e.select(); return 1; })()`);
  await send("Input.dispatchKeyEvent", { type: "keyDown", key: "Backspace", code: "Backspace", windowsVirtualKeyCode: 8 });
  await send("Input.dispatchKeyEvent", { type: "keyUp", key: "Backspace", code: "Backspace", windowsVirtualKeyCode: 8 });
  if (text) await send("Input.insertText", { text });
};
// date and time inputs take their value from the picker; set it as the picker would, with an input event
const pick = (fid, value) => evaluate(`(() => { const e = document.getElementById(${JSON.stringify(fid)}); e.value = ${JSON.stringify(value)}; e.dispatchEvent(new Event('input', {bubbles: true})); return e.value; })()`);
const submit = () => navigated(() => evaluate("document.getElementById('go').click(), 1"));

const results = [];
async function journey(name, fn) {
  problems = [];
  try { const d = (await fn()) || ""; if (problems.length) throw new Error(problems.slice(0, 3).join(" | ")); results.push({ name, ok: true }); console.log(`✓ ${name}${d ? ` (${d})` : ""}`); }
  catch (e) { results.push({ name, ok: false, detail: String(e.message || e) }); console.log(`✗ ${name}: ${e.message || e}`); }
}
const expect = (c, m) => { if (!c) throw new Error(m); };

await send("Runtime.enable"); await send("Page.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 900, height: 1300, deviceScaleFactor: 1, mobile: false });
await send("Emulation.setTimezoneOverride", { timezoneId: "Asia/Kolkata" });
await navigated(() => send("Page.navigate", { url: formUrl }));

await journey("the form labels every field and fills the browser's zone", async () => {
  const r = await evaluate(`(() => { const unl = [...document.querySelectorAll('input:not([type=checkbox]),select')].filter(e => !document.querySelector('label[for="'+e.id+'"]')).map(e => e.id);
    return { unl, tz: document.getElementById('timezone').value, title: document.title, zones: document.querySelectorAll('#zones option').length }; })()`);
  expect(r.unl.length === 0, "fields without a label: " + r.unl.join(", "));
  expect(/^Asia\/(Kolkata|Calcutta)$/.test(r.tz), "time zone not pre-filled: " + r.tz);
  expect(r.zones > 300, "zone suggestions missing: " + r.zones);
  await shot("01-form");
  return `${r.zones} zones offered`;
});

await journey("a missing date stays on the form, keeps what was typed and focuses the alert", async () => {
  await type("name", "Example Person");
  await type("place_label", "New Delhi, India");
  await type("latitude", "28.6139");
  await type("longitude", "77.2090");
  await pick("time", "14:30");
  await submit();
  const r = await evaluate(`({ alert: document.querySelector('.alert')?.innerText || '', focused: document.activeElement?.className,
    name: document.getElementById('name').value, invalid: document.getElementById('date').getAttribute('aria-invalid') })`);
  expect(/birth date is required/.test(r.alert), "no alert: " + r.alert);
  expect(r.focused === "alert", "alert not focused: " + r.focused);
  expect(r.name === "Example Person", "the typed name was lost");
  expect(r.invalid === "true", "date not marked invalid");
  await shot("02-missing-date");
});

await journey("a time in a daylight-saving gap returns the engine's code", async () => {
  await pick("date", "2021-03-14");
  await pick("time", "02:30");
  await type("timezone", "America/New_York");
  await submit();
  const alert = await evaluate(`document.querySelector('.alert')?.innerText || ''`);
  expect(/ESH-[A-Z-]+/.test(alert), "no engine code: " + alert);
  expect(!existsSync(join(out, "example-person-2021-03-14.html")), "a file was written for an invalid entry");
  await shot("03-engine-refusal");
  return alert.match(/ESH-[A-Z-]+/)[0];
});

await journey("a valid entry shows the chart studio", async () => {
  await pick("date", "1990-05-15");
  await pick("time", "14:30");
  await type("timezone", "Asia/Kolkata");
  await submit();
  await sleep(1500);
  const r = await evaluate(`({ title: document.title, data: !!document.getElementById('eshana-data'), url: location.pathname })`);
  expect(r.data, "not the studio: " + r.title);
  expect(/Example Person/.test(r.title), "title: " + r.title);
  expect(r.url.endsWith("/chart"), "url: " + r.url);
  await shot("04-chart");
  return r.title;
});

await journey("the command stops, prints the new file, and the server is gone", async () => {
  const code = await Promise.race([exited, sleep(30000).then(() => "still running")]);
  expect(code === 0, `exit ${code}: ${stderr}`);
  const path = stdout.trim();
  expect(path === join(out, "example-person-1990-05-15.html"), "printed: " + path);
  expect(readFileSync(path, "utf8").includes("Example Person"), "file does not hold the chart");
  let refused = false;
  try { await fetch(formUrl); } catch { refused = true; }
  expect(refused, "the server still answers");
  return path.split("/").pop();
});

ws.close(); chrome.kill(); cmd.kill();
const failed = results.filter((r) => !r.ok);
console.log(JSON.stringify({ journeys: results.length, failed: failed.length }));
process.exit(failed.length ? 1 : 0);
