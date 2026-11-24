//! ESHANA time: from a civil moment to the time scales an ephemeris needs.
//!
//! Scope and honesty about the model:
//! - Civil input is resolved with the IANA tz database bundled by `jiff` (historical offsets, LMT, DST). Ambiguous
//!   and skipped local times are errors unless a [`FoldPolicy`] says otherwise.
//! - UTC is used as UT1. They differ by under 0.9 s; the effect is below 0.001° for the Moon and 0.004°
//!   for the Ascendant, and it is recorded as an assumption in provenance.
//! - 1972 onward uses the leap-second table (TAI−UTC) exactly; later instants assume TAI−UTC = 37 s.
//! - Before 1972 TT = UT + ΔT with the Espenak–Meeus polynomials (−500 to 2150), and the Morrison–Stephenson
//!   long-term parabola outside that range.
//! - Dates are proleptic Gregorian (ISO 8601) unless a [`Calendar`] says otherwise: `julian`, or `auto` (Julian
//!   before 1582-10-15, Gregorian from then, the historical cut-over). Years are astronomical: year 0 is 1 BCE and
//!   year −43 is 44 BCE.

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

/// Which calendar a civil date is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Calendar {
    /// Proleptic Gregorian (ISO 8601), for every date. The default.
    Gregorian,
    /// Proleptic Julian, for every date.
    Julian,
    /// Julian before 1582-10-15 (Gregorian), Gregorian from then: the historical cut-over of 1582-10-04/15.
    Auto,
}

impl Calendar {
    /// Parse `gregorian`, `julian` or `auto`.
    ///
    /// # Errors
    /// [`EshanaError::Input`] for anything else.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "gregorian" => Ok(Self::Gregorian),
            "julian" => Ok(Self::Julian),
            "auto" => Ok(Self::Auto),
            o => Err(EshanaError::Input(format!(
                "unknown calendar '{o}' (gregorian, julian, auto)"
            ))),
        }
    }
}

/// JD of 1582-10-15 00:00 (Gregorian), the first day of the Gregorian calendar.
pub const JD_GREGORIAN_START: f64 = 2_299_160.5;

/// Julian day for a proleptic-Julian calendar date with fractional day (astronomical years).
#[must_use]
pub fn jd_from_julian(year: i32, month: u32, day_frac: f64) -> f64 {
    let (mut y, mut m) = (f64::from(year), f64::from(month));
    if m <= 2.0 {
        y -= 1.0;
        m += 12.0;
    }
    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day_frac - 1524.5
}

/// Inverse of [`jd_from_julian`]: (year, month, day, fraction of day) in the proleptic Julian calendar.
#[must_use]
pub fn julian_from_jd(jd: f64) -> (i32, u32, u32, f64) {
    civil_from_jd(jd, false)
}

/// Days in a month of the given calendar.
#[must_use]
pub fn days_in_month(year: i32, month: u32, julian: bool) -> u32 {
    let leap = if julian {
        year.rem_euclid(4) == 0
    } else {
        year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
    };
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    }
}

/// A civil date read in a calendar, as the proleptic-Gregorian date the rest of the engine uses, plus a note when the
/// calendar changed it (`"julian 1500-02-29 = gregorian 1500-03-10"`).
///
/// Accepts `YYYY-MM-DD` and signed astronomical years (`-0043-03-15` is 44 BCE). Under [`Calendar::Auto`] a date
/// in the skipped days 1582-10-05 … 1582-10-14 is an error (those days never existed).
///
/// # Errors
/// [`EshanaError::Input`] when unreadable, not a real date in that calendar, or in the 1582 gap under `auto`.
pub fn parse_date_in(s: &str, cal: Calendar) -> Result<(Date, Option<String>)> {
    let t = s.trim();
    let bad = || {
        EshanaError::Input(format!(
            "date '{t}' must be YYYY-MM-DD (a signed year such as -0043-03-15 for 44 BCE)"
        ))
    };
    let (neg, body) = t.strip_prefix('-').map_or((false, t), |b| (true, b));
    let parts: Vec<&str> = body.split('-').collect();
    if parts.len() != 3
        || parts[0].len() < 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || !parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err(bad());
    }
    let year = parts[0].parse::<i32>().map_err(|_| bad())? * if neg { -1 } else { 1 };
    let (month, day) = (
        parts[1].parse::<u32>().map_err(|_| bad())?,
        parts[2].parse::<u32>().map_err(|_| bad())?,
    );
    let julian = match cal {
        Calendar::Gregorian => false,
        Calendar::Julian => true,
        Calendar::Auto => {
            if (year, month, day) > (1582, 10, 4) && (year, month, day) < (1582, 10, 15) {
                return Err(EshanaError::Input(format!(
                    "{t} does not exist: the Gregorian calendar began on 1582-10-15, the day after Julian 1582-10-04"
                )));
            }
            (year, month, day) < (1582, 10, 15)
        }
    };
    if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month, julian) {
        return Err(EshanaError::Input(format!(
            "{t} is not a date in the {} calendar",
            if julian { "Julian" } else { "Gregorian" }
        )));
    }
    let (gy, gm, gd) = if julian {
        let (y, m, d, _) = gregorian_from_jd(jd_from_julian(year, month, f64::from(day)));
        (y, m, d)
    } else {
        (year, month, day)
    };
    let date = Date::new(
        i16::try_from(gy).map_err(|_| bad())?,
        i8::try_from(gm).map_err(|_| bad())?,
        i8::try_from(gd).map_err(|_| bad())?,
    )
    .map_err(|_| EshanaError::Input(format!("{t} is outside the supported years (-9999 to 9999)")))?;
    let note = julian.then(|| format!("julian {t} = gregorian {}", iso_date(gy, gm, gd)));
    Ok((date, note))
}

/// `YYYY-MM-DD` with a sign for negative (astronomical) years.
#[must_use]
pub fn iso_date(y: i32, m: u32, d: u32) -> String {
    if y < 0 {
        format!("-{:04}-{m:02}-{d:02}", -y)
    } else {
        format!("{y:04}-{m:02}-{d:02}")
    }
}

/// Julian day for a proleptic-Gregorian calendar date with fractional day (astronomical years).
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

/// Inverse of [`jd_from_gregorian`]: (year, month, day, fraction of day) in the proleptic Gregorian calendar.
///
/// (Before 0.4.5 this switched to the Julian calendar before 1582, so pre-1582 dates did not round-trip.)
#[must_use]
pub fn gregorian_from_jd(jd: f64) -> (i32, u32, u32, f64) {
    civil_from_jd(jd, true)
}

/// Meeus ch. 7, valid for JD ≥ 0; Gregorian or Julian.
fn civil_from_jd(jd: f64, gregorian: bool) -> (i32, u32, u32, f64) {
    let z0 = (jd + 0.5).floor();
    let f = jd + 0.5 - z0;
    let a = if gregorian {
        let alpha = ((z0 - 1_867_216.25) / 36_524.25).floor();
        z0 + 1.0 + alpha - (alpha / 4.0).floor()
    } else {
        z0
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

/// ΔT = TT − UT in seconds: the Espenak–Meeus polynomials (NASA, "Five Millennium Canon of Solar Eclipses",
/// 2006) from −500 to 2150, and the Morrison–Stephenson long-term parabola −20 + 32 u² (u = (year − 1820)/100)
/// outside them. Used before 1972 and, as a growth curve, for forecasts after the last leap-second year.
#[must_use]
pub fn delta_t_polynomial(year: f64) -> f64 {
    let y = year;
    let long_term = |y: f64| {
        let u = (y - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    };
    if y < -500.0 {
        long_term(y)
    } else if y < 500.0 {
        let u = y / 100.0;
        10583.6 - 1014.41 * u + 33.78311 * u.powi(2) - 5.952053 * u.powi(3) - 0.1798452 * u.powi(4)
            + 0.022174192 * u.powi(5)
            + 0.0090316521 * u.powi(6)
    } else if y < 1600.0 {
        let u = (y - 1000.0) / 100.0;
        1574.2 - 556.01 * u + 71.23472 * u.powi(2) + 0.319781 * u.powi(3)
            - 0.8503463 * u.powi(4)
            - 0.005050998 * u.powi(5)
            + 0.0083572073 * u.powi(6)
    } else if y < 1700.0 {
        let t = y - 1600.0;
        120.0 - 0.9808 * t - 0.01532 * t.powi(2) + t.powi(3) / 7129.0
    } else if y < 1800.0 {
        let t = y - 1700.0;
        8.83 + 0.1603 * t - 0.0059285 * t.powi(2) + 0.00013336 * t.powi(3) - t.powi(4) / 1_174_000.0
    } else if y < 1860.0 {
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
    } else if y < 2050.0 {
        let t = y - 2000.0;
        62.92 + 0.32217 * t + 0.005589 * t.powi(2)
    } else if y < 2150.0 {
        long_term(y) - 0.5628 * (2150.0 - y)
    } else {
        long_term(y)
    }
}

/// UT1 − UTC at 2027.0 and its yearly rate, from IERS Bulletin A Vol. XXXIX No. 047 (19 November 2026):
/// UT1 − UTC = −0.1287 − 0.00049 (MJD − 61370) − (UT2 − UT1), i.e. about −0.10 s at 2027.0 and −0.179 s a year.
const BULLETIN_A_UT1_UTC_2027: f64 = -0.10;
const BULLETIN_A_UT1_UTC_RATE: f64 = -0.179;

/// ΔT forecast after 2027 (no leap seconds are announced that far): the IERS Bulletin A trend (TT − UTC = 69.184 s
/// plus the predicted drift of UT1 − UTC), blended quadratically into the Espenak–Meeus curve so that it reaches
/// that curve in 2050 and follows it after. A forecast: its uncertainty grows by roughly a second a year.
#[must_use]
pub fn delta_t_forecast(year: f64) -> f64 {
    let trend = 69.184 - (BULLETIN_A_UT1_UTC_2027 + BULLETIN_A_UT1_UTC_RATE * (year - 2027.0));
    let w = ((year - 2027.0) / 23.0).clamp(0.0, 1.0).powi(2);
    trend * (1.0 - w) + delta_t_polynomial(year) * w
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
            // Later: a forecast, flagged as such (TIME-05). See [`delta_t_forecast`].
            Some(_) => {
                let y = 2000.0 + (jd_utc - JD_J2000) / 365.25;
                (
                    delta_t_forecast(y),
                    "iers-bulletin-a-trend-blended-to-espenak-meeus-by-2050",
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
        let date = iso_date(y, mo, d);
        if ms == 0 {
            format!("{date}T{h:02}:{mi:02}:{s:02}Z")
        } else {
            format!("{date}T{h:02}:{mi:02}:{s:02}.{ms:03}Z")
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

    /// Like [`Instant::from_local_str`], with the date read in a [`Calendar`]; returns the note when the calendar
    /// converted the date.
    ///
    /// # Errors
    /// Same as [`Instant::from_local_str`] plus a date that is not real in that calendar.
    pub fn from_local_str_in(
        date: &str,
        time: &str,
        zone: &str,
        fold: FoldPolicy,
        cal: Calendar,
    ) -> Result<(Self, Option<String>)> {
        let (d, note) = parse_date_in(date, cal)?;
        Ok((Self::from_local(d, parse_time(time)?, zone, fold)?, note))
    }

    /// Like [`Instant::from_offset_str`], with the date read in a [`Calendar`].
    ///
    /// # Errors
    /// Same as [`Instant::from_offset_str`] plus a date that is not real in that calendar.
    pub fn from_offset_str_in(
        date: &str,
        time: &str,
        offset_minutes: i32,
        cal: Calendar,
    ) -> Result<(Self, Option<String>)> {
        let (d, note) = parse_date_in(date, cal)?;
        Ok((Self::from_offset_str(&d.to_string(), time, offset_minutes)?, note))
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
    parse_date_in(s, Calendar::Gregorian).map(|(d, _)| d)
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
    fn julian_calendar_matches_meeus() {
        // Meeus 7.b: 333 Jan 27.5 (Julian) = JD 1842713.0; -584 May 28.63 = JD 1507900.13
        assert!((jd_from_julian(333, 1, 27.5) - 1_842_713.0).abs() < 1e-9);
        assert!((jd_from_julian(-584, 5, 28.63) - 1_507_900.13).abs() < 1e-6);
        // the cut-over: Julian 1582-10-04 is followed by Gregorian 1582-10-15
        assert!((jd_from_julian(1582, 10, 4.0) - 2_299_159.5).abs() < 1e-9);
        assert!((jd_from_gregorian(1582, 10, 15.0) - JD_GREGORIAN_START).abs() < 1e-9);
        assert_eq!(julian_from_jd(1_842_713.0), (333, 1, 27, 0.5));
    }

    #[test]
    fn proleptic_gregorian_roundtrips_before_1582() {
        for (y, m, d) in [
            (1500, 3, 10),
            (100, 1, 1),
            (0, 2, 29),
            (-43, 3, 15),
            (-500, 12, 31),
            (1582, 10, 5),
        ] {
            let jd = jd_from_gregorian(y, m, f64::from(d));
            assert_eq!(gregorian_from_jd(jd), (y, m, d, 0.0), "{y}-{m}-{d}");
        }
    }

    #[test]
    fn calendars_read_dates() {
        let (g, n) = parse_date_in("1500-02-29", Calendar::Julian).unwrap();
        assert_eq!(
            (g.to_string(), n.as_deref()),
            (
                "1500-03-10".to_string(),
                Some("julian 1500-02-29 = gregorian 1500-03-10")
            )
        );
        assert!(parse_date_in("1500-02-29", Calendar::Gregorian).is_err()); // not a Gregorian leap year
        assert!(parse_date_in("1582-10-10", Calendar::Auto).is_err()); // the skipped days
        assert_eq!(parse_date_in("1582-10-15", Calendar::Auto).unwrap().1, None);
        assert_eq!(
            parse_date_in("1582-10-04", Calendar::Auto).unwrap().0.to_string(),
            "1582-10-14"
        );
        let (bce, _) = parse_date_in("-0043-03-15", Calendar::Julian).unwrap();
        assert_eq!(i32::from(bce.year()), -43);
        assert_eq!(iso_date(-43, 3, 13), "-0043-03-13");
    }

    #[test]
    fn delta_t_follows_espenak_meeus_across_millennia() {
        // values printed in the NASA canon's table (seconds), within the stated uncertainty of each era
        for (year, want, tol) in [
            (-500.0, 17190.0, 30.0),
            (0.0, 10580.0, 20.0),
            (500.0, 5710.0, 20.0),
            (1000.0, 1570.0, 10.0),
            (1600.0, 120.0, 2.0),
            (1700.0, 9.0, 1.0),
            (1800.0, 14.0, 1.0),
            (1900.0, -3.0, 1.0),
        ] {
            let got = delta_t_polynomial(year);
            assert!((got - want).abs() <= tol, "{year}: {got} vs {want}");
        }
        // no explosion far outside the polynomial eras, and the year 100 is sane
        assert!(delta_t_polynomial(-3000.0) > 40_000.0 && delta_t_polynomial(-3000.0) < 100_000.0);
        let i = Instant::from_jd_utc(jd_from_julian(100, 1, 1.0));
        assert!(i.delta_t_s > 8_000.0 && i.delta_t_s < 11_000.0, "{}", i.delta_t_s);
    }

    #[test]
    fn forecast_follows_bulletin_a_then_the_long_term_curve() {
        // Bulletin A (19 Nov 2026): UT1-UTC = -0.1287 - 0.00049 (MJD - 61370) - (UT2-UT1); at 2027.0 (MJD 61406)
        // that is about -0.146 s plus a seasonal term of a few hundredths: ΔT ≈ 69.30 s
        assert!((delta_t_forecast(2027.0) - 69.284).abs() < 0.05);
        // one year on, the predicted drift (0.179 s a year) dominates the blend
        assert!(
            (delta_t_forecast(2028.0) - (69.284 + 0.179)).abs() < 0.1,
            "{}",
            delta_t_forecast(2028.0)
        );
        // continuous with the long-term curve at 2050 and smooth in between
        assert!((delta_t_forecast(2050.0) - delta_t_polynomial(2050.0)).abs() < 1e-9);
        let i = Instant::parse_utc("2030-06-01T00:00:00Z").unwrap();
        assert_eq!(
            i.time_model,
            "iers-bulletin-a-trend-blended-to-espenak-meeus-by-2050"
        );
        assert!(i.delta_t_s > 69.5 && i.delta_t_s < 72.0, "{}", i.delta_t_s);
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
