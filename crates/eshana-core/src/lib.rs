//! ESHANA core: typed angles, configuration enums, errors with stable codes, epistemic tags and provenance.
//!
//! The doctrine is **Compute → Derive → Interpret**. Everything here exists to keep those three layers apart
//! and to make each result carry the settings that produced it.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Version of the output contract (the JSON envelope). Bumped independently of the engine version.
pub const SCHEMA_VERSION: &str = "0.4.0";

/// Canonical expansion of the acronym. One expansion everywhere; never vary it between manifests and docs.
pub const EXPANSION: &str =
    "Extensible Spatiotemporal Hypersystem for Astroinformatics, Natural Cycles & Associations";

/// Tagline.
pub const TAGLINE: &str = "From celestial positions to connected meaning.";

// ───────────────────────────── angles ─────────────────────────────

/// Normalise an angle in degrees to `[0, 360)`.
#[must_use]
pub fn norm360(x: f64) -> f64 {
    let r = x.rem_euclid(360.0);
    // rem_euclid can return exactly 360.0 for tiny negative inputs because of rounding.
    if r >= 360.0 {
        0.0
    } else {
        r
    }
}

/// Normalise an angle in degrees to `[-180, 180)`.
#[must_use]
pub fn norm180(x: f64) -> f64 {
    let r = norm360(x);
    if r >= 180.0 {
        r - 360.0
    } else {
        r
    }
}

/// Smallest signed difference `a - b` in degrees, in `[-180, 180)`.
#[must_use]
pub fn angular_diff(a: f64, b: f64) -> f64 {
    norm180(a - b)
}

/// Degrees to radians.
#[must_use]
pub fn rad(deg: f64) -> f64 {
    deg.to_radians()
}

/// Radians to degrees.
#[must_use]
pub fn deg(rad: f64) -> f64 {
    rad.to_degrees()
}

/// Sexagesimal rendering of a longitude within its sign, e.g. `17°23′05″`.
#[must_use]
pub fn dms_in_sign(lon_deg: f64) -> String {
    let within = norm360(lon_deg) % 30.0;
    let total_arcsec = (within * 3600.0).round() as i64;
    let d = total_arcsec / 3600;
    let m = (total_arcsec % 3600) / 60;
    let s = total_arcsec % 60;
    format!("{d}°{m:02}′{s:02}″")
}

// ───────────────────────────── errors ─────────────────────────────

/// Every error carries a stable machine-readable code. Add a code here before using it.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EshanaError {
    /// Malformed or out-of-range input.
    #[error("invalid input: {0}")]
    Input(String),
    /// Conflicting or incomplete configuration (for example sidereal without an ayanamsha).
    #[error("invalid configuration: {0}")]
    Config(String),
    /// The IANA zone name is not in the bundled tz database.
    #[error("unknown time zone: {0}")]
    TimeZoneUnknown(String),
    /// The local time occurs twice (DST fall-back) and no policy was given.
    #[error("ambiguous local time: {0}")]
    TimeAmbiguous(String),
    /// The local time never occurred (DST spring-forward gap).
    #[error("non-existent local time: {0}")]
    TimeNonexistent(String),
    /// The result needs a precise birth time but only a window or a date was given.
    #[error("birth time too uncertain: {0}")]
    TimeUncertain(String),
    /// The instant is outside what the provider can compute.
    #[error("outside ephemeris range: {0}")]
    OutOfRange(String),
    /// The provider cannot supply this body.
    #[error("body not supported by provider: {0}")]
    BodyUnsupported(String),
    /// Houses cannot be defined at this latitude for this system. Never extrapolated silently.
    #[error("unsupported latitude: {0}")]
    LatitudeUnsupported(String),
    /// The ephemeris provider failed (file missing, corrupt, wrong format).
    #[error("ephemeris provider error: {0}")]
    Provider(String),
    /// Rule-pack or numerology policy problem.
    #[error("rules error: {0}")]
    Rules(String),
    /// The method id is not one the engine implements (see the `methods` operation).
    #[error("unsupported method: {0}")]
    MethodUnsupported(String),
    /// A range is empty or reversed (end not after start).
    #[error("invalid range: {0}")]
    RangeInvalid(String),
    /// The request would exceed a documented work limit (span, rows, results).
    #[error("work limit exceeded: {0}")]
    WorkLimit(String),
}

impl EshanaError {
    /// Stable code, for programs and for the documentation.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Input(_) => "ESH-INPUT",
            Self::Config(_) => "ESH-CONFIG",
            Self::TimeZoneUnknown(_) => "ESH-TZ-UNKNOWN",
            Self::TimeAmbiguous(_) => "ESH-TIME-AMBIGUOUS",
            Self::TimeNonexistent(_) => "ESH-TIME-NONEXISTENT",
            Self::TimeUncertain(_) => "ESH-TIME-UNCERTAIN",
            Self::OutOfRange(_) => "ESH-RANGE",
            Self::BodyUnsupported(_) => "ESH-BODY",
            Self::LatitudeUnsupported(_) => "ESH-LATITUDE",
            Self::Provider(_) => "ESH-PROVIDER",
            Self::Rules(_) => "ESH-RULES",
            Self::MethodUnsupported(_) => "ESH-METHOD",
            Self::RangeInvalid(_) => "ESH-RANGE-INVALID",
            Self::WorkLimit(_) => "ESH-LIMIT",
        }
    }

    /// Every code with a one-line meaning, for `eshana doctor` and the reference docs.
    #[must_use]
    pub fn catalog() -> &'static [(&'static str, &'static str)] {
        &[
            ("ESH-INPUT", "Malformed or out-of-range input"),
            ("ESH-CONFIG", "Conflicting or incomplete configuration"),
            ("ESH-TZ-UNKNOWN", "Unknown IANA time zone"),
            (
                "ESH-TIME-AMBIGUOUS",
                "Local time occurs twice; give a fold policy",
            ),
            ("ESH-TIME-NONEXISTENT", "Local time was skipped by a DST change"),
            ("ESH-TIME-UNCERTAIN", "Result needs a more precise birth time"),
            ("ESH-RANGE", "Instant outside the provider's range"),
            ("ESH-BODY", "Provider cannot supply this body"),
            ("ESH-LATITUDE", "Houses undefined at this latitude"),
            ("ESH-PROVIDER", "Ephemeris provider failure"),
            ("ESH-RULES", "Rule-pack or numerology policy problem"),
            (
                "ESH-METHOD",
                "Method id not implemented; list them with the methods operation",
            ),
            ("ESH-RANGE-INVALID", "Range end is not after its start"),
            ("ESH-LIMIT", "Request exceeds a documented work limit"),
        ]
    }
}

/// Result alias.
pub type Result<T> = std::result::Result<T, EshanaError>;

// ───────────────────────────── epistemic layers ─────────────────────────────

/// The three-layer epistemic model, with the interpretive layer split into two honest labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Epistemic {
    /// Mathematically verifiable against reference data (positions, Julian days, separations).
    Calculated,
    /// Deterministic under a named tradition, method and rule version. Traditions can disagree.
    TraditionRule,
    /// Symbolic or narrative reading. A belief, not an established causal fact.
    SpeculativeInterpretation,
    /// A claim offered for testing, with no validation implied.
    ResearchHypothesis,
}

// ───────────────────────────── bodies and configuration ─────────────────────────────

/// A celestial body or chart point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Body {
    /// Sun.
    Sun,
    /// Moon.
    Moon,
    /// Mercury.
    Mercury,
    /// Venus.
    Venus,
    /// Mars.
    Mars,
    /// Jupiter.
    Jupiter,
    /// Saturn.
    Saturn,
    /// Uranus.
    Uranus,
    /// Neptune.
    Neptune,
    /// Pluto.
    Pluto,
    /// Mean lunar ascending node.
    MeanNode,
    /// True (osculating-corrected) lunar ascending node.
    TrueNode,
    /// (1) Ceres. Optional: needs an SPK kernel with it (sb441-n16).
    Ceres,
    /// (2) Pallas. Optional, as Ceres.
    Pallas,
    /// (3) Juno. Optional, as Ceres.
    Juno,
    /// (4) Vesta. Optional, as Ceres.
    Vesta,
    /// (2060) Chiron. Optional: needs an SPK kernel with it (for example one generated by JPL Horizons).
    Chiron,
}

impl Body {
    /// Optional bodies, never in a default chart: they need a kernel that carries them.
    pub const OPTIONAL: [Body; 5] = [Body::Ceres, Body::Pallas, Body::Juno, Body::Vesta, Body::Chiron];

    /// The ten classical and modern planets, in conventional order.
    pub const PLANETS: [Body; 10] = [
        Body::Sun,
        Body::Moon,
        Body::Mercury,
        Body::Venus,
        Body::Mars,
        Body::Jupiter,
        Body::Saturn,
        Body::Uranus,
        Body::Neptune,
        Body::Pluto,
    ];

    /// Stable snake_case id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Body::Sun => "sun",
            Body::Moon => "moon",
            Body::Mercury => "mercury",
            Body::Venus => "venus",
            Body::Mars => "mars",
            Body::Jupiter => "jupiter",
            Body::Saturn => "saturn",
            Body::Uranus => "uranus",
            Body::Neptune => "neptune",
            Body::Pluto => "pluto",
            Body::MeanNode => "mean_node",
            Body::TrueNode => "true_node",
            Body::Ceres => "ceres",
            Body::Pallas => "pallas",
            Body::Juno => "juno",
            Body::Vesta => "vesta",
            Body::Chiron => "chiron",
        }
    }

    /// Parse an id such as `"mars"`.
    ///
    /// # Errors
    /// Returns [`EshanaError::Input`] for an unknown id.
    pub fn parse(s: &str) -> Result<Body> {
        let s = s.trim().to_ascii_lowercase();
        let all = [
            Body::PLANETS.as_slice(),
            &[Body::MeanNode, Body::TrueNode],
            Body::OPTIONAL.as_slice(),
        ]
        .concat();
        all.into_iter()
            .find(|b| b.id() == s)
            .ok_or_else(|| EshanaError::Input(format!("unknown body '{s}'")))
    }

    /// True for the lunar nodes, which are points rather than bodies.
    #[must_use]
    pub fn is_node(self) -> bool {
        matches!(self, Body::MeanNode | Body::TrueNode)
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// Tropical or sidereal zodiac. Never merged silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZodiacSystem {
    /// Measured from the equinox of date.
    Tropical,
    /// Measured from a fixed-star reference; requires an [`Ayanamsha`].
    Sidereal,
}

/// Named ayanāṃśa (precession offset) models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ayanamsha {
    /// Lahiri / Chitrapaksha (Indian Calendar Reform Committee, 1956).
    Lahiri,
    /// B. V. Raman.
    Raman,
    /// Krishnamurti (KP).
    Krishnamurti,
    /// Fagan–Bradley (Western sidereal).
    FaganBradley,
    /// Sri Yukteshwar (The Holy Science).
    Yukteshwar,
    /// J. N. Bhasin.
    JnBhasin,
    /// Ushā–Shashi (Revatī-based).
    Ushashashi,
    /// Sassanian (zero point in AD 564).
    Sassanian,
    /// De Luce.
    Deluce,
    /// Djwhal Khul.
    DjwhalKhul,
    /// True Citrā: Spica's apparent position held at 180° (AYA-02).
    TrueCitra,
    /// True Revatī: ζ Piscium held at 359°50′.
    TrueRevati,
    /// True Puṣya: δ Cancri held at 106°.
    TruePushya,
    /// True Mūla: λ Scorpii held at 240°.
    TrueMula,
}

impl Ayanamsha {
    /// Every supported model.
    pub const ALL: [Ayanamsha; 14] = [
        Ayanamsha::Lahiri,
        Ayanamsha::Raman,
        Ayanamsha::Krishnamurti,
        Ayanamsha::FaganBradley,
        Ayanamsha::Yukteshwar,
        Ayanamsha::JnBhasin,
        Ayanamsha::Ushashashi,
        Ayanamsha::Sassanian,
        Ayanamsha::Deluce,
        Ayanamsha::DjwhalKhul,
        Ayanamsha::TrueCitra,
        Ayanamsha::TrueRevati,
        Ayanamsha::TruePushya,
        Ayanamsha::TrueMula,
    ];

    /// Stable id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Ayanamsha::Lahiri => "lahiri",
            Ayanamsha::Raman => "raman",
            Ayanamsha::Krishnamurti => "krishnamurti",
            Ayanamsha::FaganBradley => "fagan_bradley",
            Ayanamsha::Yukteshwar => "yukteshwar",
            Ayanamsha::JnBhasin => "jn_bhasin",
            Ayanamsha::Ushashashi => "ushashashi",
            Ayanamsha::Sassanian => "sassanian",
            Ayanamsha::Deluce => "deluce",
            Ayanamsha::DjwhalKhul => "djwhal_khul",
            Ayanamsha::TrueCitra => "true_citra",
            Ayanamsha::TrueRevati => "true_revati",
            Ayanamsha::TruePushya => "true_pushya",
            Ayanamsha::TrueMula => "true_mula",
        }
    }

    /// Parse an id.
    ///
    /// # Errors
    /// [`EshanaError::Input`] for an unknown model.
    pub fn parse(s: &str) -> Result<Ayanamsha> {
        let s = s.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        let s = match s.as_str() {
            "true_chitra" | "true_chitrapaksha" | "true_citrapaksha" => "true_citra".to_string(),
            _ => s,
        };
        Ayanamsha::ALL
            .into_iter()
            .find(|a| a.id() == s)
            .ok_or_else(|| EshanaError::Input(format!("unknown ayanamsha '{s}'")))
    }
}

/// House system. Latitude limits are enforced, not extrapolated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseSystem {
    /// Each sign is a house; house 1 is the sign of the Ascendant.
    WholeSign,
    /// Equal 30° houses from the Ascendant.
    Equal,
    /// Placidus semi-arc houses; undefined beyond the polar circles.
    Placidus,
    /// Porphyry: each quadrant between the angles trisected in ecliptic longitude.
    Porphyry,
    /// Śrīpati (bhāva chalit): cusps are the midpoints between Porphyry cusps, which become bhāva madhyas.
    Sripati,
    /// Koch (birthplace): cusps are the Ascendants at the sidereal times that trisect the Midheaven degree's
    /// semi-diurnal arc; undefined beyond the polar circles.
    Koch,
    /// Regiomontanus: the celestial equator divided into twelve 30° parts, projected by circles through the
    /// north and south points of the horizon.
    Regiomontanus,
    /// Campanus: the prime vertical divided into twelve 30° parts, projected the same way.
    Campanus,
    /// Alcabitius: the Ascendant degree's diurnal and nocturnal semi-arcs trisected in right ascension.
    Alcabitius,
    /// Topocentric (Polich–Page): Regiomontanus-style circles with pole heights of one and two thirds of the
    /// latitude's tangent.
    Topocentric,
    /// Morinus: the equator divided from the east point, projected along circles of ecliptic longitude.
    Morinus,
    /// Meridian (axial rotation): the equator divided from the Midheaven, projected along circles of declination.
    Meridian,
}

impl HouseSystem {
    /// Stable id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            HouseSystem::WholeSign => "whole_sign",
            HouseSystem::Equal => "equal",
            HouseSystem::Placidus => "placidus",
            HouseSystem::Porphyry => "porphyry",
            HouseSystem::Sripati => "sripati",
            HouseSystem::Koch => "koch",
            HouseSystem::Regiomontanus => "regiomontanus",
            HouseSystem::Campanus => "campanus",
            HouseSystem::Alcabitius => "alcabitius",
            HouseSystem::Topocentric => "topocentric",
            HouseSystem::Morinus => "morinus",
            HouseSystem::Meridian => "meridian",
        }
    }

    /// Parse an id.
    ///
    /// # Errors
    /// [`EshanaError::Input`] for an unknown system.
    pub fn parse(s: &str) -> Result<HouseSystem> {
        match s.trim().to_ascii_lowercase().replace(['-', ' '], "_").as_str() {
            "whole_sign" => Ok(HouseSystem::WholeSign),
            "equal" => Ok(HouseSystem::Equal),
            "placidus" => Ok(HouseSystem::Placidus),
            "porphyry" => Ok(HouseSystem::Porphyry),
            "sripati" | "bhava_chalit" => Ok(HouseSystem::Sripati),
            "koch" => Ok(HouseSystem::Koch),
            "regiomontanus" => Ok(HouseSystem::Regiomontanus),
            "campanus" => Ok(HouseSystem::Campanus),
            "alcabitius" => Ok(HouseSystem::Alcabitius),
            "topocentric" | "polich_page" => Ok(HouseSystem::Topocentric),
            "morinus" => Ok(HouseSystem::Morinus),
            "meridian" | "axial" | "axial_rotation" => Ok(HouseSystem::Meridian),
            other => Err(EshanaError::Input(format!("unknown house system '{other}'"))),
        }
    }
}

/// Mean or true lunar node. The two are never substituted for each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeModel {
    /// Mean node.
    Mean,
    /// True node.
    True,
}

impl NodeModel {
    /// The body that carries this node model.
    #[must_use]
    pub fn body(self) -> Body {
        match self {
            NodeModel::Mean => Body::MeanNode,
            NodeModel::True => Body::TrueNode,
        }
    }
}

/// The coordinate frame every position is expressed in. A single frame in 0.1, named so that later frames
/// are additive rather than silent changes.
pub const FRAME_APPARENT_ECLIPTIC_OF_DATE: &str = "geocentric_apparent_ecliptic_true_equinox_of_date";

/// Everything that can change a number. Echoed in every result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculationConfig {
    /// Provider id, e.g. `"analytic"` or `"spk"`.
    pub provider: String,
    /// Tropical or sidereal.
    pub zodiac: ZodiacSystem,
    /// Required when `zodiac` is sidereal; must be absent when tropical.
    #[serde(default)]
    pub ayanamsha: Option<Ayanamsha>,
    /// House system.
    pub house_system: HouseSystem,
    /// Mean or true node.
    pub node_model: NodeModel,
}

impl CalculationConfig {
    /// Western default: tropical, whole sign, mean node.
    #[must_use]
    pub fn western(provider: &str) -> Self {
        Self {
            provider: provider.into(),
            zodiac: ZodiacSystem::Tropical,
            ayanamsha: None,
            house_system: HouseSystem::WholeSign,
            node_model: NodeModel::Mean,
        }
    }

    /// Vedic default: sidereal with the named ayanāṃśa, whole sign, mean node.
    #[must_use]
    pub fn vedic(provider: &str, ayanamsha: Ayanamsha) -> Self {
        Self {
            provider: provider.into(),
            zodiac: ZodiacSystem::Sidereal,
            ayanamsha: Some(ayanamsha),
            house_system: HouseSystem::WholeSign,
            node_model: NodeModel::Mean,
        }
    }

    /// Reject contradictory settings instead of guessing.
    ///
    /// # Errors
    /// [`EshanaError::Config`] when sidereal has no ayanāṃśa or tropical has one.
    pub fn validate(&self) -> Result<()> {
        match (self.zodiac, self.ayanamsha) {
            (ZodiacSystem::Sidereal, None) => Err(EshanaError::Config(
                "sidereal zodiac requires an explicit ayanamsha (lahiri, raman, krishnamurti, fagan_bradley, …; see `eshana methods`); \
                 ESHANA never picks one silently"
                    .into(),
            )),
            (ZodiacSystem::Tropical, Some(a)) => Err(EshanaError::Config(format!(
                "tropical zodiac cannot take ayanamsha '{}'; choose sidereal or drop it",
                a.id()
            ))),
            _ => Ok(()),
        }
    }
}

// ───────────────────────────── provenance ─────────────────────────────

/// Everything needed to replay a result. `engine_version` is the crate version; `provider_data` names the
/// ephemeris data identity; `tzdb_version` pins historical civil time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// ESHANA engine version.
    pub engine_version: String,
    /// Provider id.
    pub provider: String,
    /// Provider implementation version.
    pub provider_version: String,
    /// What data the provider used (model name, or the kernel file identity).
    pub provider_data: String,
    /// Stated accuracy of the provider, in words and numbers it was measured at.
    pub provider_accuracy: String,
    /// IANA tz database version, when a zone was used.
    pub tzdb_version: Option<String>,
    /// Time-scale model (leap seconds / ΔT polynomial) used.
    pub time_model: String,
    /// Rule packs applied, as `id@version`.
    pub rulesets: Vec<String>,
    /// Every assumption the engine made on the caller's behalf.
    pub assumptions: Vec<String>,
}

/// A value paired with its epistemic layer and the rule that produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedAttribute {
    /// Stable id, e.g. `vedic.moon.nakshatra`.
    pub id: String,
    /// Tradition that governs the derivation, e.g. `vedic.parashari`.
    pub tradition_id: String,
    /// Rule pack and version.
    pub rule_version: String,
    /// Epistemic layer.
    pub epistemic: Epistemic,
    /// The derived value.
    pub value: serde_json::Value,
    /// Ids of the positions or attributes this came from.
    pub derived_from: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm360_wraps() {
        assert_eq!(norm360(-0.0), 0.0);
        assert!((norm360(-1.0) - 359.0).abs() < 1e-12);
        assert!((norm360(721.5) - 1.5).abs() < 1e-12);
        assert!(norm360(-1e-20) < 360.0);
    }

    #[test]
    fn norm180_range() {
        assert!((norm180(190.0) + 170.0).abs() < 1e-12);
        assert!((norm180(-190.0) - 170.0).abs() < 1e-12);
        assert_eq!(norm180(180.0), -180.0);
    }

    #[test]
    fn diff_across_zero() {
        assert!((angular_diff(1.0, 359.0) - 2.0).abs() < 1e-12);
        assert!((angular_diff(359.0, 1.0) + 2.0).abs() < 1e-12);
    }

    #[test]
    fn dms_rendering() {
        assert_eq!(dms_in_sign(47.5), "17°30′00″");
    }

    #[test]
    fn sidereal_needs_ayanamsha() {
        let mut c = CalculationConfig::western("analytic");
        c.zodiac = ZodiacSystem::Sidereal;
        let e = c.validate().unwrap_err();
        assert_eq!(e.code(), "ESH-CONFIG");
        c.ayanamsha = Some(Ayanamsha::Lahiri);
        assert!(c.validate().is_ok());
        c.zodiac = ZodiacSystem::Tropical;
        assert!(c.validate().is_err());
    }

    #[test]
    fn error_catalog_covers_every_code() {
        let codes: Vec<_> = EshanaError::catalog().iter().map(|c| c.0).collect();
        for e in [
            EshanaError::Input(String::new()),
            EshanaError::Config(String::new()),
            EshanaError::TimeZoneUnknown(String::new()),
            EshanaError::TimeAmbiguous(String::new()),
            EshanaError::TimeNonexistent(String::new()),
            EshanaError::TimeUncertain(String::new()),
            EshanaError::OutOfRange(String::new()),
            EshanaError::BodyUnsupported(String::new()),
            EshanaError::LatitudeUnsupported(String::new()),
            EshanaError::Provider(String::new()),
            EshanaError::Rules(String::new()),
        ] {
            assert!(codes.contains(&e.code()), "{} missing from catalog", e.code());
        }
    }

    #[test]
    fn body_parse_roundtrip() {
        for b in Body::PLANETS {
            assert_eq!(Body::parse(b.id()).unwrap(), b);
        }
        assert_eq!(Body::parse("TRUE_NODE").unwrap(), Body::TrueNode);
        assert!(Body::parse("vulcan").is_err());
    }
}
