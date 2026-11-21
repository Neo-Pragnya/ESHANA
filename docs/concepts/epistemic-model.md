---
title: "The epistemic model"
description: "Three layers, four labels. How ESHANA keeps a calculation from being mistaken for validation of a tradition."
---

# The epistemic model

The most important sentence in the project: **a calculation is not an endorsement.** That Mars is at 23°31′ Pisces is
a fact about geometry. That this placement "means" anything is a tradition's claim. ESHANA keeps the two apart in
the data, not just in the docs.

## Three layers

| Layer | Label in JSON | What it is | How it is checked |
|---|---|---|---|
| Calculated | `calculated` | UTC instant, Julian day, longitude, latitude, speed, Ascendant | against JPL DE440s and the Swiss Ephemeris ([accuracy](../reference/accuracy.md)) |
| Derived by a named system | `tradition_rule` | nakshatra, divisional sign, aspect, house, HD gate, name number | by unit tests of the rule and by published worked examples |
| Interpretive | `speculative_interpretation`, `research_hypothesis` | character, narrative, compatibility, "meaning" | **not produced by the engine** |

Every position has `"epistemic": "calculated"`. Every derived attribute has `"epistemic": "tradition_rule"`, a
`tradition_id`, a `rule_version` (`pack@version`) and the ids it was `derived_from`. The `interpretations` array is
present in every result and is always empty: an interpretation service may fill it downstream, with its own model
version, and must say so.

## Invariants the engine enforces

These are tested in `crates/eshana-engine/tests/behaviour.rs` and in the acceptance suite.

- **Tropical and sidereal are never merged.** A sidereal request without an ayanāṃśa is `ESH-CONFIG`. A tropical request
  with one is `ESH-CONFIG`. Both longitudes are reported; the difference is the ayanāṃśa.
- **No silent defaults in the library.** Only the CLI's `chart vedic` defaults to Lahiri, and the result says so in
  `provenance.assumptions`. Python and the raw API require it.
- **Mean and true node are distinct** and never substituted.
- **House systems are explicit** and refused where undefined (Placidus within the polar circles is `ESH-LATITUDE`).
- **Birth time is never invented.** Unknown or approximate times produce intervals and stability flags; the Ascendant and
  houses are omitted with a warning rather than guessed.
- **Schools are never merged.** Pythagorean and Chaldean are separate rule packs; there is no blended number.
- **No causal edges.** The graph's relation classes include `causal` and `correlational` so researchers can label their
  own graphs, but the engine emits only `computational` and `traditional_correspondence`.
- **No universal scores.** There is no `destiny_score` and no `compatibility_score`, and there will not be one without a
  named, versioned, validated method.

## Why this matters for products

A product that shows a nakshatra next to a position is showing a rule applied to a measurement. A product that shows a
paragraph about personality is showing someone's interpretation. Mixing them makes the second inherit the authority of
the first. The envelope is shaped so a UI can show them differently, and so an auditor can find the rule behind any
derived number.
