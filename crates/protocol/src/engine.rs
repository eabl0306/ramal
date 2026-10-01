//! `engine/info`: version del motor, del protocolo y de git.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Parametros de `engine/info`: ninguno (`{}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
#[ts(export)]
pub struct EngineInfoParams {}

/// El git que usa el motor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GitInfo {
    /// Ruta del ejecutable.
    pub path: String,
    /// Version detectada, `2.45.1`.
    pub version: String,
    /// La version llega a la minima (2.34).
    pub supported: bool,
}

/// Resultado de `engine/info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EngineInfo {
    /// Version del motor.
    pub engine_version: String,
    /// `PROTOCOL_VERSION` del motor.
    pub protocol_version: u32,
    /// El git que va a usar.
    pub git: GitInfo,
}
