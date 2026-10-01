//! `EngineDescription` -> `EngineInfo`.

use ramal_core_application::EngineDescription;
use ramal_protocol::PROTOCOL_VERSION;
use ramal_protocol::engine as dto;

/// Descripcion del motor con la version del protocolo, que no conoce el nucleo.
#[must_use]
pub fn description_to_dto(description: &EngineDescription) -> dto::EngineInfo {
    dto::EngineInfo {
        engine_version: description.engine_version.clone(),
        protocol_version: PROTOCOL_VERSION,
        git: dto::GitInfo {
            path: description.git.path.to_string_lossy().into_owned(),
            version: description.git.version.to_string(),
            supported: description.git.is_supported(),
        },
    }
}
