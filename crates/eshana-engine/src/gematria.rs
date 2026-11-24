//! Gematria and abjad (NUM-04): the letter values of a name written in Hebrew or Arabic letters, from the
//! `numerology.gematria` rule pack. The name must be in that script: nothing is transliterated, because a Latin
//! spelling does not determine the letters.

use eshana_core::{EshanaError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The pack source.
pub const SOURCE: &str = include_str!("../rulesets/gematria.yaml");

#[derive(Debug, Deserialize)]
struct School {
    script: String,
    letters: BTreeMap<String, u32>,
}

#[derive(Debug, Deserialize)]
struct Pack {
    id: String,
    version: String,
    schools: BTreeMap<String, School>,
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    P.get_or_init(|| serde_norway::from_str(SOURCE).expect("gematria.yaml is valid"))
}

/// Whether a school id belongs to this pack.
#[must_use]
pub fn handles(school: &str) -> bool {
    pack().schools.contains_key(school)
}

/// Letter values, the total and its digit reduction.
///
/// # Errors
/// `ESH-RULES` when the name has no letters of the school's script.
pub fn analyze(name: &str, school: &str) -> Result<Value> {
    let s = pack()
        .schools
        .get(school)
        .ok_or_else(|| EshanaError::Rules(format!("unknown school {school}")))?;
    let mut letters = vec![];
    let mut ignored = vec![];
    for ch in name.chars() {
        let k = ch.to_string();
        match s.letters.get(&k) {
            Some(v) => letters.push(json!({"letter": k, "value": v})),
            None if ch.is_whitespace() || matches!(ch as u32, 0x0591..=0x05C7 | 0x064B..=0x065F | 0x0670) => {
            } // spaces, vowel points, harakat
            None => ignored.push(k),
        }
    }
    if letters.is_empty() {
        return Err(EshanaError::Rules(format!(
            "write the name in {} letters for {school}; a Latin spelling does not determine them",
            s.script
        )));
    }
    let total: u32 = letters
        .iter()
        .map(|l| l["value"].as_u64().unwrap_or(0) as u32)
        .sum();
    Ok(
        json!({"school": school, "script": s.script, "rule": format!("{}@{}", pack().id, pack().version),
              "letters": letters, "total": total, "reduced": crate::numerology::reduce(total, false), "ignored": ignored}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        // שלום (shalom) = 300 + 30 + 6 + 40 = 376; חי (chai) = 18
        assert_eq!(analyze("שלום", "hebrew_gematria").unwrap()["total"], 376);
        assert_eq!(analyze("חי", "hebrew_gematria").unwrap()["total"], 18);
        // بسم = 2 + 60 + 40 = 102; the basmala's 786 is the classic check of the abjad table
        assert_eq!(analyze("بسم", "abjad").unwrap()["total"], 102);
        assert_eq!(analyze("بسم الله الرحمن الرحيم", "abjad").unwrap()["total"], 786);
        assert!(analyze("Shalom", "hebrew_gematria").is_err());
    }
}
