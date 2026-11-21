---
title: "ADR-002: Time representation"
description: "Civil input resolved once, to a typed instant on every scale the engine uses, with the tz database version recorded."
---

# ADR-002: Time representation

**Status.** Accepted.

## Decision

Civil time (date, time, IANA zone) is resolved exactly once into an `Instant` carrying UTC text, `jd_ut`, `jd_tt`, the ΔT
used and the model name. The zone name, the offset applied and the tz database version are recorded. Ambiguous and
non-existent local times are distinct errors; there is no default fold policy. UTC is treated as UT1 (recorded
assumption). Unknown or approximate times are represented as a window, never as a guessed instant.

## Consequences

Round-trips and monotonicity are property-tested; DST edge cases are unit-tested; results can be audited without the tz
database because the resolved instant is echoed. Leap seconds beyond 2027 are a forecast and flagged.
