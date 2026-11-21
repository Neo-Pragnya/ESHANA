//! Engine results against the Swiss Ephemeris used as an offline oracle (numbers only; nothing from it ships).

use eshana_core::angular_diff;
use eshana_engine::Engine;
use serde_json::{json, Value};

fn oracle() -> Value {
    serde_json::from_str(include_str!("../../../benchmarks/golden/swiss_oracle.json")).unwrap()
}

fn run(e: &Engine, v: Value) -> Value {
    serde_json::from_str(&e.execute_json(&v.to_string())).unwrap()
}

fn pos<'a>(env: &'a Value, id: &str) -> &'a Value {
    env["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap()
}

#[test]
fn sidereal_sun_and_moon_match_swiss_for_every_ayanamsha() {
    let e = Engine::new();
    for row in oracle()["instants"].as_array().unwrap() {
        for (name, tol_sun, tol_moon) in [
            ("lahiri", 0.02, 0.02),
            ("raman", 0.02, 0.02),
            ("krishnamurti", 0.02, 0.02),
            ("fagan_bradley", 0.02, 0.02),
        ] {
            let env = run(
                &e,
                json!({"op": "positions", "when": {"utc": row["utc"]},
                "config": {"zodiac": "sidereal", "ayanamsha": name}}),
            );
            assert_eq!(env["kind"], "positions", "{env}");
            let sun = pos(&env, "sun")["longitude_deg"].as_f64().unwrap();
            let moon = pos(&env, "moon")["longitude_deg"].as_f64().unwrap();
            let (ws, wm) = (
                row["sid_sun"][name].as_f64().unwrap(),
                row["sid_moon"][name].as_f64().unwrap(),
            );
            assert!(
                angular_diff(sun, ws).abs() < tol_sun,
                "{} {name} sun {sun} vs {ws}",
                row["utc"]
            );
            assert!(
                angular_diff(moon, wm).abs() < tol_moon,
                "{} {name} moon {moon} vs {wm}",
                row["utc"]
            );
        }
    }
}

#[test]
fn lunar_nodes_match_swiss() {
    let e = Engine::new();
    for row in oracle()["instants"].as_array().unwrap() {
        for (model, key, tol) in [("mean", "mean_node", 0.01), ("true", "true_node", 0.06)] {
            let env = run(
                &e,
                json!({"op": "positions", "when": {"utc": row["utc"]},
                "config": {"zodiac": "tropical", "node_model": model}}),
            );
            let got = pos(&env, key)["longitude_deg"].as_f64().unwrap();
            let want = row["nodes"][key].as_f64().unwrap();
            assert!(
                angular_diff(got, want).abs() < tol,
                "{} {key}: {got} vs {want}",
                row["utc"]
            );
            // The south node is exactly opposite.
            let south = pos(&env, "south_node")["longitude_deg"].as_f64().unwrap();
            assert!((angular_diff(south, got + 180.0)).abs() < 1e-9);
        }
    }
}

#[test]
fn engine_houses_match_swiss_in_the_tropical_zodiac() {
    let e = Engine::new();
    for row in oracle()["instants"].as_array().unwrap() {
        for (place, h) in row["houses"].as_object().unwrap() {
            if place == "reykjavik" || place == "helsinki" {
                continue; // Placidus handled in the geometry tests; keep this loop quick
            }
            let env = run(
                &e,
                json!({"op": "chart", "system": "western", "when": {"utc": row["utc"]},
                "place": {"latitude": h["lat"], "longitude": h["lon"]},
                "config": {"zodiac": "tropical", "house_system": "placidus"}}),
            );
            let asc = env["houses"]["ascendant"]["longitude_deg"].as_f64().unwrap();
            assert!(
                angular_diff(asc, h["P"]["asc"].as_f64().unwrap()).abs() < 0.01,
                "{} {place} asc",
                row["utc"]
            );
        }
    }
}
