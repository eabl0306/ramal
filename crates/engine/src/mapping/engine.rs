//! `EngineDescription` -> `EngineInfo`.

use git_rudder_core_application::{EngineDescription, GitVersionInfo};
use git_rudder_protocol::PROTOCOL_VERSION;
use git_rudder_protocol::engine as dto;

/// Descripcion del motor con la version del protocolo, que no conoce el nucleo.
#[must_use]
pub fn description_to_dto(description: &EngineDescription) -> dto::EngineInfo {
    dto::EngineInfo {
        engine_version: description.engine_version.clone(),
        protocol_version: PROTOCOL_VERSION,
        git: git_to_dto(&description.git),
    }
}

/// El git que va a usar el motor.
#[must_use]
pub fn git_to_dto(git: &GitVersionInfo) -> dto::GitInfo {
    dto::GitInfo {
        path: git.path.to_string_lossy().into_owned(),
        version: git.version.to_string(),
        supported: git.is_supported(),
    }
}
