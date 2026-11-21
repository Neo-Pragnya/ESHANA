---
title: "Vedic chart"
description: "Sidereal positions, rāśi, nakshatra, bhāva, divisional charts and Vimśottarī daśā."
---

# Vedic chart

```bash
eshana chart vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 \
                   --ayanamsha lahiri --houses whole_sign --node mean --divisions 1,9,10 --dasha
```

## Choices you must be able to state

| Choice | Options | Notes |
|---|---|---|
| Ayanāṃśa | `lahiri`, `raman`, `krishnamurti`, `fagan_bradley` | recorded as `ayanamsha-<id>@0.1`; the CLI defaults to Lahiri **and records it** |
| Node | `mean`, `true` | the true node is osculating; both are reported with the south node opposite |
| Houses | `whole_sign` (traditional), `equal`, `placidus` | Vedic bhāva is whole-sign from the lagna in `vedic.*.bhava`; the `houses` block follows your choice |
| Daśā year | `365.25`, `365.2425`, `360`, `365.25636` | changes the dates; recorded in the result |

## What you get

For each body and the lagna: `vedic.<body>.rashi`, `.nakshatra` (index, name, pada, lord), `.bhava`, and `.d<N>` for each
requested division. With `--dasha`, `vedic.dasha.vimshottari`: nine mahādaśās each with nine antardaśās, the first clipped
at birth, and the balance at birth in years.

```json
{ "id": "vedic.dasha.vimshottari", "value": {
    "starting_lord": "Mercury", "balance_at_birth_years": 1.54, "year_convention": "365.25",
    "mahadashas": [ { "lord": "Mercury", "clipped_at_birth": true, "start_utc": "1992-05-28T14:35:00Z",
                      "end_utc": "1993-12-12T18:07:21.579Z", "antardashas": [ "…" ] } ] } }
```

## Divisional charts

Implemented under `vedic.parashari@0.3`: D1, D3, D4, D7, D9, D10, D12. D30 and D60 return `ESH-RULES`: they need a
documented school choice first ([roadmap](../roadmap.md)). Formulas: [Calculation conventions](../concepts/calculation-conventions.md).

## Reading disagreements

Two software packages may disagree on a nakshatra pada because they use different ayanāṃśas, nodes or sidereal-time
conventions, not because one is wrong. `eshana compare` shows exactly which setting moves which placement:

```bash
eshana compare vedic ... --a ayanamsha=lahiri --b ayanamsha=krishnamurti
```

## Accuracy

At the `analytic` provider's accuracy a body within about 0.01° of a sign, nakshatra or pada boundary may fall on the
other side. Use `--provider spk` for boundary cases, or an uncertainty window (see [Uncertain birth time](uncertain-birth-time.md)).
