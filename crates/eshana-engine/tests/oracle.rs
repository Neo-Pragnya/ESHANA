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

/// AST-03: every house system against the oracle's cusps for the same ARMC, latitude and obliquity (seven latitudes
/// from −50° to 64°, twelve sidereal times). Agreement within 0.01°.
#[test]
fn house_systems_match_the_oracle() {
    use eshana_core::HouseSystem;
    let doc: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/houses_oracle.json")).unwrap();
    let mut checked = 0;
    for row in doc["rows"].as_array().unwrap() {
        let (lat, armc, eps) = (
            row["lat"].as_f64().unwrap(),
            row["armc"].as_f64().unwrap(),
            row["eps"].as_f64().unwrap(),
        );
        for (name, want) in row["cusps"].as_object().unwrap() {
            let Some(want) = want.as_array() else { continue };
            let sys = HouseSystem::parse(name).unwrap();
            let got = eshana_engine::astro::house_cusps(sys, armc, lat, eps)
                .unwrap_or_else(|e| panic!("{name} lat {lat} armc {armc}: {e}"));
            for (i, w) in want.iter().enumerate() {
                let w = w.as_f64().unwrap();
                assert!(
                    angular_diff(got[i], w).abs() < 0.01,
                    "{name} lat {lat} armc {armc} cusp {}: got {:.4} want {w:.4}",
                    i + 1,
                    got[i]
                );
            }
            checked += 1;
        }
    }
    assert!(checked > 500, "{checked}");
}

/// AST-08: rise, set and upper transit of the Sun, the Moon and Venus at four places (including Tromsø, inside
/// the Arctic circle) on three dates, against the oracle's rise_trans. Within 30 s, and the same events exist.
#[test]
fn rise_and_set_match_the_oracle() {
    let doc: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/rise_set_oracle.json")).unwrap();
    let e = Engine::new();
    let mut checked = 0;
    for row in doc["rows"].as_array().unwrap() {
        let env = run(
            &e,
            json!({"op": "rise_set", "place": {"latitude": row["lat"], "longitude": row["lon"]},
            "date": row["date"], "utc_offset_minutes": 0, "bodies": ["sun", "moon", "venus"], "twilight": false}),
        );
        for b in env["days"][0]["bodies"].as_array().unwrap() {
            let id = b["body"].as_str().unwrap();
            for k in ["rise", "set", "transit"] {
                let want = row["events"][id][k].as_f64();
                let got = b[k]["jd_ut"].as_f64();
                match (want, got) {
                    (None, None) => {}
                    (Some(w), Some(g)) => {
                        assert!(
                            (g - w).abs() * 86_400.0 < 30.0,
                            "{} {} {id} {k}: {:.1} s",
                            row["place"],
                            row["date"],
                            (g - w) * 86_400.0
                        );
                        checked += 1;
                    }
                    _ => panic!(
                        "{} {} {id} {k}: oracle {want:?}, ESHANA {got:?}",
                        row["place"], row["date"]
                    ),
                }
            }
        }
    }
    assert!(checked > 80, "{checked}");
}

/// AST-08: the topocentric shift (topocentric minus geocentric longitude) of the Moon and the Sun at three places
/// (including Quito at 2,850 m) against the oracle's. Comparing the shift keeps the provider's own error out of it.
#[test]
fn topocentric_shift_matches_the_oracle() {
    let doc: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/topocentric_oracle.json")).unwrap();
    let e = Engine::new();
    for row in doc["rows"].as_array().unwrap() {
        let req = |topo: bool| {
            json!({"op": "positions", "when": {"utc": row["utc"]}, "bodies": ["moon", "sun"],
            "place": {"latitude": row["lat"], "longitude": row["lon"], "altitude_m": row["alt"]}, "config": {"topocentric": topo}})
        };
        let (topo, geo) = (run(&e, req(true)), run(&e, req(false)));
        assert_eq!(topo["calculation"]["positions_reference"], "topocentric");
        for b in ["moon", "sun"] {
            let ours = angular_diff(
                pos(&topo, b)["longitude_deg"].as_f64().unwrap(),
                pos(&geo, b)["longitude_deg"].as_f64().unwrap(),
            );
            let want = angular_diff(
                row["topocentric"][b].as_f64().unwrap(),
                row["geocentric"][b].as_f64().unwrap(),
            );
            assert!(
                (ours - want).abs() * 3600.0 < 2.0,
                "{} {} {b}: shift {:.1}″ vs {:.1}″",
                row["place"],
                row["utc"],
                ours * 3600.0,
                want * 3600.0
            );
        }
    }
}

/// AST-06: every solar and lunar eclipse from 2000 to 2040 (the oracle's list, including the hybrids of 2005, 2013 and
/// 2023), found with the same type and the time of greatest eclipse within 2 minutes; and nothing extra.
#[test]
fn eclipses_match_the_oracle() {
    let doc: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/eclipses_oracle.json")).unwrap();
    let want = doc["rows"].as_array().unwrap();
    let env = run(
        &Engine::new(),
        json!({"op": "eclipses", "start_utc": "2000-01-01T00:00:00Z", "end_utc": "2040-01-01T00:00:00Z"}),
    );
    // ESHANA also finds the faint penumbral lunar eclipse of 2016-08-18 (magnitude 0.019; NASA's canon lists it at
    // 0.017), which the oracle's shadow model misses: extra penumbral eclipses under 0.03 are accepted
    let got: Vec<&Value> = env["eclipses"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|g| {
            !(g["type"] == "penumbral"
                && g["penumbral_magnitude"].as_f64().unwrap() < 0.03
                && !want.iter().any(|w| {
                    (w["max_jd_ut"].as_f64().unwrap() - g["max_jd_ut"].as_f64().unwrap()).abs() < 0.01
                }))
        })
        .collect();
    assert_eq!(
        got.len(),
        want.len(),
        "count: ESHANA {} vs oracle {}",
        got.len(),
        want.len()
    );
    for (g, w) in got.iter().zip(want) {
        let (gj, wj) = (g["max_jd_ut"].as_f64().unwrap(), w["max_jd_ut"].as_f64().unwrap());
        assert_eq!(g["kind"], w["kind"], "{}", g["max_utc"]);
        assert!(
            (gj - wj).abs() * 1440.0 < 2.0,
            "{} {}: {:.1} min",
            g["kind"],
            g["max_utc"],
            (gj - wj) * 1440.0
        );
        assert_eq!(g["type"], w["type"], "{} {}", g["kind"], g["max_utc"]);
    }
}

/// WES-03 / AST-06: solar and lunar returns against the oracle's crossing solvers (first return after each start),
/// with the default analytic provider (VSOP87 Sun since 0.5.0): within 60 s for the Sun and 30 s for the Moon.
#[test]
fn returns_match_the_oracle() {
    let doc: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/returns_oracle.json")).unwrap();
    let e = Engine::new();
    for row in doc["rows"].as_array().unwrap() {
        let body = row["body"].as_str().unwrap();
        let start = row["start"].as_str().unwrap();
        let days = if body == "sun" { 370 } else { 31 };
        let end = eshana_time::Instant::from_jd_utc(
            eshana_time::Instant::parse_utc(start).unwrap().jd_ut + f64::from(days),
        )
        .utc;
        let env = run(
            &e,
            json!({"op": "returns", "natal": {"when": {"utc": row["birth_utc"]}, "system": "western"},
            "body": body, "start_utc": start, "end_utc": end, "charts": false}),
        );
        let first = &env["returns"][0];
        let (got, want) = (
            first["jd_ut"].as_f64().unwrap(),
            row["return_jd_ut"].as_f64().unwrap(),
        );
        let tol = if body == "sun" { 60.0 } else { 30.0 };
        assert!(
            (got - want).abs() * 86_400.0 < tol,
            "{body} return after {start}: {:.1} s",
            (got - want) * 86_400.0
        );
        // the natal longitude itself agrees with the oracle's within a few arc-seconds
        let nl = env["calculation"]["natal_longitude_deg"].as_f64().unwrap();
        assert!(
            angular_diff(nl, row["natal_lon"].as_f64().unwrap()).abs() * 3600.0 < 15.0,
            "{body} natal longitude"
        );
    }
}
