---
title: "Aspects (drishti)"
description: "Every kind of aspect ESHANA computes and how: graha drishti with special aspects, rāśi drishti, sputa drishti in virūpas, aspects on houses and cusps, the outer planets, and the optional additional aspects from a retrograde graha's shadow and from parivartana."
---

# Aspects (drishti)

ESHANA computes aspects as rules, from positions. Each result names its rule pack (`vedic.core` or `vedic.bala`)
and the convention it uses. Nothing here says what an aspect means.

```bash
eshana chart vedic --date 1879-12-30 --time 01:00 --offset-minutes 321 --lat 9.9333 --lon 78.1167 \
  --features drishti,aspects
```

| Attribute | What | Rule |
|---|---|---|
| `vedic.drishti.graha` | graha drishti: which signs each graha aspects, by whole-sign house count | `vedic.core` |
| `vedic.drishti.rashi` | rāśi (Jaimini) drishti: which signs each sign aspects, with the grahas in them | `vedic.core` |
| `vedic.drishti.additional` | optional extra aspects: retrograde shadow and parivartana (below) | `vedic.core` |
| `vedic.aspects` | sputa drishti in virūpas: graha × graha (D1 and D9), on houses, on house + lord | `vedic.bala` |

## Graha drishti

Every graha aspects the **7th** sign from itself. Some add **special aspects**, counted from their own sign:

| Graha | Special aspects |
|---|---|
| Mars | 4th and 8th |
| Jupiter | 5th and 9th |
| Saturn | 3rd and 10th |
| Rāhu and Ketu | 5th and 9th (a common convention; some schools use the 7th only) |
| Uranus, Neptune and Pluto | 5th and 9th: a modern convention, by analogy with the nodes, since classical texts do not use the outer planets |

Each entry gives the house counted from the graha (`house_from_graha`), the target sign, the bhāva it falls in from
the lagna, the grahas there, and whether it is a full (7th) or special aspect. The partial aspects (¼ on the 3rd and
10th, ½ on the 5th and 9th, ¾ on the 4th and 8th) appear as `fraction` in `vedic.aspects`.

## Rāśi drishti

Signs aspect signs:
- movable signs aspect the fixed signs, except the one next to them;
- fixed signs aspect the movable signs, except the one next to them;
- dual signs aspect the other dual signs.

A graha aspects by rāśi drishti every sign its own sign aspects. Since 0.3.1 this includes Uranus, Neptune and Pluto
when the chart has them.

## Sputa drishti (virūpas)

The strength of an aspect, 0 to 60 virūpas, from the exact angular distance (Bṛhat Parāśara Horā Śāstra). The special
aspects add Mars +15 at 90–120° and 210–240°, Jupiter +30 at 120–150° and 240–270°, and Saturn +45 at 60–90° and
270–300°, capped at 60. The nodes and the outer planets have no sputa drishti.

`vedic.aspects` holds:
- the graha × graha matrix for D1, and for D9 using navāṃśa degrees;
- `houses`: aspects on the lagna and on each house madhya, for D1 (Śrīpati madhyas) and D9 (equal houses from the
  navāṃśa lagna degree);
- `house_and_lord`: the mean of the virūpas on a house madhya and on that house's sign lord.

## Additional aspects (optional)

Two conventions used by some practitioners. They are computed in `vedic.drishti.additional` and shown in the studio
only when **additional aspects** is switched on. It is off by default.

### Retrograde shadow (ʳ)

A retrograde graha is read as also present in the **sign before** the one it occupies, its shadow, and casts its
aspects from there too:
- its graha drishti (the 7th and its special aspects);
- its rāśi drishti.

Rāhu and Ketu are excluded, since they always move backwards.

*Example.* Saturn retrograde in Pisces also acts from Aquarius. From Aquarius its 3rd, 7th and 10th fall on Aries,
Leo and Scorpio, and its rāśi drishti falls on the movable signs other than Capricorn: Aries, Cancer and Libra.

### Parivartana (⇄)

Two grahas each in the other's sign (a sign exchange, parivartana yoga) are read as **exchanged**. Each casts its
aspects from the other's sign, which is its own.

*Example.* In Sri Ramana Maharishi's chart, Jupiter is in Aquarius (Saturn's sign) and Saturn is in Pisces (Jupiter's
sign). Read as exchanged, Jupiter casts its 5th, 7th and 9th from Pisces (Cancer, Virgo, Scorpio) and Saturn its 3rd,
7th and 10th from Aquarius (Aries, Leo, Scorpio).

### In the data

Each entry has:
- `basis`: `retrograde_shadow` or `parivartana`;
- `partner`: the exchanged graha, for parivartana;
- `from_sign`: where the aspect is cast from;
- `drishti`: `graha` or `rashi`;
- `house_from_graha`, `kind`, the target `sign`, `aspects_bhava`, and `aspects_grahas`. The targets are the grahas'
  real positions; only the caster is moved.

```json
{"from": "jupiter", "basis": "parivartana", "partner": "saturn", "from_sign": 11, "drishti": "graha",
 "house_from_graha": 5, "sign": 3, "kind": "special", "aspects_grahas": [], "aspects_bhava": 11}
```

## In ESHANA Studio

| Where | Normal aspects | Additional aspects (when switched on) |
|---|---|---|
| Jyotish Chakra and charts | translucent marks `Ju Sa*` (graha drishti, `*` special), `◇Ju` (rāśi drishti) | `Saʳ` (retrograde shadow) and `Ju⇄` (parivartana), dotted-underlined |
| Hover a graha | flowing curves to every sign it aspects | dotted curves for its additional aspects |
| Graha legend | the houses it aspects | `+ ʳ` or `⇄` with the target signs |
| Aspects page | the matrix, aspects on houses and cusps (D1, D9) | a table: who casts, why, from where, onto what |
| Graha page | aspects cast and received | the additional aspects it casts |

Switch them on in the **Layers** of the Jyotish Chakra, **Show** on the Charts page, or **Additional** on the
Aspects page. One switch covers all of them. The **Include** switches also hide the outer planets and the upagrahas
everywhere.
