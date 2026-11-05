//! Parsers de la salida de git. Cada formato vive junto a su parser para que no se
//! desincronicen.

pub mod blame;
pub mod changes;
pub mod commit;
pub mod config;
pub mod diff;
pub mod file_history;
pub mod inventory;
pub mod log;
pub mod operation;
pub mod quote;
pub mod refs;
pub mod status;

use thiserror::Error;

/// La salida de git no tiene la forma esperada.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unexpected git output: {0}")]
pub struct ParseError(pub String);
