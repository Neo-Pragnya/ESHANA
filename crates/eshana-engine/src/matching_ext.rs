//! Supplementary matching factors beside the 36-point Aṣṭakūṭa (which this module never touches): Rajju, Vedha,
//! Mahendra and Strī-dīrgha from the two Moons; Maṅgala doṣa per reference point with every cancellation kept as a
//! trace; and pair factors across the two charts (each partner's grahas read in the other's Aṣṭakavarga, and
//! sign aspects from one chart onto the other). Tables live in `rulesets/matching-supplementary.yaml`.

use crate::jyotish::{sign_of, SEVEN};
use crate::packs;
use crate::vedic;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pack, verbatim.
pub const SOURCE: &str = include_str!("../rulesets/matching-supplementary.yaml");

/// Rajju table.
#[derive(Debug, Deserialize)]
pub struct Rajju {
    /// The nine-nakṣatra block.
    pub block: Vec<String>,
}
/// Vedha table.
#[derive(Debug, Deserialize)]
pub struct Vedha {
    /// 1-based pairs.
    pub pairs: Vec<[usize; 2]>,
}
/// Mahendra counts.
#[derive(Debug, Deserialize)]
pub struct Mahendra {
    /// Counts from b to a that make Mahendra.
    pub counts: Vec<usize>,
}
/// Strī-dīrgha threshold.
#[derive(Debug, Deserialize)]
pub struct StriDirgha {
    /// Count from b to a must exceed this.
    pub threshold: usize,
}
/// One cancellation rule.
#[derive(Debug, Deserialize)]
pub struct Cancellation {
    /// Id.
    pub id: String,
    /// Text of the rule.
    pub rule: String,
    /// `all_references`, `per_reference` or `pair`.
    pub scope: String,
    /// Signs of Mars that cancel (own or exalted).
    #[serde(default)]
    pub mars_signs: Vec<usize>,
    /// House → signs of Mars that cancel in that house.
    #[serde(default)]
    pub by_house: BTreeMap<String, Vec<usize>>,
}
/// Maṅgala doṣa rules.
#[derive(Debug, Deserialize)]
pub struct Manglik {
    /// Houses from a reference that give the doṣa.
    pub houses: Vec<usize>,
    /// Reference points.
    pub references: Vec<String>,
    /// Cancellations.
    pub cancellations: Vec<Cancellation>,
}
/// Cross-chart aspects.
#[derive(Debug, Deserialize)]
pub struct CrossAspects {
    /// Extra sign aspects per graha (every graha also aspects the 7th).
    pub special: BTreeMap<String, Vec<usize>>,
}
/// The pack.
#[derive(Debug, Deserialize)]
pub struct Pack {
    /// Id.
    pub id: String,
    /// Version.
    pub version: String,
    /// Rajju.
    pub rajju: Rajju,
    /// Vedha.
    pub vedha: Vedha,
    /// Mahendra.
    pub mahendra: Mahendra,
    /// Strī-dīrgha.
    pub stri_dirgha: StriDirgha,
    /// Maṅgala doṣa.
    pub manglik: Manglik,
    /// Cross aspects.
    pub cross_aspects: CrossAspects,
}

/// The pack, parsed once and checked: the rajju block has nine entries; vedha pairs are distinct and in range.
#[must_use]
pub fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| {
        let p: Pack =
            serde_norway::from_str(SOURCE).unwrap_or_else(|e| panic!("matching-supplementary.yaml: {e}"));
        assert_eq!(p.rajju.block.len(), 9, "rajju block");
        assert!(p
            .vedha
            .pairs
            .iter()
            .all(|[x, y]| x != y && (1..=27).contains(x) && (1..=27).contains(y)));
        p
    })
}

/// Rule id of the pack.
#[must_use]
pub fn rule() -> String {
    let p = pack();
    packs::rule_id(&p.id, &p.version)
}

/// Rajju of a nakṣatra (0–26).
#[must_use]
pub fn rajju_of(nak: usize) -> &'static str {
    &pack().rajju.block[nak % 9]
}

/// Whether two nakṣatras (0–26) are in vedha.
#[must_use]
pub fn in_vedha(x: usize, y: usize) -> bool {
    pack()
        .vedha
        .pairs
        .iter()
        .any(|[p, q]| (p - 1, q - 1) == (x, y) || (p - 1, q - 1) == (y, x))
}

fn count(from: usize, to: usize) -> usize {
    (to + 27 - from) % 27 + 1
}

/// The four Moon-based factors. Rajju and Vedha are symmetric; Mahendra and Strī-dīrgha count from b to a.
#[must_use]
pub fn moon_factors(moon_a: f64, moon_b: f64) -> Value {
    let p = pack();
    let (na, nb) = (
        vedic::nakshatra_of(moon_a).index,
        vedic::nakshatra_of(moon_b).index,
    );
    let (ra, rb) = (rajju_of(na), rajju_of(nb));
    let c = count(nb, na);
    let rid = rule();
    json!([
        {"id": "rajju", "rule": format!("{rid}#rajju"), "a": ra, "b": rb, "dosha": ra == rb,
         "basis": "both Moons' nakṣatras in the same rajju is the doṣa", "direction": "symmetric"},
        {"id": "vedha", "rule": format!("{rid}#vedha"), "a": vedic::NAKSHATRAS[na], "b": vedic::NAKSHATRAS[nb],
         "dosha": in_vedha(na, nb), "basis": "the two Moons' nakṣatras obstruct each other", "direction": "symmetric"},
        {"id": "mahendra", "rule": format!("{rid}#mahendra"), "count_from_b": c,
         "present": p.mahendra.counts.contains(&c), "basis": format!("count from b's nakṣatra to a's in {:?}", p.mahendra.counts),
         "direction": "b to a"},
        {"id": "stri_dirgha", "rule": format!("{rid}#stri_dirgha>{}", p.stri_dirgha.threshold), "count_from_b": c,
         "present": c > p.stri_dirgha.threshold, "basis": format!("count from b's nakṣatra to a's exceeds {}", p.stri_dirgha.threshold),
         "direction": "b to a"},
    ])
}

/// Signs of one chart's grahas and lagna, from its envelope (`None` for the lagna without a reliable Ascendant).
#[must_use]
pub fn signs_of(env: &Value) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for p in env["positions"].as_array().into_iter().flatten() {
        if let (Some(id), Some(l)) = (p["id"].as_str(), p["longitude_deg"].as_f64()) {
            out.insert(id.to_string(), sign_of(l));
        }
    }
    if let Some(l) = env["houses"]["ascendant"]["longitude_deg"].as_f64() {
        out.insert("lagna".into(), sign_of(l));
    }
    out
}

fn jupiter_aspects(jup: usize, target: usize) -> bool {
    let h = (target + 12 - jup) % 12 + 1;
    [1, 5, 7, 9].contains(&h)
}

/// Maṅgala doṣa of one chart: the raw finding per reference point and each cancellation with its rule and scope.
#[must_use]
pub fn manglik(signs: &BTreeMap<String, usize>) -> Value {
    let m = &pack().manglik;
    let Some(&mars) = signs.get("mars") else {
        return crate::methods::unavailable("Mars is not in this chart", &["mars"]);
    };
    let jup = signs.get("jupiter").copied();
    let mut refs = serde_json::Map::new();
    for r in &m.references {
        let Some(&from) = signs.get(r.as_str()) else {
            refs.insert(r.clone(), json!({"availability": crate::methods::unavailable(
                "this reference point needs an exact birth time and a place", &[if r == "lagna" { "ascendant" } else { r.as_str() }])}));
            continue;
        };
        let house = (mars + 12 - from) % 12 + 1;
        let raw = m.houses.contains(&house);
        let mut cancels = vec![];
        if raw {
            for c in &m.cancellations {
                let hit = match c.id.as_str() {
                    "own_or_exalted" => c.mars_signs.contains(&mars),
                    "house_sign_exception" => c
                        .by_house
                        .get(&house.to_string())
                        .is_some_and(|s| s.contains(&mars)),
                    "jupiter_association" => jup.is_some_and(|j| jupiter_aspects(j, mars)),
                    _ => false,
                };
                if hit {
                    cancels.push(json!({"id": c.id, "rule": c.rule, "scope": c.scope, "rule_id": format!("{}#{}", rule(), c.id)}));
                }
            }
        }
        refs.insert(
            r.clone(),
            json!({"reference_sign": from, "mars_house": house, "raw": raw, "cancellations": cancels,
                   "after_cancellations": raw && cancels.is_empty()}),
        );
    }
    json!({"mars_sign": mars, "houses": m.houses, "by_reference": refs})
}

/// Pair-scope cancellations: both partners with the raw doṣa from the same reference point.
#[must_use]
pub fn manglik_pair(a: &Value, b: &Value) -> Value {
    let m = &pack().manglik;
    let mut out = vec![];
    for r in &m.references {
        let ra = a["by_reference"][r]["raw"].as_bool();
        let rb = b["by_reference"][r]["raw"].as_bool();
        if ra == Some(true) && rb == Some(true) {
            let c = m.cancellations.iter().find(|c| c.scope == "pair");
            out.push(json!({"reference": r, "id": c.map(|c| c.id.clone()), "rule": c.map(|c| c.rule.clone()), "scope": "pair"}));
        }
    }
    json!({"cancellations": out, "rule": rule()})
}

/// Each of `from`'s seven grahas read in `to`'s BAV of the same graha and in `to`'s SAV.
#[must_use]
pub fn pair_ashtakavarga(from: &BTreeMap<String, usize>, to_av: Option<&Value>) -> Value {
    let Some(av) = to_av else {
        return crate::methods::unavailable(
            "the other chart's Aṣṭakavarga needs its Ascendant",
            &["ascendant"],
        );
    };
    let mut out = serde_json::Map::new();
    for g in SEVEN {
        if let Some(&s) = from.get(g) {
            out.insert(
                g.into(),
                json!({"sign": s, "bav": av["bhinnashtakavarga"][g]["bindus"][s], "sav": av["sarvashtakavarga"]["bindus"][s],
                       "cells_read": {"bav": format!("{g}[{s}]"), "sav": format!("sav[{s}]")}}),
            );
        }
    }
    Value::Object(out)
}

/// Sign aspects from `from`'s grahas onto `to`'s grahas and lagna.
#[must_use]
pub fn cross_aspects(from: &BTreeMap<String, usize>, to: &BTreeMap<String, usize>) -> Value {
    let sp = &pack().cross_aspects.special;
    let mut out = vec![];
    for g in SEVEN.iter().copied().chain(["mean_node", "true_node"]) {
        let Some(&gs) = from.get(g) else { continue };
        let mut hs = vec![7];
        hs.extend(sp.get(g).cloned().unwrap_or_default());
        for (t, &ts) in to {
            if t == "south_node" {
                continue;
            }
            let h = (ts + 12 - gs) % 12 + 1;
            if hs.contains(&h) {
                out.push(json!({"from": g, "to": t, "house_from_aspecting": h}));
            }
            if h == 1 {
                out.push(json!({"from": g, "to": t, "conjunction": true}));
            }
        }
    }
    json!(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rajju_groups_have_their_members() {
        let mut n: BTreeMap<&str, usize> = BTreeMap::new();
        for k in 0..27 {
            *n.entry(rajju_of(k)).or_default() += 1;
        }
        assert_eq!(n["pada"], 6);
        assert_eq!(n["kati"], 6);
        assert_eq!(n["nabhi"], 6);
        assert_eq!(n["kantha"], 6);
        assert_eq!(n["shiras"], 3);
        // the head of the rope: Mṛgaśirā, Citrā, Dhaniṣṭhā
        assert_eq!([rajju_of(4), rajju_of(13), rajju_of(22)], ["shiras"; 3]);
        assert_eq!([rajju_of(0), rajju_of(8), rajju_of(9), rajju_of(26)], ["pada"; 4]);
    }

    #[test]
    fn vedha_is_symmetric_and_covers_every_nakshatra() {
        for x in 0..27 {
            assert!(
                (0..27).any(|y| in_vedha(x, y)),
                "nakshatra {x} has no vedha partner"
            );
            for y in 0..27 {
                assert_eq!(in_vedha(x, y), in_vedha(y, x));
            }
            assert!(!in_vedha(x, x));
        }
        // the classical pairs: Aśvinī–Jyeṣṭhā, Āśleṣā–Mūla, Maghā–Revatī; not Āśleṣā–Maghā
        assert!(in_vedha(0, 17) && in_vedha(8, 18) && in_vedha(9, 26) && !in_vedha(8, 9));
        // the head nakṣatras obstruct each other
        assert!(in_vedha(4, 13) && in_vedha(13, 22) && in_vedha(4, 22));
    }

    #[test]
    fn directional_factors_differ_by_order_and_symmetric_ones_do_not() {
        let deg = |nak: usize| nak as f64 * 360.0 / 27.0 + 1.0;
        // b in Aśvinī (0), a in Rohiṇī (3): count from b to a = 4 → Mahendra; reversed: 25 → also Mahendra,
        // so take a in Kṛttikā (2): count 3 (no), reversed 26 (no).
        let f = moon_factors(deg(3), deg(0));
        assert_eq!(f[2]["count_from_b"], 4);
        assert_eq!(f[2]["present"], true);
        let g = moon_factors(deg(0), deg(3));
        assert_eq!(g[2]["count_from_b"], 25);
        // Strī-dīrgha: count 4 is not > 13; reversed 25 is
        assert_eq!(f[3]["present"], false);
        assert_eq!(g[3]["present"], true);
        // symmetric ones agree in both orders
        assert_eq!(f[0]["dosha"], g[0]["dosha"]);
        assert_eq!(f[1]["dosha"], g[1]["dosha"]);
    }

    #[test]
    fn manglik_trace_keeps_raw_and_each_cancellation() {
        // Mars in Meṣa (own sign), lagna in Meṣa: 1st house → raw, cancelled by own_or_exalted
        let s: BTreeMap<String, usize> = [
            ("mars", 0),
            ("lagna", 0),
            ("moon", 6),
            ("venus", 3),
            ("jupiter", 2),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
        let m = manglik(&s);
        let l = &m["by_reference"]["lagna"];
        assert_eq!(l["raw"], true);
        assert_eq!(l["cancellations"][0]["id"], "own_or_exalted");
        assert_eq!(l["after_cancellations"], false);
        // from the Moon in Tulā, Mars is in the 7th: raw, also own sign
        assert_eq!(m["by_reference"]["moon"]["mars_house"], 7);
        // from Venus in Karka Mars is in the 10th: no doṣa, no cancellations listed
        assert_eq!(m["by_reference"]["venus"]["raw"], false);
        assert_eq!(
            m["by_reference"]["venus"]["cancellations"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        // no lagna → that reference is unavailable, not "no doṣa"
        let mut s2 = s.clone();
        s2.remove("lagna");
        assert_eq!(
            manglik(&s2)["by_reference"]["lagna"]["availability"]["status"],
            "unavailable"
        );
    }

    #[test]
    fn seventh_house_cross_aspects_are_mutual() {
        let a: BTreeMap<String, usize> = [("sun".to_string(), 0)].into_iter().collect();
        let b: BTreeMap<String, usize> = [("moon".to_string(), 6)].into_iter().collect();
        let ab = cross_aspects(&a, &b);
        let a2: BTreeMap<String, usize> = [("moon".to_string(), 6)].into_iter().collect();
        let b2: BTreeMap<String, usize> = [("sun".to_string(), 0)].into_iter().collect();
        let ba = cross_aspects(&a2, &b2);
        assert_eq!(ab[0]["house_from_aspecting"], 7);
        assert_eq!(ba[0]["house_from_aspecting"], 7);
    }
}
