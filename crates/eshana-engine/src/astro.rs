//! Coordinate geometry: ayanāṃśa, signs, Ascendant and Midheaven, houses and aspects.

use eshana_core::{angular_diff, deg, norm360, rad, Ayanamsha, EshanaError, HouseSystem, Result};
use eshana_ephemeris::frames::{general_precession_deg, nutation_deg, true_obliquity_deg};

/// Western sign names, index 0 = Aries.
pub const SIGNS: [&str; 12] = [
    "Aries",
    "Taurus",
    "Gemini",
    "Cancer",
    "Leo",
    "Virgo",
    "Libra",
    "Scorpio",
    "Sagittarius",
    "Capricorn",
    "Aquarius",
    "Pisces",
];

/// Sanskrit rāśi names, index 0 = Meṣa.
pub const RASHIS: [&str; 12] = [
    "Mesha",
    "Vrishabha",
    "Mithuna",
    "Karka",
    "Simha",
    "Kanya",
    "Tula",
    "Vrishchika",
    "Dhanu",
    "Makara",
    "Kumbha",
    "Meena",
];

/// Index of the sign (0–11) a longitude falls in.
#[must_use]
pub fn sign_index(lon_deg: f64) -> usize {
    ((norm360(lon_deg) / 30.0).floor() as usize).min(11)
}

/// Mean ayanāṃśa in degrees at TT Julian centuries `t` since J2000.
///
/// Each model is anchored to its value at J2000.0 (taken from the Swiss Ephemeris as a published reference
/// number) and carried forward with the IAU 2006 general precession in longitude. Lahiri's anchor also follows
/// from the Indian Calendar Reform Committee value of 23°15′00.658″ on 1956-03-21. The measured deviation from
/// the Swiss Ephemeris over 1950–2040 is checked in the golden tests.
#[must_use]
pub fn ayanamsha_deg(model: Ayanamsha, t: f64) -> f64 {
    let at_j2000 = match model {
        Ayanamsha::Lahiri => 23.857_092_354,
        Ayanamsha::Raman => 22.410_791_040,
        Ayanamsha::Krishnamurti => 23.760_240_040,
        Ayanamsha::FaganBradley => 24.740_299_994,
        // Mean values at J2000.0 (TT) from the Swiss Ephemeris tables, as published reference numbers.
        Ayanamsha::Yukteshwar => 22.478_803_0,
        Ayanamsha::JnBhasin => 22.762_137_0,
        Ayanamsha::Ushashashi => 20.057_541_0,
        Ayanamsha::Sassanian => 19.992_959_4,
        Ayanamsha::Deluce => 27.815_752_8,
        Ayanamsha::DjwhalKhul => 28.359_678_6,
    };
    at_j2000 + general_precession_deg(t)
}

/// How a model is defined: its anchor epoch and value, the frame it is measured in, and where the number comes from.
#[must_use]
pub fn ayanamsha_definition(model: Ayanamsha) -> serde_json::Value {
    let (epoch, src) = match model {
        Ayanamsha::Lahiri => (
            "1956-03-21 (23°15′00.658″, Indian Calendar Reform Committee); carried as its J2000.0 value",
            "Indian Calendar Reform Committee; Swiss Ephemeris SIDM_LAHIRI",
        ),
        Ayanamsha::Raman => (
            "J2000.0 value of B. V. Raman's definition",
            "Swiss Ephemeris SIDM_RAMAN",
        ),
        Ayanamsha::Krishnamurti => (
            "J2000.0 value of K. S. Krishnamurti's definition",
            "Swiss Ephemeris SIDM_KRISHNAMURTI",
        ),
        Ayanamsha::FaganBradley => (
            "J2000.0 value of the Fagan–Bradley definition",
            "Swiss Ephemeris SIDM_FAGAN_BRADLEY",
        ),
        Ayanamsha::Yukteshwar => ("J2000.0 (22.478803°)", "Swiss Ephemeris SIDM_YUKTESHWAR"),
        Ayanamsha::JnBhasin => ("J2000.0 (22.762137°)", "Swiss Ephemeris SIDM_JN_BHASIN"),
        Ayanamsha::Ushashashi => ("J2000.0 (20.057541°)", "Swiss Ephemeris SIDM_USHASHASHI"),
        Ayanamsha::Sassanian => (
            "zero point AD 564; carried as its J2000.0 value",
            "Swiss Ephemeris SIDM_SASSANIAN",
        ),
        Ayanamsha::Deluce => (
            "zero point at the start of the era; carried as its J2000.0 value",
            "Swiss Ephemeris SIDM_DELUCE",
        ),
        Ayanamsha::DjwhalKhul => ("J2000.0 (28.359679°)", "Swiss Ephemeris SIDM_DJWHAL_KHUL"),
    };
    serde_json::json!({
        "id": model.id(), "epoch": epoch, "value_at_j2000_deg": ayanamsha_deg(model, 0.0),
        "frame": "mean equinox and ecliptic of date; carried from J2000.0 by the IAU 2006 general precession in longitude; nutation removed from the apparent longitude before it is subtracted",
        "source": src,
    })
}

/// Identifier used in rule provenance for each ayanāṃśa anchor.
#[must_use]
pub fn ayanamsha_rule(model: Ayanamsha) -> String {
    format!("ayanamsha-{}@0.1", model.id())
}

/// Sidereal longitude from an apparent (true-equinox) tropical longitude: remove the nutation in longitude to
/// reach the mean equinox of date, then subtract the mean ayanāṃśa. This matches the Swiss Ephemeris convention.
#[must_use]
pub fn to_sidereal(apparent_tropical_deg: f64, model: Ayanamsha, t: f64) -> f64 {
    norm360(apparent_tropical_deg - nutation_deg(t).0 - ayanamsha_deg(model, t))
}

/// Ascendant and Midheaven (tropical, true equinox of date), degrees.
///
/// `lst_deg` is the local apparent sidereal time (the right ascension of the Midheaven).
#[must_use]
pub fn asc_mc(lst_deg: f64, lat_deg: f64, t: f64) -> (f64, f64) {
    let eps = rad(true_obliquity_deg(t));
    let th = rad(lst_deg);
    let phi = rad(lat_deg);
    let mc = deg(th.sin().atan2(th.cos() * eps.cos()));
    let asc = deg(th.cos().atan2(-(th.sin() * eps.cos() + phi.tan() * eps.sin())));
    let (mc, mut asc) = (norm360(mc), norm360(asc));
    // The Ascendant is always within 0..180° east of the Midheaven (counter-clockwise in longitude).
    if norm360(asc - mc) > 180.0 {
        asc = norm360(asc + 180.0);
    }
    (asc, mc)
}

/// Local apparent sidereal time in degrees: Greenwich mean sidereal time plus the equation of the equinoxes
/// plus east longitude.
#[must_use]
pub fn local_apparent_sidereal_deg(gmst_deg: f64, east_longitude_deg: f64, t: f64) -> f64 {
    let (dpsi, _) = nutation_deg(t);
    norm360(gmst_deg + dpsi * rad(true_obliquity_deg(t)).cos() + east_longitude_deg)
}

/// House cusps, twelve values, tropical true-equinox-of-date degrees. Index 0 is house 1.
///
/// # Errors
/// [`EshanaError::LatitudeUnsupported`] for Placidus at or beyond the polar circles, or when the semi-arc
/// iteration has no solution.
pub fn houses(system: HouseSystem, lst_deg: f64, lat_deg: f64, t: f64) -> Result<[f64; 12]> {
    let (asc, mc) = asc_mc(lst_deg, lat_deg, t);
    let mut c = [0.0; 12];
    match system {
        HouseSystem::WholeSign => {
            let start = (asc / 30.0).floor() * 30.0;
            for (i, v) in c.iter_mut().enumerate() {
                *v = norm360(start + 30.0 * i as f64);
            }
        }
        HouseSystem::Equal => {
            for (i, v) in c.iter_mut().enumerate() {
                *v = norm360(asc + 30.0 * i as f64);
            }
        }
        HouseSystem::Porphyry | HouseSystem::Sripati => {
            let ic = norm360(mc + 180.0);
            let dsc = norm360(asc + 180.0);
            let tri = |from: f64, to: f64, k: f64| norm360(from + norm360(to - from) * k / 3.0);
            let p = [
                asc,
                tri(asc, ic, 1.0),
                tri(asc, ic, 2.0),
                ic,
                tri(ic, dsc, 1.0),
                tri(ic, dsc, 2.0),
                dsc,
                tri(dsc, mc, 1.0),
                tri(dsc, mc, 2.0),
                mc,
                tri(mc, asc, 1.0),
                tri(mc, asc, 2.0),
            ];
            if system == HouseSystem::Porphyry {
                c = p;
            } else {
                for (k, v) in c.iter_mut().enumerate() {
                    let prev = p[(k + 11) % 12];
                    *v = norm360(prev + norm360(p[k] - prev) / 2.0);
                }
            }
        }
        HouseSystem::Placidus => {
            let eps_deg = true_obliquity_deg(t);
            if lat_deg.abs() >= 90.0 - eps_deg {
                return Err(EshanaError::LatitudeUnsupported(format!(
                    "Placidus houses are undefined at latitude {lat_deg:.2}° (polar circle is {:.2}°); \
                     use whole_sign or equal",
                    90.0 - eps_deg
                )));
            }
            let eps = rad(eps_deg);
            let phi = rad(lat_deg);
            // Cusp by fixed-point iteration on the semi-diurnal arc of the cusp's own declination.
            let cusp = |above: bool, frac: f64| -> Result<f64> {
                let mut sda = 90.0_f64;
                let mut lon = 0.0;
                for _ in 0..200 {
                    let ra = if above {
                        lst_deg + frac * sda
                    } else {
                        lst_deg + 180.0 - frac * (180.0 - sda)
                    };
                    let ra_r = rad(ra);
                    lon = deg(ra_r.sin().atan2(ra_r.cos() * eps.cos()));
                    let dec = (eps.sin() * rad(lon).sin()).asin();
                    let x = -phi.tan() * dec.tan();
                    if x.abs() >= 1.0 {
                        return Err(EshanaError::LatitudeUnsupported(format!(
                            "Placidus cusp does not exist at latitude {lat_deg:.2}° for this sidereal time \
                             (circumpolar degree); use whole_sign or equal"
                        )));
                    }
                    let next = deg(x.acos());
                    if (next - sda).abs() < 1e-11 {
                        sda = next;
                        break;
                    }
                    sda = next;
                }
                let _ = sda;
                Ok(norm360(lon))
            };
            let c11 = cusp(true, 1.0 / 3.0)?;
            let c12 = cusp(true, 2.0 / 3.0)?;
            let c2 = cusp(false, 2.0 / 3.0)?;
            let c3 = cusp(false, 1.0 / 3.0)?;
            c = [
                asc,
                c2,
                c3,
                norm360(mc + 180.0),
                norm360(c11 + 180.0),
                norm360(c12 + 180.0),
                norm360(asc + 180.0),
                norm360(c2 + 180.0),
                norm360(c3 + 180.0),
                mc,
                c11,
                c12,
            ];
        }
    }
    Ok(c)
}

/// House number (1–12) that contains a longitude, given the cusps.
#[must_use]
pub fn house_of(lon_deg: f64, cusps: &[f64; 12]) -> usize {
    for i in 0..12 {
        let start = cusps[i];
        let end = cusps[(i + 1) % 12];
        let span = norm360(end - start);
        let off = norm360(lon_deg - start);
        if off < span || (span == 0.0 && off == 0.0) {
            return i + 1;
        }
    }
    12
}

/// A named aspect.
#[derive(Debug, Clone, Copy)]
pub struct AspectDef {
    /// Name.
    pub name: &'static str,
    /// Exact angle in degrees.
    pub angle: f64,
    /// Default orb in degrees.
    pub orb: f64,
}

/// Ptolemaic aspects with conventional orbs. Orbs are a tradition-level choice and can be scaled by the caller.
pub const PTOLEMAIC: [AspectDef; 5] = [
    AspectDef {
        name: "conjunction",
        angle: 0.0,
        orb: 8.0,
    },
    AspectDef {
        name: "sextile",
        angle: 60.0,
        orb: 4.0,
    },
    AspectDef {
        name: "square",
        angle: 90.0,
        orb: 6.0,
    },
    AspectDef {
        name: "trine",
        angle: 120.0,
        orb: 6.0,
    },
    AspectDef {
        name: "opposition",
        angle: 180.0,
        orb: 8.0,
    },
];

/// One aspect between two bodies.
#[derive(Debug, Clone, PartialEq)]
pub struct Aspect {
    /// Aspect name.
    pub name: &'static str,
    /// Exact angle.
    pub angle: f64,
    /// Actual angular separation in degrees, `[0, 180]`.
    pub separation: f64,
    /// Distance from exactness in degrees (always ≥ 0).
    pub orb: f64,
    /// True when the aspect is getting tighter.
    pub applying: bool,
}

/// Aspect between two longitudes with their speeds, or `None` when outside every orb.
#[must_use]
pub fn aspect_between(lon_a: f64, speed_a: f64, lon_b: f64, speed_b: f64, orb_scale: f64) -> Option<Aspect> {
    let signed = angular_diff(lon_b, lon_a);
    let sep = signed.abs();
    let rate = signed.signum() * (speed_b - speed_a);
    PTOLEMAIC.iter().find_map(|d| {
        let dev = sep - d.angle;
        (dev.abs() <= d.orb * orb_scale).then(|| Aspect {
            name: d.name,
            angle: d.angle,
            separation: sep,
            orb: dev.abs(),
            applying: dev * rate < 0.0,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eshana_time::Instant;

    fn oracle() -> serde_json::Value {
        serde_json::from_str(include_str!("../../../benchmarks/golden/swiss_oracle.json")).unwrap()
    }

    #[test]
    fn ayanamsha_matches_swiss_over_a_century() {
        for row in oracle()["instants"].as_array().unwrap() {
            let inst = Instant::parse_utc(row["utc"].as_str().unwrap()).unwrap();
            for m in Ayanamsha::ALL {
                // this file holds the original four; every model is checked at eight epochs in
                // tests/ayanamsha_oracle.rs
                let Some(want) = row["ayanamsha"][m.id()].as_f64() else {
                    continue;
                };
                let got = ayanamsha_deg(m, inst.t_tt());
                assert!(
                    (got - want).abs() < 0.0006,
                    "{} {}: got {got:.6} want {want:.6}",
                    row["utc"],
                    m.id()
                );
            }
        }
    }

    #[test]
    fn asc_mc_and_houses_match_swiss() {
        for row in oracle()["instants"].as_array().unwrap() {
            let inst = Instant::parse_utc(row["utc"].as_str().unwrap()).unwrap();
            let t = inst.t_tt();
            for (place, h) in row["houses"].as_object().unwrap() {
                let (lat, lon) = (h["lat"].as_f64().unwrap(), h["lon"].as_f64().unwrap());
                let lst = local_apparent_sidereal_deg(inst.gmst_deg(), lon, t);
                let (asc, mc) = asc_mc(lst, lat, t);
                for (k, got) in [("asc", asc), ("mc", mc)] {
                    let want = h["P"][k].as_f64().unwrap();
                    assert!(
                        angular_diff(got, want).abs() < 0.01,
                        "{} {place} {k}: got {got:.4} want {want:.4}",
                        row["utc"]
                    );
                }
                for (code, sys) in [
                    ("P", HouseSystem::Placidus),
                    ("E", HouseSystem::Equal),
                    ("W", HouseSystem::WholeSign),
                ] {
                    let want = h[code]["cusps"].as_array().unwrap();
                    let got = match houses(sys, lst, lat, t) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    for i in 0..12 {
                        let w = want[i].as_f64().unwrap();
                        assert!(
                            angular_diff(got[i], w).abs() < 0.02,
                            "{} {place} {code} cusp {}: got {:.4} want {w:.4}",
                            row["utc"],
                            i + 1,
                            got[i]
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn placidus_refuses_the_polar_circle() {
        let e = houses(HouseSystem::Placidus, 100.0, 70.0, 0.0).unwrap_err();
        assert_eq!(e.code(), "ESH-LATITUDE");
        assert!(houses(HouseSystem::WholeSign, 100.0, 70.0, 0.0).is_ok());
    }

    #[test]
    fn house_membership_wraps_at_zero() {
        let c = houses(HouseSystem::Equal, 10.0, 17.0, 0.0).unwrap();
        for lon in [0.0, 29.99, 120.5, 359.99] {
            let h = house_of(lon, &c);
            assert!((1..=12).contains(&h));
        }
        assert_eq!(house_of(c[0] + 0.001, &c), 1);
        assert_eq!(house_of(c[0] - 0.001, &c), 12);
    }

    #[test]
    fn aspects_and_applying() {
        // Mars at 10°, Venus at 72° moving toward it: a sextile 2° wide, applying when Venus is faster.
        let a = aspect_between(10.0, 0.5, 72.0, 0.0, 1.0).unwrap();
        assert_eq!(a.name, "sextile");
        assert!((a.orb - 2.0).abs() < 1e-9);
        assert!(a.applying);
        let s = aspect_between(10.0, 0.0, 72.0, 0.5, 1.0).unwrap();
        assert!(!s.applying);
        assert!(aspect_between(0.0, 0.0, 45.0, 0.0, 1.0).is_none());
    }

    #[test]
    fn aspects_across_zero_degrees() {
        let a = aspect_between(359.0, 1.0, 1.0, 0.0, 1.0).unwrap();
        assert_eq!(a.name, "conjunction");
        assert!((a.separation - 2.0).abs() < 1e-9);
    }

    #[test]
    fn sign_boundaries() {
        assert_eq!(sign_index(0.0), 0);
        assert_eq!(sign_index(29.999_999), 0);
        assert_eq!(sign_index(30.0), 1);
        assert_eq!(sign_index(359.999_999), 11);
        assert_eq!(sign_index(-0.000_001), 11);
    }
}
