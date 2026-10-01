//! Parsers de la salida de git. Cada formato vive junto a su parser para que no se
//! desincronicen.

pub mod log;
pub mod refs;

use thiserror::Error;

/// La salida de git no tiene la forma esperada.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unexpected git output: {0}")]
pub struct ParseError(pub String);
