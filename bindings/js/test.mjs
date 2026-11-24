// The JavaScript package against the engine: a chart, an error with its code, and the same envelope the native CLI
// gives (when ESHANA_BIN is set). Run with `node test.mjs` after `make js`.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { load, EshanaError } from "./index.mjs";

const eshana = await load();
assert.match(String(eshana.version), /^\d+\.\d+\.\d+/);
const req = { op: "chart", system: "vedic", when: { date: "1990-05-15", time: "14:30", timezone: "Asia/Kolkata" },
  place: { latitude: 28.6139, longitude: 77.209 }, config: { zodiac: "sidereal", ayanamsha: "lahiri" } };
const chart = eshana.execute(req);
assert.equal(chart.kind, "computed_chart");
assert.ok(chart.positions.find((p) => p.id === "moon").longitude_deg > 0);
assert.throws(() => eshana.execute({ op: "chart", system: "vedic", when: { date: "1990-05-15", time: "14:30" } }),
  (e) => e instanceof EshanaError && e.code.startsWith("ESH-"));
const raw = JSON.parse(eshana.executeJson("{ not json"));
assert.equal(raw.kind, "error");
if (process.env.ESHANA_BIN) {
  const cli = JSON.parse(execFileSync(process.env.ESHANA_BIN, ["--compact", "request", "-"], { input: JSON.stringify(req) }).toString());
  const moon = (e) => e.positions.find((p) => p.id === "moon").longitude_deg;
  assert.ok(Math.abs(moon(cli) - moon(chart)) < 1e-9, "same Moon as the native CLI");
}
console.log(`eshana ${eshana.version} (WebAssembly) ok: chart, error codes, raw JSON${process.env.ESHANA_BIN ? ", parity with the CLI" : ""}`);
