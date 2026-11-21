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

## Not here yet

Name-change cycles, other alphabets (Hebrew gematria, abjad), Kua numbers and compound-number readings. Each will
be its own named rule pack, with a cited source.
