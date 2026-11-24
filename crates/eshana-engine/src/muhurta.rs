//! Muhūrta windows: factor timelines over a range, composed by partition, evaluation and merging.
//!
//! Every factor is a step function of time: a list of `[start, end)` segments, each with one value (a tithi, a
//! choghaḍiyā, "inside Rāhu kālam" …). To compose constraints the engine
//!
//! 1. collects every boundary of every factor a constraint names, and cuts the range at all of them;
//! 2. evaluates each piece once: every factor has one value there, and each constraint passes or fails;
//! 3. merges neighbouring pieces whose values and verdicts are equal, so a factor's identity is never lost in a
//!    merge and a boundary survives only where something changed.
//!
//! When nothing passes, the result says how much of the range each constraint removed on its own.
//!
//! Day-based factors (vāra, hora, choghaḍiyā, the kālams, Abhijit, Brahma muhūrta) need sunrise. Where the Sun does
//! not rise or set (polar day or night) they are *unavailable* for that span, and a constraint on an unavailable
//! factor fails there with that reason, never silently passes.

use crate::methods::{interval, unavailable};
use crate::packs;
use crate::panchanga::{self, Site};
use eshana_core::{norm360, Ayanamsha, Body, EshanaError, Result};
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The profile pack, verbatim.
pub const SOURCE: &str = include_str!("../rulesets/muhurta-profiles.yaml");

/// One constraint: a factor and an include or exclude set.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    /// Factor id.
    pub factor: String,
    /// The factor must take one of these values.
    #[serde(default)]
    pub include: Option<Vec<Value>>,
    /// The factor must not take any of these values.
    #[serde(default)]
    pub exclude: Option<Vec<Value>>,
    /// One line for the reader.
    #[serde(default)]
    pub note: Option<String>,
}

/// A named profile.
#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    /// Stable id.
    pub id: String,
    /// Version.
    pub version: String,
    /// `stable` or `experimental`.
    pub status: String,
    /// Title.
    pub title: String,
    /// Constraint ids.
    pub constraints: Vec<String>,
    /// What an experimental preset does not cover yet.
    #[serde(default)]
    pub missing: Vec<String>,
}

/// The pack.
#[derive(Debug, Clone, Deserialize)]
pub struct Pack {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Factor ids with their definitions.
    pub factors: BTreeMap<String, String>,
    /// Named constraints.
    pub constraints: BTreeMap<String, Constraint>,
    /// Profiles.
    pub profiles: Vec<Profile>,
}

/// The pack, parsed and checked once.
#[must_use]
pub fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| {
        let p: Pack = serde_norway::from_str(SOURCE).unwrap_or_else(|e| panic!("muhurta-profiles.yaml: {e}"));
        for (id, c) in &p.constraints {
            assert!(
                p.factors.contains_key(&c.factor),
                "constraint {id}: unknown factor {}",
                c.factor
            );
            assert!(
                c.include.is_some() != c.exclude.is_some(),
                "constraint {id}: give include or exclude"
            );
        }
        for pr in &p.profiles {
            assert!(
                ["stable", "experimental"].contains(&pr.status.as_str()),
                "profile {}",
                pr.id
            );
            for c in &pr.constraints {
                assert!(
                    p.constraints.contains_key(c),
                    "profile {}: unknown constraint {c}",
                    pr.id
                );
            }
        }
        p
    })
}

/// Profile ids for the method registry (built once).
#[must_use]
pub fn profile_methods() -> Vec<crate::methods::Method> {
    static M: OnceLock<Vec<crate::methods::Method>> = OnceLock::new();
    M.get_or_init(|| {
        pack()
            .profiles
            .iter()
            .map(|p| crate::methods::Method {
                id: Box::leak(p.id.clone().into_boxed_str()),
                status: if p.status == "stable" {
                    "stable"
                } else {
                    "experimental"
                },
                note: Box::leak(p.title.clone().into_boxed_str()),
            })
            .collect()
    })
    .clone()
}

/// Local time text for a UTC Julian day: the zone's offset at that instant when a zone is known (so DST and
/// historic offsets are right on each side of a change), else the site's fixed offset.
///
/// # Errors
/// `ESH-TZ-UNKNOWN`.
pub fn local_text(jd: f64, site: Site, zone: Option<&str>) -> Result<String> {
    let mut s = site;
    if let Some(z) = zone {
        s.offset_s = f64::from(eshana_time::zone_offset_seconds(z, jd)?);
    }
    Ok(panchanga::stamp(jd, s)["local"]
        .as_str()
        .unwrap_or_default()
        .to_string())
}

/// One segment of a factor timeline.
#[derive(Debug, Clone)]
struct Seg {
    a: f64,
    b: f64,
    v: Value,
}

/// A step function `index = floor(f / width)` of a quantity that only grows (mod 360), split at its steps.
fn monotone_steps(
    f: &dyn Fn(f64) -> Result<f64>,
    width: f64,
    a: f64,
    b: f64,
    label: &dyn Fn(usize) -> Value,
) -> Result<Vec<Seg>> {
    let idx = |x: f64| -> Result<usize> { Ok((norm360(f(x)?) / width).floor() as usize) };
    let step = 1.0 / 24.0;
    let mut out = vec![];
    let (mut start, mut cur) = (a, idx(a)?);
    let mut t = a;
    while t < b {
        let u = (t + step).min(b);
        let iu = idx(u)?;
        if iu != cur {
            let (mut lo, mut hi) = (t, u);
            for _ in 0..40 {
                let m = 0.5 * (lo + hi);
                if idx(m)? == cur {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            out.push(Seg {
                a: start,
                b: hi,
                v: label(cur),
            });
            start = hi;
            cur = idx(hi)?;
        }
        t = u;
    }
    out.push(Seg {
        a: start,
        b,
        v: label(cur),
    });
    Ok(out)
}

/// All factor timelines needed, over `[a, b)`.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn timelines(
    p: &dyn EphemerisProvider,
    a: f64,
    b: f64,
    site: Site,
    ayan: Ayanamsha,
    wanted: &[String],
    natal_moon: Option<f64>,
    partner_moon: Option<f64>,
) -> Result<BTreeMap<String, Vec<Seg>>> {
    let pk = packs::panchanga();
    let sun = |x: f64| panchanga::sidereal(p, Body::Sun, x, ayan);
    let moon = |x: f64| panchanga::sidereal(p, Body::Moon, x, ayan);
    let elong = |x: f64| -> Result<f64> { Ok(norm360(moon(x)? - sun(x)?)) };
    let ysum = |x: f64| -> Result<f64> { Ok(norm360(moon(x)? + sun(x)?)) };
    let nak = 360.0 / 27.0;
    let mut out = BTreeMap::new();
    let want = |f: &str| wanted.iter().any(|w| w == f);
    if want("tithi") {
        out.insert(
            "tithi".into(),
            monotone_steps(
                &elong,
                12.0,
                a,
                b,
                &|i| json!({"index": i + 1, "name": pk.tithis[i.min(29)]}),
            )?,
        );
    }
    if want("nakshatra") {
        out.insert(
            "nakshatra".into(),
            monotone_steps(
                &moon,
                nak,
                a,
                b,
                &|i| json!({"index": i + 1, "name": crate::vedic::NAKSHATRAS[i.min(26)]}),
            )?,
        );
    }
    if want("yoga") {
        out.insert(
            "yoga".into(),
            monotone_steps(
                &ysum,
                nak,
                a,
                b,
                &|i| json!({"index": i + 1, "name": pk.yogas[i.min(26)]}),
            )?,
        );
    }
    if want("karana") {
        let name = |k: usize| -> String {
            match k {
                0 => pk.karana_first.clone(),
                57..=59 => pk.karana_last[k - 57].clone(),
                k => pk.karana_movable[(k - 1) % 7].clone(),
            }
        };
        out.insert(
            "karana".into(),
            monotone_steps(
                &elong,
                6.0,
                a,
                b,
                &|i| json!({"index": i + 1, "name": name(i.min(59))}),
            )?,
        );
    }
    for (personal, who, tara) in [
        ("tara_bala", natal_moon, true),
        ("chandra_bala", natal_moon, false),
        ("tara_bala_b", partner_moon, true),
        ("chandra_bala_b", partner_moon, false),
    ] {
        if !want(personal) {
            continue;
        }
        let Some(nm) = who else {
            let na = json!({"availability": unavailable("tārā and candra bala need the natal Moon (a birth chart)", &[if personal.ends_with("_b") { "partner.moon" } else { "natal.moon" }])});
            out.insert(personal.into(), vec![Seg { a, b, v: na }]);
            continue;
        };
        let segs = if tara {
            let from = crate::vedic::nakshatra_of(nm).index;
            monotone_steps(&moon, nak, a, b, &|i| {
                let count = (i + 27 - from) % 27 + 1;
                let tara = &pk.taras[(count - 1) % 9];
                json!({"index": count, "name": tara, "quality": if pk.tara_favourable.contains(tara) { "favourable" } else { "unfavourable" }})
            })?
        } else {
            let from = crate::astro::sign_index(nm);
            monotone_steps(&moon, 30.0, a, b, &|i| {
                let house = (i + 12 - from) % 12 + 1;
                json!({"index": house, "quality": if pk.chandra_bala_favourable.contains(&house) { "favourable" } else { "unfavourable" }})
            })?
        };
        out.insert(personal.into(), segs);
    }
    if want("moon_sign") {
        out.insert(
            "moon_sign".into(),
            monotone_steps(
                &moon,
                30.0,
                a,
                b,
                &|i| json!({"index": i, "name": crate::astro::RASHIS[i.min(11)]}),
            )?,
        );
    }
    const DAY: [&str; 8] = [
        "vara",
        "hora",
        "choghadiya",
        "rahu_kalam",
        "yamagandam",
        "gulika_kalam",
        "abhijit",
        "brahma_muhurta",
    ];
    if DAY.iter().any(|f| want(f)) {
        let mut segs: BTreeMap<&str, Vec<Seg>> = DAY.iter().map(|f| (*f, vec![])).collect();
        let mut t = a;
        // Start from the Vedic day containing `a` (it may begin before `a`; pieces are clipped below).
        while t < b {
            let Some(day) = panchanga::vedic_day(p, t, site)? else {
                let until = (panchanga::local_midnight(t, site) + 1.0)
                    .min(b)
                    .max(t + 1.0 / 24.0);
                let na = json!({"availability": unavailable("the Sun does not rise or set here on this date", &["sunrise"])});
                for v in segs.values_mut() {
                    v.push(Seg {
                        a: t,
                        b: until,
                        v: na.clone(),
                    });
                }
                t = until;
                continue;
            };
            let (sr, ss, nsr) = (day.sunrise, day.sunset, day.next_sunrise);
            let (dl, nl) = (ss - sr, nsr - ss);
            let vara = &pk.varas[day.weekday];
            segs.get_mut("vara").expect("listed").push(Seg {
                a: sr,
                b: nsr,
                v: json!({"index": day.weekday, "name": vara.name, "english": vara.english, "lord": vara.lord}),
            });
            let first = pk.hora_order.iter().position(|g| *g == vara.lord).unwrap_or(0);
            for k in 0..24usize {
                let (s, len) = if k < 12 {
                    (sr + k as f64 * dl / 12.0, dl / 12.0)
                } else {
                    (ss + (k - 12) as f64 * nl / 12.0, nl / 12.0)
                };
                segs.get_mut("hora").expect("listed").push(Seg {
                    a: s,
                    b: s + len,
                    v: json!({"index": k + 1, "lord": pk.hora_order[(first + k) % 7]}),
                });
            }
            let ch = &pk.choghadiya;
            for k in 0..16usize {
                let (si, stp, s, len) = if k < 8 {
                    (
                        ch.day_start[day.weekday],
                        ch.day_step,
                        sr + k as f64 * dl / 8.0,
                        dl / 8.0,
                    )
                } else {
                    (
                        ch.night_start[day.weekday],
                        ch.night_step,
                        ss + (k - 8) as f64 * nl / 8.0,
                        nl / 8.0,
                    )
                };
                let i = (si as i32 + stp * (k % 8) as i32).rem_euclid(7) as usize;
                let name = &ch.names[i];
                segs.get_mut("choghadiya").expect("listed").push(Seg {
                    a: s,
                    b: s + len,
                    v: json!({"index": k + 1, "name": name, "quality": ch.quality.get(name)}),
                });
            }
            let part = |n: usize| (sr + (n as f64 - 1.0) * dl / 8.0, sr + n as f64 * dl / 8.0);
            let flag = |segs: &mut Vec<Seg>, (x, y): (f64, f64)| {
                segs.push(Seg {
                    a: sr,
                    b: x,
                    v: json!(false),
                });
                segs.push(Seg {
                    a: x,
                    b: y,
                    v: json!(true),
                });
                segs.push(Seg {
                    a: y,
                    b: nsr,
                    v: json!(false),
                });
            };
            flag(
                segs.get_mut("rahu_kalam").expect("listed"),
                part(pk.rahu_kalam[day.weekday]),
            );
            flag(
                segs.get_mut("yamagandam").expect("listed"),
                part(pk.yamagandam[day.weekday]),
            );
            flag(
                segs.get_mut("gulika_kalam").expect("listed"),
                part(pk.gulika_kalam[day.weekday]),
            );
            flag(
                segs.get_mut("abhijit").expect("listed"),
                (sr + 7.0 * dl / 15.0, sr + 8.0 * dl / 15.0),
            );
            flag(
                segs.get_mut("brahma_muhurta").expect("listed"),
                (nsr - 2.0 * nl / 15.0, nsr - nl / 15.0),
            );
            t = nsr;
        }
        for (k, v) in segs {
            if want(k) {
                out.insert(k.to_string(), clip(v, a, b));
            }
        }
    }
    Ok(out)
}

fn clip(v: Vec<Seg>, a: f64, b: f64) -> Vec<Seg> {
    v.into_iter()
        .filter(|s| s.b > a && s.a < b && s.b > s.a)
        .map(|s| Seg {
            a: s.a.max(a),
            b: s.b.min(b),
            v: s.v,
        })
        .collect()
}

fn value_at(segs: &[Seg], t: f64) -> Value {
    segs.iter()
        .find(|s| t >= s.a && t < s.b)
        .map_or(Value::Null, |s| s.v.clone())
}

/// Whether a factor value matches one wanted value: an index, a name, a lord, a quality, or a boolean.
fn matches(v: &Value, w: &Value) -> bool {
    if v == w {
        return true;
    }
    match w {
        Value::Number(_) => v.get("index") == Some(w),
        Value::String(s) => ["name", "lord", "quality", "english"].iter().any(|k| {
            v.get(*k)
                .and_then(Value::as_str)
                .is_some_and(|x| x.eq_ignore_ascii_case(s))
        }),
        _ => false,
    }
}

/// `Ok(true)` passes, `Err(reason)` fails.
fn check(c: &Constraint, v: &Value) -> std::result::Result<(), &'static str> {
    if v.get("availability").is_some() || v.is_null() {
        return Err("unavailable");
    }
    if let Some(inc) = &c.include {
        if !inc.iter().any(|w| matches(v, w)) {
            return Err("not included");
        }
    }
    if let Some(exc) = &c.exclude {
        if exc.iter().any(|w| matches(v, w)) {
            return Err("excluded");
        }
    }
    Ok(())
}

/// Named constraints, in order.
pub type Constraints = Vec<(String, Constraint)>;

/// Resolve the constraints of a request: a profile (minus `disable`) plus extra named or inline constraints.
///
/// # Errors
/// `ESH-METHOD` for an unknown profile; `ESH-INPUT` for an unknown constraint id or factor.
pub fn resolve(
    profile: Option<&str>,
    disable: &[String],
    extra: &[Value],
) -> Result<(Constraints, Option<&'static Profile>)> {
    let pk = pack();
    let prof = match profile {
        Some(id) => {
            crate::methods::require("muhurta_profile", id)?;
            pk.profiles.iter().find(|p| p.id == id)
        }
        None => None,
    };
    for d in disable {
        if !prof.is_some_and(|p| p.constraints.contains(d)) {
            return Err(EshanaError::Input(format!(
                "cannot disable '{d}': it is not in the profile"
            )));
        }
    }
    let mut out: Vec<(String, Constraint)> = prof
        .map(|p| {
            p.constraints
                .iter()
                .filter(|c| !disable.contains(c))
                .map(|c| (c.clone(), pk.constraints[c].clone()))
                .collect()
        })
        .unwrap_or_default();
    for (i, e) in extra.iter().enumerate() {
        if let Some(id) = e.as_str() {
            let c = pk.constraints.get(id).ok_or_else(|| {
                EshanaError::Input(format!(
                    "unknown constraint '{id}' (one of: {})",
                    pk.constraints.keys().cloned().collect::<Vec<_>>().join(", ")
                ))
            })?;
            out.push((id.to_string(), c.clone()));
        } else {
            let c: Constraint = serde_json::from_value(e.clone())
                .map_err(|err| EshanaError::Input(format!("constraint {i}: {err}")))?;
            if !pk.factors.contains_key(&c.factor) {
                return Err(EshanaError::Input(format!(
                    "unknown factor '{}' (one of: {})",
                    c.factor,
                    pk.factors.keys().cloned().collect::<Vec<_>>().join(", ")
                )));
            }
            if c.include.is_some() == c.exclude.is_some() {
                return Err(EshanaError::Input(format!(
                    "constraint {i}: give include or exclude, not both"
                )));
            }
            out.push((format!("custom_{}", i + 1), c));
        }
    }
    if out.is_empty() {
        return Err(EshanaError::Input(
            "give a profile or at least one constraint".into(),
        ));
    }
    Ok((out, prof))
}

/// Compose constraints over `[a, b)` at a site.
///
/// # Errors
/// Provider errors.
#[allow(clippy::too_many_arguments)]
pub fn windows(
    p: &dyn EphemerisProvider,
    a: f64,
    b: f64,
    site: Site,
    ayan: Ayanamsha,
    constraints: &[(String, Constraint)],
    show: &[String],
    zone: Option<&str>,
    natal_moon: Option<f64>,
    partner_moon: Option<f64>,
) -> Result<Value> {
    let mut factors: Vec<String> = constraints.iter().map(|(_, c)| c.factor.clone()).collect();
    factors.extend(show.iter().cloned());
    factors.sort();
    factors.dedup();
    let tl = timelines(p, a, b, site, ayan, &factors, natal_moon, partner_moon)?;
    // 1. partition at every boundary
    let mut cuts: Vec<f64> = vec![a, b];
    for segs in tl.values() {
        for s in segs {
            cuts.push(s.a);
            cuts.push(s.b);
        }
    }
    cuts.retain(|x| *x >= a && *x <= b);
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|x, y| (*x - *y).abs() < 1e-9);
    // 2. evaluate each piece
    struct Piece {
        a: f64,
        b: f64,
        values: BTreeMap<String, Value>,
        failed: Vec<(String, &'static str)>,
    }
    let mut pieces: Vec<Piece> = vec![];
    for w in cuts.windows(2) {
        let mid = 0.5 * (w[0] + w[1]);
        let values: BTreeMap<String, Value> = tl.iter().map(|(k, s)| (k.clone(), value_at(s, mid))).collect();
        let failed = constraints
            .iter()
            .filter_map(|(id, c)| check(c, &values[&c.factor]).err().map(|r| (id.clone(), r)))
            .collect();
        // 3. merge equal neighbours: same values and same verdicts
        if let Some(last) = pieces.last_mut() {
            if last.values == values && last.failed == failed && (last.b - w[0]).abs() < 1e-9 {
                last.b = w[1];
                continue;
            }
        }
        pieces.push(Piece {
            a: w[0],
            b: w[1],
            values,
            failed,
        });
    }
    let stamp = |jd: f64| Instant::from_jd_utc(jd).utc;
    let mut removed: BTreeMap<String, f64> = constraints.iter().map(|(id, _)| (id.clone(), 0.0)).collect();
    let mut passing = vec![];
    let mut all = vec![];
    for pc in &pieces {
        let ok = pc.failed.is_empty();
        for (id, _) in &pc.failed {
            *removed.get_mut(id).expect("listed") += pc.b - pc.a;
        }
        let mut w = interval(&stamp(pc.a), Some(&stamp(pc.b)));
        w["start_local"] = json!(local_text(pc.a, site, zone)?);
        w["end_local"] = json!(local_text(pc.b, site, zone)?);
        w["duration_minutes"] = json!((pc.b - pc.a) * 1440.0);
        w["factors"] = json!(pc.values);
        w["passes"] = json!(ok);
        w["failed"] = json!(pc
            .failed
            .iter()
            .map(|(id, why)| json!({"constraint": id, "why": why}))
            .collect::<Vec<_>>());
        if ok {
            passing.push(w.clone());
        }
        all.push(w);
    }
    let total_min = (b - a) * 1440.0;
    let mut out = json!({
        "windows": passing,
        "timeline": all,
        "constraints": constraints.iter().map(|(id, c)| json!({"id": id, "factor": c.factor, "include": c.include, "exclude": c.exclude, "note": c.note})).collect::<Vec<_>>(),
        "removed_minutes": removed.iter().map(|(k, v)| (k.clone(), json!(v * 1440.0))).collect::<serde_json::Map<_, _>>(),
        "range_minutes": total_min,
    });
    if passing.is_empty() {
        out["empty_because"] = json!(removed
            .iter()
            .filter(|(_, v)| **v > 0.0)
            .map(|(k, v)| json!({"constraint": k, "removed_minutes": v * 1440.0, "share_of_range": v * 1440.0 / total_min}))
            .collect::<Vec<_>>());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_is_consistent() {
        let p = pack();
        assert!(p
            .profiles
            .iter()
            .any(|x| x.status == "experimental" && !x.missing.is_empty()));
        for pr in &p.profiles {
            if pr.status == "experimental" {
                assert!(!pr.missing.is_empty(), "{} must say what it lacks", pr.id);
            }
        }
    }

    #[test]
    fn matching_by_index_name_and_flag() {
        let v = json!({"index": 4, "name": "Chaturthi", "quality": "auspicious"});
        assert!(matches(&v, &json!(4)));
        assert!(matches(&v, &json!("chaturthi")));
        assert!(matches(&v, &json!("auspicious")));
        assert!(!matches(&v, &json!(5)));
        assert!(matches(&json!(true), &json!(true)));
        assert!(!matches(&json!(false), &json!(true)));
    }

    #[test]
    fn unknown_profile_is_esh_method_and_disable_must_be_in_profile() {
        assert_eq!(resolve(Some("nope"), &[], &[]).unwrap_err().code(), "ESH-METHOD");
        assert_eq!(
            resolve(Some("daily_avoidances"), &["no_panchaka".into()], &[])
                .unwrap_err()
                .code(),
            "ESH-INPUT"
        );
        assert_eq!(
            resolve(None, &[], &[json!({"factor": "nope", "include": [1]})])
                .unwrap_err()
                .code(),
            "ESH-INPUT"
        );
    }
}
