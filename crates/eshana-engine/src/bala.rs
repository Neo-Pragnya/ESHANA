//! Strength and special points (rule packs `vedic.bala` and `vedic.core`): sputa drishti, ṣaḍbala, bhāva bala,
//! viṃśopaka bala, and the special points and conditions shown beside a chart (yogi and avayogi, Indu lagna, dagdha
//! rāśis, 64th navāṃśa, 22nd drekkāṇa, lordships and functional nature, kartari, graha yuddha, element balance).
//!
//! Every formula is written out here with its convention named in `rulesets/bala.yaml`; the engine never guesses a
//! convention silently. All longitudes are sidereal.

use crate::jyotish::{dignity, house_from, lord_of, natural_relation, sign_of, VChart, SEVEN};
use crate::packs::{self, VashyaGroup};
use crate::vedic;
use eshana_core::{angular_diff, norm360};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Everything the time-dependent strengths need besides the chart.
#[derive(Debug, Clone, Default)]
pub struct Sky {
    /// Declination by graha, degrees.
    pub decl: BTreeMap<String, f64>,
    /// Ecliptic latitude by graha, degrees.
    pub ecl_lat: BTreeMap<String, f64>,
    /// Hour angle of the Sun, degrees in (-180, 180]; 0 at local apparent noon.
    pub sun_hour_angle: Option<f64>,
    /// Sunrise, sunset and next sunrise of the Vedic day (UT Julian days).
    pub day: Option<(f64, f64, f64)>,
    /// The instant (UT Julian day).
    pub jd: f64,
    /// Weekday lord of the Vedic day.
    pub vara_lord: Option<String>,
    /// Lord of the current horā.
    pub hora_lord: Option<String>,
    /// Abda (year) lord.
    pub abda_lord: Option<String>,
    /// Māsa (month) lord.
    pub masa_lord: Option<String>,
}

/// Sputa (degree-based) drishti of `from` (at `a`) onto a point at `b`, in virūpas (0-60).
#[must_use]
pub fn sputa_drishti(from: &str, a: f64, b: f64) -> f64 {
    let d = norm360(b - a);
    let base = match d {
        x if x < 30.0 => 0.0,
        x if x < 60.0 => (x - 30.0) / 2.0,
        x if x < 90.0 => x - 45.0,
        x if x < 120.0 => (120.0 - x) / 2.0 + 30.0,
        x if x < 150.0 => 150.0 - x,
        x if x < 180.0 => (x - 150.0) * 2.0,
        x if x < 300.0 => (300.0 - x) / 2.0,
        _ => 0.0,
    };
    let in_range = |lo: f64, hi: f64| d >= lo && d < hi;
    let extra = match from {
        "mars" if in_range(90.0, 120.0) || in_range(210.0, 240.0) => 15.0,
        "jupiter" if in_range(120.0, 150.0) || in_range(240.0, 270.0) => 30.0,
        "saturn" if in_range(60.0, 90.0) || in_range(270.0, 300.0) => 45.0,
        _ => 0.0,
    };
    (base + extra).min(60.0)
}

/// Graha drishti fraction by whole signs: 7th full; special aspects full; otherwise 3rd/10th 1/4, 5th/9th 1/2,
/// 4th/8th 3/4.
#[must_use]
pub fn drishti_fraction(from: &str, house: usize) -> Option<(u8, &'static str)> {
    let special = packs::core()
        .drishti
        .special
        .get(from)
        .is_some_and(|h| h.contains(&house));
    if house == 7 || special {
        return Some((4, "4/4"));
    }
    match house {
        4 | 8 => Some((3, "3/4")),
        5 | 9 => Some((2, "1/2")),
        3 | 10 => Some((1, "1/4")),
        _ => None,
    }
}

/// Aspects between the seven grahas: whole-sign fraction and sputa virūpas, for any chart (D1 or a varga).
#[must_use]
pub fn aspect_matrix(vc: &VChart) -> Value {
    let mut out = Map::new();
    for a in SEVEN.iter().chain(["rahu", "ketu"].iter()) {
        let Some(la) = vc.lon.get(*a) else { continue };
        let mut row = Map::new();
        for b in SEVEN.iter().chain(["rahu", "ketu"].iter()) {
            if a == b {
                continue;
            }
            let Some(lb) = vc.lon.get(*b) else { continue };
            let h = house_from(sign_of(*la), sign_of(*lb));
            let frac = drishti_fraction(a, h);
            let virupa = if SEVEN.contains(a) {
                sputa_drishti(a, *la, *lb)
            } else {
                0.0
            };
            if frac.is_some() || virupa > 0.0 {
                row.insert(
                    (*b).into(),
                    json!({"house": h, "fraction": frac.map(|f| f.1), "virupa": virupa}),
                );
            }
        }
        out.insert((*a).into(), Value::Object(row));
    }
    Value::Object(out)
}

/// Compound (pañcadhā) relationship of `a` towards `b` in a chart.
fn compound(vc: &VChart, a: &str, b: &str) -> &'static str {
    let (Some(sa), Some(sb)) = (
        vc.lon.get(a).map(|l| sign_of(*l)),
        vc.lon.get(b).map(|l| sign_of(*l)),
    ) else {
        return "neutral";
    };
    let temporary_friend = packs::core()
        .temporary_friend_houses
        .contains(&house_from(sa, sb));
    match (natural_relation(a, b), temporary_friend) {
        ("friend", true) => "great_friend",
        ("friend", false) | ("enemy", true) => "neutral",
        ("neutral", true) => "friend",
        ("neutral", false) => "enemy",
        _ => "great_enemy",
    }
}

/// Dignity of a graha in the sign a varga puts it in: `own` (own or exaltation sign), else its compound relation
/// to that sign's lord.
fn varga_dignity(vc: &VChart, g: &str, varga_sign: usize) -> &'static str {
    let c = packs::core();
    let exalted = c
        .dignity
        .get(g)
        .is_some_and(|d| d.exaltation[0] as usize == varga_sign);
    let lord = lord_of(varga_sign);
    if exalted || lord == g {
        "own"
    } else {
        compound(vc, g, lord)
    }
}

/// Viṃśopaka bala in the four schemes (out of 20) for the seven grahas.
#[must_use]
pub fn vimshopaka(vc: &VChart) -> Value {
    let b = packs::bala();
    let mut out = Map::new();
    for g in SEVEN {
        let Some(lon) = vc.lon.get(g) else { continue };
        let mut row = Map::new();
        for (name, scheme) in &b.vimshopaka.schemes {
            let mut total = 0.0;
            let mut parts = Map::new();
            for (d, w) in scheme {
                let Ok(s) = vedic::divisional_sign(*lon, *d) else {
                    continue;
                };
                let dig = varga_dignity(vc, g, s);
                let pts = b.vimshopaka.points[dig];
                total += w * pts / 20.0;
                parts.insert(format!("D{d}"), json!({"sign": s, "dignity": dig, "weight": w}));
            }
            row.insert(
                name.clone(),
                json!({"score": total, "max": 20.0, "vargas": parts}),
            );
        }
        out.insert(g.into(), Value::Object(row));
    }
    Value::Object(out)
}

fn motion_state(g: &str, speed: f64) -> &'static str {
    if speed < 0.0 {
        return "retrograde";
    }
    let mean = packs::bala().mean_daily_motion.get(g).copied().unwrap_or(1.0);
    match speed / mean {
        r if r < 0.1 => "stationary",
        r if r < 0.5 => "very_slow",
        r if r < 0.9 => "slow",
        r if r <= 1.1 => "mean",
        r if r <= 1.5 => "fast",
        _ => "very_fast",
    }
}

/// Ṣaḍbala of the seven grahas. Components that need the time of day or the place are `null` when unknown.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn shadbala(vc: &VChart, sky: &Sky) -> Value {
    let b = packs::bala();
    let core = packs::core();
    let lagna = vc.asc.map(sign_of);
    let sun = vc.lon.get("sun").copied();
    let moon = vc.lon.get("moon").copied();
    let elong = match (sun, moon) {
        (Some(s), Some(m)) => Some(norm360(m - s)),
        _ => None,
    };
    let paksha_raw = elong.map(|e| if e <= 180.0 { e / 3.0 } else { (360.0 - e) / 3.0 });
    let waxing = elong.is_some_and(|e| e < 180.0);
    let mut table = Map::new();
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for g in SEVEN {
        let Some(&lon) = vc.lon.get(g) else { continue };
        let s = sign_of(lon);
        let deg_in = norm360(lon) - 30.0 * s as f64;
        // ── sthāna bala ──
        let deb = core.dignity[g].debilitation;
        let uchcha = angular_diff(lon, deb[0] * 30.0 + deb[1]).abs() / 3.0;
        let mut sapta = 0.0;
        for d in &b.saptavarga_divisions {
            let vs = vedic::divisional_sign(lon, *d).unwrap_or(s);
            let key = if *d == 1 && dignity(g, lon) == "moolatrikona" {
                "moolatrikona"
            } else if lord_of(vs) == g {
                "own"
            } else {
                compound(vc, g, lord_of(vs))
            };
            sapta += b.saptavarga_points[key];
        }
        let female = b.odd_even_females.iter().any(|x| x == g);
        let nav = vedic::divisional_sign(lon, 9).unwrap_or(0);
        let odd_sign = s % 2 == 0;
        let odd_nav = nav % 2 == 0;
        let oja = 15.0 * (f64::from(u8::from(odd_sign != female)) + f64::from(u8::from(odd_nav != female)));
        let kendradi = lagna.map(|l| {
            let h = house_from(l, s);
            match h {
                1 | 4 | 7 | 10 => b.kendradi_points["kendra"],
                2 | 5 | 8 | 11 => b.kendradi_points["panapara"],
                _ => b.kendradi_points["apoklima"],
            }
        });
        let decan = ((deg_in / 10.0).floor() as usize).min(2);
        let drekkana = match b.drekkana_gender[g].as_str() {
            "male" if decan == 0 => 15.0,
            "neuter" if decan == 1 => 15.0,
            "female" if decan == 2 => 15.0,
            _ => 0.0,
        };
        let sthana = uchcha + sapta + oja + kendradi.unwrap_or(0.0) + drekkana;
        // ── dig bala ──
        let dig = match (vc.asc, vc.mc) {
            (Some(a), Some(m)) => crate::jyotish::dig_bala(g, lon, a, m),
            _ => None,
        };
        // ── kāla bala ──
        let natonnata = sky.sun_hour_angle.map(|h| {
            let from_noon = h.abs() / 180.0;
            if g == "mercury" {
                60.0
            } else if b.day_strong.iter().any(|x| x == g) {
                60.0 * (1.0 - from_noon)
            } else {
                60.0 * from_noon
            }
        });
        let paksha = paksha_raw.map(|p| {
            let v = if b.paksha_benefics.iter().any(|x| x == g) {
                p
            } else {
                60.0 - p
            };
            if g == "moon" {
                2.0 * v
            } else {
                v
            }
        });
        let tribhaga = sky.day.map(|(sr, ss, nsr)| {
            if b.tribhaga["always"].iter().any(|x| x == g) {
                return 60.0;
            }
            let (part, list) = if sky.jd < ss {
                (
                    ((sky.jd - sr) / ((ss - sr) / 3.0)).floor() as usize,
                    &b.tribhaga["day"],
                )
            } else {
                (
                    ((sky.jd - ss) / ((nsr - ss) / 3.0)).floor() as usize,
                    &b.tribhaga["night"],
                )
            };
            if list.get(part.min(2)).is_some_and(|x| x == g) {
                60.0
            } else {
                0.0
            }
        });
        let lord_pts = |l: &Option<String>, pts: f64| l.as_ref().map(|x| if x == g { pts } else { 0.0 });
        let abda = lord_pts(&sky.abda_lord, 15.0);
        let masa = lord_pts(&sky.masa_lord, 30.0);
        let vara = lord_pts(&sky.vara_lord, 45.0);
        let hora = lord_pts(&sky.hora_lord, 60.0);
        let ayana = sky.decl.get(g).map(|d| {
            let v = if g == "mercury" {
                (24.0 + d.abs()) * 60.0 / 48.0
            } else if b.ayana_south_strong.iter().any(|x| x == g) {
                (24.0 - d) * 60.0 / 48.0
            } else {
                (24.0 + d) * 60.0 / 48.0
            };
            if g == "sun" {
                2.0 * v
            } else {
                v
            }
        });
        let kala = [natonnata, paksha, tribhaga, abda, masa, vara, hora, ayana]
            .iter()
            .flatten()
            .sum::<f64>();
        // ── ceṣṭā bala ──
        let chesta = match g {
            "sun" => ayana,
            "moon" => paksha_raw,
            _ => vc.speed.get(g).map(|sp| b.chesta_points[motion_state(g, *sp)]),
        };
        // ── naisargika and dṛk bala ──
        let naisargika = core.naisargika_bala.get(g).copied().unwrap_or(0.0);
        let mut drik = 0.0;
        for p in SEVEN {
            if p == g {
                continue;
            }
            let Some(&lp) = vc.lon.get(p) else { continue };
            let v = sputa_drishti(p, lp, lon);
            let benefic = b.drik_benefics.iter().any(|x| x == p) || (p == "moon" && waxing);
            let malefic = b.drik_malefics.iter().any(|x| x == p) || (p == "moon" && !waxing);
            if benefic {
                drik += v;
            } else if malefic {
                drik -= v;
            }
        }
        drik /= 4.0;
        let total = sthana + dig.unwrap_or(0.0) + kala + chesta.unwrap_or(0.0) + naisargika + drik;
        let rupas = total / 60.0;
        let required = b.required_rupas[g];
        totals.insert(g.into(), total);
        table.insert(
            g.into(),
            json!({
                "sthana": {"uchcha": uchcha, "saptavargaja": sapta, "ojayugma": oja, "kendradi": kendradi,
                           "drekkana": drekkana, "total": sthana},
                "dig": dig,
                "kala": {"natonnata": natonnata, "paksha": paksha, "tribhaga": tribhaga, "abda": abda, "masa": masa,
                         "vara": vara, "hora": hora, "ayana": ayana, "yuddha": 0.0, "total": kala},
                "chesta": chesta, "chesta_state": vc.speed.get(g).map(|sp| motion_state(g, *sp)),
                "naisargika": naisargika, "drik": drik,
                "total_virupa": total, "rupas": rupas, "required_rupas": required, "ratio": rupas / required,
            }),
        );
    }
    let mut rank: Vec<(&String, &f64)> = totals.iter().collect();
    rank.sort_by(|a, b| {
        (b.1 / 1.0)
            .partial_cmp(&(a.1 / 1.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let ratio_rank: Vec<String> = {
        let mut v: Vec<(String, f64)> = table
            .iter()
            .map(|(k, x)| (k.clone(), x["ratio"].as_f64().unwrap_or(0.0)))
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().map(|x| x.0).collect()
    };
    json!({
        "grahas": table,
        "strongest_by_ratio": ratio_rank,
        "unit": "virupa (60 virupas = 1 rupa)",
        "note": "Components that need the time of day, the place or the Sun's ingresses are null when unknown; yuddha bala is reported as a condition, not added",
    })
}

/// Porphyry cusps (bhāva madhyas) from a sidereal Ascendant and Midheaven.
#[must_use]
pub fn bhava_madhyas(asc: f64, mc: f64) -> [f64; 12] {
    let ic = norm360(mc + 180.0);
    let dsc = norm360(asc + 180.0);
    let tri = |from: f64, to: f64, k: f64| norm360(from + norm360(to - from) * k / 3.0);
    [
        asc,
        tri(asc, ic, 1.0),
        tri(asc, ic, 2.0),
        ic,
        tri(ic, dsc, 1.0),
        tri(ic, dsc, 2.0),
        dsc,
        tri(dsc, mc, 1.0),
        tri(dsc, mc, 2.0),
        mc,
        tri(mc, asc, 1.0),
        tri(mc, asc, 2.0),
    ]
}

/// Bhāva bala for the twelve bhāvas: lord's ṣaḍbala, bhāva dig bala and bhāva drishti bala.
#[must_use]
pub fn bhava_bala(vc: &VChart, shad: &Value) -> Option<Value> {
    let (asc, mc) = (vc.asc?, vc.mc?);
    let b = packs::bala();
    let madhya = bhava_madhyas(asc, mc);
    let moon_waxing = match (vc.lon.get("sun"), vc.lon.get("moon")) {
        (Some(s), Some(m)) => norm360(m - s) < 180.0,
        _ => true,
    };
    let mut rows = vec![];
    for (i, &m) in madhya.iter().enumerate() {
        let h = i + 1;
        let s = sign_of(m);
        let lord = lord_of(s);
        let adhipati = shad["grahas"][lord]["total_virupa"].as_f64().unwrap_or(0.0);
        let nature = match &b.sign_nature[s] {
            VashyaGroup::One(n) => n.clone(),
            VashyaGroup::Split(v) => v[usize::from(norm360(m) % 30.0 >= 15.0)].clone(),
        };
        let strong = b.bhava_dig_strong_house[&nature];
        let dist = {
            let d = (h as i32 - strong as i32).rem_euclid(12);
            d.min(12 - d) as f64
        };
        let dig = (6.0 - dist) * 10.0;
        let mut drishti = 0.0;
        for p in SEVEN {
            let Some(&lp) = vc.lon.get(p) else { continue };
            let v = sputa_drishti(p, lp, m);
            let benefic = b.drik_benefics.iter().any(|x| x == p) || (p == "moon" && moon_waxing);
            drishti += if benefic { v } else { -v };
        }
        drishti /= 4.0;
        let total = adhipati + dig + drishti;
        rows.push(json!({"bhava": h, "madhya": m, "sign": s, "lord": lord, "adhipati": adhipati, "dig": dig,
                         "sign_nature": nature, "drishti": drishti, "total_virupa": total, "rupas": total / 60.0}));
    }
    Some(json!({"bhavas": rows, "madhya": "Porphyry (Sripati) bhava madhyas"}))
}

/// Special points and conditions: yogi and avayogi, Indu lagna, dagdha rāśis, 64th navāṃśa, 22nd drekkāṇa,
/// lordships and functional nature, kartari, graha yuddha and the element/modality balance.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn special_points(vc: &VChart) -> Value {
    let c = packs::core();
    let mut out = Map::new();
    let nak_lord = |l: f64| {
        vedic::DASHA_ORDER[vedic::nakshatra_of(l).index % 9]
            .0
            .to_lowercase()
    };
    if let (Some(&s), Some(&m)) = (vc.lon.get("sun"), vc.lon.get("moon")) {
        let yogi = norm360(s + m + c.yogi_offset_deg);
        let avayogi = norm360(yogi + c.avayogi_offset_deg);
        out.insert(
            "yogi".into(),
            json!({"point": yogi, "sign": sign_of(yogi), "nakshatra": vedic::nakshatra_of(yogi).name,
            "yogi_graha": nak_lord(yogi), "duplicate_yogi": lord_of(sign_of(yogi))}),
        );
        out.insert(
            "avayogi".into(),
            json!({"point": avayogi, "sign": sign_of(avayogi), "nakshatra": vedic::nakshatra_of(avayogi).name,
            "avayogi_graha": nak_lord(avayogi)}),
        );
        let elong = norm360(m - s);
        let tithi = ((elong / 12.0).floor() as u32).min(29) + 1;
        let in_paksha = if tithi > 15 { tithi - 15 } else { tithi };
        out.insert(
            "dagdha_rashis".into(),
            json!({"tithi": tithi, "tithi_in_paksha": in_paksha,
            "signs": c.dagdha_rashi_by_tithi.get(&in_paksha).cloned().unwrap_or_default()}),
        );
        let n64 = |l: f64| ((norm360(l) / vedic::PADA_SPAN).floor() as usize + 63) % 12;
        let d22 = |l: f64| {
            let k = ((norm360(l) / 10.0).floor() as usize + 21) % 36;
            (k + 1, (k / 3 + [0, 4, 8][k % 3]) % 12)
        };
        out.insert(
            "navamsha_64th_from_moon".into(),
            json!({"sign": n64(m), "lord": lord_of(n64(m))}),
        );
        let (k, s22) = d22(m);
        out.insert(
            "drekkana_22nd_from_moon".into(),
            json!({"drekkana": k, "sign": s22, "lord": lord_of(s22)}),
        );
        if let Some(a) = vc.asc {
            out.insert(
                "navamsha_64th_from_lagna".into(),
                json!({"sign": n64(a), "lord": lord_of(n64(a))}),
            );
        }
        if let Some(asc) = vc.asc {
            let lagna = sign_of(asc);
            let ninth = |from: usize| lord_of((from + 8) % 12);
            let kala = |g: &str| c.indu_kalas.get(g).copied().unwrap_or(0);
            let sum = kala(ninth(lagna)) + kala(ninth(sign_of(m)));
            let r = match sum % 12 {
                0 => 12,
                x => x as usize,
            };
            let indu = (sign_of(m) + r - 1) % 12;
            out.insert(
                "indu_lagna".into(),
                json!({"sign": indu, "kalas_sum": sum,
                "ninth_lord_from_lagna": ninth(lagna), "ninth_lord_from_moon": ninth(sign_of(m))}),
            );
        }
    }
    if let Some(asc) = vc.asc {
        let lagna = sign_of(asc);
        let d22 = ((norm360(asc) / 10.0).floor() as usize + 21) % 36;
        let d22_sign = (d22 / 3 + [0, 4, 8][d22 % 3]) % 12;
        out.insert(
            "drekkana_22nd_from_lagna".into(),
            json!({"drekkana": d22 + 1, "sign": d22_sign, "lord": lord_of(d22_sign)}),
        );
        let f = &c.functional;
        let mut lords = Map::new();
        for g in SEVEN {
            let owned: Vec<usize> = (1..=12).filter(|h| lord_of((lagna + h - 1) % 12) == g).collect();
            let has = |hs: &[usize]| owned.iter().any(|h| hs.contains(h));
            let nature = if owned.contains(&1) || (has(&[5, 9]) && !has(&[6, 8, 12])) {
                "benefic"
            } else if has(&f.malefic_houses) && !has(&f.benefic_houses) {
                "malefic"
            } else {
                "neutral"
            };
            lords.insert(
                g.into(),
                json!({"houses_owned": owned, "functional_nature": nature,
                "placed_in_house": vc.lon.get(g).map(|l| house_from(lagna, sign_of(*l)))}),
            );
        }
        out.insert("lordships".into(), json!({"grahas": lords,
            "rule": "lagna lord benefic; lords of 5 and 9 benefic unless they also own 6, 8 or 12; lords of only 3, 6, 8 or 11 malefic; others neutral"}));
    }
    // kartari: a point hemmed between malefics (papa) or benefics (shubha) in the 12th and 2nd signs from it
    let benefic = |g: &str| c.natural_benefics.iter().any(|x| x == g) && g != "moon";
    let malefic = |g: &str| c.natural_malefics.iter().any(|x| x == g);
    let mut points: Vec<(String, f64)> = vc
        .lon
        .iter()
        .filter(|(k, _)| c.grahas.contains(k))
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    if let Some(a) = vc.asc {
        points.push(("lagna".into(), a));
    }
    let mut kartari = Map::new();
    for (name, l) in &points {
        let s = sign_of(*l);
        let in_sign = |t: usize| -> Vec<&str> {
            vc.lon
                .iter()
                .filter(|(k, v)| c.grahas.contains(k) && *k != name && sign_of(**v) == t)
                .map(|(k, _)| k.as_str())
                .collect()
        };
        let (prev, next) = (in_sign((s + 11) % 12), in_sign((s + 1) % 12));
        let kind = if !prev.is_empty()
            && !next.is_empty()
            && prev.iter().all(|g| malefic(g))
            && next.iter().all(|g| malefic(g))
        {
            Some("papa_kartari")
        } else if !prev.is_empty()
            && !next.is_empty()
            && prev.iter().all(|g| benefic(g))
            && next.iter().all(|g| benefic(g))
        {
            Some("shubha_kartari")
        } else {
            None
        };
        if let Some(k) = kind {
            kartari.insert(name.clone(), json!({"kind": k, "twelfth": prev, "second": next}));
        }
    }
    out.insert("kartari".into(), Value::Object(kartari));
    // graha yuddha
    let warriors = ["mars", "mercury", "jupiter", "venus", "saturn"];
    let mut wars = vec![];
    for (i, a) in warriors.iter().enumerate() {
        for b in &warriors[i + 1..] {
            if let (Some(la), Some(lb)) = (vc.lon.get(*a), vc.lon.get(*b)) {
                let sep = angular_diff(*la, *lb).abs();
                if sep < 1.0 {
                    wars.push(json!({"grahas": [a, b], "separation_deg": sep}));
                }
            }
        }
    }
    out.insert("graha_yuddha".into(), json!(wars));
    // element and modality balance of the seven grahas and the lagna
    let mut el = [0u32; 4];
    let mut md = [0u32; 3];
    let mut grid: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, l) in points
        .iter()
        .filter(|(n, _)| SEVEN.contains(&n.as_str()) || n == "lagna")
    {
        let s = sign_of(*l);
        el[s % 4] += 1;
        md[s % 3] += 1;
        grid.entry(format!(
            "{}_{}",
            ["fire", "earth", "air", "water"][s % 4],
            ["movable", "fixed", "dual"][s % 3]
        ))
        .or_default()
        .push(name.clone());
    }
    out.insert(
        "balance".into(),
        json!({"elements": {"fire": el[0], "earth": el[1], "air": el[2], "water": el[3]},
        "modalities": {"movable": md[0], "fixed": md[1], "dual": md[2]}, "grid": grid}),
    );
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart() -> VChart {
        let mut c = VChart {
            asc: Some(151.9),
            mc: Some(61.7),
            ..Default::default()
        };
        for (g, l, sp) in [
            ("sun", 30.55, 0.96),
            ("moon", 271.89, 12.3),
            ("mars", 324.5, 0.74),
            ("mercury", 14.3, -0.13),
            ("jupiter", 75.8, 0.19),
            ("venus", 348.95, 1.14),
            ("saturn", 271.46, -0.017),
            ("rahu", 287.6, -0.05),
            ("ketu", 107.6, -0.05),
        ] {
            c.lon.insert(g.into(), l);
            c.speed.insert(g.into(), sp);
        }
        c
    }

    #[test]
    fn sputa_drishti_curve() {
        assert_eq!(sputa_drishti("sun", 0.0, 180.0), 60.0);
        assert_eq!(sputa_drishti("sun", 0.0, 90.0), 45.0);
        assert_eq!(sputa_drishti("sun", 0.0, 120.0), 30.0);
        assert_eq!(sputa_drishti("sun", 0.0, 20.0), 0.0);
        assert_eq!(sputa_drishti("saturn", 0.0, 75.0), 60.0); // 30 + 45 capped
        assert_eq!(sputa_drishti("jupiter", 0.0, 130.0), 50.0);
        assert_eq!(sputa_drishti("mars", 0.0, 100.0), 55.0);
    }

    #[test]
    fn vimshopaka_schemes_are_out_of_twenty() {
        let v = vimshopaka(&chart());
        for g in SEVEN {
            for s in ["shadvarga", "saptavarga", "dashavarga", "shodashavarga"] {
                let x = v[g][s]["score"].as_f64().unwrap();
                assert!((5.0..=20.0).contains(&x), "{g} {s} {x}");
            }
        }
    }

    #[test]
    fn shadbala_components_stay_in_range() {
        let sky = Sky {
            sun_hour_angle: Some(30.0),
            day: Some((0.0, 0.5, 1.0)),
            jd: 0.25,
            vara_lord: Some("mars".into()),
            hora_lord: Some("sun".into()),
            ..Default::default()
        };
        let s = shadbala(&chart(), &sky);
        for g in SEVEN {
            let x = &s["grahas"][g];
            let u = x["sthana"]["uchcha"].as_f64().unwrap();
            assert!((0.0..=60.0).contains(&u));
            assert!(x["dig"].as_f64().unwrap() <= 60.0);
            assert!(x["total_virupa"].as_f64().unwrap() > 0.0, "{g}");
        }
        // Venus exalted near its deep exaltation (27 Pisces): uchcha close to 60.
        assert!(s["grahas"]["venus"]["sthana"]["uchcha"].as_f64().unwrap() > 55.0);
        // Mars gets the vara bala (Tuesday).
        assert_eq!(s["grahas"]["mars"]["kala"]["vara"], 45.0);
    }

    #[test]
    fn bhava_bala_has_twelve_bhavas() {
        let c = chart();
        let s = shadbala(&c, &Sky::default());
        let b = bhava_bala(&c, &s).unwrap();
        assert_eq!(b["bhavas"].as_array().unwrap().len(), 12);
    }

    #[test]
    fn special_points_follow_their_rules() {
        let p = special_points(&chart());
        let yogi = p["yogi"]["point"].as_f64().unwrap();
        assert!((yogi - norm360(30.55 + 271.89 + 93.333_333_333_333_33)).abs() < 1e-9);
        // Kanya lagna: lagna lord Mercury is benefic, Mars owns 3 and 8 -> malefic.
        assert_eq!(
            p["lordships"]["grahas"]["mercury"]["functional_nature"],
            "benefic"
        );
        assert_eq!(p["lordships"]["grahas"]["mars"]["functional_nature"], "malefic");
        let el = &p["balance"]["elements"];
        let total: u64 = ["fire", "earth", "air", "water"]
            .iter()
            .map(|k| el[k].as_u64().unwrap())
            .sum();
        assert_eq!(total, 8);
    }
}
