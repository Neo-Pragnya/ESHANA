---
title: "Research models: specifications before scores"
description: "Every score ESHANA computes is specified here first: factors, weights, worked examples and how weights are redistributed when an input is missing."
---

# Research models: specifications before scores

**Rule (RES-01).** No score is built until its specification is written here: the factors, their weights, worked
examples a reader can recompute by hand, and the basis for redistributing weight when a factor is unavailable. A
model has an independent name and a version; it is labelled *experimental* in every result; and it is never part of
a tradition's own output (the 36-point Aṣṭakūṭa, ṣaḍbala and the like stay as their texts define them).

A score here is a **research hypothesis** (epistemic `research_hypothesis`), not a reading. Interpretations remain
empty.

## Model 1: `research.window_score@0.1` (muhūrta window score)

**Purpose.** Rank the passing windows of a `muhurta_windows` or calendar result, so a reader can see which ones carry
more of the favourable factors. It never turns a failing window into a passing one: the constraints decide
eligibility, the score only orders what passed.

**Factors and weights.** Each factor gives a value `s` in [0, 1]; the score is the weighted mean, times 100.

| Factor | Value `s` | Weight `w` | Why this weight |
|---|---|---|---|
| `tara_bala` | 1 favourable, 0 unfavourable | 0.25 | the person's own nakṣatra relation; personal factors first |
| `chandra_bala` | 1 favourable, 0 unfavourable | 0.20 | the person's Moon-sign relation |
| `choghadiya` | auspicious 1, neutral 0.5, inauspicious 0 | 0.20 | a named quality from the pañcāṅga pack |
| `hora` | 1 when the horā lord is a natural benefic (Jupiter, Venus, Mercury, Moon), else 0.5 | 0.15 | weaker evidence; a common secondary check |
| `abhijit` | 1 inside Abhijit (not Wednesday), else 0 | 0.10 | a bonus, not a requirement |
| `duration` | min(1, minutes / 96) | 0.10 | a window shorter than two muhūrtas is less usable |

Weights sum to 1.00. They are a starting hypothesis, stated so they can be argued with; changing one is a new model
version.

**Missing inputs and redistribution.** A factor can be unavailable (no birth chart for tārā and candra bala, no
sunrise at a polar latitude). Its weight is **redistributed proportionally** over the available factors:
`w'_i = w_i / Σ_available w_j`. The result lists the factors used, the ones missing, and `coverage = Σ_available w`,
so a score computed from 55 % of the weights is never shown as if it were complete. With coverage below 0.5 the
score is not computed (unavailable).

**Outputs.** `score` (0–100), `contributions` (`w'_i · s_i · 100` per factor; they sum to the score), `sensitivity`
(`w'_i · 100`: how far the score moves if that factor's value moves from 0 to 1), `missing`, `coverage`, `model`.

**Worked example A (all factors available).** A 120-minute window in a favourable tārā (1) and an unfavourable candra
bala (0), an auspicious choghaḍiyā (1), a Saturn horā (0.5), outside Abhijit (0):

- duration `s` = min(1, 120/96) = 1;
- score = 100 × (0.25·1 + 0.20·0 + 0.20·1 + 0.15·0.5 + 0.10·0 + 0.10·1) = 100 × 0.625 = **62.5**;
- contributions 25, 0, 20, 7.5, 0, 10 (sum 62.5); sensitivities 25, 20, 20, 15, 10, 10.

**Worked example B (no birth chart).** The same window without tārā and candra bala: available weight 0.55; each
remaining weight is divided by 0.55:

- score = 100 × (0.20·1 + 0.15·0.5 + 0.10·0 + 0.10·1) / 0.55 = 100 × 0.375 / 0.55 = **68.18**;
- coverage 0.55, missing `tara_bala`, `chandra_bala`; contributions 36.36, 13.64, 0, 18.18.

These two examples are the model's unit tests.

## Model 2: birth-time evidence evaluator (design, RES-03)

See [Birth-time evidence evaluator](#birth-time-evidence-evaluator-res-03) below. It is specified, not built.

## Birth-time evidence evaluator (RES-03)

**Question.** Given an uncertain birth time (a window), and dated life events the person supplies, which candidate
times within the window are more consistent with the events under named techniques?

**Never overwrites.** The recorded birth time and its uncertainty are kept as entered. The evaluator returns
candidates with scores and the evidence behind each; choosing one is the person's act, saved as a new profile
revision (APP-02) with the evaluator's model id, never an edit of the original.

**Candidate interval.** The recorded window `[t0 − u, t0 + u]` (from `uncertainty_minutes`, or a whole day for an
unknown time), cut at every instant where a factor the techniques use changes: the Ascendant sign, the navāṃśa of the
Ascendant, the daśā sub-period at each event date. Between cuts the techniques give the same answer, so each
interval is one candidate (`[start, end)`, like every interval in the engine). The engine already provides the cuts:
`crossings` for the Ascendant and Moon, and daśā boundaries from `vedic.dasha.*`.

**Evidence.** Each event is `{date, kind, weight?}` with `kind` from a fixed list (marriage, child, career change,
relocation, loss of a parent, illness, education). A technique maps a candidate and an event to a value in [0, 1]:

| Technique id | Value for an event at a candidate | Source of the rule |
|---|---|---|
| `dasha_lord_signification` | 1 if the running daśā or antardaśā lord rules or occupies the bhāva the event kind belongs to (7th for marriage, 5th for a child, 10th for career, 4th or 12th for relocation, 9th or 4th for a parent, 6th or 8th for illness, 4th or 5th for education), else 0 | Parāśari bhāva significations |
| `transit_to_cusp` | 1 if Jupiter or Saturn transits the event bhāva's sign from the candidate's lagna on the event date, else 0 | common practice, named as such |
| `ascendant_navamsha_consistency` | 1 if the candidate's D9 Ascendant lord is placed in the event bhāva in D9, else 0 | a variant some schools use; off by default |

**Scoring.** For candidate `c`: `score(c) = Σ_events Σ_techniques v · w_event · w_technique`, normalised by the maximum
possible. Weights default to 1; each is reported. Missing data (an event without a date) is excluded and listed, never
scored as zero.

**What it reports.** Every candidate interval with its score, each event's per-technique values, the techniques
used, and a **sensitivity** check: the ranking with each technique left out. A ranking that changes when one
technique is removed is flagged as fragile.

**Not claimed.** The evaluator does not say any candidate is the true time. It reports consistency with named
techniques, which have not been validated against independently timed births. Validation needs a set of births
timed by record (hospital or civil) with events, held out from any tuning.

## Model status

| Model | Specified | Built | Label |
|---|---|---|---|
| `research.window_score@0.1` | yes | RES-02 prototype (`research_window_score`) | experimental |
| birth-time evidence evaluator | yes (design) | no | — |
