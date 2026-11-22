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
    /// Local date YYYY-MM-DD
    #[arg(long)]
    date: Option<String>,
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
    /// lahiri, raman, krishnamurti, fagan_bradley. Required with --zodiac sidereal; only `chart vedic` defaults to lahiri, and says so
    #[arg(long)]
    ayanamsha: Option<String>,
    /// whole_sign (default), equal, placidus
    #[arg(long)]
    houses: Option<String>,
    /// mean or true
    #[arg(long)]
    node: Option<String>,
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
    /// Print the built-in YAML rule packs
    Rulesets,
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
           "utc_offset_minutes": m.offset_minutes, "fold": m.fold, "uncertainty_minutes": m.uncertainty_minutes})
}

fn chart_json(o: &ChartOpts, system: Option<&str>, assume: bool) -> Value {
    let place = match (o.when.lat, o.when.lon) {
        (Some(la), Some(lo)) => json!({"latitude": la, "longitude": lo}),
        _ => Value::Null,
    };
    json!({
        "when": when_json(&o.when), "place": place, "system": system,
        "config": {"provider": o.set.provider, "zodiac": o.set.zodiac, "ayanamsha": o.set.ayanamsha,
                   "house_system": o.set.houses, "node_model": o.set.node},
        "spk_path": o.set.spk, "bodies": o.bodies, "divisions": o.divisions, "dasha": o.dasha,
        "year_convention": o.year_convention, "aspects": o.aspects, "orb_scale": o.orb_scale,
        "assume_defaults": assume, "features": o.features, "dasha_levels": o.dasha_levels,
        "dasha_systems": o.dasha_systems, "reference_utc": o.reference_utc, "sunrise": o.sunrise,
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
        Cmd::Rulesets => json!({"op": "rulesets"}),
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
    };
    let out = engine.execute_json(&strip_nulls(request).to_string());
    let v: Value = serde_json::from_str(&out).expect("engine returns JSON");
    let failed = v["kind"] == "error";
    // Graph exports print their content directly so the output pipes into other tools.
    let printable = match (&cli.cmd, v["content"].as_str()) {
        (Cmd::Graph { format, .. }, Some(c)) if format != "json" => c.to_string(),
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
