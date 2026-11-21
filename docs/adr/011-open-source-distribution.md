---
title: "ADR-011: Open-source distribution as libraries"
description: "ESHANA is an open-source tool stack published as libraries, not a commercial service. This closes the commercial caveats in ADR-001, 007, 008 and 009."
---

# ADR-011: Open-source distribution as libraries

**Status.** Accepted (owner decision, 2026-11-20). Amends [ADR-001](001-ephemeris-provider-and-licensing.md),
[ADR-007](007-naming-and-scope.md), [ADR-008](008-licensing-model.md) and [ADR-009](009-human-design-rights.md).

## Context

Earlier ADRs left three questions open "before a commercial launch": name clearance, Human Design rights and a Swiss
Ephemeris adapter. The owner has decided that ESHANA will **not** be launched commercially. It is an open-source tool
stack, published as libraries (PyPI first, crates.io next) for anyone to use, study and extend.

## Decision

1. **Licence.** Every ESHANA crate and the Python package stay **Apache-2.0**. A permissive licence lets other
   open-source projects, notebooks and apps depend on ESHANA without changing their own licence.
2. **Human Design.** The calculation (the 88° solar-arc solve, the gate wheel, channels, centres and the
   type/authority/profile/definition derivation) is implemented in full and published. ADR-009 is **closed for this
   scope**: there is no commercial launch to clear. One rule still holds, and it is about copyright rather than our
   licence: ESHANA ships no copied descriptive text, artwork or third-party tables. That keeps the open-source
   package clean for everyone who redistributes it.
3. **Swiss Ephemeris.** Allowed as an **optional, separately distributed adapter** (`eshana-swisseph`) under
   **AGPL-3.0**, the Swiss Ephemeris's own open-source licence. It is never linked into the Apache-2.0 crates or
   wheels, so the core stays permissive and users choose whether to add the AGPL component. It is planned (`PROV-03`),
   not built. Until it exists, the Swiss Ephemeris remains an offline test oracle only.
4. **Name.** The package name `eshana` was free on PyPI and crates.io on 2026-11-20 (both returned 404). A formal
   trademark search is not required for a non-commercial open-source release. The residual risk (someone else's
   prior mark) is accepted and would be handled by renaming the distribution, not the import name.
5. **Publishing.** Releases go to PyPI with `uv publish`. The token comes from the maintainer's environment (`.env`,
   git-ignored), never from the repository. See [publishing](../guides/publishing.md).

## Consequences

- `NAME-01`, `NAME-02` and `HD-05` are closed by this decision. `PROV-03` becomes "build the separate AGPL adapter".
- Nothing in the code changes because of this ADR. The doctrine (Compute → Derive → Interpret, labelled values, no
  interpretation in the engine) is unaffected.
- If the project ever becomes commercial, this ADR is superseded by a new one and the open questions return.
