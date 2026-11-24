---
title: "Strength, aspects and special points"
description: "Ṣaḍbala, bhāva bala, viṃśopaka, sputa drishti in virūpas, bhāva chalit, declination and the special points (yogi, avayogi, dagdha, Indu lagna, 64th navāṃśa, 22nd dreshkāṇa), with the conventions each one uses."
---

# Strength, aspects and special points

These features add the tables a Parāśari practitioner expects next to the chart. Each one is a `tradition_rule`
derived from the positions, under the rule packs `vedic.bala@0.3` ([`bala.yaml`](../../crates/eshana-engine/rulesets/bala.yaml))
and `vedic.core@0.4` (with `vedic.dignity`, `vedic.strength` and `vedic.relationships`). Where schools differ, the rule pack names the convention it uses. The engine does not
interpret any of them.

```bash
eshana chart vedic --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090 \
  --features shadbala,bhava_bala,vimshopaka,aspects,special_points,bhava_chalit
```

In Python, pass `features=[...]` to `ChartEngine().vedic_chart(...)`. In [ESHANA Studio](html-report.md) they appear on the
**Strength**, **Aspects**, **Lagnas and points** and graha pages.

| Feature id | Attribute | What it holds |
|---|---|---|
| `shadbala` | `vedic.shadbala` | the six strengths for the seven grahas, each component kept, total in virūpas and rūpas, the required rūpas, the ratio |
| `bhava_bala` | `vedic.bhava_bala` | for each bhāva: the lord's ṣaḍbala, bhāva dig bala, drishti on the bhāva madhya, total |
| `vimshopaka` | `vedic.vimshopaka` | strength out of 20 in the ṣaḍvarga, saptavarga, daśavarga and ṣoḍaśavarga schemes |
| `aspects` | `vedic.aspects` | the graha × graha sputa drishti matrix in virūpas, for D1 and D9, with the graha drishti fraction and house |
| `special_points` | `vedic.special_points` | yogi, avayogi, duplicate yogi, dagdha rāśis, Indu lagna, 64th navāṃśa and 22nd dreshkāṇa from the Moon and lagna, lordships with functional nature, kartari, graha yuddha, the balance of elements |
| `bhava_chalit` | `vedic.bhava_chalit` | Sripati bhāva madhyas and sandhis, and the bhāva each graha falls in |

Every position now also carries `right_ascension_deg` and `declination_deg`, computed with the true obliquity of the
date. Two more house systems are available: `--houses sripati` (alias `bhava_chalit`) and `--houses porphyry`.

## Ṣaḍbala

| Component | Parts | Convention named in the pack |
|---|---|---|
| Sthāna | uchcha, saptavargaja (D1, D2, D3, D7, D9, D12, D30), oja-yugma, kendrādi, dreshkāṇa | compound relationships from the rāśi chart in every varga; kendras counted whole-sign |
| Dig | distance from the house of directional strength | |
| Kāla | natonnata, pakṣa, tribhāga, abda, māsa, vāra, horā, ayana, yuddha | abda and māsa lords are the weekday lords of the last Mesha and sign ingresses of the Sun |
| Ceṣṭā | from the motion state, against the mean daily motion | the Sun takes its ayana bala and the Moon its pakṣa bala |
| Naisargika | the fixed natural order | |
| Dṛk | (sputa drishti from benefics − from malefics) / 4 | the Moon counts by its pakṣa |

Required rūpas: Sun 6.5, Moon 6, Mars 5, Mercury 7, Jupiter 6.5, Venus 5.5, Saturn 5.

> [!NOTE]
> Against a published reference chart, positions, declinations, ayanāṃśa, special lagnas and the special points
> match. Ṣaḍbala totals agree to within about 60 virūpas per graha. Reconciling the remaining kāla and ceṣṭā
> differences is tracked as `VED-14`, and that test is marked expected-to-fail until then. See the
> [accuracy suite](accuracy-suite.md).

## Sputa drishti

Aspect strength in virūpas (0–60) by the angular distance from the aspecting graha. Mars, Jupiter and Saturn add their
special aspects: Mars +15 at 90–120° and 210–240°, Jupiter +30 at 120–150° and 240–270°, Saturn +45 at 60–90° and
270–300°, capped at 60. The same function feeds dṛk bala and bhāva bala.

## Viṃśopaka

The dignity in each varga scores own 20, great friend 18, friend 15, neutral 10, enemy 7, great enemy 5. These scores
are weighted by scheme, and each scheme's weights sum to 20 (checked when the pack loads):

| Scheme | Weights |
|---|---|
| Ṣaḍvarga | D1 6, D2 2, D3 4, D9 5, D12 2, D30 1 |
| Saptavarga | D1 5, D2 2, D3 3, D7 2.5, D9 4.5, D12 2, D30 1 |
| Daśavarga | D1 3, D60 5, the other eight 1.5 each |
| Ṣoḍaśavarga | D1 3.5, D9 3, D60 4, D16 2, D2/D3/D30 1, the rest 0.5 |

## Special points

| Point | Rule |
|---|---|
| Yogi point | Sun + Moon + 93°20′; its nakshatra lord is the yogi, the lord of its sign the duplicate yogi |
| Avayogi | the nakshatra lord of the point at yogi point + 186°40′ |
| Dagdha rāśis | by the tithi, from the table in `vedic-core.yaml` |
| Indu lagna | the kalās of the 9th lords from the lagna and the Moon, counted from the Moon |
| 64th navāṃśa, 22nd dreshkāṇa | counted from the Moon and from the lagna |
| Kartari | a sign hemmed in by malefics (pāpa) or benefics (śubha) on both sides |
| Graha yuddha | Mars to Saturn within 1° of longitude; the one further north in ecliptic latitude wins |
| Functional nature | from lordship for the lagna, as stated in `vedic-core.yaml` (`functional`) |

## Vargas D5, D6 and D8

D5 (pañcāṃśa), D6 (ṣaṣṭhāṃśa) and D8 (aṣṭāṃśa) join the existing vargas, for 19 in all. Every varga attribute now
also gives `degree_in_varga`. The accuracy suite re-derives all three, independently, for every reference chart.
