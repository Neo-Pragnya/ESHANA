//! Pañcāṅga and day divisions: sunrise, sunset, moonrise, moonset; tithi, vāra, nakṣatra, yoga and karaṇa with
//! their end times; horā, choghaḍiyā, Rāhu kālam, Yamagaṇḍam and Gulika kālam; the Bhāva, Horā and Ghaṭī lagnas;
//! Tārā bala and Candra bala against a natal Moon. Names and tables come from `rulesets/panchanga.yaml`.
//!
//! Rise and set are found by searching the apparent altitude of the disc centre against a standard altitude
//! (Sun −0°50′; Moon 0.7275 × horizontal parallax − 0°34′), with geocentric positions from the active provider.

use crate::astro;
use crate::packs;
use crate::vedic::{self, NAKSHATRA_SPAN};
use eshana_core::{deg, norm360, rad, Ayanamsha, Body, Result};
use eshana_ephemeris::{frames, EphemerisProvider};
use eshana_time::Instant;
use serde_json::{json, Value};

const AU_KM: f64 = 149_597_870.7;
const EARTH_RADIUS_KM: f64 = 6378.14;

/// How sunrise is defined. Almanacs use the upper limb with refraction; many Indian traditions (and the
/// common Jyotiṣa software default) use the centre of the disc without refraction, a few minutes later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rising {
    /// Upper limb touching the horizon, standard refraction (−0°50′ for the Sun).
    UpperLimbRefracted,
    /// Centre of the disc, standard refraction (−0°34′).
    CentreRefracted,
    /// Centre of the disc on the geometric horizon, no refraction.
    CentreUnrefracted,
}

impl Rising {
    /// Parse `upper_limb_refracted` (default), `centre_refracted` or `centre_unrefracted`.
    ///
    /// # Errors
    /// [`eshana_core::EshanaError::Input`] for another value.
    pub fn parse(s: Option<&str>) -> Result<Self> {
        match s.unwrap_or("upper_limb_refracted") {
            "upper_limb_refracted" => Ok(Rising::UpperLimbRefracted),
            "centre_refracted" | "center_refracted" => Ok(Rising::CentreRefracted),
            "centre_unrefracted" | "center_unrefracted" => Ok(Rising::CentreUnrefracted),
            other => Err(eshana_core::EshanaError::Input(format!(
                "unknown sunrise convention '{other}' (upper_limb_refracted, centre_refracted, centre_unrefracted)"
            ))),
        }
    }
    /// Stable id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Rising::UpperLimbRefracted => "upper_limb_refracted",
            Rising::CentreRefracted => "centre_refracted",
            Rising::CentreUnrefracted => "centre_unrefracted",
        }
    }
}

/// Where and how a day is reckoned.
#[derive(Debug, Clone, Copy)]
pub struct Site {
    /// Degrees north.
    pub lat: f64,
    /// Degrees east.
    pub lon: f64,
    /// Civil offset in seconds east of UTC used to define the local calendar date.
    pub offset_s: f64,
    /// Sunrise convention.
    pub rising: Rising,
    /// Special lagnas count from the Sun at birth (`true`, the common software convention) or at sunrise.
    pub lagna_sun_at_birth: bool,
}

/// Apparent altitude minus the standard altitude for rise/set, degrees, at a UT Julian day.
fn altitude_excess(p: &dyn EphemerisProvider, body: Body, jd_ut: f64, site: Site) -> Result<f64> {
    let inst = Instant::from_jd_utc(jd_ut);
    let t = inst.t_tt();
    let (lam, beta, dist) = p.apparent(inst.jd_tt, body)?;
    let eps = rad(frames::true_obliquity_deg(t));
    let (l, b) = (rad(lam), rad(beta));
    let ra = (l.sin() * eps.cos() - b.tan() * eps.sin()).atan2(l.cos());
    let dec = (b.sin() * eps.cos() + b.cos() * eps.sin() * l.sin()).asin();
    let last = rad(astro::local_apparent_sidereal_deg(inst.gmst_deg(), site.lon, t));
    let phi = rad(site.lat);
    let alt = deg((phi.sin() * dec.sin() + phi.cos() * dec.cos() * (last - ra).cos()).asin());
    const REFRACTION: f64 = 0.5667;
    const SUN_SEMIDIAMETER: f64 = 0.2667;
    let h0 = match body {
        Body::Moon => {
            // Geocentric altitude of the Moon when its topocentric disc meets the horizon: add the horizontal
            // parallax; the semidiameter is 0.2725 x parallax.
            let km = dist.unwrap_or(0.002_57) * AU_KM;
            let parallax = deg((EARTH_RADIUS_KM / km).asin());
            match site.rising {
                Rising::UpperLimbRefracted => 0.7275 * parallax - REFRACTION,
                Rising::CentreRefracted => parallax - REFRACTION,
                Rising::CentreUnrefracted => parallax,
            }
        }
        _ => match site.rising {
            Rising::UpperLimbRefracted => -(REFRACTION + SUN_SEMIDIAMETER),
            Rising::CentreRefracted => -REFRACTION,
            Rising::CentreUnrefracted => 0.0,
        },
    };
    Ok(alt - h0)
}

/// Treat "outside the provider's range" as "no event in range": a search near the end of a provider's span must not
/// fail the whole pañcāṅga.
fn in_range<T>(r: Result<Option<T>>) -> Result<Option<T>> {
    match r {
        Err(e) if e.code() == "ESH-RANGE" => Ok(None),
        other => other,
    }
}

/// First rise (`rising = true`) or set after `jd_start` within `span_days`, as a UT Julian day; `None` when there is
/// none (polar day or night) or the search would leave the provider's range.
fn next_event(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_start: f64,
    span_days: f64,
    rising: bool,
    site: Site,
) -> Result<Option<f64>> {
    in_range(next_event_raw(p, body, jd_start, span_days, rising, site))
}

fn next_event_raw(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_start: f64,
    span_days: f64,
    rising: bool,
    site: Site,
) -> Result<Option<f64>> {
    let step = 10.0 / 1440.0;
    let mut a = jd_start;
    let mut fa = altitude_excess(p, body, a, site)?;
    while a < jd_start + span_days {
        let b = a + step;
        let fb = altitude_excess(p, body, b, site)?;
        let crosses = if rising {
            fa < 0.0 && fb >= 0.0
        } else {
            fa >= 0.0 && fb < 0.0
        };
        if crosses {
            let (mut lo, mut hi, mut flo) = (a, b, fa);
            for _ in 0..30 {
                let mid = 0.5 * (lo + hi);
                let fm = altitude_excess(p, body, mid, site)?;
                if (fm < 0.0) == (flo < 0.0) {
                    lo = mid;
                    flo = fm;
                } else {
                    hi = mid;
                }
            }
            return Ok(Some(0.5 * (lo + hi)));
        }
        a = b;
        fa = fb;
    }
    Ok(None)
}

/// UT Julian day of local midnight starting the civil date that contains `jd_ut`.
pub(crate) fn local_midnight(jd_ut: f64, site: Site) -> f64 {
    let local = jd_ut + site.offset_s / 86_400.0;
    (local - 0.5).floor() + 0.5 - site.offset_s / 86_400.0
}

/// Weekday (0 = Sunday) of the local civil date containing `jd_ut`.
fn weekday(jd_ut: f64, site: Site) -> usize {
    let local = jd_ut + site.offset_s / 86_400.0;
    ((local + 1.5).floor() as i64).rem_euclid(7) as usize
}

pub(crate) fn stamp(jd: f64, site: Site) -> Value {
    let utc = Instant::from_jd_utc(jd).utc;
    let local = Instant::from_jd_utc(jd + site.offset_s / 86_400.0)
        .utc
        .trim_end_matches('Z')
        .to_string();
    let off = site.offset_s.round() as i64;
    let sign = if off < 0 { '-' } else { '+' };
    json!({"utc": utc, "local": format!("{local}{sign}{:02}:{:02}", off.abs() / 3600, (off.abs() % 3600) / 60), "jd_ut": jd})
}

pub(crate) fn sidereal(p: &dyn EphemerisProvider, body: Body, jd_ut: f64, ayan: Ayanamsha) -> Result<f64> {
    let inst = Instant::from_jd_utc(jd_ut);
    let (lam, _, _) = p.apparent(inst.jd_tt, body)?;
    Ok(astro::to_sidereal(lam, ayan, inst.t_tt()))
}

/// When a quantity that grows steadily (mod 360) next reaches a multiple of `width`, searched up to `span` days.
fn next_boundary(f: &dyn Fn(f64) -> Result<f64>, jd: f64, width: f64, span: f64) -> Result<Option<f64>> {
    let q0 = f(jd)?;
    let target = (q0 / width).floor() * width + width - q0; // degrees still to go
    let gone = |x: f64| -> Result<f64> { Ok(norm360(f(x)? - q0)) };
    let step = 1.0 / 24.0;
    let (mut a, mut b) = (jd, jd + step);
    while b <= jd + span {
        if gone(b)? >= target {
            for _ in 0..40 {
                let m = 0.5 * (a + b);
                if gone(m)? >= target {
                    b = m;
                } else {
                    a = m;
                }
            }
            return Ok(Some(b));
        }
        a = b;
        b += step;
    }
    Ok(None)
}

/// The Vedic day containing an instant: sunrise before it, sunset, and the next sunrise.
pub(crate) struct Day {
    pub(crate) sunrise: f64,
    pub(crate) sunset: f64,
    pub(crate) next_sunrise: f64,
    pub(crate) weekday: usize,
}

pub(crate) fn vedic_day(p: &dyn EphemerisProvider, jd: f64, site: Site) -> Result<Option<Day>> {
    let mid = local_midnight(jd, site);
    let today = next_event(p, Body::Sun, mid - 0.1, 1.2, true, site)?;
    let start = match today {
        Some(sr) if sr <= jd => sr,
        _ => match next_event(p, Body::Sun, mid - 1.1, 1.2, true, site)? {
            Some(sr) => sr,
            None => return Ok(None),
        },
    };
    let Some(sunset) = next_event(p, Body::Sun, start, 1.0, false, site)? else {
        return Ok(None);
    };
    let Some(next) = next_event(p, Body::Sun, sunset, 1.0, true, site)? else {
        return Ok(None);
    };
    Ok(Some(Day {
        sunrise: start,
        sunset,
        next_sunrise: next,
        weekday: weekday(start, site),
    }))
}

/// Pañcāṅga at an instant and place. `natal_moon` (sidereal) adds Tārā bala and Candra bala.
///
/// # Errors
/// Provider errors (for example an instant outside the provider's range).
#[allow(clippy::too_many_lines)]
pub fn panchanga(
    p: &dyn EphemerisProvider,
    inst: &Instant,
    site: Site,
    ayan: Ayanamsha,
    natal_moon: Option<f64>,
) -> Result<Value> {
    let pk = packs::panchanga();
    let jd = inst.jd_ut;
    let sun = sidereal(p, Body::Sun, jd, ayan)?;
    let moon = sidereal(p, Body::Moon, jd, ayan)?;
    let elong = norm360(moon - sun);
    let tithi = ((elong / 12.0).floor() as usize).min(29);
    let karana_k = ((elong / 6.0).floor() as usize).min(59);
    let karana = match karana_k {
        0 => pk.karana_first.clone(),
        57..=59 => pk.karana_last[karana_k - 57].clone(),
        k => pk.karana_movable[(k - 1) % 7].clone(),
    };
    let yoga_sum = norm360(sun + moon);
    let yoga = ((yoga_sum / NAKSHATRA_SPAN).floor() as usize).min(26);
    let nk = vedic::nakshatra_of(moon);

    let elong_f = |x: f64| -> Result<f64> {
        Ok(norm360(
            sidereal(p, Body::Moon, x, ayan)? - sidereal(p, Body::Sun, x, ayan)?,
        ))
    };
    let moon_f = |x: f64| sidereal(p, Body::Moon, x, ayan);
    let yoga_f = |x: f64| -> Result<f64> {
        Ok(norm360(
            sidereal(p, Body::Moon, x, ayan)? + sidereal(p, Body::Sun, x, ayan)?,
        ))
    };
    let ends = |f: &dyn Fn(f64) -> Result<f64>, w: f64| -> Result<Value> {
        Ok(in_range(next_boundary(f, jd, w, 2.5))?.map_or(Value::Null, |e| stamp(e, site)))
    };

    let mut out = json!({
        "tithi": {"index": tithi + 1, "name": pk.tithis[tithi], "paksha": if tithi < 15 { "shukla" } else { "krishna" },
                  "elongation_deg": elong, "fraction_elapsed": (elong % 12.0) / 12.0, "ends": ends(&elong_f, 12.0)?},
        "nakshatra": {"index": nk.index, "name": nk.name, "pada": nk.pada, "lord": nk.lord, "ends": ends(&moon_f, NAKSHATRA_SPAN)?},
        "yoga": {"index": yoga + 1, "name": pk.yogas[yoga], "ends": ends(&yoga_f, NAKSHATRA_SPAN)?},
        "karana": {"index": karana_k + 1, "name": karana, "ends": ends(&elong_f, 6.0)?},
        "sun_sidereal_deg": sun,
        "moon_sidereal_deg": moon,
    });

    match vedic_day(p, jd, site)? {
        None => {
            out["vara"] = Value::Null;
            out["note"] =
                json!("the Sun does not rise or set on this date at this latitude; day divisions omitted");
        }
        Some(day) => {
            let vara = &pk.varas[day.weekday];
            let day_len = day.sunset - day.sunrise;
            let night_len = day.next_sunrise - day.sunset;
            let part = |n: usize| -> Value {
                let a = day.sunrise + (n as f64 - 1.0) * day_len / 8.0;
                json!({"part": n, "start": stamp(a, site), "end": stamp(a + day_len / 8.0, site)})
            };
            let first = pk.hora_order.iter().position(|g| *g == vara.lord).unwrap_or(0);
            let mut horas = vec![];
            for k in 0..24usize {
                let (a, len) = if k < 12 {
                    (day.sunrise + k as f64 * day_len / 12.0, day_len / 12.0)
                } else {
                    (day.sunset + (k - 12) as f64 * night_len / 12.0, night_len / 12.0)
                };
                horas.push(
                    json!({"n": k + 1, "lord": pk.hora_order[(first + k) % 7], "start": stamp(a, site),
                                  "end": stamp(a + len, site), "current": jd >= a && jd < a + len}),
                );
            }
            let ch = &pk.choghadiya;
            let mut chog = vec![];
            for k in 0..16usize {
                let (start_idx, step, a, len) = if k < 8 {
                    (
                        ch.day_start[day.weekday],
                        ch.day_step,
                        day.sunrise + k as f64 * day_len / 8.0,
                        day_len / 8.0,
                    )
                } else {
                    (
                        ch.night_start[day.weekday],
                        ch.night_step,
                        day.sunset + (k - 8) as f64 * night_len / 8.0,
                        night_len / 8.0,
                    )
                };
                let i = (start_idx as i32 + step * (k % 8) as i32).rem_euclid(7) as usize;
                let name = &ch.names[i];
                chog.push(json!({"n": k + 1, "period": if k < 8 { "day" } else { "night" }, "name": name,
                                 "quality": ch.quality.get(name), "start": stamp(a, site), "end": stamp(a + len, site),
                                 "current": jd >= a && jd < a + len}));
            }
            let base = if site.lagna_sun_at_birth {
                sun
            } else {
                sidereal(p, Body::Sun, day.sunrise, ayan)?
            };
            let minutes = (jd - day.sunrise) * 1440.0;
            let lagna = |deg_per_min: f64| -> Value {
                let l = norm360(base + minutes * deg_per_min);
                json!({"longitude_deg": l, "sign": astro::sign_index(l)})
            };
            out["vara"] =
                json!({"index": day.weekday, "name": vara.name, "english": vara.english, "lord": vara.lord});
            out["sunrise"] = stamp(day.sunrise, site);
            out["sunset"] = stamp(day.sunset, site);
            out["next_sunrise"] = stamp(day.next_sunrise, site);
            out["day_length_hours"] = json!(day_len * 24.0);
            out["rahu_kalam"] = part(pk.rahu_kalam[day.weekday]);
            out["yamagandam"] = part(pk.yamagandam[day.weekday]);
            out["gulika_kalam"] = part(pk.gulika_kalam[day.weekday]);
            out["horas"] = json!(horas);
            out["choghadiya"] = json!(chog);
            out["special_lagnas"] = json!({
                "rule": "Sun's sidereal longitude plus elapsed time since sunrise: bhava lagna +30° per 2 hours, hora lagna +30° per hour, ghati lagna +30° per 24 minutes",
                "sun_taken_at": if site.lagna_sun_at_birth { "birth" } else { "sunrise" },
                "minutes_since_sunrise": minutes,
                "bhava_lagna": lagna(0.25), "hora_lagna": lagna(0.5), "ghati_lagna": lagna(1.25),
            });
        }
    }
    let mid = local_midnight(jd, site);
    out["moonrise"] =
        next_event(p, Body::Moon, mid, 1.0, true, site)?.map_or(Value::Null, |e| stamp(e, site));
    out["moonset"] =
        next_event(p, Body::Moon, mid, 1.0, false, site)?.map_or(Value::Null, |e| stamp(e, site));

    if let Some(nm) = natal_moon {
        let from = vedic::nakshatra_of(nm).index;
        let count = (nk.index + 27 - from) % 27 + 1;
        let tara = &pk.taras[(count - 1) % 9];
        let house = (astro::sign_index(moon) + 12 - astro::sign_index(nm)) % 12 + 1;
        out["tara_bala"] =
            json!({"count": count, "tara": tara, "favourable": pk.tara_favourable.contains(tara)});
        out["chandra_bala"] = json!({"house_from_natal_moon": house, "favourable": pk.chandra_bala_favourable.contains(&house)});
    }
    out["sunrise_convention"] = json!(site.rising.id());
    out["rule_version"] = json!(packs::rule_id(&pk.id, &pk.version));
    Ok(out)
}
