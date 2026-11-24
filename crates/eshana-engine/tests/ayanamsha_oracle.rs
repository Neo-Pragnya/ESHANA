//! Every ayanāṃśa at several epochs (1800–2100) against an independent computation
//! (benchmarks/golden/ayanamsha_oracle.json, Swiss Ephemeris mean ayanāṃśa, offline).

use eshana_core::Ayanamsha;
use eshana_engine::astro;
use serde_json::Value;

#[test]
fn every_model_matches_the_oracle_at_every_epoch() {
    let o: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/ayanamsha_oracle.json")).unwrap();
    let mut worst: Vec<(String, f64)> = vec![];
    for m in Ayanamsha::ALL {
        let mut w: f64 = 0.0;
        for e in o["epochs"].as_array().unwrap() {
            let t = e["t_centuries_tt"].as_f64().unwrap();
            let want = e["ayanamsha_deg"][m.id()]
                .as_f64()
                .unwrap_or_else(|| panic!("{} missing", m.id()));
            let d = (astro::ayanamsha_deg(m, t) - want).abs() * 3600.0;
            w = w.max(d);
            // mean models: measured worst 0.05″ (De Luce) over 1800–2100; star-anchored models: see below
            let tol = if m.id().starts_with("true_") { 1.0 } else { 0.1 };
            assert!(d < tol, "{} at {}: {d:.3}″", m.id(), e["year"]);
        }
        worst.push((m.id().to_string(), w));
    }
    eprintln!("worst differences (″): {worst:?}");
}

/// The star-anchored models (AYA-02) through the seasons, 1800–2200: the annual aberration (±20″) and the Sun's
/// light deflection must be followed, not just the precession. Measured worst 0.62″, from the low-precision Sun
/// used for both corrections; the oracle reads its own star catalogue, so this also checks the Hipparcos data.
#[test]
fn star_anchored_models_follow_their_star_through_the_year() {
    let o: Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/ayanamsha_oracle.json")).unwrap();
    let rows = o["true_epochs"].as_array().unwrap();
    assert!(rows.len() >= 100);
    for m in [
        Ayanamsha::TrueCitra,
        Ayanamsha::TrueRevati,
        Ayanamsha::TruePushya,
        Ayanamsha::TrueMula,
    ] {
        let mut w: f64 = 0.0;
        for e in rows {
            let t = e["t_centuries_tt"].as_f64().unwrap();
            let want = e["ayanamsha_deg"][m.id()].as_f64().unwrap();
            let d = (astro::ayanamsha_deg(m, t) - want).abs() * 3600.0;
            w = w.max(d);
            assert!(d < 1.0, "{} at JD {}: {d:.3}″", m.id(), e["jd_tt"]);
        }
        eprintln!("{}: worst {w:.2}″", m.id());
    }
}

#[test]
fn every_model_states_its_epoch_and_frame() {
    for m in Ayanamsha::ALL {
        let d = astro::ayanamsha_definition(m);
        assert!(
            d["epoch"].as_str().unwrap().len() > 5 && d["frame"].as_str().unwrap().contains("equinox"),
            "{d}"
        );
        assert_eq!(Ayanamsha::parse(m.id()).unwrap(), m);
    }
}
