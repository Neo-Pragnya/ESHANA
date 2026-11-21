---
title: "Pañcāṅga"
description: "The five limbs with their end times, sunrise and moonrise under a stated convention, horā, choghaḍiyā, Rāhu kālam, special lagnas, Tārā and Candra bala."
---

# Pañcāṅga

```bash
eshana panchanga --date 1990-05-15 --time 14:30 --tz Asia/Kolkata --lat 28.6139 --lon 77.2090
eshana panchanga --utc 2026-11-21T06:00:00Z --lat 28.61 --lon 77.21 --natal-moon 271.89   # with Tārā bala
```

```python
ChartEngine().panchanga(ctx, ayanamsha="lahiri", natal_moon_deg=271.89)["panchanga"]
```

| Field | Content |
|---|---|
| `tithi`, `nakshatra`, `yoga`, `karana` | index, name (pakṣa for the tithi, pada and lord for the nakshatra) and when it **ends**, found by search |
| `vara` | weekday of the Vedic day, which starts at local sunrise |
| `sunrise`, `sunset`, `next_sunrise`, `moonrise`, `moonset` | UTC and local times; `null` when there is no event that day (polar day and night) |
| `rahu_kalam`, `yamagandam`, `gulika_kalam` | the eighth of the daytime each weekday assigns |
| `horas` | 24 horās (12 day, 12 night) with lords, the one containing the instant marked |
| `choghadiya` | 16 choghaḍiyās with quality, the current one marked |
| `special_lagnas` | Bhāva, Horā and Ghaṭī lagnas |
| `tara_bala`, `chandra_bala` | with `natal_moon_deg`: the tārā (Janma … Ati-Mitra) and the Moon's house from the natal Moon |

## Conventions you choose

Sunrise has more than one definition, so it is a stated choice that is recorded in the result
(`sunrise_convention`):

| `--sunrise` | Definition | Typical use |
|---|---|---|
| `upper_limb_refracted` (default) | upper limb on the horizon, standard refraction (−0°50′) | almanacs and most calendars |
| `centre_refracted` | centre of the disc with refraction (−0°34′) | some pañcāṅgas |
| `centre_unrefracted` | centre of the disc on the geometric horizon | a common Jyotiṣa-software convention |

Special lagnas count from the Sun's longitude **at birth** by default (`special_lagna_sun: sunrise` uses the Sun at
sunrise). With `centre_unrefracted` and the Sun at birth, they agree with PyJHora's book-checked values within 2′.

## Accuracy

Against the Swiss Ephemeris (upper limb, 40 charts in 16 cities), rise and set agree within 3.5 s for the Sun and 11 s
for the Moon after allowing for latitude. The limbs are checked against the Swiss Sun and Moon. See the
[accuracy suite](accuracy-suite.md).
