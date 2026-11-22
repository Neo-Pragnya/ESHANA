//! Annual and daily: Tājika varṣaphala (solar return, varṣa lagna, muntha, the five office-bearers, year lord and
//! pañcavargīya bala), Tamil pañca pakṣī (birth bird and the day and night activity table), and the KP significator
//! table added to `vedic.kp`. Tables and every convention choice live in `rulesets/annual.yaml` (ADR-012).

use crate::analysis::{AnalysisCtx, Derived};
use crate::events::lon_at;
use crate::jyotish::{house_from, kp_lords, lord_of, sign_of, VChart, NINE, SEVEN};
use crate::panchanga::{self, Rising, Site};
use crate::{astro, packs, vedic};
use eshana_core::{norm360, Body, Result};
use eshana_ephemeris::frames;
use eshana_time::Instant;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Rule id of the varṣaphala attribute.
pub const RULE_VARSHA: &str = "vedic.varshaphala@0.1";
/// Rule id of the pañca pakṣī attribute.
pub const RULE_PAKSHI: &str = "vedic.panch_pakshi@0.1";
/// Vedic days around the reference day given as `nearby_days` (offset 0 is the reference day table itself).
const NEARBY_DAYS: std::ops::RangeInclusive<i32> = -3..=3;
/// Mean sidereal year in days, used only as the first guess of a solar return.
const SIDEREAL_YEAR: f64 = 365.256_363;

// ── the pack ─────────────────────────────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Deserialize)]
struct Pack {
    id: String,
    version: String,
    tajika: Tajika,
    pakshi: Pakshi,
}

#[derive(Debug, Deserialize)]
struct Maxima {
    griha: f64,
    uchcha: f64,
    hudda: f64,
    drekkana: f64,
    navamsha: f64,
}

#[derive(Debug, Deserialize)]
struct Fraction {
    own: f64,
    friend: f64,
    neutral: f64,
    enemy: f64,
}

#[derive(Debug, Deserialize)]
struct Tajika {
    maxima: Maxima,
    fraction: Fraction,
    divisor: f64,
    friend_houses: Vec<usize>,
    enemy_houses: Vec<usize>,
    hudda: Vec<Vec<(String, f64)>>,
    drekkana_cycle: Vec<String>,
    trirashi: Vec<[String; 2]>,
    aspect_houses: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct ByPaksha {
    shukla: Vec<String>,
    krishna: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ByHalf {
    shukla_day: Vec<String>,
    shukla_night: Vec<String>,
    krishna_day: Vec<String>,
    krishna_night: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Relations {
    friends: Vec<String>,
    enemies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Pakshi {
    birds: Vec<String>,
    nakshatra_groups: Vec<usize>,
    birth_bird: ByPaksha,
    relations: BTreeMap<String, Relations>,
    day_ruler: ByPaksha,
    night_ruler: ByPaksha,
    dying: ByPaksha,
    cycle: ByHalf,
    order: ByHalf,
    parts: BTreeMap<String, f64>,
}

impl ByPaksha {
    fn get(&self, krishna: bool) -> &[String] {
        if krishna {
            &self.krishna
        } else {
            &self.shukla
        }
    }
}

impl ByHalf {
    fn get(&self, krishna: bool, night: bool) -> &[String] {
        match (krishna, night) {
            (false, false) => &self.shukla_day,
            (false, true) => &self.shukla_night,
            (true, false) => &self.krishna_day,
            (true, true) => &self.krishna_night,
        }
    }
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| {
        // A built-in pack that does not parse is a build defect, caught by the unit tests before release.
        serde_norway::from_str(include_str!("../rulesets/annual.yaml"))
            .unwrap_or_else(|e| panic!("rule pack annual.yaml is invalid: {e}"))
    })
}

/// Pack id with version (`vedic.annual@0.1`), recorded alongside each rule.
#[must_use]
pub fn pack_rule() -> String {
    let p = pack();
    packs::rule_id(&p.id, &p.version)
}

// ── module entry ─────────────────────────────────────────────────────────────────────────────────────────────────
/// Varṣaphala and pañca pakṣī for a chart. The KP significator table is added to `vedic.kp` by the chart op
/// through [`kp_significator_table`], because the KP cusps are computed there.
///
/// # Errors
/// Propagates ephemeris errors.
pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>> {
    let mut out = vec![];
    let seven = SEVEN.iter().all(|g| x.vc.lon.contains_key(*g));
    if x.place.is_some() && x.vc.asc.is_some() && seven {
        out.push(Derived {
            id: "vedic.varshaphala".into(),
            rule: RULE_VARSHA.into(),
            value: varshaphala(x)?,
            from: vec!["sun".into(), "houses".into(), "place".into()],
        });
    }
    if x.place.is_some() && seven {
        out.push(Derived {
            id: "vedic.panch_pakshi".into(),
            rule: RULE_PAKSHI.into(),
            value: panch_pakshi(x)?,
            from: vec!["moon".into(), "sun".into(), "panchanga".into()],
        });
    }
    Ok(out)
}

// ── shared helpers ───────────────────────────────────────────────────────────────────────────────────────────────
fn site(x: &AnalysisCtx) -> Site {
    let (lat, lon) = x.place.unwrap_or((0.0, 0.0));
    let rising = x
        .panchanga
        .and_then(|p| p["sunrise_convention"].as_str())
        .and_then(|s| Rising::parse(Some(s)).ok())
        .unwrap_or(Rising::UpperLimbRefracted);
    Site {
        lat,
        lon,
        offset_s: x.offset_s.unwrap_or(lon / 15.0 * 3600.0),
        rising,
        lagna_sun_at_birth: true,
    }
}

fn stamp(jd: f64, offset_s: f64) -> Value {
    let utc = Instant::from_jd_utc(jd).utc;
    let local = Instant::from_jd_utc(jd + offset_s / 86_400.0)
        .utc
        .trim_end_matches('Z')
        .to_string();
    let off = offset_s.round() as i64;
    let sign = if off < 0 { '-' } else { '+' };
    json!({"utc": utc, "local": format!("{local}{sign}{:02}:{:02}", off.abs() / 3600, (off.abs() % 3600) / 60), "jd_ut": jd})
}

/// Signed shortest difference `a − b` in degrees, in (−180, 180].
fn signed(a: f64, b: f64) -> f64 {
    let d = norm360(a - b);
    if d > 180.0 {
        d - 360.0
    } else {
        d
    }
}

fn graha_body(g: &str) -> Body {
    match g {
        "sun" => Body::Sun,
        "moon" => Body::Moon,
        "mars" => Body::Mars,
        "mercury" => Body::Mercury,
        "jupiter" => Body::Jupiter,
        "venus" => Body::Venus,
        _ => Body::Saturn,
    }
}

fn graha(name: &str) -> &'static str {
    SEVEN.iter().copied().find(|g| *g == name).unwrap_or("sun")
}

// ── Tājika ───────────────────────────────────────────────────────────────────────────────────────────────────────
/// Tājika relationship of `g` (at sign `gs`) towards a division lord standing at sign `ls`.
fn tajika_relation(g: &str, lord: &str, gs: usize, ls: usize) -> &'static str {
    if g == lord {
        return "own";
    }
    let t = &pack().tajika;
    let h = house_from(gs, ls);
    if t.friend_houses.contains(&h) {
        "friend"
    } else if t.enemy_houses.contains(&h) {
        "enemy"
    } else {
        "neutral"
    }
}

fn fraction(rel: &str) -> f64 {
    let f = &pack().tajika.fraction;
    match rel {
        "own" => f.own,
        "friend" => f.friend,
        "enemy" => f.enemy,
        _ => f.neutral,
    }
}

/// Hudda (Egyptian term) lord of a sidereal longitude.
fn hudda_lord(lon: f64) -> &'static str {
    let s = sign_of(lon);
    let deg = norm360(lon) - 30.0 * s as f64;
    let terms = &pack().tajika.hudda[s];
    let name = terms
        .iter()
        .find(|(_, end)| deg < *end)
        .map_or(terms[terms.len() - 1].0.as_str(), |(g, _)| g.as_str());
    graha(name)
}

/// Drekkāṇa lord in the Tājika (Chaldean decan) scheme.
fn drekkana_lord(lon: f64) -> &'static str {
    let k = ((norm360(lon) / 10.0).floor() as usize).min(35);
    let cyc = &pack().tajika.drekkana_cycle;
    graha(&cyc[k % cyc.len()])
}

/// Pañcavargīya bala of the seven grahas from sidereal longitudes.
///
/// # Errors
/// Never in practice (D9 is always implemented).
pub fn pancavargiya(lon: &BTreeMap<String, f64>) -> Result<Value> {
    let t = &pack().tajika;
    let m = &t.maxima;
    let mut out = serde_json::Map::new();
    for g in SEVEN {
        let Some(&l) = lon.get(g) else { continue };
        let gs = sign_of(l);
        let sign_at = |lord: &str| lon.get(lord).map_or(gs, |v| sign_of(*v));
        let part = |lord: &'static str, max: f64| -> (Value, f64) {
            let rel = tajika_relation(g, lord, gs, sign_at(lord));
            let v = max * fraction(rel);
            (json!({"lord": lord, "relation": rel, "value": v}), v)
        };
        let (griha, a) = part(lord_of(gs), m.griha);
        let (hudda, c) = part(hudda_lord(l), m.hudda);
        let (drek, d) = part(drekkana_lord(l), m.drekkana);
        let (nav, e) = part(lord_of(vedic::divisional_sign(l, 9)?), m.navamsha);
        let deb = packs::core()
            .dignity
            .get(g)
            .map_or(0.0, |d| d.debilitation[0] * 30.0 + d.debilitation[1]);
        let from_deb = signed(l, deb).abs();
        let b = m.uchcha * from_deb / 180.0;
        let total = (a + b + c + d + e) / t.divisor;
        out.insert(
            g.into(),
            json!({"griha": a, "uchcha": b, "hudda": c, "drekkana": d, "navamsha": e, "total": total,
                   "detail": {"griha": griha, "hudda": hudda, "drekkana": drek, "navamsha": nav,
                              "uchcha": {"degrees_from_debilitation": from_deb, "value": b}}}),
        );
    }
    Ok(Value::Object(out))
}

/// The instant (UT JD) when the Sun returns to `natal_sun` (sidereal) nearest `guess`.
///
/// # Errors
/// Provider errors.
pub fn solar_return(x: &AnalysisCtx, natal_sun: f64, guess: f64) -> Result<f64> {
    let mut jd = guess;
    for _ in 0..12 {
        let now = lon_at(x.provider, Body::Sun, jd, Some(x.ayanamsha))?;
        let d = signed(natal_sun, now);
        jd += d / 0.9856;
        if d.abs() < 1e-8 {
            break;
        }
    }
    Ok(jd)
}

/// Sidereal Ascendant at a UT JD and place (the same reduction as the chart op).
fn sidereal_asc(x: &AnalysisCtx, jd: f64, lat: f64, lon_e: f64) -> f64 {
    let inst = Instant::from_jd_utc(jd);
    let t = inst.t_tt();
    let lst = astro::local_apparent_sidereal_deg(inst.gmst_deg(), lon_e, t);
    let (asc, _) = astro::asc_mc(lst, lat, t);
    norm360(asc - frames::nutation_deg(t).0 - astro::ayanamsha_deg(x.ayanamsha, t))
}

/// The node model of the natal chart: the one whose position at birth matches Rahu.
fn node_body(x: &AnalysisCtx) -> Body {
    let Some(&r) = x.vc.lon.get("rahu") else {
        return Body::TrueNode;
    };
    let at = |b| lon_at(x.provider, b, x.inst.jd_ut, Some(x.ayanamsha)).unwrap_or(f64::NAN);
    let (t, m) = (at(Body::TrueNode), at(Body::MeanNode));
    if signed(r, m).abs() < signed(r, t).abs() {
        Body::MeanNode
    } else {
        Body::TrueNode
    }
}

/// One varṣa (year `n`, 0 = the birth year) as JSON.
#[allow(clippy::too_many_lines)]
fn varsha_year(x: &AnalysisCtx, n: i64) -> Result<Value> {
    let (lat, lon_e) = x.place.unwrap_or((0.0, 0.0));
    let st = site(x);
    let natal_sun = x.vc.lon["sun"];
    let natal_asc = x.vc.asc.unwrap_or(0.0);
    let jd = if n == 0 {
        x.inst.jd_ut
    } else {
        solar_return(x, natal_sun, x.inst.jd_ut + n as f64 * SIDEREAL_YEAR)?
    };
    // the varṣa chart (year 0 is the birth chart itself)
    let mut lon: BTreeMap<String, f64> = BTreeMap::new();
    for g in SEVEN {
        let v = if n == 0 {
            x.vc.lon[g]
        } else {
            lon_at(x.provider, graha_body(g), jd, Some(x.ayanamsha))?
        };
        lon.insert(g.into(), v);
    }
    let rahu = if n == 0 {
        x.vc.lon.get("rahu").copied()
    } else {
        Some(lon_at(x.provider, node_body(x), jd, Some(x.ayanamsha))?)
    };
    if let Some(r) = rahu {
        lon.insert("rahu".into(), r);
        lon.insert("ketu".into(), norm360(r + 180.0));
    }
    let asc = if n == 0 {
        natal_asc
    } else {
        sidereal_asc(x, jd, lat, lon_e)
    };
    let lagna = sign_of(asc);
    // day or night at the return, and its weekday, from the pañcāṅga of that instant
    let pc = panchanga::panchanga(x.provider, &Instant::from_jd_utc(jd), st, x.ayanamsha, None)?;
    let day = pc["sunset"]["jd_ut"].as_f64().is_none_or(|s| jd < s);
    // muntha: the natal lagna advanced one sign per completed year
    let muntha = norm360(natal_asc + 30.0 * n as f64);
    let ms = sign_of(muntha);
    let bala = pancavargiya(&lon)?;
    let t = &pack().tajika;
    let total = |g: &str| bala[g]["total"].as_f64().unwrap_or(0.0);
    let night = usize::from(!day);
    let offices: [(&str, &'static str, String); 5] = [
        (
            "muntha_pati",
            lord_of(ms),
            format!("lord of the muntha sign ({})", astro::RASHIS[ms]),
        ),
        (
            "janma_lagna_pati",
            lord_of(sign_of(natal_asc)),
            "lord of the natal lagna".into(),
        ),
        (
            "varsha_lagna_pati",
            lord_of(lagna),
            "lord of the varṣa lagna".into(),
        ),
        (
            "trirashi_pati",
            graha(&t.trirashi[lagna][night]),
            format!(
                "trirāśi pati of {} by {}",
                astro::RASHIS[lagna],
                if day { "day" } else { "night" }
            ),
        ),
        (
            "dinaratri_pati",
            if day {
                lord_of(sign_of(lon["sun"]))
            } else {
                lord_of(sign_of(lon["moon"]))
            },
            if day {
                "day return: lord of the Sun's sign".into()
            } else {
                "night return: lord of the Moon's sign".into()
            },
        ),
    ];
    let aspects = |g: &str| t.aspect_houses.contains(&house_from(sign_of(lon[g]), lagna));
    let office_json: Vec<Value> = offices
        .iter()
        .map(|(role, g, why)| {
            json!({"role": role, "graha": g, "why": why, "total": total(g), "aspects_varsha_lagna": aspects(g),
                   "house_from_varsha_lagna": house_from(lagna, sign_of(lon[*g]))})
        })
        .collect();
    // year lord: the strongest office-bearer aspecting the varṣa lagna, else the strongest (ties: listed order)
    let pick = |only_aspecting: bool| -> Option<&'static str> {
        let mut best: Option<(&'static str, f64)> = None;
        for (_, g, _) in &offices {
            if only_aspecting && !aspects(g) {
                continue;
            }
            let v = total(g);
            if best.is_none_or(|(_, b)| v > b + 1e-12) {
                best = Some((g, v));
            }
        }
        best.map(|(g, _)| g)
    };
    let (year_lord, why) = match pick(true) {
        Some(g) => (
            g,
            "the strongest office-bearer (pañcavargīya total) that aspects the varṣa lagna",
        ),
        None => (
            pick(false).unwrap_or("sun"),
            "no office-bearer aspects the varṣa lagna: the strongest of them",
        ),
    };
    let houses: BTreeMap<String, usize> = lon
        .iter()
        .map(|(g, l)| (g.clone(), house_from(lagna, sign_of(*l))))
        .collect();
    Ok(json!({
        "completed_years": n,
        "return": stamp(jd, st.offset_s),
        "day_return": day,
        "sunrise": pc["sunrise"], "sunset": pc["sunset"],
        "weekday": pc["vara"]["index"], "weekday_lord": pc["vara"]["lord"],
        "varsha_lagna": {"longitude_deg": asc, "sign": lagna, "lord": lord_of(lagna)},
        "positions": lon,
        "houses_from_varsha_lagna": houses,
        "muntha": {"longitude_deg": muntha, "sign": ms, "lord": lord_of(ms), "house_from_varsha_lagna": house_from(lagna, ms)},
        "office_bearers": office_json,
        "year_lord": {"graha": year_lord, "total": total(year_lord), "why": why},
        "pancavargiya_bala": bala,
    }))
}

/// Number of completed solar years at `jd` (UT).
fn completed_years(x: &AnalysisCtx, jd: f64) -> Result<i64> {
    let natal_sun = x.vc.lon["sun"];
    let ret = |n: i64| solar_return(x, natal_sun, x.inst.jd_ut + n as f64 * SIDEREAL_YEAR);
    let mut n = ((jd - x.inst.jd_ut) / SIDEREAL_YEAR).floor() as i64;
    for _ in 0..3 {
        if n > 0 && ret(n)? > jd {
            n -= 1;
        } else if ret(n + 1)? <= jd {
            n += 1;
        } else {
            break;
        }
    }
    Ok(n.max(0))
}

fn varshaphala(x: &AnalysisCtx) -> Result<Value> {
    let current = match x.reference_jd {
        Some(r) if r >= x.inst.jd_ut => varsha_year(x, completed_years(x, r)?)?,
        _ => Value::Null,
    };
    Ok(json!({
        "current": current,
        "birth_year": varsha_year(x, 0)?,
        "natal_sun_deg": x.vc.lon["sun"],
        "rule_pack": pack_rule(),
        "conventions": {
            "return": "the Sun returns to its natal sidereal longitude (same ayanāṃśa); the varṣa lagna is cast for that instant at the birth place",
            "muntha": "the natal lagna advanced one sign per completed year, keeping its degree",
            "office_bearers": "muntha pati, janma lagna pati, varṣa lagna pati, trirāśi pati (Tājika table by day or night), dinarātri pati (day return: lord of the Sun's sign; night return: lord of the Moon's sign)",
            "year_lord": "the strongest office-bearer that aspects the varṣa lagna in the Tājika way (not from the 2nd, 6th, 8th or 12th); if none does, the strongest",
            "pancavargiya": "gṛha 30, uchcha 20, hudda 15 (Egyptian terms), drekkāṇa 10 (Chaldean decans), navāṃśa 5; own 1, friend 3/4, neutral 1/2, enemy 1/4 by the Tājika relationship (3/5/9/11 friend, 1/4/7/10 enemy, others neutral, counted from the graha to the division lord); total = sum / 4",
            "current_year": "the year whose solar return is the last one at or before the reference instant",
        },
    }))
}

// ── pañca pakṣī ──────────────────────────────────────────────────────────────────────────────────────────────────
fn paksha_krishna(x: &AnalysisCtx, jd: f64) -> Result<bool> {
    let s = lon_at(x.provider, Body::Sun, jd, Some(x.ayanamsha))?;
    let m = lon_at(x.provider, Body::Moon, jd, Some(x.ayanamsha))?;
    Ok(norm360(m - s) >= 180.0)
}

fn relation(selfb: &str, other: &str) -> &'static str {
    if selfb == other {
        return "self";
    }
    match pack().pakshi.relations.get(selfb) {
        Some(r) if r.friends.iter().any(|b| b == other) => "friend",
        Some(r) if r.enemies.iter().any(|b| b == other) => "enemy",
        _ => "neutral",
    }
}

/// Activity of every bird in each of the five yāmas of a day (or night): one map bird → activity per yāma.
fn half_activities(krishna: bool, night: bool, weekday: usize) -> Vec<BTreeMap<String, String>> {
    let p = &pack().pakshi;
    let cyc = p.cycle.get(krishna, night);
    let order = p.order.get(krishna, night);
    let ruler = if night {
        &p.night_ruler.get(krishna)[weekday % 7]
    } else {
        &p.day_ruler.get(krishna)[weekday % 7]
    };
    let r_pos = order.iter().position(|b| b == ruler).unwrap_or(0);
    let rule_step = cyc.iter().position(|a| a == "ruling").unwrap_or(0);
    (0..5)
        .map(|y| {
            (0..5)
                .map(|k| {
                    (
                        order[(r_pos + k) % 5].clone(),
                        cyc[(rule_step + k + y) % 5].clone(),
                    )
                })
                .collect()
        })
        .collect()
}

/// Sub-periods of one yāma for `selfb` doing `act`: activities follow the cycle from `act`, birds the half's order
/// from `selfb`, lengths in the 30/36/48/18/12 parts.
fn sub_periods(krishna: bool, night: bool, selfb: &str, act: &str, span: (f64, f64), off: f64) -> Vec<Value> {
    let p = &pack().pakshi;
    let cyc = p.cycle.get(krishna, night);
    let order = p.order.get(krishna, night);
    let s0 = cyc.iter().position(|c| c == act).unwrap_or(0);
    let b0 = order.iter().position(|o| o == selfb).unwrap_or(0);
    let total: f64 = p.parts.values().sum();
    let mut t = span.0;
    let mut out = vec![];
    for k in 0..5 {
        let activity = &cyc[(s0 + k) % 5];
        let bird = &order[(b0 + k) % 5];
        let len = (span.1 - span.0) * p.parts.get(activity).copied().unwrap_or(28.8) / total;
        out.push(
            json!({"bird": bird, "activity": activity, "relation": relation(selfb, bird),
                        "start": stamp(t, off), "end": stamp(t + len, off)}),
        );
        t += len;
    }
    out
}

/// The pañca pakṣī table of the Vedic day (sunrise to the next sunrise) containing `anchor_jd`, for the bird
/// `selfb`, marking the period that contains `at`.
fn pakshi_day(x: &AnalysisCtx, anchor_jd: f64, selfb: &str, at: f64) -> Result<Value> {
    let st = site(x);
    let pc = panchanga::panchanga(
        x.provider,
        &Instant::from_jd_utc(anchor_jd),
        st,
        x.ayanamsha,
        None,
    )?;
    let (Some(sr), Some(ss), Some(nsr), Some(wd)) = (
        pc["sunrise"]["jd_ut"].as_f64(),
        pc["sunset"]["jd_ut"].as_f64(),
        pc["next_sunrise"]["jd_ut"].as_f64(),
        pc["vara"]["index"].as_u64().map(|v| v as usize % 7),
    ) else {
        return Ok(json!({"note": "the Sun does not rise or set on this date at this latitude; no yāmas"}));
    };
    let krishna = paksha_krishna(x, sr)?;
    let p = &pack().pakshi;
    let mut yamas = vec![];
    for (night, a0, len) in [(false, sr, (ss - sr) / 5.0), (true, ss, (nsr - ss) / 5.0)] {
        for (y, acts) in half_activities(krishna, night, wd).into_iter().enumerate() {
            let a = a0 + y as f64 * len;
            let act = acts.get(selfb).cloned().unwrap_or_default();
            let birds: Vec<Value> = p
                .birds
                .iter()
                .map(|b| json!({"bird": b, "activity": acts.get(b), "relation": relation(selfb, b)}))
                .collect();
            yamas.push(json!({
                "half": if night { "night" } else { "day" }, "yama": y + 1,
                "start": stamp(a, st.offset_s), "end": stamp(a + len, st.offset_s),
                "activity": act, "birds": birds,
                "sub_periods": sub_periods(krishna, night, selfb, &act, (a, a + len), st.offset_s),
                "current": at >= a && at < a + len,
            }));
        }
    }
    Ok(json!({
        "weekday": wd, "vara": pc["vara"]["name"], "paksha": if krishna { "krishna" } else { "shukla" },
        "sunrise": stamp(sr, st.offset_s), "sunset": stamp(ss, st.offset_s), "next_sunrise": stamp(nsr, st.offset_s),
        "day_ruling_bird": p.day_ruler.get(krishna)[wd], "night_ruling_bird": p.night_ruler.get(krishna)[wd],
        "dying_bird": p.dying.get(krishna)[wd],
        "yamas": yamas,
    }))
}

/// Birth bird from the Moon's nakṣatra (0-based) and the pakṣa.
fn birth_bird(nak: usize, krishna: bool) -> &'static str {
    let p = &pack().pakshi;
    let g = p.nakshatra_groups.iter().position(|e| nak < *e).unwrap_or(4);
    let name = &p.birth_bird.get(krishna)[g];
    ["vulture", "owl", "crow", "cock", "peacock"]
        .into_iter()
        .find(|b| b == name)
        .unwrap_or("vulture")
}

fn panch_pakshi(x: &AnalysisCtx) -> Result<Value> {
    let nk = vedic::nakshatra_of(x.vc.lon["moon"]);
    let krishna = paksha_krishna(x, x.inst.jd_ut)?;
    let bird = birth_bird(nk.index, krishna);
    let p = &pack().pakshi;
    let rel = &p.relations[bird];
    // ruling and dying birds of the birth's Vedic weekday (sunrise to sunrise)
    let vwd = x
        .panchanga
        .and_then(|v| v["vara"]["index"].as_u64())
        .map(|v| v as usize % 7);
    // the tables are for the Vedic day (sunrise to the next sunrise) containing the instant
    let birth_table = pakshi_day(x, x.inst.jd_ut, bird, x.inst.jd_ut)?;
    let today = match x.reference_jd {
        Some(r) => pakshi_day(x, r, bird, r)?,
        None => Value::Null,
    };
    // the Vedic days around the reference day, so a reader can step through them: each anchored one hour after
    // the reference sunrise plus whole days, which lies inside that Vedic day at any latitude with a sunrise
    let mut nearby = vec![];
    if let (Some(r), Some(sr)) = (x.reference_jd, today["sunrise"]["jd_ut"].as_f64()) {
        for k in NEARBY_DAYS.filter(|k| *k != 0) {
            let t = pakshi_day(x, sr + f64::from(k) + 1.0 / 24.0, bird, r)?;
            nearby.push(json!({"offset": k, "table": t}));
        }
    }
    Ok(json!({
        "birth_bird": bird,
        "nakshatra": {"index": nk.index, "name": nk.name},
        "paksha": if krishna { "krishna" } else { "shukla" },
        "friends": rel.friends, "enemies": rel.enemies,
        "birth_day": vwd.map(|w| json!({
            "weekday": w,
            "ruling_bird": p.day_ruler.get(krishna)[w],
            "night_ruling_bird": p.night_ruler.get(krishna)[w],
            "dying_bird": p.dying.get(krishna)[w],
        })),
        "birth_day_table": birth_table,
        "reference_day_table": today,
        "nearby_days": nearby,
        "rule_pack": pack_rule(),
        "source": "Tamil pañca pakṣī śāstra (traditional tables as commonly reprinted in Tamil almanacs); conventions in rulesets/annual.yaml",
        "conventions": {
            "birth_bird": "the Moon's nakṣatra in groups 1-5, 6-11, 12-16, 17-21, 22-27 (Vulture, Owl, Crow, Cock, Peacock), reversed in the dark half",
            "birth_day": "ruling and dying birds of the Vedic weekday (sunrise to sunrise) of the birth",
            "birth_day_table": "the Vedic day (sunrise to the next sunrise) containing the birth: five yāmas by day and five by night",
            "reference_day_table": "the Vedic day containing the reference instant",
            "nearby_days": "the three Vedic days before and the three after the reference day (offset −3 … +3, the reference day itself is reference_day_table), each with its own sunrise, sunset, pakṣa and weekday",
            "first_yama": "the day's (or night's) ruling bird rules the first yāma; the other birds follow the half's bird order with the next activities of its cycle; every bird moves one step per yāma",
            "sub_periods": "Eating 30, Walking 36, Ruling 48, Sleeping 18, Dying 12 parts of 144, in cycle order from the yāma's activity, birds in the half's order from the birth bird",
        },
    }))
}

// ── KP significator table ────────────────────────────────────────────────────────────────────────────────────────
fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|s| s.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// The KP significator table, derived from the `vedic.kp` value ([`crate::jyotish::kp`]): per bhāva the cusp, its
/// sign, star, sub and sub-sub lords and the four significator levels (grahas in the stars of occupants, occupants,
/// grahas in the star of the owner, owner); per graha its lords and the houses it signifies at four strengths
/// (very strong: the house its star lord occupies; strong: the house it occupies; normal: houses owned by its star
/// lord; weak: houses it owns). Also the lords of equal cusps from the Ascendant, a common alternative view.
#[must_use]
pub fn kp_significator_table(vc: &VChart, kp: &Value) -> Value {
    let mut houses = vec![];
    for h in 1..=12usize {
        let c = &kp["cusps"][h.to_string()];
        let l = &c["lords"];
        let cusp = c["cusp"].as_f64().unwrap_or(0.0);
        houses.push(json!({
            "house": h, "cusp": cusp, "sign": sign_of(cusp),
            "sign_lord": l["sign_lord"], "star_lord": l["star_lord"], "sub_lord": l["sub_lord"], "sub_sub_lord": l["sub_sub_lord"],
            "in_star_of_occupants": c["A_in_star_of_occupants"],
            "occupants": c["B_occupants"],
            "in_star_of_owner": c["C_in_star_of_lord"],
            "owner": c["D_lord"],
        }));
    }
    let mut grahas = vec![];
    for g in NINE {
        let (Some(l), Some(pv)) = (kp["planets"].get(g), kp["significators_planet_view"].get(g)) else {
            continue;
        };
        let very: Vec<Value> = pv["A_house_of_star_lord"]
            .as_u64()
            .map(|v| vec![json!(v)])
            .unwrap_or_default();
        grahas.push(json!({
            "graha": g, "longitude_deg": vc.lon.get(g),
            "sign_lord": l["sign_lord"], "star_lord": l["star_lord"], "sub_lord": l["sub_lord"], "sub_sub_lord": l["sub_sub_lord"],
            "very_strong": very, "strong": [pv["B_house_occupied"]],
            "normal": pv["C_houses_owned_by_star_lord"], "weak": pv["D_houses_owned"],
        }));
    }
    // house → significator grahas, strongest level first
    let mut by_house = serde_json::Map::new();
    for h in &houses {
        let mut list: Vec<String> = vec![];
        for key in ["in_star_of_occupants", "occupants", "in_star_of_owner"] {
            for g in strs(&h[key]) {
                if !list.contains(&g) {
                    list.push(g);
                }
            }
        }
        if let Some(o) = h["owner"].as_str() {
            if !list.iter().any(|g| g == o) {
                list.push(o.into());
            }
        }
        by_house.insert(h["house"].to_string(), json!(list));
    }
    let equal = vc.asc.map_or(Value::Null, |asc| {
        Value::Array(
            (0..12)
                .map(|k| {
                    let c = norm360(asc + 30.0 * f64::from(k));
                    let l = kp_lords(c);
                    json!({"house": k + 1, "cusp": c, "sign_lord": l["sign_lord"], "star_lord": l["star_lord"],
                           "sub_lord": l["sub_lord"], "sub_sub_lord": l["sub_sub_lord"]})
                })
                .collect(),
        )
    });
    json!({
        "houses": houses, "grahas": grahas, "significators_by_house": by_house,
        "equal_cusps_from_ascendant": equal,
        "levels": {"very_strong": "the house occupied by the graha's star lord", "strong": "the house the graha occupies",
                   "normal": "houses owned by its star lord", "weak": "houses it owns"},
        "note": "Rahu and Ketu also act as agents of their sign lord and of grahas joined with them; that agency is not added here",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lons(v: &[(&str, f64)]) -> BTreeMap<String, f64> {
        v.iter().map(|(g, l)| ((*g).to_string(), *l)).collect()
    }

    #[test]
    fn pack_parses_and_is_complete() {
        let p = pack();
        assert_eq!(p.tajika.hudda.len(), 12);
        assert!(p
            .tajika
            .hudda
            .iter()
            .all(|t| t.len() == 5 && (t[4].1 - 30.0).abs() < 1e-9));
        assert_eq!(p.tajika.trirashi.len(), 12);
        for w in [&p.pakshi.day_ruler, &p.pakshi.night_ruler, &p.pakshi.dying] {
            assert_eq!(w.shukla.len(), 7);
            assert_eq!(w.krishna.len(), 7);
        }
        assert!((p.pakshi.parts.values().sum::<f64>() - 144.0).abs() < 1e-9);
        assert_eq!(pack_rule(), "vedic.annual@0.1");
    }

    #[test]
    fn term_and_decan_lords() {
        assert_eq!(hudda_lord(8.0 * 30.0 + 15.6), "venus"); // Sagittarius 12-17: Venus
        assert_eq!(hudda_lord(2.0 * 30.0 + 28.3), "saturn");
        assert_eq!(drekkana_lord(0.5), "mars");
        assert_eq!(drekkana_lord(8.0 * 30.0 + 15.6), "moon"); // Sagittarius 2nd decan
        assert_eq!(drekkana_lord(11.0 * 30.0 + 17.0), "jupiter");
    }

    #[test]
    fn pancavargiya_worked_example() {
        // Sri Ramana Maharshi's natal grahas (sidereal, Lahiri); the published table reads
        // Su 22.50 7.29 7.50 2.50 5.00 = 11.20 and Ma 30.00 10.67 15.00 5.00 2.50 = 15.79.
        let l = lons(&[
            ("sun", 255.595),
            ("moon", 88.328),
            ("mars", 21.978),
            ("mercury", 233.109),
            ("jupiter", 316.535),
            ("venus", 210.478),
            ("saturn", 347.023),
        ]);
        let b = pancavargiya(&l).unwrap();
        let near = |g: &str, k: &str, v: f64| {
            assert!((b[g][k].as_f64().unwrap() - v).abs() < 0.01, "{g} {k}");
        };
        near("sun", "griha", 22.5);
        near("sun", "uchcha", 7.29);
        near("sun", "hudda", 7.5);
        near("sun", "drekkana", 2.5);
        near("sun", "navamsha", 5.0);
        near("sun", "total", 11.20);
        near("mars", "total", 15.79);
        near("saturn", "hudda", 11.25);
    }

    #[test]
    fn birth_bird_groups() {
        assert_eq!(birth_bird(6, true), "cock"); // Punarvasu, dark half
        assert_eq!(birth_bird(6, false), "owl");
        assert_eq!(birth_bird(0, false), "vulture");
        assert_eq!(birth_bird(26, true), "vulture");
    }

    #[test]
    fn pakshi_first_yama_matches_reference_day() {
        // Tuesday, dark half, day: Cock rules, Owl walks, Peacock eats, Crow dies, Vulture sleeps.
        let y = half_activities(true, false, 2);
        assert_eq!(y[0]["cock"], "ruling");
        assert_eq!(y[0]["owl"], "walking");
        assert_eq!(y[0]["peacock"], "eating");
        assert_eq!(y[0]["crow"], "dying");
        assert_eq!(y[0]["vulture"], "sleeping");
        let cock: Vec<&str> = y.iter().map(|m| m["cock"].as_str()).collect();
        assert_eq!(cock, ["ruling", "walking", "eating", "dying", "sleeping"]);
        for m in &y {
            let mut v: Vec<&String> = m.values().collect();
            v.sort();
            v.dedup();
            assert_eq!(v.len(), 5, "each yāma is a permutation of the five activities");
        }
    }

    #[test]
    fn sub_periods_fill_the_yama() {
        let s = sub_periods(true, false, "cock", "ruling", (0.0, 0.1), 0.0);
        assert_eq!(s.len(), 5);
        assert_eq!(s[0]["bird"], "cock");
        assert_eq!(s[1]["bird"], "owl");
        assert_eq!(s[1]["activity"], "walking");
        let end = s[4]["end"]["jd_ut"].as_f64().unwrap();
        assert!((end - 0.1).abs() < 1e-12);
    }

    #[test]
    fn tajika_relations() {
        assert_eq!(tajika_relation("sun", "jupiter", 8, 10), "friend"); // 3rd
        assert_eq!(tajika_relation("moon", "saturn", 2, 11), "enemy"); // 10th
        assert_eq!(tajika_relation("mars", "venus", 0, 7), "neutral"); // 8th
        assert_eq!(tajika_relation("mars", "mars", 0, 0), "own");
    }
}
