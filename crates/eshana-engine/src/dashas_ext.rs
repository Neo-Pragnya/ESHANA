//! More daśā systems: Aṣṭottarī, Ṣoḍaśottarī, Chara (K. N. Rao), Nārāyaṇa, Navāṃśa, Śūla and Kālachakra.
//!
//! Each system is its own attribute `vedic.dasha.<system>` with mahādaśās and their antardaśās (start and end in
//! UTC), the lords or signs, the chain running at the reference instant and a `conventions` object that names
//! every choice. Tables live in `rulesets/dashas-extended.yaml`; this module applies them.
//!
//! Sign-based systems compare signs by the Jaimini strength rules in [`stronger`]; the rule that decided is
//! reported with the result so that a reader can check it.

use crate::analysis::{AnalysisCtx, Derived};
use crate::astro::RASHIS;
use crate::jyotish::{self, house_from, lord_of, rashi_drishti_targets, sign_of, VChart, NINE};
use crate::vedic::{self, nakshatra_of, NAKSHATRAS, NAKSHATRA_SPAN, PADA_SPAN};
use eshana_core::{norm360, Result};
use eshana_time::Instant;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Raw YAML of the pack.
pub const SOURCE: &str = include_str!("../rulesets/dashas-extended.yaml");

#[derive(Debug, Deserialize)]
struct Pack {
    id: String,
    version: String,
    year_days: f64,
    ashtottari: StarTable,
    shodashottari: StarTable,
    savya_signs: Vec<usize>,
    co_lords: BTreeMap<usize, [String; 2]>,
    shoola_years: f64,
    kalachakra_years: [f64; 12],
    kalachakra: Kala,
}

#[derive(Debug, Deserialize)]
struct StarTable {
    lords: Vec<StarLord>,
    #[serde(default)]
    first_nakshatra: usize,
    applicability: String,
}

#[derive(Debug, Deserialize)]
struct StarLord {
    lord: String,
    years: f64,
    #[serde(default)]
    nakshatras: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct Kala {
    savya_ashvini: Vec<[usize; 9]>,
    savya_bharani: Vec<[usize; 9]>,
    apasavya_rohini: Vec<[usize; 9]>,
    apasavya_mrigashira: Vec<[usize; 9]>,
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    // A built-in pack that does not parse is a build defect, caught by the unit tests.
    P.get_or_init(|| {
        serde_norway::from_str(SOURCE)
            .unwrap_or_else(|e| panic!("rule pack dashas-extended.yaml is invalid: {e}"))
    })
}

fn rule() -> String {
    let p = pack();
    format!("{}@{}", p.id, p.version)
}

const SIGN_EN: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

fn iso(jd: f64) -> String {
    Instant::from_jd_utc(jd).utc
}

// ── periods ──────────────────────────────────────────────────────────────────────────────────────────────────────
/// One period: a sign (sign-based systems) or a graha (nakṣatra-based systems).
#[derive(Debug, Clone)]
struct Period {
    sign: Option<usize>,
    lord: Option<String>,
    start: f64,
    end: f64,
    years: f64,
    kids: Vec<Period>,
    extra: Value,
}

impl Period {
    fn sign(sign: usize, start: f64, years: f64, yd: f64) -> Self {
        Period {
            sign: Some(sign),
            lord: None,
            start,
            end: start + years * yd,
            years,
            kids: vec![],
            extra: Value::Null,
        }
    }
    fn graha(lord: &str, start: f64, years: f64, yd: f64) -> Self {
        Period {
            sign: None,
            lord: Some(lord.to_string()),
            start,
            end: start + years * yd,
            years,
            kids: vec![],
            extra: Value::Null,
        }
    }
    fn label(&self) -> String {
        match (&self.sign, &self.lord) {
            (Some(s), _) => SIGN_EN[*s].to_string(),
            (None, Some(l)) => l.clone(),
            _ => String::new(),
        }
    }
    fn head(&self, birth: f64) -> Value {
        let mut v = json!({"label": self.label(), "years": self.years, "start_utc": iso(self.start.max(birth)),
                           "end_utc": iso(self.end), "clipped_at_birth": self.start < birth});
        if let Some(s) = self.sign {
            v["sign"] = json!(s);
            v["rashi"] = json!(RASHIS[s]);
        }
        if let Some(l) = &self.lord {
            v["lord"] = json!(l);
        }
        if let Value::Object(m) = &self.extra {
            for (k, x) in m {
                v[k] = x.clone();
            }
        }
        v
    }
    fn to_json(&self, birth: f64) -> Value {
        let mut v = self.head(birth);
        if !self.kids.is_empty() {
            v["antardashas"] = Value::Array(
                self.kids
                    .iter()
                    .filter(|k| k.end > birth)
                    .map(|k| k.head(birth))
                    .collect(),
            );
        }
        v
    }
}

type Subdivide<'a> = &'a dyn Fn(&Period) -> Vec<Period>;

/// The chain of periods containing `at`, given a function that subdivides a period (for deeper levels).
fn chain(mahas: &[Period], at: f64, deeper: Subdivide) -> Option<Value> {
    let m = mahas.iter().find(|p| p.start <= at && at < p.end)?;
    let mut out = vec![(vedic::LEVEL_NAMES[0], m.clone())];
    let mut cur = m.clone();
    for name in vedic::LEVEL_NAMES.iter().skip(1) {
        let kids = if cur.kids.is_empty() {
            deeper(&cur)
        } else {
            cur.kids.clone()
        };
        let Some(k) = kids.into_iter().find(|p| p.start <= at && at < p.end) else {
            break;
        };
        out.push((name, k.clone()));
        cur = k;
    }
    let periods: Vec<Value> = out
        .iter()
        .map(|(lvl, p)| {
            let mut v =
                json!({"level": lvl, "label": p.label(), "start_utc": iso(p.start), "end_utc": iso(p.end)});
            if let Some(s) = p.sign {
                v["sign"] = json!(s);
            }
            if let Some(l) = &p.lord {
                v["lord"] = json!(l);
            }
            v
        })
        .collect();
    Some(json!({"at_utc": iso(at), "periods": periods}))
}

/// Years of daśā to cover: at least 150 years from birth, and 80 years past the reference instant (at most 300).
fn horizon_years(birth: f64, reference: Option<f64>, yd: f64) -> f64 {
    let r = reference.map_or(0.0, |r| (r - birth) / yd + 80.0);
    r.clamp(150.0, 300.0)
}

// ── nakṣatra-based systems (Aṣṭottarī, Ṣoḍaśottarī) ──────────────────────────────────────────────────────────────
/// Sub-periods of a graha period: the sequence from the period's own lord, each share proportional to its years.
fn graha_children(p: &Period, seq: &[(String, f64)], yd: f64) -> Vec<Period> {
    let total: f64 = seq.iter().map(|x| x.1).sum();
    let Some(i0) = seq.iter().position(|x| Some(&x.0) == p.lord.as_ref()) else {
        return vec![];
    };
    let n = seq.len();
    let mut t = p.start;
    (0..n)
        .map(|k| {
            let (l, y) = &seq[(i0 + k) % n];
            let mut q = Period::graha(l, t, p.years * y / total, yd);
            if k == n - 1 {
                q.end = p.end;
            }
            t = q.end;
            q
        })
        .collect()
}

fn graha_system(
    seq: &[(String, f64)],
    first: usize,
    elapsed_years: f64,
    birth: f64,
    reference: Option<f64>,
    yd: f64,
) -> (Vec<Period>, Option<Value>) {
    let horizon = birth + horizon_years(birth, reference, yd) * yd;
    let mut t = birth - elapsed_years * yd;
    let mut mahas = vec![];
    let mut k = 0;
    while t < horizon {
        let (l, y) = &seq[(first + k) % seq.len()];
        let mut p = Period::graha(l, t, *y, yd);
        p.kids = graha_children(&p, seq, yd);
        t = p.end;
        if p.end > birth {
            mahas.push(p);
        }
        k += 1;
    }
    let cur = reference.and_then(|at| chain(&mahas, at, &|p| graha_children(p, seq, yd)));
    (mahas, cur)
}

/// The 28-nakṣatra position of a sidereal longitude: (index, with 27 = Abhijit; fraction elapsed in it).
fn nakshatra28(lon: f64) -> (usize, f64) {
    let lon = norm360(lon);
    let a0 = 276.0 + 40.0 / 60.0;
    let a1 = 280.0 + 53.0 / 60.0 + 20.0 / 3600.0;
    let u0 = 20.0 * NAKSHATRA_SPAN;
    let s1 = 22.0 * NAKSHATRA_SPAN;
    if (a0..a1).contains(&lon) {
        return (27, (lon - a0) / (a1 - a0));
    }
    if (u0..a0).contains(&lon) {
        return (20, (lon - u0) / (a0 - u0));
    }
    if (a1..s1).contains(&lon) {
        return (21, (lon - a1) / (s1 - a1));
    }
    let nk = nakshatra_of(lon);
    (nk.index, nk.elapsed)
}

fn paksha_and_day(x: &AnalysisCtx) -> (Option<&'static str>, Option<bool>) {
    let paksha = match (x.vc.lon.get("sun"), x.vc.lon.get("moon")) {
        (Some(s), Some(m)) => Some(if norm360(m - s) < 180.0 {
            "shukla"
        } else {
            "krishna"
        }),
        _ => None,
    };
    let jd = |k: &str| x.panchanga.and_then(|p| p[k]["jd_ut"].as_f64());
    let day = match (jd("sunrise"), jd("sunset")) {
        (Some(r), Some(s)) => Some(x.inst.jd_ut >= r && x.inst.jd_ut < s),
        _ => None,
    };
    (paksha, day)
}

fn ashtottari(x: &AnalysisCtx, moon: f64, yd: f64) -> Value {
    let t = &pack().ashtottari;
    let seq: Vec<(String, f64)> = t.lords.iter().map(|l| (l.lord.clone(), l.years)).collect();
    let (nk28, frac) = nakshatra28(moon);
    let (gi, pos) = t
        .lords
        .iter()
        .enumerate()
        .find_map(|(i, l)| l.nakshatras.iter().position(|&n| n == nk28).map(|p| (i, p)))
        .unwrap_or((0, 0));
    let g = &t.lords[gi];
    let elapsed = (pos as f64 + frac) / g.nakshatras.len() as f64 * g.years;
    let birth = x.inst.jd_ut;
    let (mahas, cur) = graha_system(&seq, gi, elapsed, birth, x.reference_jd, yd);
    let lagna = x.vc.asc.map(sign_of);
    let rahu = x.vc.lon.get("rahu").map(|l| sign_of(*l));
    let rule1 = match (lagna, rahu) {
        (Some(lg), Some(r)) => x.vc.lon.get(lord_of(lg)).map(|l| {
            let h = house_from(sign_of(*l), r);
            r != lg && [1, 4, 5, 7, 9, 10].contains(&h)
        }),
        _ => None,
    };
    let (paksha, day) = paksha_and_day(x);
    let rule2 = match (paksha, day) {
        (Some(p), Some(d)) => Some((p == "krishna" && d) || (p == "shukla" && !d)),
        _ => None,
    };
    let nk_name = if nk28 == 27 { "Abhijit" } else { NAKSHATRAS[nk28] };
    let mut v = json!({
        "system": "ashtottari", "kind": "graha", "cycle_years": 108, "year_days": yd,
        "moon_nakshatra": nk_name, "starting_lord": g.lord,
        "balance_at_birth_years": g.years - elapsed,
        "applicability": {"applies": rule1, "rule": t.applicability,
            "rahu_kendra_trikona_from_lagna_lord": rule1, "paksha": paksha, "day_birth": day,
            "paksha_day_night_rule": rule2},
        "conventions": {
            "nakshatras": "28-nakṣatra scheme with Abhijit, groups counted from Ārdrā (Sun 4, Moon 3, Mars 4, Mercury 3, Saturn 4 with Abhijit, Jupiter 3, Rāhu 4, Venus 3)",
            "balance": "the part of the group of nakṣatras of the lord not yet traversed by the Moon, times the years of the lord",
            "antardasha": "the eight lords from the mahādaśā lord, each share its years out of 108",
            "year": format!("{yd}-day year"),
            "applicability": "computed for every chart; applicability.applies is the BPHS Rāhu rule, paksha_day_night_rule the later one",
        },
        "mahadashas": mahas.iter().map(|p| p.to_json(birth)).collect::<Vec<_>>(),
    });
    if let Some(c) = cur {
        v["current"] = c;
    }
    v
}

fn shodashottari(x: &AnalysisCtx, moon: f64, yd: f64) -> Value {
    let t = &pack().shodashottari;
    let seq: Vec<(String, f64)> = t.lords.iter().map(|l| (l.lord.clone(), l.years)).collect();
    let nk = nakshatra_of(moon);
    let first = ((nk.index + 27 - t.first_nakshatra) % 27) % seq.len();
    let elapsed = nk.elapsed * seq[first].1;
    let birth = x.inst.jd_ut;
    let (mahas, cur) = graha_system(&seq, first, elapsed, birth, x.reference_jd, yd);
    let (paksha, _) = paksha_and_day(x);
    let hora = x.vc.asc.map(|a| {
        let s = sign_of(a);
        let first_half = norm360(a) - 30.0 * (s as f64) < 15.0;
        if (s % 2 == 0) == first_half {
            "sun"
        } else {
            "moon"
        }
    });
    let applies = match (hora, paksha) {
        (Some(h), Some(p)) => Some((h == "sun" && p == "krishna") || (h == "moon" && p == "shukla")),
        _ => None,
    };
    let mut v = json!({
        "system": "shodashottari", "kind": "graha", "cycle_years": 116, "year_days": yd,
        "moon_nakshatra": nk.name, "starting_lord": seq[first].0,
        "balance_at_birth_years": seq[first].1 - elapsed,
        "applicability": {"applies": applies, "rule": t.applicability, "lagna_hora": hora, "paksha": paksha},
        "conventions": {
            "start": "count the janma nakṣatra from Puṣya; the count mod 8 picks the lord (Sun, Mars, Jupiter, Saturn, Ketu, Moon, Mercury, Venus)",
            "balance": "the part of the janma nakṣatra not yet traversed, times the years of the starting lord",
            "antardasha": "the eight lords from the mahādaśā lord, each share its years out of 116",
            "year": format!("{yd}-day year"),
            "applicability": "computed for every chart; applicability.applies says whether the BPHS condition holds",
        },
        "mahadashas": mahas.iter().map(|p| p.to_json(birth)).collect::<Vec<_>>(),
    });
    if let Some(c) = cur {
        v["current"] = c;
    }
    v
}

// ── Jaimini sign strength ────────────────────────────────────────────────────────────────────────────────────────
fn occupants(vc: &VChart, s: usize) -> Vec<&'static str> {
    NINE.iter()
        .copied()
        .filter(|g| vc.lon.get(*g).is_some_and(|l| sign_of(*l) == s))
        .collect()
}

fn graha_sign(vc: &VChart, g: &str) -> Option<usize> {
    vc.lon.get(g).map(|l| sign_of(*l))
}

/// The lords of a sign: its graha lord, and the node co-lord of Vṛścika and Kumbha.
fn lords(s: usize) -> Vec<String> {
    match pack().co_lords.get(&s) {
        Some(l) => l.to_vec(),
        None => vec![lord_of(s).to_string()],
    }
}

fn has_dignity(vc: &VChart, g: &str, d: &str) -> bool {
    jyotish::SEVEN.contains(&g) && vc.lon.get(g).is_some_and(|l| jyotish::dignity(g, *l) == d)
}

/// Jaimini strength of two signs. Rules in order, the first that separates them decides: (1) more grahas in the
/// sign; (2) more of Jupiter, Mercury and the lord(s) of the sign joining or aspecting it by rāśi dṛṣṭi; (3) more
/// exalted grahas in it; (4) its lord in a sign of the other oddity; (5) dual over fixed over movable; (6) its lord
/// further advanced in its sign. Returns the stronger sign and the rule that decided (0 = a tie).
fn stronger(vc: &VChart, a: usize, b: usize) -> (usize, u8) {
    let r1 = |s: usize| occupants(vc, s).len() as i64;
    let r2 = |s: usize| {
        let mut who = vec!["jupiter".to_string(), "mercury".to_string()];
        for l in lords(s) {
            if !who.contains(&l) {
                who.push(l);
            }
        }
        who.iter()
            .filter(|g| graha_sign(vc, g).is_some_and(|gs| gs == s || rashi_drishti_targets(gs).contains(&s)))
            .count() as i64
    };
    let r3 = |s: usize| {
        occupants(vc, s)
            .iter()
            .filter(|g| has_dignity(vc, g, "exalted"))
            .count() as i64
    };
    let r4 = |s: usize| i64::from(graha_sign(vc, lord_of(s)).is_some_and(|ls| ls % 2 != s % 2));
    let r5 = |s: usize| (s % 3) as i64;
    let r6 = |s: usize| {
        vc.lon
            .get(lord_of(s))
            .map_or(0, |l| (norm360(*l) % 30.0 * 3600.0) as i64)
    };
    let rules: [&dyn Fn(usize) -> i64; 6] = [&r1, &r2, &r3, &r4, &r5, &r6];
    for (i, r) in rules.iter().enumerate() {
        let (x, y) = (r(a), r(b));
        if x != y {
            return (if x > y { a } else { b }, i as u8 + 1);
        }
    }
    (a, 0)
}

const STRENGTH_RULES: &str = "Jaimini strength, the first rule that separates the two signs: (1) more grahas in the sign; (2) more of Jupiter, Mercury and the lord of the sign joining or aspecting it by rāśi dṛṣṭi; (3) more exalted grahas in it; (4) its lord in a sign of the other oddity; (5) dual over fixed over movable; (6) its lord further advanced in its sign";

/// For Vṛścika and Kumbha, the stronger of the two lords: if one is in the sign, the other; else the one with more
/// grahas beside it; then an exalted one; then the one further advanced in its sign.
fn effective_lord(vc: &VChart, s: usize, nodes: bool) -> String {
    let ls = lords(s);
    if ls.len() == 1 || !nodes {
        return ls[0].clone();
    }
    let (a, b) = (&ls[0], &ls[1]);
    let (sa, sb) = (graha_sign(vc, a), graha_sign(vc, b));
    if sa == Some(s) && sb != Some(s) {
        return b.clone();
    }
    if sb == Some(s) && sa != Some(s) {
        return a.clone();
    }
    let n = |g: Option<usize>| g.map_or(0, |x| occupants(vc, x).len());
    if n(sa) != n(sb) {
        return if n(sa) > n(sb) { a.clone() } else { b.clone() };
    }
    let (ea, eb) = (has_dignity(vc, a, "exalted"), has_dignity(vc, b, "exalted"));
    if ea != eb {
        return if ea { a.clone() } else { b.clone() };
    }
    let d = |g: &str| vc.lon.get(g).map_or(0.0, |l| norm360(*l) % 30.0);
    if d(b) > d(a) {
        b.clone()
    } else {
        a.clone()
    }
}

/// Daśā years of a sign: signs counted from it to its lord (forward or backward), less one; 12 when the lord is in
/// the sign; one more for an exalted lord, one less for a debilitated lord (grahas only).
fn sign_years(vc: &VChart, s: usize, forward: bool, nodes: bool) -> (f64, String) {
    let lord = effective_lord(vc, s, nodes);
    let Some(ls) = graha_sign(vc, &lord) else {
        return (0.0, lord);
    };
    let steps = if forward {
        (ls + 12 - s) % 12
    } else {
        (s + 12 - ls) % 12
    };
    if steps == 0 {
        return (12.0, lord);
    }
    let mut y = steps as f64;
    if has_dignity(vc, &lord, "exalted") {
        y += 1.0;
    }
    if has_dignity(vc, &lord, "debilitated") {
        y -= 1.0;
    }
    (y, lord)
}

fn savya(s: usize) -> bool {
    pack().savya_signs.contains(&(s % 12))
}

fn step(s: usize, k: usize, forward: bool) -> usize {
    if forward {
        (s + k) % 12
    } else {
        (s + 12 * (k / 12 + 1) - k) % 12
    }
}

/// Twelve equal antardaśās from `first`, moving forward or backward.
fn equal_antars(p: &Period, first: usize, forward: bool, yd: f64) -> Vec<Period> {
    let y = p.years / 12.0;
    let mut t = p.start;
    (0..12)
        .map(|k| {
            let mut q = Period::sign(step(first, k, forward), t, y, yd);
            if k == 11 {
                q.end = p.end;
            }
            t = q.end;
            q
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Cycles {
    /// Every cycle repeats the first cycle of years.
    Repeat,
    /// The second cycle runs 12 minus the first cycle of years, the third the first again, and so on.
    TwelveMinus,
}

/// How a sign-based system runs.
struct SignRun<'a> {
    start: usize,
    forward: bool,
    years: &'a dyn Fn(usize) -> f64,
    cycles: Cycles,
    antars: Subdivide<'a>,
}

/// A sign-based system: signs in order from `start` (forward or backward), each nested with its antardaśās.
fn sign_system(run: &SignRun, birth: f64, reference: Option<f64>, yd: f64) -> (Vec<Period>, Option<Value>) {
    let horizon = birth + horizon_years(birth, reference, yd) * yd;
    let mut t = birth;
    let mut mahas = vec![];
    let mut k = 0usize;
    while t < horizon && k < 12 * 30 {
        let s = step(run.start, k % 12, run.forward);
        let first = (run.years)(s);
        let cycle = k / 12;
        let y = if run.cycles == Cycles::TwelveMinus && cycle % 2 == 1 {
            12.0 - first
        } else {
            first
        };
        k += 1;
        if y <= 0.0 {
            continue;
        }
        let mut p = Period::sign(s, t, y, yd);
        p.extra = json!({"cycle": cycle + 1});
        p.kids = (run.antars)(&p);
        t = p.end;
        mahas.push(p);
    }
    let cur = reference.and_then(|at| chain(&mahas, at, &|_| vec![]));
    (mahas, cur)
}

fn sign_value(
    system: &str,
    run: &SignRun,
    ctx: (f64, Option<f64>, f64),
    extra: Value,
    conventions: Value,
) -> Value {
    let (birth, reference, yd) = ctx;
    let (mahas, cur) = sign_system(run, birth, reference, yd);
    let mut v = json!({
        "system": system, "kind": "sign", "year_days": yd,
        "start_sign": run.start, "start_rashi": RASHIS[run.start],
        "direction": if run.forward { "forward" } else { "backward" },
        "conventions": conventions,
        "mahadashas": mahas.iter().map(|p| p.to_json(birth)).collect::<Vec<_>>(),
    });
    if let Value::Object(m) = extra {
        for (k, x) in m {
            v[k] = x;
        }
    }
    if let Some(c) = cur {
        v["current"] = c;
    }
    v
}

fn years_table(vc: &VChart, forward: &dyn Fn(usize) -> bool, nodes: bool) -> Value {
    Value::Array(
        (0..12)
            .map(|s| {
                let (y, l) = sign_years(vc, s, forward(s), nodes);
                json!({"sign": s, "rashi": RASHIS[s], "years": y, "lord": l,
                       "counted": if forward(s) { "forward" } else { "backward" }})
            })
            .collect(),
    )
}

fn chara(vc: &VChart, lagna: usize, birth: f64, reference: Option<f64>, yd: f64) -> Value {
    let fwd = |s: usize| savya(s);
    let years = |s: usize| sign_years(vc, s, fwd(s), true).0;
    let antars = |p: &Period| {
        let m = p.sign.unwrap_or(0);
        let f = savya(step(m, 8, true));
        equal_antars(p, step(m, 1, f), f, yd)
    };
    let run = SignRun {
        start: lagna,
        forward: savya(step(lagna, 8, true)),
        years: &years,
        cycles: Cycles::Repeat,
        antars: &antars,
    };
    sign_value(
        "chara",
        &run,
        (birth, reference, yd),
        json!({"method": "K. N. Rao", "years_by_sign": years_table(vc, &fwd, true)}),
        json!({
            "method": "K. N. Rao, Chara daśā",
            "start": "the lagna sign",
            "order": "consecutive signs; forward when the 9th sign from the lagna is savya (Meṣa, Vṛṣabha, Mithuna, Tulā, Vṛścika, Dhanu), else backward",
            "years": "count from the sign to its lord (forward for savya signs, backward for the others), less one; 12 when the lord is in the sign; +1 exalted, −1 debilitated lord",
            "co_lords": "Vṛścika (Mars, Ketu) and Kumbha (Saturn, Rāhu): if one lord is in the sign take the other, else the lord with more grahas beside it, then an exalted lord, then the one further advanced",
            "cycles": "every cycle repeats the years of the first (this reproduces the published reference chart)",
            "antardasha": "12 equal parts; from the sign next to the mahādaśā sign, forward or backward by the 9th sign from the mahādaśā sign, ending with the mahādaśā sign itself",
            "year": format!("{yd}-day year counted from the birth instant"),
        }),
    )
}

fn narayana_antars(vc: &VChart, p: &Period, yd: f64) -> Vec<Period> {
    let m = p.sign.unwrap_or(0);
    let (first, _) = stronger(vc, m, step(m, 6, true));
    equal_antars(p, first, m % 2 == 0, yd)
}

fn narayana(vc: &VChart, lagna: usize, birth: f64, reference: Option<f64>, yd: f64) -> Value {
    let (start, why) = stronger(vc, lagna, step(lagna, 6, true));
    let fwd = |s: usize| s % 2 == 0;
    let years = |s: usize| sign_years(vc, s, fwd(s), true).0;
    let antars = |p: &Period| narayana_antars(vc, p, yd);
    let run = SignRun {
        start,
        forward: savya(step(start, 8, true)),
        years: &years,
        cycles: Cycles::TwelveMinus,
        antars: &antars,
    };
    sign_value(
        "narayana",
        &run,
        (birth, reference, yd),
        json!({"start_from": {"lagna": lagna, "seventh": step(lagna, 6, true), "chosen": start, "decided_by_rule": why},
               "years_by_sign": years_table(vc, &fwd, true)}),
        json!({
            "start": "the stronger of the lagna and the 7th sign from it",
            "strength": STRENGTH_RULES,
            "order": "consecutive signs; forward when the 9th sign from the starting sign is savya, else backward",
            "years": "count from the sign to its lord (forward for odd signs, backward for even signs), less one; 12 when the lord is in the sign; +1 exalted, −1 debilitated lord",
            "co_lords": "as for Chara daśā (the stronger of the two lords)",
            "cycles": "the second cycle runs 12 minus the years of the first, the third repeats the first",
            "antardasha": "12 equal parts from the stronger of the mahādaśā sign and its 7th, forward for odd mahādaśā signs, backward for even",
            "year": format!("{yd}-day year counted from the birth instant"),
        }),
    )
}

fn navamsha(vc: &VChart, d9_lagna: usize, birth: f64, reference: Option<f64>, yd: f64) -> Value {
    let fwd = |_: usize| true;
    let years = |s: usize| sign_years(vc, s, true, false).0;
    let antars = |p: &Period| narayana_antars(vc, p, yd);
    let run = SignRun {
        start: d9_lagna,
        forward: savya(step(d9_lagna, 8, true)),
        years: &years,
        cycles: Cycles::TwelveMinus,
        antars: &antars,
    };
    sign_value(
        "navamsha",
        &run,
        (birth, reference, yd),
        json!({"years_by_sign": years_table(vc, &fwd, false)}),
        json!({
            "start": "the navāṃśa (D9) lagna sign",
            "order": "consecutive signs; forward when the 9th sign from the starting sign is savya, else backward",
            "years": "from the rāśi chart: count forward from the sign to its graha lord (Mars for Vṛścika, Saturn for Kumbha), less one; 12 when the lord is in the sign; +1 exalted, −1 debilitated lord (these choices reproduce the published reference chart)",
            "cycles": "the second cycle runs 12 minus the years of the first, the third repeats the first",
            "antardasha": "12 equal parts from the stronger of the mahādaśā sign and its 7th (rāśi chart), forward for odd mahādaśā signs, backward for even",
            "strength": STRENGTH_RULES,
            "year": format!("{yd}-day year counted from the birth instant"),
        }),
    )
}

fn shoola(vc: &VChart, lagna: usize, birth: f64, reference: Option<f64>, yd: f64) -> Value {
    let (start, why) = stronger(vc, lagna, step(lagna, 6, true));
    let ys = pack().shoola_years;
    let years = |_: usize| ys;
    let antars = |p: &Period| {
        let m = p.sign.unwrap_or(0);
        equal_antars(p, m, m % 2 == 0, yd)
    };
    let run = SignRun {
        start,
        forward: savya(step(start, 8, true)),
        years: &years,
        cycles: Cycles::Repeat,
        antars: &antars,
    };
    sign_value(
        "shoola",
        &run,
        (birth, reference, yd),
        json!({"start_from": {"lagna": lagna, "seventh": step(lagna, 6, true), "chosen": start, "decided_by_rule": why}}),
        json!({
            "start": "the stronger of the lagna and the 7th sign from it",
            "strength": STRENGTH_RULES,
            "order": "consecutive signs; forward when the 9th sign from the starting sign is savya, else backward",
            "years": format!("{ys} years for every sign"),
            "antardasha": "12 equal parts from the mahādaśā sign itself, forward for odd signs, backward for even",
            "year": format!("{yd}-day year counted from the birth instant"),
        }),
    )
}

// ── Kālachakra ───────────────────────────────────────────────────────────────────────────────────────────────────
/// The pada row (nine signs) for nakṣatra `n` (0–26) and pada `p` (0–3), and whether it is savya.
fn kala_row(n: usize, p: usize) -> ([usize; 9], bool) {
    let k = &pack().kalachakra;
    let ty = n % 3;
    if (n / 3) % 2 == 0 {
        let t = if ty == 1 {
            &k.savya_bharani
        } else {
            &k.savya_ashvini
        };
        (t[p], true)
    } else {
        let t = if ty == 1 {
            &k.apasavya_mrigashira
        } else {
            &k.apasavya_rohini
        };
        (t[p], false)
    }
}

fn gati(a: usize, b: usize) -> &'static str {
    match (b + 12 - a) % 12 {
        1 | 11 => "regular",
        2 | 10 => "manduka",
        4 | 8 => "simhavalokana",
        _ => "markata",
    }
}

fn kalachakra(moon: f64, birth: f64, reference: Option<f64>, yd: f64) -> Value {
    let ky = pack().kalachakra_years;
    let nk = nakshatra_of(moon);
    let within = norm360(moon) - nk.index as f64 * NAKSHATRA_SPAN;
    let pada = ((within / PADA_SPAN).floor() as usize).min(3);
    let frac = (within - pada as f64 * PADA_SPAN) / PADA_SPAN;
    let (row0, sv0) = kala_row(nk.index, pada);
    let para = |r: &[usize; 9]| r.iter().map(|s| ky[*s]).sum::<f64>();
    let elapsed = frac * para(&row0);
    let horizon = birth + horizon_years(birth, reference, yd) * yd;
    let mut t = birth - elapsed * yd;
    let (mut n, mut p) = (nk.index, pada);
    let mut mahas = vec![];
    let mut prev: Option<usize> = None;
    while t < horizon {
        let (row, sv) = kala_row(n, p);
        let pa = para(&row);
        let (deha, jiva) = if sv { (row[0], row[8]) } else { (row[8], row[0]) };
        for (i, &s) in row.iter().enumerate() {
            let mut m = Period::sign(s, t, ky[s], yd);
            m.extra = json!({"nakshatra": NAKSHATRAS[n], "pada": p + 1, "chakra": if sv { "savya" } else { "apasavya" },
                             "deha": deha, "jiva": jiva, "paramayus": pa,
                             "gati_from_previous": prev.map(|a| gati(a, s))});
            let mut tt = t;
            m.kids = (0..9)
                .map(|k| {
                    let ss = row[(i + k) % 9];
                    let mut q = Period::sign(ss, tt, ky[s] * ky[ss] / pa, yd);
                    if k == 8 {
                        q.end = m.end;
                    }
                    tt = q.end;
                    q
                })
                .collect();
            t = m.end;
            prev = Some(s);
            if m.end > birth {
                mahas.push(m);
            }
        }
        p += 1;
        if p == 4 {
            p = 0;
            n = (n + 1) % 27;
        }
    }
    let cur = reference.and_then(|at| chain(&mahas, at, &|_| vec![]));
    let (deha, jiva) = if sv0 {
        (row0[0], row0[8])
    } else {
        (row0[8], row0[0])
    };
    let mut v = json!({
        "system": "kalachakra", "kind": "sign", "year_days": yd,
        "moon_nakshatra": nk.name, "moon_pada": pada + 1, "chakra": if sv0 { "savya" } else { "apasavya" },
        "janma_pada_signs": row0, "paramayus": para(&row0), "deha": deha, "jiva": jiva,
        "deha_rashi": RASHIS[deha], "jiva_rashi": RASHIS[jiva],
        "balance_at_birth_years": para(&row0) - elapsed,
        "conventions": {
            "tables": "BPHS pada rows: savya Aśvinī type (also Kṛttikā type) and Bharaṇī type; apasavya Rohiṇī type (also Ārdrā type) and Mṛgaśirā type; sign years Meṣa 7, Vṛṣabha 16, Mithuna 9, Karka 21, Siṃha 5, Kanyā 9, Tulā 16, Vṛścika 7, Dhanu 10, Makara 4, Kumbha 4, Mīna 10",
            "deha_jiva": "savya: the first sign of the janma pada is the deha, the ninth the jīva; apasavya: the reverse",
            "balance": "the part of the janma pada traversed by the Moon, times the paramāyus of the pada, is already elapsed at birth",
            "sequence": "after the nine daśās of the janma pada, the rows of the following padas in order (next pada, then the next nakṣatra)",
            "gati": "each mahādaśā says how it follows the previous sign: regular, maṇḍūka (a jump of two signs), siṃhāvalokana (a jump to a trine) or markaṭa (any other turn)",
            "antardasha": "the nine signs of the same pada row from the mahādaśā sign, each share its years out of the paramāyus",
            "year": format!("{yd}-day year"),
        },
        "mahadashas": mahas.iter().map(|m| m.to_json(birth)).collect::<Vec<_>>(),
    });
    if let Some(c) = cur {
        v["current"] = c;
    }
    v
}

/// Compute this module attributes for a chart: one `vedic.dasha.<system>` per system (the sign-based systems
/// need the Ascendant).
///
/// # Errors
/// Propagates divisional-chart errors.
pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>> {
    let Some(&moon) = x.vc.lon.get("moon") else {
        return Ok(vec![]);
    };
    let yd = pack().year_days;
    let birth = x.inst.jd_ut;
    let r = rule();
    let d = |id: &str, v: Value, from: &[&str]| Derived {
        id: format!("vedic.dasha.{id}"),
        rule: r.clone(),
        value: v,
        from: from.iter().map(|s| (*s).to_string()).collect(),
    };
    let mut out = vec![
        d("ashtottari", ashtottari(x, moon, yd), &["moon", "rahu", "houses"]),
        d("shodashottari", shodashottari(x, moon, yd), &["moon", "houses"]),
        d(
            "kalachakra",
            kalachakra(moon, birth, x.reference_jd, yd),
            &["moon"],
        ),
    ];
    if let Some(asc) = x.vc.asc {
        let lagna = sign_of(asc);
        let all = &["positions", "houses"];
        out.push(d("chara", chara(x.vc, lagna, birth, x.reference_jd, yd), all));
        out.push(d(
            "narayana",
            narayana(x.vc, lagna, birth, x.reference_jd, yd),
            all,
        ));
        let d9 = vedic::divisional_sign(asc, 9)?;
        out.push(d("navamsha", navamsha(x.vc, d9, birth, x.reference_jd, yd), all));
        out.push(d("shoola", shoola(x.vc, lagna, birth, x.reference_jd, yd), all));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Ramana chart (Lahiri, true node) as the engine computes it.
    fn ramana() -> VChart {
        let mut c = VChart::default();
        for (g, l) in [
            ("sun", 255.594_811),
            ("moon", 88.328_216),
            ("mars", 21.977_757),
            ("mercury", 233.108_701),
            ("jupiter", 316.535_448),
            ("venus", 210.477_596),
            ("saturn", 347.023_339),
            ("rahu", 263.335_781),
            ("ketu", 83.335_781),
        ] {
            c.lon.insert(g.into(), l);
        }
        c.asc = Some(178.719_725);
        c
    }
    const BIRTH: f64 = 2_407_713.318_75;
    const YD: f64 = 365.2425;

    fn starts(v: &Value) -> Vec<(String, String)> {
        v["mahadashas"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                (
                    m["label"].as_str().unwrap().to_string(),
                    m["start_utc"].as_str().unwrap()[..10].to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn pack_parses_and_kalachakra_rows_have_classical_spans() {
        let p = pack();
        assert_eq!(p.ashtottari.lords.iter().map(|l| l.years).sum::<f64>(), 108.0);
        assert_eq!(p.shodashottari.lords.iter().map(|l| l.years).sum::<f64>(), 116.0);
        for n in 0..27 {
            for q in 0..4 {
                let r = kala_row(n, q).0;
                let span: f64 = r.iter().map(|s| p.kalachakra_years[*s]).sum();
                assert!([83.0, 85.0, 86.0, 100.0].contains(&span), "{n} {q} {span}");
            }
        }
        assert_eq!(nakshatra28(278.0).0, 27);
        assert_eq!(nakshatra28(270.0).0, 20);
        assert_eq!(nakshatra28(285.0).0, 21);
        assert_eq!(step(2, 3, false), 11);
        assert_eq!(step(2, 14, false), 0);
    }

    #[test]
    fn jaimini_strength_worked_examples() {
        let c = ramana();
        // Kanyā (empty) against Mīna (Saturn): more grahas
        assert_eq!(stronger(&c, 5, 11), (11, 1));
        // Mithuna (Moon, Ketu) against Dhanu (Sun, Rāhu): ties on rules 1-3; Mercury, lord of Mithuna, is in an
        // even sign and Jupiter, lord of Dhanu, in an odd one, so Mithuna wins on rule 4
        assert_eq!(stronger(&c, 2, 8), (2, 4));
        // Karka against Makara, both empty: Jupiter and Mercury aspect Karka, only Mercury aspects Makara
        assert_eq!(stronger(&c, 3, 9), (3, 2));
        // co-lords: Ketu (with the Moon) beats Mars (alone); Rāhu (with the Sun) beats Saturn (alone)
        assert_eq!(effective_lord(&c, 7, true), "ketu");
        assert_eq!(effective_lord(&c, 10, true), "rahu");
    }

    #[test]
    fn chara_reproduces_the_rao_years() {
        let c = ramana();
        let y: Vec<f64> = (0..12).map(|s| sign_years(&c, s, savya(s), true).0).collect();
        assert_eq!(
            y,
            vec![12.0, 6.0, 5.0, 1.0, 8.0, 10.0, 1.0, 7.0, 2.0, 10.0, 2.0, 1.0]
        );
        let v = chara(&c, 5, BIRTH, None, YD);
        let s = starts(&v);
        assert_eq!(s[0], ("Virgo".to_string(), "1879-12-29".to_string()));
        // repeating cycles of 65 years: Scorpio from 2020-12 in the third cycle
        assert!(
            s.iter().any(|(l, d)| l == "Scorpio" && d.starts_with("2020-12")),
            "{s:?}"
        );
        // Scorpio antardaśās run backward from Tulā and end with Scorpio itself
        let sco = v["mahadashas"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["sign"] == 7)
            .unwrap();
        assert_eq!(sco["antardashas"][0]["sign"], 6);
        assert_eq!(sco["antardashas"][11]["sign"], 7);
    }

    #[test]
    fn shoola_and_narayana_start_from_the_stronger_of_lagna_and_seventh() {
        let c = ramana();
        let v = shoola(&c, 5, BIRTH, None, YD);
        assert_eq!(v["start_sign"], 11);
        assert_eq!(v["direction"], "forward");
        let m = &v["mahadashas"][16];
        assert_eq!(m["sign"], 3);
        assert_eq!(&m["start_utc"].as_str().unwrap()[..7], "2023-12");
        let n = narayana(&c, 5, BIRTH, None, YD);
        assert_eq!(n["start_sign"], 11);
        let ari: Vec<&Value> = n["mahadashas"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|m| m["sign"] == 0 && m["cycle"] == 3)
            .collect();
        assert_eq!(&ari[0]["start_utc"].as_str().unwrap()[..7], "2024-12");
        assert_eq!(ari[0]["antardashas"][1]["sign"], 1);
    }

    #[test]
    fn antardashas_tile_their_mahadasha() {
        let c = ramana();
        for v in [
            chara(&c, 5, BIRTH, None, YD),
            narayana(&c, 5, BIRTH, None, YD),
            kalachakra(88.328_216, BIRTH, None, YD),
        ] {
            for m in v["mahadashas"].as_array().unwrap() {
                let a = m["antardashas"].as_array().unwrap();
                assert_eq!(a.last().unwrap()["end_utc"], m["end_utc"]);
            }
        }
    }

    #[test]
    fn kalachakra_balance_and_deha_jiva() {
        // Punarvasu pada 3 is the third Aśvinī-type row: Vṛṣabha … Mithuna, 83 years, deha Vṛṣabha, jīva Mithuna
        let v = kalachakra(88.328_216, BIRTH, None, YD);
        assert_eq!(v["paramayus"], 83.0);
        assert_eq!((v["deha"].as_u64(), v["jiva"].as_u64()), (Some(1), Some(2)));
        let bal = v["balance_at_birth_years"].as_f64().unwrap();
        assert!((bal - 83.0 * (1.0 - 0.498_465)).abs() < 0.01, "{bal}");
        // the Moon about half-way through the pada: birth falls in the sixth daśā (Dhanu)
        assert_eq!(v["mahadashas"][0]["sign"], 8);
    }

    #[test]
    fn shodashottari_starting_lord_counts_from_pushya() {
        let seq: Vec<String> = pack()
            .shodashottari
            .lords
            .iter()
            .map(|l| l.lord.clone())
            .collect();
        let nk = nakshatra_of(88.328_216);
        assert_eq!(seq[((nk.index + 27 - 7) % 27) % 8], "Jupiter");
        assert_eq!(seq[0], "Sun");
    }
}
