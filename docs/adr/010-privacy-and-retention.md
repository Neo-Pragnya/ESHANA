---
title: "ADR-010: Privacy and data retention"
description: "Local-first; zero retention by default."
---

# ADR-010: Privacy and data retention

**Status.** Accepted.

## Decision

Birth data is personal data. The engine computes locally, writes no files, logs nothing and sends nothing. Any future service
defaults to zero server retention with explicit opt-in storage, encryption at rest and a deletion workflow. Uncertainty is
represented, not defaulted. These behaviours are acceptance scenarios (`@cap_privacy`).
