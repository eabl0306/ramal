//! Binario de Ramal Desktop. Todo el arranque vive en la biblioteca para que los
//! tests lo cubran; aqui solo se llama y se traduce el resultado a un codigo de
//! salida.

// En Windows, sin esto un build de release abre ademas una consola vacia.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
    unused_crate_dependencies,
    reason = "the binary only calls the library, which is where the dependencies are used"
)]

use std::process::ExitCode;

fn main() -> ExitCode {
    match ramal_desktop::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "Ramal Desktop could not start");
            ExitCode::FAILURE
        }
    }
}
