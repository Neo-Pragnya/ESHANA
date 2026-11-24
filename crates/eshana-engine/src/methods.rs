//! The method registry and the envelope helpers every newer operation shares.
//!
//! A *method* is a named way of computing something where traditions or conventions differ: an ayanāṃśa, a house
//! system, a cusp-occupancy rule, a muhūrta profile. Every method has a stable id in a family; a request names the
//! id, the result echoes it, and an id the engine does not implement is refused with `ESH-METHOD` rather than
//! replaced by a default. The `methods` operation lists the registry so a caller can discover ids without reading
//! the source.
//!
//! The helpers here give three shapes the envelope schema describes:
//! - [`unavailable`]: a typed "could not compute" value with the reason and the missing inputs (never a zero);
//! - [`interval`]: a `[start, end)` UTC interval;
//! - [`config_hash`]: a short hash of the normalised calculation settings, so two results can be compared for
//!   "same settings" without diffing every field.

use eshana_core::{Ayanamsha, CalculationConfig, EshanaError, Result};
use serde_json::{json, Value};

/// One method in a family.
#[derive(Debug, Clone, Copy)]
pub struct Method {
    /// Stable id, unique within its family.
    pub id: &'static str,
    /// `stable` (default behaviour is fixed), `opt_in` (stable but never applied unless asked) or `experimental`.
    pub status: &'static str,
    /// One line: what the method is and where its definition comes from.
    pub note: &'static str,
}

const fn m(id: &'static str, status: &'static str, note: &'static str) -> Method {
    Method { id, status, note }
}

/// House systems.
pub const HOUSE_SYSTEMS: &[Method] = &[
    m(
        "whole_sign",
        "stable",
        "each sign is one house, counted from the rising sign",
    ),
    m("equal", "stable", "30° houses from the Ascendant degree"),
    m(
        "placidus",
        "stable",
        "time-trisection of the diurnal and nocturnal semi-arcs; undefined near the poles",
    ),
    m(
        "porphyry",
        "stable",
        "each quadrant between the angles trisected in ecliptic longitude",
    ),
    m(
        "sripati",
        "stable",
        "bhāva chalit: midpoints of the Porphyry cusps, which become bhāva madhyas",
    ),
    m(
        "koch",
        "stable",
        "Ascendants at the times that trisect the MC degree's semi-diurnal arc; undefined near the poles",
    ),
    m(
        "regiomontanus",
        "stable",
        "the equator in 30° parts, projected by circles through the horizon's north and south points",
    ),
    m(
        "campanus",
        "stable",
        "the prime vertical in 30° parts, projected the same way",
    ),
    m(
        "alcabitius",
        "stable",
        "the Ascendant degree's semi-arcs trisected in right ascension; undefined near the poles",
    ),
    m(
        "topocentric",
        "stable",
        "Polich–Page: Regiomontanus-style circles with pole heights of 1/3 and 2/3 of tan φ",
    ),
    m(
        "morinus",
        "stable",
        "the equator in 30° parts from the east point, projected along circles of ecliptic longitude",
    ),
    m(
        "meridian",
        "stable",
        "axial rotation: the equator in 30° parts from the MC, projected along circles of declination",
    ),
];

/// Lunar node models.
pub const NODE_MODELS: &[Method] = &[
    m("mean", "stable", "mean lunar node"),
    m("true", "stable", "osculating (true) lunar node"),
];

/// Daśā year lengths.
pub const YEAR_CONVENTIONS: &[Method] = &[
    m("julian", "stable", "365.25 days"),
    m("gregorian", "stable", "365.2425 days"),
    m("savana", "stable", "360 days"),
    m("sidereal", "stable", "365.25636 days"),
];

/// Sunrise definitions for the pañcāṅga.
pub const SUNRISE: &[Method] = &[
    m(
        "upper_limb_refracted",
        "stable",
        "upper limb on the horizon with standard refraction (default)",
    ),
    m(
        "centre_refracted",
        "stable",
        "disc centre on the horizon with standard refraction",
    ),
    m(
        "centre_unrefracted",
        "stable",
        "disc centre on the geometric horizon",
    ),
];

/// Event searches.
pub const EVENT_SEARCHES: &[Method] = &[
    m("ingresses", "stable", "sign entries of one body"),
    m(
        "stations",
        "stable",
        "roots of longitudinal speed, with retrograde intervals",
    ),
    m(
        "crossings",
        "stable",
        "crossings of a longitude and aspects to it, with near-miss touches",
    ),
];

/// Numerology schools.
pub const NUMEROLOGY_SCHOOLS: &[Method] = &[
    m("pythagorean", "stable", "1–9 letter values in alphabet order"),
    m("chaldean", "stable", "1–8 letter values of the Chaldean table"),
    m(
        "hebrew_gematria",
        "opt_in",
        "mispar hechrechi: Hebrew letters 1–400 (the name in Hebrew script)",
    ),
    m(
        "abjad",
        "opt_in",
        "eastern abjad order: Arabic letters 1–1000 (the name in Arabic script)",
    ),
];

/// Every family with the methods it holds. Families added by later operations register here too.
#[must_use]
pub fn registry() -> Vec<(&'static str, Vec<Method>)> {
    let ayan: Vec<Method> = Ayanamsha::ALL
        .iter()
        .map(|a| {
            let st = if matches!(
                a,
                Ayanamsha::Lahiri | Ayanamsha::Raman | Ayanamsha::Krishnamurti | Ayanamsha::FaganBradley
            ) {
                "stable"
            } else {
                "opt_in"
            };
            m(
                a.id(),
                st,
                "sidereal offset; `info.ayanamsha_definitions` gives its epoch, frame and source",
            )
        })
        .collect();
    let vargas: Vec<Method> = crate::vedic::DIVISIONS
        .iter()
        .map(|d| m(varga_id(*d), "stable", "Parāśari divisional chart"))
        .collect();
    let mut out = vec![
        ("ayanamsha", ayan),
        ("house_system", HOUSE_SYSTEMS.to_vec()),
        ("node_model", NODE_MODELS.to_vec()),
        ("divisional_chart", vargas),
        ("year_convention", YEAR_CONVENTIONS.to_vec()),
        ("sunrise", SUNRISE.to_vec()),
        ("event_search", EVENT_SEARCHES.to_vec()),
        ("numerology_school", NUMEROLOGY_SCHOOLS.to_vec()),
    ];
    out.extend(extra_families());
    out
}

/// Families owned by the newer operations (KP, matching, muhūrta, progressions …).
fn extra_families() -> Vec<(&'static str, Vec<Method>)> {
    vec![
        ("kp_cusp_occupancy", KP_OCCUPANCY.to_vec()),
        ("muhurta_profile", crate::muhurta::profile_methods()),
        ("varga_method", crate::vargas_ext::METHODS.to_vec()),
        ("progression_angles", crate::progress::ANGLE_METHODS.to_vec()),
        (
            "research_model",
            vec![m(
                "research.window_score@0.1",
                "experimental",
                "orders passing muhūrta windows; see docs/design/research-models.md",
            )],
        ),
    ]
}

/// KP cusp occupancy.
pub const KP_OCCUPANCY: &[Method] = &[
    m(
        "cusp_start",
        "stable",
        "a planet belongs to the house whose cusp it has passed (default)",
    ),
    m(
        "cusp_midpoint",
        "opt_in",
        "the cusp is the middle of its house; boundaries at midpoints between cusps",
    ),
];

fn varga_id(d: u32) -> &'static str {
    const IDS: [&str; 61] = [
        "", "d1", "d2", "d3", "d4", "d5", "d6", "d7", "d8", "d9", "d10", "d11", "d12", "d13", "d14", "d15",
        "d16", "d17", "d18", "d19", "d20", "d21", "d22", "d23", "d24", "d25", "d26", "d27", "d28", "d29",
        "d30", "d31", "d32", "d33", "d34", "d35", "d36", "d37", "d38", "d39", "d40", "d41", "d42", "d43",
        "d44", "d45", "d46", "d47", "d48", "d49", "d50", "d51", "d52", "d53", "d54", "d55", "d56", "d57",
        "d58", "d59", "d60",
    ];
    IDS.get(d as usize).copied().unwrap_or("")
}

/// The `methods` operation's result.
#[must_use]
pub fn listing() -> Value {
    let fams: Vec<Value> = registry()
        .into_iter()
        .map(|(f, ms)| {
            json!({"family": f, "methods": ms.iter()
                .map(|x| json!({"id": x.id, "status": x.status, "note": x.note})).collect::<Vec<_>>()})
        })
        .collect();
    json!({"schema_version": eshana_core::SCHEMA_VERSION, "kind": "methods", "families": fams, "interpretations": []})
}

/// Check that `id` is a method of `family`; `ESH-METHOD` names the ids that are.
///
/// # Errors
/// [`EshanaError::MethodUnsupported`] for an unknown id or family.
pub fn require(family: &str, id: &str) -> Result<&'static str> {
    let reg = registry();
    let Some((_, ms)) = reg.iter().find(|(f, _)| *f == family) else {
        return Err(EshanaError::MethodUnsupported(format!(
            "no method family '{family}'"
        )));
    };
    ms.iter().find(|x| x.id == id).map(|x| x.id).ok_or_else(|| {
        EshanaError::MethodUnsupported(format!(
            "'{id}' is not a {family} method (one of: {})",
            ms.iter().map(|x| x.id).collect::<Vec<_>>().join(", ")
        ))
    })
}

/// A typed unavailable value: never a zero standing in for "could not compute".
#[must_use]
pub fn unavailable(reason: &str, missing: &[&str]) -> Value {
    json!({"status": "unavailable", "reason": reason, "missing": missing})
}

/// A `[start, end)` interval in UTC; `end` may be `None` for an interval still open at the end of the range.
#[must_use]
pub fn interval(start_utc: &str, end_utc: Option<&str>) -> Value {
    json!({"start_utc": start_utc, "end_utc": end_utc, "bounds": "[start, end)"})
}

/// 64-bit FNV-1a of the normalised settings, as `fnv1a64:<16 hex>`.
///
/// Normalised means: serialised through serde with keys sorted (serde_json's default map), so field order and
/// formatting never change the hash; a changed value always does.
#[must_use]
pub fn config_hash(cfg: &CalculationConfig) -> String {
    let v = serde_json::to_value(cfg).unwrap_or(Value::Null);
    hash_value(&v)
}

/// The same hash over any JSON value (used for method parameters that are not part of [`CalculationConfig`]).
#[must_use]
pub fn hash_value(v: &Value) -> String {
    let s = serde_json::to_string(v).unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_within_each_family() {
        for (f, ms) in registry() {
            let mut ids: Vec<_> = ms.iter().map(|x| x.id).collect();
            ids.sort_unstable();
            let n = ids.len();
            ids.dedup();
            assert_eq!(n, ids.len(), "duplicate id in {f}");
            assert!(
                ms.iter()
                    .all(|x| ["stable", "opt_in", "experimental"].contains(&x.status)),
                "{f}"
            );
        }
    }

    #[test]
    fn unknown_method_is_esh_method() {
        assert_eq!(
            require("house_system", "vehlow").unwrap_err().code(),
            "ESH-METHOD"
        );
        assert_eq!(require("house_system", "koch").unwrap(), "koch");
        assert_eq!(require("nope", "x").unwrap_err().code(), "ESH-METHOD");
        assert_eq!(require("house_system", "placidus").unwrap(), "placidus");
    }

    #[test]
    fn config_hash_ignores_nothing_and_is_stable() {
        let a = CalculationConfig::western("analytic");
        let mut b = a.clone();
        assert_eq!(config_hash(&a), config_hash(&b));
        b.provider = "spk".into();
        assert_ne!(config_hash(&a), config_hash(&b));
        assert!(config_hash(&a).starts_with("fnv1a64:"));
        // FNV-1a test vector: empty input.
        assert_eq!(hash_value(&json!("")), {
            let mut h: u64 = 0xcbf2_9ce4_8422_2325;
            for b in b"\"\"" {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
            format!("fnv1a64:{h:016x}")
        });
    }
}
