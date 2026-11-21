# Contributing to ESHANA

Thank you. Read [`AGENTS.md`](AGENTS.md) for the non-negotiables (the epistemic layers, one entry point, no silent defaults, no
AGPL in the build) and [`docs/concepts/epistemic-model.md`](docs/concepts/epistemic-model.md) for the reasoning.

## Setup

```bash
git clone https://github.com/Neo-Pragnya/ESHANA && cd ESHANA
make install      # needs Rust (rustup.rs) and uv
make verify
```

## A good change

1. **A test that fails first** and states the behaviour (`crates/*/tests`, or a Gherkin scenario in `acceptance/features`
   when it is user-visible).
2. **A reference for any new number**: an independent computation (Skyfield, JPL), a published worked example (cite it),
   or, for rules, the source text and edition. "It looks right" is not evidence.
3. **Docs**: the page that describes the behaviour, and `docs/reference/errors.md` for a new code.
4. **The tracker**: update `planning/backlog.json` (keep the id; `verified` needs evidence) and run `make tracking`.
5. `make verify` and `make acceptance` pass.

## Adding a tradition or a rule

Tradition id, named convention, source and edition, worked examples from the source, boundary tests, a pack version, and a
statement of what the rule does **not** claim. Do not blend traditions. Do not copy third-party text or tables; implement the
mathematics and the structure independently, and note any rights question as an ADR.

## Adding an ephemeris provider

Match the single frame, state a measured accuracy with its method, and add a golden test with a tolerance you can defend.
A provider with a copyleft or commercial licence goes in its own crate behind a feature flag, off by default (ADR-001).

## Reporting problems

Wrong numbers: include the request JSON, the `engine_version` and the reference you compared with. Security: see
[`SECURITY.md`](SECURITY.md).
