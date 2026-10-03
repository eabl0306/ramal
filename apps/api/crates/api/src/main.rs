//! Binario de la API de Git Rudder.
//!
//! Aqui, y solo aqui, se eligen los adaptadores concretos: PostgreSQL para las
//! sondas, el reloj del sistema, Salvo para el transporte y figment para el
//! entorno. Los casos de uso no saben nada de esa eleccion.
//!
//! ```text
//! git-rudder-api                     arranca el servidor
//! git-rudder-api --openapi <ruta>    escribe el documento OpenAPI y sale, sin tocar la base
//! ```

mod bootstrap;
mod cli;
mod error;
mod shutdown;

use std::future::Future;
use std::io::Write;
use std::process::ExitCode;

use git_rudder_api_infrastructure::{AppConfig, TelemetryConfig, init_tracing};

use crate::cli::Command;
use crate::error::StartupError;

// Solo lo usan los tests del binario en tests/; el lint de dependencias sin usar
// mira cada target por separado.
#[cfg(test)]
use tempfile as _;

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            write_line(
                &mut std::io::stderr(),
                &format!("{error}\n\n{}", cli::USAGE),
            );
            return ExitCode::from(2);
        }
    };
    match command {
        Command::Help => {
            write_line(&mut std::io::stdout(), cli::USAGE);
            ExitCode::SUCCESS
        }
        Command::Serve => start_serving(),
        Command::WriteOpenApi(path) => {
            install_tracing(&TelemetryConfig::default());
            // El pool perezoso que cablea el router necesita un runtime alrededor.
            finish(block_on(async { bootstrap::write_openapi(&path) }))
        }
    }
}

fn start_serving() -> ExitCode {
    let config = AppConfig::from_env();
    // Sin configuracion valida tambien hay que poder contarlo en el log.
    install_tracing(&config.as_ref().map_or_else(
        |_| TelemetryConfig::default(),
        |loaded| loaded.telemetry.clone(),
    ));
    let outcome = config
        .map_err(StartupError::from)
        .and_then(|config| block_on(bootstrap::serve(config)));
    finish(outcome)
}

/// Sin subscriber no hay logs, pero tampoco motivo para no arrancar.
fn install_tracing(config: &TelemetryConfig) {
    if let Err(error) = init_tracing(config) {
        write_line(
            &mut std::io::stderr(),
            &format!("tracing disabled: {error}"),
        );
    }
}

fn block_on<F>(future: F) -> Result<(), StartupError>
where
    F: Future<Output = Result<(), StartupError>>,
{
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(StartupError::Runtime)?
        .block_on(future)
}

fn finish(outcome: Result<(), StartupError>) -> ExitCode {
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(%error, "git-rudder api failed");
            ExitCode::FAILURE
        }
    }
}

/// Escribe una linea en la consola. Si la consola esta cerrada no queda nadie a
/// quien contarselo, asi que el fallo se ignora.
fn write_line(output: &mut impl Write, text: &str) {
    let _ = writeln!(output, "{text}");
}
