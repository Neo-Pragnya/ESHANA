//! A synthetic chart built from a real chart's own positions derives exactly the same factors as the real chart,
//! is labelled synthetic, and needs no moment, place or ephemeris.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

#[test]
fn real_positions_placed_by_hand_give_the_real_charts_factors() {
    let real = run(
        &json!({"op": "chart", "system": "vedic", "features": ["drishti", "ashtakavarga", "yogas", "relationships"],
        "when": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata"}, "place": {"latitude": 28.61, "longitude": 77.21},
        "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}),
    );
    let placed: Vec<Value> = real["positions"].as_array().unwrap().iter()
        .filter(|p| p["id"] != "south_node")
        .map(|p| json!({"body": p["id"], "longitude_deg": p["longitude_deg"], "speed_deg_per_day": p["speed_deg_per_day"]}))
        .collect();
    let asc = real["houses"]["ascendant"]["longitude_deg"].clone();
    let syn = run(&json!({"op": "synthetic_chart", "positions": placed, "ascendant_deg": asc}));
    assert_eq!(syn["kind"], "synthetic_chart", "{syn}");
    assert_eq!(syn["provenance"]["positions_source"], "synthetic");
    assert!(syn["calculation"]["ephemeris"].is_null());
    let real_attr = |id: &str| {
        real["derived_attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == id)
            .unwrap()["value"]
            .clone()
    };
    let syn_attr = |id: &str| {
        syn["synthetic"]["derived"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == id)
            .unwrap()["value"]
            .clone()
    };
    assert_eq!(syn_attr("vedic.ashtakavarga"), real_attr("vedic.ashtakavarga"));
    assert_eq!(syn_attr("vedic.relationships"), real_attr("vedic.relationships"));
    assert_eq!(syn_attr("vedic.yogas"), real_attr("vedic.yogas"));
    assert_eq!(syn_attr("vedic.drishti"), real_attr("vedic.drishti.graha"));
    assert_eq!(
        syn_attr("vedic.rashi_drishti"),
        real_attr("vedic.drishti.rashi")["aspects"]
    );
}
