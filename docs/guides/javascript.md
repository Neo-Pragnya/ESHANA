---
title: "JavaScript and WebAssembly"
description: "The engine in a browser or Node: the eshana package, built from the same Rust code as the CLI and the Python package."
---

# JavaScript and WebAssembly

`bindings/js` packages the engine compiled to WebAssembly for browsers and Node 18 or later. It is the same
`Engine::execute_json` as every other front end, so the numbers match the CLI exactly (the package's test checks
that). See [ADR-014](../adr/014-javascript-binding.md) for the design.

```js
import { load, EshanaError } from "eshana";

const eshana = await load();
const env = eshana.execute({
  op: "chart", system: "vedic",
  when: { date: "1990-05-15", time: "14:30", timezone: "Asia/Kolkata" },
  place: { latitude: 28.6139, longitude: 77.209 },
  config: { zodiac: "sidereal", ayanamsha: "lahiri" },
});
```

- `load(source?)`: in Node the `.wasm` beside the module; in a browser it is fetched beside it. Pass a URL, a
  `Response`, bytes or a compiled `WebAssembly.Module` to load it from elsewhere.
- `execute(request)` returns the envelope or throws `EshanaError` (with `.code`); `executeJson(text)` returns JSON
  text and never throws.
- Every [operation](../reference/request.md) works with the built-in `analytic` provider (VSOP87, arc-second level).
  The `spk` provider needs a kernel file, which the module cannot open, so it answers `ESH-PROVIDER`.
- The module has no imports: no network, no file system, no clock (give `utc` or a local date and zone).

Build it from the repository with `make js`, which compiles the module, copies it into `bindings/js` and runs the
parity test against the native CLI.
