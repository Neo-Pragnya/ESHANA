---
title: "CLI reference"
description: "Every eshana command and flag. Output is always one JSON envelope; errors exit 2."
---

# CLI reference

```text
eshana [--compact] <command> [options]
```

`--compact` prints one line of JSON (the bytes the engine returned). Without it the JSON is indented. Every command
sends a JSON request through the engine's single entry point; `eshana request` lets you send one yourself.

## Moment and place (shared)

| Flag | Meaning |
|---|---|
| `--utc 2000-01-01T12:00:00Z` | a UTC instant, or an RFC 3339 string with an offset |
| `--date YYYY-MM-DD` `--time HH:MM[:SS]` `--tz Zone` | local civil time in an IANA zone. Omit `--time` for an unknown birth time |
| `--offset-minutes N` | fixed UTC offset east, when no zone is known (instead of `--tz`) |
| `--fold reject\|earliest\|latest` | what to do with a repeated local time (default `reject`) |
| `--uncertainty-minutes N` | birth-time uncertainty, ± minutes |
| `--lat DEG --lon DEG` | place; needed for houses and the Ascendant |

## Settings (shared)

| Flag | Values | Default |
|---|---|---|
| `--provider` | `analytic`, `spk` | `analytic` |
| `--spk PATH` | JPL SPK kernel (or `ESHANA_SPK`) | none |
| `--zodiac` | `tropical`, `sidereal` | tropical (sidereal for `chart vedic`) |
| `--ayanamsha` | `lahiri`, `raman`, `krishnamurti`, `fagan_bradley` | **required** with sidereal; only `chart vedic` defaults to `lahiri` and records it |
| `--houses` | `whole_sign`, `equal`, `placidus` | `whole_sign` (recorded) |
| `--node` | `mean`, `true` | `mean` (recorded); `true` for `human-design` |
| `--bodies a,b,c` | body ids | ten planets + the node |
| `--divisions 1,9,10` | 1, 3, 4, 7, 9, 10, 12 | none |
| `--dasha`, `--year-convention` | `365.25`, `365.2425`, `360`, `365.25636` | `365.25` |
| `--aspects`, `--orb-scale F` | | off, 1.0 |

## Commands

| Command | Purpose |
|---|---|
| `eshana positions` | apparent geocentric positions only |
| `eshana chart vedic\|western` | a chart with houses and derived attributes |
| `eshana human-design` | design instant, activations, BodyGraph |
| `eshana numerology --name N --dob D [--school S] [--no-master] [--y-vowel] [--life-path-method M] [--year Y]` | name and date numbers |
| `eshana compare vedic\|western ... --a KEY=VAL --b KEY=VAL` | the same input under two configurations (keys: `provider`, `zodiac`, `ayanamsha`, `house_system`, `node_model`) |
| `eshana graph vedic\|western ... --format json\|mermaid\|graphml` | a typed symbolic graph; mermaid and graphml print raw text |
| `eshana request FILE\|-` | send a raw JSON request |
| `eshana doctor` | versions, tz database, providers, rule packs, error codes |
| `eshana explain ESH-CODE` | what a code means |
| `eshana schema` | the JSON Schema of the envelope |

## Exit status

`0` success. `2` any refusal or error: stdout carries an error envelope
(`{"kind": "error", "error": {"code": "ESH-…", "message": "…"}}`) and stderr one line `CODE: message`.
