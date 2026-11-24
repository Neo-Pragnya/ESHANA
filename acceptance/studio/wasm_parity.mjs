// The browser engine gives the same envelope as the native CLI for the same request (ADR-013): identical structure,
// strings and integers, and floating-point numbers within 1e-9 relative (the two builds use different libm
// implementations, so the last bit of a transcendental function can differ). Error envelopes compare by code.
//
//   node acceptance/studio/wasm_parity.mjs <eshana_wasm.wasm> <eshana binary>
//
// Loads the WebAssembly module with no imports, runs a set of requests through its three-function ABI, and compares
// each envelope with `eshana --compact request -`. Exits 1 on any difference.
import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const [wasmPath, bin] = process.argv.slice(2);
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const x = instance.exports, enc = new TextEncoder(), dec = new TextDecoder();
export function execute(req) {
  const b = enc.encode(JSON.stringify(req));
  const p = x.eshana_alloc(b.length);
  new Uint8Array(x.memory.buffer, p, b.length).set(b);
  const out = x.eshana_execute(p, b.length);
  const n = new DataView(x.memory.buffer).getUint32(out, true);
  const s = dec.decode(new Uint8Array(x.memory.buffer, out + 4, n).slice());
  x.eshana_free(out, 4 + n); x.eshana_free(p, b.length);
  return s;
}
const natal = { when: { date: "1990-05-15", time: "14:30", timezone: "Asia/Kolkata" }, place: { latitude: 28.61, longitude: 77.21 },
  config: { zodiac: "sidereal", ayanamsha: "lahiri" } };
const requests = [
  { op: "methods" },
  { op: "chart", system: "vedic", ...natal, features: ["all"], dasha: true, reference_utc: "2026-11-23T00:00:00Z" },
  { op: "chart", system: "western", when: natal.when, place: natal.place, config: { zodiac: "tropical", house_system: "placidus" }, aspects: true },
  { op: "synthetic_chart", positions: [{ body: "sun", longitude_deg: 30 }, { body: "moon", longitude_deg: 100 }], ascendant_deg: 10 },
  { op: "progressions", natal: { ...natal, config: { zodiac: "tropical" } }, at_utc: "2026-11-23T00:00:00Z", angle_method: "naibod_ra" },
  { op: "kp_horary", number: 100, chart: { when: { utc: "2026-11-23T10:00:00Z" }, place: { latitude: 13.08, longitude: 80.27 }, config: { zodiac: "sidereal", ayanamsha: "krishnamurti" } } },
  { op: "positions", when: { utc: "2000-01-01T12:00:00Z" }, config: { provider: "spk" }, spk_path: "/nonexistent.bsp" },
];
let bad = 0;
for (const r of requests) {
  const w = execute(r);
  let n;
  try { n = execFileSync(bin, ["--compact", "request", "-"], { input: JSON.stringify(r), stdio: ["pipe", "pipe", "ignore"] }).toString(); }
  catch (e) { n = e.stdout.toString(); } // an error envelope exits 2 and still prints the envelope
  n = n.trimEnd();
  const close = (a, b, path) => {
    if (typeof a === "number" && typeof b === "number") return Number.isInteger(a) && Number.isInteger(b) ? a === b : Math.abs(a - b) <= 1e-9 * Math.max(1, Math.abs(a), Math.abs(b)) || path;
    if (a === null || b === null || typeof a !== "object" || typeof b !== "object") return a === b || path;
    if (Array.isArray(a) !== Array.isArray(b)) return path;
    const ka = Object.keys(a), kb = Object.keys(b);
    if (ka.length !== kb.length || ka.some((k, i) => k !== kb[i])) return path + " (keys)";
    for (const k of ka) { const r = close(a[k], b[k], path + "." + k); if (r !== true) return r; }
    return true;
  };
  const W = JSON.parse(w), N = JSON.parse(n);
  const same = W.kind === "error" && N.kind === "error" ? W.error.code === N.error.code || "error code" : close(W, N, "$");
  const ok = same === true;
  if (!ok) bad++;
  let at = 0; while (at < w.length && w[at] === n[at]) at++;
  console.log(`${ok ? "✓" : "✗"} ${r.op}${r.system ? " " + r.system : ""} (${w.length} bytes${w === n ? ", identical bytes" : ""})${ok ? "" : `\n  ${same}; first byte difference at ${at}:\n  wasm:   …${w.slice(Math.max(0, at - 120), at + 80)}\n  native: …${n.slice(Math.max(0, at - 120), at + 80)}`}`);
}
process.exit(bad ? 1 : 0);
