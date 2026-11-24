//! Secondary progressions as an engine operation: a day after birth for each year of life.
//!
//! The age is fractional and explicit: `age_years = (at − birth) / year_days`, with the year length named in the
//! result (the tropical year, 365.24219 days, by default). The progressed instant is `birth + age_years` days, and
//! the progressed bodies are ordinary positions at that instant.
//!
//! Progressed angles depend on a method, which must be named (family `progression_angles`):
//! - `solar_arc_longitude`: the MC advances by the Sun's progressed arc in longitude;
//! - `naibod_ra`: the right ascension of the MC advances by the Naibod rate, 0.985647° per year;
//! - `quotidian`: the local sidereal time at the progressed instant at the birth place (the "daily" angles).
//!
//! The Ascendant follows from the progressed MC's right ascension at the birth latitude. Without a method, a place
//! or an exact birth time the angles are *unavailable*: nothing is filled in.

use crate::astro;
use crate::methods::Method;
use eshana_core::{deg, norm360, rad, EshanaError, Result};
use eshana_ephemeris::frames::true_obliquity_deg;

/// The tropical year in days.
pub const TROPICAL_YEAR_DAYS: f64 = 365.242_19;
/// The Naibod rate: the Sun's mean daily motion in right ascension, degrees per year of life.
pub const NAIBOD_DEG_PER_YEAR: f64 = 0.985_647_358;

/// Angle methods.
pub const ANGLE_METHODS: &[Method] = &[
    Method {
        id: "solar_arc_longitude",
        status: "stable",
        note: "MC + the progressed Sun's arc in longitude",
    },
    Method {
        id: "naibod_ra",
        status: "stable",
        note: "RAMC + 0.985647° per year (Naibod)",
    },
    Method {
        id: "quotidian",
        status: "stable",
        note: "local sidereal time at the progressed instant at the birth place",
    },
];

/// Right ascension of the ecliptic point at longitude `lon` (latitude 0), degrees, at `t`.
#[must_use]
pub fn ra_of_ecliptic(lon: f64, t: f64) -> f64 {
    let e = rad(true_obliquity_deg(t));
    let l = rad(lon);
    norm360(deg((l.sin() * e.cos()).atan2(l.cos())))
}

/// Progressed RAMC (degrees) under a method.
///
/// `ramc_natal` and `mc_natal` (tropical) are the natal angles; `sun_arc` is progressed minus natal Sun longitude;
/// `lst_at_progressed` is the local apparent sidereal time at the progressed instant at the birth longitude.
///
/// # Errors
/// `ESH-METHOD` for an unknown id.
pub fn progressed_ramc(
    method: &str,
    ramc_natal: f64,
    mc_natal: f64,
    sun_arc: f64,
    age_years: f64,
    lst_at_progressed: f64,
    t: f64,
) -> Result<f64> {
    match method {
        "solar_arc_longitude" => Ok(ra_of_ecliptic(norm360(mc_natal + sun_arc), t)),
        "naibod_ra" => Ok(norm360(ramc_natal + NAIBOD_DEG_PER_YEAR * age_years)),
        "quotidian" => Ok(lst_at_progressed),
        other => Err(EshanaError::MethodUnsupported(format!(
            "'{other}' is not a progression_angles method (one of: {})",
            ANGLE_METHODS.iter().map(|m| m.id).collect::<Vec<_>>().join(", ")
        ))),
    }
}

/// Ascendant and MC (tropical) for a RAMC at a latitude.
#[must_use]
pub fn angles_from_ramc(ramc: f64, lat: f64, t: f64) -> (f64, f64) {
    astro::asc_mc(ramc, lat, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ra_of_ecliptic_inverts_the_mc_formula() {
        for k in 0..36 {
            let ramc = f64::from(k) * 10.0 + 3.0;
            let (_, mc) = astro::asc_mc(ramc, 40.0, 0.2);
            assert!(
                (ra_of_ecliptic(mc, 0.2) - ramc).abs() < 1e-9
                    || (ra_of_ecliptic(mc, 0.2) - ramc).abs() > 359.999_999
            );
        }
    }

    #[test]
    fn naibod_and_solar_arc() {
        assert!(
            (progressed_ramc("naibod_ra", 10.0, 0.0, 0.0, 30.0, 0.0, 0.0).unwrap()
                - (10.0 + 30.0 * NAIBOD_DEG_PER_YEAR))
                .abs()
                < 1e-12
        );
        let r = progressed_ramc("solar_arc_longitude", 0.0, 100.0, 29.5, 30.0, 0.0, 0.0).unwrap();
        assert!((angles_from_ramc(r, 40.0, 0.0).1 - 129.5).abs() < 1e-9);
        assert_eq!(
            progressed_ramc("placidus_arc", 0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                .unwrap_err()
                .code(),
            "ESH-METHOD"
        );
    }
}
