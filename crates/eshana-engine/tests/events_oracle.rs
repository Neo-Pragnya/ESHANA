//! Stations and crossings against an independent oracle: benchmarks/golden/events_oracle.json, made by
//! benchmarks/make_event_oracle.py from the Swiss Ephemeris' own analytical speeds (numbers only; nothing shipped).
//!
//! Event times inherit position error: a station sits where speed is zero, so a position error of e arcseconds moves
//! it by roughly e / (acceleration), which for slow planets is hours. The tolerances below are per body and per
//! provider, measured and recorded in docs/reference/accuracy.md, never loosened to make a test pass.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(v: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&v.to_string())).unwrap()
}

fn oracle() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../benchmarks/golden/events_oracle.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn config(frame: &str, provider: &str) -> Value {
    let mut c = if frame == "tropical" {
        json!({"zodiac": "tropical"})
    } else {
        json!({"zodiac": "sidereal", "ayanamsha": "lahiri"})
    };
    c["provider"] = json!(provider);
    c
}

/// Largest station-time difference in hours, per body, for a provider.
fn station_errors(provider: &str, spk: Option<&str>) -> Vec<(String, String, f64)> {
    let o = oracle();
    let (start, end) = ("2024-01-01T00:00:00Z", "2027-01-01T00:00:00Z");
    let mut out = vec![];
    for (frame, bodies) in o["stations"].as_object().unwrap() {
        for (body, expect) in bodies.as_object().unwrap() {
            let mut req = json!({"op": "stations", "body": body, "start_utc": start, "end_utc": end,
                                 "config": config(frame, provider)});
            if let Some(p) = spk {
                req["spk_path"] = json!(p);
            }
            let got = run(&req);
            let got = got["stations"].as_array().unwrap_or_else(|| panic!("{got}"));
            let expect = expect.as_array().unwrap();
            assert_eq!(
                got.len(),
                expect.len(),
                "{provider} {frame} {body}: station count"
            );
            let mut worst: f64 = 0.0;
            for (g, e) in got.iter().zip(expect) {
                assert_eq!(g["station"], e["station"], "{frame} {body} at {}", e["utc"]);
                // where the planet stations is well conditioned even when when it does is not
                let dlon = (g["longitude_deg"].as_f64().unwrap() - e["longitude_deg"].as_f64().unwrap())
                    .abs()
                    * 3600.0;
                if provider == "spk" {
                    assert!(
                        dlon <= 2.0,
                        "{frame} {body} at {}: station longitude {dlon:.2}″",
                        e["utc"]
                    );
                }
                let dh = (g["jd_ut"].as_f64().unwrap() - e["jd_ut"].as_f64().unwrap()).abs() * 24.0;
                worst = worst.max(dh);
            }
            out.push((frame.clone(), body.clone(), worst));
        }
    }
    out
}

/// Hours. The instant of a station is ill-conditioned: Δt = Δspeed / acceleration, and a slow planet barely
/// accelerates there, so a speed difference of 1e-5 °/day moves Saturn's station by minutes while its longitude agrees
/// to 0.3″. With the JPL kernel the tropical frame is 5-16× worse than the sidereal one: tropical speed of date carries
/// the nutation rate, and ESHANA's 4-term nutation series (about 0.5″, docs/reference/accuracy.md) is not the oracle's
/// full series. Measured worst 2024-2026: spk tropical 0.013 h Mars, 0.054 h Jupiter, 0.122 h Saturn; spk sidereal
/// ≤ 0.0095 h; analytic 0.04 h Mercury, 0.24 h Venus, 0.11 h Mars, 1.35 h Jupiter, 2.0 h Saturn.
fn station_tolerance_hours(provider: &str, frame: &str, body: &str) -> f64 {
    match (provider, frame, body) {
        ("spk", "tropical", "mercury" | "venus") => 0.01,
        ("spk", "tropical", "mars") => 0.03,
        ("spk", "tropical", _) => 0.2,
        ("spk", _, _) => 0.02,
        (_, _, "mercury") => 0.1,
        (_, _, "venus" | "mars") => 0.5,
        _ => 4.0,
    }
}

#[test]
fn stations_match_the_oracle_with_the_analytic_provider() {
    for (frame, body, worst) in station_errors("analytic", None) {
        eprintln!("analytic {frame:16} {body:8} worst {worst:.4} h");
        assert!(
            worst <= station_tolerance_hours("analytic", &frame, &body),
            "analytic {frame} {body}: {worst:.3} h"
        );
    }
}

#[test]
fn stations_match_the_oracle_with_the_jpl_kernel() {
    let Ok(path) = std::env::var("ESHANA_SPK") else {
        eprintln!("ESHANA_SPK not set; skipping the SPK station check");
        return;
    };
    for (frame, body, worst) in station_errors("spk", Some(&path)) {
        eprintln!("spk {frame:16} {body:8} worst {worst:.4} h");
        assert!(
            worst <= station_tolerance_hours("spk", &frame, &body),
            "spk {frame} {body}: {worst:.3} h"
        );
    }
}

/// The analytic provider's measured worst longitude error in arcseconds (benchmarks/golden/accuracy.md); the JPL
/// kernel is good to 2″. A crossing time can only be as good as the position: compare the time error times speed.
fn position_error_arcsec(provider: &str, body: &str) -> f64 {
    match (provider, body) {
        ("spk", _) => 2.0,
        (_, "mercury") => 27.8,
        (_, "venus") => 22.4,
        (_, "mars") => 31.5,
        (_, "jupiter") => 342.3,
        (_, "saturn") => 1025.6,
        _ => 60.0,
    }
}

fn crossing_check(provider: &str, spk: Option<&str>) {
    let o = oracle();
    for c in o["crossings"].as_array().unwrap() {
        let body = c["body"].as_str().unwrap();
        let frame = c["frame"].as_str().unwrap();
        let mut req = json!({"op": "crossings", "body": body, "point_deg": c["point_deg"],
            "start_utc": "2024-01-01T00:00:00Z", "end_utc": "2027-01-01T00:00:00Z",
            "touch_orb_deg": 0.0, "config": config(frame, provider)});
        if let Some(p) = spk {
            req["spk_path"] = json!(p);
        }
        let got = run(&req);
        let events = got["events"].as_array().unwrap_or_else(|| panic!("{got}"));
        let expect = c["events"].as_array().unwrap();
        assert_eq!(events.len(), expect.len(), "{body}: {got}");
        for (g, e) in events.iter().zip(expect) {
            assert_eq!(g["direction"], e["direction"]);
            let jd = g["jd_ut"].as_f64().unwrap();
            let dt_days = (jd - e["jd_ut"].as_f64().unwrap()).abs();
            let arcsec = dt_days * speed_at(body, g["utc"].as_str().unwrap(), frame) * 3600.0;
            eprintln!(
                "{provider} {body} {frame} {} Δt {:.2} min ≈ {arcsec:.1}″",
                e["utc"],
                dt_days * 1440.0
            );
            assert!(
                arcsec <= position_error_arcsec(provider, body),
                "{provider} {body} at {}: {arcsec:.1}″",
                e["utc"]
            );
            assert!(g["residual_deg"].as_f64().unwrap().abs() < 1e-4);
        }
    }
}

/// |speed| in degrees per day at an instant: two rows of the ephemeris op an hour apart.
fn speed_at(body: &str, utc: &str, frame: &str) -> f64 {
    let rows = run(
        &json!({"op": "ephemeris", "start_utc": utc, "step_days": 1.0 / 24.0, "count": 2,
        "bodies": [body], "config": config(frame, "analytic")}),
    );
    let lon = |k: usize| {
        rows["rows"][k]["longitudes"][body]
            .as_f64()
            .unwrap_or_else(|| panic!("{rows}"))
    };
    let d = (lon(1) - lon(0) + 540.0).rem_euclid(360.0) - 180.0;
    (d * 24.0).abs()
}

#[test]
fn crossings_match_the_oracle_including_retrograde_loops() {
    crossing_check("analytic", None);
}

#[test]
fn crossings_match_the_oracle_with_the_jpl_kernel() {
    let Ok(path) = std::env::var("ESHANA_SPK") else {
        eprintln!("ESHANA_SPK not set; skipping the SPK crossing check");
        return;
    };
    crossing_check("spk", Some(&path));
}

#[test]
fn retrograde_intervals_are_bounded_by_stations_and_clipped_at_the_range() {
    // Mercury was retrograde from 2024-04-01 to 2024-04-25: start the range inside that loop
    let got = run(
        &json!({"op": "stations", "body": "mercury", "start_utc": "2024-04-10T00:00:00Z",
        "end_utc": "2024-09-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    );
    let iv = got["retrograde_intervals"].as_array().unwrap();
    assert_eq!(iv[0]["clipped_start"], true);
    assert_eq!(iv[0]["start_utc"], "2024-04-10T00:00:00Z");
    assert_eq!(iv[0]["clipped_end"], false);
    let stations = got["stations"].as_array().unwrap();
    assert_eq!(stations[0]["station"], "direct");
    assert_eq!(iv[0]["end_jd_ut"], stations[0]["jd_ut"]);
    // the next loop (2024-08-05 to 08-28) is complete inside the range
    assert_eq!(iv[1]["clipped_start"], false);
    assert_eq!(iv[1]["clipped_end"], false);
    assert_eq!(iv.len(), 2);
}

#[test]
fn bodies_that_never_station_return_none_and_say_why() {
    let got = run(
        &json!({"op": "stations", "body": "moon", "start_utc": "2024-01-01T00:00:00Z",
        "end_utc": "2024-02-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    );
    assert_eq!(got["stations"], json!([]));
    assert_eq!(got["note"], "moon never stations");
}

#[test]
fn a_touch_without_crossing_is_reported_apart_from_crossings() {
    // Mercury's 2024-04-01 station sits at 27.2186° tropical: a point 0.05° beyond it is approached, never reached
    let got = run(&json!({"op": "crossings", "body": "mercury", "point_deg": 27.27,
        "start_utc": "2024-03-20T00:00:00Z", "end_utc": "2024-04-15T00:00:00Z",
        "touch_orb_deg": 0.1, "config": {"zodiac": "tropical"}}));
    let events = got["events"].as_array().unwrap();
    assert_eq!(events.len(), 1, "{got}");
    assert_eq!(events[0]["kind"], "touch");
    let gap = events[0]["separation_deg"].as_f64().unwrap();
    assert!(gap < 0.0 && gap > -0.1, "{gap}");
}

#[test]
fn aspects_other_than_conjunction_and_opposition_have_two_points() {
    let got = run(
        &json!({"op": "crossings", "body": "sun", "point_deg": 0.0, "aspects": [90],
        "start_utc": "2025-01-01T00:00:00Z", "end_utc": "2026-01-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    );
    let targets: Vec<f64> = got["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["target_deg"].as_f64().unwrap())
        .collect();
    assert_eq!(targets, vec![90.0, 270.0], "{got}"); // in date order: 90° at the June solstice, 270° in December
}

#[test]
fn bad_ranges_and_settings_are_refused_with_stable_codes() {
    let reversed = run(
        &json!({"op": "stations", "body": "mars", "start_utc": "2025-01-01T00:00:00Z", "end_utc": "2024-01-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    );
    assert_eq!(reversed["error"]["code"], "ESH-RANGE-INVALID", "{reversed}");
    let too_long = run(
        &json!({"op": "crossings", "body": "mars", "point_deg": 10, "start_utc": "1800-01-01T00:00:00Z", "end_utc": "2090-01-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    );
    assert_eq!(too_long["error"]["code"], "ESH-LIMIT", "{too_long}");
    for req in [
        json!({"op": "stations", "body": "mars", "start_utc": "2024-01-01T00:00:00Z", "end_utc": "2025-01-01T00:00:00Z", "tolerance_seconds": 0.0, "config": {"zodiac": "tropical"}}),
        json!({"op": "crossings", "body": "mars", "point_deg": 10, "aspects": [], "start_utc": "2024-01-01T00:00:00Z", "end_utc": "2025-01-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
        json!({"op": "crossings", "body": "mars", "point_deg": 10, "max_results": 0, "start_utc": "2024-01-01T00:00:00Z", "end_utc": "2025-01-01T00:00:00Z", "config": {"zodiac": "tropical"}}),
    ] {
        let got = run(&req);
        assert_eq!(got["error"]["code"], "ESH-INPUT", "{req} → {got}");
    }
}
