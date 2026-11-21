//! Time series and events: an ephemeris table for animation and progressions, sign ingresses found by search
//! (including retrograde re-entries), and Sāḍe Sātī windows built from Saturn's ingresses.

use crate::astro;
use eshana_core::{norm360, Ayanamsha, Body, Result};
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde_json::{json, Value};

/// Longitude of a body at a UT Julian day: sidereal when `ayan` is given, tropical otherwise.
///
/// # Errors
/// Provider errors.
pub fn lon_at(p: &dyn EphemerisProvider, body: Body, jd_ut: f64, ayan: Option<Ayanamsha>) -> Result<f64> {
    let inst = Instant::from_jd_utc(jd_ut);
    let (lam, _, _) = p.apparent(inst.jd_tt, body)?;
    Ok(match ayan {
        Some(a) => astro::to_sidereal(lam, a, inst.t_tt()),
        None => norm360(lam),
    })
}

/// Longitudes of several bodies at `count` instants `step_days` apart.
///
/// # Errors
/// Provider errors.
pub fn series(
    p: &dyn EphemerisProvider,
    bodies: &[Body],
    jd_start: f64,
    step_days: f64,
    count: usize,
    ayan: Option<Ayanamsha>,
) -> Result<Value> {
    let mut rows = vec![];
    for k in 0..count {
        let jd = jd_start + step_days * k as f64;
        let mut lons = serde_json::Map::new();
        for b in bodies {
            lons.insert(b.id().into(), json!(lon_at(p, *b, jd, ayan)?));
        }
        rows.push(json!({"utc": Instant::from_jd_utc(jd).utc, "jd_ut": jd, "longitudes": lons}));
    }
    Ok(json!(rows))
}

/// Every sign ingress of a body between two UT Julian days, to about a minute.
///
/// # Errors
/// Provider errors.
pub fn ingresses(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_a: f64,
    jd_b: f64,
    ayan: Option<Ayanamsha>,
) -> Result<Vec<Value>> {
    let step = match body {
        Body::Moon => 0.25,
        Body::Sun | Body::Mercury | Body::Venus | Body::Mars => 0.5,
        _ => 2.0,
    };
    let sign = |jd: f64| -> Result<usize> { Ok(astro::sign_index(lon_at(p, body, jd, ayan)?)) };
    let mut out = vec![];
    let mut a = jd_a;
    let mut sa = sign(a)?;
    while a < jd_b {
        let b = (a + step).min(jd_b);
        let sb = sign(b)?;
        if sb != sa {
            let (mut lo, mut hi) = (a, b);
            for _ in 0..30 {
                let m = 0.5 * (lo + hi);
                if sign(m)? == sa {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let forward = (sb + 12 - sa) % 12 == 1;
            out.push(
                json!({"utc": Instant::from_jd_utc(hi).utc, "jd_ut": hi, "from_sign": sa, "to_sign": sb,
                            "direction": if forward { "direct" } else { "retrograde" }}),
            );
        }
        a = b;
        sa = sb;
    }
    Ok(out)
}

/// Sāḍe Sātī: the periods when Saturn transits the 12th, 1st and 2nd signs from the natal Moon, with phases.
///
/// # Errors
/// Provider errors.
pub fn sade_sati(
    p: &dyn EphemerisProvider,
    natal_moon_sign: usize,
    jd_a: f64,
    jd_b: f64,
    ayan: Ayanamsha,
) -> Result<Value> {
    let phase = |s: usize| -> Option<&'static str> {
        match (s + 12 - natal_moon_sign) % 12 {
            11 => Some("rising"),
            0 => Some("peak"),
            1 => Some("setting"),
            _ => None,
        }
    };
    let ing = ingresses(p, Body::Saturn, jd_a, jd_b, Some(ayan))?;
    let mut segments: Vec<(f64, f64, usize)> = vec![];
    let mut t = jd_a;
    let mut s = astro::sign_index(lon_at(p, Body::Saturn, jd_a, Some(ayan))?);
    for e in &ing {
        let at = e["jd_ut"].as_f64().unwrap_or(t);
        segments.push((t, at, s));
        t = at;
        s = e["to_sign"].as_u64().unwrap_or(0) as usize;
    }
    segments.push((t, jd_b, s));
    let mut windows: Vec<Value> = vec![];
    let mut current: Option<(f64, Vec<Value>)> = None;
    for (a, b, sign) in segments {
        match (phase(sign), current.as_mut()) {
            (Some(ph), Some((_, parts))) => parts.push(json!({"phase": ph, "sign": sign,
                "start": Instant::from_jd_utc(a).utc, "end": Instant::from_jd_utc(b).utc})),
            (Some(ph), None) => {
                current = Some((
                    a,
                    vec![json!({"phase": ph, "sign": sign,
                "start": Instant::from_jd_utc(a).utc, "end": Instant::from_jd_utc(b).utc})],
                ))
            }
            (None, Some(_)) => {
                let (start, parts) = current.take().expect("some");
                windows.push(
                    json!({"start": Instant::from_jd_utc(start).utc, "end": Instant::from_jd_utc(a).utc,
                                    "clipped_at_start": start <= jd_a, "phases": parts}),
                );
            }
            (None, None) => {}
        }
    }
    if let Some((start, parts)) = current {
        windows.push(
            json!({"start": Instant::from_jd_utc(start).utc, "end": Instant::from_jd_utc(jd_b).utc,
                            "clipped_at_start": start <= jd_a, "clipped_at_end": true, "phases": parts}),
        );
    }
    Ok(
        json!({"natal_moon_sign": natal_moon_sign, "range": [Instant::from_jd_utc(jd_a).utc, Instant::from_jd_utc(jd_b).utc],
              "windows": windows, "saturn_ingresses": ing}),
    )
}
