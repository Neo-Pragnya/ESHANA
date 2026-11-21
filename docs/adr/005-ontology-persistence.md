---
title: "ADR-005: Ontology persistence"
description: "In-memory graph built from a result, exported as JSON, Mermaid and GraphML; no database in the core."
---

# ADR-005: Ontology persistence

**Status.** Accepted.

## Decision

The symbolic graph is derived on demand from a result envelope. It is not stored by the engine. Exports are JSON,
Mermaid and GraphML. Edges carry a relation class so causal, correlational, traditional and computational links are never one
unlabelled field; the engine emits only the last two. Database connectors, if ever, are optional adapters outside the core.

## Consequences

The core needs no graph database and stays embeddable. Chitkala and other consumers read the JSON.
