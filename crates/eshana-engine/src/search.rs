//! The shared event solver: roots of a continuous function of time, found by sampling and bisection.
//!
//! Every event search in the engine (sign ingresses, stations, crossings of a longitude) uses the same two steps:
//! sample at a step short enough for the body, then refine each bracket by bisection to a stated tolerance. Angles are
//! compared as signed separations in (-180°, 180°], so 359.99° → 0.01° is a crossing of 0°, never a jump.
//!
//! A sampling step can hide two roots: a planet that crosses a point, stations and crosses back inside one step leaves
//! the same sign at both samples. Every local minimum of |f| among the samples is therefore searched for the extremum
//! between its neighbours; when the extremum has the other sign, both roots are refined, and when it only comes within
//! `touch_deg` of zero it is reported as a touch, not as a crossing.
//!
//! The tolerance describes the solver, not the ephemeris: an analytic provider's position error moves an event by far
//! more than a second for the slow outer planets.

use eshana_core::Result;

/// A root of `f` found inside one bracket: the time and the value of `f` there (the residual).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Root {
    /// UT Julian day.
    pub jd: f64,
    /// `f(jd)`: how far from exact the refined root is.
    pub residual: f64,
    /// `true` when `f` rises through zero, `false` when it falls.
    pub rising: bool,
}

/// What a search finds: crossings, and touches that came within `touch` of zero without crossing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Found {
    /// Sign changes of `f`, in time order.
    pub roots: Vec<Root>,
    /// Closest approaches that did not cross: (jd, f at that time).
    pub touches: Vec<(f64, f64)>,
}

/// Bisect `[lo, hi]` (where `f` changes sign) to `tol_days`, or for `max_iter` halvings when given.
///
/// # Errors
/// Errors of `f`.
pub fn bisect<F>(f: &mut F, mut lo: f64, mut hi: f64, tol_days: f64, max_iter: Option<u32>) -> Result<f64>
where
    F: FnMut(f64) -> Result<f64>,
{
    let f_lo = f(lo)?;
    let mut iter = 0;
    loop {
        if let Some(m) = max_iter {
            if iter >= m {
                break;
            }
        } else if hi - lo <= tol_days {
            break;
        }
        let mid = 0.5 * (lo + hi);
        if (f(mid)? > 0.0) == (f_lo > 0.0) {
            lo = mid;
        } else {
            hi = mid;
        }
        iter += 1;
        if iter > 200 {
            break;
        }
    }
    Ok(hi)
}

/// Extremum of `s·f` on `[a, b]` by golden-section search (`s` = ±1 picks a minimum or a maximum of `f`).
fn golden_extremum<F>(f: &mut F, mut a: f64, mut b: f64, s: f64, tol_days: f64) -> Result<(f64, f64)>
where
    F: FnMut(f64) -> Result<f64>,
{
    const R: f64 = 0.618_033_988_749_894_9;
    let mut c = b - R * (b - a);
    let mut d = a + R * (b - a);
    let (mut fc, mut fd) = (s * f(c)?, s * f(d)?);
    let mut guard = 0;
    while b - a > tol_days && guard < 200 {
        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - R * (b - a);
            fc = s * f(c)?;
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + R * (b - a);
            fd = s * f(d)?;
        }
        guard += 1;
    }
    let x = 0.5 * (a + b);
    Ok((x, f(x)?))
}

/// All roots of `f` in `[start, end)` sampled every `step` days and refined to `tol_days`.
///
/// `f` must be continuous (for angles, a signed separation from the target). A sign change between samples where
/// either side exceeds `wrap_guard` in magnitude is a wrap of the signed separation (the far side of the circle),
/// not a root; pass `f64::INFINITY` for functions that never wrap. A root exactly at `end` is not included; a root
/// exactly at `start` is.
///
/// # Errors
/// Errors of `f`.
pub fn find_roots<F>(
    f: &mut F,
    start: f64,
    end: f64,
    step: f64,
    tol_days: f64,
    touch: f64,
    wrap_guard: f64,
) -> Result<Found>
where
    F: FnMut(f64) -> Result<f64>,
{
    let mut out = Found::default();
    if end <= start || step <= 0.0 {
        return Ok(out);
    }
    let n = ((end - start) / step).ceil() as usize;
    let mut t = Vec::with_capacity(n + 1);
    let mut v = Vec::with_capacity(n + 1);
    for k in 0..=n {
        let x = (start + step * k as f64).min(end);
        t.push(x);
        v.push(f(x)?);
    }
    let real_change = |a: f64, b: f64| (a > 0.0) != (b > 0.0) && a.abs() < wrap_guard && b.abs() < wrap_guard;
    for k in 0..n {
        let (a, b) = (v[k], v[k + 1]);
        if a == 0.0 && k == 0 {
            out.roots.push(Root {
                jd: t[0],
                residual: 0.0,
                rising: b > 0.0,
            });
            continue;
        }
        if real_change(a, b) {
            let jd = bisect(f, t[k], t[k + 1], tol_days, None)?;
            if jd < end {
                out.roots.push(Root {
                    jd,
                    residual: f(jd)?,
                    rising: b > a,
                });
            }
        }
    }
    // hidden pairs and touches: a sample nearer zero than both neighbours, with no sign change on either side
    for k in 1..n {
        let (p, c, q) = (v[k - 1], v[k], v[k + 1]);
        if c.abs() > p.abs() || c.abs() > q.abs() || real_change(p, c) || real_change(c, q) {
            continue;
        }
        if c.abs() >= wrap_guard {
            continue;
        }
        let s = if c > 0.0 { 1.0 } else { -1.0 };
        let (xm, fm) = golden_extremum(f, t[k - 1], t[k + 1], s, tol_days)?;
        if (fm > 0.0) != (c > 0.0) {
            for (lo, hi) in [(t[k - 1], xm), (xm, t[k + 1])] {
                let jd = bisect(f, lo, hi, tol_days, None)?;
                if jd >= start && jd < end {
                    let after = f((jd + tol_days).min(hi))?;
                    out.roots.push(Root {
                        jd,
                        residual: f(jd)?,
                        rising: after > 0.0,
                    });
                }
            }
        } else if fm.abs() <= touch {
            out.touches.push((xm, fm));
        }
    }
    out.roots.sort_by(|a, b| a.jd.total_cmp(&b.jd));
    out.roots.dedup_by(|b, a| (b.jd - a.jd).abs() < tol_days);
    out.touches.sort_by(|a, b| a.0.total_cmp(&b.0));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eshana_core::angular_diff;

    const SEC: f64 = 1.0 / 86_400.0;

    #[test]
    fn a_simple_root_is_refined_to_the_tolerance() {
        let mut f = |x: f64| Ok(x - 1.234_567);
        let r = find_roots(&mut f, 0.0, 3.0, 0.5, SEC, 0.0, f64::INFINITY).unwrap();
        assert_eq!(r.roots.len(), 1);
        assert!((r.roots[0].jd - 1.234_567).abs() <= SEC);
        assert!(r.roots[0].rising);
    }

    #[test]
    fn the_angle_wrap_is_a_crossing_of_zero_and_the_far_side_is_not() {
        // longitude moving 359 → 1 (through 0) then on to 181 → 183 (through 180, the far side of target 0)
        let lon = |x: f64| (359.0 + x).rem_euclid(360.0);
        let mut f = |x: f64| Ok(angular_diff(lon(x), 0.0));
        let r = find_roots(&mut f, 0.0, 190.0, 0.5, SEC, 0.0, 90.0).unwrap();
        assert_eq!(r.roots.len(), 1, "{r:?}");
        assert!((r.roots[0].jd - 1.0).abs() <= SEC);
    }

    #[test]
    fn two_crossings_inside_one_step_are_both_found() {
        // a parabola dipping below zero for 0.2 days, sampled every 2 days: no sign change at any sample
        let mut f = |x: f64| Ok((x - 5.03).powi(2) - 0.01);
        let r = find_roots(&mut f, 0.0, 10.0, 2.0, SEC, 0.0, f64::INFINITY).unwrap();
        assert_eq!(r.roots.len(), 2, "{r:?}");
        assert!((r.roots[0].jd - 4.93).abs() < 2.0 * SEC);
        assert!((r.roots[1].jd - 5.13).abs() < 2.0 * SEC);
        assert!(!r.roots[0].rising && r.roots[1].rising);
    }

    #[test]
    fn a_tangent_touch_is_not_a_crossing() {
        let mut f = |x: f64| Ok((x - 5.03).powi(2) + 1e-4);
        let r = find_roots(&mut f, 0.0, 10.0, 2.0, SEC, 1e-3, f64::INFINITY).unwrap();
        assert!(r.roots.is_empty());
        assert_eq!(r.touches.len(), 1);
        assert!((r.touches[0].0 - 5.03).abs() < 1e-3);
    }

    #[test]
    fn a_root_on_a_sample_is_reported_once() {
        let mut f = |x: f64| Ok(x - 2.0);
        let r = find_roots(&mut f, 0.0, 4.0, 1.0, SEC, 0.0, f64::INFINITY).unwrap();
        assert_eq!(r.roots.len(), 1, "{r:?}");
    }

    #[test]
    fn a_root_at_the_end_belongs_to_the_next_range() {
        let mut f = |x: f64| Ok(x - 4.0);
        let r = find_roots(&mut f, 0.0, 4.0, 1.0, SEC, 0.0, f64::INFINITY).unwrap();
        assert!(r.roots.is_empty(), "{r:?}");
        let r = find_roots(&mut f, 4.0, 8.0, 1.0, SEC, 0.0, f64::INFINITY).unwrap();
        assert_eq!(r.roots.len(), 1);
    }
}
