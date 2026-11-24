---
title: "ADR-014: One WebAssembly package for browsers and Node"
description: "The JavaScript binding is the WebAssembly engine with a small loader, not a native Node add-on."
---

# ADR-014: One WebAssembly package for browsers and Node

**Status:** Accepted (2026-11-24) · **Tasks:** BIND-03, BIND-04

## Context

The roadmap listed Node.js bindings through NAPI-RS (BIND-03) and browser bindings through WebAssembly (BIND-04).
The studio already runs the engine as a WebAssembly module (ADR-013) with a three-function ABI and no imports.

## Decision

Ship one JavaScript package, `bindings/js`, that loads that same module in both places: Node reads the `.wasm` next to
the loader, a browser fetches it. The API is `load()` → `{ execute, executeJson, version }`, with `EshanaError`
carrying the stable code, as in Python.

## Consequences

- One artefact for every JavaScript host, built once on any machine; no per-platform native builds, no prebuilt
  binaries to sign, no node-gyp.
- The results are the WebAssembly engine's, already held to the native CLI by the parity test (1e-9 relative).
- The `spk` provider is not available (a WebAssembly module cannot open files); the `analytic` provider (VSOP87 since
  0.5.0) is arc-second class without data.
- A native add-on stays possible later if a host needs kernel files from JavaScript; nothing here prevents it.
