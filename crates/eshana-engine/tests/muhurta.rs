//! Muhūrta windows: partition and merge invariants, exact boundaries, constraint toggling, empty results that say
//! why, polar days, DST and half-hour zones.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn delhi(extra: Value) -> Value {
    let mut r = json!({"op": "muhurta_windows", "place": {"latitude": 28.61, "longitude": 77.21},
        "timezone": "Asia/Kolkata", "start_utc": "2026-11-23T00:00:00Z", "end_utc": "2026-11-26T00:00:00Z",
        "config": {"ayanamsha": "lahiri"}});
    for (k, v) in extra.as_object().unwrap() {
        r[k] = v.clone();
    }
    r
}

fn minutes(ws: &Value) -> f64 {
    ws.as_array()
        .unwrap()
        .iter()
        .map(|w| w["duration_minutes"].as_f64().unwrap())
        .sum()
}

#[test]
fn pieces_tile_the_range_and_no_two_neighbours_are_equal() {
    let got = run(&delhi(
        json!({"profile": "general_auspicious", "show": ["nakshatra", "choghadiya"]}),
    ));
    let tl = got["timeline"].as_array().unwrap();
    assert_eq!(tl[0]["start_utc"], "2026-11-23T00:00:00Z", "{got}");
    assert_eq!(tl.last().unwrap()["end_utc"], "2026-11-26T00:00:00Z");
    for w in tl.windows(2) {
        assert_eq!(w[0]["end_utc"], w[1]["start_utc"]);
        assert!(
            w[0]["factors"] != w[1]["factors"] || w[0]["failed"] != w[1]["failed"],
            "unmerged neighbours {w:?}"
        );
        assert_eq!(w[0]["bounds"], "[start, end)");
    }
    assert!((minutes(&got["timeline"]) - got["range_minutes"].as_f64().unwrap()).abs() < 1e-6);
}

#[test]
fn tithi_boundaries_are_the_panchanga_end_times() {
    let got = run(&delhi(
        json!({"constraints": [{"factor": "tithi", "exclude": [0]}]}),
    ));
    let tl = got["timeline"].as_array().unwrap();
    // the first tithi change inside the range
    let cut = tl
        .windows(2)
        .find(|w| w[0]["factors"]["tithi"] != w[1]["factors"]["tithi"])
        .unwrap()[0]["end_utc"]
        .as_str()
        .unwrap()
        .to_string();
    let p = run(
        &json!({"op": "panchanga", "chart": {"when": {"utc": "2026-11-23T00:00:00Z"},
        "place": {"latitude": 28.61, "longitude": 77.21}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}}),
    );
    let ends = p["panchanga"]["tithi"]["ends"]["utc"].as_str().unwrap();
    assert_eq!(&cut[..19], &ends[..19], "window cut {cut} vs pañcāṅga {ends}");
}

#[test]
fn disabling_one_constraint_changes_only_its_own_intervals() {
    let full = run(&delhi(json!({"profile": "general_auspicious"})));
    let less = run(&delhi(
        json!({"profile": "general_auspicious", "disable": ["no_rikta_tithi"]}),
    ));
    // pieces of the full run that failed only on no_rikta_tithi
    let only_rikta: f64 = full["timeline"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|w| {
            let f = w["failed"].as_array().unwrap();
            f.len() == 1 && f[0]["constraint"] == "no_rikta_tithi"
        })
        .map(|w| w["duration_minutes"].as_f64().unwrap())
        .sum();
    assert!(only_rikta > 0.0);
    let (a, b) = (minutes(&full["windows"]), minutes(&less["windows"]));
    assert!((b - (a + only_rikta)).abs() < 1e-6, "{a} + {only_rikta} != {b}");
    // every window of the full run is still inside a window of the reduced run
    for w in full["windows"].as_array().unwrap() {
        let s = w["start_utc"].as_str().unwrap();
        assert!(less["windows"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["start_utc"].as_str().unwrap() <= s
                && x["end_utc"].as_str().unwrap() >= w["end_utc"].as_str().unwrap()));
    }
}

#[test]
fn an_impossible_request_is_empty_and_says_which_constraints_removed_it() {
    let got = run(&delhi(
        json!({"constraints": ["abhijit_only", "brahma_muhurta_only"]}),
    ));
    assert_eq!(got["windows"].as_array().unwrap().len(), 0);
    let why: Vec<&str> = got["empty_because"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["constraint"].as_str().unwrap())
        .collect();
    assert!(
        why.contains(&"abhijit_only") && why.contains(&"brahma_muhurta_only"),
        "{got}"
    );
}

#[test]
fn polar_night_makes_day_factors_unavailable_never_passing() {
    let got = run(
        &json!({"op": "muhurta_windows", "place": {"latitude": 78.22, "longitude": 15.65},
        "timezone": "Arctic/Longyearbyen", "start_utc": "2026-12-20T00:00:00Z", "end_utc": "2026-12-22T00:00:00Z",
        "constraints": ["no_rahu_kalam"], "config": {"ayanamsha": "lahiri"}}),
    );
    assert_eq!(got["windows"].as_array().unwrap().len(), 0, "{got}");
    let w = &got["timeline"][0];
    assert_eq!(
        w["factors"]["rahu_kalam"]["availability"]["status"],
        "unavailable"
    );
    assert_eq!(w["failed"][0]["why"], "unavailable");
}

#[test]
fn local_times_follow_dst_and_half_hour_zones() {
    // London clocks go back at 01:00 UTC on 2026-10-25.
    let got = run(
        &json!({"op": "muhurta_windows", "place": {"latitude": 51.5, "longitude": -0.12},
        "timezone": "Europe/London", "start_utc": "2026-10-24T12:00:00Z", "end_utc": "2026-10-25T12:00:00Z",
        "constraints": ["no_rahu_kalam"], "config": {"ayanamsha": "lahiri"}}),
    );
    let tl = got["timeline"].as_array().unwrap();
    assert!(
        tl[0]["start_local"].as_str().unwrap().ends_with("+01:00"),
        "{}",
        tl[0]
    );
    assert!(tl.last().unwrap()["end_local"]
        .as_str()
        .unwrap()
        .ends_with("+00:00"));
    let k = run(
        &json!({"op": "muhurta_windows", "place": {"latitude": 27.7, "longitude": 85.3},
        "timezone": "Asia/Kathmandu", "start_utc": "2026-01-01T00:00:00Z", "end_utc": "2026-01-02T00:00:00Z",
        "constraints": ["no_rahu_kalam"], "config": {"ayanamsha": "lahiri"}}),
    );
    assert!(k["timeline"][0]["start_local"]
        .as_str()
        .unwrap()
        .ends_with("+05:45"));
}

#[test]
fn ranges_and_profiles_are_checked() {
    assert_eq!(
        run(&delhi(json!({"profile": "nope"})))["error"]["code"],
        "ESH-METHOD"
    );
    assert_eq!(
        run(&delhi(
            json!({"end_utc": "2027-03-01T00:00:00Z", "profile": "daily_avoidances"})
        ))["error"]["code"],
        "ESH-LIMIT"
    );
    assert_eq!(run(&delhi(json!({})))["error"]["code"], "ESH-INPUT");
    let exp = run(&delhi(json!({"profile": "travel"})));
    assert_eq!(exp["profile"]["status"], "experimental");
    assert!(!exp["profile"]["missing"].as_array().unwrap().is_empty());
    assert!(exp["warnings"][0]["message"]
        .as_str()
        .unwrap()
        .contains("experimental"));
}
