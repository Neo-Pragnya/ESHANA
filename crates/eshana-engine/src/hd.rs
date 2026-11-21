//! Human Design calculation rule pack `human_design.basic@0.1`.
//!
//! What is implemented is the **mathematics and the published structure**: the design instant (the moment the
//! Sun stood exactly 88° of ecliptic longitude before its birth position, solved numerically, not "88 days"),
//! the 64-gate wheel starting at 2°00′ Aquarius, lines and colours, the 36 channels, the nine centres, and the
//! derived type, authority, profile and definition.
//!
//! Gate numbers are I Ching hexagram numbers placed on the ecliptic; no proprietary text, diagrams or
//! interpretations are included. Rights in branded nomenclature and in third-party mapping material are an open
//! decision (ADR-009); this pack is labelled as a tradition rule and is flagged as pending legal review.

use eshana_core::{angular_diff, norm360, EshanaError, Result};
use eshana_time::Instant;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Rule pack id and version.
pub const RULESET: &str = "human_design.basic@0.1";
/// Gate 41 begins here: 2°00′00″ Aquarius.
pub const WHEEL_START_DEG: f64 = 302.0;
/// Width of a gate: 360 / 64.
pub const GATE_SPAN: f64 = 5.625;
/// Width of a line: a sixth of a gate.
pub const LINE_SPAN: f64 = GATE_SPAN / 6.0;
/// The design instant is when the Sun was this many degrees of longitude earlier.
pub const DESIGN_ARC_DEG: f64 = 88.0;

/// Gate sequence around the wheel starting at [`WHEEL_START_DEG`].
pub const WHEEL: [u8; 64] = [
    41, 19, 13, 49, 30, 55, 37, 63, 22, 36, 25, 17, 21, 51, 42, 3, 27, 24, 2, 23, 8, 20, 16, 35, 45, 12, 15,
    52, 39, 53, 62, 56, 31, 33, 7, 4, 29, 59, 40, 64, 47, 6, 46, 18, 48, 57, 32, 50, 28, 44, 1, 43, 14, 34,
    9, 5, 26, 11, 10, 58, 38, 54, 61, 60,
];

/// The 36 channels as gate pairs.
pub const CHANNELS: [(u8, u8); 36] = [
    (1, 8),
    (2, 14),
    (3, 60),
    (4, 63),
    (5, 15),
    (6, 59),
    (7, 31),
    (9, 52),
    (10, 20),
    (10, 34),
    (10, 57),
    (11, 56),
    (12, 22),
    (13, 33),
    (16, 48),
    (17, 62),
    (18, 58),
    (19, 49),
    (20, 34),
    (20, 57),
    (21, 45),
    (23, 43),
    (24, 61),
    (25, 51),
    (26, 44),
    (27, 50),
    (28, 38),
    (29, 46),
    (30, 41),
    (32, 54),
    (34, 57),
    (35, 36),
    (37, 40),
    (39, 55),
    (42, 53),
    (47, 64),
];

/// The nine centres with their gates.
pub const CENTERS: [(&str, &[u8]); 9] = [
    ("head", &[64, 61, 63]),
    ("ajna", &[47, 24, 4, 17, 43, 11]),
    ("throat", &[62, 23, 56, 35, 12, 45, 33, 8, 31, 20, 16]),
    ("g", &[1, 13, 25, 46, 2, 15, 10, 7]),
    ("heart", &[21, 40, 26, 51]),
    ("spleen", &[48, 57, 44, 50, 32, 28, 18]),
    ("solar_plexus", &[36, 22, 37, 6, 49, 55, 30]),
    ("sacral", &[5, 14, 29, 59, 9, 3, 42, 27, 34]),
    ("root", &[53, 60, 52, 19, 39, 41, 58, 38, 54]),
];

const MOTORS: [&str; 4] = ["sacral", "solar_plexus", "heart", "root"];

/// Centre that owns a gate.
#[must_use]
pub fn center_of(gate: u8) -> &'static str {
    CENTERS
        .iter()
        .find(|(_, g)| g.contains(&gate))
        .map_or("?", |c| c.0)
}

/// Gate, line and colour at an ecliptic longitude (tropical, apparent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Activation {
    /// 1–64.
    pub gate: u8,
    /// 1–6.
    pub line: u8,
    /// 1–6.
    pub color: u8,
}

/// Map a longitude onto the wheel.
#[must_use]
pub fn activation_at(lon_deg: f64) -> Activation {
    let off = norm360(lon_deg - WHEEL_START_DEG);
    let gi = ((off / GATE_SPAN).floor() as usize).min(63);
    let in_gate = off - gi as f64 * GATE_SPAN;
    let line = ((in_gate / LINE_SPAN).floor() as u8).min(5);
    let in_line = in_gate - f64::from(line) * LINE_SPAN;
    let color = ((in_line / (LINE_SPAN / 6.0)).floor() as u8).min(5);
    Activation {
        gate: WHEEL[gi],
        line: line + 1,
        color: color + 1,
    }
}

/// Solve for the design instant: the UT moment when `sun_lon(instant)` equals the birth Sun longitude minus 88°.
///
/// `sun_lon` must return the apparent tropical longitude of the Sun at an instant.
///
/// # Errors
/// [`EshanaError::Rules`] if the root finder does not converge (it always does for a real Sun).
pub fn design_instant(birth: &Instant, sun_lon: &dyn Fn(&Instant) -> Result<f64>) -> Result<Instant> {
    let target = norm360(sun_lon(birth)? - DESIGN_ARC_DEG);
    let f = |jd: f64| -> Result<f64> { Ok(angular_diff(sun_lon(&Instant::from_jd_utc(jd))?, target)) };
    // The Sun moves 0.953–1.019°/day, so 88° is 86–92 days; bracket 80..100 days before birth.
    let (mut lo, mut hi) = (birth.jd_ut - 100.0, birth.jd_ut - 80.0);
    let (mut flo, fhi) = (f(lo)?, f(hi)?);
    if flo.signum() == fhi.signum() {
        return Err(EshanaError::Rules(
            "design instant is not bracketed; solver aborted".into(),
        ));
    }
    for _ in 0..100 {
        let mid = 0.5 * (lo + hi);
        let fm = f(mid)?;
        if fm.abs() < 1e-10 || (hi - lo) < 1e-10 {
            return Ok(Instant::from_jd_utc(mid));
        }
        if fm.signum() == flo.signum() {
            lo = mid;
            flo = fm;
        } else {
            hi = mid;
        }
    }
    Ok(Instant::from_jd_utc(0.5 * (lo + hi)))
}

/// Derived BodyGraph attributes from the set of active gates.
#[must_use]
pub fn bodygraph(gates: &BTreeSet<u8>, personality_sun_line: u8, design_sun_line: u8) -> Value {
    let channels: Vec<(u8, u8)> = CHANNELS
        .iter()
        .copied()
        .filter(|(a, b)| gates.contains(a) && gates.contains(b))
        .collect();
    let mut defined: BTreeSet<&str> = BTreeSet::new();
    for (a, b) in &channels {
        defined.insert(center_of(*a));
        defined.insert(center_of(*b));
    }
    // Adjacency between centres through defined channels.
    let mut adj: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (a, b) in &channels {
        adj.entry(center_of(*a)).or_default().insert(center_of(*b));
        adj.entry(center_of(*b)).or_default().insert(center_of(*a));
    }
    let reach = |from: &str| -> BTreeSet<&str> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut stack = vec![CENTERS.iter().find(|c| c.0 == from).map_or("", |c| c.0)];
        while let Some(c) = stack.pop() {
            if seen.insert(c) {
                for n in adj.get(c).into_iter().flatten() {
                    stack.push(n);
                }
            }
        }
        seen
    };
    let is_def = |c: &str| defined.contains(c);
    let throat_reach = if is_def("throat") {
        reach("throat")
    } else {
        BTreeSet::new()
    };
    let motor_to_throat = MOTORS.iter().any(|m| throat_reach.contains(m));
    let kind = if defined.is_empty() {
        "reflector"
    } else if is_def("sacral") {
        if motor_to_throat {
            "manifesting_generator"
        } else {
            "generator"
        }
    } else if motor_to_throat {
        "manifestor"
    } else {
        "projector"
    };
    let authority = if kind == "reflector" {
        "lunar"
    } else if is_def("solar_plexus") {
        "emotional"
    } else if is_def("sacral") {
        "sacral"
    } else if is_def("spleen") {
        "splenic"
    } else if is_def("heart") {
        if throat_reach.contains("heart") {
            "ego_manifested"
        } else {
            "ego_projected"
        }
    } else if is_def("g") {
        "self_projected"
    } else {
        "mental_environment"
    };
    // Definition: connected components among defined centres.
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut comps = 0;
    for c in &defined {
        if !seen.contains(c) {
            comps += 1;
            seen.extend(reach(c));
        }
    }
    let definition = match comps {
        0 => "none",
        1 => "single",
        2 => "split",
        3 => "triple_split",
        _ => "quadruple_split",
    };
    let all: Vec<&str> = CENTERS.iter().map(|c| c.0).collect();
    json!({
        "type": kind,
        "authority": authority,
        "profile": format!("{personality_sun_line}/{design_sun_line}"),
        "definition": definition,
        "defined_centers": all.iter().filter(|c| is_def(c)).collect::<Vec<_>>(),
        "undefined_centers": all.iter().filter(|c| !is_def(c)).collect::<Vec<_>>(),
        "channels": channels.iter().map(|(a, b)| format!("{a}-{b}")).collect::<Vec<_>>(),
        "active_gates": gates.iter().collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_is_a_permutation_of_1_to_64() {
        let s: BTreeSet<u8> = WHEEL.iter().copied().collect();
        assert_eq!(s.len(), 64);
        assert_eq!(s.iter().min(), Some(&1));
        assert_eq!(s.iter().max(), Some(&64));
    }

    #[test]
    fn thirty_six_unique_channels_over_valid_gates() {
        let s: BTreeSet<(u8, u8)> = CHANNELS.iter().copied().collect();
        assert_eq!(s.len(), 36);
        for (a, b) in CHANNELS {
            assert!((1..=64).contains(&a) && (1..=64).contains(&b) && a != b);
            assert_ne!(center_of(a), "?");
            assert_ne!(
                center_of(a),
                center_of(b),
                "channel {a}-{b} joins a centre to itself"
            );
        }
    }

    #[test]
    fn centres_partition_the_gates() {
        let mut all: Vec<u8> = CENTERS.iter().flat_map(|c| c.1.iter().copied()).collect();
        all.sort_unstable();
        assert_eq!(all, (1..=64).collect::<Vec<u8>>());
    }

    #[test]
    fn every_gate_is_in_at_least_one_channel() {
        let in_channel: BTreeSet<u8> = CHANNELS.iter().flat_map(|c| [c.0, c.1]).collect();
        assert_eq!(
            in_channel.len(),
            64,
            "missing: {:?}",
            (1..=64).filter(|g| !in_channel.contains(g)).collect::<Vec<_>>()
        );
    }

    #[test]
    fn wheel_geometry() {
        assert_eq!(activation_at(302.0).gate, 41);
        assert_eq!(activation_at(302.0).line, 1);
        assert_eq!(activation_at(301.999).gate, 60);
        assert_eq!(activation_at(302.0 + 5.625).gate, 19);
        assert_eq!(activation_at(302.0 + 0.9375).line, 2);
        assert_eq!(activation_at(302.0 + 5.624_999).line, 6);
        // Every gate gets exactly 5.625 degrees.
        let mut counts = BTreeMap::new();
        for i in 0..64 * 6 {
            *counts
                .entry(activation_at(302.0 + (f64::from(i) + 0.5) * LINE_SPAN).gate)
                .or_insert(0) += 1;
        }
        assert!(counts.values().all(|&c| c == 6));
    }

    #[test]
    fn colors_run_one_to_six() {
        let a = activation_at(302.0 + 0.01);
        assert_eq!(a.color, 1);
        assert_eq!(activation_at(302.0 + LINE_SPAN - 0.001).color, 6);
    }

    #[test]
    fn design_solver_finds_88_degrees_with_a_uniform_sun() {
        // A Sun moving exactly 1 deg/day makes the answer exactly 88 days earlier.
        let birth = Instant::parse_utc("2000-06-01T00:00:00Z").unwrap();
        let sun = |i: &Instant| -> Result<f64> { Ok(norm360(i.jd_ut - 2_451_545.0 + 10.0)) };
        let d = design_instant(&birth, &sun).unwrap();
        assert!((birth.jd_ut - d.jd_ut - 88.0).abs() < 1e-6);
    }

    #[test]
    fn types_from_channels() {
        // Reflector: nothing defined.
        let none: BTreeSet<u8> = BTreeSet::new();
        assert_eq!(bodygraph(&none, 1, 1)["type"], "reflector");
        // Generator: channel 5-15 (sacral to G) only.
        let g: BTreeSet<u8> = [5, 15].into();
        let v = bodygraph(&g, 3, 5);
        assert_eq!(v["type"], "generator");
        assert_eq!(v["authority"], "sacral");
        assert_eq!(v["profile"], "3/5");
        // Manifesting generator: sacral 34-20 (sacral to throat).
        let mg: BTreeSet<u8> = [34, 20].into();
        assert_eq!(bodygraph(&mg, 1, 1)["type"], "manifesting_generator");
        // Manifestor: heart 21-45 (heart to throat), no sacral.
        let m: BTreeSet<u8> = [21, 45].into();
        let v = bodygraph(&m, 1, 1);
        assert_eq!(v["type"], "manifestor");
        assert_eq!(v["authority"], "ego_manifested");
        // Projector: G to throat 1-8 only.
        let p: BTreeSet<u8> = [1, 8].into();
        let v = bodygraph(&p, 1, 1);
        assert_eq!(v["type"], "projector");
        assert_eq!(v["authority"], "self_projected");
        // Solar plexus beats sacral for authority.
        let sp: BTreeSet<u8> = [5, 15, 37, 40].into();
        assert_eq!(bodygraph(&sp, 1, 1)["authority"], "emotional");
    }

    #[test]
    fn split_definition_is_detected() {
        // 5-15 (sacral-G) and 28-38 (spleen-root): two separate components.
        let s: BTreeSet<u8> = [5, 15, 28, 38].into();
        assert_eq!(bodygraph(&s, 1, 1)["definition"], "split");
        let s1: BTreeSet<u8> = [5, 15].into();
        assert_eq!(bodygraph(&s1, 1, 1)["definition"], "single");
    }
}
