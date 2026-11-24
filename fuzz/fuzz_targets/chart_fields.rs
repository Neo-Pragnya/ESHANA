//! A valid chart request with its date, time, zone, calendar, coordinates and ayanamsha replaced by fuzzed values,
//! so the fuzzer reaches the time resolution, the ephemeris and the Vedic features instead of stopping at the JSON
//! parser.
#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::json;
use std::sync::OnceLock;

static ENGINE: OnceLock<eshana_engine::api::Engine> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }
    let (nums, rest) = data.split_at(16);
    let Ok(text) = std::str::from_utf8(rest) else { return };
    let mut f = text.splitn(5, '|');
    let mut field = || f.next().unwrap_or("").to_string();
    let (date, time, zone, cal, aya) = (field(), field(), field(), field(), field());
    let lat = f64::from(i16::from_le_bytes([nums[0], nums[1]])) / 300.0;
    let lon = f64::from(i16::from_le_bytes([nums[2], nums[3]])) / 150.0;
    let system = if nums[4] & 1 == 0 { "vedic" } else { "western" };
    let mut req = json!({
        "op": "chart", "system": system,
        "when": { "date": date, "time": time, "timezone": zone, "calendar": cal },
        "place": { "latitude": lat, "longitude": lon },
        "config": { "zodiac": if system == "vedic" { "sidereal" } else { "tropical" }, "ayanamsha": aya },
        "divisions": [1 + (nums[5] % 60)],
        "dasha": nums[6] & 1 == 1,
    });
    if nums[7] & 1 == 1 && system == "vedic" {
        req["features"] = json!(["all"]);
    }
    if nums[8] & 1 == 1 {
        req["when"] = json!({ "utc": date });
    }
    let out = ENGINE.get_or_init(eshana_engine::api::Engine::new).execute_json(&req.to_string());
    assert!(serde_json::from_str::<serde_json::Value>(&out).is_ok());
});
