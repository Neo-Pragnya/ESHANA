//! The request/response facade. One entry point, [`Engine::execute`], takes a JSON request and returns a JSON
//! envelope. The CLI, the Python binding and any future binding are thin shells over this, which is what makes
//! cross-language parity a property of the design rather than something to re-test per language.

use crate::astro::{self, RASHIS, SIGNS};
use crate::{graph, hd, numerology, numerology_ext, vedic};
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
    /// Extra Vedic features: `panchanga`, `strength`, `relationships`, `drishti`, `ashtakavarga`, `jaimini`,
    /// `kp`, `yogas`, `sade_sati`; `all` for every one.
    #[serde(default)]
    pub features: Vec<String>,
    /// Vimśottarī depth, 1–5 (default 2: mahādaśā and antardaśā).
    pub dasha_levels: Option<usize>,
    /// Daśā systems with `dasha`: `vimshottari` (default) and `yogini`.
    #[serde(default)]
    pub dasha_systems: Vec<String>,
    /// Instant for "current" periods and transit-relative values (UTC ISO). Omitted, nothing is "current", so a
    /// result never depends on when it was computed.
    pub reference_utc: Option<String>,
    /// Sunrise convention for the pañcāṅga and special lagnas: `upper_limb_refracted` (default),
    /// `centre_refracted`, `centre_unrefracted`.
    pub sunrise: Option<String>,
    /// Sun for the special lagnas: `birth` (default) or `sunrise`.
    pub special_lagna_sun: Option<String>,
}

/// Every Vedic feature name accepted in `features`.
pub const FEATURES: [&str; 21] = [
    "panchanga",
    "strength",
    "relationships",
    "drishti",
    "ashtakavarga",
    "jaimini",
    "kp",
    "yogas",
    "sade_sati",
    "shadbala",
    "bhava_bala",
    "vimshopaka",
    "special_points",
    "aspects",
    "bhava_chalit",
    "upagrahas",
    "maitri",
    "dashas_extended",
    "auxiliary",
    "chakras",
    "annual",
];

/// An ephemeris table request.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EphemerisRequest {
    /// First instant, UTC ISO.
    pub start_utc: String,
    /// Step in days (may be fractional).
    pub step_days: f64,
    /// Number of rows, at most 20 000.
    pub count: usize,
    /// Body ids; default the ten planets and the mean node.
    pub bodies: Option<Vec<String>>,
    /// Settings (zodiac, ayanāṃśa, provider).
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path.
    pub spk_path: Option<String>,
}

/// A sign-ingress search.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IngressRequest {
    /// Body id.
    pub body: String,
    /// Range start, UTC ISO.
    pub start_utc: String,
    /// Range end, UTC ISO.
    pub end_utc: String,
    /// Settings.
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path.
    pub spk_path: Option<String>,
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
    /// Also compute every school's extended numbers (default false).
    pub extended: Option<bool>,
    /// `YYYY-MM-DD` for the personal year, month and day cycles in the extended numbers.
    pub reference_date: Option<String>,
    /// `male` or `female`, when known: picks the Kua number (and any other sex-dependent value).
    pub gender: Option<String>,
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
    /// Pañcāṅga and day divisions for a moment and place (`chart` with `place` required).
    Panchanga {
        /// Moment, place and settings (sidereal with an ayanāṃśa).
        chart: ChartRequest,
        /// Natal Moon (sidereal degrees) for Tārā bala and Candra bala.
        natal_moon_deg: Option<f64>,
    },
    /// Aṣṭakūṭa compatibility of two births (a = the role classical tables give the groom, b = the bride).
    Matching {
        /// First partner.
        a: ChartRequest,
        /// Second partner.
        b: ChartRequest,
    },
    /// Longitudes of bodies at regular steps (transit animation, progressions).
    Ephemeris(EphemerisRequest),
    /// Sign ingresses of one body in a range.
    Ingresses(IngressRequest),
    /// The built-in YAML rule packs.
    Rulesets,
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
            Request::Panchanga {
                chart,
                natal_moon_deg,
            } => self.panchanga_op(chart, *natal_moon_deg),
            Request::Matching { a, b } => self.matching_op(a, b),
            Request::Ephemeris(r) => self.ephemeris_op(r),
            Request::Ingresses(r) => self.ingress_op(r),
            Request::Rulesets => Ok(json!({
                "schema_version": SCHEMA_VERSION, "kind": "rulesets",
                "packs": crate::packs::SOURCES.iter().map(|(n, src)| json!({"file": n, "yaml": src})).collect::<Vec<_>>(),
            })),
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
            "divisional_charts": vedic::DIVISIONS,
            "vedic_features": FEATURES,
            "dasha_systems": ["vimshottari", "yogini"],
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
        if !r.features.is_empty() && system != Some("vedic") {
            return Err(EshanaError::Config(
                "features are Vedic: use them with system=vedic".into(),
            ));
        }
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
        let mut equ_by_body: BTreeMap<String, (f64, f64, f64)> = BTreeMap::new(); // id -> (RA, dec, ecl. latitude)
        let eps_true = eshana_core::rad(frames::true_obliquity_deg(t));
        let equatorial = |lam: f64, beta: f64| -> (f64, f64) {
            let (l, b) = (eshana_core::rad(lam), eshana_core::rad(beta));
            let ra = (l.sin() * eps_true.cos() - b.tan() * eps_true.sin()).atan2(l.cos());
            let dec = (b.sin() * eps_true.cos() + b.cos() * eps_true.sin() * l.sin()).asin();
            (norm360(eshana_core::deg(ra)), eshana_core::deg(dec))
        };
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
            let (ra, dec) = equatorial(p.longitude_deg, p.latitude_deg);
            o.insert("right_ascension_deg".into(), json!(ra));
            o.insert("declination_deg".into(), json!(dec));
            equ_by_body.insert(b.id().into(), (ra, dec, p.latitude_deg));
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
                    "right_ascension_deg": equatorial(norm360(p.longitude_deg + 180.0), 0.0).0,
                    "declination_deg": equatorial(norm360(p.longitude_deg + 180.0), 0.0).1,
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
        let mut mc_lon: Option<f64> = None;
        let mut angles: Option<(f64, f64)> = None; // (local apparent sidereal time, zodiac offset) for KP cusps
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
                mc_lon = Some(mc_z);
                angles = Some((lst, off));
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
                            json!({"division": d, "rashi": RASHIS[s], "index": s,
                                   "degree_in_varga": (norm360(*lon) % 30.0 * f64::from(*d)) % 30.0}),
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
                            json!({"division": d, "rashi": RASHIS[s], "index": s,
                                   "degree_in_varga": (norm360(a) % 30.0 * f64::from(*d)) % 30.0}),
                            &["houses".into()],
                        ));
                    }
                }
                let reference_jd = r
                    .reference_utc
                    .as_deref()
                    .map(|u| Instant::parse_utc(u).map(|i| i.jd_ut))
                    .transpose()?;
                if r.dasha {
                    if hw > 0.0 {
                        warnings.push(json!({"code": "ESH-TIME-UNCERTAIN",
                            "message": "dasha omitted: its dates depend on the Moon's exact longitude at birth"}));
                    } else {
                        let (moon, _) = *pos_by_body.get("moon").ok_or_else(|| {
                            EshanaError::Input("dasha needs the moon among the bodies".into())
                        })?;
                        let conv = r.year_convention.as_deref().unwrap_or("365.25");
                        let levels = r.dasha_levels.unwrap_or(2);
                        let systems: Vec<&str> = if r.dasha_systems.is_empty() {
                            vec!["vimshottari"]
                        } else {
                            r.dasha_systems.iter().map(String::as_str).collect()
                        };
                        for sys in systems {
                            let v = match sys {
                                "vimshottari" => {
                                    vedic::vimshottari_levels(moon, inst, conv, levels, reference_jd)?
                                }
                                "yogini" => vedic::yogini(moon, inst, conv, levels, reference_jd)?,
                                other => {
                                    return Err(EshanaError::Input(format!(
                                        "unknown dasha system '{other}' (vimshottari, yogini)"
                                    )))
                                }
                            };
                            derived.push(attr(
                                &format!("vedic.dasha.{sys}"),
                                "vedic.parashari",
                                vedic::RULESET,
                                Epistemic::TraditionRule,
                                v,
                                &["moon".into()],
                            ));
                        }
                    }
                }
                let feats = expand_features(&r.features)?;
                if !feats.is_empty() {
                    self.vedic_features(
                        &feats,
                        &cfg,
                        provider.as_ref(),
                        inst,
                        r,
                        &pos_by_body,
                        &equ_by_body,
                        (asc_lon, mc_lon, angles),
                        hw,
                        reference_jd,
                        &mut derived,
                        &mut warnings,
                        &mut rulesets,
                    )?;
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

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    fn vedic_features(
        &self,
        feats: &BTreeSet<&str>,
        cfg: &CalculationConfig,
        provider: &dyn EphemerisProvider,
        inst: &Instant,
        r: &ChartRequest,
        pos_by_body: &BTreeMap<String, (f64, f64)>,
        equ: &BTreeMap<String, (f64, f64, f64)>,
        (asc, mc, angles): (Option<f64>, Option<f64>, Option<(f64, f64)>),
        hw: f64,
        reference_jd: Option<f64>,
        derived: &mut Vec<Value>,
        warnings: &mut Vec<Value>,
        rulesets: &mut Vec<String>,
    ) -> Result<()> {
        use crate::{bala, events, jyotish, packs, panchanga, vedic};
        let vc = jyotish::VChart::from_positions(pos_by_body, asc, mc);
        let core = packs::core();
        let core_rule = packs::rule_id(&core.id, &core.version);
        let mut push = |id: &str, rule: &str, v: Value, from: &[&str]| {
            derived.push(attr(
                id,
                "vedic.parashari",
                rule,
                Epistemic::TraditionRule,
                v,
                &from.iter().map(|s| (*s).to_string()).collect::<Vec<_>>(),
            ));
        };
        if hw > 0.0 {
            warnings.push(json!({"code": "ESH-TIME-UNCERTAIN",
                "message": "features are computed at the centre of the birth-time window; lagna-based values are omitted"}));
        }
        let mut omitted: Vec<String> = vec![];
        let mut need = |what: &str, why: &str| omitted.push(format!("{what} omitted: {why}"));
        rulesets.push(core_rule.clone());
        if feats.contains("strength") {
            push(
                "vedic.strength",
                &core_rule,
                jyotish::strength(&vc),
                &["positions", "houses"],
            );
        }
        if feats.contains("relationships") {
            push(
                "vedic.relationships",
                &core_rule,
                jyotish::panchadha_maitri(&vc),
                &["positions"],
            );
        }
        if feats.contains("drishti") {
            push(
                "vedic.drishti.graha",
                &core_rule,
                jyotish::graha_drishti(&vc),
                &["positions"],
            );
            push(
                "vedic.drishti.additional",
                &core_rule,
                jyotish::additional_drishti(&vc),
                &["positions"],
            );
            push(
                "vedic.drishti.rashi",
                &core_rule,
                json!({"rule": "movable signs aspect fixed signs except the adjacent one; fixed aspect movable except the adjacent one; dual aspect dual",
                "aspects": jyotish::rashi_drishti(&vc)}),
                &["positions"],
            );
        }
        if feats.contains("ashtakavarga") {
            let pk = packs::ashtakavarga();
            let rule = packs::rule_id(&pk.id, &pk.version);
            match jyotish::ashtakavarga(&vc) {
                Some(v) => {
                    rulesets.push(rule.clone());
                    push("vedic.ashtakavarga", &rule, v, &["positions", "houses"]);
                }
                None => need(
                    "ashtakavarga",
                    "the lagna contributes bindus, so an exact birth time and place are needed",
                ),
            }
        }
        if feats.contains("jaimini") {
            push(
                "vedic.jaimini.chara_karakas",
                &core_rule,
                jyotish::chara_karakas(&vc),
                &["positions"],
            );
            push(
                "vedic.jaimini.rashi_drishti",
                &core_rule,
                jyotish::rashi_drishti(&vc),
                &["positions"],
            );
            match jyotish::arudha_padas(&vc) {
                Some(v) => push(
                    "vedic.jaimini.arudha_padas",
                    &core_rule,
                    v,
                    &["positions", "houses"],
                ),
                None => need("arudha padas", "they are counted from the lagna"),
            }
            // pada navāṃśa: the same ārūḍha rule applied inside the navāṃśa chart (signs from D9)
            if let Some(v) = jyotish::arudha_padas(&jyotish::varga_chart(&vc, 9)) {
                push(
                    "vedic.jaimini.arudha_padas_d9",
                    &core_rule,
                    v,
                    &["positions", "houses"],
                );
            }
        }
        let rising = panchanga::Rising::parse(r.sunrise.as_deref())?;
        let site = r.place.as_ref().map(|pl| panchanga::Site {
            lat: pl.latitude,
            lon: pl.longitude,
            offset_s: inst
                .civil_offset_s
                .map_or(pl.longitude / 15.0 * 3600.0, f64::from),
            rising,
            lagna_sun_at_birth: r.special_lagna_sun.as_deref() != Some("sunrise"),
        });
        let ayan = cfg.ayanamsha.expect("vedic charts are sidereal");
        let mut day_lord: Option<String> = None;
        let mut pc_value: Option<Value> = None;
        if feats.contains("panchanga")
            || feats.contains("kp")
            || feats.contains("shadbala")
            || feats.contains("bhava_bala")
            || feats.contains("upagrahas")
            || feats.contains("auxiliary")
            || feats.contains("chakras")
            || feats.contains("annual")
        {
            match site {
                Some(site) => {
                    let pc = panchanga::panchanga(provider, inst, site, ayan, None)?;
                    day_lord = pc["vara"]["lord"].as_str().map(String::from);
                    pc_value = Some(pc.clone());
                    if feats.contains("panchanga") {
                        let pk = packs::panchanga();
                        let rule = packs::rule_id(&pk.id, &pk.version);
                        rulesets.push(rule.clone());
                        push("vedic.panchanga", &rule, pc, &["sun", "moon", "place"]);
                    }
                }
                None => need("panchanga", "sunrise and the day divisions need a place"),
            }
        }
        if feats.contains("kp") {
            match (angles, r.place.as_ref()) {
                (Some((lst, off)), Some(pl)) if hw == 0.0 => {
                    match astro::houses(HouseSystem::Placidus, lst, pl.latitude, inst.t_tt()) {
                        Ok(mut cusps) => {
                            for c in &mut cusps {
                                *c = norm360(*c - off);
                            }
                            let mut v = jyotish::kp(&vc, &cusps);
                            if let Some(dl) = &day_lord {
                                v["ruling_planets"] = jyotish::ruling_planets(&vc, dl).unwrap_or(Value::Null);
                            }
                            if cfg.ayanamsha != Some(Ayanamsha::Krishnamurti) {
                                v["note"] = json!("KP is normally used with the krishnamurti ayanamsha; this chart uses another one");
                            }
                            v["significator_table"] = crate::annual::kp_significator_table(&vc, &v);
                            push("vedic.kp", &core_rule, v, &["positions", "houses"]);
                        }
                        Err(e) => need("kp", &e.to_string()),
                    }
                }
                _ => need("kp", "KP cusps need an exact birth time and a place"),
            }
        }
        if feats.contains("yogas") {
            let pk = packs::yogas();
            let rule = packs::rule_id(&pk.id, &pk.version);
            rulesets.push(rule.clone());
            push(
                "vedic.yogas",
                &rule,
                jyotish::yogas(&vc),
                &["positions", "houses"],
            );
        }
        let need_shad = feats.contains("shadbala") || feats.contains("bhava_bala");
        if need_shad
            || feats.contains("vimshopaka")
            || feats.contains("special_points")
            || feats.contains("aspects")
            || feats.contains("bhava_chalit")
            || feats.contains("upagrahas")
        {
            let bp = packs::bala();
            let bala_rule = packs::rule_id(&bp.id, &bp.version);
            rulesets.push(bala_rule.clone());
            if need_shad {
                let mut sky = bala::Sky {
                    jd: inst.jd_ut,
                    vara_lord: day_lord.clone(),
                    ..Default::default()
                };
                for (id, (_, dec, lat)) in equ {
                    let key = match id.as_str() {
                        "mean_node" | "true_node" => "rahu",
                        "south_node" => "ketu",
                        o => o,
                    };
                    sky.decl.insert(key.into(), *dec);
                    sky.ecl_lat.insert(key.into(), *lat);
                }
                if let Some(pc) = &pc_value {
                    let jd = |v: &Value| v["jd_ut"].as_f64();
                    if let (Some(a), Some(b), Some(c)) =
                        (jd(&pc["sunrise"]), jd(&pc["sunset"]), jd(&pc["next_sunrise"]))
                    {
                        sky.day = Some((a, b, c));
                    }
                    sky.hora_lord = pc["horas"]
                        .as_array()
                        .and_then(|h| h.iter().find(|x| x["current"] == true))
                        .and_then(|x| x["lord"].as_str())
                        .map(String::from);
                }
                if let (Some(pl), Some(&(ra_sun, _, _))) = (r.place.as_ref(), equ.get("sun")) {
                    let last = astro::local_apparent_sidereal_deg(inst.gmst_deg(), pl.longitude, inst.t_tt());
                    sky.sun_hour_angle = Some(eshana_core::norm180(last - ra_sun));
                }
                if let Some(site) = site {
                    // abda and masa lords: weekday lords on the local dates of the Sun's last entries into Mesha and
                    // into its current sign (convention named in bala.yaml)
                    let ing =
                        events::ingresses(provider, Body::Sun, inst.jd_ut - 400.0, inst.jd_ut, Some(ayan))
                            .unwrap_or_default();
                    let wd = |jd: f64| {
                        ((jd + site.offset_s / 86_400.0 + 1.5).floor() as i64).rem_euclid(7) as usize
                    };
                    let lord_of_day = |jd: f64| packs::panchanga().varas[wd(jd)].lord.clone();
                    if let Some(e) = ing
                        .iter()
                        .rev()
                        .find(|e| e["to_sign"] == 0)
                        .and_then(|e| e["jd_ut"].as_f64())
                    {
                        sky.abda_lord = Some(lord_of_day(e));
                    }
                    if let Some(e) = ing.last().and_then(|e| e["jd_ut"].as_f64()) {
                        sky.masa_lord = Some(lord_of_day(e));
                    }
                }
                let shad = bala::shadbala(&vc, &sky);
                if feats.contains("bhava_bala") {
                    match bala::bhava_bala(&vc, &shad) {
                        Some(bb) => push("vedic.bhava_bala", &bala_rule, bb, &["positions", "houses"]),
                        None => need("bhava bala", "the bhavas need an exact birth time and place"),
                    }
                }
                if feats.contains("shadbala") {
                    let mut v = shad;
                    v["sky"] = json!({"vara_lord": sky.vara_lord, "hora_lord": sky.hora_lord, "abda_lord": sky.abda_lord,
                                      "masa_lord": sky.masa_lord, "sun_hour_angle_deg": sky.sun_hour_angle});
                    push(
                        "vedic.shadbala",
                        &bala_rule,
                        v,
                        &["positions", "houses", "panchanga"],
                    );
                }
            }
            if feats.contains("vimshopaka") {
                push(
                    "vedic.vimshopaka",
                    &bala_rule,
                    bala::vimshopaka(&vc),
                    &["positions"],
                );
            }
            if feats.contains("upagrahas") {
                let mut u = Map::new();
                let one = |l: f64| json!({"longitude_deg": l, "sign": astro::sign_index(l)});
                if let Some(sun) = vc.lon.get("sun") {
                    for (n, l) in bala::sun_upagrahas(*sun) {
                        u.insert(n.into(), one(l));
                    }
                }
                let mut gm_note = "Gulika and Mandi need the place and the day's sunrise and sunset";
                if let (Some(pc), Some(pl)) = (&pc_value, r.place.as_ref()) {
                    let jd = |v: &Value| v["jd_ut"].as_f64();
                    if let (Some(a), Some(b), Some(c), Some(wd)) = (
                        jd(&pc["sunrise"]),
                        jd(&pc["sunset"]),
                        jd(&pc["next_sunrise"]),
                        pc["vara"]["index"].as_u64(),
                    ) {
                        let (g, m, by_day) = bala::gulika_mandi_times(inst.jd_ut, a, b, c, wd as usize);
                        let asc_at = |jd_ut: f64| -> f64 {
                            let at = Instant::from_jd_utc(jd_ut);
                            let t = at.t_tt();
                            let lst = astro::local_apparent_sidereal_deg(at.gmst_deg(), pl.longitude, t);
                            norm360(
                                astro::asc_mc(lst, pl.latitude, t).0
                                    - frames::nutation_deg(t).0
                                    - astro::ayanamsha_deg(ayan, t),
                            )
                        };
                        for (n, t) in [("gulika", g), ("mandi", m)] {
                            let mut v = one(asc_at(t));
                            v["rises_jd_ut"] = json!(t);
                            u.insert(n.into(), v);
                        }
                        u.insert("birth_in".into(), json!(if by_day { "day" } else { "night" }));
                        gm_note = "Gulika: the lagna at the start of Saturn's eighth of the day or night (eighths ruled in weekday order from the day's lord; at night from the fifth weekday's lord). Mandi: the lagna at the classical ghati for the weekday (day 26, 22, 18, 14, 10, 6, 2; night 10, 6, 2, 26, 22, 18, 14 from Sunday), scaled to that day or night";
                    }
                }
                u.insert("conventions".into(), json!({"sun_based": "BPHS: Dhuma = Sun + 133 deg 20 min; Vyatipata = 360 - Dhuma; Parivesha = Vyatipata + 180; Indrachapa = 360 - Parivesha; Upaketu = Indrachapa + 16 deg 40 min", "gulika_mandi": gm_note}));
                push(
                    "vedic.upagrahas",
                    &core_rule,
                    Value::Object(u),
                    &["sun", "place", "panchanga"],
                );
            }
            if feats.contains("special_points") {
                push(
                    "vedic.special_points",
                    &core_rule,
                    bala::special_points(&vc),
                    &["positions", "houses"],
                );
            }
            if feats.contains("aspects") {
                // D9 positions are navamsa longitudes: the D9 sign plus the degree within it (deg_in_sign x 9)
                let nav = |l: f64| -> Option<f64> {
                    vedic::divisional_sign(l, 9)
                        .ok()
                        .map(|s| s as f64 * 30.0 + (l.rem_euclid(30.0) * 9.0) % 30.0)
                };
                let mut d9 = vc.clone();
                for (g, l) in &vc.lon {
                    if let Some(x) = nav(*l) {
                        d9.lon.insert(g.clone(), x);
                    }
                }
                let mut v = json!({"d1": bala::aspect_matrix(&vc), "d9": bala::aspect_matrix(&d9),
                    "note": "fraction: whole-sign graha drishti (3rd/10th 1/4, 5th/9th 1/2, 4th/8th 3/4, 7th and special aspects full); virupa: sputa drishti (0-60) of D1 longitudes, and of navamsa longitudes for D9; the nodes have no sputa drishti"});
                if let (Some(a), Some(m)) = (asc, mc) {
                    if hw == 0.0 {
                        let madhya = bala::bhava_madhyas(a, m);
                        let mut pts: Vec<(String, f64)> = vec![("lagna".into(), a)];
                        pts.extend(madhya.iter().enumerate().map(|(k, x)| ((k + 1).to_string(), *x)));
                        v["houses"] = json!({"d1": bala::aspects_on_points(&vc, &pts)});
                        v["house_and_lord"] = json!({"d1": bala::aspects_house_and_lord(&vc, &madhya)});
                        v["house_madhya"] = json!({"d1": madhya.to_vec()});
                        if let Some(a9) = nav(a) {
                            let mut m9 = [0.0; 12];
                            for (k, x) in m9.iter_mut().enumerate() {
                                *x = norm360(a9 + 30.0 * k as f64);
                            }
                            let mut p9: Vec<(String, f64)> = vec![("lagna".into(), a9)];
                            p9.extend(m9.iter().enumerate().map(|(k, x)| ((k + 1).to_string(), *x)));
                            v["houses"]["d9"] = bala::aspects_on_points(&d9, &p9);
                            v["house_and_lord"]["d9"] = bala::aspects_house_and_lord(&d9, &m9);
                            v["house_madhya"]["d9"] = json!(m9.to_vec());
                        }
                        v["house_conventions"] = json!("D1 houses: Sripati bhava madhyas (Porphyry trisection); D9 houses: equal houses from the navamsa lagna degree; house + lord: mean of the virupas on the madhya and on the house's sign lord (the madhya alone when the aspecting graha is that lord)");
                    }
                }
                push("vedic.aspects", &bala_rule, v, &["positions", "houses"]);
            }
            if feats.contains("bhava_chalit") {
                match (asc, mc) {
                    (Some(a), Some(m)) if hw == 0.0 => {
                        let madhya = bala::bhava_madhyas(a, m);
                        let mut cusps = [0.0; 12];
                        for (k, v) in cusps.iter_mut().enumerate() {
                            let prev = madhya[(k + 11) % 12];
                            *v = norm360(prev + norm360(madhya[k] - prev) / 2.0);
                        }
                        let placed: BTreeMap<String, usize> = vc
                            .lon
                            .iter()
                            .map(|(g, l)| (g.clone(), astro::house_of(*l, &cusps)))
                            .collect();
                        push(
                            "vedic.bhava_chalit",
                            &core_rule,
                            json!({"system": "sripati", "bhava_madhya": madhya,
                            "bhava_sandhi": cusps, "grahas": placed}),
                            &["positions", "houses"],
                        );
                    }
                    _ => need("bhava chalit", "it needs an exact birth time and place"),
                }
            }
        }
        if feats.contains("sade_sati") {
            let moon = vc
                .lon
                .get("moon")
                .copied()
                .ok_or_else(|| EshanaError::Input("sade_sati needs the moon".into()))?;
            let info = provider.info();
            let start = inst.jd_ut.max(info.jd_min + 1.0);
            let end = (inst.jd_ut + 100.0 * 365.25).min(info.jd_max - 1.0);
            if end > start {
                let mut v = events::sade_sati(provider, jyotish::sign_of(moon), start, end, ayan)?;
                if let Some(at) = reference_jd {
                    let at_s = Instant::from_jd_utc(at).utc;
                    v["active_at_reference"] = json!(v["windows"].as_array().is_some_and(|w| w
                        .iter()
                        .any(|x| x["start"].as_str().is_some_and(|a| a <= at_s.as_str())
                            && x["end"].as_str().is_some_and(|b| at_s.as_str() < b))));
                }
                if end < inst.jd_ut + 100.0 * 365.25 - 1.0 {
                    v["note"] = json!("range clipped to the provider's limits");
                }
                push("vedic.sade_sati", &core_rule, v, &["moon", "saturn"]);
            }
        }
        // analysis modules (crate::analysis): each returns its own attributes with their rule ids
        {
            use crate::analysis::{AnalysisCtx, Compute};
            let actx = AnalysisCtx {
                vc: &vc,
                provider,
                inst,
                place: r.place.as_ref().map(|p| (p.latitude, p.longitude)),
                offset_s: inst.civil_offset_s.map(f64::from),
                ayanamsha: ayan,
                panchanga: pc_value.as_ref(),
                reference_jd,
                equ,
            };
            let modules: [(&str, Compute); 5] = [
                ("maitri", crate::maitri::compute),
                ("dashas_extended", crate::dashas_ext::compute),
                ("auxiliary", crate::auxiliary::compute),
                ("chakras", crate::chakras::compute),
                ("annual", crate::annual::compute),
            ];
            for (feat, f) in modules {
                if feats.contains(feat) {
                    for d in f(&actx)? {
                        rulesets.push(d.rule.clone());
                        let from: Vec<&str> = d.from.iter().map(String::as_str).collect();
                        push(&d.id, &d.rule, d.value, &from);
                    }
                }
            }
        }
        for m in omitted {
            warnings.push(json!({"code": "ESH-INPUT", "message": m}));
        }
        Ok(())
    }

    fn panchanga_op(&self, chart: &ChartRequest, natal_moon: Option<f64>) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(
            &chart.config,
            Some("vedic"),
            chart.assume_defaults,
            &mut assumptions,
        )?;
        let provider = self.provider(&cfg.provider, chart.spk_path.as_deref())?;
        let mo = resolve_when(&chart.when)?;
        let pl = chart
            .place
            .as_ref()
            .ok_or_else(|| EshanaError::Input("panchanga needs a place".into()))?;
        let offset_s = mo.center.civil_offset_s.map_or_else(
            || {
                assumptions
                    .push("no civil zone: the local date uses mean solar time at the longitude".into());
                pl.longitude / 15.0 * 3600.0
            },
            f64::from,
        );
        let rising = crate::panchanga::Rising::parse(chart.sunrise.as_deref())?;
        let site = crate::panchanga::Site {
            lat: pl.latitude,
            lon: pl.longitude,
            offset_s,
            rising,
            lagna_sun_at_birth: chart.special_lagna_sun.as_deref() != Some("sunrise"),
        };
        let ayan = cfg
            .ayanamsha
            .ok_or_else(|| EshanaError::Config("panchanga needs an ayanamsha".into()))?;
        let v = crate::panchanga::panchanga(provider.as_ref(), &mo.center, site, ayan, natal_moon)?;
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "panchanga", "input": {"instant_utc": mo.center.utc,
                  "place": {"latitude_deg": pl.latitude, "longitude_deg": pl.longitude}},
                  "calculation": {"ayanamsha": ayan, "provider": cfg.provider},
                  "panchanga": v, "interpretations": [],
                  "provenance": {"engine_version": ENGINE_VERSION, "assumptions": assumptions,
                                 "rulesets": [crate::packs::rule_id(&crate::packs::panchanga().id, &crate::packs::panchanga().version)]}}),
        )
    }

    fn matching_op(&self, a: &ChartRequest, b: &ChartRequest) -> Result<Value> {
        let mut out = vec![];
        for side in [a, b] {
            let mut r = side.clone();
            r.system = Some("vedic".into());
            r.features = vec!["yogas".into()];
            let env = self.chart(&r, Some("vedic"), "computed_chart")?;
            out.push(env);
        }
        let moon = |e: &Value| -> Result<f64> {
            e["positions"]
                .as_array()
                .and_then(|p| p.iter().find(|x| x["id"] == "moon"))
                .and_then(|m| m["longitude_deg"].as_f64())
                .ok_or_else(|| EshanaError::Input("matching needs both Moons".into()))
        };
        let manglik = |e: &Value| -> Value {
            e["derived_attributes"]
                .as_array()
                .and_then(|d| d.iter().find(|x| x["id"] == "vedic.yogas"))
                .map(|y| {
                    json!(y["value"]["doshas"].as_array().map(|ds| ds
                        .iter()
                        .filter(|x| x["category"] == "manglik")
                        .map(|x| json!({"id": x["id"], "present": x["present"]}))
                        .collect::<Vec<_>>()))
                })
                .unwrap_or(Value::Null)
        };
        let k = crate::matching::ashtakoota(moon(&out[0])?, moon(&out[1])?);
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "matching", "ashtakoota": k,
                  "manglik": {"a": manglik(&out[0]), "b": manglik(&out[1])},
                  "charts": {"a": out[0], "b": out[1]}, "interpretations": []}),
        )
    }

    fn ephemeris_op(&self, r: &EphemerisRequest) -> Result<Value> {
        if r.count == 0 || r.count > 20_000 || !(r.step_days.is_finite() && r.step_days > 0.0) {
            return Err(EshanaError::Input(
                "count must be 1-20000 and step_days positive".into(),
            ));
        }
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, None, false, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let bodies: Vec<Body> = match &r.bodies {
            Some(l) => l.iter().map(|s| Body::parse(s)).collect::<Result<_>>()?,
            None => {
                let mut v = Body::PLANETS.to_vec();
                v.push(cfg.node_model.body());
                v
            }
        };
        let start = Instant::parse_utc(&r.start_utc)?;
        let ayan = if cfg.zodiac == ZodiacSystem::Sidereal {
            cfg.ayanamsha
        } else {
            None
        };
        let rows = crate::events::series(
            provider.as_ref(),
            &bodies,
            start.jd_ut,
            r.step_days,
            r.count,
            ayan,
        )?;
        Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "ephemeris",
                  "calculation": {"zodiac": cfg.zodiac, "ayanamsha": cfg.ayanamsha, "provider": cfg.provider},
                  "rows": rows, "interpretations": []}))
    }

    fn ingress_op(&self, r: &IngressRequest) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, None, false, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let body = Body::parse(&r.body)?;
        let (a, b) = (Instant::parse_utc(&r.start_utc)?, Instant::parse_utc(&r.end_utc)?);
        if b.jd_ut <= a.jd_ut || b.jd_ut - a.jd_ut > 200.0 * 366.0 {
            return Err(EshanaError::Input(
                "end must be after start and within 200 years".into(),
            ));
        }
        let ayan = if cfg.zodiac == ZodiacSystem::Sidereal {
            cfg.ayanamsha
        } else {
            None
        };
        let ev = crate::events::ingresses(provider.as_ref(), body, a.jd_ut, b.jd_ut, ayan)?;
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "ingresses", "body": body.id(),
                  "calculation": {"zodiac": cfg.zodiac, "ayanamsha": cfg.ayanamsha, "provider": cfg.provider},
                  "ingresses": ev, "interpretations": []}),
        )
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
                acts.push(json!({"body": name, "longitude_deg": lon, "gate": a.gate, "line": a.line, "color": a.color, "tone": a.tone, "base": a.base,
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
    let gender = match r.gender.as_deref() {
        None => None,
        Some(g @ ("male" | "female")) => Some(g),
        Some(o) => return Err(EshanaError::Rules(format!("unknown gender '{o}' (male, female)"))),
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
    if r.extended.unwrap_or(false) {
        let ymd = |d: &str| -> Option<(i32, u32, u32)> {
            let p: Vec<&str> = d.split('-').collect();
            match p.as_slice() {
                [a, b, c] => Some((a.parse().ok()?, b.parse().ok()?, c.parse().ok()?)),
                _ => None,
            }
        };
        let dob = r.date_of_birth.as_deref().and_then(ymd);
        let reference = r.reference_date.as_deref().and_then(ymd);
        let name = r.name.as_deref();
        let src: Vec<String> = [
            r.name.as_ref().map(|_| "name".to_string()),
            dob.map(|_| "date_of_birth".to_string()),
        ]
        .into_iter()
        .flatten()
        .collect();
        let srcs: Vec<&str> = src.iter().map(String::as_str).collect();
        let owned: Vec<String> = srcs.iter().map(|x| (*x).to_string()).collect();
        derived.push(attr(
            "numerology.pythagorean.extended",
            "numerology.pythagorean",
            "numerology.extended@0.1",
            Epistemic::TraditionRule,
            numerology_ext::pythagorean(name, dob, reference, y, keep),
            &owned,
        ));
        derived.push(attr(
            "numerology.chaldean.extended",
            "numerology.chaldean",
            "numerology.extended@0.1",
            Epistemic::TraditionRule,
            numerology_ext::chaldean(name, dob),
            &owned,
        ));
        derived.push(attr(
            "numerology.vedic",
            "numerology.vedic",
            "numerology.vedic@0.1",
            Epistemic::TraditionRule,
            numerology_ext::vedic(name, dob),
            &owned,
        ));
        if let Some(d) = dob {
            derived.push(attr(
                "numerology.lo_shu",
                "numerology.lo_shu",
                "numerology.lo_shu@0.1",
                Epistemic::TraditionRule,
                {
                    let mut v = numerology_ext::lo_shu(d);
                    if let Some(g) = gender {
                        // the Kua for this person's sex; both stay in the value
                        let k = v["kua"][g].clone();
                        let grp = v["kua"][format!("{g}_group")].clone();
                        v["kua"]["gender"] = json!(g);
                        v["kua"]["number"] = k;
                        v["kua"]["group"] = grp;
                    }
                    v
                },
                &["date_of_birth".into()],
            ));
        }
    }
    Ok(json!({
        "schema_version": SCHEMA_VERSION,
        "kind": "numerology_analysis",
        "input": {"name": r.name, "date_of_birth": r.date_of_birth, "gender": gender},
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

/// Expand `all` and check every feature name.
fn expand_features(list: &[String]) -> Result<BTreeSet<&'static str>> {
    let mut out = BTreeSet::new();
    for f in list {
        if f == "all" {
            out.extend(FEATURES);
            continue;
        }
        let known = FEATURES.iter().find(|x| **x == f.as_str()).ok_or_else(|| {
            EshanaError::Input(format!(
                "unknown feature '{f}' (one of: {}, all)",
                FEATURES.join(", ")
            ))
        })?;
        out.insert(*known);
    }
    Ok(out)
}
