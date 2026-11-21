---
title: "ADR-006: API shape"
description: "One JSON entry point; every binding a thin shell; six crates at launch."
---

# ADR-006: API shape

**Status.** Accepted.

## Decision

`Engine::execute_json(request) -> envelope` is the only way in. The CLI and the Python module are shells around it. Bindings
never reimplement a rule, so cross-language parity is structural and is a gate scenario. Start with six crates and keep
traditions as modules until the APIs stabilise; the master plan's sixteen-crate layout is the destination, not the start.

## Consequences

Adding Node (NAPI-RS) or WASM is a thin wrapper. Typed Rust callers can use `Engine::execute(&Request)`. The envelope's
`schema_version` is the contract; results are validated against `schemas/envelope.schema.json`.
