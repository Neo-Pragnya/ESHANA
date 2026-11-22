//! Extended numerology: every commonly used number in each school, computed by named, stated rules.
//!
//! Pythagorean core, karmic, cycle and plane numbers; Chaldean compound and root numbers; the Indian (Vedic)
//! mulank, bhagyank and name number with ruling planets; and the Lo Shu grid with its arrows. Schools are kept
//! apart. Meanings are not computed here: they are interpretation and live with the report's reading notes.

use crate::jyotish::natural_relation;
use crate::numerology::{normalize, reduce, School, YPolicy};
use serde_json::{json, Map, Value};

const KARMIC_DEBT: [u32; 4] = [13, 14, 16, 19];

fn digits(n: u32) -> u32 {
    n.to_string().bytes().map(|b| u32::from(b - b'0')).sum()
}

fn vowel(c: char, y: YPolicy) -> bool {
    matches!(c, 'A' | 'E' | 'I' | 'O' | 'U') || (c == 'Y' && y == YPolicy::Vowel)
}

/// A number block: the compound total, the reduced value and any karmic debt number met on the way down.
fn block(total: u32, keep: bool) -> Value {
    let mut n = total;
    let mut debt = None;
    while n > 9 {
        if KARMIC_DEBT.contains(&n) {
            debt = Some(n);
        }
        if keep && matches!(n, 11 | 22 | 33) {
            break;
        }
        n = digits(n);
    }
    json!({"compound": total, "reduced": n, "karmic_debt": debt})
}

fn reduced(v: &Value) -> u32 {
    v["reduced"].as_u64().unwrap_or(0) as u32
}

/// The Pythagorean planes of expression (letter groups as commonly published).
fn plane_of(c: char) -> &'static str {
    match c {
        'E' | 'W' | 'D' | 'M' => "physical",
        'A' | 'H' | 'J' | 'N' | 'P' | 'G' | 'L' => "mental",
        'I' | 'O' | 'R' | 'Z' | 'B' | 'S' | 'T' | 'X' => "emotional",
        _ => "intuitive",
    }
}

/// Pythagorean numbers from a name and/or a birth date, with the cycles at a reference date.
#[must_use]
pub fn pythagorean(
    name: Option<&str>,
    dob: Option<(i32, u32, u32)>,
    reference: Option<(i32, u32, u32)>,
    y: YPolicy,
    keep: bool,
) -> Value {
    let p = School::Pythagorean;
    let mut out = Map::new();
    let mut expression = None;
    let mut first_name_total = None;
    if let Some(name) = name {
        let (letters, _) = normalize(name);
        if !letters.is_empty() {
            let val = |c: char| p.value(c);
            let total: u32 = letters.iter().map(|&c| val(c)).sum();
            let vow: u32 = letters.iter().filter(|&&c| vowel(c, y)).map(|&c| val(c)).sum();
            let ex = block(total, keep);
            expression = Some(reduced(&ex));
            out.insert("expression".into(), ex);
            out.insert("soul_urge".into(), block(vow, keep));
            out.insert("personality".into(), block(total - vow, keep));
            let mut counts = [0u32; 10];
            for &c in &letters {
                counts[val(c) as usize] += 1;
            }
            let max = *counts[1..].iter().max().unwrap_or(&0);
            out.insert(
                "inclusion".into(),
                json!((1..=9).map(|k| counts[k]).collect::<Vec<_>>()),
            );
            out.insert(
                "hidden_passion".into(),
                json!((1..=9)
                    .filter(|&k| counts[k] == max && max > 0)
                    .collect::<Vec<_>>()),
            );
            let lessons: Vec<usize> = (1..=9).filter(|&k| counts[k] == 0).collect();
            out.insert("subconscious_self".into(), json!(9 - lessons.len()));
            out.insert("karmic_lessons".into(), json!(lessons));
            let mut planes = Map::new();
            for pl in ["physical", "mental", "emotional", "intuitive"] {
                planes.insert(
                    pl.into(),
                    json!(letters.iter().filter(|&&c| plane_of(c) == pl).count()),
                );
            }
            out.insert("planes_of_expression".into(), Value::Object(planes));
            let words: Vec<Vec<char>> = name
                .split_whitespace()
                .map(|w| normalize(w).0)
                .filter(|w| !w.is_empty())
                .collect();
            if let Some(first) = words.first() {
                let ft: u32 = first.iter().map(|&c| val(c)).sum();
                first_name_total = Some(ft);
                out.insert(
                    "cornerstone".into(),
                    json!({"letter": first[0].to_string(), "value": val(first[0])}),
                );
                out.insert("capstone".into(), json!({"letter": first[first.len() - 1].to_string(), "value": val(first[first.len() - 1])}));
                if let Some(&fv) = first.iter().find(|&&c| vowel(c, y)) {
                    out.insert(
                        "first_vowel".into(),
                        json!({"letter": fv.to_string(), "value": val(fv)}),
                    );
                }
            }
            let initials: u32 = words.iter().map(|w| val(w[0])).sum();
            out.insert("balance".into(), block(initials, false));
        }
    }
    if let Some((yy, mm, dd)) = dob {
        let (m, d, yr) = (
            reduce(mm, keep),
            reduce(dd, keep),
            reduce(yy.unsigned_abs(), keep),
        );
        let lp = block(m + d + yr, keep);
        let lp_r = reduced(&lp);
        out.insert("life_path".into(), lp);
        out.insert("birthday".into(), block(dd, keep));
        out.insert("attitude".into(), block(mm + dd, keep));
        // pinnacles and challenges use single digits; the first pinnacle ends at 36 minus the life path (one digit)
        let (m1, d1, y1) = (
            reduce(mm, false),
            reduce(dd, false),
            reduce(yy.unsigned_abs(), false),
        );
        let end1 = 36 - reduce(lp_r, false);
        let pin = [
            reduce(m1 + d1, keep),
            reduce(d1 + y1, keep),
            reduce(reduce(m1 + d1, false) + reduce(d1 + y1, false), keep),
            reduce(m1 + y1, keep),
        ];
        let c1 = m1.abs_diff(d1);
        let c2 = d1.abs_diff(y1);
        let ages = [
            (0, end1),
            (end1 + 1, end1 + 9),
            (end1 + 10, end1 + 18),
            (end1 + 19, 120),
        ];
        out.insert(
            "pinnacles".into(),
            json!((0..4)
                .map(|i| json!({"number": pin[i], "from_age": ages[i].0, "to_age": ages[i].1}))
                .collect::<Vec<_>>()),
        );
        out.insert(
            "challenges".into(),
            json!([
                {"number": c1, "from_age": ages[0].0, "to_age": ages[0].1},
                {"number": c2, "from_age": ages[1].0, "to_age": ages[1].1},
                {"number": c1.abs_diff(c2), "from_age": ages[2].0, "to_age": ages[2].1},
                {"number": m1.abs_diff(y1), "from_age": ages[3].0, "to_age": 120},
            ]),
        );
        out.insert("life_cycles".into(), json!([
            {"number": reduce(mm, keep), "from_age": 0, "to_age": ages[0].1, "source": "month"},
            {"number": reduce(dd, keep), "from_age": ages[1].0, "to_age": ages[2].1, "source": "day"},
            {"number": reduce(yy.unsigned_abs(), keep), "from_age": ages[3].0, "to_age": 120, "source": "year"},
        ]));
        if let Some(ex) = expression {
            out.insert("maturity".into(), block(lp_r + ex, keep));
            out.insert(
                "bridge_life_path_expression".into(),
                json!(reduce(lp_r, false).abs_diff(reduce(ex, false))),
            );
        }
        if let Some(ft) = first_name_total {
            out.insert("rational_thought".into(), block(reduce(ft, false) + d1, keep));
        }
        if let Some((ry, rm, rd)) = reference {
            let py = reduce(m1 + d1 + reduce(ry.unsigned_abs(), false), keep);
            let pm = reduce(reduce(py, false) + rm, keep);
            let pd = reduce(reduce(pm, false) + rd, keep);
            let uy = reduce(digits(ry.unsigned_abs()), keep);
            out.insert("cycles_at_reference".into(), json!({
                "date": format!("{ry:04}-{rm:02}-{rd:02}"), "personal_year": py, "personal_month": pm, "personal_day": pd,
                "universal_year": uy, "universal_month": reduce(reduce(uy, false) + rm, keep),
                "universal_day": reduce(reduce(reduce(uy, false) + rm, false) + rd, keep),
            }));
        }
    }
    if let (Some(s), Some(pe)) = (
        out.get("soul_urge").map(reduced),
        out.get("personality").map(reduced),
    ) {
        out.insert(
            "bridge_soul_personality".into(),
            json!(reduce(s, false).abs_diff(reduce(pe, false))),
        );
    }
    out.insert("conventions".into(), json!({
        "keep_master_numbers": keep, "y": if y == YPolicy::Vowel { "vowel" } else { "consonant" },
        "life_path": "reduce month, day and year separately, then the sum",
        "karmic_debt": "13, 14, 16 or 19 met while reducing a total",
        "pinnacles": "first ends at age 36 minus the single-digit life path; then 9, 9 years and the rest of life",
        "life_cycles": "month, day and year numbers over the first, middle two and last pinnacle periods",
        "planes": "physical E W D M; mental A H J N P G L; emotional I O R Z B S T X; intuitive K F Q U Y C V",
    }));
    Value::Object(out)
}

/// Chaldean compound and root numbers.
#[must_use]
pub fn chaldean(name: Option<&str>, dob: Option<(i32, u32, u32)>) -> Value {
    let c = School::Chaldean;
    let mut out = Map::new();
    if let Some(name) = name {
        let (letters, _) = normalize(name);
        if !letters.is_empty() {
            let total: u32 = letters.iter().map(|&ch| c.value(ch)).sum();
            out.insert("name".into(), block(total, false));
            let words: Vec<Value> = name
                .split_whitespace()
                .map(|w| (w, normalize(w).0))
                .filter(|(_, l)| !l.is_empty())
                .map(|(w, l)| json!({"word": w, "number": block(l.iter().map(|&ch| c.value(ch)).sum(), false)}))
                .collect();
            out.insert("words".into(), json!(words));
        }
    }
    if let Some((yy, mm, dd)) = dob {
        out.insert("psychic".into(), block(dd, false));
        out.insert(
            "destiny".into(),
            block(digits(dd) + digits(mm) + digits(yy.unsigned_abs()), false),
        );
    }
    out.insert(
        "conventions".into(),
        json!("letter values 1-8 (no 9); compound numbers kept before reduction"),
    );
    Value::Object(out)
}

/// The planet traditionally ruling each root number in Indian numerology.
pub const NUMBER_PLANET: [&str; 10] = [
    "", "sun", "moon", "jupiter", "rahu", "mercury", "venus", "ketu", "saturn", "mars",
];

fn as_classical(p: &str) -> &str {
    match p {
        "rahu" => "saturn",
        "ketu" => "mars",
        o => o,
    }
}

/// Indian (Vedic) numerology: mulank, bhagyank, Chaldean name number, ruling planets and number friendships.
#[must_use]
pub fn vedic(name: Option<&str>, dob: Option<(i32, u32, u32)>) -> Value {
    let mut out = Map::new();
    let relation = |a: u32, b: u32| -> &'static str {
        let (pa, pb) = (
            as_classical(NUMBER_PLANET[a as usize]),
            as_classical(NUMBER_PLANET[b as usize]),
        );
        if pa == pb {
            "friend"
        } else {
            natural_relation(pa, pb)
        }
    };
    let friends = |n: u32| -> Value {
        let mut f = vec![];
        let mut e = vec![];
        let mut ne = vec![];
        for k in 1..=9 {
            if k == n {
                continue;
            }
            match relation(n, k) {
                "friend" => f.push(k),
                "enemy" => e.push(k),
                _ => ne.push(k),
            }
        }
        json!({"friendly": f, "neutral": ne, "enemy": e})
    };
    let mut mul = None;
    let mut bhag = None;
    if let Some((yy, mm, dd)) = dob {
        let m = reduce(dd, false);
        let b = reduce(digits(dd) + digits(mm) + digits(yy.unsigned_abs()), false);
        mul = Some(m);
        bhag = Some(b);
        out.insert(
            "mulank".into(),
            json!({"number": m, "planet": NUMBER_PLANET[m as usize], "numbers": friends(m)}),
        );
        out.insert(
            "bhagyank".into(),
            json!({"number": b, "planet": NUMBER_PLANET[b as usize], "numbers": friends(b)}),
        );
        out.insert("mulank_bhagyank".into(), json!(relation(m, b)));
    }
    if let Some(name) = name {
        let (letters, _) = normalize(name);
        if !letters.is_empty() {
            let total: u32 = letters.iter().map(|&ch| School::Chaldean.value(ch)).sum();
            let n = reduce(total, false);
            let mut v = json!({"compound": total, "number": n, "planet": NUMBER_PLANET[n as usize]});
            if let Some(m) = mul {
                v["with_mulank"] = json!(relation(m, n));
            }
            if let Some(b) = bhag {
                v["with_bhagyank"] = json!(relation(b, n));
            }
            out.insert("name_number".into(), v);
        }
    }
    out.insert("conventions".into(), json!({
        "planets": "1 Sun, 2 Moon, 3 Jupiter, 4 Rahu, 5 Mercury, 6 Venus, 7 Ketu, 8 Saturn, 9 Mars",
        "friendship": "from the natural relationships of the ruling planets (BPHS); Rahu counted as Saturn, Ketu as Mars",
        "name_number": "Chaldean letter values",
    }));
    Value::Object(out)
}

/// The Lo Shu grid of the birth date (zeros ignored), with and without the driver (mulank) and conductor (bhagyank).
#[must_use]
pub fn lo_shu(dob: (i32, u32, u32)) -> Value {
    let (yy, mm, dd) = dob;
    let s = format!("{dd:02}{mm:02}{:04}", yy.unsigned_abs());
    let mut counts = [0u32; 10];
    for b in s.bytes() {
        counts[(b - b'0') as usize] += 1;
    }
    let driver = reduce(dd, false);
    let conductor = reduce(digits(dd) + digits(mm) + digits(yy.unsigned_abs()), false);
    let mut with = counts;
    with[driver as usize] += 1;
    with[conductor as usize] += 1;
    let arrows = [
        ("mental plane", [4, 9, 2]),
        ("emotional plane", [3, 5, 7]),
        ("practical plane", [8, 1, 6]),
        ("thought plane", [4, 3, 8]),
        ("will plane", [9, 5, 1]),
        ("action plane", [2, 7, 6]),
        ("golden plane (success)", [4, 5, 6]),
        ("silver plane (property)", [2, 5, 8]),
    ];
    let read = |c: &[u32; 10]| -> Value {
        json!(arrows.iter().map(|(n, k)| {
            let have = k.iter().filter(|&&x| c[x as usize] > 0).count();
            json!({"name": n, "numbers": k, "state": if have == 3 { "complete" } else if have == 0 { "missing" } else { "partial" }})
        }).collect::<Vec<_>>())
    };
    json!({
        "layout": [[4, 9, 2], [3, 5, 7], [8, 1, 6]],
        "digits": s,
        "counts": (1..=9).map(|k| counts[k]).collect::<Vec<_>>(),
        "missing": (1..=9).filter(|&k| counts[k] == 0).collect::<Vec<_>>(),
        "repeated": (1..=9).filter(|&k| counts[k] > 1).collect::<Vec<_>>(),
        "arrows": read(&counts),
        "driver": driver, "conductor": conductor,
        "counts_with_driver_conductor": (1..=9).map(|k| with[k]).collect::<Vec<_>>(),
        "arrows_with_driver_conductor": read(&with),
        "kua": kua(dob),
        "conventions": "digits of DDMMYYYY with zeros ignored; the second grid adds the driver (mulank) and conductor (bhagyank) as Indian practice does",
    })
}

/// The Kua (Gua) number of Eight Mansions feng shui, for both sexes (ESHANA does not take gender as an input). The
/// Chinese solar year starts at Lichun, taken as 4 February. Reduce the year's last two digits to one digit r; before
/// 2000 the male Kua is 10 − r and the female r + 5, from 2000 on 9 − r and r + 6 (reduced to 1–9); a Kua of 5 is
/// read as 2 for a man and 8 for a woman.
#[must_use]
pub fn kua(dob: (i32, u32, u32)) -> Value {
    let (y0, m, d) = dob;
    let y = if m < 2 || (m == 2 && d < 4) { y0 - 1 } else { y0 };
    let r = reduce(y.rem_euclid(100).unsigned_abs(), false);
    let r = if y.rem_euclid(100) == 0 { 0 } else { r };
    let one = |x: i64| -> u32 {
        let x = x.rem_euclid(9) as u32;
        if x == 0 {
            9
        } else {
            x
        }
    };
    let (male, female) = if y < 2000 {
        (one(10 - i64::from(r)), one(i64::from(r) + 5))
    } else {
        (one(9 - i64::from(r)), one(i64::from(r) + 6))
    };
    let group = |k: u32| if [1, 3, 4, 9].contains(&k) { "east" } else { "west" };
    let m5 = if male == 5 { 2 } else { male };
    let f5 = if female == 5 { 8 } else { female };
    json!({"solar_year": y, "male": m5, "female": f5, "male_group": group(m5), "female_group": group(f5),
        "conventions": "Eight Mansions; year from Lichun (4 February); last two digits of the year reduced; Kua 5 is 2 for a man, 8 for a woman; east group 1, 3, 4, 9, west group 2, 6, 7, 8"})
}

#[cfg(test)]
mod tests {
    #[test]
    fn kua_numbers() {
        // 1990 (after 4 February): r = 9; man 10 − 9 = 1, woman 9 + 5 = 14 → 5 → 8
        let k = kua((1990, 5, 15));
        assert_eq!((k["male"].as_u64(), k["female"].as_u64()), (Some(1), Some(8)));
        // 2000: r = 0; man 9, woman 6
        let k = kua((2000, 6, 1));
        assert_eq!((k["male"].as_u64(), k["female"].as_u64()), (Some(9), Some(6)));
        // 2 February 1991 belongs to the 1990 solar year
        assert_eq!(kua((1991, 2, 2))["solar_year"], 1990);
        // 1985: r = 4; man 6, woman 9
        let k = kua((1985, 7, 7));
        assert_eq!((k["male"].as_u64(), k["female"].as_u64()), (Some(6), Some(9)));
    }

    use super::*;

    #[test]
    fn worked_pythagorean_numbers() {
        // 1879-12-30: month 12 -> 3, day 30 -> 3, year 1879 -> 25 -> 7; life path 13 (karmic debt) -> 4
        let v = pythagorean(
            Some("Ramana Maharishi"),
            Some((1879, 12, 30)),
            Some((2026, 11, 21)),
            YPolicy::Consonant,
            true,
        );
        assert_eq!(v["life_path"]["reduced"], 4);
        assert_eq!(v["life_path"]["karmic_debt"], 13);
        assert_eq!(v["birthday"]["reduced"], 3);
        assert_eq!(v["attitude"]["reduced"], 6); // 12 + 30 = 42 -> 6
        assert_eq!(v["pinnacles"][0]["to_age"], 32); // 36 - 4
        assert_eq!(v["pinnacles"][0]["number"], 6); // 3 + 3
        assert_eq!(v["challenges"][0]["number"], 0); // |3 - 3|
        assert_eq!(v["cycles_at_reference"]["universal_year"], 1); // 2026 -> 10 -> 1
        assert_eq!(v["inclusion"].as_array().unwrap().len(), 9);
    }

    #[test]
    fn lo_shu_counts_and_arrows() {
        let v = lo_shu((1879, 12, 30)); // 30121879 -> 3,0,1,2,1,8,7,9
        assert_eq!(v["counts"], json!([2, 1, 1, 0, 0, 0, 1, 1, 1]));
        assert_eq!(v["missing"], json!([4, 5, 6]));
        let gold = v["arrows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "golden plane (success)")
            .unwrap();
        assert_eq!(gold["state"], "missing");
    }

    #[test]
    fn vedic_numbers_use_planet_friendship() {
        let v = vedic(None, Some((1879, 12, 30)));
        assert_eq!(v["mulank"]["number"], 3);
        assert_eq!(v["mulank"]["planet"], "jupiter");
        // Jupiter's natural friends are the Sun, Moon and Mars: numbers 1, 2 and 9
        let f: Vec<u64> = v["mulank"]["numbers"]["friendly"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_u64().unwrap())
            .collect();
        assert!(f.contains(&1) && f.contains(&2) && f.contains(&9));
    }
}
