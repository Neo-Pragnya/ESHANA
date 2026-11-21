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

## Not here yet

Name-change cycles, pinnacles, karmic debts, other alphabets (Hebrew gematria, abjad). Each will be its own named
rule pack with a cited source.
