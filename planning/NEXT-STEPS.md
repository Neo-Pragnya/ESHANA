<!-- Mirror of docs/next-steps.md; edit that file and copy here. -->
# Next steps

State on 2026-11-21. The ledger of every task is [`planning/TRACKING.md`](TRACKING.md), generated from
`planning/backlog.json`. This page is the readable order of work.

## Done in this round

- **Open source as libraries** ([ADR-011](../docs/adr/011-open-source-distribution.md)): the naming, Human Design and Swiss
  Ephemeris questions are closed for a non-commercial open-source release.
- **HTML chart reports** ([guide](../docs/guides/html-report.md)): `eshana-report`, `Chart.to_html()`, Jupyter display and
  `--from-json`, built on the Panchang Chakra visual language. It includes South and North Indian charts, vargas,
  daśā, the Western wheel, a BodyGraph, numerology and provenance.
- **Birth-chart accuracy suite** ([guide](../docs/guides/accuracy-suite.md)): 44 reference charts, ProKerala and the Lahiri
  definition, independent re-derivation; about 970 checks. It found and fixed three engine bugs.
- **Architecture diagrams as code** ([diagrams](../docs/architecture/diagrams.md)) and new guides:
  [ways to use](../docs/guides/ways-to-use.md), [publishing](../docs/guides/publishing.md).
- **Released to PyPI**: [`eshana` 0.1.0](https://pypi.org/project/eshana/0.1.0/) on 2026-11-21, with a macOS arm64
  wheel and the sdist. `pip install eshana` works everywhere: other platforms build from the sdist, which needs Rust.
  The process is in [publishing](../docs/guides/publishing.md).

## Next, in order

1. **First CI run on GitHub.** Push the repository and let `ci`, `acceptance`, `accuracy`, `security`, `docs-site` and
   `workflow-lint` run. Tasks that are *implemented* become *verified* with those runs as evidence. Enable GitHub
   Pages (Settings → Pages → Source: GitHub Actions).
2. **Wheels for every platform.** Add the GitHub publisher on PyPI (trusted publishing) and push tag `v0.1.0`.
   `release.yml` then adds Linux, Intel macOS and Windows wheels to 0.1.0, so users on those platforms no longer need
   Rust. This machine has no `rustup` or running Docker, so it cannot cross-build them.
3. **More published references** (`ACC-04`). Add charts from pages that print their input in full, following the
   recipe in the accuracy guide. Each one is a few lines of JSON.
4. **Event search** (`AST-06`): transits, ingresses, stations and returns. This unlocks transit sections in the
   report (`VED-06`, `WES-03`).
5. **Swiss Ephemeris adapter** (`PROV-03`): a separate `eshana-swisseph` package under AGPL-3.0, never linked into
   the Apache-2.0 core.
6. **Bindings**: an MCP server (`BIND-05`) so assistants can call the engine; Node and WASM (`BIND-03`, `BIND-04`);
   crates.io (`BIND-07`).
7. **Report extras** (`REP-05`): a Panchāṅga day section (tithi, vāra, yoga, karaṇa), more vargas (D2, D16–D60 once
   their rules are added to the engine), and a PDF preset.

## Decisions still open

None block a release. Two choices remain for later: whether data-driven rule packs (ADR-004) should use YAML or JSON, and whether
the report should offer a non-schematic BodyGraph drawing.
