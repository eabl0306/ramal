//! `refs/list`: ramas con upstream, tags y remotos con su enlace web.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Una rama, local o remota.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BranchInfo {
    /// Nombre corto (`main`, `origin/main`).
    pub name: String,
    /// Nombre completo (`refs/heads/main`).
    pub full_name: String,
    /// Commit de la punta.
    pub sha: String,
    /// Es la rama de HEAD.
    pub is_head: bool,
    /// Upstream completo (`refs/remotes/origin/main`), si tiene.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub upstream: Option<String>,
    /// Commits por delante del upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ahead: Option<u32>,
    /// Commits por detras del upstream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub behind: Option<u32>,
    /// El upstream estaba configurado y ya no existe.
    pub upstream_gone: bool,
}

/// Un tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TagInfo {
    /// Nombre corto (`v1.0`).
    pub name: String,
    /// Nombre completo.
    pub full_name: String,
    /// Commit al que apunta.
    pub sha: String,
    /// Asunto del mensaje si es anotado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub annotation: Option<String>,
}

/// Un remoto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RemoteInfo {
    /// Nombre (`origin`).
    pub name: String,
    /// URL de fetch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fetch_url: Option<String>,
    /// URL de push si es distinta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub push_url: Option<String>,
    /// Pagina web del repositorio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub web_url: Option<String>,
}

/// Resultado de `refs/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RefListing {
    /// Repositorio.
    pub repo_id: String,
    /// Ramas locales por nombre.
    pub local: Vec<BranchInfo>,
    /// Ramas remotas por nombre.
    pub remote: Vec<BranchInfo>,
    /// Tags por nombre.
    pub tags: Vec<TagInfo>,
    /// Remotos por nombre.
    pub remotes: Vec<RemoteInfo>,
}
