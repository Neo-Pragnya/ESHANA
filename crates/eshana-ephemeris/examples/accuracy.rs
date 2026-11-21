//! Measure a provider against the DE440s golden fixture and print a Markdown table.
//! `cargo run -p eshana-ephemeris --example accuracy -- analytic|spk [kernel.bsp]`
use eshana_core::{angular_diff, Body};
use eshana_ephemeris::{provider_by_id, ProviderOptions};
use eshana_time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let id = args.get(1).map_or("analytic", String::as_str);
    let opts = ProviderOptions {
        spk_path: args.get(2).cloned(),
    };
    let p = provider_by_id(id, &opts).expect("provider");
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../benchmarks/golden/positions_de440s.json")).unwrap();
    println!("| body | max lon err (arcsec) | median lon err (arcsec) | max lat err (arcsec) |");
    println!("|---|---:|---:|---:|");
    for b in Body::PLANETS {
        let (mut lon_e, mut lat_e) = (vec![], vec![]);
        for row in fixture.as_array().unwrap() {
            let utc = row["utc"].as_str().unwrap();
            let inst = Instant::parse_utc(utc).unwrap();
            let jd_tt = row["jd_tt"].as_f64().unwrap();
            let dt_diff = (inst.jd_tt - jd_tt) * 86400.0;
            if b == Body::Sun {
                eprintln!("{utc}: ESHANA dT - fixture dT = {dt_diff:+.2} s");
            }
            let r = &row["bodies"][b.id()];
            let Ok((lon, lat, _)) = p.apparent(jd_tt, b) else {
                continue;
            };
            lon_e.push(angular_diff(lon, r["lon"].as_f64().unwrap()).abs() * 3600.0);
            lat_e.push((lat - r["lat"].as_f64().unwrap()).abs() * 3600.0);
        }
        lon_e.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!(
            "| {} | {:.1} | {:.1} | {:.1} |",
            b.id(),
            lon_e.last().copied().unwrap_or(f64::NAN),
            lon_e[lon_e.len() / 2],
            lat_e.iter().cloned().fold(0.0, f64::max)
        );
    }
}
