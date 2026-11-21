---
title: "Daśā systems"
description: "Every daśā system ESHANA computes (Vimśottarī, Yoginī, Aṣṭottarī, Ṣoḍaśottarī, Chara, Nārāyaṇa, Navāṃśa, Śūla, Kālachakra): what each value is, how it is computed, the conventions chosen, and how the studio shows them."
---

# Daśā systems

A daśā system divides a life into periods ruled by a graha or a sign. ESHANA computes nine systems. Vimśottarī and
Yoginī come with `dasha: true` (see [vedic.md](vedic.md)); the seven systems on this page come with the feature
`dashas_extended` (included in `features: ["all"]`). Every value is a **tradition rule**: it follows a named,
published method, and each attribute carries a `conventions` object that says which one.

```bash
eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
  --node true --features dashas_extended --reference-utc 2026-11-21T00:00:00Z --compact
```

## What every attribute holds

Each system is its own attribute, `vedic.dasha.<system>`, with rule id `vedic.dashas_extended@0.1`:

| Field | Meaning |
|---|---|
| `mahadashas` | the main periods in order, from birth (the first is clipped at birth), at least 150 years and 80 years past the reference instant |
| `mahadashas[].antardashas` | the sub-periods of each main period |
| `label`, `lord` or `sign`/`rashi` | the graha (nakṣatra-based systems) or the sign (sign-based systems, 0 = Meṣa) |
| `start_utc`, `end_utc`, `years` | when the period runs (UTC) and its length in daśā years |
| `current` | the chain running at `reference_utc`: mahādaśā, antardaśā and, for Aṣṭottarī and Ṣoḍaśottarī, pratyantardaśā |
| `conventions` | the method, in words, for every choice below |
| `year_days` | the length of the daśā year: 365.2425 days (the mean Gregorian year) for all seven systems |

The tables (lords, years, nakṣatra groups, Kālachakra rows) are data in
[`rulesets/dashas-extended.yaml`](../../crates/eshana-engine/rulesets/dashas-extended.yaml).

## Aṣṭottarī (108 years)

Eight lords: Sun 6, Moon 15, Mars 8, Mercury 17, Saturn 10, Jupiter 19, Rāhu 12, Venus 21 years. The nakṣatras are
counted from Ārdrā in the 28-nakṣatra scheme (with Abhijit, from 276°40′ to 280°53′20″): the Sun rules four, the Moon
three, Mars four, Mercury three, Saturn four (with Abhijit), Jupiter three, Rāhu four, Venus three. The first lord is
the one whose group holds the Moon; the part of the group the Moon has already crossed, times the lord's years, is
already spent at birth. Each antardaśā takes the mahādaśā years times its own lord's years out of 108, in order from
the mahādaśā lord.

`applicability`: BPHS uses Aṣṭottarī when Rāhu is in a kendra or trikoṇa from the lagna lord, not in the lagna
(`applies`). A later rule (birth in Kṛṣṇa pakṣa by day or Śukla pakṣa by night) is reported as
`paksha_day_night_rule`. ESHANA computes the system for every chart and only reports whether the condition holds.

## Ṣoḍaśottarī (116 years)

Eight lords: Sun 11, Mars 12, Jupiter 13, Saturn 14, Ketu 15, Moon 16, Mercury 17, Venus 18. Count the janma nakṣatra
from Puṣya (Puṣya = 1); the count, mod 8, gives the first lord. The balance is the part of the janma nakṣatra the Moon
has not yet crossed, times that lord's years. The antardaśās follow the Aṣṭottarī rule, out of 116.
`applicability.applies` is the BPHS condition: the lagna in the Sun's horā in Kṛṣṇa pakṣa, or in the Moon's horā in
Śukla pakṣa (odd signs: Sun's horā first; even signs: Moon's horā first).

## Sign-based systems: shared rules

**Strength of two signs** (Jaimini). The first rule that separates the two signs decides, and the result says which
rule did (`start_from.decided_by_rule`):

1. more grahas in the sign;
2. more of Jupiter, Mercury and the sign's lord joining it or aspecting it by rāśi dṛṣṭi;
3. more exalted grahas in it;
4. its lord in a sign of the other oddity (odd sign, lord in an even sign, or the reverse);
5. dual over fixed over movable;
6. its lord further advanced in its sign.

**Co-lords.** Vṛścika has Mars and Ketu, Kumbha has Saturn and Rāhu. If one lord is in the sign, the other is used;
otherwise the lord with more grahas beside it, then an exalted lord, then the one further advanced in its sign.

**Years of a sign.** Count the signs from the sign to its lord, less one; 12 when the lord is in the sign; one more
for an exalted lord and one less for a debilitated lord (grahas only). Each system says which way it counts.

**Order of the signs.** Consecutive signs from the first sign, forward when the 9th sign from the first sign is
savya (Meṣa, Vṛṣabha, Mithuna, Tulā, Vṛścika, Dhanu) and backward otherwise.

**Antardaśās.** Twelve equal parts of the mahādaśā; each system says where they start and which way they run.

## Chara daśā (K. N. Rao)

- First sign: the lagna.
- Years: counted forward for savya signs and backward for the others; the co-lord rule above.
- Cycles: the second and later cycles repeat the first cycle's years. Some schools run the second cycle as 12 minus
  the first; the published reference chart repeats them, and ESHANA follows it.
- Antardaśās: from the sign next to the mahādaśā sign, forward or backward by the 9th sign from the mahādaśā sign,
  ending with the mahādaśā sign itself.
- `years_by_sign` lists every sign's years, lord and counting direction.

## Nārāyaṇa daśā

- First sign: the stronger of the lagna and the 7th (strength rules above; `start_from` shows the decision).
- Years: counted forward for odd signs and backward for even signs.
- Cycles: the second cycle runs 12 minus the first cycle's years, the third repeats the first (two cycles make 144
  years).
- Antardaśās: from the stronger of the mahādaśā sign and its 7th, forward for odd and backward for even mahādaśā
  signs.

## Navāṃśa daśā

- First sign: the navāṃśa (D9) lagna.
- Years: from the rāśi chart, counted forward from the sign to its graha lord (Mars for Vṛścika, Saturn for Kumbha).
- Cycles and antardaśās as for Nārāyaṇa.
- These choices are the ones that reproduce the published reference chart (see below); they are named in the
  attribute's `conventions`.

## Śūla daśā

- First sign: the stronger of the lagna and the 7th.
- Years: 9 for every sign.
- Antardaśās: from the mahādaśā sign itself, forward for odd and backward for even signs.

## Kālachakra daśā

- The Moon's nakṣatra pada picks a row of nine signs from the BPHS tables: savya rows for the Aśvinī type (also
  Kṛttikā type) and the Bharaṇī type, apasavya rows for the Rohiṇī type (also Ārdrā type) and the Mṛgaśirā type.
- Sign years: Meṣa 7, Vṛṣabha 16, Mithuna 9, Karka 21, Siṃha 5, Kanyā 9, Tulā 16, Vṛścika 7, Dhanu 10, Makara 4,
  Kumbha 4, Mīna 10. A row's total is its paramāyus (83, 85, 86 or 100 years).
- `deha` and `jiva`: in a savya row the first sign is the deha and the ninth the jīva; in an apasavya row, the
  reverse.
- Balance: the part of the janma pada the Moon has crossed, times the paramāyus, is already spent at birth.
- After the janma pada's nine daśās come the rows of the following padas in order (next pada, then the next
  nakṣatra). Each mahādaśā reports its row (`nakshatra`, `pada`, `chakra`, `paramayus`) and
  `gati_from_previous`: regular, maṇḍūka (a jump of two signs), siṃhāvalokana (a jump to a trine) or markaṭa (any
  other turn).
- Antardaśās: the nine signs of the same row from the mahādaśā sign, each taking the mahādaśā years times its own
  years out of the paramāyus.

## Compared with a published reference chart

[`accuracy/test_dashas_reference.py`](../../accuracy/test_dashas_reference.py) compares the Ramana chart (30 Dec
1879, 01:00 LMT, Madurai) with a published reference program's period lists, read from its printed page.

| System | Result |
|---|---|
| Chara, Nārāyaṇa, Navāṃśa, Śūla | every listed period start matches to the day (local dates), and the running periods match |
| Aṣṭottarī, Ṣoḍaśottarī | every listed start within one day; the reference prints the Moon 13″ further on, which moves these dates up to about a day earlier |
| Kālachakra | known difference (below) |

**Kālachakra difference.** The rows and sign years agree: the reference runs Mithuna, Vṛṣabha, Meṣa, Mīna and then a
row with Kumbha as deha and Kanyā as jīva, which is the end of ESHANA's Puṣya pada-1 row and the start of its pada-2
row. The timing does not: the BPHS sequence (janma pada Punarvasu 3 with its balance, then Punarvasu 4, then Puṣya 1)
starts the Puṣya pada-1 row in 2007, while the reference starts it about 44 years earlier. The reference also divides
each mahādaśā into 12 equal sign antardaśās, where BPHS uses the nine signs of the row in proportion. The convention
behind the reference's timing has not been identified, so ESHANA keeps the BPHS method and the test records the
difference as an expected failure.

## In the studio

The **Daśā** page has a tab for each system the chart has (`#/dasha/<system>`). Every system uses the same zoom:
one row per level, each opening out of the period chosen above it, with a bracket showing what is zoomed, a dashed
line at the reference instant and a hover card with dates, length, the chain and the source of the value.

- Graha-based systems (Vimśottarī, Yoginī, Aṣṭottarī, Ṣoḍaśottarī) colour periods by graha and link to the graha's
  page. Vimśottarī opens five levels; Aṣṭottarī and Ṣoḍaśottarī open three (the third subdivided in the page by the
  same proportions).
- Sign-based systems (Chara, Nārāyaṇa, Navāṃśa, Śūla, Kālachakra) colour periods by element, label them with the
  rāśi glyph and name, and link each sign to the bhāva it is from the lagna.
- Below the zoom: the chain running now, the facts about the system (first lord or sign, balance, direction,
  applicability, deha and jīva) and the conventions with the years of every sign.
