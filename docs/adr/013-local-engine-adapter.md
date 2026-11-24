---
title: "ADR-013: The studio's engine adapter is the engine compiled to WebAssembly"
description: "How the local application recomputes offline: the same execute_json in the browser, opt-in per file, with report-only files as the fallback."
---

# ADR-013: The studio's engine adapter is the engine compiled to WebAssembly

**Status.** Accepted (2026-11-23, `APP-01`) by the owner as written: opt-in with `eshana-report --live`; report-only files stay the default.

## Context

The studio (`eshana-report`) is one self-contained HTML file: the engine runs once in Python, and the page renders the
envelopes it embedded. That is enough to read a chart, but a local application needs to *recompute*: move a date, add a
transit, try a placement, open a horary number. Every recomputation must use the engine's rules (AGENTS.md: the studio
never computes a factor of its own), work offline, and give the same answer as the CLI and Python.

## Options

| Route | Offline | Same rules | Cost |
|---|---|---|---|
| Report-only (today) | yes | yes, but only what was embedded | cannot recompute anything |
| Local Python server (`eshana serve` on localhost) | yes | yes | a process to start and stop; a port; a browser that may refuse mixed origins; not a single file |
| Re-implement in JavaScript | yes | **no**: a second rule base to keep equal | rejected |
| **The engine compiled to WebAssembly** | yes | yes: the same `execute_json` | ~3.0 MB module (~4.1 MB base64 inside the file); no SPK kernel in the browser |

## Decision

1. A new crate, `crates/eshana-wasm`, compiles `eshana_engine::Engine::execute_json` to `wasm32-unknown-unknown`
   with a three-function ABI (`eshana_alloc`, `eshana_execute`, `eshana_free`). It has no imports and no bindings
   generator, so it runs in any WebAssembly host. It is the only crate allowed `unsafe`, limited to that boundary.
2. `make wasm` builds it into the wheel as `eshana/data/eshana_engine.wasm`. `eshana-report --live` embeds it, base64,
   in an inert `<script type="application/octet-stream" id="eshana-engine">`; `studio-live.js` instantiates it from
   those bytes. Nothing is fetched; the file stays self-contained.
3. Without `--live` a file is report-only, as before. Pages that need recomputation say so instead of approximating.
4. **Parity contract.** The browser and native builds use different `libm` implementations, so the last bit of a
   transcendental function can differ. Envelopes are equal in structure, strings and integers, and floating-point
   values agree within 1e-9 relative; `acceptance/studio/wasm_parity.mjs` checks this on every operation family.
   Error envelopes agree on the code (a message can name a platform error).

## What a report-only file cannot do

- recompute after a settings change (ayanāṃśa, houses, node model): it shows what was computed at generation;
- transits or progressions for a date outside the embedded series;
- the playground (positions placed by hand), a new KP horary number, a new muhūrta or calendar range;
- anything with the `spk` provider in the browser, even with `--live` (the module cannot open a kernel file; it
  answers `ESH-PROVIDER`). Loading a kernel through a file picker is possible later and needs its own decision.

## Consequences

- The application's pages (compare workspace, playground) call the engine through one adapter, `LIVE.execute`, and
  fall back to embedded data or a typed "unavailable" when it is absent.
- A live file is about 4 MB larger. Report-only stays the default.
- CI builds the WebAssembly target (`rustup target add wasm32-unknown-unknown`) for the wheel and runs the parity
  check and the studio journeys against a live file.
