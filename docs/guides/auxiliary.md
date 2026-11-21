---
title: "Auxiliary analysis"
description: "Sahams, the avakahaḍā chakra, male and female rāśis, mṛtyu bhāga, the ghāta chakra, vedhasthāna, Kālachakra jīva and deha, and aṣṭakavarga śodhana with piṇḍas: what each value is, how ESHANA computes it, and which convention it follows."
---

# Auxiliary analysis

The `auxiliary` feature (included in `features: ["all"]`) adds eight attributes, one per topic. Each is a
`tradition_rule` value with rule id `vedic.aux@0.1`. The tables live in
[`rulesets/auxiliary.yaml`](../../crates/eshana-engine/rulesets/auxiliary.yaml), and the source and convention of
each one are written beside it. The engine code is in
[`auxiliary.rs`](../../crates/eshana-engine/src/auxiliary.rs).

```bash
eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
  --node true --sunrise centre_unrefracted --features auxiliary --reference-utc 2026-11-21T12:00:00Z --compact
```

| Attribute | What it holds |
|---|---|
| `vedic.aux.sahams` | 37 Tajika sahams: longitude, sign, lord, the formula used, and whether the night reversal or the +30° rule applied |
| `vedic.aux.avakahada` | The avakahaḍā (birth-name) chakra |
| `vedic.aux.male_female` | The lagna and the nine grahas, each in a male (odd) or female (even) sign |
| `vedic.aux.mrityu_bhaga` | The mṛtyu bhāga degree of each point's sign, and the point's distance from it |
| `vedic.aux.ghata_chakra` | The ghāta (ghātaka) chakra row for the natal Moon sign |
| `vedic.aux.vedhasthana` | Gochara vedha from the natal Moon, for the birth grahas and for the transits at the reference instant |
| `vedic.aux.jiva_deha` | The Kālachakra jīva and deha signs of the Moon's pada, and its paramāyus |
| `vedic.aux.shodhana` | Trikoṇa and ekādhipatya śodhana of each aṣṭakavarga, with rāśi, graha and śodhya piṇḍa |

## Sahams

A saham is a sensitive point computed as **A − B + C**. Under the Tajika "+30" rule, 30° are added when C does
not lie on the zodiacal arc that runs from B forward to A. For many sahams the tradition swaps A and B for a
night birth. ESHANA decides day or night from the birth pañcāṅga: the birth is by day when it falls between that
day's sunrise and sunset, under the sunrise convention of the request. Without a place, it falls back to the Sun's
position above or below the horizon in ecliptic houses. `birth_in` and `day_night_rule` record which rule was used.

The points used in the formulas:

- the seven grahas and the lagna;
- Punya and Vidya, computed first;
- the 2nd, 6th, 8th, 9th and 11th bhāva cusps. These are Śrīpati bhāva madhyas, a Porphyry trisection of the
  sidereal quadrants from the Ascendant and Midheaven. Without a Midheaven, they are equal houses from the lagna
  degree;
- the lord of the sign that holds a cusp;
- the lagna lord;
- Karka 15° (105°);
- for Kāryasiddhi, the Sun by day or the Moon by night, plus the lord of that luminary's sign.

Samartha uses Jupiter − Mars + lagna when Mars itself is the lagna lord.

**Convention and verification.** The list follows Tajika Nīlakaṇṭhī, but the texts and the programs that follow
them differ on several formulas. The pack writes out every formula. The published reference chart is a night
birth, and with that program's own printed longitudes the engine's unit test reproduces all 37 of its sahams within
one arc-minute. Some formulas differ from other common tables, and they are kept because the reference confirms
them:

- Mṛtyu is 8th cusp − Moon + Saturn.
- Santāpa is Saturn − Moon + 6th cusp, with no reversal.
- Roga is Saturn − Moon + lagna, reversed at night.
- Bandhu, Vaṇik, Pāradāra, Śraddhā and Prīti are not reversed at night.
- The cusps are bhāva madhyas, not whole-sign or equal houses.

Three sahams, Āśā, Vyāpāra and Prasava, have no form in the reference set ESHANA follows that matches. For these,
the night form is the one the reference prints, and the day form is inferred: Āśā = Saturn − Venus + lagna,
reversed at night; Vyāpāra = Mars − Mercury + lagna, not reversed; Prasava = Jupiter − Mercury + lagna, reversed at
night. Each row carries `day_form_inferred: true`, and the studio marks it.

Each row's `meaning` names what the saham signifies in the Tajika tradition. That is interpretation, paraphrased,
and the studio labels it as such.

## Avakahaḍā chakra

This is the almanac table used for naming and matching. Everything comes from the Moon except the ascendant and
the Western sun sign.

- **Ascendant / lord, sign / lord**: the rising sign and the Moon sign, each with its lord.
- **Nakshatra / pada**: the Moon's nakshatra and quarter.
- **Yoga / karaṇa**: the birth pañcāṅga's yoga and karaṇa, with the karaṇa's traditional animal (Bava lion, Bālava
  tiger, Kaulava pig, Taitila donkey, …).
- **Yoni, gaṇa, nāḍī, varṇa, vaśya**: the same tables as aṣṭakūṭa matching
  ([`matching.yaml`](../../crates/eshana-engine/rulesets/matching.yaml)), so the two features always agree. Nāḍī is
  printed as Adya, Madhya or Antya.
- **Varga**: from the first sound of the name syllable. Vowels give Garuḍa; the ka row Mārjāra, the ca row Siṃha,
  the ṭa row Śvāna, the ta row Sarpa, the pa row Mūṣaka; ya, ra, la, va give Mṛga; śa, ṣa, sa, ha give Meṣa.
- **Yuñjā**: the part of the nakshatra circle. Pūrva is Revatī to Mṛgaśirā, Madhya is Ārdrā to Jyeṣṭhā, and Para
  is the rest.
- **Haṃsaka**: the tattva of the Moon sign (Agni, Bhūmi, Vāyu, Jala).
- **Nāmākṣara**: the syllable of the Moon's pada, from the common Hindi-almanac list of 108. A published reference
  chart prints it with a trailing "h" ("Hah"); ESHANA prints "Ha".
- **Pāyā**: by sign, from the Moon's house counted from the lagna: 1, 6, 11 gold; 2, 5, 9 silver; 3, 7, 10 copper;
  4, 8, 12 iron. By nakshatra: gold for Revatī, Aśvinī and Bharaṇī; silver for Ārdrā to Anurādhā; copper for
  Jyeṣṭhā to Uttara Bhādrapadā; iron for Kṛttikā, Rohiṇī and Mṛgaśirā. Sources disagree on the nakshatra grouping;
  this is the common almanac one.
- **Sun sign (Western)**: the tropical Sun, which is the sidereal longitude plus the ayanāṃśa and the nutation.

## Male and female rāśis

The lagna and each of the nine grahas sit in either an odd sign (Meṣa, Mithuna, Siṃha, Tulā, Dhanu, Kumbha), which
is male, or an even sign, which is female. This is computed in the rāśi chart (D1).

## Mṛtyu bhāga

Each sign has a "death portion" degree for each graha, for the nodes and for the lagna. ESHANA uses the classical
table as reprinted with Phaladīpikā and Jātaka Pārijāta; the lagna, Rahu and Ketu columns follow the same
reprints. For every point the attribute gives:

- the mṛtyu bhāga degree of its sign;
- the distance, |degree in sign − mṛtyu bhāga|;
- a `within_1_deg` flag, set when the point is within 1° of it.

## Ghāta chakra

The ghātaka chakra lists, for each natal Moon sign, the times held inauspicious for that native: a lunar month,
three tithis counted within the pakṣa, a weekday, a nakshatra, a yoga, a karaṇa, a prahara (one of the four watches
of the day) and a transit Moon sign. The table is the one printed in muhūrta almanacs following Muhūrta Chintāmaṇi.
Regional almanacs differ in a few cells, and only the Mithuna row is checked against a published reference.
`birth_coincides_with` lists which of the birth's own tithi, weekday, nakshatra, yoga and karaṇa fall on the ghāta
values.

## Vedhasthāna

Gochara (transit) results are read from the natal Moon. A graha in one of its favourable houses gives good results
unless another graha stands in the matching vedha (obstruction) house. The table follows Phaladīpikā ch. 26. The
Sun and Saturn never obstruct each other, and neither do the Moon and Mercury. For each graha the attribute gives:

- the house from the Moon;
- whether that house is favourable;
- the vedha house;
- the grahas that obstruct it;
- a one-line summary such as `Me 6 Ausp, X by Ju in 9`.

The attribute holds two sets:

- `natal` places the birth grahas. A published reference chart prints vedhasthāna this way: its houses match the
  natal positions, not the transits at its print date.
- `transit` places the grahas at the request's `reference_utc` and is computed with the chart's ephemeris. It is
  empty when no reference instant is given.

## Jīva and deha

In the Kālachakra daśā (BPHS ch. 46) each nakshatra pada owns a sequence of nine signs. The nakshatras fall into
four groups, two savya and two apasavya, and each group has four pada sequences. The first sign of the sequence is
the **deha** (body) and the last is the **jīva** (life). The sum of the nine signs' daśā years is the pada's
**paramāyus**. The years per sign are Meṣa 7, Vṛṣabha 16, Mithuna 9, Karka 21, Siṃha 5, Kanyā 9, Tulā 16, Vṛścika 7,
Dhanu 10, Makara 4, Kumbha 4 and Mīna 10, and every pada's span is 100, 85, 83 or 86 years.

For the Ramana chart (Punarvasu pada 3) the sequence runs Vṛṣabha, Meṣa, Mīna, Kumbha, Makara, Dhanu, Meṣa, Vṛṣabha,
Mithuna: deha Vṛṣabha, jīva Mithuna, 83 years. This matches the reference ("Lifespan 83 yrs"). Paramāyus is a
structural number of the daśā system, not a prediction.

## Śodhana and piṇḍa

The reduction of the aṣṭakavargas follows BPHS ch. 67–69. ESHANA reduces the bhinnāṣṭakavarga of each of the seven
grahas (from `vedic.ashtakavarga`) and the lagna's aṣṭakavarga (BPHS ch. 66, 49 bindus).

1. **Trikoṇa śodhana**: in each triplet of trine signs, subtract the smallest count. When all three are equal,
   they become zero.
2. **Ekādhipatya śodhana**: applies to the five pairs of signs that share a lord. Karka and Siṃha are never
   reduced. A sign counts as occupied when one of the seven grahas or the lagna is in it.
   - If both signs are occupied, or either is zero, nothing changes.
   - If both are empty and equal, both become zero; if both are empty and unequal, both take the smaller count.
   - If one is occupied, the empty one becomes zero when its count is no greater than the occupied one's; otherwise
     it is reduced to the occupied one's count.
3. **Piṇḍas**:
   - rāśi piṇḍa = Σ (reduced count × rāśi multiplier). The multipliers are Meṣa 7, Vṛṣabha 10, Mithuna 8, Karka 4,
     Siṃha 10, Kanyā 6, Tulā 7, Vṛścika 8, Dhanu 9, Makara 5, Kumbha 11 and Mīna 12;
   - graha piṇḍa = Σ (reduced count in each graha's sign × graha multiplier). The multipliers are Sun 5, Moon 5,
     Mars 8, Mercury 5, Jupiter 10, Venus 7 and Saturn 5;
   - śodhya piṇḍa = rāśi piṇḍa + graha piṇḍa.

**Convention.** Counting the lagna as an occupant, and Kanyā's multiplier of 6, are what reproduce the published
reference: all eight rāśi and graha piṇḍas match exactly. Without the lagna, that chart's Kanyā counts would be
removed. The rule for an empty sign holding more bindus than its occupied partner varies between texts, and the
reference does not exercise it; the pack states the choice made.

## What the studio shows

The **Auxiliary analysis** page (group Analysis) has eight tabs, and every value has a hover with its rule:

- **Sahams**: a small South Indian square with each saham written into its sign, and a table of all 37 with formula,
  degree and lord. Hovering a row lights its sign; hovering a sign lists its sahams. Rows mark the night reversal
  and the inferred day forms.
- **Avakahaḍā**: eleven tiles.
- **Male/female**: a strip of the twelve signs, and the points grouped as male and female.
- **Mṛtyu bhāga**: each point's degree (dot) against its sign's mṛtyu bhāga (diamond). The bar between them draws
  in, and points within 1° are red.
- **Ghāta**: eight tiles; the ones the birth itself falls on are outlined.
- **Vedhasthāna**: the twelve houses from the Moon. Hovering a graha lights its favourable houses in green, its
  vedha house in red, and the house where it stands. You can switch between the birth grahas and the transits at the
  reference instant.
- **Jīva/deha**: the nine signs of the pada in order, with deha and jīva highlighted and a running total of years.
- **Śodhana**: the piṇḍa table with bars. Clicking a row shows that aṣṭakavarga reduced step by step, with the
  changed cells marked.

## Accuracy

[`accuracy/test_auxiliary_reference.py`](../../accuracy/test_auxiliary_reference.py) compares every item with a
published reference chart for Sri Ramana Maharishi. All values match. Sahams are compared within 6′ because they
add and subtract up to three longitudes, and ESHANA's 1879 Jupiter and Saturn differ from that program's by about 3′
and 2′. The largest difference, Śāstra, is 5′. With the reference's printed longitudes the unit test agrees within
1′. Mṛtyu bhāga distances are compared within 4′ for the same reason.
