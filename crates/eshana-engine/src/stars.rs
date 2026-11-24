//! Anchor stars for the true (star-anchored) ayanāṃśas (AYA-02).
//!
//! Each star's position and proper motion are from the Hipparcos new reduction (van Leeuwen 2007, CDS I/311; read
//! from the ESA archive table `public.hipparcos_newreduction`), ICRS at epoch J1991.25. The star is moved by its
//! proper motion (linear; radial velocity is negligible over centuries at this precision), turned into the J2000
//! ecliptic, precessed to the mean ecliptic and equinox of date, and given the annual aberration and the Sun's
//! light deflection, so the apparent star sits exactly on its anchor longitude, the same convention as the
//! apparent planets it is compared with. Nutation is left out on both sides, as for the mean models.

use eshana_core::{deg, norm360, rad};
use eshana_ephemeris::frames::precess_ecliptic_from_j2000;

/// A star that anchors a true ayanāṃśa.
#[derive(Debug, Clone, Copy)]
pub struct AnchorStar {
    /// Hipparcos number.
    pub hip: u32,
    /// Usual name.
    pub name: &'static str,
    /// Right ascension and declination at J1991.25, degrees (ICRS).
    pub ra_deg: f64,
    /// Declination, degrees.
    pub dec_deg: f64,
    /// Proper motion in right ascension times cos(declination), milliarcseconds per Julian year.
    pub pm_ra_mas: f64,
    /// Proper motion in declination, milliarcseconds per Julian year.
    pub pm_dec_mas: f64,
    /// The sidereal longitude the star is defined to sit at, degrees.
    pub anchor_deg: f64,
}

/// Spica (α Vir, Citrā) at 180°.
pub const SPICA: AnchorStar = AnchorStar {
    hip: 65474,
    name: "Spica (α Vir)",
    ra_deg: 201.298_352_284_272_3,
    dec_deg: -11.161_244_938_592_992,
    pm_ra_mas: -42.35,
    pm_dec_mas: -30.67,
    anchor_deg: 180.0,
};
/// ζ Piscium (Revatī) at 359°50′.
pub const ZETA_PSC: AnchorStar = AnchorStar {
    hip: 5737,
    name: "ζ Psc (Revatī)",
    ra_deg: 18.432_508_418_248_02,
    dec_deg: 7.575_489_378_231_63,
    pm_ra_mas: 145.0,
    pm_dec_mas: -55.69,
    anchor_deg: 359.0 + 50.0 / 60.0,
};
/// δ Cancri (Puṣya) at 106°.
pub const DELTA_CNC: AnchorStar = AnchorStar {
    hip: 42911,
    name: "δ Cnc (Puṣya)",
    ra_deg: 131.171_291_908_619_08,
    dec_deg: 18.154_863_727_742_615,
    pm_ra_mas: -17.67,
    pm_dec_mas: -229.26,
    anchor_deg: 106.0,
};
/// λ Scorpii (Mūla) at 240°.
pub const LAMBDA_SCO: AnchorStar = AnchorStar {
    hip: 85927,
    name: "λ Sco (Mūla)",
    ra_deg: 263.402_193_181_996_55,
    dec_deg: -37.103_748_685_815_525,
    pm_ra_mas: -8.53,
    pm_dec_mas: -30.8,
    anchor_deg: 240.0,
};

/// JD (TT) of the Hipparcos catalogue epoch J1991.25.
const J1991_25: f64 = 2_448_349.062_5;
/// Obliquity of the J2000 ecliptic used for the ICRS → ecliptic rotation (IAU 2006, at J2000), degrees.
const EPS_J2000: f64 = 23.439_279_444;

/// The Sun's geometric longitude, mean equinox of date (Meeus ch. 25, low precision: 0.01°, plenty for aberration
/// and deflection).
fn sun_longitude(t: f64) -> f64 {
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    let m = rad(357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t);
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    norm360(l0 + c)
}

/// The star's apparent ecliptic longitude (mean equinox of date, no nutation) at TT centuries `t` since J2000.
#[must_use]
pub fn apparent_longitude(s: &AnchorStar, t: f64) -> f64 {
    let years = (2_451_545.0 + t * 36_525.0 - J1991_25) / 365.25;
    let dec = s.dec_deg + s.pm_dec_mas / 3.6e6 * years;
    let ra = s.ra_deg + s.pm_ra_mas / 3.6e6 * years / rad(s.dec_deg).cos();
    let (ra, dec, e) = (rad(ra), rad(dec), rad(EPS_J2000));
    let (x, y, z) = (dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin());
    let (y, z) = (y * e.cos() + z * e.sin(), -y * e.sin() + z * e.cos());
    let (lon, lat) = (norm360(deg(y.atan2(x))), deg(z.asin()));
    let (lon, lat) = precess_ecliptic_from_j2000(lon, lat, t);
    // annual aberration (Meeus 23.2, with the e-terms)
    let sun = sun_longitude(t);
    let kappa = 20.495_52 / 3600.0;
    let ecc = 0.016_708_634 - 0.000_042_037 * t;
    let peri = 102.937_35 + 1.719_46 * t;
    let d_ab = (-kappa * rad(sun - lon).cos() + ecc * kappa * rad(peri - lon).cos()) / rad(lat).cos();
    // the Sun's light deflection, 0.00407″·(1 + cos E)/sin E away from the Sun
    let (l, b, sl) = (rad(lon), rad(lat), rad(sun));
    let u = [b.cos() * l.cos(), b.cos() * l.sin(), b.sin()];
    let sv = [sl.cos(), sl.sin(), 0.0];
    let ce = u[0] * sv[0] + u[1] * sv[1];
    let se = (1.0 - ce * ce).max(1e-12).sqrt();
    let k = rad(0.004_07 / 3600.0) * (1.0 + ce) / se / se;
    let v = [u[0] + k * (u[0] - ce * sv[0]), u[1] + k * (u[1] - ce * sv[1])];
    let d_defl = norm360(deg(v[1].atan2(v[0])) - lon + 180.0) - 180.0;
    norm360(lon + d_ab + d_defl)
}

/// The true ayanāṃśa a star defines: its apparent longitude of date minus its anchor.
#[must_use]
pub fn true_ayanamsha(s: &AnchorStar, t: f64) -> f64 {
    norm360(apparent_longitude(s, t) - s.anchor_deg + 180.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spica_sits_near_its_classical_value() {
        // True Citrā at J2000 is close to Lahiri (23.857°): the two differ by a few arc-minutes at most
        let a = true_ayanamsha(&SPICA, 0.0);
        assert!((a - 23.84).abs() < 0.02, "{a}");
    }
}
