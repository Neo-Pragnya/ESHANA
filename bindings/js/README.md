# eshana (JavaScript)

The ESHANA engine for JavaScript: the same Rust engine as the CLI and the Python package, compiled to WebAssembly,
for browsers and Node 18+. One entry point: a JSON request in, a JSON envelope out. Every rule is named, unknown
inputs are refused with a stable `ESH-*` code, and the `interpretations` field is always empty.

```js
import { load, EshanaError } from "eshana";

const eshana = await load(); // Node: the .wasm next to the module; browsers: fetched next to it
const chart = eshana.execute({
  op: "chart", system: "vedic",
  when: { date: "1990-05-15", time: "14:30", timezone: "Asia/Kolkata" },
  place: { latitude: 28.6139, longitude: 77.209 },
  config: { zodiac: "sidereal", ayanamsha: "lahiri" },
});
chart.positions.find((p) => p.id === "moon").sign.name; // "Makara"

try { eshana.execute({ op: "chart", system: "vedic", when: { date: "1990-05-15", time: "14:30" } }); }
catch (e) { if (e instanceof EshanaError) console.log(e.code); } // ESH-INPUT: no zone is ever assumed
```

- `load(source?)` takes a URL, a `Response`, bytes or a compiled `WebAssembly.Module` to load the engine from
  elsewhere (a CDN, a bundler asset).
- `execute(request)` returns the envelope or throws `EshanaError`; `executeJson(text)` returns JSON text and never
  throws.
- Every operation of [the request reference](https://neo-pragnya.github.io/ESHANA/docs/reference/request/) works,
  with the built-in `analytic` provider. The `spk` provider needs a kernel file, which a WebAssembly module cannot
  open: it answers `ESH-PROVIDER`.
- Nothing is sent anywhere: the module has no imports and needs no network.

Build from the repository: `make js` (builds the module, copies it here and runs `test.mjs`, which also checks the
result against the native CLI). Licence: Apache-2.0.
