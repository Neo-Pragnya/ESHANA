---
title: "Symbolic graph"
description: "Export a chart as a typed graph (JSON, Mermaid, GraphML, SVG, draw.io, Chitkala) with relation classes that never claim causation."
---

# Symbolic graph

```bash
eshana graph vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 --divisions 9 --format mermaid
```

## Nodes

`input`, `position`, `derived_attribute`, `concept` (a rāśi or nakshatra name), `rule` and `source`. Each has an
`epistemic` label.

## Edges

`calculated_from`, `classified_as`, `uses_rule`, `associated_with`, `contrasts_with`, `derived_from`, `interpreted_as`,
`cites`, each with a **relation class**:

| Class | Meaning | Emitted by the engine |
|---|---|---|
| `computational` | produced by a calculation or a rule application | yes |
| `traditional_correspondence` | a tradition's symbolic association | yes |
| `correlational` | an observed statistical association | **no** (for your own graphs) |
| `causal` | a claimed causal mechanism | **no** |
| `hypothesized` | proposed, untested | **no** |

`Position → classified_as → Nakshatra` is a *traditional correspondence*; `Position → calculated_from → Input` is
*computational*. A test asserts the graph never contains a causal or correlational edge.

## Formats

- `json`: `{nodes, edges, relation_classes, note}`.
- `mermaid`: a `flowchart LR` coloured by epistemic class; correspondences are dashed. Pipes straight into a renderer.
- `graphml`: well-formed XML for Gephi, yEd and similar (checked by the acceptance suite).
- `svg`: a self-contained picture, one column per node type (inputs, positions, attributes, concepts, rules,
  sources), boxes coloured by epistemic class, long labels shortened with the full text in a tooltip.
- `drawio`: an `mxfile` that diagrams.net opens and edits, with the same layout and colours.
- `chitkala`: the graph in Chitkala's import schema (metadata, universes, nodes, edges, sources), for interactive
  2D and 3D exploration in [Chitkala](https://github.com/Neo-Pragnya).
