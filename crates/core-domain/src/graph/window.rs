//! Recorte de la disposicion a la ventana `[offset, offset + limit)` que pide la
//! interfaz, con las etiquetas de refs de cada fila.
//!
//! La interfaz virtualiza las filas y pide ventanas pequenas; el motor nunca manda
//! el historial entero por IPC.

use std::collections::HashMap;

use thiserror::Error;

use super::{GraphEdge, GraphLayout};
use crate::commit::CommitRecord;
use crate::refs::{RefLabel, RefSnapshot};

/// Maximo de filas por ventana. Mas filas por peticion no pinta antes: solo
/// alarga la respuesta.
pub const MAX_WINDOW_LIMIT: usize = 1000;

/// Ventana pedida que no se puede servir.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WindowError {
    /// `limit` fuera de `1..=1000`.
    #[error("limit must be between 1 and {max}, got {limit}")]
    LimitOutOfRange {
        /// Limite pedido.
        limit: usize,
        /// Maximo admitido.
        max: usize,
    },
    /// `offset + limit` no cabe en un entero de esta plataforma.
    #[error("offset {offset} is too large")]
    OffsetTooLarge {
        /// Desplazamiento pedido.
        offset: usize,
    },
}

/// Rango de filas validado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRange {
    offset: usize,
    limit: usize,
}

impl WindowRange {
    /// Valida la ventana.
    ///
    /// # Errors
    ///
    /// [`WindowError`] si `limit` esta fuera de `1..=1000` o el final de la
    /// ventana (mas el commit de comprobacion) no cabe en un `usize`.
    pub const fn new(offset: usize, limit: usize) -> Result<Self, WindowError> {
        if limit == 0 || limit > MAX_WINDOW_LIMIT {
            return Err(WindowError::LimitOutOfRange {
                limit,
                max: MAX_WINDOW_LIMIT,
            });
        }
        match offset.checked_add(limit + 1) {
            Some(_) => Ok(Self { offset, limit }),
            None => Err(WindowError::OffsetTooLarge { offset }),
        }
    }

    /// Primera fila de la ventana.
    #[must_use]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// Numero maximo de filas.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Fila siguiente a la ultima de la ventana.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.offset + self.limit
    }

    /// Commits que hay que leer: hasta el final de la ventana mas uno, que solo
    /// dice si hay mas.
    #[must_use]
    pub const fn commits_to_read(&self) -> usize {
        self.end() + 1
    }
}

/// Una fila de la ventana.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRow {
    /// Fila absoluta.
    pub row: usize,
    /// El commit.
    pub commit: CommitRecord,
    /// Carril del commit.
    pub lane: usize,
    /// Aristas a los padres.
    pub edges: Vec<GraphEdge>,
    /// Refs que apuntan a este commit, en orden de presentacion.
    pub refs: Vec<RefLabel>,
    /// HEAD apunta a este commit.
    pub is_head: bool,
}

/// Ventana del grafo lista para pintar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphWindow {
    /// Primera fila pedida.
    pub offset: usize,
    /// Filas de la ventana; menos que `limit` al final del historial.
    pub rows: Vec<WindowRow>,
    /// Hay commits despues de la ventana.
    pub has_more: bool,
    /// Carriles que necesita la columna del grafo hasta el final de la ventana.
    /// Crece o se mantiene al bajar, nunca encoge, asi la columna no salta.
    pub lane_count: usize,
}

impl GraphWindow {
    /// Recorta `layout`, que tiene que ser la disposicion de `commits`.
    #[must_use]
    pub fn cut(
        range: WindowRange,
        commits: Vec<CommitRecord>,
        layout: GraphLayout,
        refs: &RefSnapshot,
    ) -> Self {
        let has_more = commits.len() > range.end();
        let head = refs.head_target();
        let mut labels: HashMap<_, Vec<RefLabel>> = refs.labels_by_commit();
        let rows = commits
            .into_iter()
            .zip(layout.rows)
            .enumerate()
            .skip(range.offset())
            .take(range.limit())
            .map(|(row, (commit, layout_row))| WindowRow {
                row,
                refs: labels.remove(&commit.node.sha).unwrap_or_default(),
                is_head: head == Some(commit.node.sha),
                lane: layout_row.lane,
                edges: layout_row.edges,
                commit,
            })
            .collect();
        Self {
            offset: range.offset(),
            rows,
            has_more,
            lane_count: layout.width,
        }
    }
}
