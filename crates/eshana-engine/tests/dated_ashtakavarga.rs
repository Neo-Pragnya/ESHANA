//! Dated Aṣṭakavarga: segments change exactly at ingresses, read the right natal cell (recomputed here by hand from
//! the classical contribution houses, independently of the YAML pack), and no table is invented for the nodes.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn natal() -> Value {
    json!({"when": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata"},
           "place": {"latitude": 28.6139, "longitude": 77.209}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}})
}

/// Saturn's Bhinnāṣṭakavarga (Bṛhat Parāśara Horā Śāstra): houses counted from each contributor that get a bindu.
const SATURN_BAV: [(&str, &[usize]); 8] = [
    ("sun", &[1, 2, 4, 7, 8, 10, 11]),
    ("moon", &[3, 6, 11]),
    ("mars", &[3, 5, 6, 10, 11, 12]),
    ("mercury", &[6, 8, 9, 10, 11, 12]),
    ("jupiter", &[5, 6, 11, 12]),
    ("venus", &[6, 11, 12]),
    ("saturn", &[3, 5, 6, 11]),
    ("lagna", &[1, 3, 4, 6, 10, 11]),
];

#[test]
fn saturn_cells_match_a_hand_computation_and_change_only_at_ingresses() {
    let req = json!({"op": "ashtakavarga_transits", "natal": natal(), "bodies": ["saturn", "jupiter"],
                     "start_utc": "2024-01-01T00:00:00Z", "end_utc": "2027-01-01T00:00:00Z"});
    let got = run(&req);
    assert_eq!(got["kind"], "ashtakavarga_transits", "{got}");
    // natal signs from an ordinary chart
    let mut chart = natal();
    chart["op"] = json!("chart");
    chart["system"] = json!("vedic");
    let c = run(&chart);
    let sign = |id: &str| -> usize {
        if id == "lagna" {
            return (c["houses"]["ascendant"]["longitude_deg"].as_f64().unwrap() / 30.0).floor() as usize;
        }
        let p = c["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap();
        (p["longitude_deg"].as_f64().unwrap() / 30.0).floor() as usize
    };
    let mut hand = [0u64; 12];
    for (who, houses) in SATURN_BAV {
        for h in houses {
            hand[(sign(who) + h - 1) % 12] += 1;
        }
    }
    assert_eq!(hand.iter().sum::<u64>(), 39);
    let segs = got["grahas"]["saturn"]["segments"].as_array().unwrap();
    for s in segs {
        let sg = s["transit_sign"].as_u64().unwrap() as usize;
        assert_eq!(s["bav"].as_u64().unwrap(), hand[sg], "saturn in sign {sg}: {s}");
        assert_eq!(s["cells_read"]["bav"], format!("saturn[{sg}]"));
    }
    // boundaries are the ingress instants, and segments tile the range
    let ing = run(
        &json!({"op": "ingresses", "body": "saturn", "start_utc": "2024-01-01T00:00:00Z",
        "end_utc": "2027-01-01T00:00:00Z", "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}),
    );
    let cuts: Vec<f64> = ing["ingresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["jd_ut"].as_f64().unwrap())
        .collect();
    assert_eq!(segs.len(), cuts.len() + 1);
    for (i, cut) in cuts.iter().enumerate() {
        assert_eq!(segs[i]["end_jd_ut"].as_f64().unwrap(), *cut);
        assert_eq!(segs[i + 1]["start_jd_ut"].as_f64().unwrap(), *cut);
        assert_ne!(segs[i]["transit_sign"], segs[i + 1]["transit_sign"]);
    }
    assert_eq!(segs[0]["clipped_start"], true);
    assert_eq!(segs.last().unwrap()["clipped_end"], true);
    // SAV cell read is the natal SAV of the transit sign
    let sav = &got["natal_sarvashtakavarga"]["bindus"];
    for s in got["grahas"]["jupiter"]["segments"].as_array().unwrap() {
        assert_eq!(s["sav"], sav[s["transit_sign"].as_u64().unwrap() as usize]);
    }
}

#[test]
fn nodes_and_missing_birth_times_are_unavailable_not_zero() {
    let got = run(
        &json!({"op": "ashtakavarga_transits", "natal": natal(), "bodies": ["mean_node"],
                          "start_utc": "2025-01-01T00:00:00Z", "end_utc": "2025-06-01T00:00:00Z"}),
    );
    assert_eq!(
        got["grahas"]["mean_node"]["availability"]["status"], "unavailable",
        "{got}"
    );
    assert!(got["grahas"]["mean_node"].get("segments").is_none());
    let mut no_time = natal();
    no_time["when"]["time"] = Value::Null;
    let got = run(
        &json!({"op": "ashtakavarga_transits", "natal": no_time, "bodies": ["saturn"],
                          "start_utc": "2025-01-01T00:00:00Z", "end_utc": "2025-06-01T00:00:00Z"}),
    );
    assert_eq!(
        got["grahas"]["saturn"]["availability"]["missing"][0], "natal.ascendant",
        "{got}"
    );
}

#[test]
fn spans_past_fifty_years_are_a_work_limit() {
    let got = run(&json!({"op": "ashtakavarga_transits", "natal": natal(),
                          "start_utc": "2000-01-01T00:00:00Z", "end_utc": "2060-01-01T00:00:00Z"}));
    assert_eq!(got["error"]["code"], "ESH-LIMIT");
}
