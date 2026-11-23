---
title: "Numerology"
description: "Pythagorean and Chaldean as separate schemas, with every choice and every ignored character visible."
---

# Numerology

```bash
eshana numerology --name "Ada Lovelace" --dob 1815-12-10 --year 2026
eshana numerology --name "Ada Lovelace" --school chaldean
```

Numerology is a symbolic tradition. These outputs are `tradition_rule`, not measurements, and say nothing about a person.

## Schools are separate

`numerology.pythagorean@0.1` and `numerology.chaldean@0.1` have different letter tables and reduction habits. There is no
blended number. A request names one; the result's `ruleset` says which.

For "Ada Lovelace" (Pythagorean): letters `ADALOVELACE`, expression 36 → 9, soul urge (vowels) 19 → 1, personality
(consonants) 17 → 8. Every letter and its value is in `letters`.

## Choices

| Option | Default | Effect |
|---|---|---|
| `--school` | `pythagorean` | letter table |
| `--no-master` | masters kept | keep 11, 22, 33 when reducing (Pythagorean only); `2000-04-05` is life path 11 with, 2 without |
| `--y-vowel` | consonant | how Y splits vowels from consonants |
| `--life-path-method` | `reduce_parts` | reduce month, day and year separately then sum; or `total_digits` |
| `--year` | none | adds the personal year for that calendar year |
| `--gender` | none | `male` or `female`; picks the person's own Kua number (anything else is refused with `ESH-RULES`) |

## Normalisation is visible

Input is normalised (NFKD, combining marks dropped, ß/æ/œ/ø/đ/ł expanded, uppercase A–Z). Anything else (digits,
Devanagari, emoji) is listed in `ignored_characters`. A name with no Latin letters is `ESH-RULES`: ESHANA will not
silently transliterate for you, because transliteration is itself a choice.

```text
$ eshana numerology --name 'José Ram राम'      → normalized "JOSERAM"; ignored_characters lists the Devanagari
$ eshana numerology --name 'राम'               → ESH-RULES
```

## Every number, in every school (`extended`)

Pass `extended=True` (Python) or `"extended": true` (request) to add four more attributes. Add `reference_date`
(`YYYY-MM-DD`) for the cycles. ESHANA Studio always asks for them.

| Attribute | Rule | Numbers |
|---|---|---|
| `numerology.pythagorean.extended` | `numerology.extended@0.1` | life path, expression, soul urge, personality, birthday, maturity, attitude, balance, rational thought; hidden passion, karmic lessons, subconscious self, cornerstone, capstone, first vowel; karmic debt (13, 14, 16, 19) met while reducing; inclusion table, planes of expression; pinnacles, challenges and life cycles with ages; personal year, month and day, universal year, month and day; bridge numbers |
| `numerology.chaldean.extended` | `numerology.extended@0.1` | name (compound and root), each word, psychic, destiny |
| `numerology.vedic` | `numerology.vedic@0.1` | mulank (driver) and bhagyank (conductor) with their planets, the name number (Chaldean values), and friendly, neutral and enemy numbers |
| `numerology.lo_shu` | `numerology.lo_shu@0.1` | the Lo Shu grid of the birth date: counts, missing and repeated numbers, and the eight arrows. A second grid adds the driver and conductor |

The conventions are stated in each result's `conventions` field:
- Pinnacles: the first ends at 36 minus the one-digit life path.
- Planes: the letter groups as commonly published.
- Number friendships: taken from the natural relationships of the ruling planets in BPHS, with Rāhu counted as
  Saturn and Ketu as Mars, rather than from an unsourced table.

What each number is *said to describe* is interpretation. It is not in the engine: the studio shows paraphrased
notes, labelled as such.

## Growth, intensity, letter transits and essence

`numerology.pythagorean.extended` also carries, for a name:

| Field | What it is |
|---|---|
| `growth` | the first name alone, reduced (also called the key number) |
| `intensity` | the digits that occur in the name more often than the average count per digit (letters ÷ 9) |
| `transits_by_age` | ages 0–99. Each year has a **physical** letter from the first name, a **mental** letter from the middle name(s) and a **spiritual** letter from the last name. Each letter rules for as many years as its value, and the name repeats |
| `transits_at_reference` | the same for the age on `reference_date` |
| `essence` | in each year, the sum of the three transit values, reduced |

For example, "Ada Lovelace" has no middle name, so it has no mental transit. Its physical letters are A (1 year),
D (4 years) and A (1 year), and then the cycle repeats. The studio shows growth, intensity and the current transits
and essence as tiles, with a year-by-year table. What each number is said to mean is labelled interpretation.

## Not here yet

Name-change cycles and other alphabets (Hebrew gematria, abjad). Each will be its own named rule pack, with a cited
source.

## Kua number (Eight Mansions)

`numerology.lo_shu` carries `kua`: the Kua (Gua) number of Eight Mansions feng shui for a man and for a woman. Both are always
listed; pass `gender` (`--gender`, `gender=` in Python) and `kua` also gains `gender`, `number` and `group` for that
person. The Chinese solar year starts at Lichun, taken as 4 February, so a birth on
2 February 1991 counts as 1990. Reduce the year's last two digits to one digit *r*; before 2000 the man's Kua is
10 − *r* and the woman's *r* + 5, from 2000 on 9 − *r* and *r* + 6 (reduced to 1–9). A Kua of 5 is read as 2 for a
man and 8 for a woman. Kua 1, 3, 4 and 9 form the east group, 2, 6, 7 and 8 the west group. Example: 15 May 1990
gives man 1 (east), woman 8 (west). The studio shows both in a card under the Lo Shu grid and highlights the person's own when a gender is given.

## Compound numbers (Chaldean)

When a Chaldean total between 10 and 52 is reduced, the studio's number tile shows `compound/root`; opening the tile
adds what the compound is said to signify, paraphrased in our own words from Cheiro's *Book of Numbers* (1926) and
labelled interpretation. Cheiro gives his own readings for 10–32, 37, 43 and 51; the other compounds share the
reading of a lower number, as he says.
