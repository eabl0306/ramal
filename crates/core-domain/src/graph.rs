//! Disposicion del grafo de commits: carril de cada commit y aristas hacia sus
//! padres, ya enrutadas para no atravesar nodos ajenos.
//!
//! La entrada tiene que venir en orden topologico (hijos antes que padres), que es
//! lo que da `git log --topo-order`; asegurarlo es cosa del lector de git. El
//! calculo es incremental por filas: el carril de la fila `n` solo depende de las
//! filas `0..=n`, asi que pedir una ventana mas abajo no cambia los carriles de las
//! filas ya pintadas. Los colores de carril (`carril % 6`) son presentacion y no
//! viven aqui.

mod edges;
mod lanes;
mod window;

pub use edges::{EdgeKind, GraphEdge};
pub use window::{GraphWindow, MAX_WINDOW_LIMIT, WindowError, WindowRange, WindowRow};

use crate::commit::{CommitNode, CommitRecord};
use crate::refs::RefSnapshot;
use crate::sha::Sha;

/// Carril y aristas de una fila.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutRow {
    /// Carril del commit, empezando en 0 por la izquierda.
    pub lane: usize,
    /// Una arista por padre, en el orden de los padres.
    pub edges: Vec<GraphEdge>,
}

/// Disposicion completa de una lista de commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphLayout {
    /// Una fila por commit, en el mismo orden que la entrada.
    pub rows: Vec<LayoutRow>,
    /// Maximo de carriles ocupados a la vez, puntas fijadas incluidas.
    pub width: usize,
}

/// Calcula la disposicion. `pinned` son las puntas que se reservan desde el
/// principio, en orden de carril: la rama por defecto queda siempre a la
/// izquierda aunque haya ramas con commits mas recientes.
#[must_use]
pub fn layout(commits: &[CommitNode], pinned: &[Sha]) -> GraphLayout {
    let assignment = lanes::assign(commits, pinned);
    let rows = edges::build(commits, &assignment);
    GraphLayout {
        rows,
        width: assignment.width,
    }
}

/// Puntas fijadas del primer corte: solo la rama por defecto, si existe.
#[must_use]
pub fn pinned_tips(refs: &RefSnapshot) -> Vec<Sha> {
    refs.default_branch_tip().into_iter().collect()
}

/// Del log y las refs a la ventana que ve la interfaz: dispone todos los commits
/// leidos (desde la fila 0, para que los carriles no dependan de la ventana) y
/// recorta `range`.
#[must_use]
pub fn graph_window(
    range: WindowRange,
    commits: Vec<CommitRecord>,
    refs: &RefSnapshot,
) -> GraphWindow {
    let nodes: Vec<CommitNode> = commits.iter().map(|record| record.node.clone()).collect();
    let layout = layout(&nodes, &pinned_tips(refs));
    GraphWindow::cut(range, commits, layout, refs)
}
