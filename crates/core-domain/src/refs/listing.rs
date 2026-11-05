//! Ramas, tags y remotos tal como los ensenan la barra lateral, los arboles de VS
//! Code y la barra de estado.
//!
//! El grafo solo necesita a que commit apunta cada ref ([`super::RefSnapshot`]);
//! la barra lateral quiere ademas el upstream de cada rama con su distancia y el
//! asunto de los tags anotados.

use super::RefName;
use crate::remote::Remote;
use crate::sha::Sha;
use crate::status::AheadBehind;

/// Upstream de una rama local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracking {
    /// Nombre completo del upstream (`refs/remotes/origin/main`).
    pub upstream: String,
    /// Delante y detras; `None` si el upstream ya no existe (`gone`).
    pub distance: Option<AheadBehind>,
}

impl Tracking {
    /// El upstream estaba configurado y la rama remota ya no existe.
    #[must_use]
    pub const fn is_gone(&self) -> bool {
        self.distance.is_none()
    }
}

/// Una rama, local o remota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchInfo {
    /// Nombre completo.
    pub name: RefName,
    /// Commit de la punta.
    pub target: Sha,
    /// Es la rama de HEAD.
    pub is_head: bool,
    /// Upstream (solo ramas locales que lo tienen).
    pub tracking: Option<Tracking>,
}

/// Un tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagInfo {
    /// Nombre completo.
    pub name: RefName,
    /// Commit al que apunta, ya pelado si es anotado.
    pub target: Sha,
    /// Asunto del mensaje si el tag es anotado.
    pub annotation: Option<String>,
}

/// Todo lo que lista la barra lateral.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RefListing {
    /// Ramas locales por nombre.
    pub local: Vec<BranchInfo>,
    /// Ramas remotas por nombre.
    pub remote: Vec<BranchInfo>,
    /// Tags por nombre.
    pub tags: Vec<TagInfo>,
    /// Remotos por nombre.
    pub remotes: Vec<Remote>,
}

impl RefListing {
    /// Ordena cada grupo por nombre, para no depender del orden de git.
    #[must_use]
    pub fn sorted(mut self) -> Self {
        self.local.sort_by(|left, right| left.name.cmp(&right.name));
        self.remote
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.tags.sort_by(|left, right| left.name.cmp(&right.name));
        self.remotes
            .sort_by(|left, right| left.name.cmp(&right.name));
        self
    }
}
