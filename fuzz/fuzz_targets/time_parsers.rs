//! Every text parser on the time path, fed arbitrary bytes split into fields: none may panic, and a date that
//! parses must print and parse back to itself.
#![no_main]
use eshana_time::{parse_date, parse_date_in, parse_time, Calendar, FoldPolicy, Instant};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let mut f = text.splitn(5, '|');
    let (date, time, zone, cal, fold) = (
        f.next().unwrap_or(""),
        f.next().unwrap_or(""),
        f.next().unwrap_or(""),
        f.next().unwrap_or(""),
        f.next().unwrap_or(""),
    );
    let _ = parse_time(time);
    let _ = Instant::parse_utc(text);
    let fold = FoldPolicy::parse(fold).unwrap_or_default();
    for c in [Calendar::parse(cal).unwrap_or(Calendar::Gregorian), Calendar::Julian, Calendar::Auto] {
        if let Ok((d, _)) = parse_date_in(date, c) {
            let _ = d.to_string();
        }
        let _ = Instant::from_local_str_in(date, time, zone, fold, c);
        let _ = Instant::from_offset_str_in(date, time, i32::from(data.first().copied().unwrap_or(0)) * 7 - 840, c);
    }
    if let Ok(d) = parse_date(date) {
        assert_eq!(parse_date(&d.to_string()).ok(), Some(d), "a parsed date does not round-trip");
    }
});
