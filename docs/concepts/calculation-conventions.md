---
title: "Calculation conventions"
description: "Every convention ESHANA applies, in one place, so a number can be reproduced by hand."
---

# Calculation conventions

If a result surprises you, the answer is in this page or in the result's `provenance.assumptions`.

## Frame and units

- **Frame**: geocentric, apparent, ecliptic longitude and latitude of the true equinox of date. Distances are in AU (the
  Moon's too). Angles are degrees; longitudes are in `[0, 360)`.
- **Speed** is the symmetric difference of longitude over ±0.01 day, in degrees per day. **Retrograde** means speed < 0.
- **Nodes** lie on the ecliptic: latitude 0, no distance. The **south node** is `node + 180°`, reported as its own
  position derived from the node you chose.

## Zodiac

- **Tropical**: measured from the equinox of date; this is the apparent longitude.
- **Sidereal**: `sidereal = apparent − Δψ − ayanāṃśa`, where Δψ is the nutation in longitude. That is, subtract the
  nutation to reach the *mean* equinox of date, then the *mean* ayanāṃśa. This is the Swiss Ephemeris convention; the
  tests compare Sun and Moon to it for every supported ayanāṃśa.
- **Ayanāṃśa models**: `lahiri`, `raman`, `krishnamurti`, `fagan_bradley`. Each is anchored at its J2000.0 value and
  carried by IAU 2006 general precession (5028.796195″ per century plus a quadratic term). Lahiri's anchor,
  23.857092°, follows from the Calendar Reform Committee value of 23°15′00.658″ at 1956-03-21. Measured deviation from
  the Swiss Ephemeris over 1950–2040 is under 0.0006° (2″).
- **True (star-anchored) ayanāṃśas**: `true_citra` (Spica at 180°), `true_revati` (ζ Psc at 359°50′), `true_pushya`
  (δ Cnc at 106°) and `true_mula` (λ Sco at 240°). The star's Hipparcos position and proper motion (new reduction,
  J1991.25) are carried to the date, precessed to the mean equinox of date and given annual aberration and the Sun's
  light deflection, so the apparent star sits exactly on its anchor; nutation is left out on both sides, as for the
  mean models. Within 0.62″ of the Swiss Ephemeris true modes at 160 instants over 1800–2200.
- **Sign boundaries** are half-open: `[0°, 30°)` is Aries/Meṣa; exactly 30° is Taurus/Vṛṣabha.

## Time

See [Time](time.md). In short: UTC is treated as UT1; 1972–2017 uses the leap-second table; later instants assume
TAI−UTC = 37 s until 2027 and a forecast after; earlier instants use ΔT polynomials.

## Houses

- **Whole sign**: house 1 is the sign of the Ascendant; cusps are at 0° of each sign.
- **Equal**: cusp 1 is the Ascendant; each cusp 30° later.
- **Placidus**: semi-arc, solved by fixed-point iteration to 1e-11°. Refused (`ESH-LATITUDE`) at or beyond latitude
  `90° − ε` (about 66.56°) and wherever the iteration has no solution. It is never extrapolated.
- **Ascendant and Midheaven** use the local *apparent* sidereal time (GMST plus the equation of the equinoxes) and the
  true obliquity. In a sidereal chart the cusps are shifted by Δψ + ayanāṃśa like the planets.
- Ascendant and houses require an exact time. A window wider than 30 minutes omits them with a warning.

## Vedic rule pack `vedic.parashari@0.3`

- **Nakshatra**: 27 divisions of 13°20′, each with four 3°20′ padas; the lord follows the Vimśottarī order.
- **Bhāva**: whole sign from the lagna.
- **Divisional charts** (Parāśara conventions): D1; D3 `s + 4k`; D4 `s + 3k`; D7 odd signs from the sign, even signs
  from the 7th; D9 `floor(λ·9/30) mod 12`; D10 odd from the sign, even from the 9th; D12 from the sign. D30 and D60 are
  **not implemented**: school choices (and, for D30, unequal parts) must be decided and cited first, so asking for them
  returns `ESH-RULES` rather than a guess.
- **Vimśottarī daśā**: 120-year cycle in the order Ketu 7, Venus 20, Sun 6, Moon 10, Mars 7, Rahu 18, Jupiter 16,
  Saturn 19, Mercury 17. The first period's balance is `(1 − elapsed fraction of the Moon's nakshatra) × lord's years`.
  Antardaśās divide each mahādaśā in proportion to the lords' years. The **year length is a named option**:
  `365.25` (default), `365.2425`, `360` (savana), `365.25636` (sidereal). The dates change with it; the result records
  which was used.

## Western rule pack `western.basic@0.1`

- Ptolemaic aspects with orbs: conjunction 8°, sextile 4°, square 6°, trine 6°, opposition 8° (scalable with
  `orb_scale`). Orbs are a tradition-level choice, not a measurement.
- **Applying or separating** from the angular rate of the pair: applying when the separation is moving toward the exact
  angle. Nodes are excluded from aspects by default.

## Human Design rule pack `human_design.basic@0.1`

- **Design instant**: the UT moment when the Sun's apparent longitude equals its birth longitude minus **88°**, found by
  bisection to 1e-10°. It is 86–92 days earlier, not 88: the Sun's speed varies through the year.
- **Wheel**: gate 41 begins at 2°00′00″ Aquarius (302.0°); 64 gates of 5.625°; 6 lines of 0.9375°; 6 colours per line.
  Earth is Sun + 180°, the south node is node + 180°.
- **Structure**: 36 channels, 9 centres (the tables are cross-checked in tests: the gates partition into the centres,
  every gate is in a channel).
- **Type**, **authority**, **profile** and **definition** are derived from the defined channels. These are the standard
  published rules, labelled `tradition_rule`.

## Numerology

- **Pythagorean**: A=1 … I=9, repeating. **Chaldean**: values 1–8 (never 9): A1 B2 C3 D4 E5 F8 G3 H5 I1 J1 K2 L3 M4
  N5 O7 P8 Q1 R2 S3 T4 U6 V6 W6 X5 Y1 Z7.
- **Normalisation**: NFKD, combining marks dropped, ß/æ/œ/ø/đ/ł expanded, uppercase A–Z only. Anything else is listed in
  `ignored_characters`; a name with no Latin letters is an error.
- **Reduction**: digit sum until one digit; with `keep_master_numbers` (Pythagorean only) 11, 22, 33 stop the
  reduction. **Y** is a consonant unless `y_policy=vowel`.
- **Life path**: `reduce_parts` (reduce month, day and year separately, then sum) or `total_digits`. They can differ.
