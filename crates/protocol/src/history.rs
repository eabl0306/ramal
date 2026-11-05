//! Historia mas alla del grafo: `graph/search`, `history/file`, `blame/file` y
//! `compare/refs`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::commit::{ChangedFile, CommitSummary};
use crate::diff::FileStatus;
use crate::graph::GraphFilter;

/// Parametros de `graph/search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct GraphSearchParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Lo que escribio el usuario: un texto, un autor o un sha corto.
    pub query: String,
    /// Maximo de resultados; por defecto 100, como mucho 500.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub limit: Option<u32>,
    /// El filtro del grafo, para dar la fila en ese grafo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub filter: Option<GraphFilter>,
}

/// Por que coincide un commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum MatchKind {
    /// Su sha empieza por el texto.
    Sha,
    /// Su mensaje lo contiene.
    Message,
    /// Su autor lo contiene.
    Author,
}

/// Un commit encontrado.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchHit {
    /// El commit.
    pub commit: CommitSummary,
    /// Por que coincide.
    pub matched: MatchKind,
    /// Fila absoluta en el grafo, si sale en el.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub row: Option<u32>,
}

/// Resultado de `graph/search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GraphSearchResult {
    /// Repositorio.
    pub repo_id: String,
    /// Coincidencias en orden topologico, las de sha primero.
    pub hits: Vec<SearchHit>,
}

/// Parametros de `history/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct FileHistoryParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Fichero, relativo a la raiz.
    pub path: String,
    /// Primera entrada.
    pub offset: u32,
    /// Entradas, de 1 a 1000.
    pub limit: u32,
}

/// Una entrada de la historia de un fichero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileHistoryEntry {
    /// El commit.
    pub commit: CommitSummary,
    /// Ruta del fichero en ese commit.
    pub path: String,
    /// Ruta anterior si ese commit lo renombro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub old_path: Option<String>,
    /// Que le hizo el commit.
    pub status: FileStatus,
}

/// Resultado de `history/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileHistoryPage {
    /// Entradas, la mas reciente primero.
    pub entries: Vec<FileHistoryEntry>,
    /// Hay mas.
    pub has_more: bool,
}

/// Parametros de `blame/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct BlameParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Fichero, relativo a la raiz.
    pub path: String,
    /// Sha del commit; sin el, el arbol de trabajo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub revision: Option<String>,
}

/// Un commit del blame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlameCommit {
    /// Sha; el cero para las lineas sin commit.
    pub sha: String,
    /// Nombre del autor.
    pub author_name: String,
    /// Correo del autor.
    pub author_email: String,
    /// Fecha de autor, ISO 8601.
    pub authored_at: String,
    /// Asunto.
    pub summary: String,
    /// Ruta del fichero en ese commit.
    pub path: String,
    /// Son cambios sin commit del arbol de trabajo.
    pub uncommitted: bool,
}

/// Una linea del blame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BlameLine {
    /// Numero de linea, desde 1.
    pub line: u32,
    /// Numero de linea en el commit que la introdujo.
    pub original_line: u32,
    /// Sha del commit (sus datos estan en `commits`).
    pub sha: String,
    /// Texto.
    pub text: String,
}

/// Resultado de `blame/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Blame {
    /// Lineas en orden.
    pub lines: Vec<BlameLine>,
    /// Commits, una vez cada uno.
    pub commits: Vec<BlameCommit>,
}

/// Parametros de `compare/refs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct CompareParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Base: rama, tag o sha.
    pub base: String,
    /// Cabeza: rama, tag o sha.
    pub head: String,
}

/// Resultado de `compare/refs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Comparison {
    /// Commit de la base.
    pub base: String,
    /// Commit de la cabeza.
    pub head: String,
    /// Ancestro comun; falta si no comparten historia.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub merge_base: Option<String>,
    /// Commits de la cabeza que no tiene la base.
    pub ahead: Vec<CommitSummary>,
    /// Commits de la base que no tiene la cabeza.
    pub behind: Vec<CommitSummary>,
    /// Ficheros del ancestro comun a la cabeza.
    pub files: Vec<ChangedFile>,
}
