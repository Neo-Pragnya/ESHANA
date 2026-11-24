//! Further divisional charts, each under an explicit method id so schools are never mixed.
//!
//! The Parāśari charts D1–D60 stay where they are (`divisional_chart` family, [`crate::vedic::divisional_sign`]).
//! This module adds the `varga_method` family:
//!
//! - **Parivṛtti (cyclic) vargas**: divide the zodiac into `12·N/12 … ` equal parts of `30/N` degrees counted
//!   continuously from 0° Meṣa, the k-th part going to sign `k mod 12`. For N = 9 this is the Parāśari navāṃśa,
//!   which the tests use as a check. Here: D2, D3 (an alternative to the Parāśari horā and drekkāṇa), D81
//!   (navanavāṃśa) and D108 (aṣṭottarāṃśa, in its cyclic form).
//! - **D144 (dvādaśa-dvādaśāṃśa)**: the Parāśari D12 taken of the D12 position, i.e. D12 applied twice.

use crate::methods::Method;
use crate::vedic::divisional_sign;
use eshana_core::{norm360, EshanaError, Result};
use serde_json::{json, Value};

/// The method ids, with their schools.
pub const METHODS: &[Method] = &[
    Method {
        id: "d2_parivritti",
        status: "opt_in",
        note: "horā as two cyclic parts per sign from 0° Meṣa (parivṛtti school)",
    },
    Method {
        id: "d3_parivritti",
        status: "opt_in",
        note: "drekkāṇa as three cyclic parts per sign from 0° Meṣa (parivṛtti school)",
    },
    Method {
        id: "d81_navanavamsha",
        status: "opt_in",
        note: "navanavāṃśa: 81 cyclic parts per sign (the navāṃśa of the navāṃśa)",
    },
    Method {
        id: "d108_ashtottaramsha_cyclic",
        status: "opt_in",
        note: "aṣṭottarāṃśa: 108 cyclic parts per sign from 0° Meṣa",
    },
    Method {
        id: "d144_dvadasha_dvadashamsha",
        status: "opt_in",
        note: "Parāśari dvādaśāṃśa applied to the dvādaśāṃśa position",
    },
    Method {
        id: "d30_parivritti",
        status: "opt_in",
        note: "triṃśāṃśa as thirty equal 1° parts per sign, cyclic from 0° Meṣa (the default D30 is the Parāśari unequal 5-5-8-7-5 scheme)",
    },
    Method {
        id: "d60_parivritti",
        status: "opt_in",
        note: "ṣaṣṭyāṃśa as sixty cyclic 0°30′ parts from 0° Meṣa (the default D60 counts from the sign itself, Parāśari)",
    },
    Method {
        id: "d150_parivritti",
        status: "opt_in",
        note: "nāḍyāṃśa as 150 cyclic 0°12′ parts from 0° Meṣa; the Candra Kalā Nāḍī mapping with its sign-type orders is not implemented",
    },
];

fn cyclic(lon: f64, n: u32) -> usize {
    ((norm360(lon) * f64::from(n) / 30.0).floor() as u64 % 12) as usize
}

/// The D12 position as a longitude: the D12 sign plus the place inside the 2°30′ part, stretched twelvefold.
fn d12_longitude(lon: f64) -> Result<f64> {
    let lon = norm360(lon);
    let s = (lon / 30.0).floor();
    let within = lon - s * 30.0;
    let part = (within / 2.5).floor();
    let s12 = divisional_sign(lon, 12)?;
    Ok(s12 as f64 * 30.0 + (within - part * 2.5) * 12.0)
}

/// Sign (0–11) of a sidereal longitude under a `varga_method` id.
///
/// # Errors
/// `ESH-METHOD` for an unknown id.
pub fn sign(id: &str, lon: f64) -> Result<usize> {
    match id {
        "d2_parivritti" => Ok(cyclic(lon, 2)),
        "d3_parivritti" => Ok(cyclic(lon, 3)),
        "d81_navanavamsha" => Ok(cyclic(lon, 81)),
        "d108_ashtottaramsha_cyclic" => Ok(cyclic(lon, 108)),
        "d144_dvadasha_dvadashamsha" => divisional_sign(d12_longitude(lon)?, 12),
        "d30_parivritti" => Ok(cyclic(lon, 30)),
        "d60_parivritti" => Ok(cyclic(lon, 60)),
        "d150_parivritti" => Ok(cyclic(lon, 150)),
        other => Err(EshanaError::MethodUnsupported(format!(
            "'{other}' is not a varga_method (one of: {})",
            METHODS.iter().map(|m| m.id).collect::<Vec<_>>().join(", ")
        ))),
    }
}

/// The signs of every body and the lagna under each requested method.
///
/// # Errors
/// `ESH-METHOD` for an unknown id.
pub fn charts(ids: &[String], lons: &[(String, f64)]) -> Result<Value> {
    let mut out = vec![];
    for id in ids {
        let m = METHODS.iter().find(|m| m.id == id.as_str());
        let mut signs = serde_json::Map::new();
        for (who, l) in lons {
            signs.insert(who.clone(), json!(sign(id, *l)?));
        }
        out.push(json!({"method": {"family": "varga_method", "id": id}, "school": m.map(|m| m.note), "signs": signs}));
    }
    Ok(json!(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cyclic_d30_d60_d150() {
        assert_eq!(sign("d30_parivritti", 0.5).unwrap(), 0); // the first 1° part of Meṣa
        assert_eq!(sign("d30_parivritti", 31.5).unwrap(), 31 % 12); // the 32nd part
        assert_eq!(sign("d60_parivritti", 30.25).unwrap(), 60 % 12); // 0°15′ Vṛṣabha: part 60
        assert_eq!(sign("d150_parivritti", 0.19).unwrap(), 0); // 0°11′ Meṣa: part 0
        assert_eq!(sign("d150_parivritti", 0.21).unwrap(), 1); // 0°12.6′: part 1
    }

    #[test]
    fn cyclic_navamsha_is_the_parashari_navamsha() {
        let mut x = 0.0;
        while x < 360.0 {
            assert_eq!(cyclic(x, 9), divisional_sign(x, 9).unwrap(), "{x}");
            x += 0.137;
        }
    }

    #[test]
    fn every_boundary_of_every_cyclic_method() {
        for (id, n) in [
            ("d2_parivritti", 2_u32),
            ("d3_parivritti", 3),
            ("d81_navanavamsha", 81),
            ("d108_ashtottaramsha_cyclic", 108),
        ] {
            let parts = 12 * n;
            for k in 1..parts {
                let b = f64::from(k) * 30.0 / f64::from(n);
                let before = sign(id, b - 1e-9).unwrap();
                let at = sign(id, b + 1e-9).unwrap();
                assert_eq!(before, ((k - 1) % 12) as usize, "{id} before part {k}");
                assert_eq!(at, (k % 12) as usize, "{id} at part {k}");
            }
            assert_eq!(sign(id, 359.999_999_9).unwrap(), ((parts - 1) % 12) as usize);
        }
    }

    #[test]
    fn d144_is_d12_of_d12_at_every_boundary() {
        // 144 parts of 12.5′ per sign; part k (0–143) of sign s lies in D12 part k / 12 and its sub-part k mod 12,
        // so the sign is s + ⌊k/12⌋ + (k mod 12), mod 12. Checked on both sides of every boundary.
        for s in 0..12_usize {
            for k in 0..144_usize {
                let start = s as f64 * 30.0 + k as f64 * 30.0 / 144.0;
                let want = (s + k / 12 + k % 12) % 12;
                assert_eq!(
                    sign("d144_dvadasha_dvadashamsha", start + 1e-9).unwrap(),
                    want,
                    "sign {s} part {k} start"
                );
                assert_eq!(
                    sign("d144_dvadasha_dvadashamsha", start + 30.0 / 144.0 - 1e-9).unwrap(),
                    want,
                    "sign {s} part {k} end"
                );
            }
        }
    }

    #[test]
    fn schools_are_never_mixed_silently() {
        assert_eq!(sign("d2", 10.0).unwrap_err().code(), "ESH-METHOD");
        assert_ne!(cyclic(20.0, 2), divisional_sign(20.0, 2).unwrap()); // Meṣa 20°: cyclic → Vṛṣabha; Parāśari → Karka
    }
}
