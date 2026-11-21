---
title: "ADR-008: Licensing model"
description: "Apache-2.0 core, enforced by an allow-list."
---

# ADR-008: Licensing model

**Status.** Accepted.

## Decision

Apache-2.0 for every crate and the Python package. `deny.toml` allows MIT, Apache-2.0, BSD, ISC, Unicode and Zlib; anything
else fails CI. Data and rule content must be independently implemented or clearly licensed; third-party text is not copied.
Open core remains the business model option; a hosted API, if built, must not be required.
