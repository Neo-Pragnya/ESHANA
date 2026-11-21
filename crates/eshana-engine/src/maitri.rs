//! Maitri (relationship) tables for the nine grahas, the nakshatra spatial matrix (tārās between the lagna and
//! the grahas) and dignities in the sixteen divisions.
//!
//! Tables and every convention choice live in `rulesets/maitri.yaml` (ADR-012); natural friendship of the seven
//! grahas, temporary-friend houses and the BPHS dignity ranges come from `vedic-core.yaml` through
//! [`crate::jyotish`]. Attributes: `vedic.maitri.relationships`, `vedic.maitri.tara_matrix`,
//! `vedic.maitri.varga_dignities`, all under rule `vedic.maitri@0.1`.

use crate::analysis::{AnalysisCtx, Derived};
use crate::jyotish::{self, house_from, lord_of, sign_of, VChart, NINE};
use crate::packs::{self, Friendship};
use crate::vedic;
use eshana_core::{norm360, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Rule id of every attribute in this module.
pub const RULE: &str = "vedic.maitri@0.1";

/// Raw YAML of the maitri pack.
pub const SOURCE: &str = include_str!("../rulesets/maitri.yaml");

/// A node's dignity signs (indices 0 = Aries … 11 = Pisces).
#[derive(Debug, Clone, Deserialize)]
pub struct NodeDignity {
    /// Exaltation sign.
    pub exaltation: usize,
    /// Debilitation sign.
    pub debilitation: usize,
    /// Mūlatrikoṇa sign.
    pub moolatrikona: usize,
    /// Own signs.
    pub own: Vec<usize>,
}

/// Mercury's division of Virgo.
#[derive(Debug, Clone, Deserialize)]
pub struct MercuryVirgo {
    /// Exalted below this degree.
    pub exalted_below: f64,
    /// Mūlatrikoṇa below this degree (own above).
    pub moolatrikona_below: f64,
}

/// One division of the sixteen.
#[derive(Debug, Clone, Deserialize)]
pub struct Division {
    /// Divisor.
    pub d: u32,
    /// Name.
    pub name: String,
}

/// One tārā.
#[derive(Debug, Clone, Deserialize)]
pub struct Tara {
    /// Name.
    pub name: String,
    /// `auspicious`, `inauspicious` or `mixed`.
    pub nature: String,
}

/// The maitri pack.
#[derive(Debug, Clone, Deserialize)]
pub struct MaitriPack {
    /// Natural relationships of Rāhu and Ketu towards the other grahas.
    pub node_natural: BTreeMap<String, Friendship>,
    /// How each of the seven regards Rāhu and Ketu.
    pub towards_nodes: BTreeMap<String, BTreeMap<String, String>>,
    /// Node dignities.
    pub node_dignity: BTreeMap<String, NodeDignity>,
    /// Mercury in Virgo.
    pub mercury_virgo: MercuryVirgo,
    /// Divisions where mūlatrikoṇa is decided by the sign alone.
    pub moolatrikona_by_sign_only: Vec<u32>,
    /// The sixteen divisions.
    pub divisions: Vec<Division>,
    /// The nine tārās in order.
    pub taras: Vec<Tara>,
}

/// The parsed pack (a pack that does not parse is a build defect, caught by the tests).
///
/// # Panics
/// If the built-in YAML is invalid.
pub fn pack() -> &'static MaitriPack {
    static P: OnceLock<MaitriPack> = OnceLock::new();
    P.get_or_init(|| {
        serde_norway::from_str(SOURCE).unwrap_or_else(|e| panic!("rule pack maitri.yaml is invalid: {e}"))
    })
}

const NODES: [&str; 2] = ["rahu", "ketu"];

/// Natural relationship of `a` towards `b` for any of the nine (`friend`, `neutral`, `enemy`).
#[must_use]
pub fn natural(a: &str, b: &str) -> &'static str {
    let p = pack();
    if let Some(f) = p.node_natural.get(a) {
        return if f.friends.iter().any(|x| x == b) {
            "friend"
        } else if f.enemies.iter().any(|x| x == b) {
            "enemy"
        } else {
            "neutral"
        };
    }
    if NODES.contains(&b) {
        return match p.towards_nodes.get(a).and_then(|m| m.get(b)).map(String::as_str) {
            Some("friend") => "friend",
            Some("enemy") => "enemy",
            _ => "neutral",
        };
    }
    jyotish::natural_relation(a, b)
}

/// Temporary relationship: is sign `sb` a friend's place counted from `sa`?
#[must_use]
pub fn temporary_friend(sa: usize, sb: usize) -> bool {
    packs::core()
        .temporary_friend_houses
        .contains(&house_from(sa, sb))
}

/// Compound (pañcadhā) relationship from a natural relationship and the temporary one.
#[must_use]
pub fn compound(natural: &str, temp_friend: bool) -> &'static str {
    match (natural, temp_friend) {
        ("friend", true) => "great_friend",
        ("friend", false) | ("enemy", true) => "neutral",
        ("neutral", true) => "friend",
        ("neutral", false) => "enemy",
        _ => "great_enemy",
    }
}

/// Every ordered pair of the nine grahas with natural, temporary and compound relationships.
#[must_use]
pub fn relationships(vc: &VChart) -> Value {
    let mut rows = vec![];
    for a in NINE {
        let Some(la) = vc.lon.get(a) else { continue };
        for b in NINE {
            if a == b {
                continue;
            }
            let Some(lb) = vc.lon.get(b) else { continue };
            let house = house_from(sign_of(*la), sign_of(*lb));
            let nat = natural(a, b);
            let tf = temporary_friend(sign_of(*la), sign_of(*lb));
            let source = if NODES.contains(&a) || NODES.contains(&b) {
                "maitri.yaml node table"
            } else {
                "vedic-core.yaml (BPHS)"
            };
            rows.push(json!({
                "from": a, "to": b, "natural": nat, "natural_source": source,
                "temporal": if tf { "friend" } else { "enemy" }, "temporal_house": house,
                "compound": compound(nat, tf),
            }));
        }
    }
    json!({
        "grahas": NINE,
        "rows": rows,
        "convention": REL_CONVENTION,
    })
}

const REL_CONVENTION: &str = "natural (BPHS for the seven; the node table in maitri.yaml) + temporary (2, 3, 4, 10, 11, 12 from the graha are friends) = compound (pañcadhā)";
const TARA_CONVENTION: &str = "count = ((row nakshatra − column nakshatra) mod 27) + 1, counted from the column to the row; tārā = ((count − 1) mod 9) + 1";
const DIG_CONVENTION: &str = "exaltation sign (whole sign; Mercury in Virgo split 0–15° exalted, 15–20° mūlatrikoṇa, 20–30° own), debilitation sign, mūlatrikoṇa by the varga degree (by sign in D30), own sign, else the compound relationship to the sign lord with the temporary part from the same varga; nodes: exaltation, debilitation, mūlatrikoṇa and own signs from maitri.yaml, otherwise neutral";

/// Tārā (1–9) of nakshatra `to` counted from nakshatra `from`, with the count (1–27).
#[must_use]
pub fn tara(from: usize, to: usize) -> (usize, usize) {
    let count = (to + 27 - from % 27) % 27 + 1;
    (count, (count - 1) % 9 + 1)
}

/// The nakshatra spatial matrix for the lagna and the nine grahas.
#[must_use]
pub fn tara_matrix(vc: &VChart) -> Value {
    let p = pack();
    let mut pts: Vec<(&str, usize)> = vec![];
    if let Some(a) = vc.asc {
        pts.push(("lagna", vedic::nakshatra_of(a).index));
    }
    for g in NINE {
        if let Some(l) = vc.lon.get(g) {
            pts.push((g, vedic::nakshatra_of(*l).index));
        }
    }
    let mut rows = vec![];
    for (r, nr) in &pts {
        for (c, nc) in &pts {
            if r == c {
                continue;
            }
            let (count, t) = tara(*nc, *nr);
            let tt = &p.taras[t - 1];
            rows.push(json!({"row": r, "column": c, "count": count, "tara": t, "name": tt.name, "nature": tt.nature}));
        }
    }
    let nak: Map<String, Value> = pts
        .iter()
        .map(|(k, n)| {
            (
                (*k).to_string(),
                json!({"index": n, "name": vedic::NAKSHATRAS[*n]}),
            )
        })
        .collect();
    json!({
        "points": pts.iter().map(|x| x.0).collect::<Vec<_>>(),
        "nakshatra": nak,
        "rows": rows,
        "taras": p.taras.iter().map(|t| json!({"name": t.name, "nature": t.nature})).collect::<Vec<_>>(),
        "convention": TARA_CONVENTION,
    })
}

/// Degree inside the varga sign: `(longitude × D) mod 30`, or `None` where the pack decides by sign only.
fn varga_degree(lon: f64, d: u32) -> Option<f64> {
    if pack().moolatrikona_by_sign_only.contains(&d) {
        None
    } else {
        Some(norm360(lon * f64::from(d)) % 30.0)
    }
}

/// Dignity of graha `g` in varga sign `sign` (degree `deg` inside it when known), with `signs` the varga signs
/// of all grahas (for the temporary part of the compound relationship). Returns the dignity and the reason.
#[must_use]
pub fn varga_dignity(
    g: &str,
    sign: usize,
    deg: Option<f64>,
    signs: &BTreeMap<&str, usize>,
) -> (&'static str, String) {
    let p = pack();
    if let Some(nd) = p.node_dignity.get(g) {
        return if sign == nd.exaltation {
            ("exalted", "exaltation sign".into())
        } else if sign == nd.debilitation {
            ("debilitated", "debilitation sign".into())
        } else if sign == nd.moolatrikona {
            ("moolatrikona", "mūlatrikoṇa sign".into())
        } else if nd.own.contains(&sign) {
            ("own", "own sign".into())
        } else {
            ("neutral", "a node in any other sign is listed as neutral".into())
        };
    }
    let c = packs::core();
    if let Some(d) = c.dignity.get(g) {
        if g == "mercury" && sign == 5 {
            let mv = &p.mercury_virgo;
            return match deg {
                Some(x) if x >= mv.moolatrikona_below => {
                    ("own", format!("Virgo {x:.1}°: the own part (20–30°)"))
                }
                Some(x) if x >= mv.exalted_below => (
                    "moolatrikona",
                    format!("Virgo {x:.1}°: the mūlatrikoṇa part (15–20°)"),
                ),
                Some(x) => ("exalted", format!("Virgo {x:.1}°: the exalted part (0–15°)")),
                None => ("exalted", "Virgo (decided by sign)".into()),
            };
        }
        if d.exaltation[0] as usize == sign {
            return ("exalted", "exaltation sign".into());
        }
        if d.debilitation[0] as usize == sign {
            return ("debilitated", "debilitation sign".into());
        }
        if let Some(m) = d.moolatrikona {
            if m[0] as usize == sign {
                match deg {
                    None => return ("moolatrikona", "mūlatrikoṇa sign (decided by sign)".into()),
                    Some(x) if x >= m[1] && x < m[2] => {
                        return (
                            "moolatrikona",
                            format!("{x:.1}° inside the mūlatrikoṇa range {}–{}°", m[1], m[2]),
                        );
                    }
                    _ => {}
                }
            }
        }
        if d.own.contains(&sign) {
            return ("own", "own sign".into());
        }
    }
    let lord = lord_of(sign);
    if lord == g {
        return ("own", "own sign".into());
    }
    let nat = natural(g, lord);
    let (Some(sg), Some(sl)) = (signs.get(g), signs.get(lord)) else {
        return (nat, format!("natural relationship to {lord}"));
    };
    let tf = temporary_friend(*sg, *sl);
    let temp = if tf { "friend" } else { "enemy" };
    let why = format!(
        "{lord} rules the sign: natural {nat}, temporary {temp} ({} from {g} in this varga)",
        house_from(*sg, *sl)
    );
    (compound(nat, tf), why)
}

/// Dignities of the nine grahas in the sixteen divisions, with a per-graha summary.
///
/// # Errors
/// A division the engine does not implement (never the case for the sixteen in the pack).
pub fn varga_dignities(vc: &VChart) -> Result<Value> {
    let p = pack();
    let mut divisions = vec![];
    let mut table: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for dv in &p.divisions {
        let mut signs: BTreeMap<&str, usize> = BTreeMap::new();
        for g in NINE {
            if let Some(l) = vc.lon.get(g) {
                signs.insert(g, vedic::divisional_sign(*l, dv.d)?);
            }
        }
        for g in NINE {
            let (Some(l), Some(s)) = (vc.lon.get(g), signs.get(g)) else {
                continue;
            };
            let deg = varga_degree(*l, dv.d);
            let (dig, why) = varga_dignity(g, *s, deg, &signs);
            table.entry(g).or_default().push(json!({
                "d": dv.d, "sign": s, "degree": deg, "lord": lord_of(*s), "dignity": dig, "reason": why,
            }));
        }
        divisions.push(json!({"d": dv.d, "name": dv.name}));
    }
    let mut rows = Map::new();
    let mut summary = Map::new();
    for g in NINE {
        let Some(cells) = table.remove(g) else { continue };
        let mut counts: BTreeMap<String, u32> = BTreeMap::new();
        for c in &cells {
            *counts
                .entry(c["dignity"].as_str().unwrap_or("").to_string())
                .or_default() += 1;
        }
        let n = |k: &str| counts.get(k).copied().unwrap_or(0);
        let dignified = n("exalted") + n("moolatrikona") + n("own");
        let friendly = n("great_friend") + n("friend");
        let unfavourable = n("enemy") + n("great_enemy") + n("debilitated");
        summary.insert(
            g.into(),
            json!({
                "counts": counts, "dignified": dignified, "friendly": friendly,
                "favourable": dignified + friendly, "unfavourable": unfavourable,
            }),
        );
        rows.insert(g.into(), Value::Array(cells));
    }
    Ok(json!({
        "divisions": divisions,
        "grahas": rows,
        "summary": summary,
        "convention": DIG_CONVENTION,
    }))
}

/// Compute this module's attributes for a chart.
///
/// # Errors
/// Propagates divisional-chart errors.
pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>> {
    let d = |id: &str, value: Value| Derived {
        id: id.into(),
        rule: RULE.into(),
        value,
        from: vec!["positions".to_string()],
    };
    Ok(vec![
        d("vedic.maitri.relationships", relationships(x.vc)),
        d("vedic.maitri.tara_matrix", tara_matrix(x.vc)),
        d("vedic.maitri.varga_dignities", varga_dignities(x.vc)?),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sidereal longitudes of the reference chart (Sri Ramana Maharshi, 30 Dec 1879, Lahiri, true node).
    fn ramana() -> VChart {
        let mut c = VChart {
            asc: Some(178.72),
            ..Default::default()
        };
        for (k, v) in [
            ("sun", 255.595),
            ("moon", 88.328),
            ("mars", 21.978),
            ("mercury", 233.109),
            ("jupiter", 316.535),
            ("venus", 210.478),
            ("saturn", 347.023),
            ("rahu", 263.336),
            ("ketu", 83.336),
        ] {
            c.lon.insert(k.into(), v);
        }
        c
    }

    fn find(v: &Value, f: impl Fn(&Value) -> bool) -> Value {
        v["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| f(r))
            .unwrap()
            .clone()
    }

    #[test]
    fn pack_parses_with_sixteen_divisions_and_nine_taras() {
        let p = pack();
        assert_eq!(p.divisions.len(), 16);
        assert_eq!(p.taras.len(), 9);
        for dv in &p.divisions {
            assert!(vedic::divisional_sign(10.0, dv.d).is_ok(), "D{}", dv.d);
        }
    }

    #[test]
    fn compound_table() {
        assert_eq!(compound("friend", true), "great_friend");
        assert_eq!(compound("friend", false), "neutral");
        assert_eq!(compound("neutral", true), "friend");
        assert_eq!(compound("neutral", false), "enemy");
        assert_eq!(compound("enemy", true), "neutral");
        assert_eq!(compound("enemy", false), "great_enemy");
    }

    #[test]
    fn tara_counts() {
        // same nakshatra: Janma; the next: Sampat; the 10th: Janma again; the 27th: the 9th tārā
        assert_eq!(tara(5, 5), (1, 1));
        assert_eq!(tara(5, 6), (2, 2));
        assert_eq!(tara(0, 9), (10, 1));
        assert_eq!(tara(3, 2), (27, 9));
    }

    #[test]
    fn worked_example_relationships() {
        // Sun in Sagittarius, Jupiter in Aquarius (3rd from the Sun): natural friend + temporary friend.
        let v = relationships(&ramana());
        let rel = |a: &str, b: &str| find(&v, |r| r["from"] == a && r["to"] == b)["compound"].clone();
        assert_eq!(rel("sun", "jupiter"), "great_friend");
        assert_eq!(rel("sun", "moon"), "neutral");
        assert_eq!(rel("rahu", "jupiter"), "great_friend");
        assert_eq!(rel("mars", "ketu"), "great_friend");
        assert_eq!(rel("rahu", "ketu"), "great_enemy");
        assert_eq!(v["rows"].as_array().unwrap().len(), 72);
    }

    #[test]
    fn worked_example_taras() {
        // Lagna in Chitra (13), Sun in Purva Ashadha (19): counted from Purva Ashadha to Chitra is the 22nd, Kshema (4).
        let v = tara_matrix(&ramana());
        let cell = find(&v, |r| r["row"] == "lagna" && r["column"] == "sun");
        assert_eq!(cell["name"], "Kshema");
        assert_eq!(cell["count"], 22);
        assert_eq!(v["rows"].as_array().unwrap().len(), 90);
    }

    #[test]
    fn worked_example_varga_dignities() {
        let v = varga_dignities(&ramana()).unwrap();
        let dig = |g: &str, d: u64| {
            let cells = v["grahas"][g].as_array().unwrap();
            cells.iter().find(|c| c["d"] == d).unwrap()["dignity"]
                .as_str()
                .unwrap()
                .to_string()
        };
        assert_eq!(dig("sun", 9), "own"); // Leo 20.4° in D9: past the 0–20° mūlatrikoṇa
        assert_eq!(dig("sun", 16), "moolatrikona"); // Leo 9.5° in D16
        assert_eq!(dig("mercury", 27), "own"); // Virgo 23.9°
        assert_eq!(dig("mercury", 60), "exalted"); // Virgo 6.5°
        assert_eq!(dig("moon", 12), "exalted"); // Taurus: the exaltation sign wins
        assert_eq!(dig("jupiter", 30), "moolatrikona"); // by sign in D30
        assert_eq!(dig("saturn", 3), "great_enemy"); // Cancer; the Moon 8th from Saturn in D3
        assert_eq!(dig("rahu", 30), "moolatrikona");
        assert_eq!(dig("rahu", 1), "neutral");
        assert_eq!(dig("ketu", 4), "own");
        let total: u64 = v["summary"]["sun"]["counts"]
            .as_object()
            .unwrap()
            .values()
            .map(|x| x.as_u64().unwrap())
            .sum();
        assert_eq!(total, 16);
    }
}
