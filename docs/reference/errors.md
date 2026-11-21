---
title: "Error codes"
description: "Every ESH-* code, what causes it, and the fix. Codes are stable."
---

# Error codes

Errors are values: `{"kind": "error", "error": {"code", "message"}}`. The codes are stable; messages may improve.
`eshana explain ESH-…` prints the one-line meaning.

| Code | Cause | Fix |
|---|---|---|
| `ESH-INPUT` | malformed JSON, unknown field, impossible date, unknown body or option, missing timezone, bad coordinates | correct the request; the message names the field |
| `ESH-CONFIG` | sidereal without an ayanāṃśa; tropical with one; Vedic with a tropical zodiac; `spk` without a kernel; Human Design with sidereal; unknown provider | state the setting; the message says which |
| `ESH-TZ-UNKNOWN` | the zone is not in the bundled IANA database | use a canonical name (`Asia/Kolkata`, not `IST`) |
| `ESH-TIME-AMBIGUOUS` | the local time occurs twice (clocks went back) | pass `fold=earliest` or `fold=latest`, or give UTC |
| `ESH-TIME-NONEXISTENT` | the local time was skipped (clocks went forward) | give the intended time or UTC; there is no policy to choose |
| `ESH-TIME-UNCERTAIN` | Human Design needs an exact time; (as a warning) houses or daśā were omitted for an uncertain time | supply an exact time, or accept the intervals |
| `ESH-RANGE` | the instant is outside the provider's range (analytic 1800–2100; spk the kernel span) | choose another provider or date |
| `ESH-BODY` | the provider cannot supply the body | pick a supported body |
| `ESH-LATITUDE` | houses are undefined at this latitude (Placidus in the polar regions) | use `whole_sign` or `equal` |
| `ESH-PROVIDER` | the kernel is missing, corrupt, big-endian or not type 2 | re-download `de440s.bsp` |
| `ESH-RULES` | an unimplemented divisional chart, an unknown year convention or school, a name with no Latin letters | choose a supported option |

## Adding a code

Add it to `EshanaError` in `eshana-core`, to `catalog()`, to this table, and give it an acceptance scenario. A unit test fails
if a variant is missing from the catalogue; the acceptance suite fails if `eshana explain` does not know a catalogued code.
