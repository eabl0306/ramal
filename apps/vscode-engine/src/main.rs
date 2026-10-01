//! Binario del sidecar. Todo el arranque vive en la biblioteca para que los tests
//! lo cubran; aqui solo se le pasan los argumentos.

#![allow(
    unused_crate_dependencies,
    reason = "the binary only calls the library, which is where the dependencies are used"
)]

fn main() -> std::process::ExitCode {
    ramal_vscode_engine::run(std::env::args_os())
}
