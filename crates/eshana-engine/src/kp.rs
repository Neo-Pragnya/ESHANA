//! KP horary: the 1–249 number table, cusp-occupancy conventions and horary cusps from a number.
//!
//! **The table.** Each nakṣatra (13°20′) is divided among the nine Vimśottarī lords in proportion to their daśā
//! years, starting from the nakṣatra's own lord: 27 × 9 = 243 subs. Nine of the twelve sign boundaries fall inside a
//! nakṣatra; six of those fall inside a sub, which the sign boundary cuts in two, giving 243 + 6 = 249 numbers. Every
//! boundary is a whole number of arc-seconds (a nakṣatra is 48 000″ and a lord of `y` years gets `400·y″`), so the
//! table is built in integer arc-seconds and tiles 0–360° exactly.
//!
//! **Horary cusps.** The querent's number fixes the sidereal Ascendant at the start of that number's arc. The engine
//! finds the local sidereal time at which that degree rises at the place, at the moment of judgement, and takes
//! Placidus cusps for that sidereal time. Planets are those of the moment of judgement.
//!
//! **Occupancy.** Which house holds a planet near a cusp is a convention. `cusp_start` (the default, unchanged) puts
//! a planet in the house whose cusp it has passed; `cusp_midpoint` (opt in) treats each cusp as the middle of its
//! house, the house running from the midpoint with the previous cusp to the midpoint with the next.

use crate::astro;
use crate::vedic::DASHA_ORDER;
use eshana_core::{angular_diff, norm360, EshanaError, HouseSystem, Result};
use serde_json::{json, Value};

/// Arc-seconds in a nakṣatra.
const NAK_AS: i64 = 48_000;
/// Arc-seconds in a sign.
const SIGN_AS: i64 = 108_000;
/// Arc-seconds in the circle.
const CIRCLE_AS: i64 = 1_296_000;

/// One horary number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoraryEntry {
    /// 1–249.
    pub number: u16,
    /// Start, arc-seconds from 0° sidereal Aries (included).
    pub start_as: i64,
    /// End, arc-seconds (excluded).
    pub end_as: i64,
    /// Sign index 0–11.
    pub sign: usize,
    /// Index into [`DASHA_ORDER`] of the star (nakṣatra) lord.
    pub star: usize,
    /// Index into [`DASHA_ORDER`] of the sub lord.
    pub sub: usize,
}

/// The 249 numbers in order.
#[must_use]
pub fn horary_table() -> Vec<HoraryEntry> {
    let mut out = Vec::with_capacity(249);
    for nak in 0..27_i64 {
        let star = (nak % 9) as usize;
        let mut at = nak * NAK_AS;
        for k in 0..9 {
            let sub = (star + k) % 9;
            let len = 400 * DASHA_ORDER[sub].1 as i64;
            let end = at + len;
            // A sign boundary strictly inside this sub splits it.
            let next_sign = (at / SIGN_AS + 1) * SIGN_AS;
            let pieces = if next_sign < end {
                vec![(at, next_sign), (next_sign, end)]
            } else {
                vec![(at, end)]
            };
            for (s, e) in pieces {
                out.push(HoraryEntry {
                    number: (out.len() + 1) as u16,
                    start_as: s,
                    end_as: e,
                    sign: (s / SIGN_AS) as usize,
                    star,
                    sub,
                });
            }
            at = end;
        }
    }
    debug_assert_eq!(out.last().map(|e| e.end_as), Some(CIRCLE_AS));
    out
}

fn dms(arcsec: i64) -> String {
    let within = arcsec.rem_euclid(SIGN_AS);
    format!(
        "{}°{:02}′{:02}″",
        within / 3600,
        (within % 3600) / 60,
        within % 60
    )
}

fn lord(i: usize) -> String {
    DASHA_ORDER[i].0.to_lowercase()
}

/// One entry as JSON: sign, star and sub lords, the arc in degrees and within its sign.
#[must_use]
pub fn entry_json(e: &HoraryEntry) -> Value {
    let end_in_sign = if e.end_as % SIGN_AS == 0 {
        "30°00′00″".to_string()
    } else {
        dms(e.end_as)
    };
    json!({
        "number": e.number,
        "sign": crate::astro::RASHIS[e.sign],
        "sign_index": e.sign,
        "star_lord": lord(e.star),
        "sub_lord": lord(e.sub),
        "start_deg": e.start_as as f64 / 3600.0,
        "end_deg": e.end_as as f64 / 3600.0,
        "start_in_sign": dms(e.start_as),
        "end_in_sign": end_in_sign,
    })
}

/// The whole table as JSON.
#[must_use]
pub fn table_json() -> Value {
    let t = horary_table();
    json!({"count": t.len(), "unit": "sidereal degrees", "bounds": "[start, end)",
           "entries": t.iter().map(entry_json).collect::<Vec<_>>()})
}

/// The entry for a number, or `ESH-INPUT`.
///
/// # Errors
/// [`EshanaError::Input`] outside 1–249.
pub fn entry(number: u32) -> Result<HoraryEntry> {
    if !(1..=249).contains(&number) {
        return Err(EshanaError::Input(format!(
            "KP horary number must be 1–249, got {number}"
        )));
    }
    Ok(horary_table()[(number - 1) as usize])
}

/// Cusp-occupancy convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occupancy {
    /// A planet belongs to the house whose cusp it has passed (cusp = start of the house). Default.
    CuspStart,
    /// The cusp is the middle of its house; boundaries are midpoints between consecutive cusps.
    CuspMidpoint,
}

impl Occupancy {
    /// Parse a method id (`kp_cusp_occupancy` family); `None` is the default.
    ///
    /// # Errors
    /// `ESH-METHOD` for an unknown id.
    pub fn parse(id: Option<&str>) -> Result<Self> {
        match id {
            None => Ok(Self::CuspStart),
            Some(s) => match crate::methods::require("kp_cusp_occupancy", s)? {
                "cusp_midpoint" => Ok(Self::CuspMidpoint),
                _ => Ok(Self::CuspStart),
            },
        }
    }

    /// Method id.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::CuspStart => "cusp_start",
            Self::CuspMidpoint => "cusp_midpoint",
        }
    }

    /// One line for the result.
    #[must_use]
    pub fn describe(self) -> &'static str {
        match self {
            Self::CuspStart => "Placidus cusps; a graha belongs to the house whose cusp precedes it",
            Self::CuspMidpoint => {
                "Placidus cusps as house middles; a house runs from the midpoint with the previous cusp to the midpoint with the next"
            }
        }
    }

    /// House 1–12 holding `lon` under this convention.
    #[must_use]
    pub fn house_of(self, lon: f64, cusps: &[f64; 12]) -> usize {
        match self {
            Self::CuspStart => astro::house_of(lon, cusps),
            Self::CuspMidpoint => {
                let mut starts = [0.0; 12];
                for i in 0..12 {
                    let prev = cusps[(i + 11) % 12];
                    starts[i] = norm360(prev + norm360(cusps[i] - prev) / 2.0);
                }
                astro::house_of(lon, &starts)
            }
        }
    }
}

/// Local apparent sidereal time (degrees) at which the tropical Ascendant equals `asc_tropical` at latitude `lat`.
///
/// The Ascendant advances monotonically through 360° as the sidereal time does, so a 1° scan brackets the root and
/// bisection refines it to 1e-9°.
///
/// # Errors
/// `ESH-LATITUDE` when no sidereal time gives that Ascendant (inside the polar circles some degrees never rise).
pub fn lst_for_ascendant(asc_tropical: f64, lat: f64, t: f64) -> Result<f64> {
    let f = |lst: f64| angular_diff(astro::asc_mc(lst, lat, t).0, asc_tropical);
    let mut prev = (0.0, f(0.0));
    for i in 1..=360 {
        let x = f64::from(i);
        let fx = f(x);
        // a genuine sign change, not the ±180° wrap of the difference
        if prev.1 <= 0.0 && fx > 0.0 && (fx - prev.1).abs() < 90.0 {
            let (mut lo, mut hi) = (prev.0, x);
            for _ in 0..80 {
                let mid = 0.5 * (lo + hi);
                if f(mid) <= 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let lst = 0.5 * (lo + hi);
            if f(lst).abs() < 1e-6 {
                return Ok(norm360(lst));
            }
        }
        prev = (x, fx);
    }
    Err(EshanaError::LatitudeUnsupported(format!(
        "no sidereal time puts {asc_tropical:.4}° on the Ascendant at latitude {lat}"
    )))
}

/// Horary cusps (sidereal, Placidus) for a number, given the zodiac offset `off` (nutation plus ayanāṃśa) at the
/// moment of judgement. Returns the sidereal time used and the twelve cusps.
///
/// # Errors
/// `ESH-INPUT` for a bad number; `ESH-LATITUDE` where the degree does not rise or Placidus is undefined.
pub fn horary_cusps(number: u32, lat: f64, t: f64, off: f64) -> Result<(f64, [f64; 12])> {
    let e = entry(number)?;
    let asc_sid = e.start_as as f64 / 3600.0;
    let lst = lst_for_ascendant(norm360(asc_sid + off), lat, t)?;
    let mut c = astro::houses(HouseSystem::Placidus, lst, lat, t)?;
    for v in &mut c {
        *v = norm360(*v - off);
    }
    // The first cusp is the number's start by construction; pin it so rounding never moves it off the arc.
    c[0] = asc_sid;
    Ok((lst, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_249_entries_that_tile_the_circle() {
        let t = horary_table();
        assert_eq!(t.len(), 249);
        assert_eq!(t[0].start_as, 0);
        assert_eq!(t[248].end_as, CIRCLE_AS);
        for w in t.windows(2) {
            assert_eq!(w[0].end_as, w[1].start_as, "gap or overlap at {}", w[1].number);
        }
        for (i, e) in t.iter().enumerate() {
            assert_eq!(usize::from(e.number), i + 1);
            assert!(e.end_as > e.start_as);
            assert_eq!(
                e.start_as / SIGN_AS,
                (e.end_as - 1) / SIGN_AS,
                "entry {} crosses a sign",
                e.number
            );
        }
    }

    #[test]
    fn exactly_six_subs_are_split_by_sign_boundaries() {
        let t = horary_table();
        let split: Vec<i64> = t
            .windows(2)
            .filter(|w| w[0].star == w[1].star && w[0].sub == w[1].sub)
            .map(|w| w[1].start_as / 3600)
            .collect();
        // 30°, 90°, 150°, 210°, 270°, 330° split a sub; 60°, 180°, 300° fall on a sub boundary;
        // 0°, 120°, 240° are nakṣatra boundaries.
        assert_eq!(split, vec![30, 90, 150, 210, 270, 330]);
    }

    #[test]
    fn spot_values() {
        let t = horary_table();
        let j = |n: usize| entry_json(&t[n - 1]);
        // 1: Aries 0°00′00″–0°46′40″, Ketu star, Ketu sub.
        let e = j(1);
        assert_eq!(
            (e["star_lord"].as_str(), e["sub_lord"].as_str()),
            (Some("ketu"), Some("ketu"))
        );
        assert_eq!(e["end_in_sign"], "0°46′40″");
        // 249: Pisces 27°53′20″–30°00′00″, Mercury star (Revatī), Saturn sub.
        let e = j(249);
        assert_eq!(e["sign"], "Meena");
        assert_eq!(
            (e["star_lord"].as_str(), e["sub_lord"].as_str()),
            (Some("mercury"), Some("saturn"))
        );
        assert_eq!(e["start_in_sign"], "27°53′20″");
        // The Kṛttikā Rāhu sub is cut at 30°: Aries 29°13′20″–30° and Taurus 0°–1°13′20″, both Sun/Rāhu.
        let a = t.iter().position(|x| x.end_as == SIGN_AS).unwrap();
        let (x, y) = (entry_json(&t[a]), entry_json(&t[a + 1]));
        assert_eq!(x["start_in_sign"], "29°13′20″");
        assert_eq!(y["end_in_sign"], "1°13′20″");
        assert_eq!(
            (x["star_lord"].as_str(), x["sub_lord"].as_str()),
            (Some("sun"), Some("rahu"))
        );
        assert_eq!(y["sub_lord"], "rahu");
    }

    #[test]
    fn numbers_outside_the_table_are_refused() {
        assert_eq!(entry(0).unwrap_err().code(), "ESH-INPUT");
        assert_eq!(entry(250).unwrap_err().code(), "ESH-INPUT");
    }

    #[test]
    fn occupancy_conventions_differ_only_near_cusps() {
        let mut cusps = [0.0; 12];
        for (i, c) in cusps.iter_mut().enumerate() {
            *c = 30.0 * i as f64 + 5.0;
        }
        // 34° is just before the 2nd cusp (35°): house 1 when the cusp starts the house, house 2 when it is its middle.
        assert_eq!(Occupancy::CuspStart.house_of(34.0, &cusps), 1);
        assert_eq!(Occupancy::CuspMidpoint.house_of(34.0, &cusps), 2);
        // Mid-house planets differ by the half-house shift; far from either boundary both agree.
        assert_eq!(Occupancy::CuspStart.house_of(40.0, &cusps), 2);
        assert_eq!(Occupancy::CuspMidpoint.house_of(40.0, &cusps), 2);
        assert_eq!(Occupancy::parse(Some("koch")).unwrap_err().code(), "ESH-METHOD");
        assert_eq!(Occupancy::parse(None).unwrap(), Occupancy::CuspStart);
    }

    #[test]
    fn horary_ascendant_is_the_start_of_the_number() {
        for n in [1_u32, 13, 14, 100, 249] {
            let (_, c) = horary_cusps(n, 13.08, 0.25, 24.0).unwrap();
            let e = entry(n).unwrap();
            assert!((c[0] - e.start_as as f64 / 3600.0).abs() < 1e-9, "{n}");
        }
    }
}
