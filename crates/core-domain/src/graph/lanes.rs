//! Asignacion de carriles con puntas fijadas y reutilizacion de huecos.
//!
//! Corrige el algoritmo del ejemplo `DIFF` de `git-rudder-data.js`, que pierde huecos
//! y no reproduce el diseno:
//!
//! - Cada hueco guarda el commit al que espera (o nada si esta libre).
//! - Un commit ocupa el primer hueco que lo espera; los demas que tambien lo
//!   esperaban (hermanos que comparten padre) quedan libres en ese momento.
//! - Si nadie lo espera, ocupa el primer hueco libre o abre uno al final.
//! - El primer padre hereda el hueco del commit; un commit raiz lo libera.
//! - Cada padre extra de un merge reutiliza la reserva que ya tenga, o si no el
//!   primer hueco libre, o uno nuevo. Nunca se duplica la reserva.
//! - Los huecos libres del final se recortan.
//!
//! Ademas se apunta, por cada padre, el hueco por el que viaja su reserva: es el
//! `travel lane` con el que se dibuja la arista sin cruzar nodos.

use crate::commit::CommitNode;
use crate::sha::Sha;

/// Resultado de la asignacion, fila a fila.
pub(super) struct LaneAssignment {
    /// Carril de cada commit.
    pub lanes: Vec<usize>,
    /// Por cada commit, el hueco por el que viaja la reserva de cada padre.
    pub travel: Vec<Vec<usize>>,
    /// Maximo de huecos ocupados a la vez.
    pub width: usize,
}

/// Recorre los commits en orden y asigna carriles.
pub(super) fn assign(commits: &[CommitNode], pinned: &[Sha]) -> LaneAssignment {
    let mut slots = Slots::pinned(pinned);
    let mut lanes = Vec::with_capacity(commits.len());
    let mut travel = Vec::with_capacity(commits.len());
    let mut width = slots.len();
    for commit in commits {
        let lane = slots.place(commit.sha);
        travel.push(slots.reserve_parents(lane, &commit.parents));
        lanes.push(lane);
        width = width.max(slots.len());
        slots.trim();
    }
    LaneAssignment {
        lanes,
        travel,
        width,
    }
}

/// Huecos de carril: cada uno espera un commit o esta libre.
struct Slots(Vec<Option<Sha>>);

impl Slots {
    fn pinned(pinned: &[Sha]) -> Self {
        Self(pinned.iter().copied().map(Some).collect())
    }

    const fn len(&self) -> usize {
        self.0.len()
    }

    fn position(&self, sha: Sha) -> Option<usize> {
        self.0.iter().position(|slot| *slot == Some(sha))
    }

    fn first_free(&self) -> usize {
        self.0
            .iter()
            .position(Option::is_none)
            .unwrap_or(self.0.len())
    }

    /// Escribe en el hueco `index`, abriendo uno nuevo si `index` es el final.
    fn set(&mut self, index: usize, value: Option<Sha>) {
        match self.0.get_mut(index) {
            Some(slot) => *slot = value,
            None => self.0.push(value),
        }
    }

    /// Carril del commit `sha`. Si varios huecos lo esperaban, se queda con el
    /// primero y libera el resto.
    fn place(&mut self, sha: Sha) -> usize {
        let Some(lane) = self.position(sha) else {
            return self.first_free();
        };
        for slot in self.0.iter_mut().skip(lane + 1) {
            if *slot == Some(sha) {
                *slot = None;
            }
        }
        lane
    }

    /// Reserva los padres del commit colocado en `lane` y devuelve el hueco por el
    /// que viaja cada reserva.
    fn reserve_parents(&mut self, lane: usize, parents: &[Sha]) -> Vec<usize> {
        let Some((first, extra)) = parents.split_first() else {
            self.set(lane, None);
            return Vec::new();
        };
        self.set(lane, Some(*first));
        let mut travel = Vec::with_capacity(parents.len());
        travel.push(lane);
        for parent in extra {
            travel.push(self.reserve_extra(*parent));
        }
        travel
    }

    /// Un padre extra de merge reutiliza su reserva si ya la tiene.
    fn reserve_extra(&mut self, parent: Sha) -> usize {
        if let Some(existing) = self.position(parent) {
            return existing;
        }
        let free = self.first_free();
        self.set(free, Some(parent));
        free
    }

    fn trim(&mut self) {
        while self.0.last().is_some_and(Option::is_none) {
            self.0.pop();
        }
    }
}
