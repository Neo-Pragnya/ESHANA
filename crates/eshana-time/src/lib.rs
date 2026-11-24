//! ESHANA time: from a civil moment to the time scales an ephemeris needs.
//!
//! Scope and honesty about the model:
//! - Civil input is resolved with the IANA tz database bundled by `jiff` (historical offsets, LMT, DST). Ambiguous
//!   and skipped local times are errors unless a [`FoldPolicy`] says otherwise.
//! - UTC is used as UT1. They differ by under 0.9 s; the effect is below 0.001° for the Moon and 0.004°
//!   for the Ascendant, and it is recorded as an assumption in provenance.
//! - 1972 onward uses the leap-second table (TAI−UTC) exactly; later instants assume TAI−UTC = 37 s.
//! - Before 1972 TT = UT + ΔT with the Espenak–Meeus polynomials.
//! - Dates are proleptic Gregorian. The Julian/Gregorian cut-over is not modelled in 0.1.

use eshana_core::{EshanaError, Result};
use jiff::civil::{Date, DateTime, Time};
use jiff::tz::{AmbiguousOffset, TimeZone};
use jiff::Span;
use serde::Serialize;

/// Julian day of the J2000.0 epoch.
pub const JD_J2000: f64 = 2_451_545.0;

/// UTC offset (seconds east) of an IANA zone at a UTC Julian day: historical offsets and DST included.
///
/// # Errors
/// `ESH-TZ-UNKNOWN` for an unknown zone; `ESH-INPUT` for an instant jiff cannot represent.
pub fn zone_offset_seconds(zone: &str, jd_utc: f64) -> Result<i32> {
    let tz = TimeZone::get(zone)
        .map_err(|_| EshanaError::TimeZoneUnknown(format!("'{zone}' is not an IANA zone")))?;
    let secs = ((jd_utc - 2_440_587.5) * 86_400.0).round() as i64;
    let ts = jiff::Timestamp::from_second(secs)
        .map_err(|e| EshanaError::Input(format!("instant out of range: {e}")))?;
    Ok(tz.to_offset(ts).seconds())
}

/// Version of the tz database bundled into this build.
#[must_use]
pub fn tzdb_version() -> &'static str {
    jiff_tzdb::VERSION
}

/// What to do with a local time that occurs twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FoldPolicy {
    /// Refuse and report both candidates.
    #[default]
    Reject,
    /// Take the first occurrence (before the clocks go back).
    Earliest,
    /// Take the second occurrence.
    Latest,
}

impl FoldPolicy {
    /// Parse `reject`, `earliest` or `latest`.
    ///
    /// # Errors
    /// [`EshanaError::Input`] for anything else.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "reject" | "" => Ok(Self::Reject),
            "earliest" => Ok(Self::Earliest),
            "latest" => Ok(Self::Latest),
            o => Err(EshanaError::Input(format!(
                "unknown fold policy '{o}' (reject, earliest, latest)"
            ))),
        }
    }
}

/// A resolved instant on every time scale the engine uses.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Instant {
    /// UTC as ISO-8601 with a `Z`.
    pub utc: String,
    /// Julian day, UTC (treated as UT1).
    pub jd_ut: f64,
    /// Julian day, Terrestrial Time.
    pub jd_tt: f64,
    /// TT − UT in seconds used for this instant.
    pub delta_t_s: f64,
    /// Which model produced `delta_t_s`.
    pub time_model: &'static str,
    /// Offset applied from civil time, in seconds east of UTC, when the instant came from local time.
    pub civil_offset_s: Option<i32>,
    /// Zone name, when one was used.
    pub zone: Option<String>,
}

const LEAP: &[(i32, u32, f64)] = &[
    (1972, 1, 10.0),
    (1972, 7, 11.0),
    (1973, 1, 12.0),
    (1974, 1, 13.0),
    (1975, 1, 14.0),
    (1976, 1, 15.0),
    (1977, 1, 16.0),
    (1978, 1, 17.0),
    (1979, 1, 18.0),
    (1980, 1, 19.0),
    (1981, 7, 20.0),
    (1982, 7, 21.0),
    (1983, 7, 22.0),
    (1985, 7, 23.0),
    (1988, 1, 24.0),
    (1990, 1, 25.0),
    (1991, 1, 26.0),
    (1992, 7, 27.0),
    (1993, 7, 28.0),
    (1994, 7, 29.0),
    (1996, 1, 30.0),
    (1997, 7, 31.0),
    (1999, 1, 32.0),
    (2006, 1, 33.0),
    (2009, 1, 34.0),
    (2012, 7, 35.0),
    (2015, 7, 36.0),
    (2017, 1, 37.0),
];

/// Julian day for a proleptic-Gregorian calendar date with fractional day.
#[must_use]
pub fn jd_from_gregorian(year: i32, month: u32, day_frac: f64) -> f64 {
    let (mut y, mut m) = (f64::from(year), f64::from(month));
    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day_frac + b - 1524.5
}

/// Inverse of [`jd_from_gregorian`]: (year, month, day, fraction of day).
#[must_use]
pub fn gregorian_from_jd(jd: f64) -> (i32, u32, u32, f64) {
    let z0 = (jd + 0.5).floor();
    let f = jd + 0.5 - z0;
    let a = if z0 < 2_299_161.0 {
        z0
    } else {
        let alpha = ((z0 - 1_867_216.25) / 36_524.25).floor();
        z0 + 1.0 + alpha - (alpha / 4.0).floor()
    };
    let b = a + 1524.0;
    let c = ((b - 122.1) / 365.25).floor();
    let d = (365.25 * c).floor();
    let e = ((b - d) / 30.6001).floor();
    let day = b - d - (30.6001 * e).floor();
    let month = if e < 14.0 { e - 1.0 } else { e - 13.0 };
    let year = if month > 2.0 { c - 4716.0 } else { c - 4715.0 };
    (year as i32, month as u32, day as u32, f)
}

fn leap_seconds_at(jd_utc: f64) -> Option<f64> {
    let mut found = None;
    for &(y, m, s) in LEAP {
        if jd_utc >= jd_from_gregorian(y, m, 1.0) {
            found = Some(s);
        } else {
            break;
        }
    }
    found
}

/// ΔT = TT − UT in seconds from the Espenak–Meeus polynomials, valid 1800–2100 (used before 1972).
#[must_use]
pub fn delta_t_polynomial(year: f64) -> f64 {
    let y = year;
    if y < 1860.0 {
        let t = y - 1800.0;
        13.72 - 0.332447 * t + 0.0068612 * t.powi(2) + 0.0041116 * t.powi(3) - 0.00037436 * t.powi(4)
            + 0.0000121272 * t.powi(5)
            - 0.0000001699 * t.powi(6)
            + 0.000000000875 * t.powi(7)
    } else if y < 1900.0 {
        let t = y - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t.powi(2) + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233_174.0
    } else if y < 1920.0 {
        let t = y - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t.powi(2) + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else if y < 1941.0 {
        let t = y - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t.powi(2) + 0.0020936 * t.powi(3)
    } else if y < 1961.0 {
        let t = y - 1950.0;
        29.07 + 0.407 * t - t.powi(2) / 233.0 + t.powi(3) / 2547.0
    } else if y < 1986.0 {
        let t = y - 1975.0;
        45.45 + 1.067 * t - t.powi(2) / 260.0 - t.powi(3) / 718.0
    } else if y < 2005.0 {
        let t = y - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t.powi(2)
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else {
        let t = y - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t.powi(2)
    }
}

impl Instant {
    /// Build from a JD in UTC.
    #[must_use]
    pub fn from_jd_utc(jd_utc: f64) -> Self {
        let (y, m, _, _) = gregorian_from_jd(jd_utc);
        let jd_2027 = jd_from_gregorian(2027, 1, 1.0);
        let (delta_t_s, model) = match leap_seconds_at(jd_utc) {
            // After the last published leap second (and until 2027) TAI-UTC stays 37 s.
            Some(tai_utc) if jd_utc < jd_2027 => (tai_utc + 32.184, "leap-seconds-1972-2017+37s-to-2027"),
            // Later: continue from the 2027 value along the Espenak-Meeus growth curve. A forecast, flagged as such.
            Some(tai_utc) => {
                let y = 2000.0 + (jd_utc - JD_J2000) / 365.25;
                (
                    tai_utc + 32.184 + delta_t_polynomial(y) - delta_t_polynomial(2027.0),
                    "leap-seconds-to-2027+espenak-meeus-extrapolation",
                )
            }
            None => (
                delta_t_polynomial(f64::from(y) + (f64::from(m) - 0.5) / 12.0),
                "espenak-meeus-polynomial",
            ),
        };
        let utc = Self::iso_from_jd(jd_utc);
        Self {
            utc,
            jd_ut: jd_utc,
            jd_tt: jd_utc + delta_t_s / 86_400.0,
            delta_t_s,
            time_model: model,
            civil_offset_s: None,
            zone: None,
        }
    }

    fn iso_from_jd(jd: f64) -> String {
        let (y, mo, d, f) = gregorian_from_jd(jd);
        let total_ms = (f * 86_400_000.0).round() as i64;
        let (mut y, mut mo, mut d, mut total_ms) = (y, mo, d, total_ms);
        if total_ms >= 86_400_000 {
            // Rounding carried into the next day.
            if let Ok(n) = Date::new(y as i16, mo as i8, d as i8).and_then(|n| n.tomorrow()) {
                (y, mo, d) = (i32::from(n.year()), n.month() as u32, n.day() as u32);
                total_ms -= 86_400_000;
            }
        }
        let (h, mi, s, ms) = (
            total_ms / 3_600_000,
            (total_ms / 60_000) % 60,
            (total_ms / 1000) % 60,
            total_ms % 1000,
        );
        if ms == 0 {
            format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
        } else {
            format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}.{ms:03}Z")
        }
    }

    /// Build from UTC calendar components.
    ///
    /// # Errors
    /// [`EshanaError::Input`] for an impossible date or time.
    pub fn from_utc(dt: DateTime) -> Result<Self> {
        let sec = f64::from(dt.second()) + f64::from(dt.subsec_nanosecond()) / 1e9;
        let day_frac = f64::from(dt.day())
            + (f64::from(dt.hour()) + f64::from(dt.minute()) / 60.0 + sec / 3600.0) / 24.0;
        let jd = jd_from_gregorian(i32::from(dt.year()), dt.month() as u32, day_frac);
        Ok(Self::from_jd_utc(jd))
    }

    /// Parse `YYYY-MM-DDTHH:MM[:SS[.fff]]Z`, `YYYY-MM-DD HH:MM:SS` (read as UTC) or an RFC 3339 string with an offset.
    ///
    /// # Errors
    /// [`EshanaError::Input`] when the text is not a date-time.
    pub fn parse_utc(s: &str) -> Result<Self> {
        let s = s.trim();
        if let Ok(ts) = s.parse::<jiff::Timestamp>() {
            let off = rfc3339_offset_seconds(s);
            let mut i = Self::from_utc(TimeZone::UTC.to_datetime(ts))?;
            i.civil_offset_s = off;
            return Ok(i);
        }
        let base = s.trim_end_matches('Z');
        if let Ok(dt) = base.replace(' ', "T").parse::<DateTime>() {
            return Self::from_utc(dt);
        }
        Err(EshanaError::Input(format!(
            "cannot read '{s}' as a UTC date-time"
        )))
    }

    /// Resolve a local civil date and time in an IANA zone.
    ///
    /// # Errors
    /// Unknown zone, ambiguous (without a policy) or non-existent local time.
    pub fn from_local(date: Date, time: Time, zone: &str, fold: FoldPolicy) -> Result<Self> {
        let tz = TimeZone::get(zone)
            .map_err(|_| EshanaError::TimeZoneUnknown(format!("'{zone}' is not an IANA zone")))?;
        let naive = date.to_datetime(time);
        let offset = match tz.to_ambiguous_timestamp(naive).offset() {
            AmbiguousOffset::Unambiguous { offset } => offset,
            AmbiguousOffset::Fold { before, after } => match fold {
                FoldPolicy::Reject => {
                    return Err(EshanaError::TimeAmbiguous(format!(
                        "{naive} in {zone} occurs twice (UTC{before} and UTC{after}); pass fold=earliest or fold=latest"
                    )))
                }
                FoldPolicy::Earliest => before,
                FoldPolicy::Latest => after,
            },
            AmbiguousOffset::Gap { .. } => {
                return Err(EshanaError::TimeNonexistent(format!(
                    "{naive} in {zone} was skipped by a clock change; the instant does not exist"
                )))
            }
        };
        let utc = naive
            .checked_add(Span::new().seconds(-i64::from(offset.seconds())))
            .map_err(|e| EshanaError::Input(format!("date out of range: {e}")))?;
        let mut i = Self::from_utc(utc)?;
        i.civil_offset_s = Some(offset.seconds());
        i.zone = Some(zone.to_string());
        Ok(i)
    }

    /// Resolve local civil text (`2000-01-01`, `12:30:00`) in an IANA zone.
    ///
    /// # Errors
    /// Same as [`Instant::from_local`] plus unreadable text.
    pub fn from_local_str(date: &str, time: &str, zone: &str, fold: FoldPolicy) -> Result<Self> {
        let d = parse_date(date)?;
        let t = parse_time(time)?;
        Self::from_local(d, t, zone, fold)
    }

    /// Resolve local civil text with a fixed UTC offset in minutes east (for when no zone is known).
    ///
    /// # Errors
    /// Unreadable text or an offset beyond ±18 h.
    pub fn from_offset_str(date: &str, time: &str, offset_minutes: i32) -> Result<Self> {
        if offset_minutes.abs() > 18 * 60 {
            return Err(EshanaError::Input(format!(
                "UTC offset {offset_minutes} min is beyond ±18 h"
            )));
        }
        let naive = parse_date(date)?.to_datetime(parse_time(time)?);
        let utc = naive
            .checked_add(Span::new().minutes(-i64::from(offset_minutes)))
            .map_err(|e| EshanaError::Input(format!("date out of range: {e}")))?;
        let mut i = Self::from_utc(utc)?;
        i.civil_offset_s = Some(offset_minutes * 60);
        Ok(i)
    }

    /// Julian centuries of TT since J2000.
    #[must_use]
    pub fn t_tt(&self) -> f64 {
        (self.jd_tt - JD_J2000) / 36_525.0
    }

    /// Greenwich mean sidereal time in degrees, `[0, 360)`.
    #[must_use]
    pub fn gmst_deg(&self) -> f64 {
        let d = self.jd_ut - JD_J2000;
        let t = d / 36_525.0;
        eshana_core::norm360(
            280.460_618_37 + 360.985_647_366_29 * d + 0.000_387_933 * t * t - t * t * t / 38_710_000.0,
        )
    }

    /// A new instant shifted by `days` of UT.
    #[must_use]
    pub fn shifted_days(&self, days: f64) -> Self {
        Self::from_jd_utc(self.jd_ut + days)
    }
}

/// Parse `YYYY-MM-DD`.
///
/// # Errors
/// [`EshanaError::Input`] when unreadable or not a real calendar date.
pub fn parse_date(s: &str) -> Result<Date> {
    let s = s.trim();
    let ok = s.len() == 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-';
    ok.then(|| s.parse::<Date>().ok())
        .flatten()
        .ok_or_else(|| EshanaError::Input(format!("date '{s}' must be YYYY-MM-DD")))
}

/// Parse `HH:MM`, `HH:MM:SS` or `HH:MM:SS.fff`.
///
/// # Errors
/// [`EshanaError::Input`] when unreadable.
pub fn parse_time(s: &str) -> Result<Time> {
    let s = s.trim();
    let shape = s.len() >= 5
        && s.as_bytes()[2] == b':'
        && s.bytes().all(|b| b.is_ascii_digit() || b == b':' || b == b'.');
    shape
        .then(|| s.parse::<Time>().ok())
        .flatten()
        .ok_or_else(|| EshanaError::Input(format!("time '{s}' must be HH:MM[:SS[.fff]]")))
}

/// Offset in seconds of an RFC 3339 string (`Z` is 0), or `None` when it has no offset suffix.
fn rfc3339_offset_seconds(s: &str) -> Option<i32> {
    if s.ends_with('Z') || s.ends_with('z') {
        return Some(0);
    }
    let tail = s.get(s.len().checked_sub(6)?..)?;
    let sign = match tail.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let (h, m) = (tail[1..3].parse::<i32>().ok()?, tail[4..6].parse::<i32>().ok()?);
    Some(sign * (h * 3600 + m * 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_is_2451545() {
        let i = Instant::parse_utc("2000-01-01T12:00:00Z").unwrap();
        assert!((i.jd_ut - 2_451_545.0).abs() < 1e-9);
        // TT-UT at J2000 is 32.184 + 32 leap seconds = 64.184 s.
        assert!((i.delta_t_s - 64.184).abs() < 1e-9);
        assert!((i.jd_tt - (2_451_545.0 + 64.184 / 86400.0)).abs() < 1e-9);
    }

    #[test]
    fn meeus_example_jd() {
        // Meeus, Astronomical Algorithms, example 7.a: 1957 Oct 4.81 -> JD 2436116.31
        assert!((jd_from_gregorian(1957, 10, 4.81) - 2_436_116.31).abs() < 1e-9);
        // 333 Jan 27.5 is Julian calendar in Meeus; we only claim Gregorian, so test a modern one:
        assert!((jd_from_gregorian(1987, 1, 27.0) - 2_446_822.5).abs() < 1e-9);
    }

    #[test]
    fn gregorian_roundtrip() {
        for jd in [2_451_545.0, 2_460_000.25, 2_415_020.5, 2_488_069.75] {
            let (y, m, d, f) = gregorian_from_jd(jd);
            let back = jd_from_gregorian(y, m, f64::from(d) + f);
            assert!((back - jd).abs() < 1e-9, "{jd} -> {back}");
        }
    }

    #[test]
    fn leap_second_boundaries() {
        let before = Instant::parse_utc("2016-12-31T23:59:59Z").unwrap();
        let after = Instant::parse_utc("2017-01-01T00:00:00Z").unwrap();
        assert!((before.delta_t_s - (36.0 + 32.184)).abs() < 1e-9);
        assert!((after.delta_t_s - (37.0 + 32.184)).abs() < 1e-9);
    }

    #[test]
    fn delta_t_polynomial_is_continuous_enough() {
        // The polynomial pieces join within a second of each other.
        for edge in [1860.0, 1900.0, 1920.0, 1941.0, 1961.0, 1986.0, 2005.0] {
            let a = delta_t_polynomial(edge - 1e-9);
            let b = delta_t_polynomial(edge);
            assert!((a - b).abs() < 1.0, "discontinuity at {edge}: {a} vs {b}");
        }
        // Known value: dT in 1900 was about -2.7 s, in 1950 about 29 s.
        assert!((delta_t_polynomial(1900.0) + 2.79).abs() < 0.01);
        assert!((delta_t_polynomial(1950.0) - 29.07).abs() < 0.01);
    }

    #[test]
    fn pre_1972_uses_polynomial() {
        let i = Instant::parse_utc("1950-01-01T00:00:00Z").unwrap();
        assert_eq!(i.time_model, "espenak-meeus-polynomial");
        assert!((i.delta_t_s - 29.0).abs() < 1.5);
    }

    #[test]
    fn kolkata_birth_resolves_to_utc() {
        let i =
            Instant::from_local_str("1992-05-28", "20:05:00", "Asia/Kolkata", FoldPolicy::Reject).unwrap();
        assert_eq!(i.utc, "1992-05-28T14:35:00Z");
        assert_eq!(i.civil_offset_s, Some(19_800));
    }

    #[test]
    fn dst_fold_is_an_error_without_policy() {
        let e = Instant::from_local_str("2024-11-03", "01:30:00", "America/New_York", FoldPolicy::Reject)
            .unwrap_err();
        assert_eq!(e.code(), "ESH-TIME-AMBIGUOUS");
        let a = Instant::from_local_str("2024-11-03", "01:30:00", "America/New_York", FoldPolicy::Earliest)
            .unwrap();
        let b = Instant::from_local_str("2024-11-03", "01:30:00", "America/New_York", FoldPolicy::Latest)
            .unwrap();
        assert_eq!(a.utc, "2024-11-03T05:30:00Z");
        assert_eq!(b.utc, "2024-11-03T06:30:00Z");
    }

    #[test]
    fn dst_gap_is_an_error() {
        let e = Instant::from_local_str("2024-03-10", "02:30:00", "America/New_York", FoldPolicy::Latest)
            .unwrap_err();
        assert_eq!(e.code(), "ESH-TIME-NONEXISTENT");
    }

    #[test]
    fn unknown_zone_is_an_error() {
        let e =
            Instant::from_local_str("2000-01-01", "00:00", "Mars/Olympus", FoldPolicy::Reject).unwrap_err();
        assert_eq!(e.code(), "ESH-TZ-UNKNOWN");
    }

    #[test]
    fn historical_zone_offset_is_applied() {
        // India was UTC+5:30 in 1992 but Kolkata used LMT-like +5:53:28 before 1906.
        let a =
            Instant::from_local_str("1900-01-01", "12:00:00", "Asia/Kolkata", FoldPolicy::Reject).unwrap();
        assert_ne!(a.civil_offset_s, Some(19_800));
    }

    #[test]
    fn gmst_at_j2000_matches_published_value() {
        // GMST at 2000-01-01 12:00 UT is 18h 41m 50.55s = 280.4606°.
        let i = Instant::parse_utc("2000-01-01T12:00:00Z").unwrap();
        assert!((i.gmst_deg() - 280.460_618).abs() < 1e-4);
    }

    #[test]
    fn meeus_gmst_example() {
        // Meeus example 12.a: 1987 April 10 at 0h UT -> GMST 13h10m46.3668s = 197.693195°
        let i = Instant::parse_utc("1987-04-10T00:00:00Z").unwrap();
        assert!((i.gmst_deg() - 197.693_195).abs() < 1e-4);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Instant::parse_utc("not a date").is_err());
        assert!(Instant::from_offset_str("2000-01-01", "10:00", 99_999).is_err());
        assert!(parse_time("25:61").is_err());
    }

    #[test]
    fn rfc3339_offset_is_converted() {
        let i = Instant::parse_utc("2000-01-01T05:30:00+05:30").unwrap();
        assert_eq!(i.utc, "2000-01-01T00:00:00Z");
    }
}

#[cfg(test)]
mod properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn jd_roundtrips(jd in 2_305_447.5f64..2_816_787.5) {
            let (y, m, d, f) = gregorian_from_jd(jd);
            let back = jd_from_gregorian(y, m, f64::from(d) + f);
            prop_assert!((back - jd).abs() < 1e-8);
        }

        #[test]
        fn jd_is_monotonic(a in 2_400_000.5f64..2_500_000.5, b in 0.001f64..1000.0) {
            let i1 = Instant::from_jd_utc(a);
            let i2 = Instant::from_jd_utc(a + b);
            prop_assert!(i2.jd_tt > i1.jd_tt);
        }

        #[test]
        fn never_panics_on_text(s in "\\PC*") {
            let _ = Instant::parse_utc(&s);
            let _ = Instant::from_local_str(&s, &s, &s, FoldPolicy::Reject);
        }
    }
}

#[cfg(test)]
mod zone_offset_tests {
    use super::*;

    #[test]
    fn offsets_follow_dst_and_half_hour_zones() {
        let jan = jd_from_gregorian(2025, 1, 15.5);
        let jul = jd_from_gregorian(2025, 7, 15.5);
        assert_eq!(zone_offset_seconds("Europe/London", jan).unwrap(), 0);
        assert_eq!(zone_offset_seconds("Europe/London", jul).unwrap(), 3600);
        assert_eq!(zone_offset_seconds("Asia/Kolkata", jan).unwrap(), 19_800);
        assert_eq!(zone_offset_seconds("Asia/Kathmandu", jan).unwrap(), 20_700);
        assert_eq!(
            zone_offset_seconds("Nope/Zone", jan).unwrap_err().code(),
            "ESH-TZ-UNKNOWN"
        );
    }
}
