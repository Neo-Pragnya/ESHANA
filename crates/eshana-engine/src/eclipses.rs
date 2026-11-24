//! Solar and lunar eclipses, found geometrically from the provider's own Sun and Moon (AST-06).
//!
//! Every new and full moon in the range is a candidate. Near each one, the time of greatest eclipse is the minimum of
//! the angular distance between the Moon and the Sun (solar) or the centre of the Earth's shadow (lunar), refined by
//! golden-section search to about a second. The geometry uses the Sun's and the Moon's horizontal parallaxes and
//! semidiameters at that moment:
//!
//! * lunar: penumbral and umbral shadow radii 1.02 × (π☾ + π☉ ± s☉) (the 2% enlargement for the atmosphere,
//!   Chauvenet / the *Explanatory Supplement*); magnitudes (R + s☾ − d) / (2 s☾); total when the umbral magnitude
//!   reaches 1, partial above 0, penumbral when only the penumbra is touched;
//! * solar: some part of the Earth sees an eclipse when d < π☾ − π☉ + s☾ + s☉; it is central when the shadow axis
//!   meets the Earth (γ = d / (π☾ − π☉) below 1, a spherical-Earth approximation of the canon's γ < 0.9972); a
//!   central eclipse is total when the Moon looks larger than the Sun even from the path's ends (about the geocentric
//!   distance), annular when it looks smaller even under the shadow axis (an Earth radius nearer the Moon), and hybrid
//!   in between.
//!
//! Times are those of greatest eclipse in the geocentric sense, which agree with the canon's to about a minute.

use crate::events::lon_at;
use eshana_core::{angular_diff, deg, rad, Body, Result};
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde_json::{json, Value};

const AU_KM: f64 = 149_597_870.7;
const EARTH_RADIUS_KM: f64 = 6378.14;
const MOON_RADIUS_KM: f64 = 1737.4;
const SUN_RADIUS_KM: f64 = 696_000.0;

/// Apparent ecliptic longitude, latitude (degrees) and distance (km).
type Place = (f64, f64, f64);

/// The Sun's and the Moon's [`Place`].
fn sun_moon(p: &dyn EphemerisProvider, jd_ut: f64) -> Result<(Place, Place)> {
    let inst = Instant::from_jd_utc(jd_ut);
    let (sl, sb, sd) = p.apparent(inst.jd_tt, Body::Sun)?;
    let (ml, mb, md) = p.apparent(inst.jd_tt, Body::Moon)?;
    Ok((
        (sl, sb, sd.unwrap_or(1.0) * AU_KM),
        (ml, mb, md.unwrap_or(0.002_57) * AU_KM),
    ))
}

/// Angular separation (degrees) of two ecliptic directions.
fn separation(l1: f64, b1: f64, l2: f64, b2: f64) -> f64 {
    let (l1, b1, l2, b2) = (rad(l1), rad(b1), rad(l2), rad(b2));
    deg((b1.sin() * b2.sin() + b1.cos() * b2.cos() * (l1 - l2).cos())
        .clamp(-1.0, 1.0)
        .acos())
}

/// The distance to minimise: Moon to Sun (solar) or Moon to the shadow centre opposite the Sun (lunar).
fn distance(p: &dyn EphemerisProvider, jd: f64, lunar: bool) -> Result<f64> {
    let ((sl, sb, _), (ml, mb, _)) = sun_moon(p, jd)?;
    Ok(if lunar {
        separation(ml, mb, sl + 180.0, -sb)
    } else {
        separation(ml, mb, sl, sb)
    })
}

/// Golden-section minimum of `distance` in [a, b].
fn minimum(p: &dyn EphemerisProvider, mut a: f64, mut b: f64, lunar: bool) -> Result<f64> {
    let g = (5f64.sqrt() - 1.0) / 2.0;
    let (mut c, mut d) = (b - g * (b - a), a + g * (b - a));
    let (mut fc, mut fd) = (distance(p, c, lunar)?, distance(p, d, lunar)?);
    while b - a > 1.0 / 86_400.0 {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - g * (b - a);
            fc = distance(p, c, lunar)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + g * (b - a);
            fd = distance(p, d, lunar)?;
        }
    }
    Ok((a + b) / 2.0)
}

/// Classify the eclipse (if any) at the syzygy near `jd`.
fn classify(p: &dyn EphemerisProvider, jd_syzygy: f64, lunar: bool) -> Result<Option<Value>> {
    let jd = minimum(p, jd_syzygy - 0.3, jd_syzygy + 0.3, lunar)?;
    let ((_, _, sd), (_, _, md)) = sun_moon(p, jd)?;
    let d = distance(p, jd, lunar)?;
    let (pi_m, pi_s) = (
        deg((EARTH_RADIUS_KM / md).asin()),
        deg((EARTH_RADIUS_KM / sd).asin()),
    );
    let (s_m, s_s) = (
        deg((MOON_RADIUS_KM / md).asin()),
        deg((SUN_RADIUS_KM / sd).asin()),
    );
    let stamp = |kind: &str, ty: &str, extra: Value| -> Value {
        let mut v = json!({"kind": kind, "type": ty, "max_utc": Instant::from_jd_utc(jd).utc, "max_jd_ut": jd,
                           "separation_deg": d, "moon_semidiameter_deg": s_m, "sun_semidiameter_deg": s_s});
        if let (Value::Object(m), Value::Object(e)) = (&mut v, extra) {
            m.extend(e);
        }
        v
    };
    if lunar {
        let (r_pen, r_umb) = (1.02 * (pi_m + pi_s + s_s), 1.02 * (pi_m + pi_s - s_s));
        let (pen, umb) = ((r_pen + s_m - d) / (2.0 * s_m), (r_umb + s_m - d) / (2.0 * s_m));
        if pen <= 0.0 {
            return Ok(None);
        }
        let ty = if umb >= 1.0 {
            "total"
        } else if umb > 0.0 {
            "partial"
        } else {
            "penumbral"
        };
        return Ok(Some(stamp(
            "lunar",
            ty,
            json!({"penumbral_magnitude": pen, "umbral_magnitude": umb}),
        )));
    }
    if d >= pi_m - pi_s + s_m + s_s {
        return Ok(None);
    }
    let gamma = d / (pi_m - pi_s);
    // the Moon's apparent size where the shadow axis meets the Earth (the observer is up to one Earth radius nearer
    // the Moon) and at the ends of the path (about the geocentric distance): total at both is total, annular at both
    // annular, total in the middle only is hybrid
    let ratio_edge = s_m / s_s;
    let near = md - EARTH_RADIUS_KM * (1.0 - gamma * gamma).max(0.0).sqrt();
    let ratio_mid = deg((MOON_RADIUS_KM / near).asin()) / s_s;
    let ty = if gamma >= 1.0 {
        "partial"
    } else if ratio_edge >= 1.0 {
        "total"
    } else if ratio_mid > 1.0 {
        "hybrid"
    } else {
        "annular"
    };
    Ok(Some(stamp(
        "solar",
        ty,
        json!({"gamma": gamma, "moon_sun_diameter_ratio": ratio_mid, "moon_sun_diameter_ratio_geocentric": ratio_edge}),
    )))
}

/// Every solar and/or lunar eclipse in `[jd_a, jd_b)`.
///
/// # Errors
/// Provider errors.
pub fn eclipses(
    p: &dyn EphemerisProvider,
    jd_a: f64,
    jd_b: f64,
    solar: bool,
    lunar: bool,
) -> Result<Vec<Value>> {
    let elong = |jd: f64| -> Result<f64> {
        Ok(angular_diff(
            lon_at(p, Body::Moon, jd, None)?,
            lon_at(p, Body::Sun, jd, None)?,
        ))
    };
    let mut out = vec![];
    let step = 1.0;
    let mut a = jd_a;
    let mut fa = elong(a)?;
    while a < jd_b {
        let b = (a + step).min(jd_b);
        let fb = elong(b)?;
        // new moon: the elongation passes 0 going up; full moon: it wraps from +180 to −180
        let new_moon = fa < 0.0 && fb >= 0.0;
        let full_moon = fa > 90.0 && fb < -90.0;
        if (new_moon && solar) || (full_moon && lunar) {
            let target = |jd: f64| -> Result<f64> {
                let e = elong(jd)?;
                Ok(if new_moon { e } else { angular_diff(e + 180.0, 0.0) })
            };
            let (mut lo, mut hi) = (a, b);
            for _ in 0..40 {
                let mid = 0.5 * (lo + hi);
                if target(mid)? < 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let syzygy = 0.5 * (lo + hi);
            // a quick latitude screen: no eclipse when the Moon is more than 1.6° from the ecliptic
            let (_, (_, mb, _)) = sun_moon(p, syzygy)?;
            if mb.abs() < 1.6 {
                if let Some(e) = classify(p, syzygy, full_moon)? {
                    out.push(e);
                }
            }
        }
        a = b;
        fa = fb;
    }
    Ok(out)
}
