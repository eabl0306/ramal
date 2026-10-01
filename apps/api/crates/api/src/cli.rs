//! Argumentos de la linea de ordenes.
//!
//! Son dos y no merecen una dependencia: se leen a mano.

use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

/// Ayuda que se imprime con `--help` o ante un argumento desconocido.
pub const USAGE: &str = "usage:
  ramal-api                    start the server (configuration from RAMAL__* variables)
  ramal-api --openapi <path>   write the OpenAPI document to <path> and exit
  ramal-api --help             show this help";

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
mod tests {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use super::{CliError, Command, parse};

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn no_arguments_start_the_server() {
        assert_eq!(parse(args(&[])), Ok(Command::Serve));
    }

    #[test]
    fn openapi_takes_the_next_argument_as_the_path() {
        assert_eq!(
            parse(args(&["--openapi", "apps/api/openapi.json"])),
            Ok(Command::WriteOpenApi(PathBuf::from(
                "apps/api/openapi.json"
            )))
        );
    }

    #[test]
    fn openapi_without_a_path_is_rejected() {
        assert_eq!(parse(args(&["--openapi"])), Err(CliError::MissingPath));
    }

    #[test]
    fn help_has_a_short_form() {
        assert_eq!(parse(args(&["--help"])), Ok(Command::Help));
        assert_eq!(parse(args(&["-h"])), Ok(Command::Help));
    }

    #[test]
    fn unknown_and_extra_arguments_are_rejected() {
        assert_eq!(
            parse(args(&["--port", "1"])),
            Err(CliError::Unexpected("--port".to_owned()))
        );
        assert_eq!(
            parse(args(&["--help", "now"])),
            Err(CliError::Unexpected("now".to_owned()))
        );
    }

    #[test]
    fn errors_explain_what_is_wrong() {
        assert_eq!(CliError::MissingPath.to_string(), "--openapi needs a path");
        assert!(
            CliError::Unexpected("x".to_owned())
                .to_string()
                .contains("\"x\"")
        );
    }
}
