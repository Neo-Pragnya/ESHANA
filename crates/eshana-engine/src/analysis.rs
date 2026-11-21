//! Shared context for the analysis modules: [`crate::maitri`] (relationships, tārās, varga dignities),
//! [`crate::dashas_ext`] (more daśā systems), [`crate::auxiliary`] (sahams, avakahaḍā, mṛtyu bhāga, ghāta chakra,
//! vedhasthāna, jīva/deha, śodhana, male/female rāśis), [`crate::chakras`] (Sarvatobhadra, Kota, Saṅghaṭṭa) and
//! [`crate::annual`] (varṣaphala, pañca pakṣī, KP significator tables).
//!
//! Each module exposes `pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>>`; the chart op pushes every
//! returned attribute with its own rule id. Everything a module needs is in the context, so modules never reach
//! into the request.

use crate::jyotish::VChart;
use eshana_core::{Ayanamsha, Result};
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde_json::Value;
use std::collections::BTreeMap;

/// Everything an analysis module may use.
pub struct AnalysisCtx<'a> {
    /// Sidereal longitudes (`lon`), speeds, Ascendant and Midheaven.
    pub vc: &'a VChart,
    /// The ephemeris (for returns, ingresses and other searches).
    pub provider: &'a dyn EphemerisProvider,
    /// The birth instant (`jd_ut`, `jd_tt`, `utc`, `civil_offset_s`).
    pub inst: &'a Instant,
    /// Latitude and east longitude in degrees, when the place is known.
    pub place: Option<(f64, f64)>,
    /// Civil UTC offset in seconds, when known.
    pub offset_s: Option<f64>,
    /// The ayanāṃśa of the chart.
    pub ayanamsha: Ayanamsha,
    /// The birth pañcāṅga (sunrise, sunset, next_sunrise, vara, tithi, nakshatra, yoga, karana), when computed.
    pub panchanga: Option<&'a Value>,
    /// The reference instant ("now") as a Julian day (UT), when given.
    pub reference_jd: Option<f64>,
    /// Right ascension, declination and ecliptic latitude per body id (degrees).
    pub equ: &'a BTreeMap<String, (f64, f64, f64)>,
}

/// One derived attribute from an analysis module.
pub struct Derived {
    /// Attribute id, e.g. `vedic.maitri`.
    pub id: String,
    /// Rule id with version, e.g. `vedic.maitri@0.1`.
    pub rule: String,
    /// The value.
    pub value: Value,
    /// What it is derived from (`positions`, `houses`, `panchanga`, …).
    pub from: Vec<String>,
}

/// The modules, by feature id.
pub type Compute = fn(&AnalysisCtx) -> Result<Vec<Derived>>;
