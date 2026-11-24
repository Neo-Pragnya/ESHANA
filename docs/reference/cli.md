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
| `--date YYYY-MM-DD` `--time HH:MM[:SS]` `--tz Zone` | local civil time in an IANA zone. Omit `--time` for an unknown birth time. A signed year (`-0043-03-15`) is BCE |
| `--calendar gregorian\|julian\|auto` | how `--date` is written (default proleptic Gregorian; `auto` switches at 1582-10-15) |
| `--offset-minutes N` | fixed UTC offset east, when no zone is known (instead of `--tz`) |
| `--fold reject\|earliest\|latest` | what to do with a repeated local time (default `reject`) |
| `--uncertainty-minutes N` | birth-time uncertainty, ± minutes |
| `--lat DEG --lon DEG` | place; needed for houses and the Ascendant |
| `--altitude-m M` | height above sea level, for `--topocentric` |

## Settings (shared)

| Flag | Values | Default |
|---|---|---|
| `--provider` | `analytic`, `spk` | `analytic` |
| `--spk PATH\|NAME` | JPL SPK kernel (or `ESHANA_SPK`); a catalogue name such as `de440` is looked up in the kernel folder | none |
| `--topocentric` | positions as seen from the place (parallax) | geocentric |
| `--zodiac` | `tropical`, `sidereal` | tropical (sidereal for `chart vedic`) |
| `--ayanamsha` | `lahiri`, `raman`, `krishnamurti`, `fagan_bradley`, `yukteshwar`, `jn_bhasin`, `ushashashi`, `sassanian`, `deluce`, `djwhal_khul` | **required** with sidereal; only `chart vedic` defaults to `lahiri` and records it |
| `--houses` | `whole_sign`, `equal`, `placidus`, `porphyry`, `sripati`, `koch`, `regiomontanus`, `campanus`, `alcabitius`, `topocentric`, `morinus`, `meridian` | `whole_sign` (recorded) |
| `--node` | `mean`, `true` | `mean` (recorded); `true` for `human-design` |
| `--bodies a,b,c` | body ids | ten planets + the node |
| `--divisions 1,9,10` | 1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60 | none |
| `--dasha`, `--year-convention` | `365.25`, `365.2425`, `360`, `365.25636` | `365.25` |
| `--dasha-levels N`, `--dasha-systems` | 1–5; `vimshottari`, `yogini` | 2; `vimshottari` |
| `--features a,b` | `panchanga`, `strength`, `relationships`, `drishti`, `ashtakavarga`, `jaimini`, `kp`, `yogas`, `sade_sati`, `all` (Vedic only) | none |
| `--reference-utc T` | instant for "current" daśā periods | none (nothing is "current") |
| `--sunrise` | `upper_limb_refracted`, `centre_refracted`, `centre_unrefracted` | `upper_limb_refracted` |
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
| `eshana panchanga ... [--natal-moon DEG] [--sunrise C]` | pañcāṅga, rise and set, horā, choghaḍiyā, Rāhu kālam, special lagnas; Tārā and Candra bala with a natal Moon |
| `eshana matching --a D,T,TZ,LAT,LON --b ... [--ayanamsha A]` | aṣṭakūṭa compatibility with Manglik flags |
| `eshana ephemeris --start T --step-days N --count N [--bodies ...]` | a longitude table (transits, progressions) |
| `eshana ingresses --body B --start T --end T` | sign ingresses, retrograde re-entries included |
| `eshana stations --body B --start T --end T [--tolerance-seconds S]` | stations and the retrograde intervals between them, clipped at the range |
| `eshana crossings --body B --point DEG --start T --end T [--aspects A,B] [--touch-orb DEG]` | when a body reaches a point or an aspect to it: direction, residual, and touches that stop short |
| `eshana rulesets` | the built-in YAML rule packs, verbatim |
| `eshana ashtakavarga-transits --start T --end T [--transit-bodies saturn,jupiter] <natal chart options>` | the natal BAV and SAV cells each transiting graha reads, as segments between ingresses |
| `eshana muhurta --lat φ --lon λ --tz ZONE --start T --end T [--profile ID] [--disable a,b] [--constraint id,…] [--show f,…]` | muhūrta windows composed from exact factor intervals; an empty result says which constraints removed it |
| `eshana calendar --birth date,time,tz,lat,lon --lat φ --lon λ --tz ZONE --start T --end T [--format json\|csv\|ics]` | personal calendar: windows with tārā and candra bala, day summaries; CSV and ICS print the file |
| `eshana pair-calendar --a SPEC --b SPEC --lat φ --lon λ --tz ZONE --start T --end T` | windows that suit both people and the event, each explained |
| `eshana progress --at T [--angles METHOD] <birth options>` | secondary progressions: fractional age, positions at birth + age days, angles under a named method |
| `eshana kp-table` | the KP 1–249 horary number table |
| `eshana kp-horary --number N --utc T --lat φ --lon λ [--kp-occupancy ID]` | KP horary chart: the number fixes the Ascendant; the moment and place are those of judgement (ayanāṃśa defaults to krishnamurti, recorded) |
| `eshana rise-set --date D --tz ZONE --lat φ --lon λ [--days N] [--bodies sun,moon,venus] [--sunrise C] [--no-twilight]` | rise, upper transit and set of each body, and civil, nautical and astronomical twilight, for local days |
| `eshana returns --start T --end T [--return-body sun\|moon\|…] [--at-lat φ --at-lon λ] [--no-charts] <birth options>` | solar, lunar or any body's returns to the natal longitude, with a chart at each (relocated with --at-lat/--at-lon) |
| `eshana gochara --start T --end T [--transit-bodies a,b] <birth options with --ayanamsha>` | transits counted from the natal Moon and lagna, with retrograde and combustion intervals |
| `eshana eclipses --start T --end T [--kinds solar,lunar]` | solar and lunar eclipses: type, time of greatest eclipse, magnitude |
| `eshana kernel list\|verify PATH\|path NAME` | the JPL kernels ESHANA knows; check a downloaded one against NAIF's MD5 |
| `eshana methods` | every method family and id the engine implements |
| `eshana request FILE\|-` | send a raw JSON request |
| `eshana doctor` | versions, tz database, providers, rule packs, error codes |
| `eshana explain ESH-CODE` | what a code means |
| `eshana schema` | the JSON Schema of the envelope |

## Exit status

`0` success. `2` any refusal or error: stdout carries an error envelope
(`{"kind": "error", "error": {"code": "ESH-…", "message": "…"}}`) and stderr one line `CODE: message`.
