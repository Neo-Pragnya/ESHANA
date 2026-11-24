//! Lal Kitab (LK-01): the Lal Kitab chart (houses fixed to the signs, grahas in their whole-sign house from the
//! lagna) and the pakka ghar (permanent house) of each graha, from the draft `vedic.lal_kitab` pack.

use crate::jyotish::VChart;
use eshana_core::{EshanaError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pack source.
pub const SOURCE: &str = include_str!("../rulesets/lal-kitab.yaml");

#[derive(Debug, Deserialize)]
struct Pack {
    id: String,
    version: String,
    status: String,
    pakka_ghar: BTreeMap<String, Vec<u32>>,
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| serde_norway::from_str(SOURCE).expect("lal-kitab.yaml is valid"))
}

/// Rule id with version.
#[must_use]
pub fn rule_id() -> String {
    format!("{}@{}", pack().id, pack().version)
}

const SIGNS: [&str; 12] = [
    "aries",
    "taurus",
    "gemini",
    "cancer",
    "leo",
    "virgo",
    "libra",
    "scorpio",
    "sagittarius",
    "capricorn",
    "aquarius",
    "pisces",
];

/// The Lal Kitab chart: house (1–12) of each graha and the fixed sign of that house; whether it is in its pakka ghar.
///
/// # Errors
/// `ESH-INPUT` without the lagna (the chart needs the birth time and place).
pub fn chart(c: &VChart) -> Result<Value> {
    let asc = c.asc.ok_or_else(|| {
        EshanaError::Input("the Lal Kitab chart needs the lagna (exact time and place)".into())
    })?;
    let lagna_sign = (asc.rem_euclid(360.0) / 30.0) as usize;
    let mut grahas = serde_json::Map::new();
    for (g, &lon) in &c.lon {
        if !pack().pakka_ghar.contains_key(g) {
            continue;
        }
        let house = ((lon.rem_euclid(360.0) / 30.0) as usize + 12 - lagna_sign) % 12 + 1;
        let pakka = &pack().pakka_ghar[g];
        grahas.insert(
            g.clone(),
            json!({"house": house, "house_sign": SIGNS[house - 1], "pakka_ghar": pakka,
                                        "in_pakka_ghar": pakka.contains(&(house as u32))}),
        );
    }
    Ok(
        json!({"status": pack().status, "rule": "houses fixed to the signs (1 = Aries); each graha in its whole-sign house from the lagna",
              "grahas": grahas, "not_implemented": ["Lal Kitab aspects", "sleeping houses", "varshphal age table", "remedies"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn houses_count_from_the_lagna_and_signs_are_fixed() {
        let mut c = VChart {
            asc: Some(95.0),
            ..Default::default()
        }; // Cancer lagna
        c.lon.insert("sun".into(), 35.0); // Taurus: 11th from Cancer
        c.lon.insert("moon".into(), 100.0); // Cancer: 1st
        let v = chart(&c).unwrap();
        assert_eq!(v["grahas"]["sun"]["house"], 11);
        assert_eq!(v["grahas"]["sun"]["house_sign"], "aquarius"); // house 11 is Aquarius in the Lal Kitab chart
        assert_eq!(v["grahas"]["moon"]["house"], 1);
        assert_eq!(v["grahas"]["moon"]["in_pakka_ghar"], false); // the Moon's pakka ghar is the 4th
        assert_eq!(v["status"], "draft");
    }
}
