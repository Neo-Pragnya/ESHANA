//! Behavioural invariants from the vision document: no silent defaults, no invented birth times,
//! tropical/sidereal never merged, errors carry stable codes, and every derived value names its rule.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(v: Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&v.to_string())).unwrap()
}

fn kolkata() -> Value {
    json!({"date": "1992-05-28", "time": "20:05:00", "timezone": "Asia/Kolkata"})
}
fn hyd() -> Value {
    json!({"latitude": 17.385, "longitude": 78.487})
}

fn code(v: &Value) -> &str {
    assert_eq!(v["kind"], "error", "expected an error, got {v}");
    v["error"]["code"].as_str().unwrap()
}

#[test]
fn sidereal_without_an_ayanamsha_is_refused() {
    let r = run(
        json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "config": {"zodiac": "sidereal"}}),
    );
    assert_eq!(code(&r), "ESH-CONFIG");
    assert!(r["error"]["message"].as_str().unwrap().contains("never picks"));
}

#[test]
fn tropical_with_an_ayanamsha_is_refused() {
    let r = run(json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"},
        "config": {"zodiac": "tropical", "ayanamsha": "lahiri"}}));
    assert_eq!(code(&r), "ESH-CONFIG");
}

#[test]
fn assume_defaults_is_recorded_as_an_assumption() {
    let r = run(
        json!({"op": "chart", "system": "vedic", "assume_defaults": true, "when": kolkata(), "place": hyd()}),
    );
    assert_eq!(r["kind"], "computed_chart");
    assert_eq!(r["calculation"]["ayanamsha"], "lahiri");
    let a = r["provenance"]["assumptions"].to_string();
    assert!(a.contains("defaulted to lahiri"), "{a}");
}

#[test]
fn vedic_without_ayanamsha_or_assume_is_refused() {
    let r = run(json!({"op": "chart", "system": "vedic", "when": kolkata(), "place": hyd()}));
    assert_eq!(code(&r), "ESH-CONFIG");
}

#[test]
fn local_time_without_a_zone_is_refused() {
    let r = run(json!({"op": "positions", "when": {"date": "2000-01-01", "time": "10:00"}}));
    assert_eq!(code(&r), "ESH-INPUT");
}

#[test]
fn ambiguous_and_missing_local_times_have_distinct_codes() {
    let amb = run(
        json!({"op": "positions", "when": {"date": "2024-11-03", "time": "01:30", "timezone": "America/New_York"}}),
    );
    assert_eq!(code(&amb), "ESH-TIME-AMBIGUOUS");
    let gap = run(
        json!({"op": "positions", "when": {"date": "2024-03-10", "time": "02:30", "timezone": "America/New_York"}}),
    );
    assert_eq!(code(&gap), "ESH-TIME-NONEXISTENT");
    let ok = run(
        json!({"op": "positions", "when": {"date": "2024-11-03", "time": "01:30", "timezone": "America/New_York", "fold": "latest"}}),
    );
    assert_eq!(ok["input"]["instant_utc"], "2024-11-03T06:30:00Z");
}

#[test]
fn unknown_birth_time_gives_intervals_and_no_houses() {
    let r = run(
        json!({"op": "chart", "system": "western", "when": {"date": "1992-05-28", "timezone": "Asia/Kolkata"},
        "place": hyd(), "config": {"zodiac": "tropical"}}),
    );
    assert_eq!(r["kind"], "computed_chart");
    assert_eq!(r["input"]["birth_time_quality"], "date_only");
    assert!(r["houses"].is_null(), "houses must not be invented");
    assert_eq!(r["warnings"][0]["code"], "ESH-TIME-UNCERTAIN");
    let moon = r["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "moon")
        .unwrap();
    let iv = moon["longitude_interval_deg"].as_array().unwrap();
    let width = (iv[1].as_f64().unwrap() - iv[0].as_f64().unwrap()).rem_euclid(360.0);
    assert!(
        (width - 13.2).abs() < 1.5,
        "the Moon moves about 13 degrees in a day, got {width}"
    );
}

#[test]
fn a_sign_that_could_change_inside_the_window_is_flagged_not_guessed() {
    // Moon at 28.8 degrees of Pisces at this birth; a +-12 h window crosses into Aries.
    let r = run(json!({"op": "chart", "system": "vedic", "assume_defaults": true,
        "when": {"date": "1992-05-28", "timezone": "Asia/Kolkata"}}));
    let a = r["derived_attributes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == "vedic.moon.rashi")
        .unwrap();
    assert_eq!(a["value"]["stable_within_uncertainty"], false);
    assert_eq!(a["value"]["candidates"].as_array().unwrap().len(), 2);
}

#[test]
fn approximate_time_within_half_an_hour_keeps_houses() {
    let r = run(
        json!({"op": "chart", "system": "western", "when": {"date": "1992-05-28", "time": "20:05", "timezone": "Asia/Kolkata", "uncertainty_minutes": 5.0},
        "place": hyd(), "config": {"zodiac": "tropical"}}),
    );
    assert!(r["houses"].is_object());
    assert_eq!(r["input"]["birth_time_quality"], "approximate");
}

#[test]
fn placidus_at_the_polar_circle_is_a_latitude_error_not_a_guess() {
    let r = run(
        json!({"op": "chart", "system": "western", "when": {"utc": "2000-01-01T12:00:00Z"},
        "place": {"latitude": 75.0, "longitude": 10.0}, "config": {"zodiac": "tropical", "house_system": "placidus"}}),
    );
    assert_eq!(code(&r), "ESH-LATITUDE");
    let ok = run(
        json!({"op": "chart", "system": "western", "when": {"utc": "2000-01-01T12:00:00Z"},
        "place": {"latitude": 75.0, "longitude": 10.0}, "config": {"zodiac": "tropical", "house_system": "whole_sign"}}),
    );
    assert_eq!(ok["kind"], "computed_chart");
}

#[test]
fn out_of_range_instants_say_so() {
    let r = run(json!({"op": "positions", "when": {"utc": "1500-01-01T00:00:00Z"}}));
    assert_eq!(code(&r), "ESH-RANGE");
}

#[test]
fn spk_provider_without_a_kernel_is_a_config_error() {
    std::env::remove_var("ESHANA_SPK");
    let r = run(
        json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "config": {"provider": "spk"}}),
    );
    assert_eq!(code(&r), "ESH-CONFIG");
    let bad = run(
        json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "config": {"provider": "spk"}, "spk_path": "/no/such.bsp"}),
    );
    assert_eq!(code(&bad), "ESH-PROVIDER");
}

#[test]
fn bad_requests_are_input_errors_with_a_code() {
    let r: Value = serde_json::from_str(&Engine::new().execute_json("{not json")).unwrap();
    assert_eq!(code(&r), "ESH-INPUT");
    let r2 = run(json!({"op": "chart", "when": {"utc": "2000-01-01T12:00:00Z"}}));
    assert_eq!(code(&r2), "ESH-INPUT"); // no system
    let r3 = run(json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "surprise": 1}));
    assert_eq!(code(&r3), "ESH-INPUT"); // unknown field
}

#[test]
fn every_derived_attribute_names_its_rule_and_epistemic_layer() {
    let r = run(
        json!({"op": "chart", "system": "vedic", "assume_defaults": true, "when": kolkata(), "place": hyd(),
        "divisions": [9], "dasha": true}),
    );
    let d = r["derived_attributes"].as_array().unwrap();
    assert!(d.len() > 30);
    for a in d {
        assert!(a["rule_version"].as_str().unwrap().contains('@'), "{a}");
        assert!(a["tradition_id"].as_str().unwrap().len() > 2, "{a}");
        assert_eq!(a["epistemic"], "tradition_rule");
        assert!(!a["derived_from"].as_array().unwrap().is_empty(), "{a}");
    }
    for p in r["positions"].as_array().unwrap() {
        assert_eq!(p["epistemic"], "calculated");
    }
    assert_eq!(
        r["interpretations"].as_array().unwrap().len(),
        0,
        "the engine never emits interpretations itself"
    );
    assert!(r["provenance"]["rulesets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == "vedic.parashari@0.3"));
    assert!(r["provenance"]["rulesets"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x == "ayanamsha-lahiri@0.1"));
}

#[test]
fn tropical_and_sidereal_longitudes_are_both_reported_and_differ_by_the_ayanamsha() {
    let r = run(
        json!({"op": "positions", "when": {"utc": "2000-01-01T12:00:00Z"}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}),
    );
    let sun = r["positions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == "sun")
        .unwrap();
    let diff = (sun["tropical_longitude_deg"].as_f64().unwrap() - sun["longitude_deg"].as_f64().unwrap())
        .rem_euclid(360.0);
    assert!((diff - 23.853).abs() < 0.01, "{diff}");
}

#[test]
fn western_aspects_carry_orb_and_applying_state() {
    let r = run(
        json!({"op": "chart", "system": "western", "aspects": true, "when": {"utc": "2000-01-01T12:00:00Z"},
        "config": {"zodiac": "tropical"}}),
    );
    let asp: Vec<&Value> = r["derived_attributes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["id"].as_str().unwrap().starts_with("western.aspect."))
        .collect();
    assert!(!asp.is_empty());
    // At J2000 the Sun is at 280.4 and the Moon at 223.3: 57 degrees apart, a sextile within a 3 degree orb.
    let sm = asp
        .iter()
        .find(|a| a["id"] == "western.aspect.moon-sun")
        .expect("Moon-Sun aspect at J2000");
    assert_eq!(sm["value"]["aspect"], "sextile");
    assert!(sm["value"]["orb_deg"].as_f64().unwrap() < 4.0);
    for a in &asp {
        assert!(a["value"]["orb_deg"].as_f64().unwrap() >= 0.0);
        assert!(a["value"]["applying"].is_boolean());
    }
}

#[test]
fn human_design_requires_an_exact_time_and_solves_88_degrees() {
    let bad = run(json!({"op": "human_design", "when": {"date": "1992-05-28", "timezone": "Asia/Kolkata"}}));
    assert_eq!(code(&bad), "ESH-TIME-UNCERTAIN");
    let r = run(json!({"op": "human_design", "when": kolkata(), "place": hyd()}));
    assert_eq!(r["kind"], "human_design");
    let arc = r["calculation"]["design_arc_deg_achieved"].as_f64().unwrap();
    assert!((arc - 88.0).abs() < 1e-6, "{arc}");
    let days = r["calculation"]["design_offset_days"].as_f64().unwrap();
    assert!((85.0..=92.0).contains(&days), "88 degrees is not 88 days: {days}");
    let acts = r["positions"]["personality"]["activations"].as_array().unwrap();
    assert_eq!(acts.len(), 13);
    for a in acts {
        assert!((1..=64).contains(&a["gate"].as_u64().unwrap()));
        assert!((1..=6).contains(&a["line"].as_u64().unwrap()));
    }
    let earth = acts.iter().find(|a| a["body"] == "earth").unwrap();
    let sun = acts.iter().find(|a| a["body"] == "sun").unwrap();
    let d =
        (earth["longitude_deg"].as_f64().unwrap() - sun["longitude_deg"].as_f64().unwrap()).rem_euclid(360.0);
    assert!((d - 180.0).abs() < 1e-9);
    let bg = &r["derived_attributes"][0]["value"];
    assert!(bg["profile"].as_str().unwrap().contains('/'));
    assert!(r["provenance"]["assumptions"].to_string().contains("ADR-009"));
    let sid = run(
        json!({"op": "human_design", "when": kolkata(), "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}),
    );
    assert_eq!(code(&sid), "ESH-CONFIG");
}

#[test]
fn numerology_end_to_end() {
    let r = run(
        json!({"op": "numerology", "name": "Ada Lovelace", "date_of_birth": "1815-12-10", "reference_year": 2026}),
    );
    assert_eq!(r["kind"], "numerology_analysis");
    let ids: Vec<&str> = r["derived_attributes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "numerology.name",
            "numerology.life_path",
            "numerology.birth_day",
            "numerology.personal_year"
        ]
    );
    let bad = run(json!({"op": "numerology", "date_of_birth": "2023-02-29"}));
    assert_eq!(code(&bad), "ESH-INPUT");
    let none = run(json!({"op": "numerology"}));
    assert_eq!(code(&none), "ESH-INPUT");
    let chal = run(json!({"op": "numerology", "name": "Ada", "school": "chaldean"}));
    assert_eq!(chal["provenance"]["rulesets"][0], "numerology.chaldean@0.1");
}

#[test]
fn compare_explains_differences_as_configuration() {
    let r = run(
        json!({"op": "compare", "base": {"system": "vedic", "when": kolkata(), "place": hyd(),
            "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}},
        "a": {}, "b": {"ayanamsha": "fagan_bradley"}}),
    );
    assert_eq!(r["kind"], "method_comparison");
    assert_eq!(r["configuration_differences"], json!(["ayanamsha"]));
    assert!(r["summary"]["changes"].as_u64().unwrap() > 0);
    // Fagan-Bradley is about 0.9 degrees ahead of Lahiri: every longitude shifts by that.
    let first = &r["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["kind"] == "position")
        .unwrap()["difference_deg"];
    assert!((first.as_f64().unwrap() + 0.883).abs() < 0.05, "{first}");
    assert!(r["note"].as_str().unwrap().contains("not software defects"));
}

#[test]
fn mean_versus_true_node_is_a_visible_difference() {
    let r = run(
        json!({"op": "compare", "base": {"system": "western", "when": {"utc": "2024-02-29T18:45:30Z"}, "config": {"zodiac": "tropical"}},
        "a": {"node_model": "mean"}, "b": {"node_model": "true"}}),
    );
    assert_eq!(r["configuration_differences"], json!(["node_model"]));
    let ids: Vec<String> = r["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.iter().any(|k| k == "position_only_in_a") && ids.iter().any(|k| k == "position_only_in_b"));
}

#[test]
fn graph_exports_are_well_formed_and_never_claim_causation() {
    let base = json!({"system": "vedic", "assume_defaults": true, "when": kolkata(), "place": hyd(), "divisions": [9]});
    let g = run(json!({"op": "graph", "chart": base, "format": "json"}));
    assert_eq!(g["kind"], "symbolic_graph");
    let edges = g["edges"].as_array().unwrap();
    assert!(edges
        .iter()
        .all(|e| e["relation_class"] != "causal" && e["relation_class"] != "correlational"));
    assert!(edges
        .iter()
        .any(|e| e["kind"] == "classified_as" && e["relation_class"] == "traditional_correspondence"));
    assert!(edges
        .iter()
        .any(|e| e["kind"] == "calculated_from" && e["relation_class"] == "computational"));
    assert!(edges.iter().any(|e| e["kind"] == "cites"));
    let ids: std::collections::BTreeSet<&str> = g["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap())
        .collect();
    for e in edges {
        assert!(
            ids.contains(e["from"].as_str().unwrap()) && ids.contains(e["to"].as_str().unwrap()),
            "dangling edge {e}"
        );
    }
    let m = run(json!({"op": "graph", "chart": base, "format": "mermaid"}));
    assert!(m["content"].as_str().unwrap().starts_with("flowchart LR"));
    let x = run(json!({"op": "graph", "chart": base, "format": "graphml"}));
    let xml = x["content"].as_str().unwrap();
    assert!(xml.contains("<graphml") && xml.contains("</graphml>"));
    let bad = run(json!({"op": "graph", "chart": base, "format": "visio"}));
    assert_eq!(code(&bad), "ESH-INPUT");
}

#[test]
fn identical_requests_give_identical_bytes() {
    let req = json!({"op": "chart", "system": "vedic", "assume_defaults": true, "when": kolkata(), "place": hyd(), "divisions": [1, 9, 10], "dasha": true}).to_string();
    let e = Engine::new();
    assert_eq!(e.execute_json(&req), e.execute_json(&req));
    assert_eq!(Engine::new().execute_json(&req), e.execute_json(&req));
}

#[test]
fn info_lists_every_error_code() {
    let r = run(json!({"op": "info"}));
    assert_eq!(
        r["error_codes"].as_array().unwrap().len(),
        eshana_core::EshanaError::catalog().len()
    );
    assert!(r["expansion"]
        .as_str()
        .unwrap()
        .starts_with("Extensible Spatiotemporal Hypersystem"));
}

/// Regression: whole-sign cusps were floored in the tropical zodiac before the ayanamsha was removed, so a
/// sidereal Ascendant early in Kanya produced houses starting in Simha. Sweep a day of Ascendants in both zodiacs.
#[test]
fn whole_sign_cusps_start_at_the_ascendant_sign_in_both_zodiacs() {
    for hour in 0..24 {
        for (system, zodiac, ayan) in [
            ("vedic", "sidereal", Some("lahiri")),
            ("western", "tropical", None),
        ] {
            let mut cfg = json!({"zodiac": zodiac, "house_system": "whole_sign"});
            if let Some(a) = ayan {
                cfg["ayanamsha"] = json!(a);
            }
            let r = run(
                json!({"op": "chart", "system": system, "when": {"utc": format!("1990-05-15T{hour:02}:10:00Z")},
                "place": {"latitude": 28.6139, "longitude": 77.209}, "config": cfg}),
            );
            let asc = r["houses"]["ascendant"]["longitude_deg"].as_f64().unwrap();
            let c0 = r["houses"]["cusps_deg"][0].as_f64().unwrap();
            assert_eq!(
                c0,
                (asc / 30.0).floor() * 30.0,
                "{zodiac} {hour}h: asc {asc} cusp1 {c0}"
            );
        }
    }
}

#[test]
fn south_node_carries_its_tropical_longitude() {
    let r = run(
        json!({"op": "chart", "system": "vedic", "when": {"utc": "1990-05-15T09:00:00Z"},
        "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}}),
    );
    let pos = r["positions"].as_array().unwrap();
    let get = |id: &str| pos.iter().find(|p| p["id"] == id).unwrap().clone();
    let (n, s) = (get("mean_node"), get("south_node"));
    let d = (s["tropical_longitude_deg"].as_f64().unwrap() - n["tropical_longitude_deg"].as_f64().unwrap())
        .rem_euclid(360.0);
    assert!((d - 180.0).abs() < 1e-9);
}

fn attr<'a>(r: &'a Value, id: &str) -> &'a Value {
    r["derived_attributes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

/// Drishti inside a varga follows the rāśi-chart rules on the varga's signs: every graha aspects the 7th sign from
/// where it sits in that varga (plus its special aspects), and the vargas reported are exactly those requested.
#[test]
fn varga_drishti_uses_the_varga_signs() {
    let r = run(
        json!({"op": "chart", "system": "vedic", "when": kolkata(), "place": hyd(),
        "config": {"zodiac": "sidereal", "ayanamsha": "lahiri", "house_system": "whole_sign", "node_model": "mean"},
        "divisions": [9, 10, 60], "features": ["drishti", "aspects"]}),
    );
    let vargas = attr(&r, "vedic.drishti.vargas")["value"]["vargas"]
        .as_array()
        .unwrap();
    let divs: Vec<u64> = vargas.iter().map(|v| v["division"].as_u64().unwrap()).collect();
    assert_eq!(divs, [9, 10, 60]);
    for v in vargas {
        let d = v["division"].as_u64().unwrap();
        for g in ["sun", "moon", "mars", "jupiter", "saturn"] {
            let sign = attr(&r, &format!("vedic.{g}.d{d}"))["value"]["index"]
                .as_u64()
                .unwrap();
            let seventh = v["graha"]
                .as_array()
                .unwrap()
                .iter()
                .find(|x| x["from"] == g && x["house_from_graha"] == 7)
                .unwrap();
            assert_eq!(seventh["sign"].as_u64().unwrap(), (sign + 6) % 12, "{g} in D{d}");
        }
        assert!(v["rashi"].as_array().is_some_and(|a| !a.is_empty()));
    }
    // sputa drishti: D1, D9 and every requested varga, each with houses from its own lagna
    let asp = &attr(&r, "vedic.aspects")["value"];
    assert_eq!(asp["divisions"], json!([1, 9, 10, 60]));
    for k in ["d1", "d9", "d10", "d60"] {
        assert!(asp[k].is_object(), "{k}");
        assert!(asp["houses"][k]["sun"]["lagna"].is_object(), "houses {k}");
    }
}
