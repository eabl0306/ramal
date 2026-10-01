//! Argumentos del sidecar: `--git-path`, `--log-level` y `--version`.
//!
//! Sin crate de argumentos: son tres opciones y el extension host es el unico que
//! lanza el binario. El ejecutable de git llega del `git.path` de VS Code; el
//! binario del motor, en cambio, nunca lo elige un ajuste de workspace.

use std::ffi::OsString;
use std::path::PathBuf;

use thiserror::Error;

/// Ayuda breve para un argumento invalido.
pub const USAGE: &str =
    "usage: ramal-vscode-engine [--git-path <path>] [--log-level <filter>] [--version]";

/// Nivel de log por defecto.
const DEFAULT_LOG_LEVEL: &str = "info";

/// Configuracion del arranque.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    /// Ejecutable de git explicito.
    pub git_path: Option<PathBuf>,
    /// Filtro de `tracing` (`info`, `debug`, `ramal_git=debug`...).
    pub log_level: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            git_path: None,
            log_level: DEFAULT_LOG_LEVEL.to_owned(),
        }
    }
}

/// Que hacer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Servir el protocolo por stdio.
    Serve(Args),
    /// Escribir la version y salir.
    Version,
}

/// Argumento que no se entiende.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArgsError {
    /// Opcion desconocida.
    #[error("unknown argument {0}")]
    Unknown(String),
    /// Opcion sin su valor.
    #[error("{0} needs a value")]
    MissingValue(&'static str),
    /// Valor que no es texto valido.
    #[error("{0} must be valid UTF-8")]
    NotUtf8(&'static str),
}

impl Args {
    /// Lee los argumentos (sin el nombre del programa).
    ///
    /// # Errors
    ///
    /// [`ArgsError`] si hay una opcion desconocida o le falta su valor.
    pub fn parse<I: IntoIterator<Item = OsString>>(arguments: I) -> Result<Command, ArgsError> {
        let mut args = Self::default();
        let mut rest = arguments.into_iter();
        while let Some(argument) = rest.next() {
            let text = argument.to_string_lossy().into_owned();
            let (name, inline) = text
                .split_once('=')
                .map_or((text.as_str(), None), |(name, value)| (name, Some(value)));
            match name {
                "--version" | "-V" => return Ok(Command::Version),
                "--git-path" => {
                    let value = take_value("--git-path", inline, &mut rest)?;
                    args.git_path = Some(PathBuf::from(value));
                }
                "--log-level" => {
                    let value = take_value("--log-level", inline, &mut rest)?;
                    args.log_level = value
                        .into_string()
                        .map_err(|_| ArgsError::NotUtf8("--log-level"))?;
                }
                _ => return Err(ArgsError::Unknown(text)),
            }
        }
        Ok(Command::Serve(args))
    }
}

/// Valor de una opcion: el de `--opcion=valor` o el siguiente argumento.
fn take_value(
    name: &'static str,
    inline: Option<&str>,
    rest: &mut impl Iterator<Item = OsString>,
) -> Result<OsString, ArgsError> {
    inline
        .map(OsString::from)
        .or_else(|| rest.next())
        .ok_or(ArgsError::MissingValue(name))
}

#[cfg(test)]
mod tests;
