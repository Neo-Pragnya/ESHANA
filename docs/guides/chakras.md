---
title: "Chakras: Sarvatobhadra, Kota and Saṅghaṭṭa"
description: "How ESHANA lays out the Sarvatobhadra chakra with its vedha lines, the Kota (fort) chakra and the Saṅghaṭṭa ring, what every value means, the conventions chosen, and what the studio page shows."
---

# Chakras: Sarvatobhadra, Kota and Saṅghaṭṭa

Three classical nakshatra chakras, computed by the engine with the `chakras` feature (or `all`) and drawn on the
**Chakras** page of [ESHANA Studio](html-report.md). Each chakra is one attribute that holds its whole layout (every
cell, place, line and placement), so any reader can draw it without repeating the rules:

| Attribute | What it holds |
|---|---|
| `vedic.chakra.sarvatobhadra` | the 9 × 9 grid, natal and transit placements, every graha’s three vedha lines, and the vedha received by the personal cells |
| `vedic.chakra.kota` | the 28 nakshatras in the four zones of the fort from the Janma nakshatra, Kota swāmī and pāla, who is inside, entering and leaving |
| `vedic.chakra.sanghatta` | the ring of 28 nakshatras, the vedha pairs and which nakshatras each graha pierces |

All three carry the rule id `vedic.chakras@0.1` and are `tradition_rule` values: they apply a named tradition with
stated conventions. They do not say what anything means; the studio labels the few sentences that do as
interpretation. The tables (names in Devanagari with transliteration, the grid, the pairs) live in
`crates/eshana-engine/rulesets/chakras.yaml`, whose header names its sources and every convention.

For example, `eshana chart vedic` with `--features chakras` and `--reference-utc 2026-11-21T12:57:29Z` returns the
three attributes with a transit layer at that instant.

## The 28 nakshatras

The chakras use the 27 equal nakshatras plus **Abhijit**, which takes the last pada of Uttara Āṣāḍhā and the first
fifteenth of Śravaṇa: 276°40′ to 280°53′20″ sidereal. A body in that span sits in Abhijit; every other nakshatra keeps
its usual span, so a graha is in the same nakshatra here as in the rest of the chart unless it is in Abhijit.

## Natal and transit layers

Every chakra places the natal grahas (the nine grahas, plus the lagna where it applies). When the request has a
reference instant (`reference_utc`, the report’s “now”), the engine also computes the transit positions of the nine
grahas at that instant (the node model follows the chart: true or mean) and adds a transit layer: placements,
vedha lines and Kota movement. If the ephemeris cannot reach the reference instant, the chakras keep the natal layer
and carry a `note` saying why.

**Benefic and malefic.** Jupiter, Venus and Mercury are counted benefic, and the Moon too in the bright half (śukla
pakṣa); the Sun, Mars, Saturn, Rāhu, Ketu and the waning Moon are counted malefic. Mercury is counted benefic without
checking its company (a simplification).

## Sarvatobhadra chakra

The grid is 9 × 9 with **east at the top**, south on the right, west at the bottom and north on the left:

- **Border:** the 28 nakshatras, seven to a side, Kṛttikā to Āśleṣā along the top (east), Maghā to Viśākhā down the
  right (south), Anurādhā to Śravaṇa along the bottom from right to left (west, with Abhijit between Uttara Āṣāḍhā
  and Śravaṇa) and Dhaniṣṭhā to Bharaṇī up the left (north).
- **Corners of the four rings:** the sixteen vowels (svara) अ आ इ ई, उ ऊ ऋ ॠ, ऌ ॡ ए ऐ, ओ औ अं अः, clockwise from the
  north-east (top-left) corner of each ring.
- **Second ring:** twenty letters (varṇa), five to a side (अ व क ह ड, म ट प र त, न य भ ज ख, ग स द च ल).
- **Third ring:** the twelve rāśis, three to a side (Vṛṣabha, Mithuna and Karka at the top, Meṣa just below the
  top-left corner of the ring).
- **Centre:** the five tithi groups with their weekdays: Nandā (tithis 1, 6, 11; Sunday and Tuesday) east of the
  centre, Bhadrā (2, 7, 12; Monday and Wednesday) south, Jayā (3, 8, 13; Thursday) west, Riktā (4, 9, 14; Friday)
  north and Pūrṇā (5, 10, 15 and the new Moon; Saturday) in the centre.

Each cell in `cells` has `row`, `col`, `kind` (`nakshatra`, `vowel`, `letter`, `rashi` or `tithi`), `name`, `dev`
(Devanagari) and `tr` (transliteration); the grahas there (`natal`, `transit`: grahas by nakshatra in nakshatra cells
and by sign in rāśi cells); `pierced_by` (every vedha line crossing it: `set`, `graha`, `line`, `nature`,
`emphasised`); and in the tithi cells `tithis`, `weekdays`, `birth_tithi` and `birth_weekday`.

### Vedha lines

From its nakshatra cell every graha pierces three lines across the grid: **front** (sammukha, straight across to the
far side), **left** (vāma) and **right** (dakṣiṇa) diagonals. Left and right are the hands of an observer standing in
the nakshatra cell and facing the centre. Every cell a line crosses (nakshatras, letters, vowels, rāśis, tithi
groups) receives that graha’s vedha. The classical check passes: a graha in Kṛttikā pierces Śravaṇa (front), Bharaṇī
and Viśākhā (the diagonals).

Each graha in `grahas.natal` and `grahas.transit` lists its `nakshatra`, `rashi`, `cell`, `rashi_cell`, the cells of
each line (`lines.front`, `lines.left`, `lines.right`), its `motion` and the line it **emphasises**:

| Motion | When | Emphasised line |
|---|---|---|
| vakrī | retrograde (Rāhu and Ketu always) | right (dakṣiṇa) |
| śīghra | faster than its mean daily motion | left (vāma) |
| sama | otherwise; the Sun and Moon always | front (sammukha) |

This is the classical motion rule (a retrograde graha looks the other way). All three lines are always computed; the
emphasis is reported, not used to drop lines.

### Vedha received

`vedha_received` lists, for each personal cell, every natal and transit graha piercing it, by line, with counts of
malefics and benefics: the **Janma nakshatra** (the Moon’s), the **lagna nakshatra**, the **Moon rāśi** and **lagna
rāśi**, the **birth tithi group** and the **birth weekday** (from the pañcāṅga, so it follows the sunrise convention).
The first letter of a name is not known to the engine; every letter cell carries its own `pierced_by`, and the studio
shows it on hover.

## Kota chakra

The fort (koṭa) has four concentric zones: **stambha** (the pillar at the core), **madhya** (middle), **prākāra**
(the wall) and **bāhya** (outside). The 28 nakshatras are counted from the **Janma nakshatra** (place 1), seven per
direction: east (top), south (right), west (bottom), north (left). In each direction the first three places enter
from the corner (bāhya, prākāra, madhya), the fourth is the stambha next to the centre, and the last three leave
along the axis (madhya, prākāra, bāhya) to the middle of the side. So the stambha holds 4 places and the other zones
8 each. `layout` gives every place with `position`, `name`, `dev`, `tr`, `zone`, `direction`, `place` (`entering`,
`core`, `leaving`) and drawing coordinates `x`, `y` in a square from −1 to 1 (`zone_size` gives each zone’s
half-size).

For each graha, `natal` and `transit` give its place, zone and direction, and `path`: a direct graha in an entering
place is **entering** the fort and one in a leaving place is **leaving**; a retrograde graha (and Rāhu and Ketu,
always) moves the other way. `natal_summary` and `transit_summary` list malefics and benefics inside (stambha or
madhya), entering and leaving, and who is in the stambha.

**Kota swāmī** is the lord of the Moon’s sign; **Kota pāla** is the lord of the Moon’s navāṃśa (the pada it occupies).
Both are given with their natal and transit places. Other schools tabulate the Kota pāla differently; this is the
convention named in the value.

## Saṅghaṭṭa chakra

The 28 nakshatras form a ring, Aśvinī first, counterclockwise from the top (`ring[i].angle_deg` is the centre of each
place, counterclockwise from the top). The **vedha pairs** are the sapta-śalākā pairs of the muhūrta texts: the two
nakshatras at the ends of one of the seven vertical or seven horizontal lines pierce each other (Kṛttikā and Śravaṇa,
Rohiṇī and Abhijit, Mṛgaśirā and Uttara Āṣāḍhā, Ārdrā and Pūrva Āṣāḍhā, Punarvasu and Mūla, Puṣya and Jyeṣṭhā, Āśleṣā
and Anurādhā, Maghā and Bharaṇī, Pūrva Phalgunī and Aśvinī, Uttara Phalgunī and Revatī, Hasta and Uttara Bhādrapadā,
Citrā and Pūrva Bhādrapadā, Svātī and Śatabhiṣā, Viśākhā and Dhaniṣṭhā). A graha pierces the partner of the
nakshatra it occupies (`natal[].pierces`, `transit[].pierces`); `received` lists every nakshatra pierced and by whom;
`janma_index` and `lagna_index` mark the Moon’s and the lagna’s nakshatras. The pairs are data in the rule pack, so
another school’s figure is a table change.

## What the studio shows

The **Chakras** page (group *Analysis*) has a tab per chakra and, when the report has a reference instant, a
Natal / Transit layer switch.

- **Sarvatobhadra:** the grid draws in line by line, cells fade in and grahas pop into their cells (solid markers
  natal, rings transit). Hover any cell for what it is, who is there and every graha piercing it. *Vedha of* picks
  one graha: its three lines light up (the emphasised one thicker) and every cell it pierces is tinted. Tables list
  the vedha received by the personal cells and each graha’s motion and emphasised line.
- **Kota:** the four walls draw in and the 28 places pop in along their entering and leaving paths, numbered from the
  Janma nakshatra in the top-left corner. Tiles show the Janma nakshatra, Kota swāmī and pāla with their places, and
  the count of malefics and benefics inside; a table lists every graha’s place, zone and movement.
- **Saṅghaṭṭa:** the ring draws in with the fourteen vedha lines; picking a graha lights the line from its nakshatra
  and tints the nakshatra it pierces.

Each tab ends with a short *How to read* note. The sentences that say what something means are labelled
interpretation and paraphrase the tradition (Narapati-jayacaryā for the Sarvatobhadra and Kota, the muhūrta texts for
the sapta-śalākā vedha).

## Comparison with a published reference chart

The layouts were checked against a published reference chart (the Sri Ramana Maharshi chart, 30 December 1879) in
`accuracy/test_chakras_reference.py`. The Sarvatobhadra layout, its natal and transit placements, the Kota layout
(Janma nakshatra Punarvasu in the top-left corner, all 28 places), the transit placements in the Kota and the natal
placements in the Saṅghaṭṭa ring all match. Known differences, recorded as expected failures with their reasons:

- The reference prints a Kota **score** (−3.9) whose formula it does not publish. ESHANA lists who is inside,
  entering and leaving instead of inventing a score.
- In the reference Saṅghaṭṭa ring, Saturn (347°01′) is drawn on the Uttara Bhādrapadā / Revatī boundary and reads as
  Uttara Bhādrapadā; ESHANA places by longitude, 0°21′ inside Revatī.
- The reference Saṅghaṭṭa ring is drawn with 27 equal sectors and its own inner figure; ESHANA draws 28 places with
  Abhijit and the sapta-śalākā pairs.
