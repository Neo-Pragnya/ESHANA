//! The 36-point output is unchanged byte-for-byte (golden captured before the supplementary factors existed), and
//! the supplementary section reads the right cells in both directions.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn person(spec: &str) -> Value {
    let f: Vec<&str> = spec.split(',').collect();
    json!({"when": {"date": f[0], "time": f[1], "timezone": f[2]},
           "place": {"latitude": f[3].parse::<f64>().unwrap(), "longitude": f[4].parse::<f64>().unwrap()},
           "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}})
}

#[test]
fn thirty_six_point_output_is_byte_for_byte_unchanged() {
    let golden: Value = serde_json::from_str(include_str!("golden/matching_36.json")).unwrap();
    for g in golden.as_array().unwrap() {
        let got = run(
            &json!({"op": "matching", "a": person(g["a"].as_str().unwrap()), "b": person(g["b"].as_str().unwrap())}),
        );
        assert_eq!(
            got["ashtakoota"].to_string(),
            g["ashtakoota"].to_string(),
            "{}",
            g["a"]
        );
        assert_eq!(got["manglik"].to_string(), g["manglik"].to_string());
        assert!(got["supplementary"]["moon_factors"].is_array());
    }
}

#[test]
fn pair_ashtakavarga_reads_the_other_charts_cells_in_each_direction() {
    let (pa, pb) = (
        person("1990-05-15,14:30,Asia/Kolkata,28.61,77.21"),
        person("1992-08-20,09:15,Asia/Kolkata,19.076,72.8777"),
    );
    let got = run(&json!({"op": "matching", "a": pa, "b": pb}));
    let s = &got["supplementary"];
    let chart = |p: &Value| {
        let mut r = p.clone();
        r["op"] = json!("chart");
        r["system"] = json!("vedic");
        r["features"] = json!(["ashtakavarga"]);
        run(&r)
    };
    let (ca, cb) = (chart(&pa), chart(&pb));
    let av = |c: &Value| {
        c["derived_attributes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == "vedic.ashtakavarga")
            .unwrap()["value"]
            .clone()
    };
    let sign = |c: &Value, id: &str| {
        (c["positions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap()["longitude_deg"]
            .as_f64()
            .unwrap()
            / 30.0)
            .floor() as usize
    };
    for g in ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"] {
        let (s_a, s_b) = (sign(&ca, g), sign(&cb, g));
        assert_eq!(
            s["pair_ashtakavarga"]["a_in_b"][g]["bav"],
            av(&cb)["bhinnashtakavarga"][g]["bindus"][s_a],
            "{g} a→b"
        );
        assert_eq!(
            s["pair_ashtakavarga"]["b_in_a"][g]["bav"],
            av(&ca)["bhinnashtakavarga"][g]["bindus"][s_b],
            "{g} b→a"
        );
        assert_eq!(
            s["pair_ashtakavarga"]["a_in_b"][g]["sav"],
            av(&cb)["sarvashtakavarga"]["bindus"][s_a]
        );
    }
    // cross aspects: every 7th-house contact one way appears the other way too
    let ab = s["cross_aspects"]["a_onto_b"].as_array().unwrap();
    let ba = s["cross_aspects"]["b_onto_a"].as_array().unwrap();
    for x in ab.iter().filter(|x| x["house_from_aspecting"] == 7) {
        if ["sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn"]
            .contains(&x["to"].as_str().unwrap())
        {
            assert!(
                ba.iter()
                    .any(|y| y["from"] == x["to"] && y["to"] == x["from"] && y["house_from_aspecting"] == 7),
                "{x}"
            );
        }
    }
    // Maṅgala doṣa names its three reference points and keeps raw and cancellations apart
    for side in ["a", "b"] {
        for r in ["lagna", "moon", "venus"] {
            let x = &s["manglik_trace"][side]["by_reference"][r];
            assert!(
                x["raw"].is_boolean() && x["cancellations"].is_array(),
                "{side} {r}: {x}"
            );
        }
    }
}
