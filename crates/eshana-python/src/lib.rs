//! PyO3 bridge: one function, `execute(request_json) -> response_json`, over the Rust engine.
//! The Python package in `python/eshana` adds typed convenience classes but never re-implements a rule.

use eshana_engine::Engine;
use pyo3::prelude::*;
use std::sync::OnceLock;

fn engine() -> &'static Engine {
    static E: OnceLock<Engine> = OnceLock::new();
    E.get_or_init(Engine::new)
}

/// Run a JSON request and return the JSON envelope (or an error envelope with a stable code).
#[pyfunction]
fn execute(py: Python<'_>, request_json: &str) -> String {
    // Release the GIL: a chart is pure computation.
    py.detach(|| engine().execute_json(request_json))
}

/// Engine version string.
#[pyfunction]
fn engine_version() -> &'static str {
    eshana_engine::ENGINE_VERSION
}

/// Run the `eshana` command line with these arguments (the first is the program name); returns the exit status.
/// Output goes to the process's stdout and stderr, exactly as the `eshana` binary writes it.
#[pyfunction]
fn cli(py: Python<'_>, args: Vec<String>) -> u8 {
    py.detach(|| eshana_cli::run(args))
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(execute, m)?)?;
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    m.add_function(wrap_pyfunction!(cli, m)?)?;
    Ok(())
}
