# Roadmap waves (from 2026-11-24)

The open backlog, ordered by the architecture: each wave builds on the one before, ships with tests, docs, tracker
evidence and a release, and leaves the engine's guarantees intact (one JSON entry point, no silent defaults, every
rule named, offline by default). Task ids are those in `planning/backlog.json`.

| Wave | Theme | Tasks | Why this order |
|---|---|---|---|
| **1** | Time and ephemeris foundations | TIME-03 calendars and BCE dates · TIME-05 ΔT across millennia and the forecast · PROV-04 SPK type 3, large kernels on demand · PROV-06 kernel catalogue, verification, checked download · AST-08 rise, set and topocentric positions · AST-03 more house systems · AYA-02 star-based ayanāṃśas | Every chart, event and tradition depends on the instant, the kernel and the frame being right. |
| **2** | Events and returns | AST-06 returns and eclipses · WES-03 solar and lunar returns · WES-05 solar arc · VED-06 gochara, retrograde windows, combustion · AST-07 optional bodies (Chiron, asteroids) from SPK | Built on the wave-1 solver inputs and the shared event solver (EVT-01). |
| **3** | Tradition packs and conventions | VED-05 and WES-04 separate rule packs · RULE-02 user packs at run time · VED-03 D30/D60 schools · VED-09 D150 and non-Parāśari vargas · MAT-02 rajju and vedha · NUM-03 cycles and karmic debts · NUM-04 other alphabets · WES-02 synastry and composite · LK-01 Lal Kitab | Rule packs first, so the new traditions arrive as named, versioned, validated data. |
| **4** | Surfaces and bindings | BIND-04 browser WASM package · BIND-03 Node bindings · BIND-05 MCP server · GRA-04 SVG and draw.io graph exports · GRA-03 CHITKALA consumer · REP-06 East Indian chart · REP-07 drag in the playground | One engine, more doors; each surface renders, none computes. |
| **5** | Quality, security, docs | QA-06 fuzzing · PERF-01 benchmarks · QA-07 website journeys · SEC-03 SBOM and attestations · DOC-04 notebooks and agent recipes | Hardening once the surface area is in place. |

## Waiting on the owner or on references (kept on the to-do list)

| Task | Needs |
|---|---|
| MUH-03, DASH-01 | expert review of the profiles and worked daśā examples |
| BASE-03, VED-14 | PyJHora permission rule or the reference program's ṣaḍbala table |
| RES-03 | design review |
| HD-04 → HD-06 | an external Human Design reference chart with documented conventions |
| CHK-23, CHK-24, FB-29, FB-31…34 | the reference program's conventions (read from its output) |
| ACC-04 | more published reference charts with full input and output |
| BIND-07 → DOC-05 | a crates.io token (and the crate names, NAME-01/02) |
| BIND-08 (Windows, Intel macOS), SEC-05 | the PyPI trusted-publisher setup |
| PROV-03 | a licensing decision: an AGPL-3.0 Swiss Ephemeris adapter as a separate package |
| new Hindi labels | owner review (OWNER-INPUTS 5b) |
