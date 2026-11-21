//! Reference-frame arithmetic shared by every provider: obliquity, nutation, ecliptic precession and the
//! conversion of an astrometric vector into an apparent longitude and latitude of date.

use eshana_core::{deg, norm360, rad};

/// Speed of light in astronomical units per day.
pub const C_AU_PER_DAY: f64 = 173.144_632_684_7;
/// Speed of light in km/s.
pub const C_KM_S: f64 = 299_792.458;
/// Kilometres per astronomical unit.
pub const AU_KM: f64 = 149_597_870.7;

/// Mean obliquity of the ecliptic (IAU 2006), degrees, `t` in Julian centuries of TT from J2000.
#[must_use]
pub fn mean_obliquity_deg(t: f64) -> f64 {
    let arcsec = 84_381.406 - 46.836_769 * t - 0.000_183_1 * t.powi(2) + 0.002_003_40 * t.powi(3)
        - 5.76e-7 * t.powi(4)
        - 4.34e-8 * t.powi(5);
    arcsec / 3600.0
}

/// Nutation in longitude and obliquity in degrees, from the four dominant terms of the 1980 series
/// (Meeus, ch. 22). Error is about half an arcsecond.
#[must_use]
pub fn nutation_deg(t: f64) -> (f64, f64) {
    let om = rad(125.04452 - 1934.136261 * t);
    let l = rad(280.4665 + 36000.7698 * t);
    let lm = rad(218.3165 + 481267.8813 * t);
    let dpsi = -17.20 * om.sin() - 1.32 * (2.0 * l).sin() - 0.23 * (2.0 * lm).sin() + 0.21 * (2.0 * om).sin();
    let deps = 9.20 * om.cos() + 0.57 * (2.0 * l).cos() + 0.10 * (2.0 * lm).cos() - 0.09 * (2.0 * om).cos();
    (dpsi / 3600.0, deps / 3600.0)
}

/// True obliquity (mean plus nutation) in degrees.
#[must_use]
pub fn true_obliquity_deg(t: f64) -> f64 {
    mean_obliquity_deg(t) + nutation_deg(t).1
}

/// General precession in longitude since J2000, degrees (IAU 2006 linear and quadratic terms).
#[must_use]
pub fn general_precession_deg(t: f64) -> f64 {
    (5028.796195 * t + 1.1054348 * t * t) / 3600.0
}

/// Carry an ecliptic longitude and latitude from the mean ecliptic and equinox of J2000 to the mean ecliptic and
/// equinox of date (Meeus, ch. 21).
#[must_use]
pub fn precess_ecliptic_from_j2000(lon_deg: f64, lat_deg: f64, t: f64) -> (f64, f64) {
    let eta = rad((47.0029 * t - 0.03302 * t * t + 0.000_060 * t.powi(3)) / 3600.0);
    let pi = rad(174.876_384 - 869.8089 * t / 3600.0 + 0.035_36 * t * t / 3600.0);
    let p = rad((5029.0966 * t + 1.11113 * t * t - 0.000_006 * t.powi(3)) / 3600.0);
    let (l0, b0) = (rad(lon_deg), rad(lat_deg));
    let a = eta.cos() * b0.cos() * (pi - l0).sin() - eta.sin() * b0.sin();
    let b = b0.cos() * (pi - l0).cos();
    let c = eta.cos() * b0.sin() + eta.sin() * b0.cos() * (pi - l0).sin();
    (norm360(deg(p + pi - a.atan2(b))), deg(c.asin()))
}

/// Ecliptic longitude and latitude (degrees) of a vector.
#[must_use]
pub fn lon_lat(v: [f64; 3]) -> (f64, f64) {
    let r = (v[0] * v[0] + v[1] * v[1]).sqrt();
    (norm360(deg(v[1].atan2(v[0]))), deg(v[2].atan2(r)))
}

/// Euclidean length.
#[must_use]
pub fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// From an astrometric geocentric vector in the ecliptic of J2000 (any length unit) and the observer's velocity
/// in AU/day in the same frame, produce the apparent true-equinox longitude and latitude of date.
///
/// Steps: first-order annual aberration, precession to the ecliptic of date, nutation in longitude.
/// Gravitational light deflection is not applied (at most 1.7″, only near the Sun).
#[must_use]
pub fn apparent_of_date(astrometric: [f64; 3], observer_vel_au_day: [f64; 3], t: f64) -> (f64, f64) {
    let d = norm(astrometric);
    let mut u = [astrometric[0] / d, astrometric[1] / d, astrometric[2] / d];
    for (k, x) in u.iter_mut().enumerate() {
        *x += observer_vel_au_day[k] / C_AU_PER_DAY;
    }
    let n = norm(u);
    let (lon, lat) = lon_lat([u[0] / n, u[1] / n, u[2] / n]);
    let (lon, lat) = precess_ecliptic_from_j2000(lon, lat, t);
    (norm360(lon + nutation_deg(t).0), lat)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obliquity_at_j2000() {
        assert!((mean_obliquity_deg(0.0) - 23.439_279_4).abs() < 1e-6);
    }

    #[test]
    fn nutation_meeus_example() {
        // Meeus example 22.a: 1987 April 10 0h TD, dpsi = -3.788", deps = +9.443" (full series).
        let t = (2_446_895.5 - 2_451_545.0) / 36_525.0;
        let (dp, de) = nutation_deg(t);
        assert!((dp * 3600.0 + 3.788).abs() < 0.6, "dpsi {}", dp * 3600.0);
        assert!((de * 3600.0 - 9.443).abs() < 0.6, "deps {}", de * 3600.0);
    }

    #[test]
    fn precession_is_identity_at_j2000() {
        let (l, b) = precess_ecliptic_from_j2000(123.0, 4.5, 0.0);
        assert!((l - 123.0).abs() < 1e-6 && (b - 4.5).abs() < 1e-6);
    }

    #[test]
    fn precession_moves_a_star_by_the_general_rate() {
        // One Julian century later, a star on the ecliptic moves ~1.396° in longitude.
        let (l, _) = precess_ecliptic_from_j2000(100.0, 0.0, 1.0);
        assert!((l - 101.397).abs() < 0.01, "{l}");
    }
}
