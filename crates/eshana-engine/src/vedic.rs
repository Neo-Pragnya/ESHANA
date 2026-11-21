//! Vedic rule pack `vedic.parashari@0.2`: nakshatra and pada, divisional charts and Vimśottarī daśā.
//!
//! Every function takes a **sidereal** longitude. Divisional-chart conventions and the daśā year length are
//! school-specific; the rule version names the exact convention implemented here.

use eshana_core::{norm360, EshanaError, Result};
use eshana_time::Instant;
use serde_json::{json, Value};

/// Rule pack id and version for everything in this module.
pub const RULESET: &str = "vedic.parashari@0.2";

/// The 27 nakshatras, index 0 = Aśvinī.
pub const NAKSHATRAS: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

/// Width of one nakshatra: 13°20′.
pub const NAKSHATRA_SPAN: f64 = 360.0 / 27.0;
/// Width of one pada: 3°20′.
pub const PADA_SPAN: f64 = NAKSHATRA_SPAN / 4.0;

/// Vimśottarī sequence with years.
pub const DASHA_ORDER: [(&str, f64); 9] = [
    ("Ketu", 7.0),
    ("Venus", 20.0),
    ("Sun", 6.0),
    ("Moon", 10.0),
    ("Mars", 7.0),
    ("Rahu", 18.0),
    ("Jupiter", 16.0),
    ("Saturn", 19.0),
    ("Mercury", 17.0),
];

/// Nakshatra, pada and lord for a sidereal longitude.
#[derive(Debug, Clone, PartialEq)]
pub struct Nakshatra {
    /// 0–26.
    pub index: usize,
    /// Name.
    pub name: &'static str,
    /// 1–4.
    pub pada: usize,
    /// Vimśottarī lord.
    pub lord: &'static str,
    /// Fraction of the nakshatra already traversed, `[0, 1)`.
    pub elapsed: f64,
}

/// Classify a sidereal longitude.
#[must_use]
pub fn nakshatra_of(sidereal_lon: f64) -> Nakshatra {
    let lon = norm360(sidereal_lon);
    let index = ((lon / NAKSHATRA_SPAN).floor() as usize).min(26);
    let within = lon - index as f64 * NAKSHATRA_SPAN;
    Nakshatra {
        index,
        name: NAKSHATRAS[index],
        pada: ((within / PADA_SPAN).floor() as usize).min(3) + 1,
        lord: DASHA_ORDER[index % 9].0,
        elapsed: within / NAKSHATRA_SPAN,
    }
}

/// Divisions implemented by [`divisional_sign`].
pub const DIVISIONS: [u32; 16] = [1, 2, 3, 4, 7, 9, 10, 12, 16, 20, 24, 27, 30, 40, 45, 60];

/// Divisional (varga) sign index 0–11 for a sidereal longitude, Parāśara conventions (BPHS ch. 6–7).
///
/// Implemented: D1, D2 (horā: Leo/Cancer), D3, D4, D7, D9, D10, D12, D16, D20, D24, D27, D30 (unequal
/// triṃśāṃśa), D40, D45, D60. Starting signs depend on the sign's odd/even parity, modality or element exactly
/// as written below.
///
/// # Errors
/// [`EshanaError::Rules`] for a division that is not implemented (never guessed).
pub fn divisional_sign(sidereal_lon: f64, d: u32) -> Result<usize> {
    let lon = norm360(sidereal_lon);
    let s = ((lon / 30.0).floor() as usize).min(11);
    let within = lon - s as f64 * 30.0;
    let part = |n: u32| ((within / (30.0 / f64::from(n))).floor() as usize).min(n as usize - 1);
    let odd = s % 2 == 0; // Aries (index 0) is the first, odd sign
    let modality = s % 3; // 0 movable, 1 fixed, 2 dual
    let by_modality = |m: usize, f: usize, du: usize| [m, f, du][modality];
    Ok(match d {
        1 => s,
        2 => {
            let first = within < 15.0;
            if odd == first {
                4 // Leo, the Sun's horā
            } else {
                3 // Cancer, the Moon's horā
            }
        }
        3 => (s + 4 * part(3)) % 12,
        4 => (s + 3 * part(4)) % 12,
        7 => (if odd { s } else { s + 6 } + part(7)) % 12,
        9 => (((lon * 9.0 / 30.0).floor() as usize) % 12).min(11),
        10 => (if odd { s } else { s + 8 } + part(10)) % 12,
        12 => (s + part(12)) % 12,
        16 => (by_modality(0, 4, 8) + part(16)) % 12,
        20 => (by_modality(0, 8, 4) + part(20)) % 12,
        24 => (if odd { 4 } else { 3 } + part(24)) % 12,
        27 => ((s % 4) * 3 + part(27)) % 12,
        30 => {
            // Unequal parts: odd signs Mars 5, Saturn 5, Jupiter 8, Mercury 7, Venus 5 (Aries, Aquarius,
            // Sagittarius, Gemini, Libra); even signs reversed (Taurus, Virgo, Pisces, Capricorn, Scorpio).
            let (bounds, signs): ([f64; 5], [usize; 5]) = if odd {
                ([5.0, 10.0, 18.0, 25.0, 30.0], [0, 10, 8, 2, 6])
            } else {
                ([5.0, 12.0, 20.0, 25.0, 30.0], [1, 5, 11, 9, 7])
            };
            let k = bounds.iter().position(|b| within < *b).unwrap_or(4);
            signs[k]
        }
        40 => (if odd { 0 } else { 6 } + part(40)) % 12,
        45 => (by_modality(0, 4, 8) + part(45)) % 12,
        60 => (s + part(60)) % 12,
        other => {
            return Err(EshanaError::Rules(format!(
                "divisional chart D{other} is not implemented in {RULESET} (implemented: {DIVISIONS:?})"
            )))
        }
    })
}

/// Length of a daśā year, in days, for a named convention.
///
/// # Errors
/// [`EshanaError::Rules`] for an unknown convention.
pub fn year_days(convention: &str) -> Result<f64> {
    match convention {
        "julian" | "365.25" => Ok(365.25),
        "gregorian" | "365.2425" => Ok(365.2425),
        "savana" | "360" => Ok(360.0),
        "sidereal" | "365.25636" => Ok(365.256_36),
        other => Err(EshanaError::Rules(format!(
            "unknown year convention '{other}' (julian=365.25, gregorian=365.2425, savana=360, sidereal=365.25636)"
        ))),
    }
}

/// Names of nested daśā levels.
pub const LEVEL_NAMES: [&str; 5] = [
    "mahadasha",
    "antardasha",
    "pratyantardasha",
    "sookshmadasha",
    "pranadasha",
];

/// Yoginī daśā: eight yoginīs with their grahas and years (36 in all).
pub const YOGINI_ORDER: [(&str, &str, f64); 8] = [
    ("Mangala", "Moon", 1.0),
    ("Pingala", "Sun", 2.0),
    ("Dhanya", "Jupiter", 3.0),
    ("Bhramari", "Mars", 4.0),
    ("Bhadrika", "Mercury", 5.0),
    ("Ulka", "Saturn", 6.0),
    ("Siddha", "Venus", 7.0),
    ("Sankata", "Rahu", 8.0),
];

/// A cyclic daśā system: the sequence of (lord, years) and its full cycle length.
struct System<'a> {
    seq: Vec<(&'a str, f64)>,
    cycle: f64,
}

impl System<'_> {
    /// Children of a period of `years` starting at `start_jd` whose lord is `lord_idx`: the same sequence from that
    /// lord, each child proportional to its years. The last child ends exactly at the parent's end.
    fn children(
        &self,
        lord_idx: usize,
        start_jd: f64,
        years: f64,
        ydays: f64,
    ) -> Vec<(usize, f64, f64, f64)> {
        let n = self.seq.len();
        let end_parent = start_jd + years * ydays;
        let mut t = start_jd;
        (0..n)
            .map(|k| {
                let i = (lord_idx + k) % n;
                let y = years * self.seq[i].1 / self.cycle;
                let end = if k == n - 1 { end_parent } else { t + y * ydays };
                let out = (i, t, end, y);
                t = end;
                out
            })
            .collect()
    }

    /// Nested periods to `depth` levels below a parent; periods that end before birth are left out.
    #[allow(clippy::too_many_arguments)]
    fn nest(
        &self,
        lord_idx: usize,
        start: f64,
        years: f64,
        ydays: f64,
        birth: f64,
        level: usize,
        depth: usize,
    ) -> Vec<Value> {
        let iso = |jd: f64| Instant::from_jd_utc(jd).utc;
        self.children(lord_idx, start, years, ydays)
            .into_iter()
            .filter(|(_, _, end, _)| *end > birth)
            .map(|(i, a, b, y)| {
                let mut v = json!({"lord": self.seq[i].0, "years": y, "start_utc": iso(a.max(birth)),
                                   "end_utc": iso(b), "clipped_at_birth": a < birth});
                if level + 1 < depth {
                    v[format!("{}s", LEVEL_NAMES[level + 1])] =
                        json!(self.nest(i, a, y, ydays, birth, level + 1, depth));
                }
                v
            })
            .collect()
    }

    /// The chain of periods (to five levels) containing `at`.
    fn chain(&self, first: usize, cycle_start: f64, ydays: f64, at: f64) -> Value {
        let iso = |jd: f64| Instant::from_jd_utc(jd).utc;
        let mut out = vec![];
        // top level: walk the cycle (repeating if `at` is past the first cycle)
        let mut t = cycle_start;
        let n = self.seq.len();
        let mut k = 0usize;
        let (mut lord, mut start, mut years);
        loop {
            let i = (first + k) % n;
            let y = self.seq[i].1;
            if at < t + y * ydays || k > 10 * n {
                lord = i;
                start = t;
                years = y;
                break;
            }
            t += y * ydays;
            k += 1;
        }
        out.push(json!({"level": LEVEL_NAMES[0], "lord": self.seq[lord].0, "start_utc": iso(start), "end_utc": iso(start + years * ydays)}));
        for name in LEVEL_NAMES.iter().skip(1) {
            let kids = self.children(lord, start, years, ydays);
            let (i, a, b, y) = *kids
                .iter()
                .find(|(_, _, end, _)| at < *end)
                .unwrap_or(&kids[kids.len() - 1]);
            out.push(json!({"level": name, "lord": self.seq[i].0, "start_utc": iso(a), "end_utc": iso(b)}));
            lord = i;
            start = a;
            years = y;
        }
        json!({"at_utc": iso(at), "periods": out})
    }
}

/// Vimśottarī daśā from the Moon's sidereal longitude at birth, nested to `levels` (1–5; 2 = mahādaśā and
/// antardaśā), plus the five-level chain current at `reference_jd` when given. Dates are UTC ISO strings.
///
/// # Errors
/// Unknown year convention, or `levels` outside 1–5.
pub fn vimshottari_levels(
    moon_sidereal_lon: f64,
    birth: &Instant,
    convention: &str,
    levels: usize,
    reference_jd: Option<f64>,
) -> Result<Value> {
    if !(1..=5).contains(&levels) {
        return Err(EshanaError::Input(format!(
            "dasha levels must be 1 to 5, got {levels}"
        )));
    }
    let ydays = year_days(convention)?;
    let nk = nakshatra_of(moon_sidereal_lon);
    let first = nk.index % 9;
    let sys = System {
        seq: DASHA_ORDER.to_vec(),
        cycle: 120.0,
    };
    let elapsed_years = nk.elapsed * DASHA_ORDER[first].1;
    let start = birth.jd_ut - elapsed_years * ydays;
    let mut mahas = vec![];
    let mut t = start;
    for k in 0..9 {
        let i = (first + k) % 9;
        let y = DASHA_ORDER[i].1;
        let mut v = json!({"lord": DASHA_ORDER[i].0, "years": y, "start_utc": Instant::from_jd_utc(t.max(birth.jd_ut)).utc,
                           "end_utc": Instant::from_jd_utc(t + y * ydays).utc, "clipped_at_birth": t < birth.jd_ut});
        if levels > 1 {
            v["antardashas"] = json!(sys.nest(i, t, y, ydays, birth.jd_ut, 1, levels));
        }
        mahas.push(v);
        t += y * ydays;
    }
    let mut out = json!({
        "system": "vimshottari",
        "year_convention": convention,
        "year_days": ydays,
        "levels": levels,
        "moon_nakshatra": nk.name,
        "starting_lord": DASHA_ORDER[first].0,
        "balance_at_birth_years": DASHA_ORDER[first].1 - elapsed_years,
        "mahadashas": mahas,
    });
    if let Some(at) = reference_jd {
        out["current"] = sys.chain(first, start, ydays, at);
    }
    Ok(out)
}

/// Vimśottarī daśā with mahādaśās and antardaśās (two levels).
///
/// # Errors
/// Unknown year convention.
pub fn vimshottari(moon_sidereal_lon: f64, birth: &Instant, convention: &str) -> Result<Value> {
    vimshottari_levels(moon_sidereal_lon, birth, convention, 2, None)
}

/// Yoginī daśā: the starting yoginī is (janma nakshatra number + 3) mod 8 (0 meaning the 8th, Saṅkaṭā), the
/// balance is the untraversed part of the nakshatra, and the 36-year cycle repeats.
///
/// # Errors
/// Unknown year convention.
pub fn yogini(
    moon_sidereal_lon: f64,
    birth: &Instant,
    convention: &str,
    levels: usize,
    reference_jd: Option<f64>,
) -> Result<Value> {
    let ydays = year_days(convention)?;
    let nk = nakshatra_of(moon_sidereal_lon);
    let y = (nk.index + 1 + 3) % 8;
    let first = (y + 7) % 8;
    let seq: Vec<(&str, f64)> = YOGINI_ORDER.iter().map(|(n, _, yrs)| (*n, *yrs)).collect();
    let sys = System { seq, cycle: 36.0 };
    let elapsed_years = nk.elapsed * YOGINI_ORDER[first].2;
    let start = birth.jd_ut - elapsed_years * ydays;
    let mut periods = vec![];
    let mut t = start;
    // 120 years of yoginī periods (three cycles and a part) to cover a life span.
    let mut k = 0usize;
    while t < birth.jd_ut + 120.0 * ydays {
        let i = (first + k) % 8;
        let yrs = YOGINI_ORDER[i].2;
        let mut v = json!({"yogini": YOGINI_ORDER[i].0, "lord": YOGINI_ORDER[i].1, "years": yrs,
                           "start_utc": Instant::from_jd_utc(t.max(birth.jd_ut)).utc,
                           "end_utc": Instant::from_jd_utc(t + yrs * ydays).utc, "clipped_at_birth": t < birth.jd_ut});
        if levels > 1 {
            v["antardashas"] = json!(sys.nest(i, t, yrs, ydays, birth.jd_ut, 1, levels.min(3)));
        }
        periods.push(v);
        t += yrs * ydays;
        k += 1;
    }
    let mut out = json!({
        "system": "yogini",
        "year_convention": convention,
        "starting_yogini": YOGINI_ORDER[first].0,
        "balance_at_birth_years": YOGINI_ORDER[first].2 - elapsed_years,
        "mahadashas": periods,
    });
    if let Some(at) = reference_jd {
        out["current"] = sys.chain(first, start, ydays, at);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nakshatra_boundaries() {
        assert_eq!(nakshatra_of(0.0).name, "Ashwini");
        assert_eq!(nakshatra_of(13.333_3).name, "Ashwini");
        assert_eq!(nakshatra_of(13.333_4).name, "Bharani");
        assert_eq!(nakshatra_of(359.999).name, "Revati");
        assert_eq!(nakshatra_of(360.0).name, "Ashwini");
        assert_eq!(nakshatra_of(-0.001).name, "Revati");
    }

    #[test]
    fn pada_runs_one_to_four() {
        for (lon, p) in [(0.1, 1), (3.4, 2), (6.7, 3), (10.0, 4), (13.4, 1)] {
            assert_eq!(nakshatra_of(lon).pada, p, "lon {lon}");
        }
    }

    #[test]
    fn lord_cycle_repeats_every_nine() {
        assert_eq!(nakshatra_of(0.0).lord, "Ketu");
        assert_eq!(nakshatra_of(9.0 * NAKSHATRA_SPAN + 0.1).lord, "Ketu");
        assert_eq!(nakshatra_of(3.0 * NAKSHATRA_SPAN + 0.1).lord, "Moon"); // Rohini
    }

    #[test]
    fn navamsha_known_values() {
        // Aries 0°-3°20' -> Aries; 3°20'-6°40' -> Taurus (movable sign starts at itself).
        assert_eq!(divisional_sign(1.0, 9).unwrap(), 0);
        assert_eq!(divisional_sign(4.0, 9).unwrap(), 1);
        // Taurus (fixed, earth) starts at Capricorn.
        assert_eq!(divisional_sign(30.5, 9).unwrap(), 9);
        // Gemini (dual, air) starts at Libra.
        assert_eq!(divisional_sign(60.5, 9).unwrap(), 6);
        // Cancer (water) starts at Cancer.
        assert_eq!(divisional_sign(90.5, 9).unwrap(), 3);
        // Pisces last navamsha is Pisces.
        assert_eq!(divisional_sign(359.9, 9).unwrap(), 11);
    }

    #[test]
    fn d1_is_identity_and_unknown_is_an_error() {
        for lon in [0.0, 45.0, 123.4, 359.9] {
            assert_eq!(divisional_sign(lon, 1).unwrap(), (lon / 30.0) as usize);
        }
        assert_eq!(divisional_sign(10.0, 5).unwrap_err().code(), "ESH-RULES");
        assert_eq!(divisional_sign(10.0, 150).unwrap_err().code(), "ESH-RULES");
    }

    #[test]
    fn d10_even_sign_starts_from_ninth() {
        // Taurus (even sign) 0°-3° -> 9th from Taurus = Capricorn.
        assert_eq!(divisional_sign(31.0, 10).unwrap(), 9);
        // Aries (odd) 0°-3° -> Aries.
        assert_eq!(divisional_sign(1.0, 10).unwrap(), 0);
    }

    #[test]
    fn dasha_structure_and_balance() {
        let birth = Instant::parse_utc("1992-05-28T14:35:00Z").unwrap();
        // Moon at exactly the start of Rohini (Moon-ruled): balance should be the full 10 years.
        let v = vimshottari(3.0 * NAKSHATRA_SPAN, &birth, "365.25").unwrap();
        assert_eq!(v["starting_lord"], "Moon");
        assert!((v["balance_at_birth_years"].as_f64().unwrap() - 10.0).abs() < 1e-9);
        let m = v["mahadashas"].as_array().unwrap();
        assert_eq!(m.len(), 9);
        assert_eq!(m[0]["antardashas"].as_array().unwrap().len(), 9);
        // The cycle ends 120 years after the start (no clipping when elapsed is 0).
        for md in m {
            let a = md["antardashas"].as_array().unwrap();
            assert_eq!(
                a.last().unwrap()["end_utc"],
                md["end_utc"],
                "periods must tile exactly"
            );
        }
        let end = Instant::parse_utc(m[8]["end_utc"].as_str().unwrap()).unwrap();
        assert!(((end.jd_ut - birth.jd_ut) / 365.25 - 120.0).abs() < 1e-6);
    }

    #[test]
    fn dasha_clips_first_period_at_birth() {
        let birth = Instant::parse_utc("2000-01-01T12:00:00Z").unwrap();
        // Halfway through Ashwini (Ketu, 7 yr): balance 3.5 yr.
        let v = vimshottari(NAKSHATRA_SPAN / 2.0, &birth, "365.25").unwrap();
        assert!((v["balance_at_birth_years"].as_f64().unwrap() - 3.5).abs() < 1e-9);
        assert_eq!(v["mahadashas"][0]["clipped_at_birth"], true);
        assert_eq!(v["mahadashas"][0]["start_utc"], "2000-01-01T12:00:00Z");
    }

    #[test]
    fn year_conventions_change_the_dates() {
        let birth = Instant::parse_utc("2000-01-01T12:00:00Z").unwrap();
        let a = vimshottari(100.0, &birth, "365.25").unwrap();
        let b = vimshottari(100.0, &birth, "360").unwrap();
        assert_ne!(a["mahadashas"][8]["end_utc"], b["mahadashas"][8]["end_utc"]);
        assert!(vimshottari(100.0, &birth, "lunar").is_err());
    }
}

#[cfg(test)]
mod dasha_level_tests {
    use super::*;

    #[test]
    fn levels_tile_exactly_and_the_chain_contains_the_reference() {
        let birth = Instant::parse_utc("1990-05-15T09:00:00Z").unwrap();
        let v = vimshottari_levels(271.8947, &birth, "365.25", 3, Some(birth.jd_ut + 12_000.0)).unwrap();
        for m in v["mahadashas"].as_array().unwrap() {
            let a = m["antardashas"].as_array().unwrap();
            assert_eq!(a.last().unwrap()["end_utc"], m["end_utc"]);
            for x in a {
                let p = x["pratyantardashas"].as_array().unwrap();
                assert_eq!(p.last().unwrap()["end_utc"], x["end_utc"]);
            }
        }
        let chain = v["current"]["periods"].as_array().unwrap();
        assert_eq!(chain.len(), 5);
        let at = v["current"]["at_utc"].as_str().unwrap();
        for p in chain {
            assert!(p["start_utc"].as_str().unwrap() <= at && at < p["end_utc"].as_str().unwrap());
        }
    }

    #[test]
    fn yogini_start_follows_the_nakshatra_rule() {
        let birth = Instant::parse_utc("2000-01-01T00:00:00Z").unwrap();
        // Ardra (6th nakshatra): (6 + 3) mod 8 = 1 -> Mangala.
        let v = yogini(5.0 * NAKSHATRA_SPAN + 1.0, &birth, "365.25", 2, None).unwrap();
        assert_eq!(v["starting_yogini"], "Mangala");
        // Ashwini (1st): (1 + 3) mod 8 = 4 -> Bhramari.
        let v = yogini(1.0, &birth, "365.25", 1, None).unwrap();
        assert_eq!(v["starting_yogini"], "Bhramari");
    }
}
