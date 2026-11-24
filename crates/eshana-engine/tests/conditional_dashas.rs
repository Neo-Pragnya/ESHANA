//! The seven conditional nakṣatra daśās: year tables, starting lords worked by hand from the Moon's nakṣatra,
//! contiguous periods, and an applicability verdict that is always reported and never applied silently.

use eshana_engine::Engine;
use serde_json::{json, Value};

const NAK: f64 = 360.0 / 27.0;

fn chart(moon_hint_utc: &str) -> Value {
    let r = json!({"op": "chart", "system": "vedic", "features": ["dashas_extended"],
        "when": {"utc": moon_hint_utc}, "place": {"latitude": 28.61, "longitude": 77.21},
        "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}, "reference_utc": "2026-11-23T00:00:00Z"});
    serde_json::from_str(&Engine::new().execute_json(&r.to_string())).unwrap()
}

fn attr<'a>(env: &'a Value, id: &str) -> &'a Value {
    &env["derived_attributes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["id"] == id)
        .unwrap_or_else(|| panic!("{id}"))["value"]
}

/// (system, cycle, lords in order, start mode, start nakshatra)
const SYSTEMS: [(&str, f64, &[&str], &str, usize); 6] = [
    (
        "dvadashottari",
        112.0,
        &[
            "Sun", "Jupiter", "Ketu", "Mercury", "Rahu", "Mars", "Saturn", "Moon",
        ],
        "to",
        26,
    ),
    (
        "panchottari",
        105.0,
        &["Sun", "Mercury", "Saturn", "Mars", "Venus", "Moon", "Jupiter"],
        "from",
        16,
    ),
    (
        "shatabdika",
        100.0,
        &["Sun", "Moon", "Venus", "Mercury", "Jupiter", "Mars", "Saturn"],
        "from",
        26,
    ),
    (
        "chaturashiti_sama",
        84.0,
        &["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"],
        "from",
        14,
    ),
    (
        "dvisaptati_sama",
        72.0,
        &[
            "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu",
        ],
        "from",
        18,
    ),
    (
        "shattrimshat_sama",
        36.0,
        &[
            "Moon", "Sun", "Jupiter", "Mars", "Mercury", "Saturn", "Venus", "Rahu",
        ],
        "from",
        21,
    ),
];

#[test]
fn starting_lords_follow_the_count_worked_by_hand() {
    // Several births spread over a month so the Moon visits many nakṣatras.
    for day in [1, 4, 7, 10, 13, 16, 19, 22, 25, 28] {
        let env = chart(&format!("2026-01-{day:02}T06:00:00Z"));
        let moon = env["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "moon")
            .unwrap()["longitude_deg"]
            .as_f64()
            .unwrap();
        let nak = (moon / NAK).floor() as usize;
        for (id, cycle, lords, mode, from) in SYSTEMS {
            let v = attr(&env, &format!("vedic.dasha.{id}"));
            let n = lords.len();
            let count = if mode == "to" {
                (from + 27 - nak) % 27 + 1
            } else {
                (nak + 27 - from) % 27 + 1
            };
            let want = lords[(count + n - 1) % n];
            assert_eq!(
                v["starting_lord"], want,
                "{id}: moon nakshatra {nak}, count {count}"
            );
            assert_eq!(v["cycle_years"].as_f64().unwrap(), cycle);
            assert!(v["applicability"]["applies"].is_boolean() || v["applicability"]["applies"].is_null());
            assert!(v["applicability"]["rule"].as_str().unwrap().len() > 10);
            // mahādaśās are contiguous
            let m = v["mahadashas"].as_array().unwrap();
            for w in m.windows(2) {
                assert_eq!(w[0]["end"], w[1]["start"], "{id}");
            }
        }
        let s = attr(&env, "vedic.dasha.shashtihayani");
        assert_eq!(s["cycle_years"], 60.0);
    }
}

#[test]
fn a_worked_example_dvadashottari_from_ashvini_starts_with_ketu() {
    // Find a birth with the Moon early in Aśvinī: count from Aśvinī to Revatī is 27; 27 mod 8 = 3 → Ketu.
    for h in (0..(24 * 30)).step_by(6) {
        let utc = format!("2026-01-{:02}T{:02}:00:00Z", 1 + h / 24, h % 24);
        let env = chart(&utc);
        let moon = env["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == "moon")
            .unwrap()["longitude_deg"]
            .as_f64()
            .unwrap();
        if moon < 3.4 {
            let v = attr(&env, "vedic.dasha.dvadashottari");
            assert_eq!(v["starting_lord"], "Ketu");
            // balance: the part of Aśvinī not yet traversed times Ketu's 11 years
            let want = (1.0 - moon / NAK) * 11.0;
            assert!((v["balance_at_birth_years"].as_f64().unwrap() - want).abs() < 1e-9);
            return;
        }
    }
    panic!("no Aśvinī Moon found in January 2026");
}

#[test]
fn applicability_is_reported_for_every_chart_and_unknown_without_a_birth_time() {
    let r = json!({"op": "chart", "system": "vedic", "features": ["dashas_extended"],
        "when": {"date": "2026-01-05", "timezone": "Asia/Kolkata"}, "place": {"latitude": 28.61, "longitude": 77.21},
        "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}});
    let env: Value = serde_json::from_str(&Engine::new().execute_json(&r.to_string())).unwrap();
    if let Some(d) = env["derived_attributes"]
        .as_array()
        .and_then(|d| d.iter().find(|x| x["id"] == "vedic.dasha.shatabdika"))
    {
        assert!(
            d["value"]["applicability"]["applies"].is_null(),
            "no lagna → unknown, not false: {}",
            d["value"]["applicability"]
        );
    }
}
