---
title: "Conformance and release gate"
description: "The test pyramid, the capability acceptance suite and the PROMOTE/CONDITIONAL/BLOCK decision."
---

# Conformance and release gate

| Level | What | Where | Gate |
|---|---|---|---|
| Unit and property | arithmetic, conventions, boundaries; fuzz-style properties of the time module | `cargo test` | mandatory |
| Golden / differential | positions vs JPL DE440s; ayanāṃśa, houses, nodes, sidereal vs the Swiss oracle | `crates/*/tests`, `benchmarks/golden` | mandatory |
| Behavioural | the doctrine's invariants through the JSON API | `crates/eshana-engine/tests/behaviour.rs` | mandatory |
| Cross-language parity | Python facade and CLI return the same JSON; both validate against the schema | `crates/eshana-python/tests`, acceptance | mandatory |
| **Capability acceptance** | 17 capabilities, 74 Gherkin scenarios driving the real `eshana` binary in sandboxes | `acceptance/` | **the release gate** |
| Supply chain | `cargo deny` (licences, advisories), `cargo audit`, dist scan of the built wheel | CI | mandatory |
| Performance | not yet baselined | n/a | tracked as `PERF-01` |

## The gate

`make acceptance` writes `acceptance/report/summary.md`:

- **PROMOTE**: every capability proven, no known issues.
- **CONDITIONAL**: a known issue, an unverified capability, a non-blocker failure or low CLI-surface coverage.
- **BLOCK**: a blocker capability failed or has no scenario.

Scenarios are tagged `@cap_<id>`; `acceptance/capabilities.yaml` lists them, with `blocker: true` for those that veto a
release. `acceptance/harness.py` is shared byte-for-byte with Ananke Plexus, Dex Trove and Dev Guru; repository-specific steps
live in `conftest.py`. `make acceptance-wheel` repeats the run against the built wheel in a fresh virtualenv, which is
what ships.

## Scenarios worth reading

Each states a promise of the doctrine as an observable behaviour: a sidereal request without an ayanāṃśa is refused; a
repeated local time is an error with a distinct code; polar Placidus is refused while the same request with whole sign works;
an unknown birth time yields intervals and no Ascendant; the design instant is 88° not 88 days; the graph has no causal
edge; the same request returns the same bytes; a chart leaves nothing on disk.
