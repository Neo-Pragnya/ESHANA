//! Personal calendar: birth and event places apart, personal factors from the natal Moon, unavailable never zero,
//! DST and polar days, and deterministic CSV and ICS exports.

use eshana_engine::Engine;
use serde_json::{json, Value};

fn run(req: &Value) -> Value {
    serde_json::from_str(&Engine::new().execute_json(&req.to_string())).unwrap()
}

fn natal() -> Value {
    json!({"when": {"date": "1990-05-15", "time": "14:30", "timezone": "Asia/Kolkata"},
           "place": {"latitude": 28.61, "longitude": 77.21}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}})
}

fn cal(m: Value, format: Option<&str>) -> Value {
    run(&json!({"op": "personal_calendar", "natal": natal(), "muhurta": m, "format": format}))
}

fn mumbai(days: u32, extra: Value) -> Value {
    let mut m = json!({"place": {"latitude": 19.07, "longitude": 72.88}, "timezone": "Asia/Kolkata",
        "start_utc": "2026-11-01T00:00:00Z", "end_utc": format!("2026-11-{:02}T00:00:00Z", 1 + days),
        "profile": "personal_general", "config": {"ayanamsha": "lahiri"}});
    for (k, v) in extra.as_object().unwrap() {
        m[k] = v.clone();
    }
    m
}

#[test]
fn birth_place_and_event_place_are_kept_apart() {
    let got = cal(mumbai(10, json!({})), None);
    assert_eq!(got["kind"], "personal_calendar", "{got}");
    assert_eq!(got["input"]["birth"]["place"]["latitude_deg"], 28.61);
    assert_eq!(got["input"]["birth"]["place_role"], "birth");
    assert_eq!(got["input"]["event_place"]["latitude_deg"], 19.07);
    assert_eq!(got["input"]["event_place"]["place_role"], "event");
    // 00:00 UTC is 05:30 in Mumbai: ten UTC days touch eleven local dates
    assert_eq!(got["days"].as_array().unwrap().len(), 11);
    // tārā and candra bala are evaluated from the natal Moon on every piece
    for w in got["timeline"].as_array().unwrap() {
        assert!(w["factors"]["tara_bala"]["quality"].is_string(), "{w}");
        assert!(w["factors"]["chandra_bala"]["index"].is_u64());
    }
    assert!(!got["windows"].as_array().unwrap().is_empty());
}

#[test]
fn personal_factors_without_a_birth_are_unavailable_not_zero() {
    let got = run(
        &json!({"op": "muhurta_windows", "place": {"latitude": 19.07, "longitude": 72.88},
        "timezone": "Asia/Kolkata", "start_utc": "2026-11-01T00:00:00Z", "end_utc": "2026-11-02T00:00:00Z",
        "constraints": ["good_tara"], "config": {"ayanamsha": "lahiri"}}),
    );
    assert_eq!(got["windows"].as_array().unwrap().len(), 0);
    assert_eq!(
        got["timeline"][0]["factors"]["tara_bala"]["availability"]["missing"][0], "natal.moon",
        "{got}"
    );
}

#[test]
fn days_follow_the_event_zone_across_dst_and_polar_days_say_unavailable() {
    let london = cal(
        json!({"place": {"latitude": 51.5, "longitude": -0.12}, "timezone": "Europe/London",
               "start_utc": "2026-10-23T23:00:00Z", "end_utc": "2026-10-28T00:00:00Z", "profile": "daily_avoidances",
               "config": {"ayanamsha": "lahiri"}}),
        None,
    );
    let dates: Vec<&str> = london["days"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["date"].as_str().unwrap())
        .collect();
    assert_eq!(
        dates,
        ["2026-10-24", "2026-10-25", "2026-10-26", "2026-10-27"],
        "{london}"
    );
    assert!(london["days"][0]["sunrise_local"]
        .as_str()
        .unwrap()
        .ends_with("+01:00"));
    assert!(london["days"][3]["sunrise_local"]
        .as_str()
        .unwrap()
        .ends_with("+00:00"));
    let polar = cal(
        json!({"place": {"latitude": 78.22, "longitude": 15.65}, "timezone": "Arctic/Longyearbyen",
               "start_utc": "2026-12-20T00:00:00Z", "end_utc": "2026-12-22T00:00:00Z", "profile": "daily_avoidances",
               "config": {"ayanamsha": "lahiri"}}),
        None,
    );
    assert_eq!(
        polar["days"][0]["sunrise_local"]["status"], "unavailable",
        "{polar}"
    );
    assert_eq!(polar["windows"].as_array().unwrap().len(), 0);
}

#[test]
fn ics_and_csv_exports_are_well_formed_and_deterministic() {
    let a = cal(mumbai(10, json!({})), Some("ics"));
    let b = cal(mumbai(10, json!({})), Some("ics"));
    let ics = a["content"].as_str().unwrap();
    assert_eq!(ics, b["content"].as_str().unwrap());
    assert_eq!(a["media_type"], "text/calendar");
    assert!(ics.starts_with("BEGIN:VCALENDAR\r\n") && ics.ends_with("END:VCALENDAR\r\n"));
    for line in ics.split("\r\n") {
        assert!(line.len() <= 75, "line longer than 75 octets: {line}");
    }
    assert_eq!(
        ics.matches("BEGIN:VEVENT").count(),
        a["windows"].as_array().unwrap().len()
    );
    assert!(ics.contains("DTSTART:2026"));
    let c = cal(mumbai(3, json!({})), Some("csv"));
    let csv = c["content"].as_str().unwrap();
    assert_eq!(csv.lines().count(), c["timeline"].as_array().unwrap().len() + 1);
    assert!(csv.starts_with("start_utc,end_utc,start_local,end_local,passes,"));
    assert_eq!(
        cal(mumbai(3, json!({})), Some("pdf"))["error"]["code"],
        "ESH-INPUT"
    );
}

#[test]
fn pair_calendar_explains_both_people_and_the_event_for_each_window() {
    // A pair whose Moons leave shared windows (for some pairs none exist in two months; empty_because then says so).
    let b = json!({"when": {"date": "1991-01-10", "time": "09:15", "timezone": "Asia/Kolkata"},
                   "place": {"latitude": 19.076, "longitude": 72.8777}, "config": {"zodiac": "sidereal", "ayanamsha": "lahiri"}});
    // Eleven constraints over two Moons are strict: two months give a handful of windows.
    let mut m = mumbai(20, json!({"end_utc": "2027-01-01T00:00:00Z"}));
    m.as_object_mut().unwrap().remove("profile");
    let got = run(&json!({"op": "pair_calendar", "a": natal(), "b": b, "muhurta": m}));
    assert_eq!(got["kind"], "pair_calendar", "{got}");
    assert_eq!(got["profile"]["id"], "pair_general");
    assert_eq!(got["input"]["partner"]["place"]["latitude_deg"], 19.076);
    let ws = got["windows"].as_array().unwrap();
    assert!(!ws.is_empty());
    for w in ws {
        for side in ["a", "b"] {
            assert_eq!(w["explained"][side]["tara_bala"]["quality"], "favourable", "{w}");
            assert_eq!(w["explained"][side]["chandra_bala"]["quality"], "favourable");
        }
        assert!(w["explained"]["event"]["rahu_kalam"] == json!(false));
    }
    // the two people's factors really differ somewhere: they are two Moons, not one copied
    let tl = got["timeline"].as_array().unwrap();
    assert!(tl
        .iter()
        .any(|w| w["factors"]["tara_bala"] != w["factors"]["tara_bala_b"]));
}
