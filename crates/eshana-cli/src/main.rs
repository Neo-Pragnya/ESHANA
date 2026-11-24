//! `eshana`: command-line access to the engine. Every command builds a JSON request and sends it through
//! `Engine::execute_json`, the same entry point the Python binding uses, so the outputs are identical.

use clap::{Args, Parser, Subcommand};
use eshana_core::EshanaError;
use eshana_engine::Engine;
use serde_json::{json, Map, Value};
use std::io::Read;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "eshana",
    version,
    about = "ESHANA - Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations",
    long_about = "Compute -> Derive -> Interpret, with provenance. Every command prints one JSON envelope.\n\
                  Errors print an error envelope with a stable ESH-* code and exit with status 2."
)]
struct Cli {
    /// Print compact JSON instead of indented.
    #[arg(long, global = true)]
    compact: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args, Clone, Default)]
struct Moment {
    /// UTC instant, e.g. 2000-01-01T12:00:00Z
    #[arg(long)]
    utc: Option<String>,
    /// Local date YYYY-MM-DD (a signed year such as -0043-03-15 is 44 BCE)
    #[arg(long, allow_hyphen_values = true)]
    date: Option<String>,
    /// Calendar of --date: gregorian (proleptic ISO 8601, the default), julian, or auto (Julian before 1582-10-15)
    #[arg(long)]
    calendar: Option<String>,
    /// Local time HH:MM[:SS]; omit for an unknown birth time
    #[arg(long)]
    time: Option<String>,
    /// IANA time zone, e.g. Asia/Kolkata
    #[arg(long = "tz")]
    timezone: Option<String>,
    /// Fixed UTC offset in minutes east, when no zone is known
    #[arg(long)]
    offset_minutes: Option<i32>,
    /// For a repeated local time: reject (default), earliest, latest
    #[arg(long)]
    fold: Option<String>,
    /// Birth-time uncertainty in minutes (plus or minus)
    #[arg(long)]
    uncertainty_minutes: Option<f64>,
    /// Degrees north
    #[arg(long, allow_hyphen_values = true)]
    lat: Option<f64>,
    /// Degrees east
    #[arg(long, allow_hyphen_values = true)]
    lon: Option<f64>,
    /// Metres above sea level (for topocentric positions and rise/set)
    #[arg(long, allow_hyphen_values = true)]
    altitude_m: Option<f64>,
}

#[derive(Args, Clone, Default)]
struct Settings {
    /// Ephemeris provider: analytic (default) or spk
    #[arg(long)]
    provider: Option<String>,
    /// Path to a JPL SPK kernel (or set ESHANA_SPK)
    #[arg(long)]
    spk: Option<String>,
    /// tropical or sidereal
    #[arg(long)]
    zodiac: Option<String>,
    /// lahiri, raman, krishnamurti, fagan_bradley, yukteshwar, jn_bhasin, ushashashi, sassanian, deluce, djwhal_khul. Required with --zodiac sidereal; only `chart vedic` defaults to lahiri, and says so
    #[arg(long)]
    ayanamsha: Option<String>,
    /// whole_sign (default), equal, placidus, porphyry, sripati, koch, regiomontanus, campanus, alcabitius,
    /// topocentric, morinus, meridian
    #[arg(long)]
    houses: Option<String>,
    /// mean or true
    #[arg(long)]
    node: Option<String>,
    /// Positions as seen from the place (parallax applied; needs --lat/--lon)
    #[arg(long)]
    topocentric: bool,
}

#[derive(Args, Clone, Default)]
struct ChartOpts {
    #[command(flatten)]
    when: Moment,
    #[command(flatten)]
    set: Settings,
    /// Comma-separated bodies (default: ten planets plus the node)
    #[arg(long, value_delimiter = ',')]
    bodies: Option<Vec<String>>,
    /// Divisional charts, e.g. 1,9,10
    #[arg(long, value_delimiter = ',')]
    divisions: Vec<u32>,
    /// Include Vimshottari dasha (vedic)
    #[arg(long)]
    dasha: bool,
    /// Daśā year: 365.25 (default), 365.2425, 360, 365.25636
    #[arg(long)]
    year_convention: Option<String>,
    /// Include Ptolemaic aspects (western)
    #[arg(long)]
    aspects: bool,
    /// Multiply every aspect orb
    #[arg(long)]
    orb_scale: Option<f64>,
    /// Vedic features: panchanga, strength, relationships, drishti, ashtakavarga, jaimini, kp, yogas, sade_sati, all
    #[arg(long, value_delimiter = ',')]
    features: Vec<String>,
    /// Vimshottari depth 1-5 (default 2)
    #[arg(long)]
    dasha_levels: Option<usize>,
    /// Dasha systems: vimshottari (default), yogini
    #[arg(long, value_delimiter = ',')]
    dasha_systems: Vec<String>,
    /// Instant for "current" periods, e.g. 2026-11-21T00:00:00Z (never the computer's clock)
    #[arg(long)]
    reference_utc: Option<String>,
    /// Sunrise: upper_limb_refracted (default), centre_refracted, centre_unrefracted
    #[arg(long)]
    sunrise: Option<String>,
    /// KP cusp occupancy: cusp_start (default) or cusp_midpoint
    #[arg(long)]
    kp_occupancy: Option<String>,
    /// Further vargas by method id, e.g. d81_navanavamsha,d144_dvadasha_dvadashamsha (see `eshana methods`)
    #[arg(long, value_delimiter = ',')]
    varga_methods: Vec<String>,
}

#[derive(Args, Clone, Default)]
struct MuhurtaOpts {
    /// Event place: degrees north
    #[arg(long, allow_hyphen_values = true)]
    lat: f64,
    /// Event place: degrees east
    #[arg(long, allow_hyphen_values = true)]
    lon: f64,
    /// IANA zone of the event place (local times follow its DST)
    #[arg(long = "tz")]
    timezone: Option<String>,
    /// Range start, UTC (included)
    #[arg(long)]
    start: String,
    /// Range end, UTC (excluded); at most 62 days
    #[arg(long)]
    end: String,
    /// Profile id (see `eshana methods`, family muhurta_profile); calendar defaults to personal_general
    #[arg(long)]
    profile: Option<String>,
    /// Profile constraints to switch off, comma separated
    #[arg(long, value_delimiter = ',')]
    disable: Vec<String>,
    /// Extra constraints by id, comma separated
    #[arg(long = "constraint", value_delimiter = ',')]
    constraints: Vec<String>,
    /// Factors to report on every window, comma separated
    #[arg(long, value_delimiter = ',')]
    show: Vec<String>,
    /// Ayanamsha (default lahiri, recorded)
    #[arg(long, default_value = "lahiri")]
    ayanamsha: String,
    /// Sunrise convention
    #[arg(long)]
    sunrise: Option<String>,
}

fn muhurta_json(m: &MuhurtaOpts) -> Value {
    json!({"place": {"latitude": m.lat, "longitude": m.lon}, "timezone": m.timezone, "start_utc": m.start,
           "end_utc": m.end, "profile": m.profile, "disable": m.disable, "constraints": m.constraints, "show": m.show,
           "sunrise": m.sunrise, "config": {"zodiac": "sidereal", "ayanamsha": m.ayanamsha}})
}

#[derive(Args, Clone, Default)]
struct Person {
    /// Local date YYYY-MM-DD
    #[arg(long)]
    date: String,
    /// Local time HH:MM[:SS]
    #[arg(long)]
    time: Option<String>,
    /// IANA time zone
    #[arg(long = "tz")]
    timezone: Option<String>,
    /// Degrees north
    #[arg(long, allow_hyphen_values = true)]
    lat: Option<f64>,
    /// Degrees east
    #[arg(long, allow_hyphen_values = true)]
    lon: Option<f64>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Apparent geocentric positions only
    Positions(ChartOpts),
    /// A Vedic (sidereal) or Western (tropical) chart
    Chart {
        /// vedic or western
        system: String,
        #[command(flatten)]
        o: ChartOpts,
    },
    /// Human Design: design instant, gates, lines, channels, type, authority, profile
    HumanDesign(ChartOpts),
    /// Name and birth-date numerology
    Numerology {
        /// Name to analyse
        #[arg(long)]
        name: Option<String>,
        /// Date of birth YYYY-MM-DD
        #[arg(long)]
        dob: Option<String>,
        /// pythagorean (default) or chaldean
        #[arg(long)]
        school: Option<String>,
        /// Do not keep 11, 22, 33
        #[arg(long)]
        no_master: bool,
        /// Treat Y as a vowel
        #[arg(long)]
        y_vowel: bool,
        /// reduce_parts (default) or total_digits
        #[arg(long)]
        life_path_method: Option<String>,
        /// Calendar year for the personal-year cycle
        #[arg(long)]
        year: Option<i32>,
        /// male or female (picks the Kua number)
        #[arg(long)]
        gender: Option<String>,
    },
    /// Same input under two configurations, with a diff of what changed
    Compare {
        /// vedic or western
        system: String,
        #[command(flatten)]
        o: ChartOpts,
        /// Overrides for side A as key=value (provider, zodiac, ayanamsha, house_system, node_model)
        #[arg(long = "a", value_name = "KEY=VALUE")]
        a: Vec<String>,
        /// Overrides for side B
        #[arg(long = "b", value_name = "KEY=VALUE")]
        b: Vec<String>,
    },
    /// Symbolic graph of a chart: json, mermaid or graphml
    Graph {
        /// vedic or western
        system: String,
        #[command(flatten)]
        o: ChartOpts,
        /// json (default), mermaid, graphml
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Panchanga and day divisions (sunrise, tithi, nakshatra, yoga, karana, hora, choghadiya, rahu kalam)
    Panchanga {
        #[command(flatten)]
        o: ChartOpts,
        /// Natal Moon in sidereal degrees, for tara bala and chandra bala
        #[arg(long)]
        natal_moon: Option<f64>,
    },
    /// Transit Ashtakavarga: the natal BAV/SAV cells each graha reads, as segments between ingresses
    AshtakavargaTransits {
        /// Range start, UTC (included)
        #[arg(long)]
        start: String,
        /// Range end, UTC (excluded)
        #[arg(long)]
        end: String,
        /// Transiting bodies (default Sun to Saturn); a node is answered as unavailable
        #[arg(long = "transit-bodies", value_delimiter = ',')]
        transit_bodies: Option<Vec<String>>,
        /// The natal chart
        #[command(flatten)]
        o: ChartOpts,
    },
    /// Muhurta windows: factor timelines over a range composed under a profile and constraints
    Muhurta {
        #[command(flatten)]
        m: MuhurtaOpts,
    },
    /// Personal calendar: muhurta windows with tara and chandra bala from a birth, day summaries; json, csv or ics
    Calendar {
        /// The birth as date,time,tz,lat,lon (its place is the birth place, not the event place)
        #[arg(long)]
        birth: String,
        /// json (default), csv or ics; csv and ics print the file itself
        #[arg(long, default_value = "json")]
        format: String,
        #[command(flatten)]
        m: MuhurtaOpts,
    },
    /// Pair calendar: windows that suit both people and the event; each window explained by both and the event
    PairCalendar {
        /// First person as date,time,tz,lat,lon (the role classical tables give the groom)
        #[arg(long)]
        a: String,
        /// Second person, same format
        #[arg(long)]
        b: String,
        /// json (default), csv or ics; csv and ics print the file itself
        #[arg(long, default_value = "json")]
        format: String,
        #[command(flatten)]
        m: MuhurtaOpts,
    },
    /// Secondary progressions: a day for a year, fractional age, angles under a named method
    Progress {
        /// The moment of life to progress to, UTC
        #[arg(long)]
        at: String,
        /// Angle method: solar_arc_longitude, naibod_ra or quotidian (without one, angles are unavailable)
        #[arg(long)]
        angles: Option<String>,
        /// Days per year of life (default 365.24219)
        #[arg(long)]
        year_days: Option<f64>,
        /// The birth
        #[command(flatten)]
        o: ChartOpts,
    },
    /// The KP 1-249 horary number table
    KpTable,
    /// KP horary chart: the number fixes the Ascendant; the moment and place are those of judgement
    KpHorary {
        /// Horary number 1-249
        #[arg(long)]
        number: u32,
        #[command(flatten)]
        o: ChartOpts,
    },
    /// Ashtakoota compatibility of two births: a = the role classical tables give the groom, b = the bride
    Matching {
        /// First partner as date,time,tz,lat,lon (e.g. 1990-05-15,14:30,Asia/Kolkata,28.61,77.21)
        #[arg(long, allow_hyphen_values = true)]
        a: String,
        /// Second partner, same format
        #[arg(long, allow_hyphen_values = true)]
        b: String,
        /// Ayanamsha (default lahiri, recorded)
        #[arg(long, default_value = "lahiri")]
        ayanamsha: String,
    },
    /// Longitudes at regular steps (transit animation, progressions)
    Ephemeris {
        /// First instant, UTC
        #[arg(long)]
        start: String,
        /// Step in days
        #[arg(long, default_value_t = 1.0)]
        step_days: f64,
        /// Number of rows (max 20000)
        #[arg(long, default_value_t = 30)]
        count: usize,
        /// Bodies, comma separated
        #[arg(long, value_delimiter = ',')]
        bodies: Option<Vec<String>>,
        #[command(flatten)]
        set: Settings,
    },
    /// Sign ingresses of one body between two instants
    Ingresses {
        /// Body id, e.g. saturn
        #[arg(long)]
        body: String,
        /// Range start, UTC
        #[arg(long)]
        start: String,
        /// Range end, UTC
        #[arg(long)]
        end: String,
        #[command(flatten)]
        set: Settings,
    },
    /// Stations and retrograde intervals of one body between two instants
    Stations {
        /// Body id, e.g. mercury
        #[arg(long)]
        body: String,
        /// Range start, UTC (included)
        #[arg(long)]
        start: String,
        /// Range end, UTC (excluded)
        #[arg(long)]
        end: String,
        /// Solver tolerance in seconds (0.01 to 3600)
        #[arg(long)]
        tolerance_seconds: Option<f64>,
        #[command(flatten)]
        set: Settings,
    },
    /// When a body reaches a longitude, or an aspect to it, between two instants
    Crossings {
        /// Body id, e.g. saturn
        #[arg(long)]
        body: String,
        /// The point in degrees of the chosen zodiac, e.g. a natal Moon
        #[arg(long, allow_hyphen_values = true)]
        point: f64,
        /// Aspect angles in degrees, comma separated (default 0)
        #[arg(long, value_delimiter = ',')]
        aspects: Option<Vec<f64>>,
        /// Range start, UTC (included)
        #[arg(long)]
        start: String,
        /// Range end, UTC (excluded)
        #[arg(long)]
        end: String,
        /// Report a closest approach that stops within this orb without crossing (default 0.1, 0 = off)
        #[arg(long)]
        touch_orb: Option<f64>,
        /// Solver tolerance in seconds (0.01 to 3600)
        #[arg(long)]
        tolerance_seconds: Option<f64>,
        #[command(flatten)]
        set: Settings,
    },
    /// Print the built-in YAML rule packs
    Rulesets,
    /// List every method family and method id the engine implements
    Methods,
    /// Send a raw JSON request from a file, or `-` for stdin
    Request {
        /// File path or -
        file: String,
    },
    /// Show the environment, providers, rule packs and error codes
    Doctor,
    /// Explain a stable error code, e.g. ESH-LATITUDE
    Explain {
        /// Error code
        code: String,
    },
    /// Print the JSON Schema of the result envelope
    Schema,
    /// Rise, upper transit and set of bodies, and twilight, for local days at a place
    RiseSet {
        /// First local date YYYY-MM-DD
        #[arg(long)]
        date: String,
        /// IANA time zone of the local days
        #[arg(long = "tz")]
        timezone: Option<String>,
        /// Fixed UTC offset in minutes east (instead of --tz)
        #[arg(long, allow_hyphen_values = true)]
        offset_minutes: Option<i32>,
        /// Degrees north
        #[arg(long, allow_hyphen_values = true)]
        lat: f64,
        /// Degrees east
        #[arg(long, allow_hyphen_values = true)]
        lon: f64,
        /// Metres above sea level
        #[arg(long)]
        altitude_m: Option<f64>,
        /// Number of days, 1 to 366 (default 1)
        #[arg(long)]
        days: Option<u32>,
        /// Comma-separated bodies (default sun,moon)
        #[arg(long, value_delimiter = ',')]
        bodies: Option<Vec<String>>,
        /// upper_limb_refracted (default), centre_refracted, centre_unrefracted
        #[arg(long)]
        sunrise: Option<String>,
        /// Leave out civil, nautical and astronomical twilight
        #[arg(long)]
        no_twilight: bool,
        #[command(flatten)]
        set: Settings,
    },
    /// JPL kernels: `list` the ones ESHANA knows, `verify PATH|NAME` against NAIF's checksum, `path NAME`
    Kernel {
        /// list, verify or path
        action: String,
        /// A kernel file or a catalogue name such as de440 (verify, path)
        target: Option<String>,
    },
}

/// `eshana kernel …`: the catalogue, a checksum check, or where a catalogue name is looked up. Offline: nothing is
/// downloaded (see scripts/fetch_kernel.py in the repository for a checked download).
fn kernel_cmd(action: &str, target: Option<&str>, compact: bool) -> ExitCode {
    use eshana_ephemeris::kernels;
    let show = |v: Value| {
        println!(
            "{}",
            if compact {
                v.to_string()
            } else {
                serde_json::to_string_pretty(&v).expect("json")
            }
        );
    };
    match (action, target) {
        ("list", _) => {
            let dir = kernels::kernel_dir();
            let rows: Vec<Value> = kernels::KERNELS
                .iter()
                .map(|k| {
                    let mut v = serde_json::to_value(k).expect("json");
                    v["url"] = json!(format!("{}{}", kernels::NAIF_PLANETS, k.file));
                    v["present"] = json!(dir.join(k.file).exists());
                    v
                })
                .collect();
            show(
                json!({"kind": "kernels", "kernel_dir": dir, "source": kernels::NAIF_PLANETS, "kernels": rows}),
            );
            ExitCode::SUCCESS
        }
        ("path", Some(name)) => match kernels::find(name) {
            Some(k) => {
                println!("{}", kernels::kernel_dir().join(k.file).display());
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("unknown kernel '{name}'; run `eshana kernel list`");
                ExitCode::from(2)
            }
        },
        ("verify", Some(t)) => match kernels::verify(t) {
            Ok(v) => {
                let ok = v.status == "verified";
                show(json!({"kind": "kernel_verification", "verification": v}));
                if ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(2)
                }
            }
            Err(e) => {
                eprintln!("{}: {e}", e.code());
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("usage: eshana kernel list | verify PATH|NAME | path NAME");
            ExitCode::from(2)
        }
    }
}

fn kv(items: &[String]) -> Value {
    let mut m = Map::new();
    for s in items {
        if let Some((k, v)) = s.split_once('=') {
            m.insert(k.trim().to_string(), json!(v.trim()));
        }
    }
    Value::Object(m)
}

fn when_json(m: &Moment) -> Value {
    json!({"utc": m.utc, "date": m.date, "time": m.time, "timezone": m.timezone,
           "utc_offset_minutes": m.offset_minutes, "fold": m.fold, "uncertainty_minutes": m.uncertainty_minutes,
           "calendar": m.calendar})
}

fn chart_json(o: &ChartOpts, system: Option<&str>, assume: bool) -> Value {
    let place = match (o.when.lat, o.when.lon) {
        (Some(la), Some(lo)) => json!({"latitude": la, "longitude": lo, "altitude_m": o.when.altitude_m}),
        _ => Value::Null,
    };
    json!({
        "when": when_json(&o.when), "place": place, "system": system,
        "config": {"provider": o.set.provider, "zodiac": o.set.zodiac, "ayanamsha": o.set.ayanamsha,
                   "house_system": o.set.houses, "node_model": o.set.node,
                   "topocentric": if o.set.topocentric { json!(true) } else { Value::Null }},
        "spk_path": o.set.spk, "bodies": o.bodies, "divisions": o.divisions, "dasha": o.dasha,
        "year_convention": o.year_convention, "aspects": o.aspects, "orb_scale": o.orb_scale,
        "assume_defaults": assume, "features": o.features, "dasha_levels": o.dasha_levels,
        "dasha_systems": o.dasha_systems, "reference_utc": o.reference_utc, "sunrise": o.sunrise,
        "kp_cusp_occupancy": o.kp_occupancy, "varga_methods": o.varga_methods,
    })
}

fn person_json(spec: &str, ayanamsha: &str) -> Value {
    let f: Vec<&str> = spec.split(',').map(str::trim).collect();
    let get = |i: usize| f.get(i).filter(|s| !s.is_empty()).map(|s| (*s).to_string());
    let num = |i: usize| get(i).and_then(|s| s.parse::<f64>().ok());
    let place = match (num(3), num(4)) {
        (Some(la), Some(lo)) => json!({"latitude": la, "longitude": lo}),
        _ => Value::Null,
    };
    json!({"when": {"date": get(0), "time": get(1), "timezone": get(2)}, "place": place,
           "config": {"zodiac": "sidereal", "ayanamsha": ayanamsha}})
}

fn settings_json(s: &Settings) -> Value {
    json!({"provider": s.provider, "zodiac": s.zodiac, "ayanamsha": s.ayanamsha, "node_model": s.node})
}

fn strip_nulls(v: Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect(),
        ),
        other => other,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let engine = Engine::new();
    let request: Value = match &cli.cmd {
        Cmd::Positions(o) => {
            let mut c = chart_json(o, None, false);
            c["op"] = json!("positions");
            c
        }
        Cmd::Chart { system, o } => {
            let mut c = chart_json(o, Some(system), system == "vedic");
            c["op"] = json!("chart");
            c
        }
        Cmd::HumanDesign(o) => {
            let mut c = chart_json(o, None, false);
            c["op"] = json!("human_design");
            c
        }
        Cmd::Numerology {
            name,
            dob,
            school,
            no_master,
            y_vowel,
            life_path_method,
            year,
            gender,
        } => json!({
            "op": "numerology", "name": name, "date_of_birth": dob, "school": school,
            "keep_master_numbers": !no_master, "y_policy": if *y_vowel { "vowel" } else { "consonant" },
            "life_path_method": life_path_method, "reference_year": year, "gender": gender,
        }),
        Cmd::Compare { system, o, a, b } => json!({
            "op": "compare", "base": chart_json(o, Some(system), system == "vedic"), "a": kv(a), "b": kv(b),
        }),
        Cmd::Graph { system, o, format } => json!({
            "op": "graph", "chart": chart_json(o, Some(system), system == "vedic"), "format": format,
        }),
        Cmd::Panchanga { o, natal_moon } => {
            let mut c = chart_json(o, Some("vedic"), true);
            c["config"]["zodiac"] = json!("sidereal");
            json!({"op": "panchanga", "chart": c, "natal_moon_deg": natal_moon})
        }
        Cmd::KpTable => json!({"op": "kp_table"}),
        Cmd::RiseSet {
            date,
            timezone,
            offset_minutes,
            lat,
            lon,
            altitude_m,
            days,
            bodies,
            sunrise,
            no_twilight,
            set,
        } => json!({
            "op": "rise_set", "date": date, "timezone": timezone, "utc_offset_minutes": offset_minutes,
            "place": {"latitude": lat, "longitude": lon, "altitude_m": altitude_m}, "days": days, "bodies": bodies,
            "sunrise": sunrise, "twilight": !no_twilight, "config": settings_json(set), "spk_path": set.spk,
        }),
        Cmd::Progress {
            at,
            angles,
            year_days,
            o,
        } => json!({"op": "progressions", "natal": chart_json(o, None, false),
            "at_utc": at, "angle_method": angles, "year_days": year_days}),
        Cmd::PairCalendar { a, b, format, m } => {
            json!({"op": "pair_calendar", "a": person_json(a, &m.ayanamsha),
            "b": person_json(b, &m.ayanamsha), "muhurta": muhurta_json(m), "format": format})
        }
        Cmd::Muhurta { m } => {
            let mut r = muhurta_json(m);
            r["op"] = json!("muhurta_windows");
            r
        }
        Cmd::Calendar { birth, format, m } => {
            let mut natal = person_json(birth, &m.ayanamsha);
            natal["system"] = json!("vedic");
            let mut mj = muhurta_json(m);
            if m.profile.is_none() && m.constraints.is_empty() {
                mj["profile"] = json!("personal_general"); // the same default as ChartEngine.personal_calendar
            }
            json!({"op": "personal_calendar", "natal": natal, "muhurta": mj, "format": format})
        }
        Cmd::AshtakavargaTransits {
            start,
            end,
            transit_bodies,
            o,
        } => {
            let mut c = chart_json(o, Some("vedic"), true);
            c["config"]["zodiac"] = json!("sidereal");
            json!({"op": "ashtakavarga_transits", "natal": c, "start_utc": start, "end_utc": end, "bodies": transit_bodies})
        }
        Cmd::KpHorary { number, o } => {
            let mut c = chart_json(o, Some("vedic"), true);
            c["config"]["zodiac"] = json!("sidereal");
            if o.set.ayanamsha.is_none() {
                c["config"]["ayanamsha"] = json!("krishnamurti");
            }
            json!({"op": "kp_horary", "number": number, "chart": c})
        }
        Cmd::Matching { a, b, ayanamsha } => json!({
            "op": "matching", "a": person_json(a, ayanamsha), "b": person_json(b, ayanamsha),
        }),
        Cmd::Ephemeris {
            start,
            step_days,
            count,
            bodies,
            set,
        } => json!({
            "op": "ephemeris", "start_utc": start, "step_days": step_days, "count": count, "bodies": bodies,
            "config": settings_json(set), "spk_path": set.spk,
        }),
        Cmd::Ingresses {
            body,
            start,
            end,
            set,
        } => json!({
            "op": "ingresses", "body": body, "start_utc": start, "end_utc": end,
            "config": settings_json(set), "spk_path": set.spk,
        }),
        Cmd::Stations {
            body,
            start,
            end,
            tolerance_seconds,
            set,
        } => json!({
            "op": "stations", "body": body, "start_utc": start, "end_utc": end,
            "tolerance_seconds": tolerance_seconds, "config": settings_json(set), "spk_path": set.spk,
        }),
        Cmd::Crossings {
            body,
            point,
            aspects,
            start,
            end,
            touch_orb,
            tolerance_seconds,
            set,
        } => json!({
            "op": "crossings", "body": body, "point_deg": point, "aspects": aspects,
            "start_utc": start, "end_utc": end, "touch_orb_deg": touch_orb,
            "tolerance_seconds": tolerance_seconds, "config": settings_json(set), "spk_path": set.spk,
        }),
        Cmd::Rulesets => json!({"op": "rulesets"}),
        Cmd::Methods => json!({"op": "methods"}),
        Cmd::Request { file } => {
            let mut s = String::new();
            let r = if file == "-" {
                std::io::stdin().read_to_string(&mut s).map(|_| ())
            } else {
                std::fs::read_to_string(file).map(|t| s = t)
            };
            if let Err(e) = r {
                eprintln!("cannot read request: {e}");
                return ExitCode::from(2);
            }
            match serde_json::from_str(&s) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("request is not valid JSON: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        Cmd::Doctor => json!({"op": "info"}),
        Cmd::Explain { code } => {
            let hit = EshanaError::catalog()
                .iter()
                .find(|(c, _)| c.eq_ignore_ascii_case(code));
            return match hit {
                Some((c, m)) => {
                    println!("{c}: {m}\nSee docs/reference/errors.md for the cause and the fix.");
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!(
                        "unknown code '{code}'. Known: {}",
                        EshanaError::catalog()
                            .iter()
                            .map(|c| c.0)
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    ExitCode::from(2)
                }
            };
        }
        Cmd::Schema => {
            println!("{}", include_str!("../../../schemas/envelope.schema.json"));
            return ExitCode::SUCCESS;
        }
        Cmd::Kernel { action, target } => return kernel_cmd(action, target.as_deref(), cli.compact),
    };
    let out = engine.execute_json(&strip_nulls(request).to_string());
    let v: Value = serde_json::from_str(&out).expect("engine returns JSON");
    let failed = v["kind"] == "error";
    // Graph exports print their content directly so the output pipes into other tools.
    let printable = match (&cli.cmd, v["content"].as_str()) {
        (
            Cmd::Graph { format, .. } | Cmd::Calendar { format, .. } | Cmd::PairCalendar { format, .. },
            Some(c),
        ) if format != "json" => c.to_string(),
        _ if cli.compact => out.clone(),
        _ => serde_json::to_string_pretty(&v).expect("json"),
    };
    println!("{printable}");
    if failed {
        eprintln!(
            "{}: {}",
            v["error"]["code"].as_str().unwrap_or("ESH-?"),
            v["error"]["message"].as_str().unwrap_or("")
        );
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}
