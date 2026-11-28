//! The `eshana` binary: a thin wrapper over [`eshana_cli::run`].

use std::process::ExitCode;

fn main() -> ExitCode {
    ExitCode::from(eshana_cli::run(std::env::args_os()))
}
