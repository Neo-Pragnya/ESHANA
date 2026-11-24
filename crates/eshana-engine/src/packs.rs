//! Data-driven rule packs (ADR-004): YAML files in `rulesets/`, compiled into the engine with `include_str!`,
//! parsed once and validated. A pack is data; the code here only reads it.

use eshana_core::{EshanaError, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Raw YAML of every built-in pack, by file name. Exposed so tools can print or diff them.
pub const SOURCES: [(&str, &str); 13] = [
    (
        "western-dignities.yaml",
        include_str!("../rulesets/western-dignities.yaml"),
    ),
    ("gematria.yaml", include_str!("../rulesets/gematria.yaml")),
    ("lal-kitab.yaml", include_str!("../rulesets/lal-kitab.yaml")),
    ("vedic-core.yaml", include_str!("../rulesets/vedic-core.yaml")),
    (
        "vedic-dignity.yaml",
        include_str!("../rulesets/vedic-dignity.yaml"),
    ),
    (
        "vedic-relationships.yaml",
        include_str!("../rulesets/vedic-relationships.yaml"),
    ),
    (
        "vedic-drishti.yaml",
        include_str!("../rulesets/vedic-drishti.yaml"),
    ),
    (
        "vedic-strength.yaml",
        include_str!("../rulesets/vedic-strength.yaml"),
    ),
    ("ashtakavarga.yaml", include_str!("../rulesets/ashtakavarga.yaml")),
    ("panchanga.yaml", include_str!("../rulesets/panchanga.yaml")),
    ("matching.yaml", include_str!("../rulesets/matching.yaml")),
    ("yogas.yaml", include_str!("../rulesets/yogas.yaml")),
    ("bala.yaml", include_str!("../rulesets/bala.yaml")),
];

fn parse<T: for<'de> Deserialize<'de>>(name: &str) -> T {
    let src = SOURCES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, s)| *s)
        .expect("pack listed");
    // A built-in pack that does not parse is a build defect, caught by the pack tests before release.
    serde_norway::from_str(src).unwrap_or_else(|e| panic!("rule pack {name} is invalid: {e}"))
}

/// Pack id with version, as recorded in provenance.
#[must_use]
pub fn rule_id(id: &str, version: &str) -> String {
    format!("{id}@{version}")
}

// ── vedic-core ───────────────────────────────────────────────────────────────────────────────────────────────────
/// Exaltation, debilitation, own signs and moolatrikona range of one graha.
#[derive(Debug, Clone, Deserialize)]
pub struct Dignity {
    /// `[sign, degree]` of exaltation.
    pub exaltation: [f64; 2],
    /// `[sign, degree]` of debilitation.
    pub debilitation: [f64; 2],
    /// Own signs.
    pub own: Vec<usize>,
    /// `[sign, from_degree, to_degree]`.
    pub moolatrikona: Option<[f64; 3]>,
}

/// Natural relationships of one graha.
#[derive(Debug, Clone, Deserialize)]
pub struct Friendship {
    /// Friends.
    pub friends: Vec<String>,
    /// Neutral.
    pub neutral: Vec<String>,
    /// Enemies.
    pub enemies: Vec<String>,
}

/// Graha drishti table.
#[derive(Debug, Clone, Deserialize)]
pub struct Drishti {
    /// Houses every graha aspects.
    pub all: Vec<usize>,
    /// Additional houses per graha.
    pub special: BTreeMap<String, Vec<usize>>,
}

/// Combustion orb: one value, or separate direct/retrograde values.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Orb {
    /// Same orb in both motions.
    One(f64),
    /// Different orbs.
    Two {
        /// Direct motion.
        direct: f64,
        /// Retrograde motion.
        retrograde: f64,
    },
}

/// `vedic-core.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct VedicCore {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Named conventions.
    pub conventions: BTreeMap<String, String>,
    /// Lord of each sign.
    pub sign_lords: Vec<String>,
    /// The nine grahas.
    pub grahas: Vec<String>,
    /// Dignities.
    pub dignity: BTreeMap<String, Dignity>,
    /// Natural relationships.
    pub natural_friendship: BTreeMap<String, Friendship>,
    /// Houses that make temporary friends.
    pub temporary_friend_houses: Vec<usize>,
    /// Graha drishti.
    pub drishti: Drishti,
    /// Combustion orbs.
    pub combustion_orbs: BTreeMap<String, Orb>,
    /// Point of full directional strength.
    pub dig_bala_strong_point: BTreeMap<String, String>,
    /// Natural strength.
    pub naisargika_bala: BTreeMap<String, f64>,
    /// Kalas for the Indu lagna.
    pub indu_kalas: BTreeMap<String, u32>,
    /// Yogi point offset from Sun + Moon.
    pub yogi_offset_deg: f64,
    /// Avayogi point offset from the yogi point.
    pub avayogi_offset_deg: f64,
    /// Dagdha rashis by tithi number in the paksha.
    pub dagdha_rashi_by_tithi: BTreeMap<u32, Vec<usize>>,
    /// Natural benefics.
    pub natural_benefics: Vec<String>,
    /// Natural malefics.
    pub natural_malefics: Vec<String>,
    /// Functional nature by house ownership.
    pub functional: Functional,
}

/// Functional nature by the houses a graha owns from the lagna.
#[derive(Debug, Clone, Deserialize)]
pub struct Functional {
    /// Houses whose lords are functional benefics.
    pub benefic_houses: Vec<usize>,
    /// Houses whose lords are functional malefics.
    pub malefic_houses: Vec<usize>,
    /// Neutral houses.
    pub neutral_houses: Vec<usize>,
}

/// One vimshopaka scheme: division -> weight.
pub type Scheme = BTreeMap<u32, f64>;

/// Vimshopaka tables.
#[derive(Debug, Clone, Deserialize)]
pub struct Vimshopaka {
    /// Points by dignity.
    pub points: BTreeMap<String, f64>,
    /// Schemes by name.
    pub schemes: BTreeMap<String, Scheme>,
}

/// `bala.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Bala {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Named conventions.
    pub conventions: BTreeMap<String, String>,
    /// Minimum rupas.
    pub required_rupas: BTreeMap<String, f64>,
    /// Saptavarga divisions.
    pub saptavarga_divisions: Vec<u32>,
    /// Saptavarga points.
    pub saptavarga_points: BTreeMap<String, f64>,
    /// Grahas strong in even signs.
    pub odd_even_females: Vec<String>,
    /// Kendradi points.
    pub kendradi_points: BTreeMap<String, f64>,
    /// Drekkana gender.
    pub drekkana_gender: BTreeMap<String, String>,
    /// Tribhaga lords.
    pub tribhaga: BTreeMap<String, Vec<String>>,
    /// Day-strong grahas.
    pub day_strong: Vec<String>,
    /// Night-strong grahas.
    pub night_strong: Vec<String>,
    /// Paksha benefics.
    pub paksha_benefics: Vec<String>,
    /// Strong in northern declination.
    pub ayana_north_strong: Vec<String>,
    /// Strong in southern declination.
    pub ayana_south_strong: Vec<String>,
    /// Chesta points by motion state.
    pub chesta_points: BTreeMap<String, f64>,
    /// Mean daily motion.
    pub mean_daily_motion: BTreeMap<String, f64>,
    /// Drik benefics.
    pub drik_benefics: Vec<String>,
    /// Drik malefics.
    pub drik_malefics: Vec<String>,
    /// Bhava dig strong house by sign nature.
    pub bhava_dig_strong_house: BTreeMap<String, usize>,
    /// Sign nature (with 15-degree splits).
    pub sign_nature: Vec<VashyaGroup>,
    /// Vimshopaka tables.
    pub vimshopaka: Vimshopaka,
}

// ── ashtakavarga ─────────────────────────────────────────────────────────────────────────────────────────────────
/// `ashtakavarga.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Ashtakavarga {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Expected totals per graha.
    pub totals: BTreeMap<String, u32>,
    /// graha -> contributor -> house counts.
    pub tables: BTreeMap<String, BTreeMap<String, Vec<usize>>>,
}

// ── panchanga ────────────────────────────────────────────────────────────────────────────────────────────────────
/// A weekday.
#[derive(Debug, Clone, Deserialize)]
pub struct Vara {
    /// Sanskrit name.
    pub name: String,
    /// English name.
    pub english: String,
    /// Graha lord.
    pub lord: String,
}

/// Choghadiya table.
#[derive(Debug, Clone, Deserialize)]
pub struct Choghadiya {
    /// The seven names.
    pub names: Vec<String>,
    /// Quality of each name.
    pub quality: BTreeMap<String, String>,
    /// First day index per weekday.
    pub day_start: Vec<usize>,
    /// First night index per weekday.
    pub night_start: Vec<usize>,
    /// Day step.
    pub day_step: i32,
    /// Night step.
    pub night_step: i32,
}

/// `panchanga.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Panchanga {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// 30 tithi names.
    pub tithis: Vec<String>,
    /// 27 yoga names.
    pub yogas: Vec<String>,
    /// First karana.
    pub karana_first: String,
    /// Seven movable karanas.
    pub karana_movable: Vec<String>,
    /// Last three karanas.
    pub karana_last: Vec<String>,
    /// Weekdays, Sunday first.
    pub varas: Vec<Vara>,
    /// Rahu kalam part per weekday.
    pub rahu_kalam: Vec<usize>,
    /// Yamagandam part per weekday.
    pub yamagandam: Vec<usize>,
    /// Gulika kalam part per weekday.
    pub gulika_kalam: Vec<usize>,
    /// Hora lord order.
    pub hora_order: Vec<String>,
    /// Choghadiya.
    pub choghadiya: Choghadiya,
    /// The nine taras.
    pub taras: Vec<String>,
    /// Favourable taras.
    pub tara_favourable: Vec<String>,
    /// Favourable Chandra bala houses.
    pub chandra_bala_favourable: Vec<usize>,
}

// ── matching ─────────────────────────────────────────────────────────────────────────────────────────────────────
/// A scoring table with named rows and columns.
#[derive(Debug, Clone, Deserialize)]
pub struct Table {
    /// Row and column names.
    pub order: Vec<String>,
    /// Scores, rows = partner a.
    pub scores: Vec<Vec<f64>>,
}

/// One vashya entry: a group, or two groups split at 15°.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VashyaGroup {
    /// Whole sign.
    One(String),
    /// First and second half.
    Split(Vec<String>),
}

/// `matching.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Matching {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Maximum score per koota.
    pub max: BTreeMap<String, f64>,
    /// Varna rank by sign.
    pub varna_rank: Vec<u8>,
    /// Varna names by rank.
    pub varna_names: BTreeMap<u8, String>,
    /// Vashya group by sign.
    pub vashya_groups: Vec<VashyaGroup>,
    /// Vashya scores.
    pub vashya_table: Table,
    /// Tara remainders that score 0.
    pub tara_bad_remainders: Vec<usize>,
    /// Yoni by nakshatra.
    pub yoni: Vec<String>,
    /// Yoni order of the matrix.
    pub yoni_order: Vec<String>,
    /// Yoni matrix.
    pub yoni_matrix: Vec<Vec<f64>>,
    /// Graha maitri scores.
    pub graha_maitri: BTreeMap<String, f64>,
    /// Gana by nakshatra.
    pub gana: Vec<String>,
    /// Gana scores.
    pub gana_table: Table,
    /// Bhakoot distances that score 0.
    pub bhakoot_dosha_pairs: Vec<[usize; 2]>,
    /// Nadi by nakshatra.
    pub nadi: Vec<String>,
}

// ── yogas ────────────────────────────────────────────────────────────────────────────────────────────────────────
/// A yoga condition (see the header of `yogas.yaml`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cond {
    /// Graha in houses from a reference.
    InHouse {
        /// Graha.
        graha: String,
        /// Reference.
        from: String,
        /// Houses.
        houses: Vec<usize>,
    },
    /// Graha dignity.
    Dignity {
        /// Graha.
        graha: String,
        /// Accepted dignities.
        any_of: Vec<String>,
    },
    /// Grahas in one sign.
    SameSign(Vec<String>),
    /// Houses occupied.
    Occupied {
        /// Reference.
        from: String,
        /// Houses.
        houses: Vec<usize>,
        /// Minimum count.
        #[serde(default)]
        min: Option<usize>,
        /// Grahas not counted.
        #[serde(default)]
        exclude: Vec<String>,
    },
    /// Houses empty.
    Empty {
        /// Reference.
        from: String,
        /// Houses.
        houses: Vec<usize>,
        /// Grahas not counted.
        #[serde(default)]
        exclude: Vec<String>,
    },
    /// Lords of two house groups conjoined.
    LordsConjoined {
        /// First group.
        houses_a: Vec<usize>,
        /// Second group.
        houses_b: Vec<usize>,
        /// Reference.
        from: String,
    },
    /// Lord of a house placed in houses.
    LordInHouse {
        /// House whose lord is checked.
        house: usize,
        /// Reference.
        from: String,
        /// Houses.
        houses: Vec<usize>,
    },
    /// Dignity of a house lord.
    LordDignity {
        /// House whose lord is checked.
        house: usize,
        /// Reference.
        from: String,
        /// Accepted dignities.
        any_of: Vec<String>,
    },
    /// All seven grahas on one side of the nodal axis.
    BetweenNodes {},
    /// Two of the seven grahas each in a sign ruled by the other (sign exchange). `kind` narrows it by the houses
    /// exchanged from `from`: `maha` (neither house is 3, 6, 8 or 12), `khala` (one is the 3rd, none is 6, 8 or 12),
    /// `dainya` (one is the 6th, 8th or 12th); `any` (the default) needs no lagna.
    Parivartana {
        /// any | maha | khala | dainya.
        #[serde(default)]
        kind: Option<String>,
        /// Reference for the houses (needed unless kind is any).
        #[serde(default)]
        from: Option<String>,
    },
    /// Every condition holds.
    AllOf(Vec<Cond>),
    /// Any condition holds.
    AnyOf(Vec<Cond>),
    /// The condition does not hold.
    Not(Box<Cond>),
    /// Per-graha evaluation of a condition with `$g`.
    ForEachGraha {
        /// Grahas to test.
        grahas: Vec<String>,
        /// Condition with `$g` placeholders.
        condition: Box<Cond>,
    },
}

/// One yoga or dosha definition.
#[derive(Debug, Clone, Deserialize)]
pub struct YogaDef {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Category.
    pub category: String,
    /// The rule, in words.
    pub rule: String,
    /// Condition. Written as plain YAML maps (`{in_house: {...}}`), read with singleton-map support.
    #[serde(with = "serde_norway::with::singleton_map_recursive")]
    pub when: Cond,
}

/// `yogas.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Yogas {
    /// Pack id.
    pub id: String,
    /// Pack version.
    pub version: String,
    /// Yogas.
    pub yogas: Vec<YogaDef>,
    /// Doshas.
    pub doshas: Vec<YogaDef>,
}

// ── access ───────────────────────────────────────────────────────────────────────────────────────────────────────
/// The Vedic packs split from `vedic.core` (VED-05), each with its own id and version.
pub const VEDIC_SPLIT: [&str; 4] = [
    "vedic-dignity.yaml",
    "vedic-relationships.yaml",
    "vedic-drishti.yaml",
    "vedic-strength.yaml",
];

/// `id@version` of one of the split packs, by file name.
///
/// # Panics
/// When the file is not a built-in pack (a build defect).
pub fn split_rule(file: &str) -> String {
    #[derive(Deserialize)]
    struct Head {
        id: String,
        version: String,
    }
    let h: Head = parse(file);
    rule_id(&h.id, &h.version)
}

/// `vedic-core.yaml` with the four split packs merged in (their tables keep their keys; their `conventions` join the
/// core's), parsed once.
pub fn core() -> &'static VedicCore {
    static P: OnceLock<VedicCore> = OnceLock::new();
    P.get_or_init(|| {
        let read = |name: &str| -> serde_norway::Mapping {
            let src = SOURCES
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, s)| *s)
                .expect("pack listed");
            serde_norway::from_str(src).unwrap_or_else(|e| panic!("rule pack {name} is invalid: {e}"))
        };
        let mut merged = read("vedic-core.yaml");
        for f in VEDIC_SPLIT {
            for (k, v) in read(f) {
                match k.as_str() {
                    Some("id" | "version") => {}
                    Some("conventions") => {
                        if let (Some(serde_norway::Value::Mapping(c)), serde_norway::Value::Mapping(more)) =
                            (merged.get_mut("conventions"), v)
                        {
                            c.extend(more);
                        }
                    }
                    _ => {
                        merged.insert(k, v);
                    }
                }
            }
        }
        serde_norway::from_value(serde_norway::Value::Mapping(merged))
            .unwrap_or_else(|e| panic!("vedic core packs are invalid: {e}"))
    })
}
/// `ashtakavarga.yaml`, parsed once.
pub fn ashtakavarga() -> &'static Ashtakavarga {
    static P: OnceLock<Ashtakavarga> = OnceLock::new();
    P.get_or_init(|| parse("ashtakavarga.yaml"))
}
/// `panchanga.yaml`, parsed once.
pub fn panchanga() -> &'static Panchanga {
    static P: OnceLock<Panchanga> = OnceLock::new();
    P.get_or_init(|| parse("panchanga.yaml"))
}
/// `matching.yaml`, parsed once.
pub fn matching() -> &'static Matching {
    static P: OnceLock<Matching> = OnceLock::new();
    P.get_or_init(|| parse("matching.yaml"))
}
/// `bala.yaml`, parsed once.
pub fn bala() -> &'static Bala {
    static P: OnceLock<Bala> = OnceLock::new();
    P.get_or_init(|| parse("bala.yaml"))
}
/// `yogas.yaml`, parsed once.
pub fn yogas() -> &'static Yogas {
    static P: OnceLock<Yogas> = OnceLock::new();
    P.get_or_init(|| parse("yogas.yaml"))
}

/// A user's yoga pack (RULE-02): the `yogas.yaml` format, parsed with the same condition grammar, then checked: at
/// most 64 KiB and 200 definitions, an id of lowercase letters, digits, `.`, `_` or `-` that does not claim the
/// built-in `vedic.` namespace, a version, and unique definition ids.
///
/// # Errors
/// `ESH-RULES` naming the problem (including the parser's message for a condition it does not know).
pub fn user_yogas(src: &str) -> Result<Yogas> {
    let bad = |m: String| EshanaError::Rules(format!("user rule pack: {m}"));
    if src.len() > 64 * 1024 {
        return Err(bad(format!("{} bytes; the limit is 64 KiB", src.len())));
    }
    let y: Yogas = serde_norway::from_str(src).map_err(|e| bad(e.to_string()))?;
    let id_ok = !y.id.is_empty()
        && y.id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'));
    if !id_ok || y.id.starts_with("vedic.") {
        return Err(bad(format!("id '{}' must be lowercase letters, digits, '.', '_' or '-', and not in the built-in vedic. namespace", y.id)));
    }
    if y.version.trim().is_empty() {
        return Err(bad("a version is required".into()));
    }
    let n = y.yogas.len() + y.doshas.len();
    if n == 0 || n > 200 {
        return Err(bad(format!("{n} definitions; give 1 to 200")));
    }
    let mut seen = std::collections::BTreeSet::new();
    for d in y.yogas.iter().chain(&y.doshas) {
        if !seen.insert(d.id.as_str()) {
            return Err(bad(format!("definition id '{}' appears twice", d.id)));
        }
    }
    Ok(y)
}

/// Check every built-in pack: it parses and its fixed facts hold (sizes, totals, symmetric matrices).
///
/// # Errors
/// [`EshanaError::Rules`] naming the first problem.
pub fn validate() -> Result<()> {
    let bad = |m: String| Err(EshanaError::Rules(m));
    let c = core();
    if c.sign_lords.len() != 12 {
        return bad("vedic-core: sign_lords must have 12 entries".into());
    }
    let a = ashtakavarga();
    for (g, contributors) in &a.tables {
        let total: usize = contributors.values().map(Vec::len).sum();
        if a.totals.get(g).copied() != Some(total as u32) {
            return bad(format!(
                "ashtakavarga: {g} gives {total} bindus, expected {:?}",
                a.totals.get(g)
            ));
        }
    }
    let p = panchanga();
    if p.tithis.len() != 30 || p.yogas.len() != 27 || p.varas.len() != 7 || p.karana_movable.len() != 7 {
        return bad("panchanga: wrong table size".into());
    }
    let m = matching();
    if m.yoni.len() != 27 || m.gana.len() != 27 || m.nadi.len() != 27 || m.vashya_groups.len() != 12 {
        return bad("matching: per-nakshatra or per-sign table has the wrong size".into());
    }
    let n = m.yoni_order.len();
    for i in 0..n {
        for j in 0..n {
            if (m.yoni_matrix[i][j] - m.yoni_matrix[j][i]).abs() > 1e-9 {
                return bad(format!("matching: yoni matrix not symmetric at {i},{j}"));
            }
        }
    }
    let max: f64 = m.max.values().sum();
    if (max - 36.0).abs() > 1e-9 {
        return bad(format!("matching: maxima sum to {max}, expected 36"));
    }
    let _ = yogas();
    let b = bala();
    for (name, scheme) in &b.vimshopaka.schemes {
        let total: f64 = scheme.values().sum();
        if (total - 20.0).abs() > 1e-9 {
            return bad(format!(
                "bala: vimshopaka scheme {name} weighs {total}, expected 20"
            ));
        }
    }
    Ok(())
}
