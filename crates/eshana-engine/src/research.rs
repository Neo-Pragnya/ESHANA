//! Research models, each specified first in `docs/design/research-models.md` (RES-01) and labelled experimental.
//!
//! `research.window_score@0.1` orders muhūrta windows that already passed their constraints: a weighted mean of six
//! factors, with the weight of an unavailable factor redistributed proportionally over the available ones. It never
//! makes a failing window pass, and it is a research hypothesis, not a reading.

use eshana_core::{EshanaError, Result};
use serde_json::{json, Value};

/// Model id with version.
pub const WINDOW_SCORE: &str = "research.window_score@0.1";
/// Factor weights (sum 1).
pub const WEIGHTS: [(&str, f64); 6] = [
    ("tara_bala", 0.25),
    ("chandra_bala", 0.20),
    ("choghadiya", 0.20),
    ("hora", 0.15),
    ("abhijit", 0.10),
    ("duration", 0.10),
];
/// Below this share of the weights the score is not computed.
pub const MIN_COVERAGE: f64 = 0.5;

fn available(v: &Value) -> bool {
    !v.is_null() && v.get("availability").is_none()
}

/// The value in [0, 1] of one factor in one window, or `None` when it is missing or unavailable.
#[must_use]
pub fn factor_value(name: &str, w: &Value) -> Option<f64> {
    let f = &w["factors"];
    let q = |k: &str| -> Option<&str> { available(&f[k]).then(|| f[k]["quality"].as_str()).flatten() };
    match name {
        "tara_bala" | "chandra_bala" => q(name).map(|s| if s == "favourable" { 1.0 } else { 0.0 }),
        "choghadiya" => q("choghadiya").map(|s| match s {
            "auspicious" => 1.0,
            "neutral" => 0.5,
            _ => 0.0,
        }),
        "hora" => available(&f["hora"])
            .then(|| f["hora"]["lord"].as_str())
            .flatten()
            .map(|l| {
                if ["jupiter", "venus", "mercury", "moon"].contains(&l) {
                    1.0
                } else {
                    0.5
                }
            }),
        "abhijit" => {
            if !available(&f["abhijit"]) {
                return None;
            }
            let wed = available(&f["vara"]) && f["vara"]["index"] == 3;
            Some(if f["abhijit"] == true && !wed { 1.0 } else { 0.0 })
        }
        "duration" => w["duration_minutes"].as_f64().map(|m| (m / 96.0).clamp(0.0, 1.0)),
        _ => None,
    }
}

/// Score one window: contributions, sensitivity, missing factors and coverage.
#[must_use]
pub fn score_window(w: &Value) -> Value {
    let vals: Vec<(&str, f64, Option<f64>)> = WEIGHTS
        .iter()
        .map(|(n, wt)| (*n, *wt, factor_value(n, w)))
        .collect();
    let coverage: f64 = vals.iter().filter(|x| x.2.is_some()).map(|x| x.1).sum();
    let missing: Vec<&str> = vals.iter().filter(|x| x.2.is_none()).map(|x| x.0).collect();
    if coverage < MIN_COVERAGE - 1e-12 {
        return json!({"model": WINDOW_SCORE, "score": crate::methods::unavailable(
            "fewer than half of the model's weights have an input", &missing), "coverage": coverage, "missing": missing});
    }
    let mut contributions = serde_json::Map::new();
    let mut sensitivity = serde_json::Map::new();
    let mut score = 0.0;
    for (n, wt, v) in &vals {
        if let Some(s) = v {
            let w2 = wt / coverage;
            contributions.insert((*n).into(), json!(100.0 * w2 * s));
            sensitivity.insert((*n).into(), json!(100.0 * w2));
            score += 100.0 * w2 * s;
        }
    }
    json!({"model": WINDOW_SCORE, "score": score, "contributions": contributions, "sensitivity": sensitivity,
           "missing": missing, "coverage": coverage, "redistribution": "proportional over the available factors"})
}

/// Score and order windows (highest first; ties keep their time order).
///
/// # Errors
/// `ESH-INPUT` when `windows` is not a list.
pub fn rank(windows: &Value) -> Result<Value> {
    let ws = windows
        .as_array()
        .ok_or_else(|| EshanaError::Input("windows must be a list".into()))?;
    let mut scored: Vec<(usize, Value)> = ws
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let mut s = score_window(w);
            s["start_utc"] = w["start_utc"].clone();
            s["end_utc"] = w["end_utc"].clone();
            (i, s)
        })
        .collect();
    let key = |v: &Value| v["score"].as_f64().unwrap_or(f64::NEG_INFINITY);
    scored.sort_by(|a, b| key(&b.1).total_cmp(&key(&a.1)).then(a.0.cmp(&b.0)));
    Ok(json!(scored.into_iter().map(|(_, v)| v).collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window(tara: Option<&str>, chandra: Option<&str>) -> Value {
        let mut f = json!({"choghadiya": {"name": "Labh", "quality": "auspicious"}, "hora": {"lord": "saturn"},
                           "abhijit": false, "vara": {"index": 2}});
        if let Some(t) = tara {
            f["tara_bala"] = json!({"quality": t});
        }
        if let Some(c) = chandra {
            f["chandra_bala"] = json!({"quality": c});
        }
        json!({"factors": f, "duration_minutes": 120.0, "start_utc": "x", "end_utc": "y"})
    }

    #[test]
    fn worked_example_a_all_factors() {
        let s = score_window(&window(Some("favourable"), Some("unfavourable")));
        assert!((s["score"].as_f64().unwrap() - 62.5).abs() < 1e-9, "{s}");
        let c = &s["contributions"];
        let sum: f64 = c.as_object().unwrap().values().map(|v| v.as_f64().unwrap()).sum();
        assert!((sum - 62.5).abs() < 1e-9);
        assert!((c["hora"].as_f64().unwrap() - 7.5).abs() < 1e-9);
        assert!((s["sensitivity"]["tara_bala"].as_f64().unwrap() - 25.0).abs() < 1e-9);
        assert_eq!(s["coverage"], 1.0);
    }

    #[test]
    fn worked_example_b_missing_birth_chart_redistributes() {
        let s = score_window(&window(None, None));
        assert!(
            (s["score"].as_f64().unwrap() - 68.181_818_181_8).abs() < 1e-6,
            "{s}"
        );
        assert!((s["coverage"].as_f64().unwrap() - 0.55).abs() < 1e-12);
        assert_eq!(s["missing"], json!(["tara_bala", "chandra_bala"]));
        assert!((s["contributions"]["choghadiya"].as_f64().unwrap() - 36.363_636).abs() < 1e-5);
    }

    #[test]
    fn sensitivity_is_the_change_when_one_factor_flips() {
        let lo = score_window(&window(Some("unfavourable"), Some("unfavourable")));
        let hi = score_window(&window(Some("favourable"), Some("unfavourable")));
        let d = hi["score"].as_f64().unwrap() - lo["score"].as_f64().unwrap();
        assert!((d - lo["sensitivity"]["tara_bala"].as_f64().unwrap()).abs() < 1e-9);
    }

    #[test]
    fn too_little_coverage_is_unavailable_and_ranking_is_stable() {
        let w = json!({"factors": {"hora": {"lord": "sun"}}, "duration_minutes": 50.0});
        assert_eq!(score_window(&w)["score"]["status"], "unavailable");
        let r = rank(&json!([
            window(None, None),
            window(Some("favourable"), Some("favourable")),
            w
        ]))
        .unwrap();
        let scores: Vec<Value> = r.as_array().unwrap().iter().map(|x| x["score"].clone()).collect();
        assert!(scores[0].as_f64().unwrap() >= scores[1].as_f64().unwrap());
        assert_eq!(scores[2]["status"], "unavailable");
    }
}
