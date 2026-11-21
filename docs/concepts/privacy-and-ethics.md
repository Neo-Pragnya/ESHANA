---
title: "Privacy and ethics"
description: "Birth data is personal data. What ESHANA does and refuses to do about it."
---

# Privacy and ethics

## Privacy

Birth date, time and place identify people. ESHANA is built so the safe path is the default:

- **Local-first.** Computation is in-process. No network access, no telemetry, no account, no kernel download (you
  supply the optional kernel).
- **No retention.** The engine writes no files, keeps no history and logs nothing. The cache holds provider data, never
  requests. The acceptance suite asserts a chart leaves the project and `$HOME` untouched and writes nothing to stderr.
- **Minimal fields.** Latitude and longitude are needed only for houses; the geocentric positions do not use them. The
  altitude is recorded, not used.
- **Uncertainty is first class.** An unknown time is represented as unknown, not as noon.
- If you persist results, treat them as personal data: encrypt, minimise logs, provide deletion.

## Ethics

- No deterministic claims about people. Do not present compatibility or personality classifications as assessments of
  suitability, health, employability or fate. The engine emits no such outputs.
- Label interpretations. Anything an interpretation layer adds belongs under `interpretations`, with its own model
  version, and a UI should visually separate it from calculated values.
- **Trading.** Astrological and numerological timing features are *unvalidated hypotheses*. If you build backtests, guard
  against look-ahead bias, p-hacking and overfitting, hold out data, and make no claim of edge without evidence. ESHANA
  provides timestamp-safe computation, not investment signals.
- **Respect for traditions.** Name the school (`vedic.parashari`, `numerology.chaldean`) and cite sources. Do not blend
  traditions silently.

## What would change this

A hosted API is on the roadmap. If it ships, the default stays zero server retention ([ADR-010](../adr/010-privacy-and-retention.md)),
with an explicit opt-in for anything stored.
