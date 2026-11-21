---
title: "Nadi direction matrix and AstroEnergetics"
description: "How ESHANA Studio shows the zodiac as a spiral through the four directions, places each graha by degree, links grahas the Nadi way, and lays out the AstroEnergetics reading of the twelve houses."
---

# Nadi direction matrix and AstroEnergetics

Two pages in [ESHANA Studio](html-report.md) arrange the same chart in other ways. Both are reading aids. The links
and groupings are labelled rules, and ESHANA does not say what they mean.

## The direction matrix

The twelve signs form a square spiral that passes through the four directions three times. Fire signs sit in the
East, earth in the South, air in the West and water in the North. The first round (Aries, Taurus, Gemini, Cancer) is
the innermost square, the second round is the middle one and the third (Sagittarius to Pisces) the outer one, so each
direction collects its three signs of one element:

| Direction | Element | Trikoṇa | Houses |
|---|---|---|---|
| East | Fire | Dharma | 1 · 5 · 9 |
| South | Earth | Artha | 2 · 6 · 10 |
| West | Air | Kāma | 3 · 7 · 11 |
| North | Water | Moksha | 4 · 8 · 12 |

Each graha (and Uranus, Neptune and Pluto when present) sits in its sign with its **degree in the sign**. The table
beside the matrix lists, for each direction, the grahas and their degrees.

## Links (`report.nadi@0.1`)

Click a graha to see the grahas it links with, drawn as flowing lines. The rule, computed in the page and labelled
`report.nadi@0.1`, links a graha with others:

- in the same sign (conjunct);
- in trine, that is, the same element and direction (5th and 9th);
- in the next sign (2nd) and the previous sign (12th);
- in the opposite sign (7th).

A retrograde graha also acts from the previous sign. Rāhu and Ketu are excluded from this, since they are always
retrograde.

The rule is a stated convention for reading, not an engine result. Like the chart markers, it is one of the few
values the page computes itself.

## AstroEnergetics

The **Energetics** page lays out a reading framework for the twelve houses. The data comes from
`eshana/data/astro_energetics.json`, a reference table (`tradition_rule`) that holds structure, not quotations:

- for each house: its names, kārakas, significations, the region of the body, the dimension of life and the
  question it answers;
- the four **quadrants** (self, family, social, universal), each a group of three houses;
- the four **trikoṇas** (dharma, artha, kāma, moksha);
- **energy bands**: each graha as hot (gives and expands), warm (keeps energy flowing), cold (restrains), or
  Rāhu/Ketu (amplifies or reduces attachment), with a nominal temperature;
- chakras and elements.

The chart's grahas are placed on this layout by their house, so you can see which quadrant and band each area of life
holds. Each bhāva page carries the same row.
