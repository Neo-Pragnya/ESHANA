//! Chakras: Sarvatobhadra (with vedha), Kota and Saṅghaṭṭa.
//!
//! Each chakra is one attribute holding its full layout, so a reader (or the studio) only draws it:
//! `vedic.chakra.sarvatobhadra` (the 9 × 9 grid, natal and transit placements, the three vedha lines of every graha
//! and the vedha received by the Janma nakshatra, the lagna nakshatra, the Moon and lagna rāśis, the birth tithi
//! group and the birth weekday), `vedic.chakra.kota` (the 28 nakshatras in four zones from the Janma nakshatra, Kota
//! swāmī and pāla, which grahas are inside the fort and which are entering or leaving it) and
//! `vedic.chakra.sanghatta` (the ring of 28 nakshatras, the sapta-śalākā vedha pairs and what each graha pierces).
//! Tables and conventions live in `rulesets/chakras.yaml`; this module only applies them. Longitudes are sidereal.

use crate::analysis::{AnalysisCtx, Derived};
use crate::events;
use crate::jyotish::{lord_of, sign_of};
use crate::vedic::NAKSHATRA_SPAN;
use eshana_core::{angular_diff, norm360, Body, Result};
use eshana_time::Instant;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Rule id with version for the three chakra attributes.
pub const RULE: &str = "vedic.chakras@0.1";
/// Raw YAML of the rule pack.
pub const SOURCE: &str = include_str!("../rulesets/chakras.yaml");

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const ORDER: [&str; 9] = [
    "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn", "rahu", "ketu",
];
const LINES: [&str; 3] = ["front", "left", "right"];
const KN: u8 = b"n"[0];
const KR: u8 = b"r"[0];
const KV: u8 = b"v"[0];
const KL: u8 = b"l"[0];
const KT: u8 = b"t"[0];

#[derive(Debug, Deserialize)]
struct Pack {
    abhijit: [f64; 2],
    nakshatras: Vec<[String; 3]>,
    rashis: Vec<[String; 3]>,
    vowels: Vec<[String; 2]>,
    letters: Vec<[String; 2]>,
    tithi_groups: Vec<TithiGroup>,
    sarvatobhadra: Vec<Vec<String>>,
    mean_motion: BTreeMap<String, f64>,
    kota: KotaPack,
    sanghatta_pairs: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
struct TithiGroup {
    name: String,
    dev: String,
    tr: String,
    tithis: Vec<u32>,
    weekdays: Vec<usize>,
}

#[derive(Debug, Deserialize)]
struct KotaPack {
    zones: Vec<String>,
    directions: Vec<String>,
    size: BTreeMap<String, f64>,
}

fn pack() -> &'static Pack {
    static P: OnceLock<Pack> = OnceLock::new();
    // A built-in pack that does not parse is a build defect, caught by the tests below before release.
    P.get_or_init(|| {
        serde_norway::from_str(SOURCE).unwrap_or_else(|e| panic!("rule pack chakras.yaml is invalid: {e}"))
    })
}

/// Index (0–27) of a sidereal longitude in the 28-nakshatra scheme with Abhijit (index 21), which takes
/// 276°40′–280°53′20″ (the last pada of Uttara Āṣāḍhā and the first fifteenth of Śravaṇa).
#[must_use]
pub fn nakshatra28(lon: f64) -> usize {
    let l = norm360(lon);
    let [a, b] = pack().abhijit;
    if (a..b).contains(&l) {
        return 21;
    }
    let i = ((l / NAKSHATRA_SPAN).floor() as usize).min(26);
    if i >= 21 {
        i + 1
    } else {
        i
    }
}

/// Name of a nakshatra in the 28 scheme.
#[must_use]
pub fn nakshatra28_name(i: usize) -> &'static str {
    &pack().nakshatras[i % 28][0]
}

/// A body placed in a chakra: id, sidereal longitude and speed (degrees per day).
#[derive(Debug, Clone)]
pub struct Point {
    /// `sun` … `ketu`, or `lagna`.
    pub id: String,
    /// Sidereal longitude.
    pub lon: f64,
    /// Speed in degrees per day (0 for the lagna).
    pub speed: f64,
}

impl Point {
    /// A point from an id, longitude and speed.
    #[must_use]
    pub fn new(id: &str, lon: f64, speed: f64) -> Self {
        Point {
            id: id.into(),
            lon,
            speed,
        }
    }
}

/// Motion class and the vedha line it emphasises: vakrī (retrograde) looks right, śīghra (faster than its mean
/// motion) looks left, sama (otherwise) looks to the front. Rāhu and Ketu are always vakrī; the Sun and Moon always
/// look to the front.
#[must_use]
pub fn motion(id: &str, speed: f64) -> (&'static str, &'static str) {
    match id {
        "rahu" | "ketu" => ("vakri", "right"),
        "sun" | "moon" | "lagna" => ("sama", "front"),
        _ if speed < 0.0 => ("vakri", "right"),
        _ if pack().mean_motion.get(id).is_some_and(|m| speed > *m) => ("shighra", "left"),
        _ => ("sama", "front"),
    }
}

/// Natural nature: Jupiter, Venus and Mercury benefic; the Moon benefic in the bright half (śukla pakṣa); the Sun,
/// Mars, Saturn, Rāhu and Ketu malefic. Mercury is counted benefic without checking its company.
#[must_use]
pub fn nature(id: &str, shukla: bool) -> &'static str {
    match id {
        "jupiter" | "venus" | "mercury" => "benefic",
        "moon" if shukla => "benefic",
        "lagna" => "neutral",
        _ => "malefic",
    }
}

/// Birth facts the Sarvatobhadra marks: tithi (1–30) and weekday (0 = Sunday), when known.
#[derive(Debug, Clone, Copy, Default)]
pub struct Birth {
    /// Tithi, 1–30.
    pub tithi: Option<u32>,
    /// Weekday, 0 = Sunday.
    pub weekday: Option<usize>,
}

struct Placed {
    id: String,
    lon: f64,
    nak: usize,
    sign: usize,
    retro: bool,
    motion: &'static str,
    emphasis: &'static str,
    nature: &'static str,
}

fn place(points: &[Point], shukla: bool) -> Vec<Placed> {
    points
        .iter()
        .map(|p| {
            let (m, e) = motion(&p.id, p.speed);
            Placed {
                id: p.id.clone(),
                lon: norm360(p.lon),
                nak: nakshatra28(p.lon),
                sign: sign_of(p.lon),
                retro: m == "vakri",
                motion: m,
                emphasis: e,
                nature: nature(&p.id, shukla),
            }
        })
        .collect()
}

// ── Sarvatobhadra ────────────────────────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq)]
struct Code {
    kind: u8,
    idx: usize,
}

fn grid() -> Vec<Vec<Code>> {
    pack()
        .sarvatobhadra
        .iter()
        .map(|row| {
            row.iter()
                .map(|c| Code {
                    kind: c.as_bytes()[0],
                    idx: c[1..].parse().unwrap_or(0),
                })
                .collect()
        })
        .collect()
}

fn find(g: &[Vec<Code>], kind: u8, idx: usize) -> (usize, usize) {
    for (r, row) in g.iter().enumerate() {
        for (c, x) in row.iter().enumerate() {
            if x.kind == kind && x.idx == idx {
                return (r, c);
            }
        }
    }
    (4, 4)
}

/// The three vedha lines from a border cell of the 9 × 9 grid, `[front, left, right]`, each the cells it passes
/// through in order (the start cell excluded). Left and right are the hands of an observer standing in the cell and
/// facing the centre.
#[must_use]
pub fn sbc_lines(r: usize, c: usize) -> [Vec<(usize, usize)>; 3] {
    let (dr, dc): (i32, i32) = if r == 0 {
        (1, 0)
    } else if r == 8 {
        (-1, 0)
    } else if c == 0 {
        (0, 1)
    } else {
        (0, -1)
    };
    // on screen (rows grow downwards) the right hand of an observer facing (dr, dc) points to (dc, -dr)
    let walk = |sr: i32, sc: i32| {
        let mut out = vec![];
        let (mut y, mut x) = (r as i32 + sr, c as i32 + sc);
        while (0..9).contains(&y) && (0..9).contains(&x) {
            out.push((y as usize, x as usize));
            y += sr;
            x += sc;
        }
        out
    };
    [walk(dr, dc), walk(dr - dc, dc + dr), walk(dr + dc, dc - dr)]
}

fn tithi_group(tithi: u32) -> usize {
    let t = (tithi.max(1) - 1) % 15 + 1;
    pack()
        .tithi_groups
        .iter()
        .position(|g| g.tithis.contains(&t))
        .unwrap_or(4)
}

fn weekday_group(w: usize) -> usize {
    pack()
        .tithi_groups
        .iter()
        .position(|g| g.weekdays.contains(&w))
        .unwrap_or(0)
}

fn cell_json(code: Code) -> Value {
    let p = pack();
    let i = code.idx;
    match code.kind {
        KN => {
            json!({"kind": "nakshatra", "index": i, "name": p.nakshatras[i][0], "dev": p.nakshatras[i][1], "tr": p.nakshatras[i][2]})
        }
        KR => {
            json!({"kind": "rashi", "index": i, "name": p.rashis[i][0], "dev": p.rashis[i][1], "tr": p.rashis[i][2]})
        }
        KV => {
            json!({"kind": "vowel", "index": i, "name": p.vowels[i][1], "dev": p.vowels[i][0], "tr": p.vowels[i][1]})
        }
        KL => {
            json!({"kind": "letter", "index": i, "name": p.letters[i][1], "dev": p.letters[i][0], "tr": p.letters[i][1]})
        }
        _ => {
            let t = &p.tithi_groups[i];
            json!({"kind": "tithi", "index": i, "name": t.name, "dev": t.dev, "tr": t.tr, "tithis": t.tithis,
                   "weekdays": t.weekdays.iter().map(|w| WEEKDAYS[*w % 7]).collect::<Vec<_>>()})
        }
    }
}

fn graha_rows(set: &str, list: &[Placed], g: &[Vec<Code>], pierced: &mut [Vec<Vec<Value>>]) -> Vec<Value> {
    let p = pack();
    list.iter()
        .map(|x| {
            let cell = find(g, KN, x.nak);
            let rcell = find(g, KR, x.sign);
            let lines = if x.id == "lagna" {
                Value::Null
            } else {
                let mut m = serde_json::Map::new();
                for (k, line) in sbc_lines(cell.0, cell.1).iter().enumerate() {
                    for &(r, c) in line {
                        pierced[r][c].push(json!({"set": set, "graha": x.id, "line": LINES[k], "nature": x.nature,
                                                  "emphasised": LINES[k] == x.emphasis}));
                    }
                    m.insert(LINES[k].into(), json!(line.iter().map(|&(r, c)| [r, c]).collect::<Vec<_>>()));
                }
                Value::Object(m)
            };
            json!({"id": x.id, "longitude_deg": x.lon, "nakshatra": p.nakshatras[x.nak][0], "nakshatra_index": x.nak,
                   "rashi": p.rashis[x.sign][0], "cell": [cell.0, cell.1], "rashi_cell": [rcell.0, rcell.1],
                   "retrograde": x.retro, "motion": x.motion, "emphasis": x.emphasis, "nature": x.nature, "lines": lines})
        })
        .collect()
}

fn occupants(list: &[Placed], code: Code) -> Vec<String> {
    list.iter()
        .filter(|x| (code.kind == KN && x.nak == code.idx) || (code.kind == KR && x.sign == code.idx))
        .map(|x| x.id.clone())
        .collect()
}

const SBC_CONVENTION: &str = "9 × 9 grid, east at the top, south right, west bottom, north left; 28 nakshatras \
with Abhijit (276°40′–280°53′20″); every graha pierces three lines from its nakshatra cell: front (straight across), \
left and right diagonals, left and right being the hands of an observer in the cell facing the centre; the line a \
graha emphasises follows its motion: vakrī (retrograde) right, śīghra (faster than mean) left, otherwise front; Rāhu \
and Ketu always vakrī, the Sun and Moon always front; natural benefics Jupiter, Venus, Mercury and the waxing Moon";

/// Build the Sarvatobhadra chakra from natal points (grahas and the lagna), optional transit points and the birth
/// tithi and weekday.
#[must_use]
pub fn sarvatobhadra(
    natal: &[Point],
    transit: Option<&[Point]>,
    shukla: bool,
    birth: Birth,
    transit_utc: Option<&str>,
) -> Value {
    let g = grid();
    let nat = place(natal, shukla);
    let tra = transit.map(|t| place(t, shukla));
    let mut pierced = vec![vec![Vec::<Value>::new(); 9]; 9];
    let natal_rows = graha_rows("natal", &nat, &g, &mut pierced);
    let transit_rows = tra.as_ref().map(|t| graha_rows("transit", t, &g, &mut pierced));
    let tg = birth.tithi.map(tithi_group);
    let wg = birth.weekday.map(weekday_group);
    let mut cells = vec![];
    for (r, row) in g.iter().enumerate() {
        for (c, code) in row.iter().enumerate() {
            let mut v = cell_json(*code);
            v["row"] = json!(r);
            v["col"] = json!(c);
            v["natal"] = json!(occupants(&nat, *code));
            v["transit"] = tra.as_ref().map_or(Value::Null, |t| json!(occupants(t, *code)));
            v["pierced_by"] = json!(pierced[r][c]);
            if code.kind == KT {
                v["birth_tithi"] = json!(tg == Some(code.idx));
                v["birth_weekday"] = json!(wg == Some(code.idx));
            }
            cells.push(v);
        }
    }
    let mut received = vec![];
    let mut target = |key: &str, label: &str, code: Code| {
        let (r, c) = find(&g, code.kind, code.idx);
        let by = &pierced[r][c];
        let of = |set: &str| by.iter().filter(|v| v["set"] == set).cloned().collect::<Vec<_>>();
        let count = |set: &str, nat: &str| {
            by.iter()
                .filter(|v| v["set"] == set && v["nature"] == nat)
                .count()
        };
        let cell = &cells[r * 9 + c];
        received.push(
            json!({"target": key, "label": label, "cell": [r, c], "kind": cell["kind"], "name": cell["name"],
            "dev": cell["dev"], "natal": of("natal"), "transit": tra.as_ref().map(|_| of("transit")),
            "occupants": cell["natal"], "transit_occupants": cell["transit"],
            "natal_malefics": count("natal", "malefic"), "natal_benefics": count("natal", "benefic"),
            "transit_malefics": tra.as_ref().map(|_| count("transit", "malefic")),
            "transit_benefics": tra.as_ref().map(|_| count("transit", "benefic"))}),
        );
    };
    if let Some(m) = nat.iter().find(|x| x.id == "moon") {
        target(
            "janma_nakshatra",
            "Janma nakshatra (Moon)",
            Code { kind: KN, idx: m.nak },
        );
        target(
            "janma_rashi",
            "Janma rāśi (Moon sign)",
            Code {
                kind: KR,
                idx: m.sign,
            },
        );
    }
    if let Some(l) = nat.iter().find(|x| x.id == "lagna") {
        target(
            "lagna_nakshatra",
            "Lagna nakshatra",
            Code { kind: KN, idx: l.nak },
        );
        target(
            "lagna_rashi",
            "Lagna rāśi",
            Code {
                kind: KR,
                idx: l.sign,
            },
        );
    }
    if let Some(t) = tg {
        target("birth_tithi", "Birth tithi group", Code { kind: KT, idx: t });
    }
    if let Some(w) = wg {
        target("birth_weekday", "Birth weekday", Code { kind: KT, idx: w });
    }
    json!({
        "convention": SBC_CONVENTION,
        "orientation": {"top": "east", "right": "south", "bottom": "west", "left": "north"},
        "size": 9,
        "cells": cells,
        "grahas": {"natal": natal_rows, "transit": transit_rows},
        "vedha_received": received,
        "birth": {"tithi": birth.tithi, "tithi_group": tg.map(|i| &pack().tithi_groups[i].name),
                  "weekday": birth.weekday.map(|w| WEEKDAYS[w % 7])},
        "transit_utc": transit_utc,
    })
}

// ── Kota ─────────────────────────────────────────────────────────────────────────────────────────────────────────
const KOTA_CONVENTION: &str = "28 nakshatras with Abhijit from the Janma nakshatra (the Moon nakshatra), seven per \
direction (east, south, west, north; east at the top); each direction enters from its corner through bāhya, prakāra \
and madhya to the stambha and leaves along its axis through madhya, prakāra and bāhya; a direct graha in an entering \
place is entering the fort, a retrograde one (Rāhu and Ketu always) is leaving it, and the other way round in the \
leaving places; inside = stambha or madhya; Kota swāmī = lord of the Moon sign; Kota pāla = lord of the Moon navāṃśa \
(pada); natural benefics Jupiter, Venus, Mercury and the waxing Moon";

fn kota_path(p: usize) -> &'static str {
    match p {
        0..=2 => "entering",
        3 => "core",
        _ => "leaving",
    }
}

/// Place of the k-th nakshatra (0 = Janma) in the Kota: zone, direction, path and drawing coordinates in a square
/// from -1 to 1 (bāhya), y growing downwards, east at the top.
#[must_use]
pub fn kota_place(k: usize) -> (&'static str, &'static str, &'static str, f64, f64) {
    let kp = &pack().kota;
    let (grp, p) = ((k % 28) / 7, k % 7);
    let zone = kp.zones[p].as_str();
    let s = kp.size.get(zone).copied().unwrap_or(1.0);
    let (mut x, mut y) = if p < 3 { (-s, -s) } else { (0.0, -s) };
    for _ in 0..grp {
        (x, y) = (-y, x);
    }
    (zone, kp.directions[grp].as_str(), kota_path(p), x, y)
}

fn kota_rows(list: &[Placed], janma: usize) -> Vec<Value> {
    list.iter()
        .filter(|x| x.id != "lagna")
        .map(|x| {
            let k = (x.nak + 28 - janma) % 28;
            let (zone, dir, path, _, _) = kota_place(k);
            let moving = match (path, x.retro) {
                ("entering", true) => "leaving",
                ("leaving", true) => "entering",
                (p, _) => p,
            };
            json!({"id": x.id, "nakshatra": pack().nakshatras[x.nak][0], "nakshatra_index": x.nak, "position": k + 1,
                   "zone": zone, "direction": dir, "place": path, "path": moving, "retrograde": x.retro,
                   "nature": x.nature})
        })
        .collect()
}

fn kota_summary(rows: &[Value]) -> Value {
    let pick = |nat: &str, f: &dyn Fn(&Value) -> bool| {
        rows.iter()
            .filter(|r| r["nature"] == nat && f(r))
            .map(|r| r["id"].clone())
            .collect::<Vec<_>>()
    };
    let inside = |r: &Value| r["zone"] == "stambha" || r["zone"] == "madhya";
    json!({
        "malefics_inside": pick("malefic", &inside), "benefics_inside": pick("benefic", &inside),
        "malefics_entering": pick("malefic", &|r: &Value| r["path"] == "entering"),
        "benefics_entering": pick("benefic", &|r: &Value| r["path"] == "entering"),
        "malefics_leaving": pick("malefic", &|r: &Value| r["path"] == "leaving"),
        "benefics_leaving": pick("benefic", &|r: &Value| r["path"] == "leaving"),
        "in_stambha": rows.iter().filter(|r| r["zone"] == "stambha").map(|r| r["id"].clone()).collect::<Vec<_>>(),
    })
}

/// Build the Kota chakra from natal points (the Moon is required) and optional transit points.
#[must_use]
pub fn kota(
    natal: &[Point],
    transit: Option<&[Point]>,
    shukla: bool,
    transit_utc: Option<&str>,
) -> Option<Value> {
    let p = pack();
    let moon = natal.iter().find(|x| x.id == "moon")?;
    let janma = nakshatra28(moon.lon);
    let layout: Vec<Value> = (0..28)
        .map(|k| {
            let n = (janma + k) % 28;
            let (zone, dir, path, x, y) = kota_place(k);
            json!({"position": k + 1, "index": n, "name": p.nakshatras[n][0], "dev": p.nakshatras[n][1],
                   "tr": p.nakshatras[n][2], "zone": zone, "direction": dir, "place": path, "x": x, "y": y})
        })
        .collect();
    let swami = lord_of(sign_of(moon.lon));
    let pada_sign = ((norm360(moon.lon) / (30.0 / 9.0)).floor() as usize) % 12;
    let pala = lord_of(pada_sign);
    let nat = kota_rows(&place(natal, shukla), janma);
    let tra = transit.map(|t| kota_rows(&place(t, shukla), janma));
    let find_row = |rows: &[Value], id: &str| rows.iter().find(|r| r["id"] == id).cloned();
    Some(json!({
        "convention": KOTA_CONVENTION,
        "janma": {"index": janma, "name": p.nakshatras[janma][0], "dev": p.nakshatras[janma][1], "tr": p.nakshatras[janma][2]},
        "kota_swami": {"graha": swami, "rule": "lord of the Moon sign", "natal": find_row(&nat, swami),
                       "transit": tra.as_ref().and_then(|t| find_row(t, swami))},
        "kota_pala": {"graha": pala, "rule": "lord of the Moon navāṃśa (pada)", "navamsha": p.rashis[pada_sign][0],
                      "natal": find_row(&nat, pala), "transit": tra.as_ref().and_then(|t| find_row(t, pala))},
        "zones": ["stambha", "madhya", "prakara", "bahya"],
        "zone_size": p.kota.size,
        "layout": layout,
        "natal": nat,
        "natal_summary": kota_summary(&nat),
        "transit": tra,
        "transit_summary": tra.as_ref().map(|t| kota_summary(t)),
        "transit_utc": transit_utc,
    }))
}

// ── Saṅghaṭṭa ────────────────────────────────────────────────────────────────────────────────────────────────────
const SANGHATTA_CONVENTION: &str = "ring of 28 nakshatras with Abhijit, Aśvinī first, counterclockwise from the top \
(angle_deg is the centre of each place, counterclockwise from the top); vedha pairs are the sapta-śalākā pairs (the \
two nakshatras at the ends of one of the seven vertical or seven horizontal lines pierce each other); a graha pierces \
the partner of the nakshatra it occupies";

/// The Saṅghaṭṭa vedha pairs as nakshatra indices (28 scheme).
#[must_use]
pub fn sanghatta_pairs() -> Vec<(usize, usize)> {
    let p = pack();
    let idx = |n: &str| p.nakshatras.iter().position(|x| x[0] == n).unwrap_or(0);
    p.sanghatta_pairs.iter().map(|[a, b]| (idx(a), idx(b))).collect()
}

fn partners(pairs: &[(usize, usize)], i: usize) -> Vec<usize> {
    pairs
        .iter()
        .filter_map(|&(a, b)| {
            if a == i {
                Some(b)
            } else if b == i {
                Some(a)
            } else {
                None
            }
        })
        .collect()
}

/// Build the Saṅghaṭṭa chakra from natal points and optional transit points.
#[must_use]
pub fn sanghatta(
    natal: &[Point],
    transit: Option<&[Point]>,
    shukla: bool,
    transit_utc: Option<&str>,
) -> Value {
    let p = pack();
    let pairs = sanghatta_pairs();
    let ring: Vec<Value> = (0..28)
        .map(|i| {
            json!({"index": i, "name": p.nakshatras[i][0], "dev": p.nakshatras[i][1], "tr": p.nakshatras[i][2],
                   "angle_deg": (i as f64 + 0.5) * 360.0 / 28.0,
                   "partners": partners(&pairs, i).iter().map(|j| &p.nakshatras[*j][0]).collect::<Vec<_>>()})
        })
        .collect();
    let rows = |list: &[Placed]| -> Vec<Value> {
        list.iter()
            .map(|x| {
                let pierces = if x.id == "lagna" {
                    vec![]
                } else {
                    partners(&pairs, x.nak)
                };
                json!({"id": x.id, "nakshatra": p.nakshatras[x.nak][0], "nakshatra_index": x.nak,
                       "pierces": pierces.iter().map(|j| &p.nakshatras[*j][0]).collect::<Vec<_>>(),
                       "pierces_index": pierces, "retrograde": x.retro, "nature": x.nature})
            })
            .collect()
    };
    let nat = place(natal, shukla);
    let tra = transit.map(|t| place(t, shukla));
    let nat_rows = rows(&nat);
    let tra_rows = tra.as_ref().map(|t| rows(t));
    let mut received = vec![];
    for i in 0..28 {
        let by = |rs: &[Value]| {
            rs.iter()
                .filter(|r| {
                    r["pierces_index"]
                        .as_array()
                        .is_some_and(|a| a.iter().any(|v| v.as_u64() == Some(i as u64)))
                })
                .map(|r| r["id"].clone())
                .collect::<Vec<_>>()
        };
        let n = by(&nat_rows);
        let t = tra_rows.as_ref().map(|r| by(r)).unwrap_or_default();
        if !n.is_empty() || !t.is_empty() {
            received.push(json!({"index": i, "name": nakshatra28_name(i), "natal": n, "transit": t}));
        }
    }
    let moon = nat.iter().find(|x| x.id == "moon").map(|x| x.nak);
    let lagna = nat.iter().find(|x| x.id == "lagna").map(|x| x.nak);
    json!({
        "convention": SANGHATTA_CONVENTION,
        "ring": ring,
        "pairs": pairs.iter().map(|(a, b)| [p.nakshatras[*a][0].as_str(), p.nakshatras[*b][0].as_str()]).collect::<Vec<_>>(),
        "pairs_index": pairs.iter().map(|(a, b)| [a, b]).collect::<Vec<_>>(),
        "natal": nat_rows,
        "transit": tra_rows,
        "received": received,
        "janma_index": moon,
        "lagna_index": lagna,
        "transit_utc": transit_utc,
    })
}

// ── chart entry point ────────────────────────────────────────────────────────────────────────────────────────────
fn natal_points(x: &AnalysisCtx) -> Vec<Point> {
    let mut out: Vec<Point> = ORDER
        .iter()
        .filter_map(|g| {
            x.vc.lon
                .get(*g)
                .map(|l| Point::new(g, *l, x.vc.speed.get(*g).copied().unwrap_or(0.0)))
        })
        .collect();
    if let Some(a) = x.vc.asc {
        out.push(Point::new("lagna", a, 0.0));
    }
    out
}

/// The node model the chart used (true or mean), recovered by comparing the chart Rāhu with both at birth.
fn node_body(x: &AnalysisCtx) -> Body {
    let Some(r) = x.vc.lon.get("rahu") else {
        return Body::TrueNode;
    };
    let at = |b| events::lon_at(x.provider, b, x.inst.jd_ut, Some(x.ayanamsha)).ok();
    match (at(Body::TrueNode), at(Body::MeanNode)) {
        (Some(t), Some(m)) if angular_diff(*r, m).abs() < angular_diff(*r, t).abs() => Body::MeanNode,
        _ => Body::TrueNode,
    }
}

fn transit_points(x: &AnalysisCtx, jd: f64) -> Result<Vec<Point>> {
    let bodies = [
        ("sun", Body::Sun),
        ("moon", Body::Moon),
        ("mars", Body::Mars),
        ("mercury", Body::Mercury),
        ("jupiter", Body::Jupiter),
        ("venus", Body::Venus),
        ("saturn", Body::Saturn),
        ("rahu", node_body(x)),
    ];
    let mut out = vec![];
    for (id, b) in bodies {
        let l = events::lon_at(x.provider, b, jd, Some(x.ayanamsha))?;
        let a = events::lon_at(x.provider, b, jd - 0.5, Some(x.ayanamsha))?;
        let z = events::lon_at(x.provider, b, jd + 0.5, Some(x.ayanamsha))?;
        out.push(Point::new(id, l, angular_diff(z, a)));
    }
    let (rl, rs) = (out[7].lon, out[7].speed);
    out.push(Point::new("ketu", norm360(rl + 180.0), rs));
    Ok(out)
}

/// Compute the three chakra attributes for a chart. Transit placements are added when the request has a reference
/// instant; if the ephemeris cannot reach it, the chakras keep their natal layer and say why.
///
/// # Errors
/// None at present (ephemeris errors for the transit layer are reported in the value).
pub fn compute(x: &AnalysisCtx) -> Result<Vec<Derived>> {
    let natal = natal_points(x);
    let (Some(moon), Some(sun)) = (x.vc.lon.get("moon"), x.vc.lon.get("sun")) else {
        return Ok(vec![]);
    };
    let shukla = norm360(moon - sun) < 180.0;
    let (transit, transit_utc, note) = match x.reference_jd {
        Some(jd) => match transit_points(x, jd) {
            Ok(t) => (Some(t), Some(Instant::from_jd_utc(jd).utc), None),
            Err(e) => (None, None, Some(format!("transit layer omitted: {e}"))),
        },
        None => (None, None, None),
    };
    let tithi = x
        .panchanga
        .and_then(|p| p["tithi"]["index"].as_u64())
        .map(|v| v as u32)
        .or_else(|| Some((norm360(moon - sun) / 12.0).floor() as u32 + 1));
    let weekday = x
        .panchanga
        .and_then(|p| p["vara"]["index"].as_u64())
        .map(|v| v as usize);
    let birth = Birth { tithi, weekday };
    let tu = transit_utc.as_deref();
    let t = transit.as_deref();
    let mut from = vec!["positions".to_string(), "houses".to_string()];
    if x.panchanga.is_some() {
        from.push("panchanga".into());
    }
    let mut out = vec![];
    let mut push = |id: &str, mut v: Value| {
        if let Some(n) = &note {
            v["note"] = json!(n);
        }
        out.push(Derived {
            id: id.into(),
            rule: RULE.into(),
            value: v,
            from: from.clone(),
        });
    };
    push(
        "vedic.chakra.sarvatobhadra",
        sarvatobhadra(&natal, t, shukla, birth, tu),
    );
    if let Some(k) = kota(&natal, t, shukla, tu) {
        push("vedic.chakra.kota", k);
    }
    push("vedic.chakra.sanghatta", sanghatta(&natal, t, shukla, tu));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds() -> BTreeMap<u8, usize> {
        let mut m = BTreeMap::new();
        for row in grid() {
            for c in row {
                *m.entry(c.kind).or_insert(0) += 1;
            }
        }
        m
    }

    #[test]
    fn grid_has_every_element_once() {
        let g = grid();
        assert_eq!(g.len(), 9);
        assert!(g.iter().all(|r| r.len() == 9));
        let k = kinds();
        assert_eq!((k[&KN], k[&KV], k[&KL], k[&KR], k[&KT]), (28, 16, 20, 12, 5));
        for (kind, n) in [(KN, 28), (KV, 16), (KL, 20), (KR, 12), (KT, 5)] {
            let mut seen: Vec<usize> = g
                .iter()
                .flatten()
                .filter(|c| c.kind == kind)
                .map(|c| c.idx)
                .collect();
            seen.sort_unstable();
            assert_eq!(seen, (0..n).collect::<Vec<_>>());
        }
        // nakshatras run round the border: Krittika first after the top-left corner, Abhijit on the bottom row
        assert_eq!(find(&g, KN, 2), (0, 1));
        assert_eq!(find(&g, KN, 21), (8, 2));
        assert_eq!(find(&g, KT, 4), (4, 4));
    }

    #[test]
    fn abhijit_takes_its_span() {
        assert_eq!(nakshatra28(0.0), 0);
        assert_eq!(nakshatra28(276.0), 20);
        assert_eq!(nakshatra28(277.0), 21);
        assert_eq!(nakshatra28(281.0), 22);
        assert_eq!(nakshatra28(359.9), 27);
        assert_eq!(nakshatra28_name(21), "Abhijit");
    }

    #[test]
    fn krittika_vedha_worked_example() {
        // the classical example: Krittika has vedha with Shravana (front), Bharani and Vishakha (the diagonals)
        let g = grid();
        let [front, left, right] = sbc_lines(0, 1);
        let name = |rc: (usize, usize)| {
            let c = g[rc.0][rc.1];
            if c.kind == KN {
                nakshatra28_name(c.idx)
            } else {
                ""
            }
        };
        assert_eq!(name(*front.last().unwrap()), "Shravana");
        assert_eq!(name(*right.last().unwrap()), "Bharani");
        assert_eq!(name(*left.last().unwrap()), "Vishakha");
        assert_eq!(front.len(), 8);
    }

    #[test]
    fn motion_rule() {
        assert_eq!(motion("rahu", 0.05), ("vakri", "right"));
        assert_eq!(motion("saturn", -0.02), ("vakri", "right"));
        assert_eq!(motion("mars", 0.7), ("shighra", "left"));
        assert_eq!(motion("mars", 0.3), ("sama", "front"));
        assert_eq!(motion("moon", 15.0), ("sama", "front"));
        assert_eq!(nature("moon", false), "malefic");
        assert_eq!(nature("mercury", false), "benefic");
    }

    #[test]
    fn vedha_received_by_the_janma_nakshatra() {
        // Moon in Krittika, Saturn in Shravana: Saturn pierces Krittika along its front line
        let natal = [
            Point::new("moon", 30.0, 13.0),
            Point::new("sun", 200.0, 1.0),
            Point::new("saturn", 287.0, 0.05),
        ];
        let v = sarvatobhadra(
            &natal,
            None,
            false,
            Birth {
                tithi: Some(17),
                weekday: Some(1),
            },
            None,
        );
        assert_eq!(v["cells"].as_array().unwrap().len(), 81);
        let rec = v["vedha_received"].as_array().unwrap();
        let janma = rec.iter().find(|r| r["target"] == "janma_nakshatra").unwrap();
        assert_eq!(janma["name"], "Krittika");
        assert!(janma["natal"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["graha"] == "saturn" && r["line"] == "front"));
        assert_eq!(v["birth"]["tithi_group"], "Bhadra");
        let wd = rec.iter().find(|r| r["target"] == "birth_weekday").unwrap();
        assert_eq!(wd["name"], "Bhadra");
    }

    #[test]
    fn kota_zones_and_paths() {
        let mut count = BTreeMap::new();
        for k in 0..28 {
            *count.entry(kota_place(k).0).or_insert(0) += 1;
        }
        assert_eq!(
            (
                count["stambha"],
                count["madhya"],
                count["prakara"],
                count["bahya"]
            ),
            (4, 8, 8, 8)
        );
        assert_eq!(kota_place(0), ("bahya", "east", "entering", -1.0, -1.0));
        assert_eq!(kota_place(3), ("stambha", "east", "core", 0.0, -0.25));
        // the second direction (south) starts at the top-right corner and leaves along the right-hand axis
        assert_eq!(kota_place(7), ("bahya", "south", "entering", 1.0, -1.0));
        assert_eq!(kota_place(13), ("bahya", "south", "leaving", 1.0, 0.0));
        // Moon in Punarvasu: Janma is Punarvasu; Mars in Magha (4th) is at the stambha; Saturn retrograde in Ashlesha
        // (3rd, an entering place) is leaving
        let natal = [
            Point::new("moon", 88.3, 13.0),
            Point::new("sun", 255.0, 1.0),
            Point::new("mars", 125.0, 0.6),
            Point::new("saturn", 110.0, -0.03),
        ];
        let v = kota(&natal, None, false, None).unwrap();
        assert_eq!(v["janma"]["name"], "Punarvasu");
        assert_eq!(v["kota_swami"]["graha"], "mercury");
        let row = |id: &str| {
            v["natal"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["id"] == id)
                .unwrap()
                .clone()
        };
        assert_eq!(row("mars")["zone"], "stambha");
        assert_eq!(row("saturn")["place"], "entering");
        assert_eq!(row("saturn")["path"], "leaving");
    }

    #[test]
    fn sanghatta_pairs_cover_all_28_once() {
        let pairs = sanghatta_pairs();
        assert_eq!(pairs.len(), 14);
        let mut all: Vec<usize> = pairs.iter().flat_map(|(a, b)| [*a, *b]).collect();
        all.sort_unstable();
        assert_eq!(all, (0..28).collect::<Vec<_>>());
        let pts = [
            Point::new("mars", 21.0, 0.2),
            Point::new("sun", 0.0, 1.0),
            Point::new("moon", 100.0, 13.0),
        ];
        let v = sanghatta(&pts, None, true, None);
        assert_eq!(v["natal"][0]["nakshatra"], "Bharani");
        assert_eq!(v["natal"][0]["pierces"], json!(["Magha"]));
    }
}
