//! Synthetic charts: Vedic factors derived from positions a person placed by hand (a teaching playground, a "what if
//! Saturn were here" question). Nothing here calls an ephemeris: the positions are the input, and the result says so.
//!
//! Every derived factor lists the inputs it depends on, so a playground can show what changes when one graha moves.
//! Factors that need a moment or a place (daśās, pañcāṅga, kāla bala, transits, house cusps other than whole-sign
//! from a given Ascendant) are returned as unavailable, never approximated.

use crate::jyotish::{self, sign_of, VChart, NINE};
use crate::methods::unavailable;
use crate::vedic;
use eshana_core::{norm360, Body, EshanaError, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// One placed body.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placed {
    /// Body id (`sun` … `saturn`, `mean_node`/`true_node` for Rāhu, `uranus` …).
    pub body: String,
    /// Sidereal longitude in degrees.
    pub longitude_deg: f64,
    /// Daily motion in degrees (negative = retrograde); default 1 (direct). Only its sign is used.
    pub speed_deg_per_day: Option<f64>,
}

/// Derive what can be derived from placed positions.
///
/// # Errors
/// `ESH-INPUT` for an unknown body, a repeated body or a non-finite longitude.
pub fn derive(placed: &[Placed], ascendant: Option<f64>) -> Result<Value> {
    if placed.is_empty() {
        return Err(EshanaError::Input("place at least one body".into()));
    }
    let mut pos: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for p in placed {
        let b = Body::parse(&p.body)?;
        if !p.longitude_deg.is_finite() {
            return Err(EshanaError::Input(format!(
                "{}: longitude must be finite",
                p.body
            )));
        }
        if pos
            .insert(
                b.id().to_string(),
                (norm360(p.longitude_deg), p.speed_deg_per_day.unwrap_or(1.0)),
            )
            .is_some()
        {
            return Err(EshanaError::Input(format!("{} is placed twice", b.id())));
        }
        if matches!(b, Body::MeanNode | Body::TrueNode) {
            pos.insert(
                "south_node".into(),
                (
                    norm360(p.longitude_deg + 180.0),
                    p.speed_deg_per_day.unwrap_or(-0.05),
                ),
            );
        }
    }
    if let Some(a) = ascendant {
        if !a.is_finite() {
            return Err(EshanaError::Input("ascendant_deg must be finite".into()));
        }
    }
    let asc = ascendant.map(norm360);
    let vc = VChart::from_positions(&pos, asc, None);
    let all: Vec<String> = pos.keys().cloned().collect();
    let mut bodies = serde_json::Map::new();
    for (id, (l, sp)) in &pos {
        let nk = vedic::nakshatra_of(*l);
        let g = match id.as_str() {
            "mean_node" | "true_node" => "rahu",
            "south_node" => "ketu",
            x => x,
        };
        let dignity = if NINE.contains(&g) {
            json!(jyotish::dignity(g, *l))
        } else {
            Value::Null
        };
        bodies.insert(
            id.clone(),
            json!({"longitude_deg": l, "sign": sign_of(*l), "sign_name": crate::astro::RASHIS[sign_of(*l)],
            "nakshatra": nk.name, "pada": nk.pada, "nakshatra_lord": nk.lord, "retrograde": *sp < 0.0,
            "dignity": dignity, "kp": jyotish::kp_lords(*l),
            "whole_sign_house": asc.map(|a| (sign_of(*l) + 12 - sign_of(a)) % 12 + 1)}),
        );
    }
    let mut attrs = vec![];
    let mut push = |id: &str, v: Value, from: Vec<String>| {
        attrs.push(json!({"id": id, "value": v, "derived_from": from}))
    };
    let mut with_asc = all.iter().map(|b| format!("position.{b}")).collect::<Vec<_>>();
    if asc.is_some() {
        with_asc.push("ascendant".into());
    }
    let every: Vec<String> = all.iter().map(|b| format!("position.{b}")).collect();
    push("vedic.drishti", jyotish::graha_drishti(&vc), every.clone());
    push("vedic.rashi_drishti", jyotish::rashi_drishti(&vc), every.clone());
    push(
        "vedic.relationships",
        jyotish::panchadha_maitri(&vc),
        every.clone(),
    );
    push(
        "vedic.jaimini.chara_karakas",
        jyotish::chara_karakas(&vc),
        every.clone(),
    );
    push("vedic.yogas", jyotish::yogas(&vc), with_asc.clone());
    match jyotish::ashtakavarga(&vc) {
        Some(v) => push("vedic.ashtakavarga", v, with_asc.clone()),
        None => push(
            "vedic.ashtakavarga",
            json!({"availability": unavailable("the Aṣṭakavarga needs the Ascendant and the seven grahas", &["ascendant"])}),
            with_asc.clone(),
        ),
    }
    let mut not_derivable = serde_json::Map::new();
    for (k, why, need) in [
        ("dashas", "a daśā needs the moment of birth", "birth time"),
        (
            "panchanga",
            "the pañcāṅga needs a moment and a place",
            "moment, place",
        ),
        (
            "shadbala",
            "kāla and ceṣṭā bala need a moment and real motion",
            "moment, place",
        ),
        ("transits", "transits need a moment", "moment"),
        (
            "placidus_cusps",
            "quadrant house cusps need a moment and a place",
            "moment, place",
        ),
    ] {
        not_derivable.insert(k.into(), unavailable(why, &[need]));
    }
    Ok(
        json!({"bodies": bodies, "ascendant": asc.map(|a| json!({"longitude_deg": a, "sign": sign_of(a)})),
              "derived": attrs, "not_derivable": not_derivable}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(body: &str, l: f64) -> Placed {
        Placed {
            body: body.into(),
            longitude_deg: l,
            speed_deg_per_day: None,
        }
    }

    #[test]
    fn derived_factors_list_their_inputs_and_time_factors_are_unavailable() {
        let v = derive(&[p("sun", 10.0), p("moon", 100.0), p("saturn", 190.0)], Some(5.0)).unwrap();
        assert_eq!(v["bodies"]["sun"]["dignity"], "exalted");
        assert_eq!(v["bodies"]["saturn"]["whole_sign_house"], 7);
        let d = v["derived"].as_array().unwrap();
        assert!(d
            .iter()
            .all(|x| !x["derived_from"].as_array().unwrap().is_empty()));
        assert_eq!(v["not_derivable"]["dashas"]["status"], "unavailable");
        // three grahas only: no Aṣṭakavarga, said so
        let av = d.iter().find(|x| x["id"] == "vedic.ashtakavarga").unwrap();
        assert_eq!(av["value"]["availability"]["status"], "unavailable");
    }

    #[test]
    fn bad_input_is_refused() {
        assert_eq!(derive(&[], None).unwrap_err().code(), "ESH-INPUT");
        assert_eq!(
            derive(&[p("sun", 1.0), p("sun", 2.0)], None).unwrap_err().code(),
            "ESH-INPUT"
        );
        assert_eq!(derive(&[p("vulcan", 1.0)], None).unwrap_err().code(), "ESH-INPUT");
    }
}
