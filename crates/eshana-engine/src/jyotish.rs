//! Vedic features beyond the basic chart: dignity and relationships, graha and rāśi drishti, avasthā,
//! combustion, dig bala, aṣṭakavarga, Jaimini (chara kārakas, kārakāṃśa, ārūḍha padas), KP sub-lords and
//! significators, and the yoga/doṣa evaluator. Tables come from the YAML rule packs ([`crate::packs`]); this
//! module only applies them. Every input longitude is sidereal.

use crate::packs::{self, Cond};
use crate::vedic::{self, DASHA_ORDER, NAKSHATRA_SPAN};
use eshana_core::{angular_diff, norm360};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The seven classical grahas.
pub const SEVEN: [&str; 7] = ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"];
/// The nine grahas.
pub const NINE: [&str; 9] = [
    "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu",
];

/// A sidereal chart reduced to what the Vedic features need.
#[derive(Debug, Clone, Default)]
pub struct VChart {
    /// Sidereal longitude by graha id (`sun` … `saturn`, `rahu`, `ketu`, and outer planets if present).
    pub lon: BTreeMap<String, f64>,
    /// Speed in degrees per day.
    pub speed: BTreeMap<String, f64>,
    /// Sidereal Ascendant.
    pub asc: Option<f64>,
    /// Sidereal Midheaven.
    pub mc: Option<f64>,
}

impl VChart {
    /// Build from engine position ids (`mean_node`/`true_node` become `rahu`, `south_node` becomes `ketu`).
    #[must_use]
    pub fn from_positions(pos: &BTreeMap<String, (f64, f64)>, asc: Option<f64>, mc: Option<f64>) -> Self {
        let mut c = VChart {
            asc,
            mc,
            ..Default::default()
        };
        for (id, (lon, sp)) in pos {
            let key = match id.as_str() {
                "mean_node" | "true_node" => "rahu",
                "south_node" => "ketu",
                other => other,
            };
            c.lon.insert(key.to_string(), *lon);
            c.speed.insert(key.to_string(), *sp);
        }
        c
    }
    fn sign(&self, g: &str) -> Option<usize> {
        self.lon.get(g).map(|l| sign_of(*l))
    }
    /// Sign index of a reference point: `lagna`, `moon`, or any graha.
    fn ref_sign(&self, from: &str) -> Option<usize> {
        if from == "lagna" {
            self.asc.map(sign_of)
        } else {
            self.sign(from)
        }
    }
    fn grahas_present(&self) -> Vec<&'static str> {
        NINE.iter()
            .copied()
            .filter(|g| self.lon.contains_key(*g))
            .collect()
    }
}

/// Sign index of a longitude.
#[must_use]
pub fn sign_of(lon: f64) -> usize {
    ((norm360(lon) / 30.0).floor() as usize).min(11)
}

/// House count (1–12) of sign `s` from sign `from`.
#[must_use]
pub fn house_from(from: usize, s: usize) -> usize {
    (s + 12 - from) % 12 + 1
}

/// Lord of a sign under the pack's convention.
#[must_use]
pub fn lord_of(sign: usize) -> &'static str {
    let l = &packs::core().sign_lords[sign % 12];
    SEVEN.iter().copied().find(|g| g == l).unwrap_or("sun")
}

// ── dignity and relationships ────────────────────────────────────────────────────────────────────────────────────
/// Dignity of a graha at a longitude: `exalted`, `debilitated`, `moolatrikona`, `own`, or the natural
/// relationship to the sign lord (`friend`, `neutral`, `enemy`).
#[must_use]
pub fn dignity(g: &str, lon: f64) -> &'static str {
    let c = packs::core();
    let s = sign_of(lon);
    let deg = norm360(lon) - 30.0 * s as f64;
    if let Some(d) = c.dignity.get(g) {
        // A moolatrikona range inside the exaltation sign (Moon in Taurus 3°-30°, Mercury in Virgo 15°-20°)
        // takes precedence over exaltation there.
        let in_mt = d
            .moolatrikona
            .is_some_and(|m| m[0] as usize == s && deg >= m[1] && deg < m[2]);
        if d.exaltation[0] as usize == s && !in_mt {
            return "exalted";
        }
        if d.debilitation[0] as usize == s {
            return "debilitated";
        }
        if let Some(m) = d.moolatrikona {
            if m[0] as usize == s && deg >= m[1] && deg < m[2] {
                return "moolatrikona";
            }
        }
        if d.own.contains(&s) {
            return "own";
        }
    }
    let lord = lord_of(s);
    if lord == g {
        return "own";
    }
    natural_relation(g, lord)
}

/// Natural relationship of `a` towards `b`: `friend`, `neutral` or `enemy` (nodes are treated as neutral).
#[must_use]
pub fn natural_relation(a: &str, b: &str) -> &'static str {
    match packs::core().natural_friendship.get(a) {
        Some(f) if f.friends.iter().any(|x| x == b) => "friend",
        Some(f) if f.enemies.iter().any(|x| x == b) => "enemy",
        _ => "neutral",
    }
}

/// Five-fold (pañcadhā) relationship of every pair of the seven grahas: natural + temporary.
#[must_use]
pub fn panchadha_maitri(c: &VChart) -> Value {
    let temp = &packs::core().temporary_friend_houses;
    let mut out = serde_json::Map::new();
    for a in SEVEN {
        let mut row = serde_json::Map::new();
        for b in SEVEN {
            if a == b {
                continue;
            }
            let (Some(sa), Some(sb)) = (c.sign(a), c.sign(b)) else {
                continue;
            };
            let natural = natural_relation(a, b);
            let temporary = if temp.contains(&house_from(sa, sb)) {
                "friend"
            } else {
                "enemy"
            };
            let combined = match (natural, temporary) {
                ("friend", "friend") => "great_friend",
                ("friend", _) | ("enemy", "friend") => "neutral",
                ("neutral", "friend") => "friend",
                ("neutral", _) => "enemy",
                _ => "great_enemy",
            };
            row.insert(
                b.into(),
                json!({"natural": natural, "temporary": temporary, "combined": combined}),
            );
        }
        out.insert(a.into(), Value::Object(row));
    }
    Value::Object(out)
}

// ── drishti ──────────────────────────────────────────────────────────────────────────────────────────────────────
/// Graha drishti: for each graha, the houses (from its own sign) it aspects, the signs, and the grahas there.
#[must_use]
pub fn graha_drishti(c: &VChart) -> Value {
    let d = &packs::core().drishti;
    let mut out = vec![];
    // the nine grahas, and the outer planets when the chart has them (aspects by the convention in vedic-core.yaml)
    let all: Vec<&str> = NINE
        .iter()
        .chain(["uranus", "neptune", "pluto"].iter())
        .copied()
        .filter(|g| c.lon.contains_key(*g))
        .collect();
    for &g in &all {
        let Some(sg) = c.sign(g) else { continue };
        let mut houses: BTreeSet<usize> = d.all.iter().copied().collect();
        if let Some(extra) = d.special.get(g) {
            houses.extend(extra.iter().copied());
        }
        for h in houses {
            let target = (sg + h - 1) % 12;
            let hit: Vec<&str> = all
                .iter()
                .copied()
                .filter(|x| *x != g && c.sign(x) == Some(target))
                .collect();
            let full = d.all.contains(&h);
            out.push(json!({
                "from": g, "house_from_graha": h, "sign": target,
                "kind": if full { "full" } else { "special" },
                "aspects_grahas": hit,
                "aspects_bhava": c.asc.map(|a| house_from(sign_of(a), target)),
            }));
        }
    }
    Value::Array(out)
}

/// The nine grahas and, when present, Uranus, Neptune and Pluto.
fn with_outer(c: &VChart) -> Vec<&'static str> {
    NINE.iter()
        .chain(["uranus", "neptune", "pluto"].iter())
        .copied()
        .filter(|g| c.lon.contains_key(*g))
        .collect()
}

/// Graha drishti houses for a graha: the 7th, plus its special aspects from `vedic-core.yaml`.
fn drishti_houses(g: &str) -> Vec<(usize, bool)> {
    let d = &packs::core().drishti;
    let mut houses: BTreeSet<usize> = d.all.iter().copied().collect();
    if let Some(extra) = d.special.get(g) {
        houses.extend(extra.iter().copied());
    }
    houses.into_iter().map(|h| (h, d.all.contains(&h))).collect()
}

/// Additional aspects (off by default in the studio), by two conventions named in `vedic-core.yaml`:
///
/// * **retrograde shadow**: a retrograde graha (not Rāhu or Ketu, which always move backwards) also acts from the
///   sign before the one it occupies, and casts its graha drishti and rāśi drishti from there;
/// * **parivartana**: two grahas in each other's signs are read as exchanged, and each casts its aspects from the
///   other's sign (its own).
///
/// Each entry names its `basis`, the sign it is cast `from_sign`, the drishti type, the target sign and the grahas
/// and bhāva there (the targets are the grahas' real positions).
#[must_use]
pub fn additional_drishti(c: &VChart) -> Value {
    let all = with_outer(c);
    let hit_in = |g: &str, target: usize| -> Vec<&str> {
        all.iter()
            .copied()
            .filter(|x| *x != g && c.sign(x) == Some(target))
            .collect()
    };
    let bhava = |target: usize| c.asc.map(|a| house_from(sign_of(a), target));
    let mut out = vec![];
    let mut cast = |g: &str, from_sign: usize, basis: &str, partner: Option<&str>| {
        for (h, full) in drishti_houses(g) {
            let target = (from_sign + h - 1) % 12;
            out.push(json!({"from": g, "basis": basis, "partner": partner, "from_sign": from_sign, "drishti": "graha",
                "house_from_graha": h, "sign": target, "kind": if full { "full" } else { "special" },
                "aspects_grahas": hit_in(g, target), "aspects_bhava": bhava(target)}));
        }
        for target in rashi_drishti_targets(from_sign) {
            out.push(json!({"from": g, "basis": basis, "partner": partner, "from_sign": from_sign, "drishti": "rashi",
                "house_from_graha": house_from(from_sign, target), "sign": target, "kind": "rashi",
                "aspects_grahas": hit_in(g, target), "aspects_bhava": bhava(target)}));
        }
    };
    for &g in &all {
        if g == "rahu" || g == "ketu" {
            continue;
        }
        let retro = c.speed.get(g).is_some_and(|v| *v < 0.0);
        if let (true, Some(sg)) = (retro, c.sign(g)) {
            cast(g, (sg + 11) % 12, "retrograde_shadow", None);
        }
    }
    for (i, a) in SEVEN.iter().enumerate() {
        for b in &SEVEN[i + 1..] {
            let (Some(sa), Some(sb)) = (c.sign(a), c.sign(b)) else {
                continue;
            };
            if sa != sb && lord_of(sa) == *b && lord_of(sb) == *a {
                cast(a, sb, "parivartana", Some(b));
                cast(b, sa, "parivartana", Some(a));
            }
        }
    }
    Value::Array(out)
}

/// Rāśi (Jaimini) drishti: movable signs aspect fixed signs except the adjacent one, fixed aspect movable except
/// the adjacent one, dual signs aspect the other dual signs.
#[must_use]
pub fn rashi_drishti_targets(s: usize) -> Vec<usize> {
    let m = s % 3;
    (0..12)
        .filter(|&t| t != s)
        .filter(|&t| match m {
            0 => t % 3 == 1 && t != (s + 1) % 12,
            1 => t % 3 == 0 && t != (s + 11) % 12,
            _ => t % 3 == 2,
        })
        .collect()
}

/// Rāśi drishti for every occupied sign: which signs and grahas each graha aspects by sign.
#[must_use]
pub fn rashi_drishti(c: &VChart) -> Value {
    let mut out = vec![];
    let all = with_outer(c);
    for &g in &all {
        let Some(sg) = c.sign(g) else { continue };
        let targets = rashi_drishti_targets(sg);
        let hit: Vec<&str> = all
            .iter()
            .copied()
            .filter(|x| *x != g && c.sign(x).is_some_and(|s| targets.contains(&s)))
            .collect();
        out.push(json!({"from": g, "sign": sg, "aspects_signs": targets, "aspects_grahas": hit}));
    }
    Value::Array(out)
}

// ── avastha, combustion, dig bala ────────────────────────────────────────────────────────────────────────────────
/// Bālādi avasthā by degree: odd signs Bāla, Kumāra, Yuva, Vṛddha, Mṛta in 6° steps; even signs reversed.
#[must_use]
pub fn baladi_avastha(lon: f64) -> &'static str {
    const NAMES: [&str; 5] = ["bala", "kumara", "yuva", "vriddha", "mrita"];
    let s = sign_of(lon);
    let deg = norm360(lon) - 30.0 * s as f64;
    let k = ((deg / 6.0).floor() as usize).min(4);
    if s % 2 == 0 {
        NAMES[k]
    } else {
        NAMES[4 - k]
    }
}

/// Whether a graha is combust (within the pack's orb of the Sun).
#[must_use]
pub fn combust(g: &str, lon: f64, sun: f64, retrograde: bool) -> Option<bool> {
    let orb = match packs::core().combustion_orbs.get(g)? {
        packs::Orb::One(o) => *o,
        packs::Orb::Two {
            direct,
            retrograde: r,
        } => {
            if retrograde {
                *r
            } else {
                *direct
            }
        }
    };
    Some(angular_diff(lon, sun).abs() < orb)
}

/// Dig bala in virūpas (0–60): 60 at the graha's strong angle, falling linearly to 0 at the opposite point.
#[must_use]
pub fn dig_bala(g: &str, lon: f64, asc: f64, mc: f64) -> Option<f64> {
    let point = match packs::core().dig_bala_strong_point.get(g)?.as_str() {
        "ascendant" => asc,
        "midheaven" => mc,
        "descendant" => asc + 180.0,
        _ => mc + 180.0,
    };
    Some((180.0 - angular_diff(lon, point).abs()) / 3.0)
}

/// Strength table: dignity, avasthā, combustion, retrogression, dig bala and naisargika bala per graha.
#[must_use]
pub fn strength(c: &VChart) -> Value {
    let core = packs::core();
    let sun = c.lon.get("sun").copied();
    let mut out = serde_json::Map::new();
    for g in c.grahas_present() {
        let lon = c.lon[g];
        let retro = c.speed.get(g).is_some_and(|s| *s < 0.0);
        out.insert(
            g.into(),
            json!({
                "dignity": dignity(g, lon),
                "avastha": baladi_avastha(lon),
                "retrograde": retro,
                "combust": if g == "sun" { None } else { sun.and_then(|s| combust(g, lon, s, retro)) },
                "dig_bala_virupa": match (c.asc, c.mc) { (Some(a), Some(m)) => dig_bala(g, lon, a, m), _ => None },
                "naisargika_bala_virupa": core.naisargika_bala.get(g),
            }),
        );
    }
    Value::Object(out)
}

// ── ashtakavarga ─────────────────────────────────────────────────────────────────────────────────────────────────
/// Bhinna- and sarvāṣṭakavarga. Needs the lagna (it contributes bindus).
#[must_use]
pub fn ashtakavarga(c: &VChart) -> Option<Value> {
    let asc = c.asc?;
    let pack = packs::ashtakavarga();
    let mut sav = [0u32; 12];
    let mut bav = serde_json::Map::new();
    for g in SEVEN {
        let table = pack.tables.get(g)?;
        let mut bindus = [0u32; 12];
        let mut by_contributor = serde_json::Map::new();
        for (contrib, houses) in table {
            let from = if contrib == "lagna" {
                sign_of(asc)
            } else {
                c.sign(contrib)?
            };
            let mut signs = vec![];
            for h in houses {
                let s = (from + h - 1) % 12;
                bindus[s] += 1;
                signs.push(s);
            }
            by_contributor.insert(contrib.clone(), json!(signs));
        }
        for (i, b) in bindus.iter().enumerate() {
            sav[i] += b;
        }
        bav.insert(
            g.into(),
            json!({"bindus": bindus, "total": bindus.iter().sum::<u32>(), "contributions": by_contributor}),
        );
    }
    Some(json!({
        "bhinnashtakavarga": bav,
        "sarvashtakavarga": {"bindus": sav, "total": sav.iter().sum::<u32>()},
        "sign_order": "index 0 = Mesha (Aries)",
    }))
}

// ── Jaimini ──────────────────────────────────────────────────────────────────────────────────────────────────────
/// Chara kārakas in both common schemes: seven (Sun–Saturn) and eight (with Rahu, counted backwards).
#[must_use]
pub fn chara_karakas(c: &VChart) -> Value {
    const SEVEN_NAMES: [&str; 7] = [
        "atmakaraka",
        "amatyakaraka",
        "bhratrikaraka",
        "matrikaraka",
        "putrakaraka",
        "gnatikaraka",
        "darakaraka",
    ];
    const EIGHT_NAMES: [&str; 8] = [
        "atmakaraka",
        "amatyakaraka",
        "bhratrikaraka",
        "matrikaraka",
        "pitrikaraka",
        "putrakaraka",
        "gnatikaraka",
        "darakaraka",
    ];
    let deg = |g: &str| c.lon.get(g).map(|l| norm360(*l) % 30.0);
    let rank = |list: &[&str], names: &[&str]| -> Value {
        let mut v: Vec<(String, f64)> = list
            .iter()
            .filter_map(|g| {
                let d = if *g == "rahu" { 30.0 - deg(g)? } else { deg(g)? };
                Some(((*g).to_string(), d))
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut m = serde_json::Map::new();
        for (name, (g, d)) in names.iter().zip(v) {
            let d9 = c.lon.get(&g).and_then(|l| vedic::divisional_sign(*l, 9).ok());
            m.insert(
                (*name).into(),
                json!({"graha": g, "degrees_in_sign": d, "navamsa_sign": d9}),
            );
        }
        Value::Object(m)
    };
    let mut eight: Vec<&str> = SEVEN.to_vec();
    eight.push("rahu");
    let seven = rank(&SEVEN, &SEVEN_NAMES);
    let karakamsa = seven
        .get("atmakaraka")
        .and_then(|a| a.get("navamsa_sign"))
        .cloned()
        .unwrap_or(Value::Null);
    json!({
        "seven_karaka_scheme": seven,
        "eight_karaka_scheme": rank(&eight, &EIGHT_NAMES),
        "karakamsa_sign": karakamsa,
        "karakamsa_note": "navamsa sign of the atmakaraka (seven-karaka scheme)",
    })
}

/// The chart of a division as a [`VChart`]: each graha and the lagna placed at the middle of its varga sign (only
/// signs matter for sign-based rules such as ārūḍha padas).
#[must_use]
pub fn varga_chart(c: &VChart, d: u32) -> VChart {
    let at = |lon: f64| {
        crate::vedic::divisional_sign(lon, d)
            .map(|sg| sg as f64 * 30.0 + 15.0)
            .ok()
    };
    // speeds are kept: a retrograde graha's additional aspects (retrograde shadow) apply in a varga too
    let mut v = VChart {
        asc: c.asc.and_then(at),
        speed: c.speed.clone(),
        ..Default::default()
    };
    for (g, l) in &c.lon {
        if let Some(x) = at(*l) {
            v.lon.insert(g.clone(), x);
        }
    }
    v
}

/// Ārūḍha padas A1 (Ārūḍha lagna) … A12 (Upapada): count from the house to its lord, as far again from the lord;
/// if that is the house itself or its 7th, take the 10th from there.
#[must_use]
pub fn arudha_padas(c: &VChart) -> Option<Value> {
    let lagna = sign_of(c.asc?);
    let mut out = serde_json::Map::new();
    for h in 1..=12usize {
        let hs = (lagna + h - 1) % 12;
        let lord = lord_of(hs);
        let ls = c.sign(lord)?;
        let n = (ls + 12 - hs) % 12;
        let mut pada = (ls + n) % 12;
        let mut exception = false;
        if pada == hs || pada == (hs + 6) % 12 {
            pada = (pada + 9) % 12;
            exception = true;
        }
        let key = match h {
            1 => "A1_arudha_lagna".to_string(),
            12 => "A12_upapada".to_string(),
            _ => format!("A{h}"),
        };
        out.insert(
            key,
            json!({"house": h, "house_sign": hs, "lord": lord, "lord_sign": ls, "pada_sign": pada,
                   "pada_house": house_from(lagna, pada), "tenth_from_exception": exception}),
        );
    }
    Some(Value::Object(out))
}

// ── KP ───────────────────────────────────────────────────────────────────────────────────────────────────────────
/// KP star, sub and sub-sub lords of a sidereal longitude (Vimśottarī proportions within each nakshatra).
#[must_use]
pub fn kp_lords(lon: f64) -> Value {
    let lon = norm360(lon);
    let nk = vedic::nakshatra_of(lon);
    let start_idx = nk.index % 9;
    let within = lon - nk.index as f64 * NAKSHATRA_SPAN; // degrees into the nakshatra
    let (sub, sub_off, sub_len) = split(start_idx, within, NAKSHATRA_SPAN);
    let (subsub, _, _) = split(sub, sub_off, sub_len);
    json!({
        "sign_lord": lord_of(sign_of(lon)),
        "star_lord": DASHA_ORDER[start_idx].0.to_lowercase(),
        "sub_lord": DASHA_ORDER[sub].0.to_lowercase(),
        "sub_sub_lord": DASHA_ORDER[subsub].0.to_lowercase(),
        "nakshatra": nk.name,
        "pada": nk.pada,
    })
}

/// Split a span into nine Vimśottarī-proportional parts starting at lord index `start`; returns the lord index
/// containing `offset`, the offset inside that part and the part length.
fn split(start: usize, offset: f64, span: f64) -> (usize, f64, f64) {
    let mut acc = 0.0;
    for k in 0..9 {
        let idx = (start + k) % 9;
        let len = span * DASHA_ORDER[idx].1 / 120.0;
        if offset < acc + len || k == 8 {
            return (idx, offset - acc, len);
        }
        acc += len;
    }
    unreachable!("nine parts cover the span")
}

/// KP tables from sidereal Placidus cusps: cusp lords, planet lords, and four-level significators in both the
/// house view and the planet view, under the default occupancy convention.
#[must_use]
pub fn kp(c: &VChart, cusps: &[f64; 12]) -> Value {
    kp_with(c, cusps, crate::kp::Occupancy::CuspStart)
}

/// [`kp`] under a named cusp-occupancy convention.
#[must_use]
pub fn kp_with(c: &VChart, cusps: &[f64; 12], occupancy: crate::kp::Occupancy) -> Value {
    let kp_house = |lon: f64, cusps: &[f64; 12]| occupancy.house_of(lon, cusps);
    let grahas = c.grahas_present();
    let star_lord = |g: &str| -> String {
        DASHA_ORDER[vedic::nakshatra_of(c.lon[g]).index % 9]
            .0
            .to_lowercase()
    };
    let occupied: BTreeMap<&str, usize> = grahas.iter().map(|g| (*g, kp_house(c.lon[*g], cusps))).collect();
    // ownership: houses whose cusp sign is ruled by the graha
    let owns = |g: &str| -> Vec<usize> { (1..=12).filter(|h| lord_of(sign_of(cusps[h - 1])) == g).collect() };
    let mut house_view = serde_json::Map::new();
    for h in 1..=12usize {
        let occupants: Vec<&str> = grahas.iter().copied().filter(|g| occupied[g] == h).collect();
        let lord = lord_of(sign_of(cusps[h - 1]));
        let in_star_of = |targets: &[&str]| -> Vec<&str> {
            grahas
                .iter()
                .copied()
                .filter(|g| targets.contains(&star_lord(g).as_str()))
                .collect()
        };
        house_view.insert(
            h.to_string(),
            json!({
                "cusp": cusps[h - 1],
                "lords": kp_lords(cusps[h - 1]),
                "A_in_star_of_occupants": in_star_of(&occupants),
                "B_occupants": occupants,
                "C_in_star_of_lord": in_star_of(&[lord]),
                "D_lord": lord,
            }),
        );
    }
    let mut planet_view = serde_json::Map::new();
    let mut planets = serde_json::Map::new();
    for g in &grahas {
        let sl = star_lord(g);
        let sl_occ = occupied.get(sl.as_str()).copied();
        planet_view.insert(
            (*g).into(),
            json!({
                "A_house_of_star_lord": sl_occ,
                "B_house_occupied": occupied[g],
                "C_houses_owned_by_star_lord": owns(&sl),
                "D_houses_owned": owns(g),
            }),
        );
        planets.insert((*g).into(), kp_lords(c.lon[*g]));
    }
    json!({"cusps": house_view, "planets": planets, "significators_planet_view": planet_view,
           "occupancy": occupancy.describe(), "occupancy_method": occupancy.id()})
}

/// KP ruling planets at the chart's instant: day lord, Moon and lagna sign/star/sub lords.
#[must_use]
pub fn ruling_planets(c: &VChart, day_lord: &str) -> Option<Value> {
    let moon = kp_lords(*c.lon.get("moon")?);
    let lagna = kp_lords(c.asc?);
    Some(json!({
        "day_lord": day_lord,
        "moon_sign_lord": moon["sign_lord"], "moon_star_lord": moon["star_lord"], "moon_sub_lord": moon["sub_lord"],
        "lagna_sign_lord": lagna["sign_lord"], "lagna_star_lord": lagna["star_lord"], "lagna_sub_lord": lagna["sub_lord"],
    }))
}

// ── yogas and doshas ─────────────────────────────────────────────────────────────────────────────────────────────
fn subst(g: &str, bind: Option<&str>) -> String {
    match (g, bind) {
        ("$g", Some(b)) => b.to_string(),
        _ => g.to_string(),
    }
}

fn evaluate(c: &VChart, cond: &Cond, bind: Option<&str>, used: &mut BTreeSet<String>) -> Option<bool> {
    Some(match cond {
        Cond::InHouse { graha, from, houses } => {
            let g = subst(graha, bind);
            let ok = houses.contains(&house_from(c.ref_sign(from)?, c.sign(&g)?));
            if ok {
                used.insert(g);
            }
            ok
        }
        Cond::Dignity { graha, any_of } => {
            let g = subst(graha, bind);
            let lon = *c.lon.get(&g)?;
            let ok = any_of.iter().any(|d| d == dignity(&g, lon));
            if ok {
                used.insert(g);
            }
            ok
        }
        Cond::SameSign(list) => {
            let signs: Option<Vec<usize>> = list.iter().map(|g| c.sign(&subst(g, bind))).collect();
            let signs = signs?;
            let ok = signs.windows(2).all(|w| w[0] == w[1]);
            if ok {
                used.extend(list.iter().map(|g| subst(g, bind)));
            }
            ok
        }
        Cond::Occupied {
            from,
            houses,
            min,
            exclude,
        } => {
            let base = c.ref_sign(from)?;
            let hits: Vec<&str> = c
                .grahas_present()
                .into_iter()
                .filter(|g| !exclude.iter().any(|x| x == g) && *g != from)
                .filter(|g| c.sign(g).is_some_and(|s| houses.contains(&house_from(base, s))))
                .collect();
            let ok = hits.len() >= min.unwrap_or(1);
            if ok {
                used.extend(hits.iter().map(|s| (*s).to_string()));
            }
            ok
        }
        Cond::Empty {
            from,
            houses,
            exclude,
        } => {
            let base = c.ref_sign(from)?;
            !c.grahas_present()
                .into_iter()
                .filter(|g| !exclude.iter().any(|x| x == g) && *g != from)
                .any(|g| c.sign(g).is_some_and(|s| houses.contains(&house_from(base, s))))
        }
        Cond::LordsConjoined {
            houses_a,
            houses_b,
            from,
        } => {
            let base = c.ref_sign(from)?;
            let lords = |hs: &[usize]| -> BTreeSet<&'static str> {
                hs.iter().map(|h| lord_of((base + h - 1) % 12)).collect()
            };
            let (la, lb) = (lords(houses_a), lords(houses_b));
            let mut ok = false;
            for a in &la {
                for b in &lb {
                    if a != b && c.sign(a).is_some() && c.sign(a) == c.sign(b) {
                        used.insert((*a).to_string());
                        used.insert((*b).to_string());
                        ok = true;
                    }
                }
            }
            ok
        }
        Cond::LordInHouse { house, from, houses } => {
            let base = c.ref_sign(from)?;
            let lord = lord_of((base + house - 1) % 12);
            let ok = houses.contains(&house_from(base, c.sign(lord)?));
            if ok {
                used.insert(lord.to_string());
            }
            ok
        }
        Cond::LordDignity { house, from, any_of } => {
            let base = c.ref_sign(from)?;
            let lord = lord_of((base + house - 1) % 12);
            any_of
                .iter()
                .any(|d| d == dignity(lord, *c.lon.get(lord).unwrap_or(&0.0)))
        }
        Cond::BetweenNodes {} => {
            let rahu = *c.lon.get("rahu")?;
            let sides: Option<Vec<bool>> = SEVEN
                .iter()
                .map(|g| c.lon.get(*g).map(|l| norm360(l - rahu) < 180.0))
                .collect();
            let sides = sides?;
            sides.iter().all(|s| *s) || sides.iter().all(|s| !*s)
        }
        Cond::Parivartana { kind, from } => {
            let kind = kind.as_deref().unwrap_or("any");
            let base = if kind == "any" {
                None
            } else {
                Some(c.ref_sign(from.as_deref().unwrap_or("lagna"))?)
            };
            let mut ok = false;
            for (i, a) in SEVEN.iter().enumerate() {
                for b in &SEVEN[i + 1..] {
                    let (Some(sa), Some(sb)) = (c.sign(a), c.sign(b)) else {
                        continue;
                    };
                    if lord_of(sa) != *b || lord_of(sb) != *a {
                        continue;
                    }
                    let hit = match base {
                        None => true,
                        Some(base) => {
                            let hs = [house_from(base, sa), house_from(base, sb)];
                            let dusthana = hs.iter().any(|h| matches!(h, 6 | 8 | 12));
                            match kind {
                                "dainya" => dusthana,
                                "khala" => !dusthana && hs.contains(&3),
                                "maha" => !dusthana && !hs.contains(&3),
                                _ => false,
                            }
                        }
                    };
                    if hit {
                        used.insert((*a).to_string());
                        used.insert((*b).to_string());
                        ok = true;
                    }
                }
            }
            ok
        }
        Cond::AllOf(list) => {
            let mut ok = true;
            for x in list {
                ok &= evaluate(c, x, bind, used)?;
            }
            ok
        }
        Cond::AnyOf(list) => {
            let mut ok = false;
            for x in list {
                ok |= evaluate(c, x, bind, used).unwrap_or(false);
            }
            ok
        }
        Cond::Not(x) => !evaluate(c, x, bind, &mut BTreeSet::new())?,
        Cond::ForEachGraha { grahas, condition } => {
            let mut ok = false;
            for g in grahas {
                if evaluate(c, condition, Some(g), used).unwrap_or(false) {
                    ok = true;
                }
            }
            ok
        }
    })
}

/// Evaluate every yoga and dosha in the pack. Each entry reports whether it holds, the grahas involved, or that it
/// could not be evaluated (for example a lagna-based yoga without a birth time).
#[must_use]
pub fn yogas(c: &VChart) -> Value {
    yogas_with(c, packs::yogas())
}

/// [`yogas`] for any pack in the yogas format: the built-in one or a user's (RULE-02).
#[must_use]
pub fn yogas_with(c: &VChart, pack: &packs::Yogas) -> Value {
    let run = |defs: &[packs::YogaDef]| -> Vec<Value> {
        defs.iter()
            .map(|d| {
                let mut used = BTreeSet::new();
                let r = evaluate(c, &d.when, None, &mut used);
                json!({
                    "id": d.id, "name": d.name, "category": d.category, "rule": d.rule,
                    "present": r, "evaluable": r.is_some(),
                    "grahas": if r == Some(true) { used.into_iter().collect::<Vec<_>>() } else { vec![] },
                })
            })
            .collect()
    };
    json!({"yogas": run(&pack.yogas), "doshas": run(&pack.doshas)})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chart(pairs: &[(&str, f64)], asc: f64) -> VChart {
        let mut c = VChart {
            asc: Some(asc),
            mc: Some(norm360(asc + 270.0)),
            ..Default::default()
        };
        for (g, l) in pairs {
            c.lon.insert((*g).into(), *l);
            c.speed.insert((*g).into(), 1.0);
        }
        c
    }
    fn sample() -> VChart {
        chart(
            &[
                ("sun", 10.0),
                ("moon", 96.0),
                ("mars", 298.0),
                ("mercury", 25.0),
                ("jupiter", 95.0),
                ("venus", 350.0),
                ("saturn", 200.0),
                ("rahu", 50.0),
                ("ketu", 230.0),
            ],
            5.0,
        )
    }

    #[test]
    fn packs_validate() {
        packs::validate().unwrap();
    }

    #[test]
    fn ashtakavarga_totals_are_fixed_by_the_tables() {
        let a = ashtakavarga(&sample()).unwrap();
        let want = [
            ("sun", 48),
            ("moon", 49),
            ("mars", 39),
            ("mercury", 54),
            ("jupiter", 56),
            ("venus", 52),
            ("saturn", 39),
        ];
        for (g, t) in want {
            assert_eq!(a["bhinnashtakavarga"][g]["total"], t, "{g}");
        }
        assert_eq!(a["sarvashtakavarga"]["total"], 337);
    }

    #[test]
    fn dignities() {
        assert_eq!(dignity("sun", 10.0), "exalted");
        assert_eq!(dignity("sun", 190.0), "debilitated");
        assert_eq!(dignity("sun", 130.0), "moolatrikona");
        assert_eq!(dignity("sun", 145.0), "own");
        assert_eq!(dignity("mars", 298.0), "exalted");
        assert_eq!(dignity("jupiter", 95.0), "exalted");
        assert_eq!(dignity("moon", 31.0), "exalted");
        assert_eq!(dignity("mercury", 167.0), "moolatrikona");
        assert_eq!(dignity("mercury", 160.0), "exalted");
        assert_eq!(dignity("moon", 50.0), "moolatrikona");
    }

    #[test]
    fn yogas_hold_where_the_rule_says() {
        let y = yogas(&sample());
        let get = |id: &str| {
            y["yogas"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap()
                .clone()
        };
        assert_eq!(get("gaja_kesari")["present"], true); // Jupiter with the Moon (1st from Moon)
        assert_eq!(get("ruchaka")["present"], true); // Mars exalted in Capricorn, 10th from an Aries lagna
        assert_eq!(get("hamsa")["present"], true); // Jupiter exalted in Cancer, 4th
        assert_eq!(get("budha_aditya")["present"], true); // Sun and Mercury in Aries
        let d = y["doshas"].as_array().unwrap();
        let ks = d.iter().find(|x| x["id"] == "kala_sarpa").unwrap();
        assert_eq!(ks["present"], false);
    }

    #[test]
    fn parivartana_yoga_and_its_kind() {
        // Kanyā lagna; Jupiter in Kumbha (6th) and Saturn in Mīna (7th): Jupiter and Saturn exchange signs, and the
        // 6th is a dusthāna, so it is a Dainya parivartana (the Ramana Maharishi chart has this exchange)
        let mut c = VChart {
            asc: Some(160.0),
            ..Default::default()
        };
        for (g, l) in [
            ("sun", 255.0),
            ("moon", 88.0),
            ("mars", 20.0),
            ("mercury", 233.0),
            ("jupiter", 316.0),
            ("venus", 210.0),
            ("saturn", 347.0),
        ] {
            c.lon.insert(g.into(), l);
        }
        let y = yogas(&c);
        let get = |id: &str| {
            y["yogas"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["id"] == id)
                .unwrap()
                .clone()
        };
        assert_eq!(get("parivartana")["present"], true);
        assert_eq!(get("parivartana")["grahas"], json!(["jupiter", "saturn"]));
        assert_eq!(get("dainya_parivartana")["present"], true);
        assert_eq!(get("maha_parivartana")["present"], false);
        assert_eq!(get("khala_parivartana")["present"], false);
    }

    #[test]
    fn rashi_drishti_rules() {
        assert_eq!(rashi_drishti_targets(0), vec![4, 7, 10]); // Aries: Leo, Scorpio, Aquarius (not Taurus)
        assert_eq!(rashi_drishti_targets(1), vec![3, 6, 9]); // Taurus: Cancer, Libra, Capricorn (not Aries)
        assert_eq!(rashi_drishti_targets(2), vec![5, 8, 11]); // Gemini: the other duals
    }

    #[test]
    fn additional_aspects_from_retro_shadow_and_exchange() {
        let mut c = VChart::default();
        // Mars retrograde in Gemini (sign 2); Sun in Aries (Mars's sign) and Mars… no exchange here
        c.lon.insert("mars".into(), 75.0);
        c.speed.insert("mars".into(), -0.2);
        // Jupiter in Gemini (Mercury's sign) and Mercury in Sagittarius (Jupiter's sign): an exchange
        c.lon.insert("jupiter".into(), 80.0);
        c.speed.insert("jupiter".into(), 0.1);
        c.lon.insert("mercury".into(), 250.0);
        c.speed.insert("mercury".into(), 1.0);
        let v = additional_drishti(&c);
        let a = v.as_array().unwrap();
        let shadow: Vec<_> = a.iter().filter(|x| x["basis"] == "retrograde_shadow").collect();
        assert!(shadow.iter().all(|x| x["from"] == "mars" && x["from_sign"] == 1)); // cast from Taurus
        assert!(shadow
            .iter()
            .any(|x| x["drishti"] == "graha" && x["house_from_graha"] == 7 && x["sign"] == 7));
        let swap: Vec<_> = a.iter().filter(|x| x["basis"] == "parivartana").collect();
        assert!(swap
            .iter()
            .any(|x| x["from"] == "jupiter" && x["from_sign"] == 8 && x["partner"] == "mercury"));
        assert!(swap.iter().any(|x| x["from"] == "mercury" && x["from_sign"] == 2));
    }

    #[test]
    fn graha_drishti_includes_special_aspects() {
        let d = graha_drishti(&sample());
        let houses: Vec<u64> = d
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["from"] == "saturn")
            .map(|x| x["house_from_graha"].as_u64().unwrap())
            .collect();
        assert_eq!(houses, vec![3, 7, 10]);
    }

    #[test]
    fn kp_sub_lords_follow_vimshottari_proportions() {
        // Start of Ashwini: star Ketu, sub Ketu, sub-sub Ketu.
        let k = kp_lords(0.0001);
        assert_eq!(
            (k["star_lord"].as_str(), k["sub_lord"].as_str()),
            (Some("ketu"), Some("ketu"))
        );
        // Ketu sub of Ashwini spans 13°20′ × 7/120 = 0°46′40″; just after it the sub is Venus.
        let k = kp_lords(13.333_333 * 7.0 / 120.0 + 0.001);
        assert_eq!(k["sub_lord"], "venus");
    }

    #[test]
    fn pada_navamsha_uses_navamsha_signs() {
        // 1° Aries is navāṃśa Aries (sign 0); 28° Aries is navāṃśa Sagittarius (sign 8)
        let mut c = VChart {
            asc: Some(1.0),
            ..Default::default()
        };
        for g in SEVEN {
            c.lon.insert(g.into(), 28.0);
        }
        let v = varga_chart(&c, 9);
        assert_eq!(sign_of(v.asc.unwrap()), 0);
        assert_eq!(sign_of(v.lon["mars"]), 8);
        // D9 lagna Aries; its lord Mars in D9 Sagittarius (9th): count 9 again from Sagittarius → Leo (5th)
        let a = arudha_padas(&v).unwrap();
        assert_eq!(a["A1_arudha_lagna"]["pada_sign"], 4);
    }

    #[test]
    fn arudha_lagna_example() {
        // Aries lagna, Mars (lord) in Cancer (4th): count 4 again from Cancer -> Libra (7th from lagna) -> exception
        // -> 10th from Libra = Cancer.
        let c = chart(
            &[
                ("sun", 10.0),
                ("moon", 40.0),
                ("mars", 100.0),
                ("mercury", 25.0),
                ("jupiter", 95.0),
                ("venus", 350.0),
                ("saturn", 200.0),
                ("rahu", 50.0),
                ("ketu", 230.0),
            ],
            5.0,
        );
        let a = arudha_padas(&c).unwrap();
        assert_eq!(a["A1_arudha_lagna"]["pada_sign"], 3);
        assert_eq!(a["A1_arudha_lagna"]["tenth_from_exception"], true);
    }

    #[test]
    fn karakas_rank_by_degree() {
        let k = chara_karakas(&sample());
        assert_eq!(k["seven_karaka_scheme"]["atmakaraka"]["graha"], "mars"); // 28°
        assert_eq!(k["seven_karaka_scheme"]["darakaraka"]["graha"], "jupiter"); // 5°, the lowest
    }

    #[test]
    fn combustion_and_dig_bala_use_the_unsigned_separation() {
        // Regression: a signed difference made every graha behind the Sun "combust".
        assert_eq!(combust("moon", 271.9, 30.6, false), Some(false));
        assert_eq!(combust("mars", 20.0, 30.0, false), Some(true));
        assert_eq!(combust("mars", 40.0, 30.0, false), Some(true));
        assert_eq!(combust("mercury", 43.0, 30.0, true), Some(false)); // 13° > 12° retrograde orb
        let asc = 100.0;
        let mc = 10.0;
        assert!((dig_bala("jupiter", asc, asc, mc).unwrap() - 60.0).abs() < 1e-9);
        assert!(dig_bala("jupiter", asc + 180.0, asc, mc).unwrap().abs() < 1e-9);
        assert!((dig_bala("jupiter", asc - 90.0, asc, mc).unwrap() - 30.0).abs() < 1e-9);
    }

    #[test]
    fn avastha_reverses_in_even_signs() {
        assert_eq!(baladi_avastha(2.0), "bala");
        assert_eq!(baladi_avastha(32.0), "mrita");
    }
}
