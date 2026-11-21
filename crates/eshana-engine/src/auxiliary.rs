//! Auxiliary analysis: sahams, avakahaḍā chakra, male and female rāśis, mṛtyu bhāga, ghāta chakra, vedhasthāna,
//! jīva and deha, śodhana and piṇḍa.
//!
//! Every table lives in `rulesets/auxiliary.yaml` (ADR-012), with its source and convention written beside it; this
//! module only applies them. All longitudes are sidereal. Each item is its own attribute `vedic.aux.<name>`.

use crate::analysis::{AnalysisCtx, Derived};
use crate::jyotish::{self, house_from, lord_of, sign_of, VChart, NINE, SEVEN};
use crate::packs::{self, VashyaGroup};
use crate::{astro, bala, events, vedic};
use eshana_core::{norm360, Body, EshanaError, Result};
use eshana_time::Instant;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Rule id of every attribute from this module.
pub const RULE: &str = "vedic.aux@0.1";

const SIGNS: [&str; 12] = [
    "Mesha",
    "Vrishabha",
    "Mithuna",
    "Karka",
    "Simha",
    "Kanya",
    "Tula",
    "Vrischika",
    "Dhanu",
    "Makara",
    "Kumbha",
    "Meena",
];
const SIGNS_EN: [&str; 12] = [
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
const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

// ── the pack ─────────────────────────────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
struct SahamDef {
    id: String,
    name: String,
    a: String,
    b: String,
    c: String,
    reverse_at_night: bool,
    meaning: String,
    #[serde(default)]
    inferred: bool,
}

#[derive(Debug, Deserialize)]
struct GhataRow {
    month: String,
    tithis: Vec<u32>,
    weekday: usize,
    nakshatra: usize,
    yoga: String,
    karana: String,
    prahara: u32,
    moon_sign: usize,
}

#[derive(Debug, Deserialize)]
struct KalaGroup {
    name: String,
    direction: String,
    nakshatras: Vec<usize>,
    padas: Vec<Vec<usize>>,
}

#[derive(Debug, Deserialize)]
struct VargaDef {
    name: String,
    animal: String,
    sounds: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Names {
    yoni: BTreeMap<String, String>,
    gana: BTreeMap<String, String>,
    nadi: BTreeMap<String, String>,
    vashya: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Pack {
    id: String,
    version: String,
    sahams: Vec<SahamDef>,
    samartha_mars_lagna_lord_substitute: String,
    mrityu_bhaga_points: Vec<String>,
    mrityu_bhaga: Vec<Vec<f64>>,
    ghata: Vec<GhataRow>,
    vedha: BTreeMap<String, Vec<[usize; 2]>>,
    vedha_exempt: Vec<[String; 2]>,
    kalachakra_years: Vec<u32>,
    kalachakra: Vec<KalaGroup>,
    naamakshara: Vec<Vec<String>>,
    varga: Vec<VargaDef>,
    yunja: BTreeMap<String, Vec<usize>>,
    hansak: Vec<String>,
    paya_by_house: Vec<String>,
    paya_by_nakshatra: Vec<String>,
    karana_animal: BTreeMap<String, String>,
    names: Names,
    rashi_multiplier: Vec<u32>,
    graha_multiplier: BTreeMap<String, u32>,
    lagna_bav: BTreeMap<String, Vec<usize>>,
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| {
        serde_norway::from_str(include_str!("../rulesets/auxiliary.yaml"))
            .unwrap_or_else(|e| panic!("rule pack auxiliary.yaml is invalid: {e}"))
    })
}

// ── small helpers ────────────────────────────────────────────────────────────────────────────────────────────────
fn dms(x: f64) -> String {
    let x = norm360(x) % 30.0;
    let total = (x * 3600.0).round() as i64;
    format!("{:02}:{:02}:{:02}", total / 3600, (total / 60) % 60, total % 60)
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

fn place(lon: f64) -> Value {
    let s = sign_of(lon);
    json!({"longitude": norm360(lon), "sign": s, "sign_name": SIGNS[s], "degree_in_sign": norm360(lon) - 30.0 * s as f64,
           "dms": dms(lon), "lord": lord_of(s)})
}

/// True when `c` lies on the zodiacal arc that runs from `b` forward to `a`.
#[must_use]
pub fn between(c: f64, b: f64, a: f64) -> bool {
    norm360(c - b) <= norm360(a - b)
}

/// A − B + C with the Tajika +30° rule: 30° are added when C is not on the arc from B forward to A.
/// Returns the longitude and whether 30° were added.
#[must_use]
pub fn saham_value(a: f64, b: f64, c: f64) -> (f64, bool) {
    let add = !between(c, b, a);
    (norm360(a - b + c + if add { 30.0 } else { 0.0 }), add)
}

/// Whether the birth is by day, and the rule used.
fn birth_by_day(x: &AnalysisCtx) -> (bool, &'static str) {
    if let Some(pc) = x.panchanga {
        if let (Some(sr), Some(ss)) = (pc["sunrise"]["jd_ut"].as_f64(), pc["sunset"]["jd_ut"].as_f64()) {
            let jd = x.inst.jd_ut;
            return (
                jd >= sr && jd < ss,
                "day = between the sunrise and sunset of the birth's pañcāṅga day",
            );
        }
    }
    let by_day = match (x.vc.lon.get("sun"), x.vc.asc) {
        (Some(s), Some(a)) => norm360(s - a) >= 180.0,
        _ => true,
    };
    (
        by_day,
        "day = the Sun above the horizon by ecliptic houses (no sunrise was available)",
    )
}

// ── 1. sahams ────────────────────────────────────────────────────────────────────────────────────────────────────
/// Every saham of the pack for a chart. `by_day` decides the night reversal; cusps are Sripati bhāva madhyas when
/// the Midheaven is known, equal houses from the lagna degree otherwise.
///
/// # Errors
/// [`EshanaError::Input`] without the lagna or one of the seven grahas.
pub fn sahams(vc: &VChart, by_day: bool) -> Result<Value> {
    let p = pack();
    let asc = vc
        .asc
        .ok_or_else(|| EshanaError::Input("sahams need the lagna".into()))?;
    let (cusps, cusp_rule) = match vc.mc {
        Some(mc) => (
            bala::bhava_madhyas(asc, mc),
            "Sripati bhāva madhyas (Porphyry trisection of the sidereal quadrants)",
        ),
        None => (
            std::array::from_fn(|i| norm360(asc + 30.0 * i as f64)),
            "equal houses from the lagna degree (no Midheaven)",
        ),
    };
    let mut pts: BTreeMap<String, f64> = BTreeMap::new();
    for g in SEVEN {
        let l = *vc
            .lon
            .get(g)
            .ok_or_else(|| EshanaError::Input(format!("sahams need {g}")))?;
        pts.insert(g.into(), l);
    }
    pts.insert("lagna".into(), asc);
    pts.insert("cancer15".into(), 105.0);
    for (i, c) in cusps.iter().enumerate() {
        pts.insert(format!("cusp{}", i + 1), *c);
        pts.insert(format!("lord{}", i + 1), pts[lord_of(sign_of(*c))]);
    }
    let lagna_lord = lord_of(sign_of(asc));
    pts.insert("lagna_lord".into(), pts[lagna_lord]);
    let lum = if by_day { "sun" } else { "moon" };
    pts.insert("luminary".into(), pts[lum]);
    let lum_lord = lord_of(sign_of(pts[lum]));
    pts.insert("luminary_lord".into(), pts[lum_lord]);
    let label = |k: &str| -> String {
        match k {
            "lagna_lord" => format!("Lagna lord ({})", cap(lagna_lord)),
            "luminary" => cap(lum),
            "luminary_lord" => format!("lord of the {lum}'s sign ({})", cap(lum_lord)),
            "cancer15" => "Karka 15°".into(),
            k if k.starts_with("cusp") => format!("{} bhāva madhya", ordinal(&k[4..])),
            k if k.starts_with("lord") => format!(
                "{} lord ({})",
                ordinal(&k[4..]),
                cap(lord_of(sign_of(cusps[k[4..].parse::<usize>().unwrap_or(1) - 1])))
            ),
            k => cap(k),
        }
    };
    let mut rows = vec![];
    for d in &p.sahams {
        let (mut a, mut b) = (d.a.clone(), d.b.clone());
        let mut special = None;
        if d.id == "samartha" && lagna_lord == "mars" {
            a.clone_from(&p.samartha_mars_lagna_lord_substitute);
            b = "mars".into();
            special = Some("Mars is the lagna lord, so Jupiter takes its place");
        }
        let reversed = d.reverse_at_night && !by_day;
        if reversed {
            std::mem::swap(&mut a, &mut b);
        }
        let get = |k: &str| {
            pts.get(k)
                .copied()
                .ok_or_else(|| EshanaError::Rules(format!("auxiliary.yaml: unknown saham point {k}")))
        };
        let (va, vb, vcc) = (get(&a)?, get(&b)?, get(&d.c)?);
        let (lon, plus30) = saham_value(va, vb, vcc);
        pts.insert(d.id.clone(), lon);
        let mut row = place(lon);
        row["id"] = json!(d.id);
        row["name"] = json!(d.name);
        row["formula"] = json!(format!(
            "{} − {} + {}{}",
            label(&a),
            label(&b),
            label(&d.c),
            if plus30 { " + 30°" } else { "" }
        ));
        row["terms"] = json!({"a": a, "b": b, "c": d.c});
        row["reversed_at_night"] = json!(reversed);
        row["plus_30"] = json!(plus30);
        row["meaning"] = json!(d.meaning);
        row["day_form_inferred"] = json!(d.inferred);
        if let Some(sp) = special {
            row["note"] = json!(sp);
        }
        rows.push(row);
    }
    Ok(json!({
        "sahams": rows,
        "birth_in": if by_day { "day" } else { "night" },
        "cusps": cusp_rule,
        "pack": format!("{}@{}", p.id, p.version),
        "rule":"A − B + C; +30° when C is not on the zodiacal arc from B to A; A and B swap for a night birth where the pack says so (Tajika Nīlakaṇṭhī as reproduced by a published reference chart)",
    }))
}

fn ordinal(n: &str) -> String {
    match n {
        "1" => "1st".into(),
        "2" => "2nd".into(),
        "3" => "3rd".into(),
        n => format!("{n}th"),
    }
}

// ── 2. avakahaḍā ─────────────────────────────────────────────────────────────────────────────────────────────────
fn varga_of(syllable: &str) -> &'static VargaDef {
    let p = pack();
    p.varga
        .iter()
        .find(|v| v.sounds.iter().any(|s| syllable.starts_with(s.as_str())))
        .unwrap_or(&p.varga[0])
}

/// The avakahaḍā chakra. `panchanga` supplies the yoga and karaṇa; `tropical_sun` the Western sun sign.
#[must_use]
pub fn avakahada(vc: &VChart, panchanga: Option<&Value>, tropical_sun: Option<f64>) -> Option<Value> {
    let p = pack();
    let m = packs::matching();
    let moon = *vc.lon.get("moon")?;
    let ms = sign_of(moon);
    let nak = vedic::nakshatra_of(moon);
    let deg = norm360(moon) - 30.0 * ms as f64;
    let vashya = match &m.vashya_groups[ms] {
        VashyaGroup::One(g) => g.clone(),
        VashyaGroup::Split(v) => v[usize::from(deg >= 15.0)].clone(),
    };
    let syllable = p.naamakshara[nak.index][nak.pada - 1].clone();
    let varga = varga_of(&syllable);
    let yunja = p
        .yunja
        .iter()
        .find(|(_, v)| v.contains(&nak.index))
        .map(|(k, _)| k.clone());
    let name = |map: &BTreeMap<String, String>, k: &str| map.get(k).cloned().unwrap_or_else(|| cap(k));
    let yoni = &m.yoni[nak.index];
    let gana = &m.gana[nak.index];
    let nadi = &m.nadi[nak.index];
    let varna = m.varna_names[&m.varna_rank[ms]].clone();
    let karana = panchanga.and_then(|pc| pc["karana"]["name"].as_str());
    let yoga = panchanga.and_then(|pc| pc["yoga"]["name"].as_str());
    let asc = vc.asc.map(|a| {
        let s = sign_of(a);
        json!({"sign": s, "sign_name": SIGNS[s], "lord": lord_of(s)})
    });
    let paya_sign = vc
        .asc
        .map(|a| p.paya_by_house[house_from(sign_of(a), ms) - 1].clone());
    Some(json!({
        "ascendant": asc,
        "moon_sign": {"sign": ms, "sign_name": SIGNS[ms], "lord": lord_of(ms)},
        "nakshatra": {"index": nak.index, "name": nak.name, "pada": nak.pada, "lord": nak.lord},
        "yoga": yoga,
        "karana": karana.map(|k| json!({"name": k, "animal": p.karana_animal.get(k)})),
        "yoni": {"key": yoni, "name": name(&p.names.yoni, yoni)},
        "gana": name(&p.names.gana, gana),
        "nadi": name(&p.names.nadi, nadi),
        "varna": varna,
        "vashya": name(&p.names.vashya, &vashya),
        "varga": {"name": varga.name, "animal": varga.animal},
        "yunja": yunja,
        "hansak": p.hansak[ms],
        "naamakshara": syllable,
        "paya": {"by_sign": paya_sign, "by_nakshatra": p.paya_by_nakshatra[nak.index]},
        "sunsign_western": tropical_sun.map(|t| SIGNS_EN[sign_of(t)]),
        "conventions": {
            "from_moon": "nakshatra, pada, yoni, gana, nadi (matching pack); varna, vashya, hansak (Moon sign); naamakshara = syllable of the Moon's pada; varga = the syllable's first sound",
            "paya": "by sign: the Moon's house from the lagna (1, 6, 11 gold; 2, 5, 9 silver; 3, 7, 10 copper; 4, 8, 12 iron); by nakshatra: the almanac grouping in auxiliary.yaml",
            "sunsign_western": "tropical Sun (sidereal + ayanāṃśa + nutation)",
        },
    }))
}

// ── 3. male and female rāśis ─────────────────────────────────────────────────────────────────────────────────────
/// Which points are in odd (male) and even (female) signs of the rāśi chart.
#[must_use]
pub fn male_female(vc: &VChart) -> Value {
    let mut male = vec![];
    let mut female = vec![];
    let mut rows = vec![];
    let mut pts: Vec<(&str, f64)> = vec![];
    if let Some(a) = vc.asc {
        pts.push(("lagna", a));
    }
    for g in NINE {
        if let Some(l) = vc.lon.get(g) {
            pts.push((g, *l));
        }
    }
    for (id, l) in pts {
        let s = sign_of(l);
        let is_male = s % 2 == 0; // Mesha (index 0) is the first, odd sign
        if is_male {
            male.push(id)
        } else {
            female.push(id)
        }
        rows.push(json!({"id": id, "sign": s, "sign_name": SIGNS[s], "gender": if is_male { "male" } else { "female" }}));
    }
    json!({"male": male, "female": female, "points": rows,
           "rule": "odd signs (Mesha, Mithuna, Simha, Tula, Dhanu, Kumbha) are male; even signs female (D1)"})
}

// ── 4. mṛtyu bhāga ───────────────────────────────────────────────────────────────────────────────────────────────
/// The mṛtyu bhāga degree of each point's sign and the point's distance from it.
#[must_use]
pub fn mrityu_bhaga(vc: &VChart) -> Value {
    let p = pack();
    let mut rows = vec![];
    for (col, id) in p.mrityu_bhaga_points.iter().enumerate() {
        let lon = if id == "lagna" {
            vc.asc
        } else {
            vc.lon.get(id.as_str()).copied()
        };
        let Some(lon) = lon else { continue };
        let s = sign_of(lon);
        let mb = p.mrityu_bhaga[s][col];
        let d = norm360(lon) - 30.0 * s as f64;
        let dist = (d - mb).abs();
        rows.push(
            json!({"id": id, "sign": s, "sign_name": SIGNS[s], "degree_in_sign": d, "mrityu_bhaga_deg": mb,
                         "distance_deg": dist, "distance_dms": dms(dist), "within_1_deg": dist < 1.0}),
        );
    }
    json!({"points": rows, "source": "classical mṛtyu-bhāga table as reprinted with Phaladīpikā and Jātaka Pārijāta (auxiliary.yaml)",
           "rule": "distance = |degree in sign − mṛtyu bhāga degree|; within 1° is flagged"})
}

// ── 5. ghāta chakra ──────────────────────────────────────────────────────────────────────────────────────────────
/// The ghāta (ghātaka) chakra row for the natal Moon sign, and which birth elements coincide with it.
#[must_use]
pub fn ghata(vc: &VChart, panchanga: Option<&Value>) -> Option<Value> {
    let p = pack();
    let ms = sign_of(*vc.lon.get("moon")?);
    let r = &p.ghata[ms];
    let mut coincide = vec![];
    if let Some(pc) = panchanga {
        if let Some(t) = pc["tithi"]["index"].as_u64() {
            let in_paksha = ((t as u32).saturating_sub(1)) % 15 + 1;
            if r.tithis.contains(&in_paksha) {
                coincide.push("tithi");
            }
        }
        if pc["vara"]["index"].as_u64() == Some(r.weekday as u64) {
            coincide.push("weekday");
        }
        if pc["nakshatra"]["index"].as_u64() == Some(r.nakshatra as u64) {
            coincide.push("nakshatra");
        }
        if pc["yoga"]["name"].as_str() == Some(r.yoga.as_str()) {
            coincide.push("yoga");
        }
        if pc["karana"]["name"].as_str() == Some(r.karana.as_str()) {
            coincide.push("karana");
        }
    }
    Some(json!({
        "natal_moon_sign": ms, "natal_moon_sign_name": SIGNS[ms],
        "month": r.month, "tithis": r.tithis, "weekday": r.weekday, "weekday_name": WEEKDAYS[r.weekday],
        "nakshatra": r.nakshatra, "nakshatra_name": vedic::NAKSHATRAS[r.nakshatra], "yoga": r.yoga,
        "karana": r.karana, "karana_animal": p.karana_animal.get(&r.karana), "prahara": r.prahara,
        "moon_sign": r.moon_sign, "moon_sign_name": SIGNS[r.moon_sign],
        "birth_coincides_with": coincide,
        "rule": "ghātaka chakra by natal Moon sign (muhurta almanac table in auxiliary.yaml); tithis are counted within the pakṣa",
    }))
}

// ── 6. vedhasthāna ───────────────────────────────────────────────────────────────────────────────────────────────
/// Gochara vedha of the seven grahas at longitudes `lon`, houses counted from the natal Moon sign.
#[must_use]
pub fn vedha(moon_sign: usize, lon: &BTreeMap<String, f64>) -> Vec<Value> {
    let p = pack();
    let exempt = |a: &str, b: &str| {
        p.vedha_exempt
            .iter()
            .any(|[x, y]| (x == a && y == b) || (x == b && y == a))
    };
    let house: BTreeMap<&str, usize> = SEVEN
        .iter()
        .filter_map(|g| lon.get(*g).map(|l| (*g, house_from(moon_sign, sign_of(*l)))))
        .collect();
    let ab = |g: &str| {
        crate::jyotish::SEVEN
            .iter()
            .position(|x| *x == g)
            .map_or("", |i| ["Su", "Mo", "Ma", "Me", "Ju", "Ve", "Sa"][i])
    };
    let mut rows = vec![];
    for g in SEVEN {
        let Some(&h) = house.get(g) else { continue };
        let pair = p.vedha.get(g).and_then(|v| v.iter().find(|[good, _]| *good == h));
        let (fav, vh) = (pair.is_some(), pair.map(|[_, v]| *v));
        let by: Vec<&str> = vh
            .map(|vh| {
                SEVEN
                    .iter()
                    .copied()
                    .filter(|o| *o != g && house.get(o) == Some(&vh) && !exempt(g, o))
                    .collect()
            })
            .unwrap_or_default();
        let state = match (fav, by.is_empty()) {
            (true, true) => "auspicious",
            (true, false) => "auspicious, obstructed",
            _ => "inauspicious",
        };
        let mut summary = format!("{} {} {}", ab(g), h, if fav { "Ausp" } else { "Inausp" });
        if let (Some(vh), false) = (vh, by.is_empty()) {
            summary += &format!(
                ", X by {} in {vh}",
                by.iter().map(|o| ab(o)).collect::<Vec<_>>().join(",")
            );
        }
        let fav_houses: Vec<usize> = p
            .vedha
            .get(g)
            .map(|v| v.iter().map(|[a, _]| *a).collect())
            .unwrap_or_default();
        rows.push(
            json!({"graha": g, "sign": sign_of(lon[g]), "house_from_moon": h, "favourable": fav,
                   "favourable_houses": fav_houses,
                         "vedha_house": vh, "obstructed_by": by, "state": state, "summary": summary}),
        );
    }
    rows
}

const BODIES: [(&str, Body); 7] = [
    ("sun", Body::Sun),
    ("moon", Body::Moon),
    ("mars", Body::Mars),
    ("mercury", Body::Mercury),
    ("jupiter", Body::Jupiter),
    ("venus", Body::Venus),
    ("saturn", Body::Saturn),
];

// ── 7. jīva and deha ─────────────────────────────────────────────────────────────────────────────────────────────
/// Kālachakra deha and jīva signs of the Moon's pada, with the pada's nine signs and paramāyus.
#[must_use]
pub fn jiva_deha(moon: f64) -> Option<Value> {
    let p = pack();
    let nak = vedic::nakshatra_of(moon);
    let g = p.kalachakra.iter().find(|g| g.nakshatras.contains(&nak.index))?;
    let seq = &g.padas[nak.pada - 1];
    let years: Vec<u32> = seq.iter().map(|s| p.kalachakra_years[*s]).collect();
    let (deha, jiva) = (seq[0], seq[seq.len() - 1]);
    Some(json!({
        "nakshatra": nak.name, "pada": nak.pada, "group": g.name, "direction": g.direction,
        "deha": {"sign": deha, "sign_name": SIGNS[deha], "lord": lord_of(deha)},
        "jiva": {"sign": jiva, "sign_name": SIGNS[jiva], "lord": lord_of(jiva)},
        "signs": seq, "sign_names": seq.iter().map(|s| SIGNS[*s]).collect::<Vec<_>>(), "years": years,
        "paramayus_years": years.iter().sum::<u32>(),
        "rule": "Kālachakra (BPHS ch. 46): the Moon's pada owns nine signs; the first is the deha, the last the jīva; paramāyus = the sum of the nine signs' daśā years (Mesha 7, Vṛṣabha 16, Mithuna 9, Karka 21, Siṃha 5, Kanyā 9, Tulā 16, Vṛścika 7, Dhanu 10, Makara 4, Kumbha 4, Mīna 10)",
    }))
}

// ── 8. śodhana and piṇḍa ─────────────────────────────────────────────────────────────────────────────────────────
/// Trikoṇa śodhana: in each triplet of trine signs subtract the smallest count.
#[must_use]
pub fn trikona(b: &[u32; 12]) -> [u32; 12] {
    let mut r = *b;
    for k in 0..4 {
        let m = r[k].min(r[k + 4]).min(r[k + 8]);
        for i in [k, k + 4, k + 8] {
            r[i] -= m;
        }
    }
    r
}

/// Ekādhipatya śodhana on the five pairs of signs with one lord; `occ` marks signs with a graha or the lagna.
#[must_use]
pub fn ekadhipatya(b: &[u32; 12], occ: &[bool; 12]) -> [u32; 12] {
    let mut r = *b;
    for (x, y) in [(0, 7), (1, 6), (2, 5), (8, 11), (9, 10)] {
        let (bx, by) = (r[x], r[y]);
        if bx == 0 || by == 0 || (occ[x] && occ[y]) {
            continue;
        }
        if !occ[x] && !occ[y] {
            let v = if bx == by { 0 } else { bx.min(by) };
            r[x] = v;
            r[y] = v;
            continue;
        }
        let (o, e) = if occ[x] { (x, y) } else { (y, x) };
        r[e] = if r[e] <= r[o] { 0 } else { r[o] };
    }
    r
}

/// Śodhana and the three piṇḍas for the seven grahas' aṣṭakavargas and the lagna's.
#[must_use]
pub fn shodhana(vc: &VChart) -> Option<Value> {
    let p = pack();
    let av = jyotish::ashtakavarga(vc)?;
    let asc = vc.asc?;
    let mut sign: BTreeMap<&str, usize> = BTreeMap::new();
    for g in SEVEN {
        sign.insert(g, sign_of(*vc.lon.get(g)?));
    }
    sign.insert("lagna", sign_of(asc));
    let mut occ = [false; 12];
    for s in sign.values() {
        occ[*s] = true;
    }
    let mut bavs: Vec<(&str, [u32; 12])> = vec![];
    for g in SEVEN {
        let arr = av["bhinnashtakavarga"][g]["bindus"].as_array()?;
        let mut b = [0u32; 12];
        for (i, v) in arr.iter().enumerate().take(12) {
            b[i] = v.as_u64()? as u32;
        }
        bavs.push((g, b));
    }
    let mut lb = [0u32; 12];
    for (contrib, houses) in &p.lagna_bav {
        let from = *sign.get(contrib.as_str())?;
        for h in houses {
            lb[(from + h - 1) % 12] += 1;
        }
    }
    bavs.push(("lagna", lb));
    let mut rows = serde_json::Map::new();
    for (g, b) in bavs {
        let t = trikona(&b);
        let e = ekadhipatya(&t, &occ);
        let rashi: u32 = (0..12).map(|i| e[i] * p.rashi_multiplier[i]).sum();
        let graha: u32 = SEVEN.iter().map(|x| e[sign[x]] * p.graha_multiplier[*x]).sum();
        rows.insert(
            g.into(),
            json!({"bindus": b, "after_trikona": t, "after_ekadhipatya": e, "rashi_pinda": rashi,
                   "graha_pinda": graha, "shodhya_pinda": rashi + graha}),
        );
    }
    Some(json!({
        "by_chart": rows, "order": ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "lagna"],
        "rashi_multiplier": p.rashi_multiplier, "graha_multiplier": p.graha_multiplier,
        "sign_order": "index 0 = Mesha (Aries)",
        "rule": "BPHS ch. 67-69: trikoṇa then ekādhipatya śodhana (the lagna counts as an occupant), then rāśi piṇḍa = Σ count × rāśi multiplier, graha piṇḍa = Σ count in each graha's sign × graha multiplier, śodhya piṇḍa = their sum",
    }))
}

// ── compute ──────────────────────────────────────────────────────────────────────────────────────────────────────
/// Compute this module's attributes for a chart.
///
/// # Errors
/// Propagates ephemeris errors (vedhasthāna at the reference instant).
pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>> {
    let vc = x.vc;
    let mut out = vec![];
    let mut push = |id: &str, value: Value, from: &[&str]| {
        out.push(Derived {
            id: format!("vedic.aux.{id}"),
            rule: RULE.into(),
            value,
            from: from.iter().map(|s| (*s).to_string()).collect(),
        });
    };
    let (by_day, day_rule) = birth_by_day(x);
    if vc.asc.is_some() && SEVEN.iter().all(|g| vc.lon.contains_key(*g)) {
        let mut v = sahams(vc, by_day)?;
        v["day_night_rule"] = json!(day_rule);
        push("sahams", v, &["positions", "houses", "panchanga"]);
    }
    let t = x.inst.t_tt();
    let tropical_sun = vc.lon.get("sun").map(|s| {
        norm360(s + astro::ayanamsha_deg(x.ayanamsha, t) + eshana_ephemeris::frames::nutation_deg(t).0)
    });
    if let Some(v) = avakahada(vc, x.panchanga, tropical_sun) {
        push("avakahada", v, &["moon", "houses", "panchanga"]);
    }
    push("male_female", male_female(vc), &["positions", "houses"]);
    push("mrityu_bhaga", mrityu_bhaga(vc), &["positions", "houses"]);
    if let Some(v) = ghata(vc, x.panchanga) {
        push("ghata_chakra", v, &["moon", "panchanga"]);
    }
    if let Some(moon) = vc.lon.get("moon") {
        let ms = sign_of(*moon);
        let natal = vedha(ms, &vc.lon);
        let transit = match x.reference_jd {
            Some(jd) => {
                let mut lon = BTreeMap::new();
                for (g, b) in BODIES {
                    lon.insert(
                        g.to_string(),
                        events::lon_at(x.provider, b, jd, Some(x.ayanamsha))?,
                    );
                }
                json!({"at_utc": Instant::from_jd_utc(jd).utc, "grahas": vedha(ms, &lon)})
            }
            None => Value::Null,
        };
        push(
            "vedhasthana",
            json!({"natal_moon_sign": ms, "natal_moon_sign_name": SIGNS[ms], "natal": natal, "transit": transit,
                   "rule": "houses from the natal Moon; favourable houses and their vedha houses from the gochara table (auxiliary.yaml); no vedha between the Sun and Saturn or the Moon and Mercury; the natal set places the birth grahas (as a published reference chart prints it), the transit set the grahas at the reference instant"}),
            &["positions", "reference_utc"],
        );
        if let Some(v) = jiva_deha(*moon) {
            push("jiva_deha", v, &["moon"]);
        }
    }
    if let Some(v) = shodhana(vc) {
        push("shodhana", v, &["positions", "houses", "ashtakavarga"]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lon(sign: usize, d: f64, m: f64, s: f64) -> f64 {
        sign as f64 * 30.0 + d + m / 60.0 + s / 3600.0
    }

    /// The Ramana chart with the longitudes a published reference chart prints (the same chart the accuracy suite
    /// checks), so the worked examples test the rules, not the ephemeris.
    fn ramana() -> VChart {
        let mut c = VChart {
            asc: Some(lon(5, 28.0, 43.0, 11.0)),
            mc: Some(lon(2, 26.0, 58.0, 50.0) + (lon(5, 28.0, 43.0, 11.0) - 178.719_725)),
            ..Default::default()
        };
        for (g, l) in [
            ("sun", lon(8, 15.0, 35.0, 35.0)),
            ("moon", lon(2, 28.0, 19.0, 55.0)),
            ("mars", lon(0, 21.0, 58.0, 7.0)),
            ("mercury", lon(7, 23.0, 6.0, 27.0)),
            ("jupiter", lon(10, 16.0, 29.0, 10.0)),
            ("venus", lon(7, 0.0, 28.0, 34.0)),
            ("saturn", lon(11, 17.0, 3.0, 15.0)),
            ("rahu", lon(8, 23.0, 20.0, 32.0)),
            ("ketu", lon(2, 23.0, 20.0, 32.0)),
        ] {
            c.lon.insert(g.into(), l);
        }
        c
    }

    #[test]
    fn pack_tables_have_their_fixed_shapes() {
        let p = pack();
        assert_eq!(p.sahams.len(), 37);
        assert_eq!(p.mrityu_bhaga.len(), 12);
        assert!(p
            .mrityu_bhaga
            .iter()
            .all(|r| r.len() == p.mrityu_bhaga_points.len()));
        assert_eq!(p.ghata.len(), 12);
        assert_eq!(p.naamakshara.len(), 27);
        assert!(p.naamakshara.iter().all(|r| r.len() == 4));
        let covered: usize = p.kalachakra.iter().map(|g| g.nakshatras.len()).sum();
        assert_eq!(covered, 27);
        let yunja: usize = p.yunja.values().map(Vec::len).sum();
        assert_eq!(yunja, 27);
        assert_eq!(p.lagna_bav.values().map(Vec::len).sum::<usize>(), 49);
        // paramayus of the four padas of each group: 100, 85, 83, 86 (savya) and 86, 83, 85, 100 (apasavya)
        for g in &p.kalachakra {
            let spans: Vec<u32> = g
                .padas
                .iter()
                .map(|s| s.iter().map(|i| p.kalachakra_years[*i]).sum())
                .collect();
            let want = if g.direction == "savya" {
                [100, 85, 83, 86]
            } else {
                [86, 83, 85, 100]
            };
            assert_eq!(spans, want, "{}", g.name);
        }
    }

    #[test]
    fn plus_30_rule() {
        // C on the arc from B to A: no addition
        assert_eq!(saham_value(100.0, 10.0, 50.0), (140.0, false));
        // C outside the arc: +30
        let (v, add) = saham_value(100.0, 10.0, 200.0);
        assert!(add && (v - 320.0).abs() < 1e-9);
    }

    #[test]
    fn sahams_match_the_reference_night_chart() {
        let v = sahams(&ramana(), false).unwrap();
        let rows = v["sahams"].as_array().unwrap();
        let find = |id: &str| {
            rows.iter().find(|r| r["id"] == id).unwrap()["longitude"]
                .as_f64()
                .unwrap()
        };
        // printed by a published reference chart (sign, d:m:s)
        let printed = "punya 11 15:59:01, vidya 1 11:27:21, yasa 7 28:13:03, mitra 8 29:58:26, mahatmya 8 04:42:17, \
            asha 1 12:08:10, samartha 0 29:51:31, bhratru 4 28:08:46, gaurava 4 29:13:20, pitru 2 27:15:12, \
            raja 2 27:15:12, matru 10 00:52:00, putra 11 10:33:47, jeeva 4 28:08:46, karma 0 29:51:31, \
            roga 10 09:59:22, kali 9 04:12:09, sastra 9 23:40:52, bandhu 10 23:29:53, mrityu 9 16:52:14, \
            paradesa 1 25:48:15, artha 5 26:23:02, paradara 4 13:36:10, vanik 2 03:56:30, karyasiddhi 4 11:50:16, \
            vivaha 1 12:08:10, santapa 7 16:52:14, sraddha 0 07:13:38, preeti 8 24:11:31, jadya 6 18:11:54, \
            vyapara 11 27:34:52, satru 4 23:48:39, jalapathana 2 00:46:46, bandhana 6 29:47:45, \
            apamrityu 5 22:32:54, prasava 3 05:20:28, labha 7 27:57:04";
        let mut n = 0;
        for item in printed.split(", ") {
            let f: Vec<&str> = item.split_whitespace().collect();
            let d: Vec<f64> = f[2].split(':').map(|x| x.parse().unwrap()).collect();
            let (id, want) = (f[0], lon(f[1].parse().unwrap(), d[0], d[1], d[2]));
            n += 1;
            let d = (find(id) - want + 540.0) % 360.0 - 180.0;
            assert!(d.abs() * 60.0 < 1.0, "{id}: {:+.2}′", d * 60.0);
        }
        assert_eq!(n, 37);
        assert_eq!(rows.len(), 37);
    }

    #[test]
    fn mrityu_bhaga_distances() {
        let v = mrityu_bhaga(&ramana());
        let rows = v["points"].as_array().unwrap();
        let get = |id: &str| {
            rows.iter().find(|r| r["id"] == id).unwrap()["distance_deg"]
                .as_f64()
                .unwrap()
        };
        assert!((get("lagna") - lon(0, 26.0, 43.0, 11.0)).abs() < 1e-6);
        assert!((get("sun") - lon(0, 6.0, 24.0, 25.0)).abs() < 1e-6);
        assert!((get("ketu") - lon(0, 3.0, 20.0, 32.0)).abs() < 1e-6);
    }

    #[test]
    fn shodhana_pindas_match_the_reference() {
        let v = shodhana(&ramana()).unwrap();
        let want = [
            ("sun", 80, 23),
            ("moon", 177, 92),
            ("mars", 100, 73),
            ("mercury", 137, 71),
            ("jupiter", 133, 65),
            ("venus", 104, 44),
            ("saturn", 92, 15),
            ("lagna", 153, 111),
        ];
        for (g, r, gr) in want {
            let row = &v["by_chart"][g];
            assert_eq!(
                (
                    row["rashi_pinda"].as_u64().unwrap(),
                    row["graha_pinda"].as_u64().unwrap()
                ),
                (r, gr),
                "{g}"
            );
            assert_eq!(row["shodhya_pinda"].as_u64().unwrap(), r + gr);
        }
    }

    #[test]
    fn ekadhipatya_rules() {
        let occ = [
            true, false, false, false, false, false, false, false, false, false, false, false,
        ];
        // Mesha occupied with 3, Vrischika empty with 2 -> 0; Vrishabha/Tula both empty 4 and 2 -> 2 and 2
        let b = [3, 4, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0];
        let r = ekadhipatya(&b, &occ);
        assert_eq!((r[0], r[7], r[1], r[6]), (3, 0, 2, 2));
        // the empty one with more is reduced to the occupied one's count
        let r = ekadhipatya(&[2, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0], &occ);
        assert_eq!((r[0], r[7]), (2, 2));
        assert_eq!(
            trikona(&[5, 2, 4, 5, 4, 3, 6, 4, 5, 4, 2, 4]),
            [1, 0, 2, 1, 0, 1, 4, 0, 1, 2, 0, 0]
        );
    }

    #[test]
    fn jiva_deha_of_punarvasu_3() {
        let v = jiva_deha(lon(2, 28.0, 19.0, 55.0)).unwrap();
        assert_eq!(v["deha"]["sign_name"], "Vrishabha");
        assert_eq!(v["jiva"]["sign_name"], "Mithuna");
        assert_eq!(v["paramayus_years"], 83);
    }

    #[test]
    fn vedha_of_the_natal_grahas() {
        let c = ramana();
        let rows = vedha(2, &c.lon);
        let s: Vec<&str> = rows.iter().map(|r| r["summary"].as_str().unwrap()).collect();
        assert_eq!(
            s,
            [
                "Su 7 Inausp",
                "Mo 1 Ausp",
                "Ma 11 Ausp",
                "Me 6 Ausp, X by Ju in 9",
                "Ju 9 Ausp, X by Sa in 10",
                "Ve 6 Inausp",
                "Sa 10 Inausp"
            ]
        );
    }

    #[test]
    fn avakahada_and_gender_and_ghata() {
        let c = ramana();
        let pc = json!({"yoga": {"name": "Indra"}, "karana": {"name": "Taitila"}, "tithi": {"index": 17},
                        "vara": {"index": 1}, "nakshatra": {"index": 6}});
        let a = avakahada(&c, Some(&pc), Some(277.78)).unwrap();
        assert_eq!(a["yoni"]["name"], "Marjara");
        assert_eq!(a["gana"], "Deva");
        assert_eq!(a["nadi"], "Adya");
        assert_eq!(a["varna"], "Shudra");
        assert_eq!(a["vashya"], "Manava");
        assert_eq!(a["varga"]["name"], "Mesha");
        assert_eq!(a["yunja"], "Madhya");
        assert_eq!(a["hansak"], "Vaayu");
        assert_eq!(a["naamakshara"], "Ha");
        assert_eq!(a["paya"]["by_sign"], "Copper");
        assert_eq!(a["paya"]["by_nakshatra"], "Silver");
        assert_eq!(a["sunsign_western"], "Capricorn");
        assert_eq!(a["karana"]["animal"], "donkey");
        let mf = male_female(&c);
        assert_eq!(mf["female"], json!(["lagna", "mercury", "venus", "saturn"]));
        let g = ghata(&c, Some(&pc)).unwrap();
        assert_eq!(
            (g["month"].as_str(), g["weekday_name"].as_str()),
            (Some("Ashadha"), Some("Monday"))
        );
        assert_eq!(g["nakshatra_name"], "Swati");
        assert_eq!(g["moon_sign_name"], "Kumbha");
        assert_eq!(g["birth_coincides_with"], json!(["tithi", "weekday"]));
    }
}
