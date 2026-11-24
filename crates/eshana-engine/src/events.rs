//! Time series and events: an ephemeris table for animation and progressions, sign ingresses found by search
//! (including retrograde re-entries), and Sāḍe Sātī windows built from Saturn's ingresses.

use crate::{astro, search};
use eshana_core::{angular_diff, norm360, Ayanamsha, Body, Result};
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
            // +1 while still in the old sign: the shared bisection, 30 halvings (as before, to well under a second)
            let mut in_old = |jd: f64| -> Result<f64> { Ok(if sign(jd)? == sa { 1.0 } else { -1.0 }) };
            let hi = search::bisect(&mut in_old, a, b, 0.0, Some(30))?;
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

/// Longitudinal speed in degrees per day, in the same frame as [`lon_at`] (a symmetric difference over ±0.01 day).
///
/// The frame matters at a station: sidereal speed is tropical speed less the precession rate, which moves a slow
/// planet's station by up to about an hour.
///
/// # Errors
/// Provider errors.
pub fn speed_at(p: &dyn EphemerisProvider, body: Body, jd_ut: f64, ayan: Option<Ayanamsha>) -> Result<f64> {
    const H: f64 = 0.01;
    Ok(angular_diff(
        lon_at(p, body, jd_ut + H, ayan)?,
        lon_at(p, body, jd_ut - H, ayan)?,
    ) / (2.0 * H))
}

/// Sampling step for station search: shorter than the shortest time between two stations of the body.
fn station_step(body: Body) -> f64 {
    match body {
        Body::Mercury | Body::Venus | Body::TrueNode => 1.0,
        Body::Mars => 2.0,
        _ => 4.0,
    }
}

/// Sampling step for crossing search: a body moves well under 90° in one step.
fn crossing_step(body: Body) -> f64 {
    match body {
        Body::Moon => 0.25,
        Body::Sun | Body::Mercury | Body::Venus | Body::Mars => 0.5,
        _ => 2.0,
    }
}

/// Bodies whose longitude never reverses (the mean node always moves backwards), so they have no stations.
#[must_use]
pub fn never_stations(body: Body) -> bool {
    matches!(body, Body::Sun | Body::Moon | Body::MeanNode)
}

/// Stations of a body in `[jd_a, jd_b)` and the retrograde intervals they bound.
///
/// A station is a root of longitudinal speed: `retrograde` when speed turns negative, `direct` when it turns
/// positive. An interval already retrograde at `jd_a` or still retrograde at `jd_b` is clipped there and says so.
///
/// # Errors
/// Provider errors.
pub fn stations(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_a: f64,
    jd_b: f64,
    ayan: Option<Ayanamsha>,
    tol_days: f64,
) -> Result<Value> {
    if never_stations(body) {
        return Ok(json!({"stations": [], "retrograde_intervals": []}));
    }
    let mut speed = |jd: f64| speed_at(p, body, jd, ayan);
    let found = search::find_roots(
        &mut speed,
        jd_a,
        jd_b,
        station_step(body),
        tol_days,
        0.0,
        f64::INFINITY,
    )?;
    let mut rows = vec![];
    for r in &found.roots {
        rows.push(json!({
            "utc": Instant::from_jd_utc(r.jd).utc, "jd_ut": r.jd,
            "station": if r.rising { "direct" } else { "retrograde" },
            "longitude_deg": lon_at(p, body, r.jd, ayan)?,
            "residual_deg_per_day": r.residual,
        }));
    }
    let mut intervals = vec![];
    let mut open: Option<(f64, bool)> = if speed_at(p, body, jd_a, ayan)? < 0.0 {
        Some((jd_a, true))
    } else {
        None
    };
    for r in &found.roots {
        if r.rising {
            if let Some((start, clipped)) = open.take() {
                intervals.push(interval(start, r.jd, clipped, false));
            }
        } else {
            open = Some((r.jd, false));
        }
    }
    if let Some((start, clipped)) = open {
        intervals.push(interval(start, jd_b, clipped, true));
    }
    Ok(json!({"stations": rows, "retrograde_intervals": intervals}))
}

fn interval(start: f64, end: f64, clipped_start: bool, clipped_end: bool) -> Value {
    json!({
        "start_utc": Instant::from_jd_utc(start).utc, "end_utc": Instant::from_jd_utc(end).utc,
        "start_jd_ut": start, "end_jd_ut": end, "days": end - start,
        "clipped_start": clipped_start, "clipped_end": clipped_end,
    })
}

/// When a body reaches `point_deg` plus each aspect angle, in `[jd_a, jd_b)`.
///
/// Every aspect other than 0° and 180° has two points (`point ± angle`). A crossing says whether the body was moving
/// direct or retrograde; a retrograde loop gives three. A touch is a closest approach within `touch_orb_deg` that
/// did not cross (the body stationed short of the point). At most `max_results` events are returned and
/// `truncated` says when there were more.
///
/// # Errors
/// Provider errors.
#[allow(clippy::too_many_arguments)]
pub fn crossings(
    p: &dyn EphemerisProvider,
    body: Body,
    point_deg: f64,
    aspects: &[f64],
    jd_a: f64,
    jd_b: f64,
    ayan: Option<Ayanamsha>,
    tol_days: f64,
    touch_orb_deg: f64,
    max_results: usize,
) -> Result<Value> {
    let mut events = vec![];
    for &asp in aspects {
        let mut targets = vec![norm360(point_deg + asp)];
        let a = norm360(asp);
        if a > 1e-9 && (a - 180.0).abs() > 1e-9 {
            targets.push(norm360(point_deg - asp));
        }
        for target in targets {
            let mut sep = |jd: f64| -> Result<f64> { Ok(angular_diff(lon_at(p, body, jd, ayan)?, target)) };
            let found = search::find_roots(
                &mut sep,
                jd_a,
                jd_b,
                crossing_step(body),
                tol_days,
                touch_orb_deg,
                90.0,
            )?;
            for r in found.roots {
                events.push(json!({
                    "kind": "crossing", "utc": Instant::from_jd_utc(r.jd).utc, "jd_ut": r.jd,
                    "aspect_deg": asp, "target_deg": target,
                    "direction": if r.rising { "direct" } else { "retrograde" },
                    "residual_deg": r.residual,
                }));
            }
            for (jd, gap) in found.touches {
                events.push(json!({
                    "kind": "touch", "utc": Instant::from_jd_utc(jd).utc, "jd_ut": jd,
                    "aspect_deg": asp, "target_deg": target, "separation_deg": gap,
                }));
            }
        }
    }
    events.sort_by(|x, y| {
        x["jd_ut"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&y["jd_ut"].as_f64().unwrap_or(0.0))
    });
    let truncated = events.len() > max_results;
    events.truncate(max_results);
    Ok(json!({"events": events, "truncated": truncated}))
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

/// Intervals in `[jd_a, jd_b)` when a graha is combust: within the vedic-core pack's orb of the Sun, using the smaller
/// retrograde orb for Mercury and Venus while they are retrograde. Found by half-day steps (a tenth of a day for the
/// Moon) and bisection to about a second; an interval open at either end is clipped and says so.
///
/// # Errors
/// Provider errors.
pub fn combustion(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_a: f64,
    jd_b: f64,
    ayan: Option<Ayanamsha>,
) -> Result<Vec<Value>> {
    let id = body.id();
    let state = |jd: f64| -> Result<bool> {
        let (l, s) = (lon_at(p, body, jd, ayan)?, lon_at(p, Body::Sun, jd, ayan)?);
        let retro = speed_at(p, body, jd, ayan)? < 0.0;
        Ok(crate::jyotish::combust(id, l, s, retro).unwrap_or(false))
    };
    if crate::jyotish::combust(id, 0.0, 0.0, false).is_none() {
        return Ok(vec![]); // no orb in the pack (the Sun itself, the nodes, the outer planets)
    }
    let step = if body == Body::Moon { 0.1 } else { 0.5 };
    let mut out = vec![];
    let (mut a, mut sa) = (jd_a, state(jd_a)?);
    let mut open = sa.then_some((jd_a, true));
    while a < jd_b {
        let b = (a + step).min(jd_b);
        let sb = state(b)?;
        if sb != sa {
            let mut f = |jd: f64| -> Result<f64> { Ok(if state(jd)? == sa { 1.0 } else { -1.0 }) };
            let t = search::bisect(&mut f, a, b, 0.0, Some(24))?;
            if sb {
                open = Some((t, false));
            } else if let Some((start, clipped)) = open.take() {
                out.push(json!({"start_utc": Instant::from_jd_utc(start).utc, "end_utc": Instant::from_jd_utc(t).utc,
                    "start_jd_ut": start, "end_jd_ut": t, "days": t - start, "clipped_start": clipped, "clipped_end": false}));
            }
        }
        a = b;
        sa = sb;
    }
    if let Some((start, clipped)) = open {
        out.push(json!({"start_utc": Instant::from_jd_utc(start).utc, "end_utc": Instant::from_jd_utc(jd_b).utc,
            "start_jd_ut": start, "end_jd_ut": jd_b, "days": jd_b - start, "clipped_start": clipped, "clipped_end": true}));
    }
    Ok(out)
}
