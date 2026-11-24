//! Rise, upper transit and set of any body, twilight, and topocentric positions (AST-08).
//!
//! Altitudes are geocentric from the provider's apparent place; the Moon's parallax is folded into its standard
//! altitude (as for the pañcāṅga sunrise) and topocentric positions apply the full parallax correction (Meeus,
//! *Astronomical Algorithms*, ch. 40) for the observer's latitude and height on the IERS ellipsoid. Refraction is the
//! standard 34′ at the horizon; no temperature or pressure model.

use crate::astro;
use crate::panchanga::Rising;
use eshana_core::{deg, norm360, rad, Body, Result};
use eshana_ephemeris::frames;
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde_json::{json, Value};

const REFRACTION: f64 = 0.5667;
const SUN_SEMIDIAMETER: f64 = 0.2667;
const AU_KM: f64 = 149_597_870.7;
const EARTH_RADIUS_KM: f64 = 6378.14;

/// Where the observer stands.
#[derive(Debug, Clone, Copy)]
pub struct Observer {
    /// Degrees north.
    pub lat: f64,
    /// Degrees east.
    pub lon: f64,
    /// Metres above the ellipsoid.
    pub height_m: f64,
}

/// Right ascension and declination (degrees) of an apparent ecliptic position, with the true obliquity of date.
fn equatorial(lam: f64, beta: f64, t: f64) -> (f64, f64) {
    let eps = rad(frames::true_obliquity_deg(t));
    let (l, b) = (rad(lam), rad(beta));
    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    let dec = (b.sin() * eps.cos() + b.cos() * eps.sin() * l.sin()).asin();
    (norm360(deg(ra)), deg(dec))
}

/// Ecliptic longitude and latitude (degrees) of an equatorial position, with the true obliquity of date.
fn ecliptic(ra: f64, dec: f64, t: f64) -> (f64, f64) {
    let eps = rad(frames::true_obliquity_deg(t));
    let (a, d) = (rad(ra), rad(dec));
    let lam = (a.sin() * eps.cos() + d.tan() * eps.sin()).atan2(a.cos());
    let beta = (d.sin() * eps.cos() - d.cos() * eps.sin() * a.sin()).asin();
    (norm360(deg(lam)), deg(beta))
}

/// The observer's geocentric coordinates ρ sin φ′ and ρ cos φ′ (Earth radii), IERS ellipsoid (b/a = 0.99664719).
fn rho(o: Observer) -> (f64, f64) {
    let phi = rad(o.lat);
    let u = (0.996_647_19 * phi.tan()).atan();
    let h = o.height_m / 6_378_140.0;
    (0.996_647_19 * u.sin() + h * phi.sin(), u.cos() + h * phi.cos())
}

/// Topocentric apparent ecliptic longitude and latitude from the geocentric ones (Meeus ch. 40): the shift is up
/// to about 1° for the Moon, a few arc-seconds for the planets, under 9″ for the Sun.
#[must_use]
pub fn topocentric(lam: f64, beta: f64, dist_au: f64, jd_ut: f64, o: Observer) -> (f64, f64) {
    let inst = Instant::from_jd_utc(jd_ut);
    let t = inst.t_tt();
    let (ra, dec) = equatorial(lam, beta, t);
    let (rs, rc) = rho(o);
    let sin_pi = rad(8.794 / 3600.0).sin() / dist_au;
    let ha = rad(astro::local_apparent_sidereal_deg(inst.gmst_deg(), o.lon, t) - ra);
    let d = rad(dec);
    let dra = (-rc * sin_pi * ha.sin()).atan2(d.cos() - rc * sin_pi * ha.cos());
    let dec_t = ((d.sin() - rs * sin_pi) * dra.cos()).atan2(d.cos() - rc * sin_pi * ha.cos());
    ecliptic(ra + deg(dra), deg(dec_t), t)
}

/// Geocentric altitude and hour angle (degrees, HA in (−180, 180]) of a body, and its distance in AU.
fn horizontal(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_ut: f64,
    o: Observer,
) -> Result<(f64, f64, Option<f64>)> {
    let inst = Instant::from_jd_utc(jd_ut);
    let t = inst.t_tt();
    let (lam, beta, dist) = p.apparent(inst.jd_tt, body)?;
    let (ra, dec) = equatorial(lam, beta, t);
    let last = astro::local_apparent_sidereal_deg(inst.gmst_deg(), o.lon, t);
    let ha = norm360(last - ra + 180.0) - 180.0;
    let (phi, d, h) = (rad(o.lat), rad(dec), rad(ha));
    Ok((
        deg((phi.sin() * d.sin() + phi.cos() * d.cos() * h.cos()).asin()),
        ha,
        dist,
    ))
}

/// The altitude a body's reference point has at rise and set under a convention: refraction, the Sun's and the
/// Moon's semidiameter (planets are points) and the Moon's horizontal parallax.
fn standard_altitude(body: Body, dist_au: Option<f64>, conv: Rising) -> f64 {
    match body {
        Body::Moon => {
            let km = dist_au.unwrap_or(0.002_57) * AU_KM;
            let parallax = deg((EARTH_RADIUS_KM / km).asin());
            match conv {
                Rising::UpperLimbRefracted => 0.7275 * parallax - REFRACTION,
                Rising::CentreRefracted => parallax - REFRACTION,
                Rising::CentreUnrefracted => parallax,
            }
        }
        Body::Sun => match conv {
            Rising::UpperLimbRefracted => -(REFRACTION + SUN_SEMIDIAMETER),
            Rising::CentreRefracted => -REFRACTION,
            Rising::CentreUnrefracted => 0.0,
        },
        _ => match conv {
            Rising::UpperLimbRefracted | Rising::CentreRefracted => -REFRACTION,
            Rising::CentreUnrefracted => 0.0,
        },
    }
}

/// Every crossing of `f` through zero in [a, b): (time, rising?) found by 10-minute steps and bisection.
fn crossings(a: f64, b: f64, f: &dyn Fn(f64) -> Result<f64>) -> Result<Vec<(f64, bool)>> {
    let step = 10.0 / 1440.0;
    let mut out = vec![];
    let (mut x0, mut f0) = (a, f(a)?);
    while x0 < b {
        let x1 = (x0 + step).min(b);
        let f1 = f(x1)?;
        if (f0 < 0.0) != (f1 < 0.0) {
            let (mut lo, mut hi, mut flo) = (x0, x1, f0);
            for _ in 0..30 {
                let mid = 0.5 * (lo + hi);
                let fm = f(mid)?;
                if (fm < 0.0) == (flo < 0.0) {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
            out.push((0.5 * (lo + hi), f0 < 0.0));
        }
        x0 = x1;
        f0 = f1;
    }
    Ok(out)
}

fn stamp(jd_ut: f64, offset_s: f64) -> Value {
    let local = Instant::from_jd_utc(jd_ut + offset_s / 86_400.0).utc;
    let sign = if offset_s < 0.0 { '-' } else { '+' };
    let m = (offset_s.abs() / 60.0).round() as i64;
    json!({"utc": Instant::from_jd_utc(jd_ut).utc, "local": format!("{}{sign}{:02}:{:02}", &local[..local.len() - 1], m / 60, m % 60), "jd_ut": jd_ut})
}

/// Rise, upper transit and set of `body` in the local day starting at UT JD `jd0` (local midnight).
///
/// # Errors
/// Provider errors (range, unsupported body).
pub fn body_day(
    p: &dyn EphemerisProvider,
    body: Body,
    jd0: f64,
    o: Observer,
    conv: Rising,
    offset_s: f64,
) -> Result<Value> {
    let alt_excess = |jd: f64| -> Result<f64> {
        let (alt, _, dist) = horizontal(p, body, jd, o)?;
        Ok(alt - standard_altitude(body, dist, conv))
    };
    let hor = crossings(jd0, jd0 + 1.0, &alt_excess)?;
    // upper transit: the hour angle passes 0 going up (it grows by ~360° a day; skip the ±180° wrap)
    let ha = |jd: f64| -> Result<f64> { Ok(horizontal(p, body, jd, o)?.1) };
    let transit = crossings(jd0, jd0 + 1.0, &ha)?
        .into_iter()
        .find(|(jd, up)| *up && ha(*jd).is_ok_and(|h| h.abs() < 1.0));
    let rise = hor.iter().find(|x| x.1).map(|x| x.0);
    let set = hor.iter().find(|x| !x.1).map(|x| x.0);
    let status = if !hor.is_empty() {
        "rises_and_sets"
    } else if alt_excess(jd0 + 0.5)? >= 0.0 {
        "above_horizon_all_day"
    } else {
        "below_horizon_all_day"
    };
    let tr = transit.map(|(jd, _)| {
        let mut v = stamp(jd, offset_s);
        v["altitude_deg"] = json!(horizontal(p, body, jd, o).map(|h| h.0).unwrap_or(f64::NAN));
        v
    });
    Ok(
        json!({"body": body.id(), "status": status, "rise": rise.map(|j| stamp(j, offset_s)), "transit": tr,
              "set": set.map(|j| stamp(j, offset_s))}),
    )
}

/// Civil, nautical and astronomical twilight for the local day starting at `jd0`: when the Sun's centre (geometric)
/// passes −6°, −12° and −18°, morning (`dawn`) and evening (`dusk`).
///
/// # Errors
/// Provider errors.
pub fn twilight(p: &dyn EphemerisProvider, jd0: f64, o: Observer, offset_s: f64) -> Result<Value> {
    let mut out = serde_json::Map::new();
    for (name, depth) in [("civil", -6.0), ("nautical", -12.0), ("astronomical", -18.0)] {
        let f = |jd: f64| -> Result<f64> { Ok(horizontal(p, Body::Sun, jd, o)?.0 - depth) };
        let c = crossings(jd0, jd0 + 1.0, &f)?;
        out.insert(
            name.into(),
            json!({
                "depression_deg": -depth,
                "dawn": c.iter().find(|x| x.1).map(|x| stamp(x.0, offset_s)),
                "dusk": c.iter().find(|x| !x.1).map(|x| stamp(x.0, offset_s)),
            }),
        );
    }
    Ok(Value::Object(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallax_moves_the_moon_most() {
        let o = Observer {
            lat: 51.5,
            lon: 0.0,
            height_m: 0.0,
        };
        let jd = 2_451_545.0;
        let (ml, mb) = topocentric(120.0, 2.0, 0.00257, jd, o); // the Moon's distance
        let (sl, sb) = topocentric(120.0, 2.0, 1.0, jd, o); // the Sun's
        let moon = ((ml - 120.0).powi(2) + (mb - 2.0).powi(2)).sqrt();
        let sun = ((sl - 120.0).powi(2) + (sb - 2.0).powi(2)).sqrt() * 3600.0;
        assert!(moon > 0.2 && moon < 1.05, "moon shift {moon}°");
        assert!(sun < 9.0, "sun shift {sun}″");
    }

    #[test]
    fn rho_at_the_equator_and_pole() {
        let (s, c) = rho(Observer {
            lat: 0.0,
            lon: 0.0,
            height_m: 0.0,
        });
        assert!(s.abs() < 1e-12 && (c - 1.0).abs() < 1e-12);
        let (s, c) = rho(Observer {
            lat: 90.0,
            lon: 0.0,
            height_m: 0.0,
        });
        assert!((s - 0.996_647_19).abs() < 1e-9 && c.abs() < 1e-9);
    }
}
