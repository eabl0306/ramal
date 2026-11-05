//! Historia ya dispuesta que crece por el final: la base de la cache del grafo.
//!
//! El calculo de carriles es incremental por filas, asi que anadir commits al
//! final no cambia nada de lo ya puesto. Pedir la ventana siguiente solo lee y
//! dispone lo que falta, en vez de leer y disponer otra vez desde la fila 0.
//!
//! Una ventana sale igual tanto si la historia tiene mas filas como si no: los
//! padres mas alla del commit de comprobacion cuentan como no leidos y los
//! carriles se cuentan hasta el final de la ventana, igual que sin cache.

use std::collections::HashMap;

use super::edges::{build_range, index_rows};
use super::lanes::LaneState;
use super::window::{GraphWindow, WindowRange, WindowRow};
use crate::commit::{CommitNode, CommitRecord};
use crate::refs::{RefLabel, RefSnapshot};
use crate::sha::Sha;

/// Commits leidos en orden topologico con sus carriles.
#[derive(Debug, Clone)]
pub struct LaidOutHistory {
    commits: Vec<CommitRecord>,
    nodes: Vec<CommitNode>,
    lanes: LaneState,
    rows_by_sha: HashMap<Sha, usize>,
    complete: bool,
}

impl LaidOutHistory {
    /// Historia vacia con las puntas fijadas a la izquierda.
    #[must_use]
    pub fn new(pinned: &[Sha]) -> Self {
        Self {
            commits: Vec::new(),
            nodes: Vec::new(),
            lanes: LaneState::new(pinned),
            rows_by_sha: HashMap::new(),
            complete: false,
        }
    }

    /// Anade commits al final. `complete` dice que no hay mas despues de estos.
    pub fn extend(&mut self, commits: Vec<CommitRecord>, complete: bool) {
        let first = self.nodes.len();
        let nodes: Vec<CommitNode> = commits.iter().map(|commit| commit.node.clone()).collect();
        self.lanes.extend(&nodes);
        index_rows(&mut self.rows_by_sha, &nodes, first);
        self.nodes.extend(nodes);
        self.commits.extend(commits);
        self.complete = complete;
    }

    /// Commits leidos.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.commits.len()
    }

    /// No hay ningun commit.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }

    /// Ya no hay mas commits que leer.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// Hay bastante para servir `range` con su commit de comprobacion.
    #[must_use]
    pub const fn covers(&self, range: WindowRange) -> bool {
        self.complete || self.commits.len() > range.end()
    }

    /// Fila de un commit, si ya se leyo.
    #[must_use]
    pub fn row_of(&self, sha: Sha) -> Option<usize> {
        self.rows_by_sha.get(&sha).copied()
    }

    /// Ventana `range`. Si la historia no la cubre, sale con lo que haya.
    #[must_use]
    pub fn window(&self, range: WindowRange, refs: &RefSnapshot) -> GraphWindow {
        let horizon = self.commits.len().min(range.commits_to_read());
        let start = range.offset().min(self.commits.len());
        let end = range.end().min(self.commits.len());
        let assignment = self.lanes.assignment();
        let layout = build_range(
            &self.nodes,
            assignment,
            &self.rows_by_sha,
            start..end,
            horizon,
        );
        let head = refs.head_target();
        let mut labels: HashMap<Sha, Vec<RefLabel>> = refs.labels_by_commit();
        let rows = (start..)
            .zip(self.commits.iter().skip(start).take(end - start))
            .zip(layout)
            .map(|((row, commit), layout_row)| WindowRow {
                row,
                refs: labels.remove(&commit.node.sha).unwrap_or_default(),
                is_head: head == Some(commit.node.sha),
                lane: layout_row.lane,
                edges: layout_row.edges,
                commit: commit.clone(),
            })
            .collect();
        let lane_count = horizon
            .checked_sub(1)
            .and_then(|last| assignment.width_until.get(last))
            .copied()
            .unwrap_or(assignment.width);
        GraphWindow {
            offset: range.offset(),
            rows,
            has_more: self.commits.len() > range.end(),
            lane_count,
            snapshot: refs.fingerprint(),
        }
    }
}
