---
title: "Annual and daily: varṣaphala, pañca pakṣī and KP significators"
description: "The Tājika annual chart (solar return, muntha, office-bearers, year lord, pañcavargīya bala), the Tamil five-bird day table, and the KP significator tables: what each value is, how it is computed, the conventions chosen and how they compare with a published reference chart."
---

# Annual and daily: varṣaphala, pañca pakṣī and KP significators

The `annual` feature adds two attributes to a Vedic chart, `vedic.varshaphala` and `vedic.panch_pakshi`, and the
`kp` feature now carries a `significator_table` inside `vedic.kp`. The feature list `all` includes both. All three
need an exact birth time and a place. Every value is a `tradition_rule`: computed by a named rule, with the tables in
[`rulesets/annual.yaml`](../../crates/eshana-engine/rulesets/annual.yaml) (rule pack `vedic.annual@0.1`). Nothing
here is interpretation; the studio labels the few lines of reading text it shows as interpretation.

```bash
eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
  --node true --sunrise centre_unrefracted --features annual,kp --reference-utc 2026-11-21T00:00:00Z --compact
```

## Varṣaphala (`vedic.varshaphala`, rule `vedic.varshaphala@0.1`)

The Tājika annual chart is cast for the moment the Sun returns to its natal sidereal longitude. ESHANA gives the
**running year** at the reference instant (`current`, present when `reference_utc` is given) and the **birth year**
(`birth_year`, year 0, which is the birth chart itself).

| Field | What it is | How it is computed |
|---|---|---|
| `completed_years` | n, the number of solar returns since birth | the last return at or before the reference instant |
| `return` | the solar-return instant (UTC, local, JD) | Newton search on the Sun’s sidereal longitude with the chart’s ephemeris and ayanāṃśa, to about 1e-8 degree |
| `day_return`, `sunrise`, `sunset`, `weekday` | whether the return falls between sunrise and sunset, and its Vedic weekday | the pañcāṅga of the return instant, with the chart’s sunrise convention |
| `varsha_lagna` | the Ascendant at the return, at the birth place | the same sidereal reduction as the chart’s own Ascendant |
| `positions`, `houses_from_varsha_lagna` | the varṣa chart: the nine grahas at the return and their whole-sign bhāvas | the node model follows the natal chart |
| `muntha` | the natal lagna moved on one sign per completed year, keeping its degree | natal lagna + 30° × n |
| `office_bearers` | the five pañcādhikārīs, each with its pañcavargīya total, whether it aspects the varṣa lagna and its varṣa bhāva | see below |
| `year_lord` | the varṣeśa | the strongest office-bearer that aspects the varṣa lagna; if none does, the strongest |
| `pancavargiya_bala` | the five-fold Tājika strength of the seven grahas in that year’s chart | see below |

**Office-bearers.** Muntha pati (lord of the muntha’s sign), janma lagna pati (lord of the natal lagna), varṣa lagna
pati (lord of the varṣa lagna), trirāśi pati (from the Tājika table by varṣa lagna sign, day or night: Aries Sun or
Jupiter, Taurus Venus or Moon, Gemini Saturn or Mercury, Cancer Venus or Mars, Leo Jupiter or Sun, Virgo Moon or Venus,
Libra Mercury or Saturn, Scorpio Mars or Venus, Sagittarius Saturn, Capricorn Mars, Aquarius Jupiter, Pisces Moon) and
dinarātri pati (for a day return the lord of the Sun’s sign, for a night return the lord of the Moon’s sign; the
weekday lord is also given as `weekday_lord` for those who use it instead).

**Aspect to the varṣa lagna.** A Tājika aspect exists from every sign except the 2nd, 6th, 8th and 12th counted from the
graha to the lagna.

**Pañcavargīya bala.** Five parts, each a maximum multiplied by the graha’s Tājika relationship to the lord of the
division it occupies (own 1, friend 3/4, neutral 1/2, enemy 1/4):

- gṛha (sign), maximum 30; hudda (the Egyptian terms), maximum 15; drekkāṇa (Chaldean decans: the first decan of Aries
  Mars, then Sun, Venus, Mercury, Moon, Saturn, Jupiter in turn), maximum 10; navāṃśa (D9), maximum 5;
- uchcha, maximum 20, in proportion to the graha’s distance from its debilitation point (20 at exaltation, 0 at
  debilitation);
- the relationship is the Tājika one, judged in the chart itself: counting signs from the graha to the division lord,
  the 3rd, 5th, 9th and 11th are friendly, the 1st, 4th, 7th and 10th inimical, the rest neutral;
- the total is the sum divided by four (maximum 20).

These choices reproduce every cell of the published reference chart’s “Varshaphala strengths” table (see below).

## Pañca pakṣī (`vedic.panch_pakshi`, rule `vedic.panch_pakshi@0.1`)

The Tamil five-bird system (Vulture, Owl, Crow, Cock, Peacock). Source: the traditional pañca pakṣī śāstra tables as
commonly reprinted in Tamil almanacs. Printed tables differ between almanacs; every choice is named in the rule pack.

| Field | What it is | Convention |
|---|---|---|
| `birth_bird` | the native’s bird | the Moon’s nakṣatra in groups 1–5, 6–11, 12–16, 17–21, 22–27 (Vulture, Owl, Crow, Cock, Peacock), the order reversed in the dark half |
| `paksha`, `nakshatra` | the pakṣa and nakṣatra at birth | |
| `friends`, `enemies` | the birth bird’s friends and enemies | fixed table |
| `birth_day` | the ruling (day and night) and dying birds of the birth’s Vedic weekday | the reprinted weekday table by pakṣa |
| `birth_day_table` | the Vedic day of the birth (sunrise to the next sunrise) | ten yāmas |
| `reference_day_table` | the Vedic day containing the reference instant, with the current yāma marked | ten yāmas |
| `nearby_days` | the three Vedic days before and the three after the reference day, as `{offset, table}` (offset −3 … +3; 0 is `reference_day_table`) | each found one hour after the reference sunrise plus whole days, so each has its own sunrise, sunset, pakṣa and weekday |

Each table has ten **yāmas**, five equal parts of the day (sunrise to sunset) and five of the night (sunset to the next
sunrise). For every yāma it gives every bird’s activity (ruling, eating, walking, sleeping or dying) and the birth
bird’s five **sub-periods**. Conventions:

- the activities follow a fixed cycle per half: bright half by day eating, walking, ruling, sleeping, dying; by night
  eating, ruling, dying, walking, sleeping; dark half by day eating, dying, sleeping, ruling, walking; by night eating,
  sleeping, walking, dying, ruling. Each bird moves one step along the cycle per yāma;
- the day’s (or night’s) ruling bird rules the first yāma, and the other birds, in the half’s bird order, take the next
  activities;
- a yāma divides in the parts eating 30, walking 36, ruling 48, sleeping 18 and dying 12 of 144, in cycle order from
  the yāma’s activity, the sub-periods going to the birds in the half’s order starting with the birth bird.

## KP significator table (`vedic.kp`, key `significator_table`)

Built from the existing KP value (Placidus cusps, Vimśottarī sub-divisions), so nothing is computed twice.

- `houses`: per bhāva the cusp and its sign, star, sub and sub-sub lords, and the four significator levels, strongest
  first: grahas in the stars of the occupants, the occupants, grahas in the star of the owner, the owner;
- `grahas`: per graha its sign, star, sub and sub-sub lords and the houses it signifies at four strengths: very strong
  (the house its star lord occupies), strong (the house it occupies), normal (houses owned by its star lord), weak
  (houses it owns);
- `significators_by_house`: the grahas signifying each bhāva at any level, strongest first;
- `equal_cusps_from_ascendant`: the lords of equal cusps 30° apart from the Ascendant, the view some programs print.

Rahu and Ketu also act as agents of their sign lord and of grahas joined with them; that agency is not added.

## What the studio shows

- **Varṣaphala** (group Time): the return instant, completed years, varṣa lagna, muntha and the year lord; the varṣa
  chart in South Indian style with the muntha (Mu) and the year lord highlighted; the office-bearers with hover
  explanations; the pañcavargīya bala as stacked bars that grow in, each part hoverable with its division lord and
  relationship; a switch between the running year and the birth year.
- **Pañca pakṣī** (group Time):
  - your bird (with how it was found: the Moon’s nakṣatra and the pakṣa at birth), its friends and enemies, and the
    chosen day’s ruling birds (by day and by night) and dying bird, with a warning when your own bird is the dying
    bird;
  - a **day picker**: the birth day, and the reference day (“Today”) with the three days either side, plus previous
    and next buttons. Every day comes from the engine (`nearby_days`); the page computes no yāma itself;
  - the day and night as coloured bands (one per yāma, coloured by your bird’s activity, the sub-periods underneath,
    a “now” line at the reference instant);
  - the full tables, day and night: five yāmas (rows) × the five birds (columns), each cell the bird’s activity,
    with the yāma start and end times. Your bird’s column is outlined; the current yāma is outlined with a “now” bar
    showing how far into it the reference instant is. Click a yāma (or press Enter on it) to open its five
    sub-periods below it; hovering a cell gives the activity, its Tamil name, its strength rank and the relation to
    your bird;
  - **info boxes** (labelled interpretation, with sources): what the system is, how the birth bird is found, what
    friends and enemies mean, the ruling bird, the dying bird, the five activities, and the strength order (Ruling,
    Eating, Walking, Sleeping, Dying, strongest first, with a ladder).

  The info-box text is paraphrased in our own words from the Tamil pañca pakṣī śāstra (traditionally attributed to
  the Siddhas) and Tamil almanac practice. It lives in `scripts/build_reading_reference.py` (written to
  `eshana/data/reading_reference.json`, key `pakshi`), not in the engine, which stays free of interpretation.
- **KP**: two new tables, significators by bhāva and what each graha signifies, plus the equal-cusp lords.

## Comparison with a published reference chart

[`accuracy/test_annual_reference.py`](../../accuracy/test_annual_reference.py) compares the Sri Ramana Maharshi chart
with the “Auxiliary Analysis” page of a published reference chart.

| Item | Result |
|---|---|
| “Varshaphala strengths” (birth year), 35 cells | all match to 0.01 |
| Muntha Libra 28:43 and the five office-bearers (Ve, Me, Ju, Sa, Ju) | match, for the year after 145 completed returns |
| The number printed beside each office-bearer, and the year lord (Jupiter) | differ: the program does not say which measure it prints; it is not the pañcavargīya total of the varṣa chart (by ESHANA’s rule Saturn is the strongest office-bearer aspecting the varṣa lagna). xfail |
| Which year is running on 21 Nov 2026 | differs: 146 returns were complete on 31 Dec 2025; the program shows the year after 145. xfail |
| Birth bird Cock, dark half, the birth day’s ruling bird Peacock and dying bird Owl | match (the Vedic weekday, Monday, of a birth at 01:00 on Tuesday) |
| Yāma start times 06:27:47 to 17:53:28 and the bird order inside a yāma | match within 3 seconds |
| The Cock’s activities through the day (ruling, walking, eating, dying, sleeping) | differ: printed weekday tables disagree; ESHANA lets the day’s ruling bird rule the first yāma. xfail |
| KP star and sub lords of the nine grahas | all match |
| KP cusp lords | match with equal cusps from the Ascendant, which that program prints; ESHANA’s own cusps are Placidus, the KP convention. xfail on Placidus |
