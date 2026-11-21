---
title: "Compatibility (Aṣṭakūṭa)"
description: "Eight-fold compatibility from the two Moons, out of 36 points, with Manglik flags, from a named YAML rule pack."
---

# Compatibility (Aṣṭakūṭa)

```bash
eshana matching --a 1990-05-15,14:30,Asia/Kolkata,28.61,77.21 --b 1992-08-20,09:15,Asia/Kolkata,19.08,72.88
eshana-report … --partner 1992-08-20,09:15,Asia/Kolkata,19.08,72.88 -o pair.html
```

```python
ChartEngine().matching(ctx_a, ctx_b, ayanamsha="lahiri")
```

The classical tables give the two partners different roles. ESHANA calls them `a` (the role the tables give the groom)
and `b` (the bride). Swapping them can change varna, vashya and gana.

| Kūṭa | Max | Rule (pack `vedic.matching.ashtakoota@0.2`) |
|---|---:|---|
| Varna | 1 | 1 when a's varna rank (by Moon sign element) is at least b's |
| Vashya | 2 | group table (chatuṣpada, mānava, jalacara, vanacara, kīṭa; Sagittarius and Capricorn split at 15°) |
| Tārā | 3 | 1.5 per direction when the count of nakshatras, mod 9, is not 3, 5 or 7 |
| Yoni | 4 | 14 × 14 animal matrix (symmetric) |
| Graha maitrī | 5 | the two Moon-sign lords' natural relationship |
| Gaṇa | 6 | deva, manuṣya, rākṣasa table |
| Bhakūṭ | 7 | 0 for 2/12, 5/9 and 6/8 sign distances |
| Nāḍī | 8 | 0 when both share a nāḍī |

The result also carries both charts and the Manglik doshas of each (from the lagna, Moon and Venus; cancellations are
not applied). Vashya and gaṇa tables differ between regions. The pack writes its tables out, so another school is
another pack. The accuracy suite recomputes the total from the YAML for 20 pairs.
