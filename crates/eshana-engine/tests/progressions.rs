//! Secondary progressions: explicit fractional age, positions at the progressed instant, and each angle method's
//! defining identity; no placeholder angles.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn natal() -> Value {
    json!({"when": {"utc": "1990-05-15T09:00:00Z"}, "place": {"latitude": 28.61, "longitude": 77.21},
           "config": {"zodiac": "tropical"}})
}

fn prog(method: Option<&str>, at: &str) -> Value {
    run(&json!({"op": "progressions", "natal": natal(), "at_utc": at, "angle_method": method}))
}

fn sdiff(a: f64, b: f64) -> f64 {
    (a - b + 540.0).rem_euclid(360.0) - 180.0
}

#[test]
fn age_is_fractional_and_positions_are_those_of_the_progressed_instant() {
    // 30 tropical years after birth (365.24219 days each)
    let at = 2_448_026.875 + 30.0 * 365.242_19; // JD of 1990-05-15 09:00 UTC + 30 years
    let at_utc = eshana_time::Instant::from_jd_utc(at).utc;
    let p = prog(None, &at_utc);
    assert_eq!(p["kind"], "progressions", "{p}");
    assert!((p["calculation"]["age_years"].as_f64().unwrap() - 30.0).abs() < 1e-6);
    let pjd = p["calculation"]["progressed_jd_ut"].as_f64().unwrap();
    assert!((pjd - (2_448_026.875 + 30.0)).abs() < 1e-6);
    let direct = run(
        &json!({"op": "positions", "when": {"utc": p["calculation"]["progressed_utc"]}, "config": {"zodiac": "tropical"}}),
    );
    let sun = |e: &Value| {
        e["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == "sun")
            .unwrap()["longitude_deg"]
            .as_f64()
            .unwrap()
    };
    assert!((sun(&p) - sun(&direct)).abs() < 1e-9);
    assert_eq!(p["angles"]["status"], "unavailable");
}

#[test]
fn each_angle_method_satisfies_its_definition() {
    let at = "2020-05-15T00:00:00Z";
    let sa = prog(Some("solar_arc_longitude"), at);
    let arc = sa["solar_arc_deg"].as_f64().unwrap();
    let mc_n = sa["angles"]["natal_midheaven_deg"].as_f64().unwrap();
    assert!(
        sdiff(sa["angles"]["midheaven_deg"].as_f64().unwrap(), mc_n + arc).abs() < 1e-6,
        "{}",
        sa["angles"]
    );
    let nb = prog(Some("naibod_ra"), at);
    let age = nb["calculation"]["age_years"].as_f64().unwrap();
    let q = prog(Some("quotidian"), at);
    // natal RAMC from the quotidian of age 0 (the birth instant itself)
    let birth = prog(Some("quotidian"), "1990-05-15T09:00:00Z");
    let ramc_n = birth["angles"]["ramc_deg"].as_f64().unwrap();
    assert!(
        sdiff(
            nb["angles"]["ramc_deg"].as_f64().unwrap(),
            ramc_n + 0.985_647_358 * age
        )
        .abs()
            < 1e-9
    );
    // quotidian: the local sidereal time of an ordinary chart cast for the progressed instant at the birth place
    let c = run(
        &json!({"op": "chart", "system": "western", "when": {"utc": q["calculation"]["progressed_utc"]},
        "place": {"latitude": 28.61, "longitude": 77.21}, "config": {"zodiac": "tropical"}}),
    );
    let lst = c["houses"]["local_apparent_sidereal_time_deg"].as_f64().unwrap();
    assert!(
        sdiff(q["angles"]["ramc_deg"].as_f64().unwrap(), lst).abs() < 1e-5, // the UTC text is rounded to the millisecond
        "{lst} vs {}",
        q["angles"]
    );
    assert_eq!(q["angles"]["method"]["id"], "quotidian");
}

#[test]
fn no_place_no_angles_and_unknown_methods_are_refused() {
    let mut n = natal();
    n.as_object_mut().unwrap().remove("place");
    let p = run(
        &json!({"op": "progressions", "natal": n, "at_utc": "2020-01-01T00:00:00Z", "angle_method": "naibod_ra"}),
    );
    assert_eq!(p["angles"]["missing"][0], "place", "{p}");
    assert_eq!(
        prog(Some("koch_arc"), "2020-01-01T00:00:00Z")["error"]["code"],
        "ESH-METHOD"
    );
    assert_eq!(
        prog(None, "1980-01-01T00:00:00Z")["error"]["code"],
        "ESH-RANGE-INVALID"
    );
}
