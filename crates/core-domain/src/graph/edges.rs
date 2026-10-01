//! Aristas de cada commit a sus padres, con el carril por el que viajan.
//!
//! Una arista se dibuja en tres tramos: curva en la fila del hijo desde su carril
//! (`from_lane`) al de viaje (`travel_lane`), vertical por el carril de viaje y
//! curva en la fila del padre hasta el carril del padre (`to_lane`) si es otro. El
//! carril de viaje es el hueco donde vivio la reserva, que nadie mas ocupa entre
//! las dos filas: por eso la arista no atraviesa nodos ajenos (el prototipo hacia
//! el tramo vertical en el carril final del padre y podia cruzarlos). En una
//! arista de primer padre el carril de viaje es el del hijo, asi que la forma de
//! "rama que sale" del diseno no cambia.

use std::collections::HashMap;

use super::LayoutRow;
use super::lanes::LaneAssignment;
use crate::commit::CommitNode;
use crate::sha::Sha;

/// Si la arista va al primer padre o a uno extra de un merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// Primer padre: la linea principal de la rama.
    FirstParent,
    /// Padre extra de un merge.
    Merge,
}

/// Arista de un commit a uno de sus padres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    /// Padre al que llega.
    pub parent: Sha,
    /// Fila absoluta del padre, o `None` si queda fuera de lo leido: la interfaz
    /// dibuja entonces la arista hasta el borde inferior en lugar de omitirla.
    pub parent_row: Option<usize>,
    /// Carril del hijo.
    pub from_lane: usize,
    /// Carril por el que viaja el tramo vertical.
    pub travel_lane: usize,
    /// Carril del padre; si el padre no se leyo, el de viaje.
    pub to_lane: usize,
    /// Primer padre o merge.
    pub kind: EdgeKind,
}

/// Construye las filas a partir de la asignacion de carriles.
pub(super) fn build(commits: &[CommitNode], assignment: &LaneAssignment) -> Vec<LayoutRow> {
    let mut rows_by_sha: HashMap<Sha, usize> = HashMap::with_capacity(commits.len());
    for (row, commit) in commits.iter().enumerate() {
        rows_by_sha.entry(commit.sha).or_insert(row);
    }
    let router = Router {
        rows_by_sha,
        lanes: &assignment.lanes,
    };
    commits
        .iter()
        .zip(&assignment.lanes)
        .zip(&assignment.travel)
        .map(|((commit, &lane), travel)| LayoutRow {
            lane,
            edges: router.edges(commit, lane, travel),
        })
        .collect()
}

/// Lo que hace falta para resolver el destino de cada arista.
struct Router<'a> {
    rows_by_sha: HashMap<Sha, usize>,
    lanes: &'a [usize],
}

impl Router<'_> {
    fn edges(&self, commit: &CommitNode, lane: usize, travel: &[usize]) -> Vec<GraphEdge> {
        commit
            .parents
            .iter()
            .zip(travel)
            .enumerate()
            .map(|(index, (parent, &travel_lane))| {
                let kind = if index == 0 {
                    EdgeKind::FirstParent
                } else {
                    EdgeKind::Merge
                };
                self.edge(*parent, lane, travel_lane, kind)
            })
            .collect()
    }

    fn edge(&self, parent: Sha, from_lane: usize, travel_lane: usize, kind: EdgeKind) -> GraphEdge {
        let parent_row = self.rows_by_sha.get(&parent).copied();
        let to_lane = parent_row
            .and_then(|row| self.lanes.get(row))
            .copied()
            .unwrap_or(travel_lane);
        GraphEdge {
            parent,
            parent_row,
            from_lane,
            travel_lane,
            to_lane,
            kind,
        }
    }
}
