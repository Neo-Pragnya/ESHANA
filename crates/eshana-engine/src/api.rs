//! The request/response facade. One entry point, [`Engine::execute`], takes a JSON request and returns a JSON
//! envelope. The CLI, the Python binding and any future binding are thin shells over this, which is what makes
//! cross-language parity a property of the design rather than something to re-test per language.

use crate::astro::{self, RASHIS, SIGNS};
use crate::{graph, hd, numerology, vedic};
use eshana_core::{
    angular_diff, dms_in_sign, norm360, Ayanamsha, Body, CalculationConfig, DerivedAttribute, Epistemic,
    EshanaError, HouseSystem, NodeModel, Provenance, Result, ZodiacSystem, FRAME_APPARENT_ECLIPTIC_OF_DATE,
    SCHEMA_VERSION,
};
use eshana_ephemeris::{frames, provider_by_id, EphemerisProvider, ProviderOptions};
use eshana_time::{FoldPolicy, Instant};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// Engine version, recorded in every provenance block.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// When something happened, in civil or UTC terms.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct When {
    /// UTC instant (`2000-01-01T12:00:00Z`) or an RFC 3339 string with an offset.
    pub utc: Option<String>,
    /// Local date `YYYY-MM-DD`.
    pub date: Option<String>,
    /// Local time `HH:MM[:SS]`. Omit for an unknown birth time.
    pub time: Option<String>,
    /// IANA zone, e.g. `Asia/Kolkata`.
    pub timezone: Option<String>,
    /// Fixed offset in minutes east of UTC, for when no zone is known.
    pub utc_offset_minutes: Option<i32>,
    /// `reject` (default), `earliest` or `latest` for a repeated local time.
    pub fold: Option<String>,
    /// Birth-time uncertainty (± minutes) for an approximate time.
    pub uncertainty_minutes: Option<f64>,
}

/// A place on Earth.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    /// Degrees north.
    pub latitude: f64,
    /// Degrees east.
    pub longitude: f64,
    /// Metres above sea level (recorded; the geocentric frame does not use it).
    pub altitude_m: Option<f64>,
}

/// Calculation settings in a request; every field optional so callers state only what they mean.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigIn {
    /// `analytic` (default) or `spk`.
    pub provider: Option<String>,
    /// `tropical` or `sidereal`.
    pub zodiac: Option<String>,
    /// Required for sidereal unless `assume_defaults` is set.
    pub ayanamsha: Option<String>,
    /// `whole_sign`, `equal`, `placidus`.
    pub house_system: Option<String>,
    /// `mean` or `true`.
    pub node_model: Option<String>,
}

/// A chart-style request.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartRequest {
    /// The moment.
    pub when: When,
    /// The place (needed for houses and the Ascendant).
    pub place: Option<Place>,
    /// Settings.
    #[serde(default)]
    pub config: ConfigIn,
    /// `vedic` or `western`.
    pub system: Option<String>,
    /// Restrict bodies, by id.
    pub bodies: Option<Vec<String>>,
    /// Divisional charts to include (Vedic).
    #[serde(default)]
    pub divisions: Vec<u32>,
    /// Include Vimśottarī daśā (Vedic).
    #[serde(default)]
    pub dasha: bool,
    /// Daśā year convention.
    pub year_convention: Option<String>,
    /// Include Ptolemaic aspects (Western).
    #[serde(default)]
    pub aspects: bool,
    /// Scale all aspect orbs.
    pub orb_scale: Option<f64>,
    /// Path to an SPK kernel for the `spk` provider.
    pub spk_path: Option<String>,
    /// Let the engine fill documented defaults (`lahiri` for a sidereal request without an ayanāṃśa) and
    /// record that it did. Off by default so the library never chooses silently.
    #[serde(default)]
    pub assume_defaults: bool,
}

/// A numerology request.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumerologyRequest {
    /// Name to analyse.
    pub name: Option<String>,
    /// Date of birth `YYYY-MM-DD`.
    pub date_of_birth: Option<String>,
    /// `pythagorean` (default) or `chaldean`.
    pub school: Option<String>,
    /// Keep 11/22/33 (default true; Pythagorean only).
    pub keep_master_numbers: Option<bool>,
    /// `consonant` (default) or `vowel`.
    pub y_policy: Option<String>,
    /// `reduce_parts` (default) or `total_digits`.
    pub life_path_method: Option<String>,
    /// Calendar year for the personal-year cycle.
    pub reference_year: Option<i32>,
}

/// A request, tagged by `op`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Request {
    /// Positions only.
    Positions(ChartRequest),
    /// Vedic or Western chart.
    Chart(ChartRequest),
    /// Human Design calculation.
    HumanDesign(ChartRequest),
    /// Name and date numerology.
    Numerology(NumerologyRequest),
    /// The same input under two configurations.
    Compare {
        /// Shared input.
        base: ChartRequest,
        /// Overrides for side A.
        a: ConfigIn,
        /// Overrides for side B.
        b: ConfigIn,
    },
    /// A symbolic graph of a chart.
    Graph {
        /// The chart to graph.
        chart: ChartRequest,
        /// `json`, `mermaid` or `graphml`.
        format: Option<String>,
    },
    /// Environment, providers and error catalogue.
    Info,
}

/// The engine. Holds provider instances so a kernel is read once per process.
#[derive(Default)]
pub struct Engine {
    cache: Mutex<BTreeMap<String, Arc<dyn EphemerisProvider>>>,
}

fn parse_system(s: Option<&str>) -> Result<&'static str> {
    match s.map(str::to_ascii_lowercase).as_deref() {
        Some("vedic") => Ok("vedic"),
        Some("western") => Ok("western"),
        Some(o) => Err(EshanaError::Input(format!(
            "unknown system '{o}' (vedic, western)"
        ))),
        None => Err(EshanaError::Input(
            "a chart request needs system = vedic or western".into(),
        )),
    }
}

/// Resolved moment plus the window of uncertainty around it.
struct Moment {
    center: Instant,
    half_width_days: f64,
    quality: &'static str,
}

fn resolve_when(w: &When) -> Result<Moment> {
    let fold = FoldPolicy::parse(w.fold.as_deref().unwrap_or("reject"))?;
    if let Some(u) = &w.utc {
        if w.date.is_some() || w.time.is_some() {
            return Err(EshanaError::Input(
                "give either utc or date/time, not both".into(),
            ));
        }
        return Ok(Moment {
            center: Instant::parse_utc(u)?,
            half_width_days: 0.0,
            quality: "exact",
        });
    }
    let date = w
        .date
        .as_deref()
        .ok_or_else(|| EshanaError::Input("when needs utc, or date (and time)".into()))?;
    let civil = |time: &str| -> Result<Instant> {
        match (&w.timezone, w.utc_offset_minutes) {
            (Some(z), _) => Instant::from_local_str(date, time, z, fold),
            (None, Some(o)) => Instant::from_offset_str(date, time, o),
            (None, None) => Err(EshanaError::Input(
                "local time needs timezone (IANA name) or utc_offset_minutes; ESHANA will not assume one"
                    .into(),
            )),
        }
    };
    match (&w.time, w.uncertainty_minutes) {
        (None, _) => {
            let start = civil("00:00")?;
            Ok(Moment {
                center: Instant::from_jd_utc(start.jd_ut + 0.5),
                half_width_days: 0.5,
                quality: "date_only",
            })
        }
        (Some(t), Some(u)) if u > 0.0 => Ok(Moment {
            center: civil(t)?,
            half_width_days: u / 1440.0,
            quality: "approximate",
        }),
        (Some(t), _) => Ok(Moment {
            center: civil(t)?,
            half_width_days: 0.0,
            quality: "exact",
        }),
    }
}

fn build_config(
    c: &ConfigIn,
    system: Option<&str>,
    assume: bool,
    assumptions: &mut Vec<String>,
) -> Result<CalculationConfig> {
    let provider = c.provider.clone().unwrap_or_else(|| "analytic".into());
    let zodiac = match (c.zodiac.as_deref(), system) {
        (Some("tropical"), _) => ZodiacSystem::Tropical,
        (Some("sidereal"), _) => ZodiacSystem::Sidereal,
        (Some(o), _) => {
            return Err(EshanaError::Input(format!(
                "unknown zodiac '{o}' (tropical, sidereal)"
            )))
        }
        (None, Some("vedic")) => ZodiacSystem::Sidereal,
        (None, _) => ZodiacSystem::Tropical,
    };
    let mut ayanamsha = c.ayanamsha.as_deref().map(Ayanamsha::parse).transpose()?;
    if zodiac == ZodiacSystem::Sidereal && ayanamsha.is_none() && assume {
        ayanamsha = Some(Ayanamsha::Lahiri);
        assumptions.push("ayanamsha not given; defaulted to lahiri because assume_defaults was set".into());
    }
    let house_system = match c.house_system.as_deref() {
        Some(h) => HouseSystem::parse(h)?,
        None => {
            assumptions.push("house_system not given; whole_sign used".into());
            HouseSystem::WholeSign
        }
    };
    let node_model = match c.node_model.as_deref() {
        Some("mean") => NodeModel::Mean,
        Some("true") => NodeModel::True,
        Some(o) => {
            return Err(EshanaError::Input(format!(
                "unknown node model '{o}' (mean, true)"
            )))
        }
        None => {
            assumptions.push("node_model not given; mean node used".into());
            NodeModel::Mean
        }
    };
    let cfg = CalculationConfig {
        provider,
        zodiac,
        ayanamsha,
        house_system,
        node_model,
    };
    cfg.validate()?;
    Ok(cfg)
}

fn zodiac_lon(trop: f64, cfg: &CalculationConfig, t: f64) -> f64 {
    match (cfg.zodiac, cfg.ayanamsha) {
        (ZodiacSystem::Sidereal, Some(a)) => astro::to_sidereal(trop, a, t),
        _ => norm360(trop),
    }
}

fn sign_value(lon: f64, vedic_names: bool) -> Value {
    let i = astro::sign_index(lon);
    json!({
        "index": i,
        "name": if vedic_names { RASHIS[i] } else { SIGNS[i] },
        "degree_in_sign": norm360(lon) % 30.0,
        "dms": dms_in_sign(lon),
    })
}

fn attr(id: &str, tradition: &str, rule: &str, e: Epistemic, value: Value, from: &[String]) -> Value {
    serde_json::to_value(DerivedAttribute {
        id: id.into(),
        tradition_id: tradition.into(),
        rule_version: rule.into(),
        epistemic: e,
        value,
        derived_from: from.to_vec(),
    })
    .expect("serialisable")
}

impl Engine {
    /// New engine with an empty provider cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn provider(&self, id: &str, spk: Option<&str>) -> Result<Arc<dyn EphemerisProvider>> {
        let key = format!("{id}|{}", spk.unwrap_or(""));
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| EshanaError::Provider("provider cache poisoned".into()))?;
        if let Some(p) = cache.get(&key) {
            return Ok(Arc::clone(p));
        }
        let boxed = provider_by_id(
            id,
            &ProviderOptions {
                spk_path: spk.map(String::from),
            },
        )?;
        let arc: Arc<dyn EphemerisProvider> = Arc::from(boxed);
        cache.insert(key, Arc::clone(&arc));
        Ok(arc)
    }

    /// Run a JSON request and return a JSON string: an envelope, or an error envelope with a stable code.
    #[must_use]
    pub fn execute_json(&self, request_json: &str) -> String {
        let out = serde_json::from_str::<Request>(request_json)
            .map_err(|e| EshanaError::Input(format!("bad request: {e}")))
            .and_then(|r| self.execute(&r));
        let v = match out {
            Ok(v) => v,
            Err(e) => error_envelope(&e),
        };
        serde_json::to_string(&v).expect("json")
    }

    /// Run a typed request.
    ///
    /// # Errors
    /// Any [`EshanaError`]; each has a stable code.
    pub fn execute(&self, req: &Request) -> Result<Value> {
        match req {
            Request::Positions(r) => self.chart(r, None, "positions"),
            Request::Chart(r) => {
                let s = parse_system(r.system.as_deref())?;
                self.chart(r, Some(s), "computed_chart")
            }
            Request::HumanDesign(r) => self.human_design(r),
            Request::Numerology(r) => numerology_run(r),
            Request::Compare { base, a, b } => self.compare(base, a, b),
            Request::Graph { chart, format } => {
                let s = parse_system(chart.system.as_deref())?;
                let env = self.chart(chart, Some(s), "computed_chart")?;
                graph::export(&env, format.as_deref().unwrap_or("json"))
            }
            Request::Info => Ok(self.info()),
        }
    }

    /// Environment report for `eshana doctor`.
    #[must_use]
    pub fn info(&self) -> Value {
        json!({
            "schema_version": SCHEMA_VERSION,
            "kind": "info",
            "engine_version": ENGINE_VERSION,
            "expansion": eshana_core::EXPANSION,
            "tzdb_version": eshana_time::tzdb_version(),
            "frame": FRAME_APPARENT_ECLIPTIC_OF_DATE,
            "providers": eshana_ephemeris::PROVIDER_IDS,
            "ayanamshas": Ayanamsha::ALL.iter().map(|a| a.id()).collect::<Vec<_>>(),
            "house_systems": ["whole_sign", "equal", "placidus"],
            "divisional_charts": [1, 3, 4, 7, 9, 10, 12],
            "numerology_schools": ["pythagorean", "chaldean"],
            "rulesets": [vedic::RULESET, WESTERN_RULESET, hd::RULESET,
                         numerology::School::Pythagorean.ruleset(), numerology::School::Chaldean.ruleset()],
            "error_codes": eshana_core::EshanaError::catalog().iter()
                .map(|(c, m)| json!({"code": c, "meaning": m})).collect::<Vec<_>>(),
        })
    }

    #[allow(clippy::too_many_lines)]
    fn chart(&self, r: &ChartRequest, system: Option<&str>, kind: &str) -> Result<Value> {
        let mut assumptions: Vec<String> = vec![];
        let cfg = build_config(&r.config, system, r.assume_defaults, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let mo = resolve_when(&r.when)?;
        let inst = &mo.center;
        let t = inst.t_tt();
        let hw = mo.half_width_days;
        let (lo_i, hi_i) = (inst.shifted_days(-hw), inst.shifted_days(hw));
        let sidereal = cfg.zodiac == ZodiacSystem::Sidereal;

        assumptions.push("UTC is used as UT1 (differs by under 0.9 s)".into());
        if inst.zone.is_some() {
            assumptions.push(format!(
                "civil time resolved with IANA tzdb {}",
                eshana_time::tzdb_version()
            ));
        }
        if mo.quality != "exact" {
            assumptions.push(format!(
                "birth time quality is {}: positions are evaluated over a window of ±{:.1} minutes and given as intervals",
                mo.quality,
                hw * 1440.0
            ));
        }
        assumptions.push(format!("coordinate frame: {FRAME_APPARENT_ECLIPTIC_OF_DATE}"));

        // ── bodies ──
        let wanted: Vec<Body> = match &r.bodies {
            Some(list) => list.iter().map(|s| Body::parse(s)).collect::<Result<_>>()?,
            None => {
                let mut v = Body::PLANETS.to_vec();
                v.push(cfg.node_model.body());
                v
            }
        };
        let mut positions = vec![];
        let mut pos_by_body: BTreeMap<String, (f64, f64)> = BTreeMap::new(); // id -> (zodiac lon, speed)
        for b in &wanted {
            let p = provider.position(inst.jd_tt, *b)?;
            let lon = zodiac_lon(p.longitude_deg, &cfg, t);
            let mut o = Map::new();
            o.insert("id".into(), json!(b.id()));
            o.insert("body".into(), json!(b.id()));
            o.insert("epistemic".into(), json!(Epistemic::Calculated));
            o.insert("longitude_deg".into(), json!(lon));
            o.insert("tropical_longitude_deg".into(), json!(p.longitude_deg));
            o.insert("latitude_deg".into(), json!(p.latitude_deg));
            o.insert("distance_au".into(), json!(p.distance_au));
            o.insert("speed_deg_per_day".into(), json!(p.speed_deg_per_day));
            o.insert("retrograde".into(), json!(p.is_retrograde()));
            o.insert("sign".into(), sign_value(lon, system == Some("vedic")));
            if hw > 0.0 {
                let a = provider.apparent(lo_i.jd_tt, *b)?.0;
                let c = provider.apparent(hi_i.jd_tt, *b)?.0;
                let (za, zc) = (zodiac_lon(a, &cfg, lo_i.t_tt()), zodiac_lon(c, &cfg, hi_i.t_tt()));
                o.insert("longitude_interval_deg".into(), json!([za, zc]));
            }
            pos_by_body.insert(b.id().into(), (lon, p.speed_deg_per_day));
            positions.push(Value::Object(o));
            if b.is_node() {
                let south = norm360(lon + 180.0);
                positions.push(json!({
                    "id": "south_node", "body": "south_node", "epistemic": Epistemic::Calculated,
                    "derived_from": b.id(), "longitude_deg": south,
                    "tropical_longitude_deg": norm360(p.longitude_deg + 180.0), "latitude_deg": 0.0,
                    "speed_deg_per_day": p.speed_deg_per_day, "retrograde": p.is_retrograde(),
                    "sign": sign_value(south, system == Some("vedic")),
                }));
                pos_by_body.insert("south_node".into(), (south, p.speed_deg_per_day));
            }
        }

        // ── houses ──
        let mut warnings: Vec<Value> = vec![];
        let mut houses_out = Value::Null;
        let mut cusps: Option<[f64; 12]> = None;
        let mut asc_lon: Option<f64> = None;
        if let Some(pl) = &r.place {
            if !(-90.0..=90.0).contains(&pl.latitude) || !(-180.0..=180.0).contains(&pl.longitude) {
                return Err(EshanaError::Input(
                    "latitude must be within ±90 and longitude within ±180".into(),
                ));
            }
            if hw * 2.0 * 1440.0 > 30.0 {
                warnings.push(json!({
                    "code": "ESH-TIME-UNCERTAIN",
                    "message": "Ascendant and houses omitted: the birth-time window is wider than 30 minutes, \
                                during which the Ascendant moves more than 7 degrees. ESHANA does not invent a time.",
                }));
                assumptions.push("houses omitted because the birth time is too uncertain".into());
            } else {
                let lst = astro::local_apparent_sidereal_deg(inst.gmst_deg(), pl.longitude, t);
                let off = if sidereal {
                    frames::nutation_deg(t).0 + astro::ayanamsha_deg(cfg.ayanamsha.expect("validated"), t)
                } else {
                    0.0
                };
                let mut c = astro::houses(cfg.house_system, lst, pl.latitude, t)?;
                for v in &mut c {
                    *v = norm360(*v - off);
                }
                let (asc, mc) = astro::asc_mc(lst, pl.latitude, t);
                let (asc_z, mc_z) = (norm360(asc - off), norm360(mc - off));
                if cfg.house_system == HouseSystem::WholeSign {
                    // Whole-sign houses start at the sign holding the Ascendant *in this zodiac*. Flooring the
                    // tropical Ascendant first and then subtracting the ayanamsha lands one sign early whenever
                    // the sidereal Ascendant is less than the ayanamsha past a sign boundary.
                    let start = (asc_z / 30.0).floor() * 30.0;
                    for (i, v) in c.iter_mut().enumerate() {
                        *v = norm360(start + 30.0 * i as f64);
                    }
                }
                asc_lon = Some(asc_z);
                cusps = Some(c);
                houses_out = json!({
                    "epistemic": Epistemic::Calculated,
                    "system": cfg.house_system,
                    "ascendant": {"longitude_deg": asc_z, "sign": sign_value(asc_z, system == Some("vedic"))},
                    "midheaven": {"longitude_deg": mc_z, "sign": sign_value(mc_z, system == Some("vedic"))},
                    "cusps_deg": c,
                    "local_apparent_sidereal_time_deg": lst,
                });
            }
        }

        // ── derived attributes ──
        let mut derived: Vec<Value> = vec![];
        let sign_attr = |id: &str,
                         tradition: &str,
                         rule: &str,
                         lon: f64,
                         interval: Option<(f64, f64)>,
                         vedic_names: bool,
                         from: &str|
         -> Value {
            let mut v = sign_value(lon, vedic_names);
            if let Some((a, b)) = interval {
                let (ia, ib) = (astro::sign_index(a), astro::sign_index(b));
                v["stable_within_uncertainty"] = json!(ia == ib);
                if ia != ib {
                    let names = if vedic_names { &RASHIS } else { &SIGNS };
                    v["candidates"] = json!([names[ia], names[ib]]);
                }
            }
            attr(
                id,
                tradition,
                rule,
                Epistemic::TraditionRule,
                v,
                &[from.to_string()],
            )
        };
        let interval_of = |pid: &str| -> Option<(f64, f64)> {
            positions
                .iter()
                .find(|p| p["id"] == pid)
                .and_then(|p| p.get("longitude_interval_deg"))
                .and_then(|v| Some((v[0].as_f64()?, v[1].as_f64()?)))
        };
        let mut rulesets: Vec<String> = vec![];
        if sidereal {
            rulesets.push(astro::ayanamsha_rule(cfg.ayanamsha.expect("validated")));
        }

        match system {
            Some("western") => {
                rulesets.push(WESTERN_RULESET.into());
                if sidereal {
                    assumptions
                        .push("western system with a sidereal zodiac: a Western sidereal variant".into());
                }
                for (id, (lon, _)) in &pos_by_body {
                    derived.push(sign_attr(
                        &format!("western.{id}.sign"),
                        "western.tropical",
                        WESTERN_RULESET,
                        *lon,
                        interval_of(id),
                        false,
                        id,
                    ));
                    if let Some(c) = &cusps {
                        derived.push(attr(
                            &format!("western.{id}.house"),
                            "western.tropical",
                            WESTERN_RULESET,
                            Epistemic::TraditionRule,
                            json!({"house": astro::house_of(*lon, c), "house_system": cfg.house_system}),
                            std::slice::from_ref(id),
                        ));
                    }
                }
                if let Some(a) = asc_lon {
                    derived.push(sign_attr(
                        "western.ascendant.sign",
                        "western.tropical",
                        WESTERN_RULESET,
                        a,
                        None,
                        false,
                        "houses",
                    ));
                }
                if r.aspects {
                    let scale = r.orb_scale.unwrap_or(1.0);
                    let ids: Vec<&String> = pos_by_body.keys().filter(|k| !k.contains("node")).collect();
                    for (i, a) in ids.iter().enumerate() {
                        for b in &ids[i + 1..] {
                            let (la, sa) = pos_by_body[*a];
                            let (lb, sb) = pos_by_body[*b];
                            if let Some(asp) = astro::aspect_between(la, sa, lb, sb, scale) {
                                derived.push(attr(
                                    &format!("western.aspect.{a}-{b}"), "western.ptolemaic", WESTERN_RULESET, Epistemic::TraditionRule,
                                    json!({"aspect": asp.name, "exact_angle": asp.angle, "separation_deg": asp.separation,
                                           "orb_deg": asp.orb, "applying": asp.applying, "orb_scale": scale}),
                                    &[(*a).clone(), (*b).clone()],
                                ));
                            }
                        }
                    }
                }
            }
            Some("vedic") => {
                rulesets.push(vedic::RULESET.into());
                if !sidereal {
                    return Err(EshanaError::Config(
                        "a vedic chart uses the sidereal zodiac; use system=western for tropical".into(),
                    ));
                }
                let lagna_sign = asc_lon.map(astro::sign_index);
                for (id, (lon, _)) in &pos_by_body {
                    derived.push(sign_attr(
                        &format!("vedic.{id}.rashi"),
                        "vedic.parashari",
                        vedic::RULESET,
                        *lon,
                        interval_of(id),
                        true,
                        id,
                    ));
                    let nk = vedic::nakshatra_of(*lon);
                    let nk_stable = interval_of(id).is_none_or(|(a, b)| {
                        vedic::nakshatra_of(a).index == vedic::nakshatra_of(b).index
                            && vedic::nakshatra_of(a).pada == vedic::nakshatra_of(b).pada
                    });
                    derived.push(attr(
                        &format!("vedic.{id}.nakshatra"),
                        "vedic.parashari",
                        vedic::RULESET,
                        Epistemic::TraditionRule,
                        json!({"index": nk.index, "name": nk.name, "pada": nk.pada, "lord": nk.lord,
                               "stable_within_uncertainty": nk_stable}),
                        std::slice::from_ref(id),
                    ));
                    if let Some(ls) = lagna_sign {
                        let house = (astro::sign_index(*lon) + 12 - ls) % 12 + 1;
                        derived.push(attr(
                            &format!("vedic.{id}.bhava"),
                            "vedic.parashari",
                            vedic::RULESET,
                            Epistemic::TraditionRule,
                            json!({"house": house, "method": "whole_sign_from_lagna"}),
                            &[id.clone(), "houses".into()],
                        ));
                    }
                    for d in &r.divisions {
                        let s = vedic::divisional_sign(*lon, *d)?;
                        derived.push(attr(
                            &format!("vedic.{id}.d{d}"),
                            "vedic.parashari",
                            vedic::RULESET,
                            Epistemic::TraditionRule,
                            json!({"division": d, "rashi": RASHIS[s], "index": s}),
                            std::slice::from_ref(id),
                        ));
                    }
                }
                if let Some(a) = asc_lon {
                    derived.push(sign_attr(
                        "vedic.lagna.rashi",
                        "vedic.parashari",
                        vedic::RULESET,
                        a,
                        None,
                        true,
                        "houses",
                    ));
                    let nk = vedic::nakshatra_of(a);
                    derived.push(attr(
                        "vedic.lagna.nakshatra",
                        "vedic.parashari",
                        vedic::RULESET,
                        Epistemic::TraditionRule,
                        json!({"index": nk.index, "name": nk.name, "pada": nk.pada, "lord": nk.lord}),
                        &["houses".into()],
                    ));
                    for d in &r.divisions {
                        let s = vedic::divisional_sign(a, *d)?;
                        derived.push(attr(
                            &format!("vedic.lagna.d{d}"),
                            "vedic.parashari",
                            vedic::RULESET,
                            Epistemic::TraditionRule,
                            json!({"division": d, "rashi": RASHIS[s], "index": s}),
                            &["houses".into()],
                        ));
                    }
                }
                if r.dasha {
                    if hw > 0.0 {
                        warnings.push(json!({"code": "ESH-TIME-UNCERTAIN",
                            "message": "dasha omitted: its dates depend on the Moon's exact longitude at birth"}));
                    } else {
                        let (moon, _) = *pos_by_body.get("moon").ok_or_else(|| {
                            EshanaError::Input("dasha needs the moon among the bodies".into())
                        })?;
                        let conv = r.year_convention.as_deref().unwrap_or("365.25");
                        derived.push(attr(
                            "vedic.dasha.vimshottari",
                            "vedic.parashari",
                            vedic::RULESET,
                            Epistemic::TraditionRule,
                            vedic::vimshottari(moon, inst, conv)?,
                            &["moon".into()],
                        ));
                    }
                }
            }
            _ => {}
        }

        let info = provider.info();
        let prov = Provenance {
            engine_version: ENGINE_VERSION.into(),
            provider: info.id,
            provider_version: info.version,
            provider_data: info.data,
            provider_accuracy: info.accuracy,
            tzdb_version: inst
                .zone
                .as_ref()
                .map(|_| eshana_time::tzdb_version().to_string()),
            time_model: inst.time_model.into(),
            rulesets,
            assumptions,
        };
        let place_json = r.place.as_ref().map(
            |p| json!({"latitude_deg": p.latitude, "longitude_deg": p.longitude, "altitude_m": p.altitude_m}),
        );
        Ok(json!({
            "schema_version": SCHEMA_VERSION,
            "kind": kind,
            "input": {
                "instant_utc": inst.utc,
                "jd_ut": inst.jd_ut,
                "jd_tt": inst.jd_tt,
                "delta_t_s": inst.delta_t_s,
                "civil_zone": inst.zone,
                "civil_offset_s": inst.civil_offset_s,
                "birth_time_quality": mo.quality,
                "time_window_minutes": if hw > 0.0 { json!(hw * 2.0 * 1440.0) } else { Value::Null },
                "place": place_json,
            },
            "calculation": {
                "system": system,
                "frame": FRAME_APPARENT_ECLIPTIC_OF_DATE,
                "provider": cfg.provider,
                "zodiac": cfg.zodiac,
                "ayanamsha": cfg.ayanamsha,
                "ayanamsha_deg": cfg.ayanamsha.map(|a| astro::ayanamsha_deg(a, t)),
                "house_system": cfg.house_system,
                "node_model": cfg.node_model,
            },
            "positions": positions,
            "houses": houses_out,
            "derived_attributes": derived,
            "interpretations": [],
            "warnings": warnings,
            "provenance": prov,
        }))
    }

    fn human_design(&self, r: &ChartRequest) -> Result<Value> {
        let mut assumptions: Vec<String> = vec![];
        let mut c = r.config.clone();
        if c.zodiac.as_deref().is_some_and(|z| z != "tropical") {
            return Err(EshanaError::Config(
                "Human Design uses the tropical zodiac only".into(),
            ));
        }
        c.zodiac = Some("tropical".into());
        if c.node_model.is_none() {
            c.node_model = Some("true".into());
            assumptions.push(
                "node_model not given; the true node is used for Human Design (the usual convention)".into(),
            );
        }
        let cfg = build_config(&c, None, r.assume_defaults, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let mo = resolve_when(&r.when)?;
        if mo.quality != "exact" {
            return Err(EshanaError::TimeUncertain(
                "Human Design needs the exact birth time: the design instant and gate lines move with it"
                    .into(),
            ));
        }
        let birth = mo.center;
        let sun = |i: &Instant| -> Result<f64> { Ok(provider.apparent(i.jd_tt, Body::Sun)?.0) };
        let design = hd::design_instant(&birth, &sun)?;
        let order: Vec<(&str, Option<Body>)> = vec![
            ("sun", Some(Body::Sun)),
            ("earth", None),
            ("moon", Some(Body::Moon)),
            ("north_node", Some(cfg.node_model.body())),
            ("south_node", None),
            ("mercury", Some(Body::Mercury)),
            ("venus", Some(Body::Venus)),
            ("mars", Some(Body::Mars)),
            ("jupiter", Some(Body::Jupiter)),
            ("saturn", Some(Body::Saturn)),
            ("uranus", Some(Body::Uranus)),
            ("neptune", Some(Body::Neptune)),
            ("pluto", Some(Body::Pluto)),
        ];
        let mut gates: BTreeSet<u8> = BTreeSet::new();
        let mut sides = Map::new();
        let mut sun_lines = BTreeMap::new();
        for (label, inst) in [("personality", &birth), ("design", &design)] {
            let mut acts = vec![];
            let mut sun_lon = 0.0;
            let mut node_lon = 0.0;
            for (name, body) in &order {
                let lon = match (name, body) {
                    (_, Some(b)) => provider.apparent(inst.jd_tt, *b)?.0,
                    (&"earth", None) => norm360(sun_lon + 180.0),
                    (&"south_node", None) => norm360(node_lon + 180.0),
                    _ => unreachable!(),
                };
                if *name == "sun" {
                    sun_lon = lon;
                }
                if *name == "north_node" {
                    node_lon = lon;
                }
                let a = hd::activation_at(lon);
                gates.insert(a.gate);
                if *name == "sun" {
                    sun_lines.insert(label, a.line);
                }
                acts.push(json!({"body": name, "longitude_deg": lon, "gate": a.gate, "line": a.line, "color": a.color,
                                 "center": hd::center_of(a.gate), "epistemic": Epistemic::TraditionRule}));
            }
            sides.insert(
                label.to_string(),
                json!({"instant_utc": inst.utc, "jd_ut": inst.jd_ut, "activations": acts}),
            );
        }
        let design_sun = provider.apparent(design.jd_tt, Body::Sun)?.0;
        let birth_sun = provider.apparent(birth.jd_tt, Body::Sun)?.0;
        let bg = hd::bodygraph(&gates, sun_lines["personality"], sun_lines["design"]);
        assumptions.push("UTC is used as UT1 (differs by under 0.9 s)".into());
        assumptions.push(
            "Human Design gate numbering and the centre/channel structure are implemented from public structure only; \
             branded terminology, text and diagrams are not included; legal review pending (ADR-009)"
                .into(),
        );
        let info = provider.info();
        Ok(json!({
            "schema_version": SCHEMA_VERSION,
            "kind": "human_design",
            "input": {"instant_utc": birth.utc, "jd_ut": birth.jd_ut, "civil_zone": birth.zone, "birth_time_quality": "exact",
                      "place": r.place.as_ref().map(|p| json!({"latitude_deg": p.latitude, "longitude_deg": p.longitude}))},
            "calculation": {
                "frame": FRAME_APPARENT_ECLIPTIC_OF_DATE, "provider": cfg.provider, "zodiac": "tropical",
                "node_model": cfg.node_model,
                "design_rule": "design instant: the moment the Sun was exactly 88 degrees of ecliptic longitude before its birth position (numerical root-solve, not 88 days)",
                "design_arc_deg_achieved": angular_diff(birth_sun, design_sun).abs(),
                "design_offset_days": birth.jd_ut - design.jd_ut,
                "wheel_start_deg": hd::WHEEL_START_DEG,
            },
            "positions": sides,
            "derived_attributes": [attr("human_design.bodygraph", "human_design", hd::RULESET, Epistemic::TraditionRule, bg, &["personality".into(), "design".into()])],
            "interpretations": [],
            "warnings": [],
            "provenance": Provenance {
                engine_version: ENGINE_VERSION.into(), provider: info.id, provider_version: info.version,
                provider_data: info.data, provider_accuracy: info.accuracy,
                tzdb_version: birth.zone.as_ref().map(|_| eshana_time::tzdb_version().to_string()),
                time_model: birth.time_model.into(), rulesets: vec![hd::RULESET.into()], assumptions,
            },
        }))
    }

    fn compare(&self, base: &ChartRequest, a: &ConfigIn, b: &ConfigIn) -> Result<Value> {
        let merge = |ov: &ConfigIn| -> ConfigIn {
            ConfigIn {
                provider: ov.provider.clone().or_else(|| base.config.provider.clone()),
                zodiac: ov.zodiac.clone().or_else(|| base.config.zodiac.clone()),
                ayanamsha: ov.ayanamsha.clone().or_else(|| base.config.ayanamsha.clone()),
                house_system: ov
                    .house_system
                    .clone()
                    .or_else(|| base.config.house_system.clone()),
                node_model: ov.node_model.clone().or_else(|| base.config.node_model.clone()),
            }
        };
        let (mut ra, mut rb) = (base.clone(), base.clone());
        ra.config = merge(a);
        rb.config = merge(b);
        let system = base.system.as_deref();
        let (ea, eb) = (
            self.chart(&ra, system, "computed_chart")?,
            self.chart(&rb, system, "computed_chart")?,
        );
        let mut changes: Vec<Value> = vec![];
        let by_id = |e: &Value, key: &str| -> BTreeMap<String, Value> {
            e[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|p| Some((p["id"].as_str()?.to_string(), p.clone())))
                .collect()
        };
        let (pa, pb) = (by_id(&ea, "positions"), by_id(&eb, "positions"));
        for (id, x) in &pa {
            if let Some(y) = pb.get(id) {
                let d = angular_diff(
                    y["longitude_deg"].as_f64().unwrap_or(0.0),
                    x["longitude_deg"].as_f64().unwrap_or(0.0),
                );
                if d.abs() > 1e-9 || x["sign"]["name"] != y["sign"]["name"] {
                    changes.push(json!({"kind": "position", "id": id, "a_longitude_deg": x["longitude_deg"], "b_longitude_deg": y["longitude_deg"],
                                        "difference_deg": d, "a_sign": x["sign"]["name"], "b_sign": y["sign"]["name"],
                                        "sign_changed": x["sign"]["name"] != y["sign"]["name"]}));
                }
            } else {
                changes.push(json!({"kind": "position_only_in_a", "id": id}));
            }
        }
        for id in pb.keys().filter(|k| !pa.contains_key(*k)) {
            changes.push(json!({"kind": "position_only_in_b", "id": id}));
        }
        let (da, db) = (by_id(&ea, "derived_attributes"), by_id(&eb, "derived_attributes"));
        for (id, x) in &da {
            match db.get(id) {
                Some(y) if x["value"] != y["value"] => {
                    changes.push(json!({"kind": "derived", "id": id, "a": x["value"], "b": y["value"]}));
                }
                None => changes.push(json!({"kind": "derived_only_in_a", "id": id})),
                _ => {}
            }
        }
        for id in db.keys().filter(|k| !da.contains_key(*k)) {
            changes.push(json!({"kind": "derived_only_in_b", "id": id}));
        }
        let (ca, cb) = (&ea["calculation"], &eb["calculation"]);
        let diff_keys: Vec<&str> = ["provider", "zodiac", "ayanamsha", "house_system", "node_model"]
            .into_iter()
            .filter(|k| ca[*k] != cb[*k])
            .collect();
        let sign_changes = changes.iter().filter(|c| c["sign_changed"] == true).count();
        Ok(json!({
            "schema_version": SCHEMA_VERSION,
            "kind": "method_comparison",
            "input": ea["input"],
            "a": ea["calculation"], "b": eb["calculation"],
            "configuration_differences": diff_keys,
            "summary": {"changes": changes.len(), "sign_changes": sign_changes},
            "changes": changes,
            "note": "Differences are consequences of the configuration differences listed, not software defects.",
            "provenance_a": ea["provenance"], "provenance_b": eb["provenance"],
        }))
    }
}

/// Western rule pack id and version.
pub const WESTERN_RULESET: &str = "western.basic@0.1";

fn numerology_run(r: &NumerologyRequest) -> Result<Value> {
    let school = numerology::School::parse(r.school.as_deref().unwrap_or("pythagorean"))?;
    let keep = r.keep_master_numbers.unwrap_or(true);
    let y = match r.y_policy.as_deref().unwrap_or("consonant") {
        "consonant" => numerology::YPolicy::Consonant,
        "vowel" => numerology::YPolicy::Vowel,
        o => {
            return Err(EshanaError::Rules(format!(
                "unknown y_policy '{o}' (consonant, vowel)"
            )))
        }
    };
    if r.name.is_none() && r.date_of_birth.is_none() {
        return Err(EshanaError::Input(
            "numerology needs a name, a date_of_birth, or both".into(),
        ));
    }
    let mut derived = vec![];
    if let Some(n) = &r.name {
        let v = numerology::analyze_name(
            n,
            numerology::NameOptions {
                school,
                keep_master: keep,
                y,
            },
        )?;
        derived.push(attr(
            "numerology.name",
            &format!("numerology.{}", v["school"].as_str().unwrap_or("")),
            school.ruleset(),
            Epistemic::TraditionRule,
            v,
            &["name".into()],
        ));
    }
    if let Some(d) = &r.date_of_birth {
        let parts: Vec<&str> = d.split('-').collect();
        let (y_, m_, d_) = match parts.as_slice() {
            [a, b, c] => (a.parse::<i32>(), b.parse::<u32>(), c.parse::<u32>()),
            _ => return Err(EshanaError::Input("date_of_birth must be YYYY-MM-DD".into())),
        };
        let (Ok(yy), Ok(mm), Ok(dd)) = (y_, m_, d_) else {
            return Err(EshanaError::Input("date_of_birth must be YYYY-MM-DD".into()));
        };
        chrono_check(yy, mm, dd)?;
        let method = r.life_path_method.as_deref().unwrap_or("reduce_parts");
        derived.push(attr(
            "numerology.life_path",
            "numerology.pythagorean",
            numerology::School::Pythagorean.ruleset(),
            Epistemic::TraditionRule,
            numerology::life_path(yy, mm, dd, method, keep)?,
            &["date_of_birth".into()],
        ));
        derived.push(attr(
            "numerology.birth_day",
            "numerology.pythagorean",
            numerology::School::Pythagorean.ruleset(),
            Epistemic::TraditionRule,
            json!({"day": dd, "reduced": numerology::reduce(dd, keep)}),
            &["date_of_birth".into()],
        ));
        if let Some(cy) = r.reference_year {
            derived.push(attr(
                "numerology.personal_year",
                "numerology.pythagorean",
                numerology::School::Pythagorean.ruleset(),
                Epistemic::TraditionRule,
                json!({"calendar_year": cy, "personal_year": numerology::personal_year(mm, dd, cy, keep)}),
                &["date_of_birth".into()],
            ));
        }
    }
    Ok(json!({
        "schema_version": SCHEMA_VERSION,
        "kind": "numerology_analysis",
        "input": {"name": r.name, "date_of_birth": r.date_of_birth},
        "calculation": {"school": format!("{school:?}").to_lowercase(), "keep_master_numbers": keep, "y_policy": r.y_policy.clone().unwrap_or_else(|| "consonant".into())},
        "positions": [],
        "derived_attributes": derived,
        "interpretations": [],
        "warnings": [],
        "provenance": {
            "engine_version": ENGINE_VERSION, "provider": null, "rulesets": [school.ruleset()],
            "assumptions": ["numerology is a symbolic tradition: these numbers are labelled tradition_rule, not measurements"],
        },
    }))
}

fn chrono_check(y: i32, m: u32, d: u32) -> Result<()> {
    let ok = (1..=12).contains(&m) && d >= 1 && {
        let dim = match m {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            _ => {
                if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                    29
                } else {
                    28
                }
            }
        };
        d <= dim
    };
    if ok {
        Ok(())
    } else {
        Err(EshanaError::Input(format!(
            "{y:04}-{m:02}-{d:02} is not a real calendar date"
        )))
    }
}

/// Error envelope with a stable code.
#[must_use]
pub fn error_envelope(e: &EshanaError) -> Value {
    json!({"schema_version": SCHEMA_VERSION, "kind": "error", "error": {"code": e.code(), "message": e.to_string()}})
}
