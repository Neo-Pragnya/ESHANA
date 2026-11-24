//! The engine's one entry point on arbitrary text: it must answer with a JSON envelope (an error envelope for
//! nonsense), never panic.
#![no_main]
use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;

static ENGINE: OnceLock<eshana_engine::api::Engine> = OnceLock::new();

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else { return };
    let out = ENGINE.get_or_init(eshana_engine::api::Engine::new).execute_json(text);
    assert!(serde_json::from_str::<serde_json::Value>(&out).is_ok(), "the engine answered with text that is not JSON");
});
