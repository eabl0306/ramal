//! Argumentos de la linea de ordenes.
//!
//! Son dos y no merecen una dependencia: se leen a mano.

use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

/// Ayuda que se imprime con `--help` o ante un argumento desconocido.
pub const USAGE: &str = "usage:
  git-rudder-api                    start the server (configuration from GIT_RUDDER__* variables)
  git-rudder-api --openapi <path>   write the OpenAPI document to <path> and exit
  git-rudder-api --help             show this help";

/// Lo que se pide al binario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Arrancar el servidor.
    Serve,
    /// Escribir el documento OpenAPI en la ruta dada.
    WriteOpenApi(PathBuf),
    /// Mostrar la ayuda.
    Help,
}

/// Argumentos que no se entienden.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CliError {
    /// `--openapi` sin ruta detras.
    #[error("--openapi needs a path")]
    MissingPath,
    /// Un argumento que no existe, o uno de mas.
    #[error("unexpected argument {0:?}")]
    Unexpected(String),
}

/// Lee los argumentos, sin el nombre del programa.
///
/// # Errors
///
/// Devuelve [`CliError`] si falta la ruta de `--openapi` o sobra algo.
pub fn parse(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, CliError> {
    let mut arguments = arguments.into_iter();
    let Some(first) = arguments.next() else {
        return Ok(Command::Serve);
    };
    let command = match first.to_str() {
        Some("--openapi") => {
            let path = arguments.next().ok_or(CliError::MissingPath)?;
            Command::WriteOpenApi(PathBuf::from(path))
        }
        Some("--help" | "-h") => Command::Help,
        _ => return Err(unexpected(&first)),
    };
    arguments
        .next()
        .map_or(Ok(command), |extra| Err(unexpected(&extra)))
}

fn unexpected(argument: &OsString) -> CliError {
    CliError::Unexpected(argument.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests;
