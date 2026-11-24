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
use eshana_time::{Calendar, FoldPolicy, Instant};
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
    /// Calendar of `date`: `gregorian` (proleptic, ISO 8601; the default), `julian`, or `auto` (Julian before
    /// 1582-10-15). Years are astronomical: `-0043` is 44 BCE.
    pub calendar: Option<String>,
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
    /// Positions as seen from the place (parallax applied; up to ~1° for the Moon) instead of from the Earth's
    /// centre. Needs `place`. Default false.
    pub topocentric: Option<bool>,
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
    /// KP cusp occupancy (`kp_cusp_occupancy` family): `cusp_start` (default) or `cusp_midpoint`.
    pub kp_cusp_occupancy: Option<String>,
    /// Further divisional charts by method id (`varga_method` family, e.g. `d81_navanavamsha`), Vedic only.
    #[serde(default)]
    pub varga_methods: Vec<String>,
}

/// Every operation (`op`) a request may name. A unit test keeps this list and the [`Request`] enum in step.
pub const OPERATIONS: [&str; 28] = [
    "positions",
    "chart",
    "human_design",
    "numerology",
    "compare",
    "graph",
    "panchanga",
    "matching",
    "ephemeris",
    "ingresses",
    "stations",
    "crossings",
    "rulesets",
    "methods",
    "ashtakavarga_transits",
    "muhurta_windows",
    "personal_calendar",
    "pair_calendar",
    "progressions",
    "kp_table",
    "kp_horary",
    "rise_set",
    "returns",
    "gochara",
    "eclipses",
    "synthetic_chart",
    "research_window_score",
    "info",
];

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

/// Rise, transit and set of bodies over local days at a place (AST-08).
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiseSetRequest {
    /// The observer (latitude, longitude, altitude).
    pub place: Place,
    /// First local date, `YYYY-MM-DD`.
    pub date: String,
    /// IANA zone for the local days.
    pub timezone: Option<String>,
    /// Fixed offset in minutes east, when no zone is known.
    pub utc_offset_minutes: Option<i32>,
    /// Number of days (1–366). Default 1.
    pub days: Option<u32>,
    /// Body ids. Default sun and moon.
    pub bodies: Option<Vec<String>>,
    /// `upper_limb_refracted` (default), `centre_refracted` or `centre_unrefracted`.
    pub sunrise: Option<String>,
    /// Include civil, nautical and astronomical twilight. Default true.
    pub twilight: Option<bool>,
    /// Settings (provider).
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path or catalogue name.
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

/// Stations and retrograde intervals of one body in a range.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationRequest {
    /// Body id.
    pub body: String,
    /// Range start, UTC ISO (included).
    pub start_utc: String,
    /// Range end, UTC ISO (excluded).
    pub end_utc: String,
    /// Solver tolerance in seconds (default 1; 0.01 to 3600). Not the ephemeris accuracy.
    pub tolerance_seconds: Option<f64>,
    /// Settings: the zodiac picks the frame the speed is measured in.
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path.
    pub spk_path: Option<String>,
}

/// When one body reaches a longitude, or an aspect to it, in a range.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrossingRequest {
    /// Body id.
    pub body: String,
    /// The point, in degrees of the requested zodiac (for example a natal Moon).
    pub point_deg: f64,
    /// Aspect angles in degrees (default `[0]`, the conjunction).
    pub aspects: Option<Vec<f64>>,
    /// Range start, UTC ISO (included).
    pub start_utc: String,
    /// Range end, UTC ISO (excluded).
    pub end_utc: String,
    /// Solver tolerance in seconds (default 1; 0.01 to 3600).
    pub tolerance_seconds: Option<f64>,
    /// Report closest approaches that stop within this many degrees without crossing (default 0.1; 0 turns off).
    pub touch_orb_deg: Option<f64>,
    /// At most this many events (default 10000); `truncated` says when there were more.
    pub max_results: Option<usize>,
    /// Settings.
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path.
    pub spk_path: Option<String>,
}

/// Muhūrta windows: factor timelines over a range composed under a profile and/or constraints.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MuhurtaRequest {
    /// The event place (where the muhūrta happens), not a birth place.
    pub place: Place,
    /// Range start, UTC ISO (included).
    pub start_utc: String,
    /// Range end, UTC ISO (excluded); at most 62 days after the start.
    pub end_utc: String,
    /// IANA zone of the event place, for local times (DST and historic offsets per instant).
    pub timezone: Option<String>,
    /// Fixed offset in minutes east, when no zone is known (otherwise mean solar time at the longitude).
    pub utc_offset_minutes: Option<i32>,
    /// A profile id (`muhurta_profile` family).
    pub profile: Option<String>,
    /// Profile constraints to switch off, by id.
    #[serde(default)]
    pub disable: Vec<String>,
    /// Extra constraints: ids from the pack, or inline `{factor, include | exclude}`.
    #[serde(default)]
    pub constraints: Vec<Value>,
    /// Factors to report on every window even when no constraint names them.
    #[serde(default)]
    pub show: Vec<String>,
    /// Sunrise convention.
    pub sunrise: Option<String>,
    /// Sidereal settings (an ayanāṃśa is required).
    #[serde(default)]
    pub config: ConfigIn,
    /// SPK kernel path.
    pub spk_path: Option<String>,
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
    /// Stations and retrograde intervals of one body in a range.
    Stations(StationRequest),
    /// Crossings of a longitude (and aspects to it) by one body in a range.
    Crossings(CrossingRequest),
    /// The built-in YAML rule packs.
    Rulesets,
    /// The method registry: every family and method id the engine implements.
    Methods,
    /// Transit Aṣṭakavarga: the natal BAV/SAV cells each graha reads, as `[start, end)` segments over a range.
    AshtakavargaTransits {
        /// The natal chart (exact time and place, sidereal settings); the transits use the same settings.
        natal: ChartRequest,
        /// Range start, UTC ISO (included).
        start_utc: String,
        /// Range end, UTC ISO (excluded).
        end_utc: String,
        /// Body ids (default the seven grahas Sun–Saturn); a node is answered as unavailable.
        bodies: Option<Vec<String>>,
    },
    /// Muhūrta windows: composed factor timelines over a range.
    MuhurtaWindows(MuhurtaRequest),
    /// A personal calendar: muhūrta windows with tārā and candra bala from a birth chart, day summaries, exports.
    PersonalCalendar {
        /// The birth (its place is the birth place, never the event place).
        natal: ChartRequest,
        /// The event place, zone, range and constraints.
        muhurta: MuhurtaRequest,
        /// `json` (default), `csv` or `ics`: the export is returned in `content`.
        format: Option<String>,
    },
    /// A pair calendar: windows that suit both people (each one's tārā and candra bala) and the event constraints.
    PairCalendar {
        /// First person (the role classical tables give the groom).
        a: ChartRequest,
        /// Second person.
        b: ChartRequest,
        /// The event place, zone, range and constraints (default profile pair_general when none is given).
        muhurta: MuhurtaRequest,
        /// `json`, `csv` or `ics`.
        format: Option<String>,
    },
    /// Secondary progressions: a day after birth for each year of life, with angles under a named method.
    Progressions {
        /// The birth (an exact time and a place for angles); its settings pick the zodiac.
        natal: ChartRequest,
        /// The moment of life to progress to, UTC ISO.
        at_utc: String,
        /// `progression_angles` method id; without it the angles are unavailable.
        angle_method: Option<String>,
        /// Days per year of life (default the tropical year, 365.24219).
        year_days: Option<f64>,
    },
    /// Vedic factors derived from hand-placed sidereal positions; no ephemeris is consulted.
    SyntheticChart {
        /// The placed bodies.
        positions: Vec<crate::synthetic::Placed>,
        /// A placed sidereal Ascendant (optional; whole-sign houses and lagna-based factors need it).
        ascendant_deg: Option<f64>,
    },
    /// Experimental research model `research.window_score@0.1` over muhūrta windows (see docs/design/research-models.md).
    ResearchWindowScore {
        /// Windows as a `muhurta_windows`, `personal_calendar` or `pair_calendar` result gives them.
        windows: Value,
    },
    /// The KP 1–249 horary number table.
    KpTable,
    /// A KP horary chart from a number (1–249) and the moment and place of judgement.
    KpHorary {
        /// The querent's number, 1–249: fixes the Ascendant at the start of its arc.
        number: u32,
        /// Moment and place of judgement (not the number), with settings.
        chart: ChartRequest,
    },
    /// Environment, providers and error catalogue.
    Info,
    /// Rise, upper transit and set of bodies, and twilight, for local days at a place.
    RiseSet(RiseSetRequest),
    /// Solar and lunar eclipses in a range, with type, time of greatest eclipse and magnitude.
    Eclipses {
        /// Range start, UTC ISO.
        start_utc: String,
        /// Range end, UTC ISO (at most 200 years).
        end_utc: String,
        /// `solar`, `lunar` (default both).
        kinds: Option<Vec<String>>,
        /// Settings (provider).
        #[serde(default)]
        config: ConfigIn,
        /// SPK kernel path or catalogue name.
        spk_path: Option<String>,
    },
    /// Gochara: transits counted from the natal Moon and lagna, with retrograde and combustion intervals.
    Gochara {
        /// The birth (sidereal settings).
        natal: ChartRequest,
        /// Range start, UTC ISO.
        start_utc: String,
        /// Range end, UTC ISO (at most 30 years).
        end_utc: String,
        /// Graha ids (default the nine grahas).
        bodies: Option<Vec<String>>,
    },
    /// Every return of a body to its natal longitude in a range (solar and lunar returns), with a chart at each.
    Returns {
        /// The birth; its settings pick the zodiac (tropical, or sidereal with an ayanāṃśa).
        natal: ChartRequest,
        /// `sun` (default), `moon` or another body.
        body: Option<String>,
        /// Range start, UTC ISO.
        start_utc: String,
        /// Range end, UTC ISO.
        end_utc: String,
        /// Where the return chart is cast (a relocated return); default the birth place.
        place: Option<Place>,
        /// Include a chart at each return (default true).
        charts: Option<bool>,
    },
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

/// Solar arc directions (WES-05): every natal point moved by the solar arc (progressed minus natal Sun), in the natal
/// chart's own zodiac, and the directed-to-natal Ptolemaic aspects within 1°. The angles follow when the birth chart
/// has them.
fn solar_arc_directions(birth: &Value, arc: f64) -> Value {
    let mut points: Vec<(String, f64)> = birth["positions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| Some((p["id"].as_str()?.to_string(), p["longitude_deg"].as_f64()?)))
        .collect();
    for (id, key) in [("ascendant", "ascendant"), ("midheaven", "midheaven")] {
        if let Some(l) = birth["houses"][key]["longitude_deg"].as_f64() {
            points.push((id.into(), l));
        }
    }
    let directed: Vec<Value> = points
        .iter()
        .map(|(id, l)| json!({"id": id, "natal_deg": l, "directed_deg": norm360(l + arc)}))
        .collect();
    let mut aspects = vec![];
    for (did, dl) in &points {
        for (nid, nl) in &points {
            let sep = angular_diff(norm360(dl + arc), *nl).abs();
            for (name, ang) in [
                ("conjunction", 0.0),
                ("sextile", 60.0),
                ("square", 90.0),
                ("trine", 120.0),
                ("opposition", 180.0),
            ] {
                let orb = (sep - ang).abs();
                if orb <= 1.0 {
                    aspects.push(json!({"directed": did, "natal": nid, "aspect": name, "orb_deg": orb}));
                }
            }
        }
    }
    json!({"arc_deg": arc, "rule": "directed = natal + (progressed Sun − natal Sun); aspects directed-to-natal within 1°", "points": directed, "aspects": aspects})
}

/// Resolved moment plus the window of uncertainty around it.
struct Moment {
    center: Instant,
    half_width_days: f64,
    quality: &'static str,
    /// How the date was read (calendar conversions, the pre-1582 reading), for the assumptions.
    notes: Vec<String>,
}

fn resolve_when(w: &When) -> Result<Moment> {
    let fold = FoldPolicy::parse(w.fold.as_deref().unwrap_or("reject"))?;
    if let Some(u) = &w.utc {
        if w.date.is_some() || w.time.is_some() {
            return Err(EshanaError::Input(
                "give either utc or date/time, not both".into(),
            ));
        }
        if w.calendar.is_some() {
            return Err(EshanaError::Input(
                "calendar applies to a local date; a utc instant is always proleptic Gregorian (ISO 8601)"
                    .into(),
            ));
        }
        return Ok(Moment {
            center: Instant::parse_utc(u)?,
            half_width_days: 0.0,
            quality: "exact",
            notes: vec![],
        });
    }
    let date = w
        .date
        .as_deref()
        .ok_or_else(|| EshanaError::Input("when needs utc, or date (and time)".into()))?;
    let cal = match w.calendar.as_deref() {
        Some(c) => Calendar::parse(c)?,
        None => Calendar::Gregorian,
    };
    let mut notes = vec![];
    // no silent historical reading: a date before the Gregorian reform without a calendar says how it was read
    let (g, conv) = eshana_time::parse_date_in(date, cal)?;
    if w.calendar.is_none() && (i32::from(g.year()), g.month(), g.day()) < (1582, 10, 15) {
        notes.push(format!(
            "{date} read as proleptic Gregorian (ISO 8601); give calendar = julian or auto for a date written in the Julian calendar"
        ));
    }
    if let Some(c) = conv {
        notes.push(format!(
            "calendar {}: {c}",
            w.calendar.as_deref().unwrap_or("gregorian")
        ));
    }
    let civil = |time: &str| -> Result<Instant> {
        match (&w.timezone, w.utc_offset_minutes) {
            (Some(z), _) => Instant::from_local_str_in(date, time, z, fold, cal).map(|x| x.0),
            (None, Some(o)) => Instant::from_offset_str_in(date, time, o, cal).map(|x| x.0),
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
                notes,
            })
        }
        (Some(t), Some(u)) if u > 0.0 => Ok(Moment {
            center: civil(t)?,
            half_width_days: u / 1440.0,
            quality: "approximate",
            notes,
        }),
        (Some(t), _) => Ok(Moment {
            center: civil(t)?,
            half_width_days: 0.0,
            quality: "exact",
            notes,
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
            Request::RiseSet(r) => self.rise_set_op(r),
            Request::Eclipses {
                start_utc,
                end_utc,
                kinds,
                config,
                spk_path,
            } => {
                let mut assumptions = vec![];
                let cfg = build_config(config, None, false, &mut assumptions)?;
                let provider = self.provider(&cfg.provider, spk_path.as_deref())?;
                let (a, b) = (Instant::parse_utc(start_utc)?, Instant::parse_utc(end_utc)?);
                if b.jd_ut <= a.jd_ut || b.jd_ut - a.jd_ut > 200.0 * 366.0 {
                    return Err(EshanaError::RangeInvalid(
                        "end must be after start and within 200 years".into(),
                    ));
                }
                let k: Vec<String> = kinds
                    .clone()
                    .unwrap_or_else(|| vec!["solar".into(), "lunar".into()]);
                if let Some(bad) = k.iter().find(|x| *x != "solar" && *x != "lunar") {
                    return Err(EshanaError::Input(format!(
                        "unknown eclipse kind '{bad}' (solar, lunar)"
                    )));
                }
                let list = crate::eclipses::eclipses(
                    provider.as_ref(),
                    a.jd_ut,
                    b.jd_ut,
                    k.iter().any(|x| x == "solar"),
                    k.iter().any(|x| x == "lunar"),
                )?;
                Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "eclipses",
                    "input": {"start_utc": a.utc, "end_utc": b.utc, "kinds": k},
                    "calculation": {"provider": cfg.provider, "method": "geometric: minimum Sun–Moon (or Moon–shadow) distance near each syzygy; shadow radii 1.02 × (π☾ + π☉ ± s☉)"},
                    "eclipses": list,
                    "provenance": {"engine_version": env!("CARGO_PKG_VERSION"), "provider_data": provider.info().data,
                        "assumptions": ["times are of greatest eclipse, geocentric", "solar types use a spherical Earth for γ (central when γ < 1)"]},
                    "interpretations": []}))
            }
            Request::Gochara {
                natal,
                start_utc,
                end_utc,
                bodies,
            } => self.gochara_op(natal, start_utc, end_utc, bodies.as_deref()),
            Request::Returns {
                natal,
                body,
                start_utc,
                end_utc,
                place,
                charts,
            } => self.returns_op(
                natal,
                body.as_deref(),
                start_utc,
                end_utc,
                place.as_ref(),
                charts.unwrap_or(true),
            ),
            Request::Stations(r) => self.stations_op(r),
            Request::Crossings(r) => self.crossings_op(r),
            Request::Rulesets => Ok(json!({
                "schema_version": SCHEMA_VERSION, "kind": "rulesets",
                "packs": crate::packs::SOURCES.iter().map(|(n, src)| json!({"file": n, "yaml": src})).collect::<Vec<_>>(),
            })),
            Request::Methods => Ok(crate::methods::listing()),
            Request::MuhurtaWindows(r) => self.muhurta_op(r),
            Request::PersonalCalendar {
                natal,
                muhurta,
                format,
            } => self.personal_calendar_op(natal, muhurta, format.as_deref()),
            Request::PairCalendar {
                a,
                b,
                muhurta,
                format,
            } => self.pair_calendar_op(a, b, muhurta, format.as_deref()),
            Request::Progressions {
                natal,
                at_utc,
                angle_method,
                year_days,
            } => self.progressions_op(natal, at_utc, angle_method.as_deref(), *year_days),
            Request::SyntheticChart {
                positions,
                ascendant_deg,
            } => {
                let v = crate::synthetic::derive(positions, *ascendant_deg)?;
                Ok(
                    json!({"schema_version": SCHEMA_VERSION, "kind": "synthetic_chart",
                    "calculation": {"zodiac": "sidereal", "positions_source": "synthetic", "ephemeris": null},
                    "synthetic": v, "interpretations": [],
                    "provenance": {"engine_version": ENGINE_VERSION, "rulesets": [crate::packs::rule_id(&crate::packs::core().id, &crate::packs::core().version)],
                        "assumptions": ["positions are synthetic: placed by the user, not computed from an ephemeris"],
                        "positions_source": "synthetic"}}),
                )
            }
            Request::ResearchWindowScore { windows } => Ok(json!({"schema_version": SCHEMA_VERSION,
                "kind": "research_window_score", "model": crate::research::WINDOW_SCORE, "status": "experimental",
                "epistemic": "research_hypothesis", "specification": "docs/design/research-models.md",
                "weights": crate::research::WEIGHTS.iter().map(|(k, w)| json!({"factor": k, "weight": w})).collect::<Vec<_>>(),
                "ranked": crate::research::rank(windows)?, "interpretations": [],
                "provenance": {"engine_version": ENGINE_VERSION, "rulesets": [crate::research::WINDOW_SCORE], "assumptions": [
                    "a research hypothesis: weights are stated, not validated; eligibility is decided by the constraints, not by this score"]}})),
            Request::KpTable => Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "kp_table",
                "table": crate::kp::table_json(), "interpretations": [],
                "provenance": {"engine_version": ENGINE_VERSION, "rulesets": [], "assumptions": [],
                  "construction": "each nakshatra divided among the nine Vimshottari lords by daśā years from its own lord; subs cut by a sign boundary are two numbers"}})),
            Request::KpHorary { number, chart } => self.kp_horary_op(*number, chart),
            Request::AshtakavargaTransits {
                natal,
                start_utc,
                end_utc,
                bodies,
            } => self.av_transits_op(natal, start_utc, end_utc, bodies.as_deref()),
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
            "ayanamsha_definitions": Ayanamsha::ALL.iter().map(|a| astro::ayanamsha_definition(*a)).collect::<Vec<_>>(),
            "house_systems": ["whole_sign", "equal", "placidus"],
            "divisional_charts": vedic::DIVISIONS,
            "vedic_features": FEATURES,
            "dasha_systems": ["vimshottari", "yogini"],
            "numerology_schools": ["pythagorean", "chaldean"],
            "rulesets": [vedic::RULESET, WESTERN_RULESET, hd::RULESET,
                         numerology::School::Pythagorean.ruleset(), numerology::School::Chaldean.ruleset()],
            "operations": OPERATIONS,
            "method_families": crate::methods::registry().iter().map(|(f, _)| *f).collect::<Vec<_>>(),
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
        assumptions.extend(mo.notes.iter().cloned());
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
        let topo = r.config.topocentric.unwrap_or(false);
        if topo && r.place.is_none() {
            return Err(EshanaError::Input("topocentric positions need place".into()));
        }
        if topo {
            assumptions.push("positions are topocentric: parallax for the place's latitude and altitude applied (Meeus ch. 40); speeds stay geocentric".into());
        }
        for b in &wanted {
            let mut p = provider.position(inst.jd_tt, *b)?;
            if let (true, Some(pl), Some(dist)) = (topo, r.place.as_ref(), p.distance_au) {
                let obs = crate::rising::Observer {
                    lat: pl.latitude,
                    lon: pl.longitude,
                    height_m: pl.altitude_m.unwrap_or(0.0),
                };
                let (l, bt) =
                    crate::rising::topocentric(p.longitude_deg, p.latitude_deg, dist, inst.jd_ut, obs);
                p.longitude_deg = l;
                p.latitude_deg = bt;
            }
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
                if !r.varga_methods.is_empty() {
                    let mut lons: Vec<(String, f64)> =
                        pos_by_body.iter().map(|(k, v)| (k.clone(), v.0)).collect();
                    if let (Some(a), true) = (asc_lon, hw == 0.0) {
                        lons.push(("lagna".into(), a));
                    }
                    let v = crate::vargas_ext::charts(&r.varga_methods, &lons)?;
                    derived.push(attr(
                        "vedic.varga_methods",
                        "vedic.parashari",
                        "vargas.extended@0.1",
                        Epistemic::TraditionRule,
                        v,
                        &["positions".to_string(), "houses".to_string()],
                    ));
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
                // the date as written and its calendar, when a calendar was named (TIME-03)
                "calendar": r.when.calendar.as_deref().map(|c| json!({"name": c, "date_as_written": r.when.date})),
            },
            "calculation": {
                "system": system,
                "frame": FRAME_APPARENT_ECLIPTIC_OF_DATE,
                "positions_reference": if r.config.topocentric.unwrap_or(false) { "topocentric" } else { "geocentric" },
                "provider": cfg.provider,
                "zodiac": cfg.zodiac,
                "ayanamsha": cfg.ayanamsha,
                "ayanamsha_deg": cfg.ayanamsha.map(|a| astro::ayanamsha_deg(a, t)),
                "house_system": cfg.house_system,
                "node_model": cfg.node_model,
                "config_hash": crate::methods::config_hash(&cfg),
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
            // the same three kinds of drishti inside each requested varga, cast between the varga's signs
            let vargas: Vec<Value> = r
                .divisions
                .iter()
                .filter(|d| **d > 1)
                .map(|&d| {
                    let v = jyotish::varga_chart(&vc, d);
                    json!({"division": d, "graha": jyotish::graha_drishti(&v),
                        "additional": jyotish::additional_drishti(&v), "rashi": jyotish::rashi_drishti(&v)})
                })
                .collect();
            if !vargas.is_empty() {
                push(
                    "vedic.drishti.vargas",
                    &core_rule,
                    json!({"rule": "graha, rashi and additional drishti by the same rules as the rasi chart, between the signs each graha occupies in the varga", "vargas": vargas}),
                    &["positions"],
                );
            }
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
                            let occ = crate::kp::Occupancy::parse(r.kp_cusp_occupancy.as_deref())?;
                            let mut v = jyotish::kp_with(&vc, &cusps, occ);
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
                // a varga's positions are varga longitudes: the varga sign plus the degree within it (deg_in_sign x d),
                // as for the navamsa; D9 always, and every other requested division
                let varga_lon = |l: f64, d: u32| -> Option<f64> {
                    vedic::divisional_sign(l, d)
                        .ok()
                        .map(|s| s as f64 * 30.0 + (l.rem_euclid(30.0) * f64::from(d)) % 30.0)
                };
                let mut divs: Vec<u32> = r.divisions.iter().copied().filter(|d| *d > 1).collect();
                divs.push(9);
                divs.sort_unstable();
                divs.dedup();
                let vcharts: Vec<(u32, jyotish::VChart)> = divs
                    .iter()
                    .map(|&d| {
                        let mut c = vc.clone();
                        c.lon = vc
                            .lon
                            .iter()
                            .filter_map(|(g, l)| varga_lon(*l, d).map(|x| (g.clone(), x)))
                            .collect();
                        (d, c)
                    })
                    .collect();
                let mut v = json!({"d1": bala::aspect_matrix(&vc),
                    "divisions": std::iter::once(1).chain(divs.iter().copied()).collect::<Vec<u32>>(),
                    "note": "fraction: whole-sign graha drishti (3rd/10th 1/4, 5th/9th 1/2, 4th/8th 3/4, 7th and special aspects full); virupa: sputa drishti (0-60) of D1 longitudes, and of varga longitudes (varga sign plus the degree in the sign times the division) for each varga; the nodes have no sputa drishti"});
                for (d, c) in &vcharts {
                    v[format!("d{d}")] = bala::aspect_matrix(c);
                }
                if let (Some(a), Some(m)) = (asc, mc) {
                    if hw == 0.0 {
                        let madhya = bala::bhava_madhyas(a, m);
                        let mut pts: Vec<(String, f64)> = vec![("lagna".into(), a)];
                        pts.extend(madhya.iter().enumerate().map(|(k, x)| ((k + 1).to_string(), *x)));
                        v["houses"] = json!({"d1": bala::aspects_on_points(&vc, &pts)});
                        v["house_and_lord"] = json!({"d1": bala::aspects_house_and_lord(&vc, &madhya)});
                        v["house_madhya"] = json!({"d1": madhya.to_vec()});
                        for (d, c) in &vcharts {
                            let Some(ad) = varga_lon(a, *d) else { continue };
                            let mut md = [0.0; 12];
                            for (k, x) in md.iter_mut().enumerate() {
                                *x = norm360(ad + 30.0 * k as f64);
                            }
                            let mut pd: Vec<(String, f64)> = vec![("lagna".into(), ad)];
                            pd.extend(md.iter().enumerate().map(|(k, x)| ((k + 1).to_string(), *x)));
                            let key = format!("d{d}");
                            v["houses"][&key] = bala::aspects_on_points(c, &pd);
                            v["house_and_lord"][&key] = bala::aspects_house_and_lord(c, &md);
                            v["house_madhya"][&key] = json!(md.to_vec());
                        }
                        v["house_conventions"] = json!("D1 houses: Sripati bhava madhyas (Porphyry trisection); varga houses (D9 and the others): equal houses from the varga lagna degree; house + lord: mean of the virupas on the madhya and on the house's sign lord (the madhya alone when the aspecting graha is that lord)");
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
        let supp = self.matching_supplementary(a, b, &out[0], &out[1], moon(&out[0])?, moon(&out[1])?)?;
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "matching", "ashtakoota": k,
                  "manglik": {"a": manglik(&out[0]), "b": manglik(&out[1])},
                  "supplementary": supp,
                  "charts": {"a": out[0], "b": out[1]}, "interpretations": []}),
        )
    }

    /// Factors beside the 36 points, in their own section; the Aṣṭakūṭa above is computed exactly as before.
    fn matching_supplementary(
        &self,
        a: &ChartRequest,
        b: &ChartRequest,
        ea: &Value,
        eb: &Value,
        moon_a: f64,
        moon_b: f64,
    ) -> Result<Value> {
        use crate::matching_ext as mx;
        let av = |side: &ChartRequest| -> Result<Option<Value>> {
            let mut r = side.clone();
            r.system = Some("vedic".into());
            r.features = vec!["ashtakavarga".into()];
            r.divisions = vec![];
            r.dasha = false;
            let env = self.chart(&r, Some("vedic"), "computed_chart")?;
            Ok(env["derived_attributes"]
                .as_array()
                .and_then(|d| d.iter().find(|x| x["id"] == "vedic.ashtakavarga"))
                .map(|x| x["value"].clone()))
        };
        let (sa, sb) = (mx::signs_of(ea), mx::signs_of(eb));
        let (ma, mb) = (mx::manglik(&sa), mx::manglik(&sb));
        let pair = mx::manglik_pair(&ma, &mb);
        let (ava, avb) = (av(a)?, av(b)?);
        Ok(json!({
            "rule": mx::rule(),
            "note": "reported beside the 36-point Aṣṭakūṭa; none of these changes its score",
            "moon_factors": mx::moon_factors(moon_a, moon_b),
            "manglik_trace": {"a": ma, "b": mb, "pair": pair,
                "reference_points": ["lagna", "moon", "venus"]},
            "pair_ashtakavarga": {"a_in_b": mx::pair_ashtakavarga(&sa, avb.as_ref()), "b_in_a": mx::pair_ashtakavarga(&sb, ava.as_ref())},
            "cross_aspects": {"a_onto_b": mx::cross_aspects(&sa, &sb), "b_onto_a": mx::cross_aspects(&sb, &sa)},
        }))
    }

    fn av_transits_op(
        &self,
        natal: &ChartRequest,
        start: &str,
        end: &str,
        bodies: Option<&[String]>,
    ) -> Result<Value> {
        let (a, b) = (Instant::parse_utc(start)?, Instant::parse_utc(end)?);
        check_span(a.jd_ut, b.jd_ut, 50.0)?;
        let mut r = natal.clone();
        r.system = Some("vedic".into());
        r.features = vec!["ashtakavarga".into()];
        r.divisions = vec![];
        r.dasha = false;
        let env = self.chart(&r, Some("vedic"), "computed_chart")?;
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, Some("vedic"), r.assume_defaults, &mut assumptions)?;
        let ayan = cfg
            .ayanamsha
            .ok_or_else(|| EshanaError::Config("dated Aṣṭakavarga needs a sidereal ayanamsha".into()))?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let natal_av = env["derived_attributes"]
            .as_array()
            .and_then(|d| d.iter().find(|x| x["id"] == "vedic.ashtakavarga"))
            .map(|x| x["value"].clone());
        let default: Vec<String> = crate::jyotish::SEVEN.iter().map(|s| (*s).to_string()).collect();
        let list = bodies.map_or(default, <[String]>::to_vec);
        let series = crate::dated::series(
            provider.as_ref(),
            &list,
            a.jd_ut,
            b.jd_ut,
            ayan,
            natal_av.as_ref(),
        )?;
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "ashtakavarga_transits",
            "input": {"natal_utc": env["input"]["instant_utc"], "start_utc": a.utc, "end_utc": b.utc},
            "calculation": {"ayanamsha": ayan, "provider": cfg.provider, "config_hash": crate::methods::config_hash(&cfg),
                "rule": "a transiting graha reads its own natal BAV cell and the SAV cell of the sign it occupies; values change only at sign ingresses",
                "range": "[start, end)"},
            "natal_sarvashtakavarga": natal_av.as_ref().map(|v| v["sarvashtakavarga"].clone()),
            "grahas": series, "interpretations": [],
            "provenance": {"engine_version": ENGINE_VERSION, "rulesets": env["provenance"]["rulesets"],
                           "assumptions": env["provenance"]["assumptions"]}}),
        )
    }

    fn muhurta_op(&self, r: &MuhurtaRequest) -> Result<Value> {
        self.muhurta_core(r, None, None)
    }

    #[allow(clippy::too_many_lines)]
    fn pair_calendar_op(
        &self,
        a: &ChartRequest,
        b: &ChartRequest,
        m: &MuhurtaRequest,
        format: Option<&str>,
    ) -> Result<Value> {
        let moon_of = |side: &ChartRequest| -> Result<(f64, Value)> {
            let mut r = side.clone();
            r.system = Some("vedic".into());
            r.features = vec![];
            r.divisions = vec![];
            r.dasha = false;
            r.config.zodiac = Some("sidereal".into());
            r.config.ayanamsha = m.config.ayanamsha.clone().or(r.config.ayanamsha);
            let env = self.chart(&r, Some("vedic"), "computed_chart")?;
            let moon = env["positions"]
                .as_array()
                .and_then(|p| p.iter().find(|x| x["id"] == "moon"))
                .and_then(|x| x["longitude_deg"].as_f64())
                .ok_or_else(|| EshanaError::Input("a pair calendar needs both Moons".into()))?;
            Ok((
                moon,
                json!({"instant_utc": env["input"]["instant_utc"], "place": env["input"]["place"], "natal_moon_deg": moon}),
            ))
        };
        let (mb, info_b) = moon_of(b)?;
        let mut m = m.clone();
        if m.profile.is_none() && m.constraints.is_empty() {
            m.profile = Some("pair_general".into());
        }
        let mut v = self.personal_calendar_op_inner(a, &m, format, Some(mb))?;
        v["kind"] = json!("pair_calendar");
        v["input"]["partner"] = info_b;
        v["input"]["partner"]["place_role"] = json!("birth");
        // Explain every window by both people's factors and the event factors.
        if let Some(ws) = v["windows"].as_array_mut() {
            for w in ws {
                let f = w["factors"].clone();
                let pick = |k: &str| f.get(k).cloned().unwrap_or(Value::Null);
                w["explained"] = json!({
                    "a": {"tara_bala": pick("tara_bala"), "chandra_bala": pick("chandra_bala")},
                    "b": {"tara_bala": pick("tara_bala_b"), "chandra_bala": pick("chandra_bala_b")},
                    "event": f.as_object().map(|o| o.iter().filter(|(k, _)| !k.starts_with("tara_bala") && !k.starts_with("chandra_bala"))
                        .map(|(k, x)| (k.clone(), x.clone())).collect::<serde_json::Map<_, _>>()),
                });
            }
        }
        Ok(v)
    }

    fn personal_calendar_op(
        &self,
        natal: &ChartRequest,
        m: &MuhurtaRequest,
        format: Option<&str>,
    ) -> Result<Value> {
        self.personal_calendar_op_inner(natal, m, format, None)
    }

    #[allow(clippy::too_many_lines)]
    fn personal_calendar_op_inner(
        &self,
        natal: &ChartRequest,
        m: &MuhurtaRequest,
        format: Option<&str>,
        partner_moon: Option<f64>,
    ) -> Result<Value> {
        let fmt = format.unwrap_or("json");
        if !["json", "csv", "ics"].contains(&fmt) {
            return Err(EshanaError::Input(format!(
                "format must be json, csv or ics, not '{fmt}'"
            )));
        }
        // The natal Moon in the calendar's own zodiac: the natal chart is computed with the calendar's ayanāṃśa.
        let mut nr = natal.clone();
        nr.system = Some("vedic".into());
        nr.features = vec!["ashtakavarga".into()];
        nr.divisions = vec![];
        nr.dasha = false;
        nr.config.zodiac = Some("sidereal".into());
        nr.config.ayanamsha = m.config.ayanamsha.clone().or(nr.config.ayanamsha);
        let env = self.chart(&nr, Some("vedic"), "computed_chart")?;
        let natal_moon = env["positions"]
            .as_array()
            .and_then(|p| p.iter().find(|x| x["id"] == "moon"))
            .and_then(|x| x["longitude_deg"].as_f64())
            .ok_or_else(|| EshanaError::Input("the natal chart has no Moon".into()))?;
        let mut v = self.muhurta_core(m, Some(natal_moon), partner_moon)?;
        // Day summaries in the event zone: sunrise, windows that start that day, the slow grahas' transit bindus.
        let (a, b) = (Instant::parse_utc(&m.start_utc)?, Instant::parse_utc(&m.end_utc)?);
        let ayan = Ayanamsha::ALL
            .iter()
            .copied()
            .find(|x| Some(x.id()) == v["calculation"]["ayanamsha"].as_str())
            .ok_or_else(|| EshanaError::Config("calendar needs an ayanamsha".into()))?;
        let provider = self.provider(
            v["calculation"]["provider"].as_str().unwrap_or("analytic"),
            m.spk_path.as_deref(),
        )?;
        let natal_av = env["derived_attributes"]
            .as_array()
            .and_then(|d| d.iter().find(|x| x["id"] == "vedic.ashtakavarga"))
            .map(|x| x["value"].clone());
        let transits = crate::dated::series(
            provider.as_ref(),
            &["jupiter".to_string(), "saturn".to_string()],
            a.jd_ut,
            b.jd_ut,
            ayan,
            natal_av.as_ref(),
        )?;
        let off_at = |jd: f64| -> Result<f64> {
            Ok(match (&m.timezone, m.utc_offset_minutes) {
                (Some(z), _) => f64::from(eshana_time::zone_offset_seconds(z, jd)?),
                (None, Some(mm)) => f64::from(mm) * 60.0,
                (None, None) => m.place.longitude / 15.0 * 3600.0,
            })
        };
        let mut days = vec![];
        let mut d = ((a.jd_ut + off_at(a.jd_ut)? / 86_400.0) - 0.5).floor() + 0.5; // local midnight, local JD
        while d - off_at(d)? / 86_400.0 < b.jd_ut {
            let utc0 = d - off_at(d)? / 86_400.0;
            let utc1 = (d + 1.0) - off_at(d + 1.0)? / 86_400.0;
            let date = Instant::from_jd_utc(d).utc[..10].to_string();
            let site = crate::panchanga::Site {
                lat: m.place.latitude,
                lon: m.place.longitude,
                offset_s: off_at(utc0)?,
                rising: crate::panchanga::Rising::parse(m.sunrise.as_deref())?,
                lagna_sun_at_birth: true,
            };
            let day = crate::panchanga::vedic_day(provider.as_ref(), (utc0 + utc1) / 2.0, site)?;
            let sunrise = match &day {
                Some(dd) if dd.sunrise >= utc0 - 0.5 => json!(crate::muhurta::local_text(
                    dd.sunrise,
                    site,
                    m.timezone.as_deref()
                )?),
                _ => crate::methods::unavailable(
                    "the Sun does not rise or set here on this date",
                    &["sunrise"],
                ),
            };
            let starts: Vec<Value> = v["windows"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|w| {
                    let s = Instant::parse_utc(w["start_utc"].as_str().unwrap_or_default()).map(|i| i.jd_ut).unwrap_or(f64::NAN);
                    s >= utc0 && s < utc1
                })
                .map(|w| json!({"start_local": w["start_local"], "end_local": w["end_local"], "duration_minutes": w["duration_minutes"]}))
                .collect();
            let mid = (utc0.max(a.jd_ut) + utc1.min(b.jd_ut)) / 2.0;
            let mut bindus = serde_json::Map::new();
            for g in ["jupiter", "saturn"] {
                let t = &transits[g];
                let cell = t["segments"].as_array().and_then(|segs| {
                    segs.iter().find(|s| {
                        s["start_jd_ut"].as_f64().is_some_and(|x| x <= mid)
                            && s["end_jd_ut"].as_f64().is_some_and(|x| mid < x)
                    })
                });
                bindus.insert(
                    g.into(),
                    cell.map_or_else(
                        || t.get("availability").cloned().unwrap_or(Value::Null),
                        |c| json!({"transit_sign": c["transit_sign"], "bav": c["bav"], "sav": c["sav"]}),
                    ),
                );
            }
            days.push(
                json!({"date": date, "sunrise_local": sunrise, "windows": starts, "transit_bindus": bindus}),
            );
            d += 1.0;
        }
        v["kind"] = json!("personal_calendar");
        v["days"] = json!(days);
        v["input"]["birth"] = json!({"instant_utc": env["input"]["instant_utc"], "place": env["input"]["place"],
            "place_role": "birth", "natal_moon_deg": natal_moon});
        let mut ev = v["input"]["place"].take();
        ev["place_role"] = json!("event");
        v["input"]["event_place"] = ev;
        if let Some(o) = v["input"].as_object_mut() {
            o.remove("place");
            o.remove("place_role");
        }
        if fmt != "json" {
            let (content, media) = if fmt == "csv" {
                (calendar_csv(&v), "text/csv")
            } else {
                (calendar_ics(&v), "text/calendar")
            };
            v["format"] = json!(fmt);
            v["media_type"] = json!(media);
            v["content"] = json!(content);
        }
        Ok(v)
    }

    #[allow(clippy::too_many_lines)]
    fn muhurta_core(
        &self,
        r: &MuhurtaRequest,
        natal_moon: Option<f64>,
        partner_moon: Option<f64>,
    ) -> Result<Value> {
        let (a, b) = (Instant::parse_utc(&r.start_utc)?, Instant::parse_utc(&r.end_utc)?);
        check_span(a.jd_ut, b.jd_ut, 62.0 / 366.0)?;
        let mut assumptions = vec![];
        let mut c = r.config.clone();
        if c.zodiac.is_none() {
            c.zodiac = Some("sidereal".into());
        }
        let cfg = build_config(&c, Some("vedic"), false, &mut assumptions)?;
        let ayan = cfg
            .ayanamsha
            .ok_or_else(|| EshanaError::Config("muhurta needs an ayanamsha".into()))?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let offset_s = match (&r.timezone, r.utc_offset_minutes) {
            (Some(z), _) => f64::from(eshana_time::zone_offset_seconds(z, a.jd_ut)?),
            (None, Some(m)) => f64::from(m) * 60.0,
            (None, None) => {
                assumptions.push("no civil zone: local dates use mean solar time at the longitude".into());
                r.place.longitude / 15.0 * 3600.0
            }
        };
        let site = crate::panchanga::Site {
            lat: r.place.latitude,
            lon: r.place.longitude,
            offset_s,
            rising: crate::panchanga::Rising::parse(r.sunrise.as_deref())?,
            lagna_sun_at_birth: true,
        };
        for f in &r.show {
            if !crate::muhurta::pack().factors.contains_key(f) {
                return Err(EshanaError::Input(format!("unknown factor '{f}' in show")));
            }
        }
        let (cons, prof) = crate::muhurta::constraint_set(r.profile.as_deref(), &r.disable, &r.constraints)?;
        let mut v = crate::muhurta::windows(
            provider.as_ref(),
            a.jd_ut,
            b.jd_ut,
            site,
            ayan,
            &cons,
            &r.show,
            r.timezone.as_deref(),
            natal_moon,
            partner_moon,
        )?;
        let pk = crate::muhurta::pack();
        v["schema_version"] = json!(SCHEMA_VERSION);
        v["kind"] = json!("muhurta_windows");
        v["input"] = json!({"place": {"latitude_deg": r.place.latitude, "longitude_deg": r.place.longitude},
            "place_role": "event", "timezone": r.timezone, "start_utc": a.utc, "end_utc": b.utc});
        v["profile"] = prof.map_or(Value::Null, |p| {
            json!({"id": p.id, "version": p.version, "status": p.status,
            "title": p.title, "disabled": r.disable, "missing": p.missing})
        });
        v["calculation"] = json!({"ayanamsha": ayan, "provider": cfg.provider, "sunrise": site.rising.id(),
            "config_hash": crate::methods::config_hash(&cfg), "range": "[start, end)",
            "composition": "cut at every factor boundary, evaluate each piece, merge neighbours with equal values and verdicts"});
        v["interpretations"] = json!([]);
        v["provenance"] = json!({"engine_version": ENGINE_VERSION, "assumptions": assumptions,
            "rulesets": [crate::packs::rule_id(&pk.id, &pk.version),
                         crate::packs::rule_id(&crate::packs::panchanga().id, &crate::packs::panchanga().version)]});
        if prof.is_some_and(|p| p.status == "experimental") {
            v["warnings"] = json!([{"code": "ESH-RULES", "message": "this profile is experimental: it covers only the factors listed and lacks those under profile.missing"}]);
        }
        Ok(v)
    }

    fn progressions_op(
        &self,
        natal: &ChartRequest,
        at_utc: &str,
        method: Option<&str>,
        year_days: Option<f64>,
    ) -> Result<Value> {
        use crate::progress as pg;
        let yd = year_days.unwrap_or(pg::TROPICAL_YEAR_DAYS);
        if !(360.0..=366.0).contains(&yd) {
            return Err(EshanaError::Input("year_days must be between 360 and 366".into()));
        }
        if let Some(m) = method {
            crate::methods::require("progression_angles", m)?;
        }
        let birth_env = self.chart(natal, None, "positions")?;
        let mo = resolve_when(&natal.when)?;
        let birth = mo.center.clone();
        let at = Instant::parse_utc(at_utc)?;
        if at.jd_ut < birth.jd_ut {
            return Err(EshanaError::RangeInvalid("at_utc is before the birth".into()));
        }
        let age = (at.jd_ut - birth.jd_ut) / yd;
        if age > 150.0 {
            return Err(EshanaError::WorkLimit(
                "progressions are limited to 150 years of life".into(),
            ));
        }
        let pjd = birth.jd_ut + age;
        let prog = Instant::from_jd_utc(pjd);
        let mut pr = natal.clone();
        pr.when = When {
            utc: Some(prog.utc.clone()),
            ..When::default()
        };
        let prog_env = self.chart(&pr, None, "positions")?;
        let lon = |e: &Value, id: &str| -> Option<f64> {
            e["positions"].as_array()?.iter().find(|p| p["id"] == id)?["tropical_longitude_deg"].as_f64()
        };
        let sun_arc = match (lon(&prog_env, "sun"), lon(&birth_env, "sun")) {
            (Some(p), Some(n)) => norm360(p - n),
            _ => return Err(EshanaError::Input("progressions need the Sun".into())),
        };
        let angles = match (method, natal.place.as_ref(), mo.half_width_days > 0.0) {
            (None, _, _) => crate::methods::unavailable(
                "no angle method was named (family progression_angles)",
                &["angle_method"],
            ),
            (Some(_), None, _) => {
                crate::methods::unavailable("progressed angles need the birth place", &["place"])
            }
            (Some(_), _, true) => {
                crate::methods::unavailable("progressed angles need an exact birth time", &["birth time"])
            }
            (Some(m), Some(pl), false) => {
                let tn = birth.t_tt();
                let ramc_n = astro::local_apparent_sidereal_deg(birth.gmst_deg(), pl.longitude, tn);
                let (_, mc_n) = astro::asc_mc(ramc_n, pl.latitude, tn);
                let tp = prog.t_tt();
                let lst_p = astro::local_apparent_sidereal_deg(prog.gmst_deg(), pl.longitude, tp);
                let ramc_p = pg::progressed_ramc(m, ramc_n, mc_n, sun_arc, age, lst_p, tp)?;
                let (asc, mc) = pg::angles_from_ramc(ramc_p, pl.latitude, tp);
                let off = prog_env["calculation"]["ayanamsha"]
                    .as_str()
                    .and_then(|a| Ayanamsha::parse(a).ok())
                    .map_or(0.0, |a| frames::nutation_deg(tp).0 + astro::ayanamsha_deg(a, tp));
                json!({"method": {"family": "progression_angles", "id": m}, "ramc_deg": ramc_p,
                       "ascendant_deg": norm360(asc - off), "midheaven_deg": norm360(mc - off),
                       "natal_midheaven_deg": norm360(mc_n - off), "zodiac_offset_deg": off})
            }
        };
        Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "progressions",
            "input": {"birth_utc": birth.utc, "at_utc": at.utc},
            "calculation": {"year_days": yd, "age_years": age, "progressed_utc": prog.utc, "progressed_jd_ut": pjd,
                "rule": "progressed instant = birth + age_years days, age_years = (at − birth) / year_days",
                "zodiac": prog_env["calculation"]["zodiac"], "ayanamsha": prog_env["calculation"]["ayanamsha"],
                "config_hash": prog_env["calculation"]["config_hash"]},
            "solar_arc_deg": sun_arc,
            "solar_arc_directions": solar_arc_directions(&birth_env, sun_arc),
            "positions": prog_env["positions"], "angles": angles, "interpretations": [],
            "provenance": prog_env["provenance"]}))
    }

    fn kp_horary_op(&self, number: u32, chart: &ChartRequest) -> Result<Value> {
        let entry = crate::kp::entry(number)?;
        let occ = crate::kp::Occupancy::parse(chart.kp_cusp_occupancy.as_deref())?;
        let pl = chart
            .place
            .as_ref()
            .ok_or_else(|| EshanaError::Input("KP horary needs the place of judgement".into()))?;
        let mut r = chart.clone();
        r.system = Some("vedic".into());
        r.features = vec!["kp".into()];
        r.divisions = vec![];
        r.dasha = false;
        let env = self.chart(&r, Some("vedic"), "computed_chart")?;
        let mo = resolve_when(&r.when)?;
        if mo.half_width_days > 0.0 {
            return Err(EshanaError::TimeUncertain(
                "KP horary needs the exact moment of judgement".into(),
            ));
        }
        let inst = &mo.center;
        let t = inst.t_tt();
        let ayan = env["calculation"]["ayanamsha"]
            .as_str()
            .and_then(|a| Ayanamsha::ALL.iter().copied().find(|x| x.id() == a))
            .ok_or_else(|| EshanaError::Config("KP horary needs a sidereal ayanamsha".into()))?;
        let off = frames::nutation_deg(t).0 + astro::ayanamsha_deg(ayan, t);
        let (lst, cusps) = crate::kp::horary_cusps(number, pl.latitude, t, off)?;
        let mut pos: BTreeMap<String, (f64, f64)> = BTreeMap::new();
        for p in env["positions"].as_array().into_iter().flatten() {
            if let (Some(id), Some(l), Some(sp)) = (
                p["id"].as_str(),
                p["longitude_deg"].as_f64(),
                p["speed_deg_per_day"].as_f64(),
            ) {
                pos.insert(id.to_string(), (l, sp));
            }
        }
        let vc = crate::jyotish::VChart::from_positions(&pos, Some(cusps[0]), Some(cusps[9]));
        let mut kp = crate::jyotish::kp_with(&vc, &cusps, occ);
        let natal_kp = env["derived_attributes"]
            .as_array()
            .and_then(|d| d.iter().find(|x| x["id"] == "vedic.kp"))
            .map(|x| x["value"].clone());
        if let Some(rp) = natal_kp.as_ref().map(|v| v["ruling_planets"].clone()) {
            kp["ruling_planets_at_judgement"] = rp;
        }
        let mut notes = vec![];
        if ayan != Ayanamsha::Krishnamurti {
            notes.push("KP is normally used with the krishnamurti ayanamsha; this chart uses another one");
        }
        Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "kp_horary",
            "input": {"number": number, "judgement_utc": inst.utc, "place": {"latitude_deg": pl.latitude, "longitude_deg": pl.longitude}},
            "number": crate::kp::entry_json(&entry),
            "calculation": {"ayanamsha": ayan, "provider": env["calculation"]["provider"], "house_system": "placidus",
                "config_hash": env["calculation"]["config_hash"],
                "local_sidereal_time_deg": lst,
                "method": {"family": "kp_cusp_occupancy", "id": occ.id()},
                "rule": "Ascendant fixed at the start of the number's arc; cusps are Placidus for the local sidereal time at which that degree rises at the place of judgement; planets are those of the moment of judgement"},
            "ascendant": {"longitude_deg": cusps[0], "sign": sign_value(cusps[0], true)},
            "cusps_deg": cusps,
            "positions": env["positions"],
            "kp": kp, "notes": notes, "interpretations": [],
            "provenance": {"engine_version": ENGINE_VERSION, "rulesets": env["provenance"]["rulesets"],
                           "assumptions": env["provenance"]["assumptions"]}}))
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

    fn stations_op(&self, r: &StationRequest) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, None, false, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let body = Body::parse(&r.body)?;
        let (a, b, tol) = event_range(&r.start_utc, &r.end_utc, r.tolerance_seconds)?;
        let ayan = sidereal_of(&cfg);
        let found = crate::events::stations(provider.as_ref(), body, a, b, ayan, tol / 86_400.0)?;
        let mut out = json!({"schema_version": SCHEMA_VERSION, "kind": "stations", "body": body.id(),
              "calculation": event_calculation(&cfg, tol), "interpretations": []});
        out["stations"] = found["stations"].clone();
        out["retrograde_intervals"] = found["retrograde_intervals"].clone();
        if crate::events::never_stations(body) {
            out["note"] = json!(format!("{} never stations", body.id()));
        }
        Ok(out)
    }

    fn crossings_op(&self, r: &CrossingRequest) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, None, false, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let body = Body::parse(&r.body)?;
        let (a, b, tol) = event_range(&r.start_utc, &r.end_utc, r.tolerance_seconds)?;
        if !r.point_deg.is_finite() {
            return Err(EshanaError::Input(
                "point_deg must be a finite number of degrees".into(),
            ));
        }
        let aspects = r.aspects.clone().unwrap_or_else(|| vec![0.0]);
        if aspects.is_empty() || aspects.len() > 16 || aspects.iter().any(|x| !x.is_finite()) {
            return Err(EshanaError::Input(
                "aspects must be 1 to 16 finite angles in degrees".into(),
            ));
        }
        let orb = r.touch_orb_deg.unwrap_or(0.1);
        if !(0.0..=5.0).contains(&orb) {
            return Err(EshanaError::Input("touch_orb_deg must be between 0 and 5".into()));
        }
        let max = r.max_results.unwrap_or(10_000);
        if max == 0 || max > 100_000 {
            return Err(EshanaError::Input(
                "max_results must be between 1 and 100000".into(),
            ));
        }
        let found = crate::events::crossings(
            provider.as_ref(),
            body,
            r.point_deg,
            &aspects,
            a,
            b,
            sidereal_of(&cfg),
            tol / 86_400.0,
            orb,
            max,
        )?;
        let mut calc = event_calculation(&cfg, tol);
        calc["touch_orb_deg"] = json!(orb);
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "crossings", "body": body.id(),
              "point_deg": r.point_deg, "aspects": aspects, "calculation": calc,
              "events": found["events"], "truncated": found["truncated"], "interpretations": []}),
        )
    }

    fn gochara_op(
        &self,
        natal: &ChartRequest,
        start_utc: &str,
        end_utc: &str,
        bodies: Option<&[String]>,
    ) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(
            &natal.config,
            Some("vedic"),
            natal.assume_defaults,
            &mut assumptions,
        )?;
        let ayan = cfg
            .ayanamsha
            .ok_or_else(|| EshanaError::Config("gochara is sidereal: give an ayanamsha".into()))?;
        let provider = self.provider(&cfg.provider, natal.spk_path.as_deref())?;
        let mo = resolve_when(&natal.when)?;
        let (a, b) = (Instant::parse_utc(start_utc)?, Instant::parse_utc(end_utc)?);
        if b.jd_ut <= a.jd_ut || b.jd_ut - a.jd_ut > 30.0 * 366.0 {
            return Err(EshanaError::RangeInvalid(
                "end must be after start and within 30 years".into(),
            ));
        }
        let p = provider.as_ref();
        let moon_sign = astro::sign_index(crate::events::lon_at(p, Body::Moon, mo.center.jd_ut, Some(ayan))?);
        // the lagna counts only with an exact time and a place
        let lagna_sign = match (&natal.place, mo.half_width_days > 0.0) {
            (Some(pl), false) => {
                let t = mo.center.t_tt();
                let lst = astro::local_apparent_sidereal_deg(mo.center.gmst_deg(), pl.longitude, t);
                Some(astro::sign_index(astro::to_sidereal(
                    astro::asc_mc(lst, pl.latitude, t).0,
                    ayan,
                    t,
                )))
            }
            _ => None,
        };
        let list: Vec<Body> = match bodies {
            Some(b) => b.iter().map(|x| Body::parse(x)).collect::<Result<_>>()?,
            None => vec![
                Body::Sun,
                Body::Moon,
                Body::Mars,
                Body::Mercury,
                Body::Jupiter,
                Body::Venus,
                Body::Saturn,
                cfg.node_model.body(),
            ],
        };
        let mut out = vec![];
        for body in list {
            let ing = crate::events::ingresses(p, body, a.jd_ut, b.jd_ut, Some(ayan))?;
            let mut sign = astro::sign_index(crate::events::lon_at(p, body, a.jd_ut, Some(ayan))?);
            let mut start = a.jd_ut;
            let mut segs = vec![];
            let seg = |sign: usize, s: f64, e: f64| {
                json!({"sign": sign, "start_utc": Instant::from_jd_utc(s).utc, "end_utc": Instant::from_jd_utc(e).utc,
                "house_from_moon": (sign + 12 - moon_sign) % 12 + 1, "house_from_lagna": lagna_sign.map(|l| (sign + 12 - l) % 12 + 1)})
            };
            for i in &ing {
                let jd = i["jd_ut"].as_f64().unwrap_or(start);
                segs.push(seg(sign, start, jd));
                sign = i["to_sign"].as_u64().unwrap_or(0) as usize;
                start = jd;
            }
            segs.push(seg(sign, start, b.jd_ut));
            let retro = if body.is_node() {
                json!([])
            } else {
                crate::events::stations(p, body, a.jd_ut, b.jd_ut, Some(ayan), 1.0 / 86_400.0)?
                    ["retrograde_intervals"]
                    .clone()
            };
            let combust = crate::events::combustion(p, body, a.jd_ut, b.jd_ut, Some(ayan))?;
            out.push(json!({"body": body.id(), "segments": segs, "retrograde_intervals": retro, "combustion_intervals": combust}));
        }
        assumptions.push(
            "houses from the natal Moon (and lagna, when the birth time is exact) by whole sign".into(),
        );
        assumptions.push("combustion: the vedic-core pack's orbs from the Sun (Mercury and Venus smaller while retrograde)".into());
        assumptions.push("gochara vedha (obstruction) is not applied".into());
        Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "gochara",
            "input": {"birth_utc": mo.center.utc, "start_utc": a.utc, "end_utc": b.utc},
            "calculation": {"zodiac": cfg.zodiac, "ayanamsha": cfg.ayanamsha, "provider": cfg.provider, "natal_moon_sign": moon_sign,
                            "natal_lagna_sign": lagna_sign, "rule": crate::packs::rule_id(&crate::packs::core().id, &crate::packs::core().version),
                            "config_hash": crate::methods::config_hash(&cfg)},
            "transits": out,
            "provenance": {"engine_version": env!("CARGO_PKG_VERSION"), "provider_data": provider.info().data, "assumptions": assumptions},
            "interpretations": []}))
    }

    fn returns_op(
        &self,
        natal: &ChartRequest,
        body: Option<&str>,
        start_utc: &str,
        end_utc: &str,
        place: Option<&Place>,
        charts: bool,
    ) -> Result<Value> {
        let body = Body::parse(body.unwrap_or("sun"))?;
        if body.is_node() {
            return Err(EshanaError::Input("a return is of a body, not a node".into()));
        }
        let mut assumptions = vec![];
        let system = natal.system.as_deref();
        let cfg = build_config(&natal.config, system, natal.assume_defaults, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, natal.spk_path.as_deref())?;
        let mo = resolve_when(&natal.when)?;
        if mo.half_width_days > 0.0 {
            return Err(EshanaError::TimeUncertain(
                "a return needs the exact birth time: the natal longitude moves with it".into(),
            ));
        }
        let (a, b) = (Instant::parse_utc(start_utc)?, Instant::parse_utc(end_utc)?);
        let limit_years = if body == Body::Moon { 10.0 } else { 120.0 };
        if b.jd_ut <= a.jd_ut || b.jd_ut - a.jd_ut > limit_years * 366.0 {
            return Err(EshanaError::RangeInvalid(format!(
                "end must be after start and within {limit_years} years for {}",
                body.id()
            )));
        }
        let ayan = if cfg.zodiac == ZodiacSystem::Sidereal {
            cfg.ayanamsha
        } else {
            None
        };
        let natal_lon = crate::events::lon_at(provider.as_ref(), body, mo.center.jd_ut, ayan)?;
        let found = crate::events::crossings(
            provider.as_ref(),
            body,
            natal_lon,
            &[0.0],
            a.jd_ut,
            b.jd_ut,
            ayan,
            1e-6,
            0.0,
            2000,
        )?;
        let mut returns = vec![];
        for ev in found["events"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|e| e["kind"] == "crossing")
        {
            let jd = ev["jd_ut"].as_f64().unwrap_or(0.0);
            let mut row = json!({"utc": ev["utc"], "jd_ut": jd, "direction": ev["direction"], "residual_deg": ev["residual_deg"]});
            if charts {
                let mut r = natal.clone();
                r.when = When {
                    utc: Some(Instant::from_jd_utc(jd).utc),
                    ..When::default()
                };
                if let Some(pl) = place {
                    r.place = Some(pl.clone());
                }
                r.features = vec![];
                r.dasha = false;
                let env = self.chart(&r, system, "computed_chart")?;
                row["chart"] = json!({"positions": env["positions"], "houses": env["houses"], "calculation": env["calculation"]});
            }
            returns.push(row);
        }
        assumptions.push(format!(
            "a return is the instant the {} reaches its natal {} longitude {:.6}° again (a crossing of that point, solved to 0.1 s)",
            body.id(),
            if ayan.is_some() { "sidereal" } else { "tropical" },
            natal_lon
        ));
        if place.is_some() {
            assumptions.push(
                "relocated return: the charts are cast for the given place, not the birth place".into(),
            );
        }
        Ok(
            json!({"schema_version": SCHEMA_VERSION, "kind": "returns", "body": body.id(),
            "input": {"birth_utc": mo.center.utc, "start_utc": a.utc, "end_utc": b.utc},
            "calculation": {"zodiac": cfg.zodiac, "ayanamsha": cfg.ayanamsha, "provider": cfg.provider,
                            "natal_longitude_deg": natal_lon, "config_hash": crate::methods::config_hash(&cfg)},
            "returns": returns,
            "provenance": {"engine_version": env!("CARGO_PKG_VERSION"), "provider_data": provider.info().data, "assumptions": assumptions},
            "interpretations": []}),
        )
    }

    fn rise_set_op(&self, r: &RiseSetRequest) -> Result<Value> {
        let mut assumptions = vec![];
        let cfg = build_config(&r.config, None, false, &mut assumptions)?;
        let provider = self.provider(&cfg.provider, r.spk_path.as_deref())?;
        let days = r.days.unwrap_or(1);
        if !(1..=366).contains(&days) {
            return Err(EshanaError::Input("days must be 1 to 366".into()));
        }
        let conv = crate::panchanga::Rising::parse(r.sunrise.as_deref())?;
        let bodies: Vec<Body> = match &r.bodies {
            Some(b) => b.iter().map(|x| Body::parse(x)).collect::<Result<_>>()?,
            None => vec![Body::Sun, Body::Moon],
        };
        if let Some(b) = bodies.iter().find(|b| b.is_node()) {
            return Err(EshanaError::Input(format!(
                "{} is a point, not a body that rises",
                b.id()
            )));
        }
        let obs = crate::rising::Observer {
            lat: r.place.latitude,
            lon: r.place.longitude,
            height_m: r.place.altitude_m.unwrap_or(0.0),
        };
        let start = eshana_time::parse_date(&r.date)?;
        let mut out = vec![];
        for k in 0..days {
            let jd = eshana_time::jd_from_gregorian(
                i32::from(start.year()),
                start.month() as u32,
                f64::from(start.day()),
            ) + f64::from(k);
            let (y, m, d, _) = eshana_time::gregorian_from_jd(jd);
            let date = eshana_time::iso_date(y, m, d);
            let midnight = match (&r.timezone, r.utc_offset_minutes) {
                (Some(z), _) => Instant::from_local_str(&date, "00:00", z, FoldPolicy::Earliest)?,
                (None, Some(o)) => Instant::from_offset_str(&date, "00:00", o)?,
                (None, None) => {
                    return Err(EshanaError::Input(
                        "rise_set needs timezone (IANA name) or utc_offset_minutes for the local days".into(),
                    ))
                }
            };
            let off = f64::from(midnight.civil_offset_s.unwrap_or(0));
            let mut day = json!({"date": date, "bodies": bodies.iter().map(|b| crate::rising::body_day(provider.as_ref(), *b, midnight.jd_ut, obs, conv, off)).collect::<Result<Vec<_>>>()?});
            if r.twilight.unwrap_or(true) {
                day["twilight"] = crate::rising::twilight(provider.as_ref(), midnight.jd_ut, obs, off)?;
            }
            out.push(day);
        }
        assumptions.push(format!("rise and set: {} (refraction 34′ at the horizon, no temperature or pressure model; the Moon's parallax and semidiameter included)", conv.id()));
        assumptions.push("twilight: the Sun's centre, geometric, at −6°, −12° and −18°".into());
        assumptions.push("transit is the upper culmination (hour angle 0)".into());
        Ok(json!({"schema_version": SCHEMA_VERSION, "kind": "rise_set",
                  "input": {"place": {"latitude_deg": obs.lat, "longitude_deg": obs.lon, "altitude_m": obs.height_m}, "date": r.date, "days": days,
                            "timezone": r.timezone, "utc_offset_minutes": r.utc_offset_minutes},
                  "calculation": {"provider": cfg.provider, "sunrise": conv.id(), "config_hash": crate::methods::config_hash(&cfg)},
                  "days": out,
                  "provenance": {"engine_version": env!("CARGO_PKG_VERSION"), "provider": cfg.provider, "provider_data": provider.info().data, "assumptions": assumptions},
                  "interpretations": []}))
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
                topocentric: ov.topocentric.or(base.config.topocentric),
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

/// A search range (UT Julian days, end after start, at most 200 years) and the solver tolerance in seconds.
fn event_range(start: &str, end: &str, tolerance_seconds: Option<f64>) -> Result<(f64, f64, f64)> {
    let (a, b) = (Instant::parse_utc(start)?, Instant::parse_utc(end)?);
    check_span(a.jd_ut, b.jd_ut, 200.0)?;
    let tol = tolerance_seconds.unwrap_or(1.0);
    if !(0.01..=3600.0).contains(&tol) {
        return Err(EshanaError::Input(
            "tolerance_seconds must be between 0.01 and 3600".into(),
        ));
    }
    Ok((a.jd_ut, b.jd_ut, tol))
}

/// `ESH-RANGE-INVALID` for an empty or reversed range, `ESH-LIMIT` past `max_years`.
pub(crate) fn check_span(a: f64, b: f64, max_years: f64) -> Result<()> {
    if b <= a {
        return Err(EshanaError::RangeInvalid("end must be after start".into()));
    }
    if b - a > max_years * 366.0 {
        return Err(EshanaError::WorkLimit(format!(
            "the range may span at most {max_years} years"
        )));
    }
    Ok(())
}

fn sidereal_of(cfg: &CalculationConfig) -> Option<Ayanamsha> {
    if cfg.zodiac == ZodiacSystem::Sidereal {
        cfg.ayanamsha
    } else {
        None
    }
}

fn event_calculation(cfg: &CalculationConfig, tol: f64) -> Value {
    json!({"zodiac": cfg.zodiac, "ayanamsha": cfg.ayanamsha, "provider": cfg.provider,
           "config_hash": crate::methods::config_hash(cfg), "tolerance_seconds": tol, "range": "[start, end)",
           "note": "the tolerance is the solver's; the provider's position error can move an event by more"})
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn label_of(v: &Value) -> String {
    if let Some(a) = v.get("availability") {
        return format!("unavailable: {}", a["reason"].as_str().unwrap_or(""));
    }
    match v {
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        _ => v
            .get("name")
            .or_else(|| v.get("lord"))
            .or_else(|| v.get("quality"))
            .or_else(|| v.get("index"))
            .map_or_else(
                || v.to_string(),
                |x| x.as_str().map_or_else(|| x.to_string(), String::from),
            ),
    }
}

/// One row per piece of the timeline: times, verdict, every factor, failed constraints.
fn calendar_csv(v: &Value) -> String {
    let tl = v["timeline"].as_array().cloned().unwrap_or_default();
    let factors: Vec<String> = tl
        .first()
        .and_then(|w| w["factors"].as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    let mut out = format!(
        "start_utc,end_utc,start_local,end_local,passes,{},failed\n",
        factors.join(",")
    );
    for w in tl {
        let mut row = vec![
            w["start_utc"].as_str().unwrap_or("").to_string(),
            w["end_utc"].as_str().unwrap_or("").to_string(),
            w["start_local"].as_str().unwrap_or("").to_string(),
            w["end_local"].as_str().unwrap_or("").to_string(),
            w["passes"].to_string(),
        ];
        row.extend(factors.iter().map(|f| label_of(&w["factors"][f])));
        row.push(
            w["failed"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|x| x["constraint"].as_str().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(" "),
        );
        out.push_str(&row.iter().map(|x| csv_field(x)).collect::<Vec<_>>().join(","));
        out.push('\n');
    }
    out
}

fn ics_time(utc: &str) -> String {
    // 2026-11-24T10:48:24.493Z → 20261124T104824Z (seconds truncated toward the window: start rounds up, see caller)
    let d: String = utc.chars().filter(char::is_ascii_digit).take(14).collect();
    format!("{}T{}Z", &d[..8], &d[8..14])
}

fn ics_fold(line: &str) -> String {
    let mut out = String::new();
    let mut n = 0;
    for ch in line.chars() {
        let w = ch.len_utf8();
        if n + w > 74 {
            out.push_str("\r\n ");
            n = 1;
        }
        out.push(ch);
        n += w;
    }
    out.push_str("\r\n");
    out
}

fn ics_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

/// One VEVENT per passing window, in UTC; deterministic UIDs and DTSTAMP (the range start), so the same request
/// gives the same bytes.
fn calendar_ics(v: &Value) -> String {
    let prof = v["profile"]["id"].as_str().unwrap_or("constraints");
    let stamp = ics_time(v["input"]["start_utc"].as_str().unwrap_or("1970-01-01T00:00:00Z"));
    let mut out = String::new();
    for l in [
        "BEGIN:VCALENDAR",
        "VERSION:2.0",
        "PRODID:-//ESHANA//muhurta calendar//EN",
        "CALSCALE:GREGORIAN",
    ] {
        out.push_str(&ics_fold(l));
    }
    for w in v["windows"].as_array().into_iter().flatten() {
        let (s, e) = (
            w["start_utc"].as_str().unwrap_or(""),
            w["end_utc"].as_str().unwrap_or(""),
        );
        let desc = w["factors"]
            .as_object()
            .map(|o| {
                o.iter()
                    .map(|(k, x)| format!("{k}: {}", label_of(x)))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        let uid = crate::methods::hash_value(&json!([prof, s, e])).replace("fnv1a64:", "");
        for l in [
            "BEGIN:VEVENT".to_string(),
            format!("UID:{uid}@eshana"),
            format!("DTSTAMP:{stamp}"),
            format!("DTSTART:{}", ics_time(s)),
            format!("DTEND:{}", ics_time(e)),
            format!("SUMMARY:{}", ics_text(&format!("Muhūrta window ({prof})"))),
            format!("DESCRIPTION:{}", ics_text(&desc)),
            "END:VEVENT".to_string(),
        ] {
            out.push_str(&ics_fold(&l));
        }
    }
    out.push_str(&ics_fold("END:VCALENDAR"));
    out
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
