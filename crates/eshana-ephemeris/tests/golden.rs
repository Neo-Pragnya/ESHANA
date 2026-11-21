//! Golden accuracy gate against JPL DE440s positions computed independently with Skyfield 1.55
//! (apparent geocentric ecliptic of date, light-time + aberration + deflection, IAU 2000A nutation).
//! The fixture's own TT is used so the test isolates the provider from the time-scale model.

use eshana_core::{angular_diff, Body};
use eshana_ephemeris::{provider_by_id, EphemerisProvider, ProviderOptions};

fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../benchmarks/golden/positions_de440s.json")).unwrap()
}

fn check(p: &dyn EphemerisProvider, tol_arcsec: impl Fn(Body) -> f64) {
    for row in fixture().as_array().unwrap() {
        let jd_tt = row["jd_tt"].as_f64().unwrap();
        for b in Body::PLANETS {
            let r = &row["bodies"][b.id()];
            let (lon, lat, _) = p.apparent(jd_tt, b).unwrap();
            let e_lon = angular_diff(lon, r["lon"].as_f64().unwrap()).abs() * 3600.0;
            let e_lat = (lat - r["lat"].as_f64().unwrap()).abs() * 3600.0;
            let tol = tol_arcsec(b);
            assert!(
                e_lon <= tol,
                "{} {}: lon error {e_lon:.1}\" > {tol}\"",
                row["utc"],
                b.id()
            );
            assert!(
                e_lat <= tol,
                "{} {}: lat error {e_lat:.1}\" > {tol}\"",
                row["utc"],
                b.id()
            );
        }
    }
}

#[test]
fn analytic_provider_stays_within_its_stated_error() {
    let p = provider_by_id("analytic", &ProviderOptions::default()).unwrap();
    check(p.as_ref(), |b| match b {
        Body::Sun | Body::Moon | Body::Mercury | Body::Venus | Body::Mars => 60.0,
        Body::Jupiter => 600.0,
        Body::Saturn => 1800.0,
        _ => 200.0,
    });
}

#[test]
fn spk_provider_agrees_to_two_arcseconds_when_a_kernel_is_available() {
    let Ok(path) = std::env::var("ESHANA_SPK") else {
        eprintln!("ESHANA_SPK not set; skipping the SPK golden check (the CI job that has a kernel runs it)");
        return;
    };
    let p = provider_by_id("spk", &ProviderOptions { spk_path: Some(path) }).unwrap();
    check(p.as_ref(), |_| 2.0);
}

#[test]
fn speeds_have_the_right_sign_and_size() {
    let p = provider_by_id("analytic", &ProviderOptions::default()).unwrap();
    // Mean Moon speed is about 13.2 deg/day; Sun about 1 deg/day; Mars retrograde in Dec 2022.
    let jd = 2_451_545.0;
    let moon = p.position(jd, Body::Moon).unwrap();
    assert!((moon.speed_deg_per_day - 13.2).abs() < 2.0);
    let sun = p.position(jd, Body::Sun).unwrap();
    assert!((sun.speed_deg_per_day - 1.0).abs() < 0.05);
    let mars_retro = p.position(2_459_920.5, Body::Mars).unwrap(); // 2022-12-01
    assert!(
        mars_retro.is_retrograde(),
        "Mars should be retrograde on 2022-12-01"
    );
    let node = p.position(jd, Body::MeanNode).unwrap();
    assert!(node.speed_deg_per_day < 0.0, "the mean node always regresses");
}

#[test]
fn spk_serves_every_body_across_the_whole_kernel_span() {
    // Regression: the mean node once borrowed the analytic provider's 1800-2100 range and failed near 2150.
    let Ok(path) = std::env::var("ESHANA_SPK") else {
        return;
    };
    let p = provider_by_id("spk", &ProviderOptions { spk_path: Some(path) }).unwrap();
    let info = p.info();
    for jd in [info.jd_min + 1.0, 2_451_545.0, info.jd_max - 1.0] {
        for b in Body::PLANETS.into_iter().chain([Body::MeanNode, Body::TrueNode]) {
            p.position(jd, b)
                .unwrap_or_else(|e| panic!("{} at JD {jd}: {e}", b.id()));
        }
    }
    assert!(p.apparent(info.jd_max + 10.0, Body::Sun).is_err());
}
