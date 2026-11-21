//! Numerology rule packs `numerology.pythagorean@0.1` and `numerology.chaldean@0.1`.
//!
//! The two schools are separate schemas and are never merged into one number. Input is normalised
//! explicitly (NFKD, combining marks dropped, a few ligatures expanded) and everything that is ignored is
//! reported back. Nothing here is a claim about character, destiny or suitability.

use eshana_core::{EshanaError, Result};
use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

/// Which letter table to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum School {
    /// Pythagorean: A=1 … I=9, J=1 … (cycle of nine).
    Pythagorean,
    /// Chaldean: values 1–8, never 9.
    Chaldean,
}

impl School {
    /// Parse `pythagorean` or `chaldean`.
    ///
    /// # Errors
    /// [`EshanaError::Rules`] for another name.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pythagorean" => Ok(Self::Pythagorean),
            "chaldean" => Ok(Self::Chaldean),
            o => Err(EshanaError::Rules(format!(
                "unknown numerology school '{o}' (pythagorean, chaldean)"
            ))),
        }
    }

    /// Rule pack id and version.
    #[must_use]
    pub fn ruleset(self) -> &'static str {
        match self {
            Self::Pythagorean => "numerology.pythagorean@0.1",
            Self::Chaldean => "numerology.chaldean@0.1",
        }
    }

    /// Value of an uppercase A–Z letter.
    #[must_use]
    pub fn value(self, c: char) -> u32 {
        let i = (c as u8 - b'A') as usize;
        match self {
            Self::Pythagorean => (i % 9) as u32 + 1,
            // A1 B2 C3 D4 E5 F8 G3 H5 I1 J1 K2 L3 M4 N5 O7 P8 Q1 R2 S3 T4 U6 V6 W6 X5 Y1 Z7
            Self::Chaldean => [
                1, 2, 3, 4, 5, 8, 3, 5, 1, 1, 2, 3, 4, 5, 7, 8, 1, 2, 3, 4, 6, 6, 6, 5, 1, 7,
            ][i],
        }
    }
}

/// How `Y` is treated when splitting vowels from consonants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YPolicy {
    /// `Y` is always a consonant.
    Consonant,
    /// `Y` is always a vowel.
    Vowel,
}

/// Options for a name calculation.
#[derive(Debug, Clone, Copy)]
pub struct NameOptions {
    /// Letter table.
    pub school: School,
    /// Keep 11, 22, 33 when reducing (Pythagorean convention). Chaldean reduction ignores this.
    pub keep_master: bool,
    /// `Y` treatment.
    pub y: YPolicy,
}

/// Normalise text to uppercase A–Z letters. Returns the letters and every ignored character.
#[must_use]
pub fn normalize(text: &str) -> (Vec<char>, Vec<String>) {
    let mut letters = vec![];
    let mut ignored = vec![];
    for ch in text.chars() {
        let expanded: String = match ch {
            'ß' => "SS".into(),
            'æ' | 'Æ' => "AE".into(),
            'œ' | 'Œ' => "OE".into(),
            'ø' | 'Ø' => "O".into(),
            'đ' | 'Đ' => "D".into(),
            'ł' | 'Ł' => "L".into(),
            c => c.to_string(),
        };
        for d in expanded.nfkd() {
            let u = d.to_ascii_uppercase();
            if u.is_ascii_uppercase() {
                letters.push(u);
            } else if is_combining(d) || d.is_whitespace() || matches!(d, '-' | '\'' | '.' | '’') {
                // marks and ordinary separators are silently dropped but not counted as "ignored letters"
            } else {
                ignored.push(d.to_string());
            }
        }
    }
    (letters, ignored)
}

fn is_combining(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x1AB0..=0x1AFF | 0x1DC0..=0x1DFF | 0x20D0..=0x20FF | 0xFE20..=0xFE2F)
}

/// Reduce by digit sum. With `keep_master`, 11, 22 and 33 stop the reduction.
#[must_use]
pub fn reduce(mut n: u32, keep_master: bool) -> u32 {
    while n > 9 {
        if keep_master && matches!(n, 11 | 22 | 33) {
            break;
        }
        n = n.to_string().bytes().map(|b| u32::from(b - b'0')).sum();
    }
    n
}

fn is_vowel(c: char, y: YPolicy) -> bool {
    matches!(c, 'A' | 'E' | 'I' | 'O' | 'U') || (c == 'Y' && y == YPolicy::Vowel)
}

/// Full name analysis: letter table, totals, vowel and consonant splits and reductions.
///
/// # Errors
/// [`EshanaError::Rules`] when the name contains no A–Z letters after normalisation.
pub fn analyze_name(name: &str, o: NameOptions) -> Result<Value> {
    let (letters, ignored) = normalize(name);
    if letters.is_empty() {
        return Err(EshanaError::Rules(
            "the name has no Latin letters after normalisation".into(),
        ));
    }
    let keep = o.keep_master && o.school == School::Pythagorean;
    let table: Vec<Value> = letters
        .iter()
        .map(|&c| json!({"letter": c.to_string(), "value": o.school.value(c), "vowel": is_vowel(c, o.y)}))
        .collect();
    let total: u32 = letters.iter().map(|&c| o.school.value(c)).sum();
    let vowels: u32 = letters
        .iter()
        .filter(|&&c| is_vowel(c, o.y))
        .map(|&c| o.school.value(c))
        .sum();
    let cons: u32 = total - vowels;
    let block = |n: u32| json!({"compound": n, "reduced": reduce(n, keep)});
    Ok(json!({
        "school": format!("{:?}", o.school).to_lowercase(),
        "ruleset": o.school.ruleset(),
        "normalized": letters.iter().collect::<String>(),
        "ignored_characters": ignored,
        "policy": {"keep_master_numbers": keep, "y": if o.y == YPolicy::Vowel { "vowel" } else { "consonant" }},
        "letters": table,
        "expression": block(total),
        "soul_urge": block(vowels),
        "personality": block(cons),
    }))
}

/// Life path from a birth date. `method` is `reduce_parts` (reduce month, day and year separately, then sum)
/// or `total_digits` (sum every digit once).
///
/// # Errors
/// [`EshanaError::Rules`] for an unknown method.
pub fn life_path(year: i32, month: u32, day: u32, method: &str, keep_master: bool) -> Result<Value> {
    let digits = |n: u32| n.to_string().bytes().map(|b| u32::from(b - b'0')).sum::<u32>();
    let y = year.unsigned_abs();
    let (parts, result) = match method {
        "reduce_parts" => {
            let (m, d, yy) = (
                reduce(month, keep_master),
                reduce(day, keep_master),
                reduce(y, keep_master),
            );
            (
                json!({"month": m, "day": d, "year": yy}),
                reduce(m + d + yy, keep_master),
            )
        }
        "total_digits" => {
            let total = digits(month) + digits(day) + digits(y);
            (json!({"digit_sum": total}), reduce(total, keep_master))
        }
        o => {
            return Err(EshanaError::Rules(format!(
                "unknown life-path method '{o}' (reduce_parts, total_digits)"
            )))
        }
    };
    Ok(json!({"method": method, "keep_master_numbers": keep_master, "parts": parts, "life_path": result}))
}

/// Personal year for a given calendar year: reduce(birth month + birth day + the year).
#[must_use]
pub fn personal_year(month: u32, day: u32, calendar_year: i32, keep_master: bool) -> u32 {
    reduce(
        reduce(month, keep_master)
            + reduce(day, keep_master)
            + reduce(calendar_year.unsigned_abs(), keep_master),
        keep_master,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(s: School) -> NameOptions {
        NameOptions {
            school: s,
            keep_master: true,
            y: YPolicy::Consonant,
        }
    }

    #[test]
    fn pythagorean_table() {
        let s = School::Pythagorean;
        assert_eq!(
            (
                s.value('A'),
                s.value('I'),
                s.value('J'),
                s.value('R'),
                s.value('Z')
            ),
            (1, 9, 1, 9, 8)
        );
    }

    #[test]
    fn chaldean_table_has_no_nine() {
        for c in 'A'..='Z' {
            let v = School::Chaldean.value(c);
            assert!((1..=8).contains(&v), "{c} -> {v}");
        }
        assert_eq!(School::Chaldean.value('F'), 8);
        assert_eq!(School::Chaldean.value('O'), 7);
    }

    #[test]
    fn reduction_and_master_numbers() {
        assert_eq!(reduce(29, true), 11);
        assert_eq!(reduce(29, false), 2);
        assert_eq!(reduce(38, true), 11); // 3+8
        assert_eq!(reduce(9, true), 9);
        assert_eq!(reduce(10, true), 1);
        assert_eq!(reduce(33, true), 33);
    }

    #[test]
    fn worked_example_pythagorean() {
        // JOHN = 1+6+8+5 = 20 -> 2 ; vowels O = 6 ; consonants J,H,N = 14 -> 5
        let v = analyze_name("John", opt(School::Pythagorean)).unwrap();
        assert_eq!(v["expression"]["compound"], 20);
        assert_eq!(v["expression"]["reduced"], 2);
        assert_eq!(v["soul_urge"]["compound"], 6);
        assert_eq!(v["personality"]["compound"], 14);
        assert_eq!(v["personality"]["reduced"], 5);
    }

    #[test]
    fn schools_are_never_merged() {
        let p = analyze_name("Elizabeth", opt(School::Pythagorean)).unwrap();
        let c = analyze_name("Elizabeth", opt(School::Chaldean)).unwrap();
        assert_ne!(p["expression"]["compound"], c["expression"]["compound"]);
        assert_eq!(p["ruleset"], "numerology.pythagorean@0.1");
        assert_eq!(c["ruleset"], "numerology.chaldean@0.1");
    }

    #[test]
    fn diacritics_and_ligatures_are_normalised_and_reported() {
        let (l, ig) = normalize("José Ångström-Ø");
        assert_eq!(l.iter().collect::<String>(), "JOSEANGSTROMO");
        assert!(ig.is_empty());
        let (l2, ig2) = normalize("Ram राम 42");
        assert_eq!(l2.iter().collect::<String>(), "RAM");
        assert!(ig2.contains(&"4".to_string()));
        assert!(ig2.iter().any(|s| s.contains('र')));
    }

    #[test]
    fn empty_or_non_latin_name_is_an_error() {
        assert!(analyze_name("राम", opt(School::Pythagorean)).is_err());
        assert!(analyze_name("   ", opt(School::Chaldean)).is_err());
    }

    #[test]
    fn y_policy_changes_the_split() {
        let c = analyze_name(
            "Mary",
            NameOptions {
                y: YPolicy::Consonant,
                ..opt(School::Pythagorean)
            },
        )
        .unwrap();
        let v = analyze_name(
            "Mary",
            NameOptions {
                y: YPolicy::Vowel,
                ..opt(School::Pythagorean)
            },
        )
        .unwrap();
        assert_ne!(c["soul_urge"]["compound"], v["soul_urge"]["compound"]);
    }

    #[test]
    fn life_path_methods() {
        // 1992-05-28: parts 5, 1 (28->10->1), 2 (1992 -> 21 -> 3)  => reduce_parts: 5 + 1 + 3 = 9
        let a = life_path(1992, 5, 28, "reduce_parts", true).unwrap();
        assert_eq!(a["life_path"], 9);
        // total_digits: 0+5 + 2+8 + 1+9+9+2 = 36 -> 9
        let b = life_path(1992, 5, 28, "total_digits", true).unwrap();
        assert_eq!(b["life_path"], 9);
        // Master-number policy matters when the parts sum to 11: 4 + 5 + 2 (2000) = 11, or 2 without masters.
        let c = life_path(2000, 4, 5, "reduce_parts", true).unwrap();
        let d = life_path(2000, 4, 5, "reduce_parts", false).unwrap();
        assert_eq!(c["life_path"], 11);
        assert_eq!(d["life_path"], 2);
        assert!(life_path(2000, 1, 1, "magic", true).is_err());
    }

    #[test]
    fn personal_year_cycle() {
        assert_eq!(personal_year(5, 28, 2026, true), reduce(5 + 1 + 1, true)); // 28 -> 10 -> 1 ; 2026 -> 10 -> 1
    }
}
