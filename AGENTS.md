# Working in ESHANA

Instructions for coding assistants and contributors. Read these before changing code.

## Boundaries (non-negotiable)

- **Compute → Derive → Interpret.** Positions are `calculated`; rule results are `tradition_rule` and name their rule pack
  (`id@version`); the engine never emits an interpretation (`interpretations` stays empty). Do not blur the layers.
- **One entry point.** `Engine::execute_json` is the only way in. The CLI and Python are thin shells. Never implement a rule
  in a binding, and never let the CLI and Python build different requests for the same thing.
- **One frame.** Providers return geocentric apparent ecliptic longitude/latitude of the true equinox of date. A new provider
  matches that definition and states a *measured* accuracy.
- **No silent defaults, no invented inputs.** Sidereal needs an explicit ayanāṃśa; local time needs a zone; an unknown birth
  time is a window, not noon; an undefined house system is an error. If you must default, record it in
  `provenance.assumptions`.
- **Errors carry stable codes.** Add the variant to `eshana-core`, the catalogue, `docs/reference/errors.md` and an acceptance
  scenario before using it.
- **No AGPL in the build.** The Swiss Ephemeris is a test oracle only (numbers in `benchmarks/golden`). `deny.toml` is the
  allow-list.
- **Privacy.** The engine writes no files, logs nothing and uses no network.

## Work discipline

- The queue is `planning/backlog.json`; keep task ids. `make tracking` regenerates `planning/TRACKING.md`; `make tracking-check`
  fails when it is stale. Status words are exact: *proposed*, *implemented*, *verified* (needs evidence), *released*.
- Each change ships code, a meaningful test, docs and, for a user-visible behaviour, an acceptance scenario.
- A new numerical claim needs a reference (an independent computation or a published worked example). Say how it was checked.
- Tables are cross-checked by invariants (Human Design gates partition into centres, etc.). Add an invariant, not just a value.

## Commands

```bash
make install            # .venv with maturin, pytest, pytest-bdd, jsonschema
make verify             # fmt-check, clippy -D warnings, all Rust tests, Python tests, tracker freshness
make acceptance         # capability scenarios + release gate (PROMOTE / CONDITIONAL / BLOCK)
make acceptance-wheel   # the same against the built wheel in a fresh venv
make accuracy           # measured error table (analytic); ESHANA_SPK=… for the kernel
make demo               # regenerate website/src/data from the real CLI (python3 scripts/build_demo.py --check to test)
make docs-site          # demo + docs check + Astro build
make scan               # build the wheel and scan it like a package firewall would
```

`acceptance/harness.py` and `scripts/scan_dist.py` are byte-identical copies from Ananke Plexus / Dev Guru. Change them there.
Regenerating a golden fixture needs `skyfield` (and `pyswisseph` for the oracle); the scripts are described in
`benchmarks/README.md`.
