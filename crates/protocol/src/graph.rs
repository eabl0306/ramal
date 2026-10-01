//! `graph/window`: una ventana del grafo con carriles y aristas ya resueltos.
//!
//! El motor calcula la disposicion y la interfaz solo pinta: en un repositorio de
//! cientos de miles de commits no se manda el historial entero por IPC.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Parametros de `graph/window`. `limit` va de 1 a 1000.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct GraphWindowParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Primera fila.
    pub offset: u32,
    /// Numero de filas, de 1 a 1000.
    pub limit: u32,
}

/// Clase de arista.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum EdgeKind {
    /// Al primer padre.
    FirstParent,
    /// A un padre extra de un merge.
    Merge,
}

/// Arista de un commit a uno de sus padres, en tres tramos: curva en la fila del
/// hijo de `fromLane` a `travelLane`, vertical por `travelLane` y curva en la fila
/// del padre hasta `toLane`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GraphEdge {
    /// Sha del padre.
    pub parent_sha: String,
    /// Fila absoluta del padre, o `null` si queda fuera de lo leido: se dibuja
    /// hasta el borde inferior.
    pub parent_row: Option<u32>,
    /// Carril del hijo.
    pub from_lane: u32,
    /// Carril del tramo vertical.
    pub travel_lane: u32,
    /// Carril del padre.
    pub to_lane: u32,
    /// Primer padre o merge.
    pub kind: EdgeKind,
}

/// Clase de ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RefKind {
    /// Rama local.
    LocalBranch,
    /// Rama remota.
    RemoteBranch,
    /// Tag.
    Tag,
}

/// Etiqueta de una ref en una fila.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RefLabel {
    /// Nombre corto: `main`, `origin/main`, `v1.0`.
    pub name: String,
    /// Nombre completo: `refs/heads/main`.
    pub full_name: String,
    /// Clase de ref.
    pub kind: RefKind,
    /// Es la rama a la que apunta HEAD.
    pub is_head: bool,
}

/// Una fila del grafo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GraphRow {
    /// Sha completo.
    pub sha: String,
    /// Sha corto que dio git.
    pub short_sha: String,
    /// Shas de los padres en orden.
    pub parents: Vec<String>,
    /// Carril del commit.
    pub lane: u32,
    /// Aristas a los padres, en el orden de los padres.
    pub edges: Vec<GraphEdge>,
    /// Primera linea del mensaje.
    pub summary: String,
    /// Nombre del autor.
    pub author_name: String,
    /// Correo del autor.
    pub author_email: String,
    /// Fecha de autor en ISO 8601 con desplazamiento.
    pub authored_at: String,
    /// Refs que apuntan a este commit, la rama de HEAD primero.
    pub refs: Vec<RefLabel>,
    /// HEAD apunta a este commit.
    pub is_head: bool,
}

/// Resultado de `graph/window`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct GraphWindow {
    /// Repositorio.
    pub repo_id: String,
    /// Primera fila pedida.
    pub offset: u32,
    /// Filas; menos que `limit` al final del historial.
    pub rows: Vec<GraphRow>,
    /// Hay commits despues de la ventana.
    pub has_more: bool,
    /// Carriles que necesita la columna del grafo hasta el final de la ventana.
    pub lane_count: u32,
}
