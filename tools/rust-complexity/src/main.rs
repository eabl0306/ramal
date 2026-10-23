//! `git-rudder-rust-complexity <raiz>...`: mide la complejidad de cada funcion de Rust
//! bajo las raices y sale con 1 si alguna pasa de los limites de ADR 0005, o con
//! 2 si no puede medir (una raiz que no existe, un fichero que no se entiende).
//! La receta `just rust-complexity` le pasa las carpetas del repo.

#![allow(
    unused_crate_dependencies,
    reason = "the binary only calls the library, which is where the dependencies are used"
)]

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

use git_rudder_rust_complexity::{Report, check};

fn main() -> ExitCode {
    let roots: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    match check(&roots) {
        Ok(report) => print_report(&report),
        Err(error) => {
            let _ = writeln!(std::io::stderr(), "rust complexity: {error}");
            ExitCode::from(2)
        }
    }
}

fn print_report(report: &Report) -> ExitCode {
    let summary = format!(
        "rust complexity: {} functions in {} files",
        report.functions, report.files
    );
    if report.violations.is_empty() {
        let _ = writeln!(std::io::stdout(), "{summary}, all within the limits");
        return ExitCode::SUCCESS;
    }
    let mut stderr = std::io::stderr().lock();
    let _ = writeln!(
        stderr,
        "{summary}, {} over the limits",
        report.violations.len()
    );
    for violation in &report.violations {
        let _ = writeln!(
            stderr,
            "  {}:{}  {}: {}",
            violation.file.display(),
            violation.score.line,
            violation.score.name,
            violation.problem
        );
    }
    let _ = writeln!(
        stderr,
        "\nrust complexity: split those functions before committing."
    );
    ExitCode::FAILURE
}
