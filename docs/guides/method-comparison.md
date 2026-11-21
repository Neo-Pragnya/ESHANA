---
title: "Method comparison"
description: "Run the same input under two configurations and see exactly which placements change and why."
---

# Method comparison

Disagreements between tools are usually configuration differences. `compare` makes them visible.

```bash
eshana compare vedic --date 1992-05-28 --time 20:05 --tz Asia/Kolkata --lat 17.385 --lon 78.487 \
        --a ayanamsha=lahiri --b ayanamsha=fagan_bradley
eshana compare western --utc 2024-02-29T18:45:30Z --a node_model=mean --b node_model=true
eshana compare western --utc 2000-01-01T12:00:00Z --a zodiac=tropical --b zodiac=sidereal --b ayanamsha=lahiri
```

The result lists:

- `configuration_differences`: which of `provider`, `zodiac`, `ayanamsha`, `house_system`, `node_model` differ;
- `changes`: every position whose longitude or sign changed (with `difference_deg`, `sign_changed`), every derived
  attribute whose value changed, and anything present on only one side (the other node, for instance);
- `summary`: counts of changes and sign changes;
- the full `calculation` and `provenance` of both sides, and a note that the differences are consequences of the
  configuration, **not software defects**.

Fagan–Bradley minus Lahiri is a constant shift of about −0.88° in 1992; mean vs true node differs by up to 1.7°; tropical vs
sidereal shifts everything by the ayanāṃśa (about 24°) and changes the sign of most bodies.
