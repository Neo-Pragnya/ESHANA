//! KP horary geometry against an independent computation (benchmarks/golden/kp_horary_oracle.json, Swiss Ephemeris
//! `houses_armc`, offline), and the KP operations end to end.

use eshana_core::HouseSystem;
use eshana_engine::{astro, kp, Engine};
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn sdiff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

#[test]
fn horary_cusps_match_the_oracle() {
    let o: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/kp_horary_oracle.json")).unwrap();
    let mut worst: f64 = 0.0;
    for c in o["cases"].as_array().unwrap() {
        let (target, lat, t) = (
            c["asc_target_deg"].as_f64().unwrap(),
            c["latitude_deg"].as_f64().unwrap(),
            c["t_centuries_tt"].as_f64().unwrap(),
        );
        let lst = kp::lst_for_ascendant(target, lat, t).unwrap();
        // Both sides solve for the same Ascendant; their obliquities of date differ by milliarcseconds.
        assert!(
            sdiff(lst, c["armc_deg"].as_f64().unwrap()).abs() < 1e-4,
            "{c}: lst {lst}"
        );
        let cusps = astro::houses(HouseSystem::Placidus, lst, lat, t).unwrap();
        for (i, want) in c["cusps_deg"].as_array().unwrap().iter().enumerate() {
            let d = sdiff(cusps[i], want.as_f64().unwrap()).abs() * 3600.0;
            worst = worst.max(d);
            assert!(d < 1.0, "case {c}: cusp {} off by {d:.3}″", i + 1);
        }
    }
    eprintln!("worst horary cusp difference: {worst:.4}″");
}

#[test]
fn horary_op_fixes_the_ascendant_and_keeps_number_and_moment_apart() {
    let got = run(&json!({"op": "kp_horary", "number": 23,
        "chart": {"when": {"utc": "2026-11-23T10:00:00Z"}, "place": {"latitude": 13.08, "longitude": 80.27},
                  "config": {"zodiac": "sidereal", "ayanamsha": "krishnamurti"}}}));
    assert_eq!(got["kind"], "kp_horary", "{got}");
    // 23 is the first Taurus number: the Kṛttikā Rāhu sub after the sign boundary.
    assert_eq!(got["number"]["sign"], "Vrishabha");
    assert_eq!(got["ascendant"]["longitude_deg"], 30.0);
    assert_eq!(got["input"]["number"], 23);
    assert_eq!(got["input"]["judgement_utc"], "2026-11-23T10:00:00Z");
    assert_eq!(got["kp"]["cusps"]["1"]["lords"]["sub_lord"], "rahu");
    assert_eq!(got["calculation"]["method"]["id"], "cusp_start");
}

#[test]
fn occupancy_is_opt_in_and_unknown_ids_are_esh_method() {
    let base = json!({"op": "chart", "system": "vedic", "features": ["kp"],
        "when": {"utc": "1990-05-15T09:00:00Z"}, "place": {"latitude": 28.61, "longitude": 77.21},
        "config": {"zodiac": "sidereal", "ayanamsha": "krishnamurti"}});
    let kp_of = |v: &Value| -> Value {
        v["derived_attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == "vedic.kp")
            .unwrap()["value"]
            .clone()
    };
    let default = kp_of(&run(&base));
    assert_eq!(default["occupancy_method"], "cusp_start");
    let mut mid = base.clone();
    mid["kp_cusp_occupancy"] = json!("cusp_midpoint");
    let mid = kp_of(&run(&mid));
    assert_eq!(mid["occupancy_method"], "cusp_midpoint");
    // Lords of cusps and planets do not depend on occupancy.
    assert_eq!(default["planets"], mid["planets"]);
    let mut bad = base.clone();
    bad["kp_cusp_occupancy"] = json!("nearest");
    assert_eq!(run(&bad)["error"]["code"], "ESH-METHOD");
}

#[test]
fn table_op_lists_249_numbers() {
    let got = run(&json!({"op": "kp_table"}));
    assert_eq!(got["table"]["count"], 249);
    assert_eq!(got["table"]["entries"][248]["sub_lord"], "saturn");
}

#[test]
fn the_operations_list_matches_the_request_enum() {
    use eshana_engine::api::OPERATIONS;
    let mut seen = std::collections::BTreeSet::new();
    for op in OPERATIONS {
        assert!(seen.insert(op), "{op} listed twice");
        let got = run(&json!({"op": op}));
        let msg = got["error"]["message"].as_str().unwrap_or("");
        assert!(
            !msg.contains("unknown variant"),
            "{op} is not an operation: {msg}"
        );
    }
    // and nothing the enum accepts is missing: serde lists every variant when one is unknown
    let got = run(&json!({"op": "no_such_op"}));
    let msg = got["error"]["message"].as_str().unwrap();
    let listed = msg.split("expected one of").nth(1).unwrap();
    let n = listed.matches('`').count() / 2;
    assert_eq!(n, OPERATIONS.len(), "{msg}");
}
