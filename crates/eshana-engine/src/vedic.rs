//! Vedic rule pack `vedic.parashari@0.1`: nakshatra and pada, divisional charts and Vimśottarī daśā.
//!
//! Every function takes a **sidereal** longitude. Divisional-chart conventions and the daśā year length are
//! school-specific; the rule version names the exact convention implemented here.

use eshana_core::{norm360, EshanaError, Result};
use eshana_time::Instant;
use serde_json::{json, Value};

/// Rule pack id and version for everything in this module.
pub const RULESET: &str = "vedic.parashari@0.1";

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

/// Divisional (varga) sign index 0–11 for a sidereal longitude, Parāśara conventions.
///
/// Implemented: D1, D3, D4, D7, D9, D10, D12.
///
/// # Errors
/// [`EshanaError::Rules`] for a division that is not implemented yet (never guessed).
pub fn divisional_sign(sidereal_lon: f64, d: u32) -> Result<usize> {
    let lon = norm360(sidereal_lon);
    let s = ((lon / 30.0).floor() as usize).min(11);
    let within = lon - s as f64 * 30.0;
    let part = |n: u32| ((within / (30.0 / f64::from(n))).floor() as usize).min(n as usize - 1);
    let odd = s % 2 == 0; // Aries (index 0) is the first, odd sign
    Ok(match d {
        1 => s,
        3 => (s + 4 * part(3)) % 12,
        4 => (s + 3 * part(4)) % 12,
        7 => (if odd { s } else { s + 6 } + part(7)) % 12,
        9 => (((lon * 9.0 / 30.0).floor() as usize) % 12).min(11),
        10 => (if odd { s } else { s + 8 } + part(10)) % 12,
        12 => (s + part(12)) % 12,
        other => {
            return Err(EshanaError::Rules(format!(
            "divisional chart D{other} is not implemented in {RULESET} (implemented: 1, 3, 4, 7, 9, 10, 12)"
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

/// Full Vimśottarī daśā timeline from the Moon's sidereal longitude at birth: nine mahādaśās, each with its
/// nine antardaśās, the first period clipped at birth. Dates are UTC ISO strings.
///
/// # Errors
/// Unknown year convention.
pub fn vimshottari(moon_sidereal_lon: f64, birth: &Instant, convention: &str) -> Result<Value> {
    let ydays = year_days(convention)?;
    let nk = nakshatra_of(moon_sidereal_lon);
    let first = nk.index % 9;
    let total_years: f64 = DASHA_ORDER.iter().map(|d| d.1).sum();
    let first_len = DASHA_ORDER[first].1;
    let elapsed_years = nk.elapsed * first_len;
    let balance_years = first_len - elapsed_years;
    let mut start_jd = birth.jd_ut - elapsed_years * ydays;
    let birth_jd = birth.jd_ut;
    let iso = |jd: f64| Instant::from_jd_utc(jd).utc;
    let mut maha = vec![];
    for k in 0..9 {
        let (lord, years) = DASHA_ORDER[(first + k) % 9];
        let end_jd = start_jd + years * ydays;
        let mut sub_start = start_jd;
        let mut antars = vec![];
        for j in 0..9 {
            let (sl, sy) = DASHA_ORDER[(first + k + j) % 9];
            // The last sub-period ends exactly where its mahadasha ends, so periods tile with no seam from
            // floating-point accumulation.
            let sub_end = if j == 8 {
                end_jd
            } else {
                sub_start + years * sy / total_years * ydays
            };
            if sub_end > birth_jd {
                antars.push(json!({
                    "lord": sl,
                    "start_utc": iso(sub_start.max(birth_jd)),
                    "end_utc": iso(sub_end),
                    "clipped_at_birth": sub_start < birth_jd,
                }));
            }
            sub_start = sub_end;
        }
        maha.push(json!({
            "lord": lord,
            "years": years,
            "start_utc": iso(start_jd.max(birth_jd)),
            "end_utc": iso(end_jd),
            "clipped_at_birth": start_jd < birth_jd,
            "antardashas": antars,
        }));
        start_jd = end_jd;
    }
    Ok(json!({
        "system": "vimshottari",
        "year_convention": convention,
        "year_days": ydays,
        "moon_nakshatra": nk.name,
        "starting_lord": DASHA_ORDER[first].0,
        "balance_at_birth_years": balance_years,
        "mahadashas": maha,
    }))
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
        assert_eq!(divisional_sign(10.0, 30).unwrap_err().code(), "ESH-RULES");
        assert_eq!(divisional_sign(10.0, 60).unwrap_err().code(), "ESH-RULES");
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
