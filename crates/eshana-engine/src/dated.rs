//! Dated Aṣṭakavarga: the natal bindus a transiting graha reads, as a step function of time.
//!
//! A transiting graha "reads" the natal Bhinnāṣṭakavarga (BAV) cell of its own table for the sign it occupies, and the
//! Sarvāṣṭakavarga (SAV) cell for the same sign. Both values are constant while the graha stays in a sign, so the
//! series is a list of `[start, end)` segments that change exactly at sign ingresses (the same ingress search as the
//! `ingresses` operation). Each segment names the cell it read, so a reader can check it against the natal tables.
//!
//! Only the seven grahas Sun–Saturn have BAV tables. A node asked for is reported as unavailable: no table is
//! invented for it.

use crate::events::{ingresses, lon_at};
use crate::methods::unavailable;
use crate::{astro, jyotish::SEVEN};
use eshana_core::{Ayanamsha, Body, Result};
use eshana_ephemeris::EphemerisProvider;
use eshana_time::Instant;
use serde_json::{json, Value};

/// Segments for one graha. `bav` is that graha's natal BAV bindus by sign, `sav` the natal SAV by sign.
///
/// # Errors
/// Provider errors.
pub fn segments(
    p: &dyn EphemerisProvider,
    body: Body,
    jd_a: f64,
    jd_b: f64,
    ayan: Ayanamsha,
    bav: &[u64; 12],
    sav: &[u64; 12],
) -> Result<Vec<Value>> {
    let ing = ingresses(p, body, jd_a, jd_b, Some(ayan))?;
    let mut sign = astro::sign_index(lon_at(p, body, jd_a, Some(ayan))?);
    let mut start = jd_a;
    let mut out = vec![];
    let mut push = |s: f64, e: f64, sign: usize, clipped_start: bool, clipped_end: bool| {
        out.push(json!({
            "start_utc": Instant::from_jd_utc(s).utc, "end_utc": Instant::from_jd_utc(e).utc, "bounds": "[start, end)",
            "start_jd_ut": s, "end_jd_ut": e,
            "transit_sign": sign, "transit_sign_name": astro::RASHIS[sign],
            "bav": bav[sign], "sav": sav[sign],
            "cells_read": {"bav": format!("{}[{sign}]", body.id()), "sav": format!("sav[{sign}]")},
            "clipped_start": clipped_start, "clipped_end": clipped_end,
        }));
    };
    let mut first = true;
    for x in &ing {
        let jd = x["jd_ut"].as_f64().unwrap_or(start);
        push(start, jd, sign, first, false);
        first = false;
        start = jd;
        sign = x["to_sign"].as_u64().map_or(sign, |v| v as usize);
    }
    push(start, jd_b, sign, first, true);
    Ok(out)
}

/// The whole result body: per graha, segments or an unavailable value.
///
/// `natal_av` is the `vedic.ashtakavarga` value of the natal chart (`None` when the natal chart had no Ascendant).
///
/// # Errors
/// Provider errors; `ESH-BODY` for a body id that is not a graha or node.
pub fn series(
    p: &dyn EphemerisProvider,
    bodies: &[String],
    jd_a: f64,
    jd_b: f64,
    ayan: Ayanamsha,
    natal_av: Option<&Value>,
) -> Result<Value> {
    let mut out = serde_json::Map::new();
    for id in bodies {
        let body = Body::parse(id)?;
        let key = body.id().to_string();
        if !SEVEN.contains(&key.as_str()) {
            out.insert(
                key,
                json!({"availability": unavailable("the Aṣṭakavarga tables cover the seven grahas Sun to Saturn; none is defined for this body", &[])}),
            );
            continue;
        }
        let Some(av) = natal_av else {
            out.insert(
                key,
                json!({"availability": unavailable("the natal Aṣṭakavarga needs the natal Ascendant (an exact birth time and a place)", &["natal.ascendant"])}),
            );
            continue;
        };
        let cells = |v: &Value| -> [u64; 12] {
            let mut a = [0u64; 12];
            for (i, x) in v.as_array().into_iter().flatten().enumerate().take(12) {
                a[i] = x.as_u64().unwrap_or(0);
            }
            a
        };
        let bav = cells(&av["bhinnashtakavarga"][&key]["bindus"]);
        let sav = cells(&av["sarvashtakavarga"]["bindus"]);
        out.insert(
            key.clone(),
            json!({"natal_bav": bav, "segments": segments(p, body, jd_a, jd_b, ayan, &bav, &sav)?}),
        );
    }
    Ok(Value::Object(out))
}
