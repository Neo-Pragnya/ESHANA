---
title: "Maitri and dignities"
description: "Natural, temporary and compound relationships of the nine grahas, the nakshatra spatial matrix of tārās, and each graha's dignity in the sixteen divisions, with every convention named."
---

# Maitri and dignities

The `maitri` feature (included in `features: ["all"]`) adds three attributes, all `tradition_rule` values under
rule `vedic.maitri@0.1`. The tables and every convention choice are in the rule pack
[`maitri.yaml`](../../crates/eshana-engine/rulesets/maitri.yaml); the natural friendship of the seven grahas, the
temporary-friend houses and the BPHS dignity ranges come from
[`vedic-core.yaml`](../../crates/eshana-engine/rulesets/vedic-core.yaml). The engine computes; it does not interpret.

```bash
eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
  --node true --features maitri --compact
```

## Relationships (`vedic.maitri.relationships`)

One row for every ordered pair of the nine grahas (Sun … Saturn, Rāhu, Ketu): 72 rows.

| Field | Meaning |
|---|---|
| `from`, `to` | The graha and the graha it regards. |
| `natural` | Naisargika relationship: `friend`, `neutral` or `enemy`. |
| `natural_source` | `vedic-core.yaml (BPHS)` for two of the seven; `maitri.yaml node table` when a node is involved. |
| `temporal` | Tatkālika relationship: `friend` when `to` is 2nd, 3rd, 4th, 10th, 11th or 12th from `from` by sign, else `enemy`. |
| `temporal_house` | That count (1–12). |
| `compound` | Pañcadhā: friend + friend = `great_friend`; friend + enemy or enemy + friend = `neutral`; neutral + friend = `friend`; neutral + enemy = `enemy`; enemy + enemy = `great_enemy`. |

**The nodes.** BPHS gives no natural-friendship table for Rāhu and Ketu, and later authors disagree. ESHANA's
table was inferred from the compound relationships of a published reference chart (by removing the temporary
part from each compound value) and is used as written:

| | Friends | Neutral | Enemies |
|---|---|---|---|
| Rāhu | Jupiter, Venus, Saturn | Mercury | Sun, Moon, Mars, Ketu |
| Ketu | Mars, Venus | Mercury, Jupiter | Sun, Moon, Saturn, Rāhu |

How the seven regard the nodes: Sun and Moon count both as enemies; Mars counts Rāhu an enemy and Ketu a friend;
Mercury and Jupiter are neutral to both; Venus is a friend to both; Saturn counts Rāhu a friend and Ketu an enemy.
This agrees with the commonly printed view that the nodes are hostile to the luminaries and friendly to Venus.
Edit `node_natural` and `towards_nodes` in the pack to follow another school.

## Nakshatra spatial matrix (`vedic.maitri.tara_matrix`)

For the lagna and the nine grahas, every ordered pair (90 cells): the tārā of the row's nakshatra counted from the
column's nakshatra.

- `count = ((row nakshatra − column nakshatra) mod 27) + 1` (1–27);
- `tara = ((count − 1) mod 9) + 1`: 1 Janma, 2 Sampat, 3 Vipat, 4 Kshema, 5 Pratyari, 6 Sadhaka, 7 Vadha, 8 Mitra,
  9 Parama mitra;
- `nature`: Vipat, Pratyari and Vadha are `inauspicious`, Janma is `mixed`, the others `auspicious` (the usual
  muhūrta grouping).

The direction (from the column to the row) is the one the reference chart uses: its lagna row and Sun column shows
Kshema, which is the lagna's nakshatra (Chitra) counted from the Sun's (Purva Ashadha), the 22nd. The value also
carries `points`, each point's `nakshatra` and the list of `taras`.

## Dignities in the sixteen divisions (`vedic.maitri.varga_dignities`)

For each graha and each of D1, D2, D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30, D40, D45 and D60 (signs from
the engine's Parāśari varga rules): the varga `sign`, the `degree` inside the varga sign, the sign `lord`, the
`dignity` and a `reason`. The rules, in order:

1. **Exaltation sign** → `exalted`. The whole sign counts, so the Moon anywhere in Taurus is exalted (its
   mūlatrikoṇa range in Taurus is not applied in the vargas). Mercury in Virgo is the exception: BPHS divides the
   sign, 0–15° exalted, 15–20° mūlatrikoṇa, 20–30° own.
2. **Debilitation sign** → `debilitated`.
3. **Mūlatrikoṇa** → `moolatrikona`, when the degree inside the varga sign, `(longitude × D) mod 30`, falls in the
   BPHS range (Sun Leo 0–20°, Mars Aries 0–12°, Jupiter Sagittarius 0–10°, Venus Libra 0–15°, Saturn Aquarius
   0–20°). In D30, whose parts are unequal, there is no varga degree and the sign alone decides.
4. **Own sign** → `own`.
5. Otherwise the **compound relationship** to the sign lord, with the temporary part counted between the graha's
   and the lord's signs in that same varga (not in the birth chart).

Rāhu and Ketu: exalted in Taurus / Scorpio, debilitated in Scorpio / Taurus, mūlatrikoṇa in Gemini / Sagittarius,
own sign Virgo and Aquarius / Pisces; in any other sign they are listed as `neutral` (no compound relationship is
applied to a node's varga sign). `summary` counts each dignity per graha and adds `dignified` (exalted +
mūlatrikoṇa + own), `friendly` (great friend + friend), `favourable` and `unfavourable` (enemy + great enemy +
debilitated).

The viṃśopaka scores on the Strength page keep their own scoring convention (`vedic.bala`); this table is the
descriptive dignity grid.

## Agreement with the reference chart

[`accuracy/test_maitri_reference.py`](../../accuracy/test_maitri_reference.py) compares all three tables, cell by
cell, with a published reference chart for Sri Ramana Maharshi. All 72 compound relationships and all 90 tārās
agree, and 143 of the 144 dignity cells agree. The one difference is Jupiter in D40: the reference prints Jupiter
at Aquarius 16°29′10″, ESHANA's position is about 3′ later, just past the 16°30′ boundary of the 22nd khavedāṃśa,
so ESHANA puts it in Aquarius (neutral) where the reference has Capricorn (debilitated). That is a position
difference near a boundary, not a rule difference.

## In the studio

The **Maitri and dignities** page (group *Grahas & bhāvas*) draws three grids that animate in along the diagonal:

- **Compound relationships**: rows are the grahas, columns the grahas they regard, coloured from great friend to
  great enemy. Hovering a cell shows the natural and temporal parts and why (for example "Jupiter is 3rd from
  Sun: a temporary friend").
- **Nakshatra spatial matrix**: tārā names coloured auspicious, mixed or inauspicious; hovering shows the two
  nakshatras and the count.
- **Dignities in the sixteen divisions**: grahas by divisions, coloured by dignity, with a summary bar per graha
  (dignified, friendly, neutral, hostile). Hovering shows the varga sign, degree, lord and the reason.

Each grid has a legend; graha heads link to the graha's page. The note on tārās is marked as interpretation and
paraphrased from muhūrta practice.
