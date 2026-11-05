//! Metodos del host sobre repositorios.
//!
//! Confiar, el estado de fondo de las tarjetas, buscar repositorios en carpetas
//! y cambiar el git. Todos llevan rutas absolutas o deciden que se ejecuta, asi
//! que son `host_only`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::status::{BranchState, Operation, StatusCounts};

/// Algo del repositorio que puede ejecutar codigo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum TrustFinding {
    /// Una clave de configuracion que lanza un comando.
    ConfigKey {
        /// La clave (`core.fsmonitor`).
        key: String,
        /// De donde sale (`file:.git/config`).
        origin: String,
    },
    /// Un hook que no es `.sample`.
    Hook {
        /// Nombre del fichero.
        name: String,
    },
}

/// Resultado de `repo/trust`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepoTrusted {
    /// Repositorio.
    pub repo_id: String,
    /// Ahora es de confianza.
    pub trusted: bool,
}

/// Parametros de `repos/summary`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct RepoSummaryParams {
    /// Carpeta del repositorio.
    pub path: String,
}

/// Resultado de `repos/summary`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepoSummary {
    /// Raiz del repositorio.
    pub root: String,
    /// Rama, upstream y distancia.
    pub branch: BranchState,
    /// Ficheros de cada grupo.
    pub counts: StatusCounts,
    /// Operacion a medias.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub operation: Option<Operation>,
}

/// Parametros de `repos/scan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct ScanReposParams {
    /// Carpeta en la que buscar.
    pub path: String,
    /// Profundidad maxima; por defecto 3, como mucho 8.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_depth: Option<u32>,
}

/// Resultado de `repos/scan`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ScanReposResult {
    /// Raices encontradas, ordenadas y sin repetir.
    pub roots: Vec<String>,
}

/// Parametros de `git/configure`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct ConfigureGitParams {
    /// Ejecutable de git; sin el, el primero del `PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub path: Option<String>,
}
