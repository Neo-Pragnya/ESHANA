# What I need from you to close the capability expansion

**Outcome so far (2026-11-23).**

- **APP-01:** ADR-013 accepted. Done.
- **APP-04:** Hindi approved. Done. The language is now a setting in the studio (English by default).
- **MUH-03 and DASH-01:** kept as they are until experts review them.
- **BASE-03:** approved, but the install still needs the permission rule in §2.

Five tracker items wait on something only you can give. Each section says what the item is, what is already built,
exactly what I need, and where to write your answer. Fill in the **Your answer** blocks (or replace the `[ ]` with
`[x]`), save the file, and tell me it is updated. You don't need to answer everything at once: a section you leave
blank stays as it is.

Quick view:

| # | Task | What I need | Effort for you |
|---|---|---|---|
| 1 | APP-01 | Accept, change or reject ADR-013 (offline engine in the studio) | 2 minutes: one decision |
| 2 | BASE-03 | A permission rule to install PyJHora (test-only), **or** a ṣaḍbala table from your reference program | 1 minute (permission) or 20 minutes (table) |
| 3 | MUH-03 | Confirm or correct the muhūrta rules and profiles | 20–40 minutes, practitioner knowledge |
| 4 | DASH-01 | One worked example per conditional daśā (7 systems), from a book or your software | Depends on your sources |
| 5 | APP-04 | Review the Hindi interface labels (49 strings) | 15 minutes, a Hindi reader |

RES-03 (the birth-time evaluator) needs nothing now; it waits until there is a set of births timed by hospital or civil
record to test it against. If you ever have such a set, tell me in section 6.

---

## 1. APP-01: accept ADR-013, the offline engine in the studio

**What it is.** `eshana-report --live` embeds the same Rust engine, compiled to WebAssembly, inside the HTML file, so
the compare workspace and the playground recompute offline with exactly the rules the CLI and Python use. The cost is
about 4 MB more per file. Without `--live` a file is report-only, as before. Read: [ADR-013](../docs/adr/013-local-engine-adapter.md).

**Already done.** Built, tested, and checked against the native engine (same results, floats within 1e-9).

**Decision needed** (tick one):

- [x] **Accept** as written (opt-in with `--live`; report-only files stay the default)
- [ ] **Accept, but make `--live` the default** (every file gets ~4 MB larger)
- [ ] **Change**: describe below
- [ ] **Reject**: remove `--live` and keep the studio report-only

Your answer / comments:

```
```

---

## 2. BASE-03: ṣaḍbala per component

**What it is.** ESHANA's ṣaḍbala totals for the reference chart (Sri Ramana Maharishi, 30-12-1879 01:00, Madurai,
LMT +5:21, Lahiri) differ from the printed totals by −66 to +41 virūpas. The printed source gives **totals only**, so I
cannot tell which of the six parts causes the difference. My side of the comparison is in
[shadbala-reconciliation.md](../docs/reference/shadbala-reconciliation.md). I will **not** adjust any component to make
a total match without knowing which one is wrong.

**I need one of these** (either is enough):

### Option A: let me use PyJHora as an offline reference (quickest)

PyJHora is an open-source Python Jyotiṣa program. It would be installed **only** in a throw-away test environment, never
added to ESHANA's dependencies; it would generate a reference file once, like the Swiss Ephemeris references already
used for the other checks. Two attempts to install it were blocked by Claude Code's permission check, so you need to
allow it. Add this to `.claude/settings.local.json` in the ESHANA repository. Approving it in this file is not enough: the check
runs before I act, so the rule itself must be in your settings (you can also type `/permissions` in Claude Code):

```json
{ "permissions": { "allow": ["Bash(uv pip install * PyJHora*)"] } }
```

- [x] I added the rule / I allow it: install PyJHora as a test-only reference

### Option B: send the table from your reference program

For the reference chart above, with Lahiri ayanāṃśa, the six components for each of the seven
grahas, in virūpas (or rūpas; say which). Fill in what your program shows:

| Graha | Sthāna | Dig | Kāla | Ceṣṭā | Naisargika | Dṛk | Total |
|---|---|---|---|---|---|---|---|
| Sun | | | | | | | |
| Moon | | | | | | | |
| Mars | | | | | | | |
| Mercury | | | | | | | |
| Jupiter | | | | | | | |
| Venus | | | | | | | |
| Saturn | | | | | | | |

If your program also breaks **kāla bala** down, these parts help most (that is where programs differ):

| Graha | Natonnata | Pakṣa | Tribhāga | Abda | Māsa | Vāra | Horā | Ayana | Yuddha |
|---|---|---|---|---|---|---|---|---|---|
| Sun | | | | | | | | | |
| Moon | | | | | | | | | |
| Mars | | | | | | | | | |
| Mercury | | | | | | | | | |
| Jupiter | | | | | | | | | |
| Venus | | | | | | | | | |
| Saturn | | | | | | | | | |

Program name and version (so the comparison can cite it):

```
```

---

## 3. MUH-03: the muhūrta rules and profiles

**What it is.** `muhurta_windows` and the personal calendar find time windows that pass a list of rules. The
definitions are in [`muhurta-profiles.yaml`](../crates/eshana-engine/rulesets/muhurta-profiles.yaml). Fixed
calculations (Rāhu kāla, Abhijit, Brahma muhūrta) are marked *stable*; the combined profiles are marked *experimental*
until a practitioner confirms them. I need you (or someone you trust) to confirm or correct each line.

### 3a. The rules

Mark each **OK** or write the correction.

| Rule | What ESHANA does now | OK? / correction |
|---|---|---|
| `no_rahu_kalam`, `no_yamagandam`, `no_gulika_kalam` | avoid these daily eighths of the day (by weekday) | |
| `no_rikta_tithi` | avoid tithis 4, 9, 14 of both pakṣas (Caturthī, Navamī, Caturdaśī) | |
| `no_amavasya` | avoid tithi 30 | |
| `no_panchaka` | avoid the Moon in Kumbha or Mīna (sidereal) | should Pañcaka start at Dhaniṣṭhā 3rd pada (mid-Makara by nakṣatra) instead of the sign boundary? |
| `no_vishti_karana` | avoid Viṣṭi (Bhadrā) karaṇa, everywhere | should Bhadrā's residence (svarga/pātāla/bhūloka by Moon sign) exempt some periods? |
| `abhijit_only` + `not_on_wednesday` | Abhijit = the 8th of 15 day muhūrtas; not used on Wednesday | |
| `brahma_muhurta_only` | the 2nd-last of 15 night muhūrtas (about 96 to 48 minutes before sunrise) | |
| `good_tara` | favourable tārās: Sampat, Kṣema, Sādhaka, Mitra, Ati-Mitra (2, 4, 6, 8, 9), counted from the natal Moon's nakṣatra | |
| `good_chandra_bala` | favourable houses of the transit Moon from the natal Moon: 1, 3, 6, 7, 10, 11 | |
| `good_choghadiya` | only Lābh, Amṛt, Śubh choghaḍiyās | should Cal (Chara) count as acceptable for travel? |

Sunrise convention used by default: upper limb, with refraction. Correct?

```
```

### 3b. The profiles

| Profile | Rules it combines | Gaps I already know about |
|---|---|---|
| `general_auspicious` | the three daily avoidances, rikta, Amāvāsyā, Pañcaka, Viṣṭi | śūnya tithis by lunar month; mṛtyu and other vāra–nakṣatra yogas; lagna śuddhi |
| `personal_general` | the above + good tārā + good candra bala | same, plus the person's daśā |
| `pair_general` | the above for both people | same, plus both daśās |
| `travel` | the daily avoidances, rikta, Amāvāsyā, good choghaḍiyā | diśā śūla (direction by weekday); the traveller's tārā and candra bala |

For each profile: is anything included that should not be, or missing that must be there before it can leave
*experimental*?

```
general_auspicious:
personal_general:
pair_general:
travel:
```

### 3c. The rules I have not built (please give the definitions you follow, with the source)

- **Śūnya tithis by lunar month** (which tithis are empty in which month; amānta or pūrṇimānta month?):

  ```
  ```

- **Vāra–nakṣatra yogas** (mṛtyu, dagdha, amṛta-siddhi, sarvārtha-siddhi…): the weekday–nakṣatra pairs for each:

  ```
  ```

- **Diśā śūla** (forbidden travel direction for each weekday):

  ```
  ```

- **Lagna śuddhi** (which rising-sign conditions you want checked):

  ```
  ```

- Any purpose-specific presets you want (marriage, griha praveśa, starting a business, vehicle…), with their rules:

  ```
  ```

Reviewed by (name, so the profile can record who confirmed it) and date:

```
```

---

## 4. DASH-01: worked examples for the conditional daśās

**What it is.** Seven daśā systems that apply only when a chart meets a condition. ESHANA computes all seven for every
chart and reports whether the condition holds. The lords, years and starting rules are in
[`dashas-extended.yaml`](../crates/eshana-engine/rulesets/dashas-extended.yaml) (`conditional:`). Each has a
hand-checked unit test, but no **published** example to check against, so they stay *implemented*, not *verified*.

| System | Cycle | Applies when | Starts from |
|---|---|---|---|
| Dvādaśottarī | 112 y | lagna in a navāṃśa of Venus | counted to Revatī from the birth nakṣatra |
| Pañcottarī | 105 y | Karka lagna in Karka dvādaśāṃśa | counted from Anurādhā |
| Śatābdikā | 100 y | vargottama lagna | counted from Revatī |
| Caturāśīti-samā | 84 y | 10th lord in the 10th | counted from Svātī |
| Dvisaptati-samā | 72 y | lagna lord in the 7th, or 7th lord in the lagna | counted from Mūla |
| Ṣaṣṭihāyanī | 60 y | Sun in the lagna | by nakṣatra groups |
| Ṣaṭtriṃśat-samā | 36 y | day birth in the Sun's horā or night birth in the Moon's horā | counted from Śravaṇa |

**I need, per system, one example** from a book or from software you trust: the birth data and the printed
mahādaśā sequence with dates (antardaśās too, if shown). Also tell me if your source uses a different condition or
starting nakṣatra than the table above, because schools differ.

Copy this block once per system:

```
System:
Source (book + page, or software + version):
Birth: date, time, time zone or offset, place (lat/lon), ayanāṃśa:
Balance at birth (lord, years-months-days), if printed:
Mahādaśās (lord, start date) in order:
Antardaśās, if printed:
Condition or start rule differs from the table above? How:
```

Even one or two systems help: each one with a matching example moves to *verified* on its own.

---

## 5. APP-04: Hindi interface labels

**What it is.** `eshana-report --locale hi` shows the studio's interface in Hindi. Only labels are translated (page
names, group names, toolbar, the "not computed" notice); values, numbers, dates and error codes stay as they are. I
wrote the catalogue; it is marked **draft** and the studio says so.

File: [`hi.json`](../crates/eshana-python/python/eshana/data/locales/hi.json): 32 page names, 8 group names and 9
interface strings.

**What I need** (any one):

- [ ] Edit `hi.json` directly and tell me; I will check it still loads and set `reviewed_by`
- [ ] Write corrections below as `key: current → better`
- [x] Approve as is

Things worth checking: whether to use Sanskrit terms (e.g. *भाव*, *दशा*) or everyday Hindi; consistency of terms with
what Hindi astrology books use; and that the short toolbar words fit.

```
Reviewer name:
Corrections:
```

---

## 5b. New Hindi labels to review (added 2026-11-24, after your approval of the catalogue)

The download buttons' tooltips and screen-reader labels now follow the interface language. These four strings are
mine, listed under `pending_review` in `hi.json`; correct any as `key: current → better`, or tick to accept.

| Key | English | Hindi (draft) |
|---|---|---|
| `ui.download_group` | Download {label} as an image | {label} को चित्र के रूप में डाउनलोड करें |
| `ui.download_title` | Download “{label}” as {kind}, as shown now | “{label}” को {kind} के रूप में डाउनलोड करें, जैसा अभी दिख रहा है |
| `ui.download_aria` | Download {label} as {kind} | {label} को {kind} के रूप में डाउनलोड करें |
| `ui.download_error` | Could not create the image: | चित्र नहीं बन सका: |

- [ ] Accept as written
- [ ] Corrections:

## 5c. Lal Kitab pakka ghar (LK-01, added 2026-11-24)

The Lal Kitab chart is built (houses fixed to the signs, grahas by whole-sign house from the lagna). The pakka ghar
table in `crates/eshana-engine/rulesets/lal-kitab.yaml` is the commonly taught one and is marked **draft**:

| Graha | Pakka ghar |
|---|---|
| Sun | 1 |
| Jupiter | 2, 5, 9, 11, 12 |
| Mars | 3, 8 |
| Moon | 4 |
| Ketu | 6 |
| Venus, Mercury | 7 |
| Saturn | 8, 10 |
| Rahu | 12 |

- [ ] Accept as written
- [ ] Corrections (edition and page):
- [ ] Which further Lal Kitab rules to add (aspects, sleeping houses, varshphal), with the source you follow

## 6. Anything else

Other priorities, changes to the order, things in the studio you want different, or a set of timed births for RES-03:

```
```
