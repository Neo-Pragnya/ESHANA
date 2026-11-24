//! The `analytic` provider: no data files, no licence entanglement.
//!
//! - Planets: JPL's published approximate Keplerian elements (E. M. Standish, "Keplerian Elements for
//!   Approximate Positions of the Major Planets", Table 1, valid 1800–2050), with light-time and aberration.
//! - Moon: the principal periodic terms of Meeus, *Astronomical Algorithms* ch. 47.
//! - Nodes: Meeus mean node; the true node is the osculating node of the Moon's own orbit.
//!
//! This is a **low-precision** provider: good for charts at degree and arc-minute resolution, not for
//! sub-arc-second work. The measured error against JPL DE440s is recorded in `benchmarks/golden` and in the
//! provider's accuracy statement. Use the `spk` provider when you need more.

use crate::frames::{
    apparent_of_date, lon_lat, mean_obliquity_deg, nutation_deg, precess_ecliptic_from_j2000, AU_KM,
    C_AU_PER_DAY,
};
use crate::{EphemerisProvider, ProviderInfo};
use eshana_core::{deg, norm360, rad, Body, EshanaError, Result};
use eshana_time::JD_J2000;

/// Elements and their rates per Julian century: a (au), e, I, L, long. perihelion, long. node (degrees).
struct Elements {
    a: [f64; 2],
    e: [f64; 2],
    i: [f64; 2],
    l: [f64; 2],
    w: [f64; 2],
    n: [f64; 2],
}

const MERCURY: Elements = Elements {
    a: [0.387_099_27, 0.000_000_37],
    e: [0.205_635_93, 0.000_019_06],
    i: [7.004_979_02, -0.005_947_49],
    l: [252.250_323_50, 149_472.674_111_75],
    w: [77.457_796_28, 0.160_476_89],
    n: [48.330_765_93, -0.125_340_81],
};
const VENUS: Elements = Elements {
    a: [0.723_335_66, 0.000_003_90],
    e: [0.006_776_72, -0.000_041_07],
    i: [3.394_676_05, -0.000_788_90],
    l: [181.979_099_50, 58_517.815_387_29],
    w: [131.602_467_18, 0.002_683_29],
    n: [76.679_842_55, -0.277_694_18],
};
const MARS: Elements = Elements {
    a: [1.523_710_34, 0.000_018_47],
    e: [0.093_394_10, 0.000_078_82],
    i: [1.849_691_42, -0.008_131_31],
    l: [-4.553_432_05, 19_140.302_684_99],
    w: [-23.943_629_59, 0.444_410_88],
    n: [49.559_538_91, -0.292_573_43],
};
const JUPITER: Elements = Elements {
    a: [5.202_887_00, -0.000_116_07],
    e: [0.048_386_24, -0.000_132_53],
    i: [1.304_396_95, -0.001_837_14],
    l: [34.396_440_51, 3_034.746_127_75],
    w: [14.728_479_83, 0.212_526_68],
    n: [100.473_909_09, 0.204_691_06],
};
const SATURN: Elements = Elements {
    a: [9.536_675_94, -0.001_250_60],
    e: [0.053_861_79, -0.000_509_91],
    i: [2.485_991_87, 0.001_936_09],
    l: [49.954_244_23, 1_222.493_622_01],
    w: [92.598_878_31, -0.418_972_16],
    n: [113.662_424_48, -0.288_677_94],
};
const URANUS: Elements = Elements {
    a: [19.189_164_64, -0.001_961_76],
    e: [0.047_257_44, -0.000_043_97],
    i: [0.772_637_83, -0.002_429_39],
    l: [313.238_104_51, 428.482_027_85],
    w: [170.954_276_30, 0.408_052_81],
    n: [74.016_925_03, 0.042_405_89],
};
const NEPTUNE: Elements = Elements {
    a: [30.069_922_76, 0.000_262_91],
    e: [0.008_590_48, 0.000_051_05],
    i: [1.770_043_47, 0.000_353_72],
    l: [-55.120_029_69, 218.459_453_25],
    w: [44.964_762_27, -0.322_414_64],
    n: [131.784_225_74, -0.005_086_64],
};
const PLUTO: Elements = Elements {
    a: [39.482_116_75, -0.000_315_96],
    e: [0.248_827_30, 0.000_051_70],
    i: [17.140_012_06, 0.000_048_18],
    l: [238.929_038_33, 145.207_805_15],
    w: [224.068_916_29, -0.040_629_42],
    n: [110.303_936_84, -0.011_834_82],
};

fn elements(body: Body) -> Option<&'static Elements> {
    Some(match body {
        Body::Mercury => &MERCURY,
        Body::Venus => &VENUS,
        Body::Mars => &MARS,
        Body::Jupiter => &JUPITER,
        Body::Saturn => &SATURN,
        Body::Uranus => &URANUS,
        Body::Neptune => &NEPTUNE,
        Body::Pluto => &PLUTO,
        _ => return None,
    })
}

/// Heliocentric position in the ecliptic and equinox of J2000, in AU.
fn heliocentric(el: &Elements, t: f64) -> [f64; 3] {
    let f = |p: [f64; 2]| p[0] + p[1] * t;
    let (a, e, i, l, w, n) = (f(el.a), f(el.e), rad(f(el.i)), f(el.l), f(el.w), f(el.n));
    let m = rad(eshana_core::norm180(l - w));
    // Kepler's equation by Newton iteration.
    let mut ea = m + e * m.sin();
    for _ in 0..30 {
        let d = (ea - e * ea.sin() - m) / (1.0 - e * ea.cos());
        ea -= d;
        if d.abs() < 1e-13 {
            break;
        }
    }
    let xp = a * (ea.cos() - e);
    let yp = a * (1.0 - e * e).sqrt() * ea.sin();
    let (om, nn) = (rad(w - n), rad(n));
    let (co, so, cn, sn, ci, si) = (om.cos(), om.sin(), nn.cos(), nn.sin(), i.cos(), i.sin());
    [
        (co * cn - so * sn * ci) * xp + (-so * cn - co * sn * ci) * yp,
        (co * sn + so * cn * ci) * xp + (-so * sn + co * cn * ci) * yp,
        (so * si) * xp + (co * si) * yp,
    ]
}

/// Moon arguments (Meeus 47): L', D, M, M', F in degrees.
fn moon_args(t: f64) -> (f64, f64, f64, f64, f64) {
    let lp = 218.316_447_7 + 481_267.881_234_21 * t - 0.001_578_6 * t * t + t.powi(3) / 538_841.0
        - t.powi(4) / 65_194_000.0;
    let d = 297.850_192_1 + 445_267.111_403_4 * t - 0.001_881_9 * t * t + t.powi(3) / 545_868.0
        - t.powi(4) / 113_065_000.0;
    let m = 357.529_109_2 + 35_999.050_290_9 * t - 0.000_153_6 * t * t + t.powi(3) / 24_490_000.0;
    let mp = 134.963_396_4 + 477_198.867_505_5 * t + 0.008_741_4 * t * t + t.powi(3) / 69_699.0
        - t.powi(4) / 14_712_000.0;
    let f = 93.272_095_0 + 483_202.017_523_3 * t - 0.003_653_9 * t * t - t.powi(3) / 3_526_000.0
        + t.powi(4) / 863_310_000.0;
    (lp, d, m, mp, f)
}

// (D, M, M', F, sine coefficient in 1e-6 degree, cosine coefficient in 1e-3 km); Meeus Table 47.A.
#[rustfmt::skip]
const MOON_LR: &[(i8, i8, i8, i8, f64, f64)] = &[
    (0,0,1,0, 6288774.0, -20905355.0), (2,0,-1,0, 1274027.0, -3699111.0), (2,0,0,0, 658314.0, -2955968.0),
    (0,0,2,0, 213618.0, -569925.0), (0,1,0,0, -185116.0, 48888.0), (0,0,0,2, -114332.0, -3149.0),
    (2,0,-2,0, 58793.0, 246158.0), (2,-1,-1,0, 57066.0, -152138.0), (2,0,1,0, 53322.0, -170733.0),
    (2,-1,0,0, 45758.0, -204586.0), (0,1,-1,0, -40923.0, -129620.0), (1,0,0,0, -34720.0, 108743.0),
    (0,1,1,0, -30383.0, 104755.0), (2,0,0,-2, 15327.0, 10321.0), (0,0,1,2, -12528.0, 0.0),
    (0,0,1,-2, 10980.0, 79661.0), (4,0,-1,0, 10675.0, -34782.0), (0,0,3,0, 10034.0, -23210.0),
    (4,0,-2,0, 8548.0, -21636.0), (2,1,-1,0, -7888.0, 24208.0), (2,1,0,0, -6766.0, 30824.0),
    (1,0,-1,0, -5163.0, -8379.0), (1,1,0,0, 4987.0, -16675.0), (2,-1,1,0, 4036.0, -12831.0),
    (2,0,2,0, 3994.0, -10445.0), (4,0,0,0, 3861.0, -11650.0), (2,0,-3,0, 3665.0, 14403.0),
    (0,1,-2,0, -2689.0, -7003.0), (2,0,-1,2, -2602.0, 0.0), (2,-1,-2,0, 2390.0, 10056.0),
    (1,0,1,0, -2348.0, 6322.0), (2,-2,0,0, 2236.0, -9884.0), (0,1,2,0, -2120.0, 5751.0),
    (0,2,0,0, -2069.0, 0.0), (2,-2,-1,0, 2048.0, -4950.0), (2,0,1,-2, -1773.0, 4130.0),
    (2,0,0,2, -1595.0, 0.0), (4,-1,-1,0, 1215.0, -3958.0), (0,0,2,2, -1110.0, 0.0),
    (3,0,-1,0, -892.0, 3258.0), (2,1,1,0, -810.0, 2616.0), (4,-1,-2,0, 759.0, -1897.0),
    (0,2,-1,0, -713.0, -2117.0), (2,2,-1,0, -700.0, 2354.0), (2,1,-2,0, 691.0, 0.0),
    (2,-1,0,-2, 596.0, 0.0), (4,0,1,0, 549.0, -1423.0), (0,0,4,0, 537.0, -1117.0),
    (4,-1,0,0, 520.0, -1571.0), (1,0,-2,0, -487.0, -1739.0), (2,1,0,-2, -399.0, 0.0),
    (0,0,2,-2, -381.0, -4421.0), (1,1,1,0, 351.0, 0.0), (3,0,-2,0, -340.0, 0.0),
    (4,0,-3,0, 330.0, 0.0), (2,-1,2,0, 327.0, 0.0), (0,2,1,0, -323.0, 1165.0),
    (1,1,-1,0, 299.0, 0.0), (2,0,3,0, 294.0, 0.0), (2,0,-1,-2, 0.0, 8752.0),
];

// (D, M, M', F, sine coefficient in 1e-6 degree); Meeus Table 47.B.
#[rustfmt::skip]
const MOON_B: &[(i8, i8, i8, i8, f64)] = &[
    (0,0,0,1, 5128122.0), (0,0,1,1, 280602.0), (0,0,1,-1, 277693.0), (2,0,0,-1, 173237.0),
    (2,0,-1,1, 55413.0), (2,0,-1,-1, 46271.0), (2,0,0,1, 32573.0), (0,0,2,1, 17198.0),
    (2,0,1,-1, 9266.0), (0,0,2,-1, 8822.0), (2,-1,0,-1, 8216.0), (2,0,-2,-1, 4324.0),
    (2,0,1,1, 4200.0), (2,1,0,-1, -3359.0), (2,-1,-1,1, 2463.0), (2,-1,0,1, 2211.0),
    (2,-1,-1,-1, 2065.0), (0,1,-1,-1, -1870.0), (4,0,-1,-1, 1828.0), (0,1,0,1, -1794.0),
    (0,0,0,3, -1749.0), (0,1,-1,1, -1565.0), (1,0,0,1, -1491.0), (0,1,1,1, -1475.0),
    (0,1,1,-1, -1410.0), (0,1,0,-1, -1344.0), (1,0,0,-1, -1335.0), (0,0,3,1, 1107.0),
    (4,0,0,-1, 1021.0), (4,0,-1,1, 833.0),
];

/// Geocentric Moon: (apparent longitude of date including nutation, latitude, distance km).
fn moon(t: f64) -> (f64, f64, f64) {
    let (lp, d, m, mp, f) = moon_args(t);
    let e = 1.0 - 0.002_516 * t - 0.000_007_4 * t * t;
    let ef = |mm: i8| match mm.abs() {
        1 => e,
        2 => e * e,
        _ => 1.0,
    };
    let (mut sl, mut sr, mut sb) = (0.0, 0.0, 0.0);
    for &(cd, cm, cmp, cf, l, r) in MOON_LR {
        let arg = rad(f64::from(cd) * d + f64::from(cm) * m + f64::from(cmp) * mp + f64::from(cf) * f);
        sl += l * ef(cm) * arg.sin();
        sr += r * ef(cm) * arg.cos();
    }
    for &(cd, cm, cmp, cf, b) in MOON_B {
        let arg = rad(f64::from(cd) * d + f64::from(cm) * m + f64::from(cmp) * mp + f64::from(cf) * f);
        sb += b * ef(cm) * arg.sin();
    }
    let a1 = rad(119.75 + 131.849 * t);
    let a2 = rad(53.09 + 479_264.290 * t);
    let a3 = rad(313.45 + 481_266.484 * t);
    sl += 3958.0 * a1.sin() + 1962.0 * rad(lp - f).sin() + 318.0 * a2.sin();
    sb += -2235.0 * rad(lp).sin()
        + 382.0 * a3.sin()
        + 175.0 * rad(119.75 + 131.849 * t - f).sin()
        + 175.0 * rad(119.75 + 131.849 * t + f).sin()
        + 127.0 * rad(lp - mp).sin()
        - 115.0 * rad(lp + mp).sin();
    let lon = norm360(lp + sl / 1e6 + nutation_deg(t).0);
    (lon, sb / 1e6, 385_000.56 + sr / 1000.0)
}

/// Mean ascending node of the lunar orbit, degrees, mean equinox of date.
fn mean_node(t: f64) -> f64 {
    norm360(
        125.044_547_9 - 1_934.136_289_1 * t + 0.002_075_4 * t * t + t.powi(3) / 467_441.0
            - t.powi(4) / 60_616_000.0,
    )
}

/// Apparent (true-equinox) longitude of the mean node. A polynomial in time, valid far beyond the analytic
/// provider's own planetary range, so the `spk` provider uses it directly.
#[must_use]
pub fn mean_node_apparent(jd_tt: f64) -> f64 {
    let t = (jd_tt - JD_J2000) / 36_525.0;
    norm360(mean_node(t) + nutation_deg(t).0)
}

/// Osculating ascending node of the Moon's geocentric orbit, from three apparent positions around `jd_tt`.
///
/// `moon_at(jd)` returns (longitude°, latitude°, distance) in the ecliptic of date. The node is where the
/// instantaneous orbital plane (from r × v) crosses the ecliptic going north. This is what the "true node" means
/// in the Swiss Ephemeris and in most software.
pub fn osculating_node(jd_tt: f64, moon_at: &dyn Fn(f64) -> Result<(f64, f64, f64)>) -> Result<f64> {
    let h = 0.05;
    let vec = |jd: f64| -> Result<[f64; 3]> {
        let (lon, lat, r) = moon_at(jd)?;
        let (lo, la) = (rad(lon), rad(lat));
        Ok([r * la.cos() * lo.cos(), r * la.cos() * lo.sin(), r * la.sin()])
    };
    let (a, b, c) = (vec(jd_tt - h)?, vec(jd_tt)?, vec(jd_tt + h)?);
    let v = [
        (c[0] - a[0]) / (2.0 * h),
        (c[1] - a[1]) / (2.0 * h),
        (c[2] - a[2]) / (2.0 * h),
    ];
    let hx = b[1] * v[2] - b[2] * v[1];
    let hy = b[2] * v[0] - b[0] * v[2];
    Ok(norm360(deg(hx.atan2(-hy))))
}

/// Heliocentric rectangular position (AU) from a VSOP87B series, dynamical ecliptic and equinox J2000.
fn vsop(planet: &crate::vsop87::Planet, jd_tt: f64) -> [f64; 3] {
    let tau = (jd_tt - JD_J2000) / 365_250.0;
    let mut lbr = [0.0; 3];
    for (v, powers) in planet.iter().enumerate() {
        let mut tk = 1.0;
        for series in powers {
            let sum: f64 = series.iter().map(|(a, b, c)| a * (b + c * tau).cos()).sum();
            lbr[v] += sum * tk;
            tk *= tau;
        }
    }
    let (l, b, r) = (lbr[0], lbr[1], lbr[2]);
    [r * b.cos() * l.cos(), r * b.cos() * l.sin(), r * b.sin()]
}

/// The VSOP87 series of a planet (Pluto has none: it keeps its Keplerian elements).
fn vsop_of(body: Body) -> Option<&'static crate::vsop87::Planet> {
    use crate::vsop87 as v;
    Some(match body {
        Body::Mercury => &v::MERCURY,
        Body::Venus => &v::VENUS,
        Body::Mars => &v::MARS,
        Body::Jupiter => &v::JUPITER,
        Body::Saturn => &v::SATURN,
        Body::Uranus => &v::URANUS,
        Body::Neptune => &v::NEPTUNE,
        _ => return None,
    })
}

/// Earth's heliocentric position and velocity (AU, AU/day), ecliptic J2000, from VSOP87B (the Earth's centre).
fn earth_state(jd_tt: f64) -> ([f64; 3], [f64; 3]) {
    let pos = |jd: f64| -> [f64; 3] { vsop(&crate::vsop87::EARTH, jd) };
    let h = 0.05;
    let (a, b) = (pos(jd_tt - h), pos(jd_tt + h));
    (
        pos(jd_tt),
        [
            (b[0] - a[0]) / (2.0 * h),
            (b[1] - a[1]) / (2.0 * h),
            (b[2] - a[2]) / (2.0 * h),
        ],
    )
}

/// The analytic provider.
#[derive(Debug, Clone, Default)]
pub struct AnalyticProvider;

impl AnalyticProvider {
    /// Create the provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Provider version, bumped when any series or constant changes.
    pub const VERSION: &'static str = "0.2.0";
}

impl EphemerisProvider for AnalyticProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: "analytic".into(),
            version: Self::VERSION.into(),
            data: "VSOP87B (IMCCE, truncated at 1e-7) for the Sun and Mercury to Neptune; JPL approximate Keplerian \
                   elements (Standish, Table 1) for Pluto; Meeus ch. 47 lunar series; no data files"
                .into(),
            accuracy: "Arc-second level for the Sun and planets (VSOP87), about 10 arc-seconds for the Moon; Pluto \
                       arc-minutes. Measured against JPL DE440s in benchmarks/golden/accuracy.md."
                .into(),
            jd_min: 2_378_496.5, // 1800-01-01
            jd_max: 2_469_807.5, // 2050-01-01 (Standish validity); later dates are accepted up to 2100 with larger error
        }
    }

    fn supports(&self, body: Body) -> bool {
        // the optional bodies (asteroids, Chiron) need a kernel
        !Body::OPTIONAL.contains(&body)
    }

    fn apparent(&self, jd_tt: f64, body: Body) -> Result<(f64, f64, Option<f64>)> {
        let info = self.info();
        // Allow a 50-year grace beyond the Standish validity window; accuracy there is lower and is reported.
        if jd_tt < info.jd_min || jd_tt > info.jd_max + 18_262.5 {
            return Err(EshanaError::OutOfRange(format!(
                "JD {jd_tt:.2} is outside the analytic provider's range (1800-2100)"
            )));
        }
        let t = (jd_tt - JD_J2000) / 36_525.0;
        match body {
            Body::Moon => {
                let (lon, lat, km) = moon(t);
                Ok((lon, lat, Some(km / AU_KM)))
            }
            Body::MeanNode => Ok((mean_node_apparent(jd_tt), 0.0, None)),
            Body::TrueNode => {
                let n = osculating_node(jd_tt, &|jd| {
                    let (lon, lat, km) = moon((jd - JD_J2000) / 36_525.0);
                    Ok((lon, lat, km))
                })?;
                Ok((n, 0.0, None))
            }
            Body::Sun => {
                let (e, v) = earth_state(jd_tt);
                // Geocentric Sun is minus the Earth's heliocentric position; the Sun is the origin so it does not move.
                let astro = [-e[0], -e[1], -e[2]];
                let (lon, lat) = apparent_of_date(astro, v, t);
                Ok((lon, lat, Some(crate::frames::norm(e))))
            }
            planet => {
                let el = elements(planet).ok_or_else(|| EshanaError::BodyUnsupported(planet.id().into()))?;
                let (e, v) = earth_state(jd_tt);
                // Light-time: the planet is seen where it was tau days ago.
                let mut tau = 0.0;
                let mut g = [0.0; 3];
                for _ in 0..4 {
                    let tp = (jd_tt - tau - JD_J2000) / 36_525.0;
                    let h = match vsop_of(planet) {
                        Some(series) => vsop(series, jd_tt - tau),
                        None => heliocentric(el, tp),
                    };
                    g = [h[0] - e[0], h[1] - e[1], h[2] - e[2]];
                    tau = crate::frames::norm(g) / C_AU_PER_DAY;
                }
                let (lon, lat) = apparent_of_date(g, v, t);
                Ok((lon, lat, Some(crate::frames::norm(g))))
            }
        }
    }
}

// Keep imports that are only used by sibling code reachable in unit tests.
#[allow(dead_code)]
fn _use(_: f64) {
    let _ = (
        mean_obliquity_deg(0.0),
        precess_ecliptic_from_j2000(0.0, 0.0, 0.0),
        lon_lat([1.0, 0.0, 0.0]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kepler_solves_for_high_eccentricity() {
        // Pluto at J2000 sits at about 29.8 AU from the Sun.
        let p = heliocentric(&PLUTO, 0.0);
        let r = crate::frames::norm(p);
        assert!((r - 30.1).abs() < 1.5, "r = {r}");
    }

    #[test]
    fn sun_at_j2000_is_in_capricorn() {
        let p = AnalyticProvider::new()
            .apparent(2_451_545.0 + 64.184 / 86400.0, Body::Sun)
            .unwrap();
        assert!((p.0 - 280.369).abs() < 0.02, "sun lon {}", p.0);
    }

    #[test]
    fn mean_node_meeus_example() {
        // Meeus example 47.b: 1992 April 12 0h TD (JDE 2448724.5) -> mean node 274.396...? published 274.40 (approx.)
        let t = (2_448_724.5 - JD_J2000) / 36_525.0;
        assert!((mean_node(t) - 274.4).abs() < 0.1, "{}", mean_node(t));
    }

    #[test]
    fn moon_meeus_example() {
        // Meeus example 47.a: 1992 April 12 0h TD: lambda = 133.162655, beta = -3.229126, distance 368409.7 km
        // (geometric, mean equinox of date; ours adds nutation of about +0.0044 deg).
        let t = (2_448_724.5 - JD_J2000) / 36_525.0;
        let (lon, lat, km) = moon(t);
        assert!((lon - 133.162_655 - nutation_deg(t).0).abs() < 0.01, "lon {lon}");
        assert!((lat + 3.229_126).abs() < 0.01, "lat {lat}");
        assert!((km - 368_409.7).abs() < 100.0, "km {km}");
    }

    #[test]
    fn out_of_range_is_an_error() {
        let e = AnalyticProvider::new()
            .apparent(1_000_000.0, Body::Sun)
            .unwrap_err();
        assert_eq!(e.code(), "ESH-RANGE");
    }
}
