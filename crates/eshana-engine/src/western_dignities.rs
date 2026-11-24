//! Western essential dignities from the `western.dignities` rule pack (WES-04): domicile, exaltation, triplicity
//! (Dorothean, by sect), Egyptian terms, Chaldean faces, detriment and fall, the score, peregrine planets and mutual
//! receptions. Tropical longitudes; the seven classical planets only (the outer planets have no classical dignity).

use eshana_core::{norm360, EshanaError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pack source.
pub const SOURCE: &str = include_str!("../rulesets/western-dignities.yaml");

/// The parsed pack.
#[derive(Debug, Deserialize)]
pub struct Pack {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Named schemes.
    pub schemes: BTreeMap<String, String>,
    /// Point weights.
    pub scores: BTreeMap<String, i32>,
    /// Domicile ruler by sign.
    pub domicile: Vec<String>,
    /// Exaltation sign and degree by planet.
    pub exaltation: BTreeMap<String, (usize, f64)>,
    /// Triplicity rulers by element: day, night, participating.
    pub triplicity: Vec<Vec<String>>,
    /// Egyptian terms by sign: (ruler, end degree).
    pub terms: Vec<Vec<(String, f64)>>,
    /// Chaldean face order from 0° Aries.
    pub faces: Vec<String>,
}

/// The pack, parsed once.
///
/// # Panics
/// When the built-in pack does not parse (a build defect, caught by the tests).
pub fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| serde_norway::from_str(SOURCE).expect("western-dignities.yaml is valid"))
}

/// Rule id with version, as recorded in provenance.
#[must_use]
pub fn rule_id() -> String {
    format!("{}@{}", pack().id, pack().version)
}

/// The classical planets the table covers.
pub const CLASSICAL: [&str; 7] = ["sun", "moon", "mercury", "venus", "mars", "jupiter", "saturn"];

fn domicile_of(sign: usize) -> &'static str {
    &pack().domicile[sign % 12]
}

/// The term ruler at a tropical longitude.
#[must_use]
pub fn term_ruler(lon: f64) -> &'static str {
    let (sign, deg) = ((norm360(lon) / 30.0) as usize, norm360(lon) % 30.0);
    pack().terms[sign]
        .iter()
        .find(|(_, end)| deg < *end)
        .map_or("", |(r, _)| r.as_str())
}

/// The face (decan) ruler at a tropical longitude.
#[must_use]
pub fn face_ruler(lon: f64) -> &'static str {
    let decan = (norm360(lon) / 10.0) as usize;
    &pack().faces[decan % 7]
}

/// The essential dignities of each classical planet. `day` is the sect (Sun above the horizon), `None` when the
/// birth time or place is not known; triplicity is then reported but not scored.
///
/// # Errors
/// Never in practice; kept for the pack contract.
pub fn dignities(lons: &BTreeMap<String, f64>, day: Option<bool>) -> Result<Value> {
    let p = pack();
    let score = |k: &str| *p.scores.get(k).unwrap_or(&0);
    let mut planets = serde_json::Map::new();
    for g in CLASSICAL {
        let Some(&lon) = lons.get(g) else { continue };
        let sign = (norm360(lon) / 30.0) as usize;
        let mut has = vec![];
        if domicile_of(sign) == g {
            has.push("domicile");
        }
        if domicile_of(sign + 6) == g {
            has.push("detriment");
        }
        if let Some(&(s, _)) = p.exaltation.get(g) {
            if s == sign {
                has.push("exaltation");
            }
            if (s + 6) % 12 == sign {
                has.push("fall");
            }
        }
        let trip = &p.triplicity[sign % 4];
        let trip_ruler = match day {
            Some(true) => Some(trip[0].as_str()),
            Some(false) => Some(trip[1].as_str()),
            None => None,
        };
        if trip_ruler == Some(g) || (day.is_some() && trip[2] == g) {
            has.push("triplicity");
        }
        if term_ruler(lon) == g {
            has.push("term");
        }
        if face_ruler(lon) == g {
            has.push("face");
        }
        let total: i32 = has.iter().map(|k| score(k)).sum();
        let essential = has
            .iter()
            .any(|k| matches!(*k, "domicile" | "exaltation" | "triplicity" | "term" | "face"));
        planets.insert(g.into(), json!({
            "longitude_deg": lon, "sign": sign, "dignities": has, "score": total,
            "peregrine": !essential,
            "rulers": {"domicile": domicile_of(sign), "exaltation": p.exaltation.iter().find(|(_, v)| v.0 == sign).map(|(k, _)| k),
                       "triplicity": {"day": trip[0], "night": trip[1], "participating": trip[2]},
                       "term": term_ruler(lon), "face": face_ruler(lon)},
        }));
    }
    // mutual reception: each in the other's domicile, or in the other's exaltation
    let mut receptions = vec![];
    let sign_of = |g: &str| lons.get(g).map(|l| (norm360(*l) / 30.0) as usize);
    for (i, a) in CLASSICAL.iter().enumerate() {
        for b in &CLASSICAL[i + 1..] {
            let (Some(sa), Some(sb)) = (sign_of(a), sign_of(b)) else {
                continue;
            };
            if domicile_of(sa) == *b && domicile_of(sb) == *a {
                receptions.push(json!({"between": [a, b], "by": "domicile"}));
            }
            let ex = |g: &str| p.exaltation.get(g).map(|e| e.0);
            if ex(b) == Some(sa) && ex(a) == Some(sb) {
                receptions.push(json!({"between": [a, b], "by": "exaltation"}));
            }
        }
    }
    if planets.is_empty() {
        return Err(EshanaError::Input(
            "essential dignities need the classical planets".into(),
        ));
    }
    Ok(
        json!({"schemes": p.schemes, "scores": p.scores, "sect": day.map(|d| if d { "day" } else { "night" }),
              "planets": planets, "mutual_receptions": receptions}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egyptian_terms_have_the_classical_totals() {
        // each sign's terms reach 30°, and the planets' totals are the classical 79, 82, 76, 66, 57 (Ptolemy I.20)
        let mut total: BTreeMap<&str, f64> = BTreeMap::new();
        for sign in &pack().terms {
            let mut start = 0.0;
            for (r, end) in sign {
                *total.entry(r.as_str()).or_default() += end - start;
                start = *end;
            }
            assert!((start - 30.0).abs() < 1e-9);
        }
        let want = [
            ("jupiter", 79.0),
            ("venus", 82.0),
            ("mercury", 76.0),
            ("mars", 66.0),
            ("saturn", 57.0),
        ];
        for (g, w) in want {
            assert!((total[g] - w).abs() < 1e-9, "{g}: {} vs {w}", total[g]);
        }
    }

    #[test]
    fn known_dignities() {
        let lons: BTreeMap<String, f64> = [
            ("sun", 10.0),
            ("moon", 33.0),
            ("mars", 290.0),
            ("venus", 355.0),
            ("saturn", 15.0),
            ("mercury", 170.0),
            ("jupiter", 100.0),
        ]
        .iter()
        .map(|(k, v)| ((*k).to_string(), *v))
        .collect();
        let d = dignities(&lons, Some(true)).unwrap();
        let has = |g: &str, k: &str| {
            d["planets"][g]["dignities"]
                .as_array()
                .unwrap()
                .iter()
                .any(|x| x == k)
        };
        assert!(has("sun", "exaltation") && has("sun", "triplicity")); // Sun in Aries by day
        assert!(has("moon", "exaltation")); // Moon in Taurus
        assert!(has("mars", "exaltation")); // Mars in Capricorn
        assert!(has("venus", "exaltation")); // Venus in Pisces
        assert!(has("saturn", "fall")); // Saturn in Aries
        assert!(has("mercury", "domicile") && has("mercury", "exaltation")); // Mercury in Virgo
        assert!(has("jupiter", "exaltation")); // Jupiter in Cancer
        assert_eq!(face_ruler(0.0), "mars");
        assert_eq!(face_ruler(355.0), "mars"); // the 36th decan: 35 mod 7 = 0
        assert_eq!(term_ruler(5.0), "jupiter");
    }
}
