//! ESHANA in the browser (ADR-013). The studio loads this module and calls the same entry point as every other
//! binding, [`eshana_engine::Engine::execute_json`]: a JSON request in, a JSON envelope out. No rule lives here.
//!
//! ABI (no bindings generator, so the module has no imports and runs in any WebAssembly host):
//! - `eshana_alloc(len) -> ptr`: reserve `len` bytes for the request;
//! - `eshana_execute(ptr, len) -> out`: run the UTF-8 request at `ptr`; `out` points to a little-endian `u32`
//!   length followed by that many bytes of UTF-8 JSON;
//! - `eshana_free(ptr, len)`: release a block (the request with its length, or `out` with `4 + length`).
//!
//! The `spk` provider needs a kernel file, which a browser module cannot open: it answers `ESH-PROVIDER`. The
//! analytic provider needs nothing and works offline.

use eshana_engine::Engine;
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(Engine::new)
}

/// Run a request held in a Rust string; the safe core of [`eshana_execute`].
#[must_use]
pub fn execute_str(request: &str) -> String {
    engine().execute_json(request)
}

/// Reserve `len` bytes and return their address.
#[no_mangle]
pub extern "C" fn eshana_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Release a block from [`eshana_alloc`] or [`eshana_execute`].
///
/// # Safety
/// `ptr` and `len` must describe a block this module handed out and that has not been freed.
#[no_mangle]
pub unsafe extern "C" fn eshana_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
    }
}

/// Run the UTF-8 JSON request at `ptr..ptr+len`; return `[u32 length LE][JSON bytes]`.
///
/// # Safety
/// `ptr..ptr+len` must be a block written by the caller after [`eshana_alloc`].
#[no_mangle]
pub unsafe extern "C" fn eshana_execute(ptr: *const u8, len: usize) -> *mut u8 {
    let bytes = std::slice::from_raw_parts(ptr, len);
    let out = match std::str::from_utf8(bytes) {
        Ok(s) => execute_str(s),
        Err(_) => r#"{"schema_version":"0","kind":"error","error":{"code":"ESH-INPUT","message":"request is not UTF-8"}}"#.to_string(),
    };
    let n = out.len() as u32;
    let mut buf = Vec::with_capacity(4 + out.len());
    buf.extend_from_slice(&n.to_le_bytes());
    buf.extend_from_slice(out.as_bytes());
    let p = buf.as_mut_ptr();
    std::mem::forget(buf);
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_through_the_abi() {
        let req = br#"{"op":"methods"}"#;
        unsafe {
            let p = eshana_alloc(req.len());
            std::ptr::copy_nonoverlapping(req.as_ptr(), p, req.len());
            let out = eshana_execute(p, req.len());
            let n = u32::from_le_bytes([*out, *out.add(1), *out.add(2), *out.add(3)]) as usize;
            let s = std::str::from_utf8(std::slice::from_raw_parts(out.add(4), n)).unwrap();
            assert!(s.contains("\"kind\":\"methods\""), "{s}");
            assert_eq!(s, execute_str(r#"{"op":"methods"}"#));
            eshana_free(out, 4 + n);
            eshana_free(p, req.len());
        }
    }
}
