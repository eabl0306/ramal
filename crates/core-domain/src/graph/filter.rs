//! Filtros del grafo: ocultar o aislar ramas y quedarse con un autor o unas
//! fechas.
//!
//! Ocultar y aislar deciden desde que refs se recorre la historia, asi que los
//! resuelve git eligiendo las revisiones. Autor y fecha quitan filas de una
//! historia ya leida, y entonces los padres de las que quedan se reescriben al
//! ancestro visible mas cercano: sin eso, una fila apuntaria a un padre que no se
//! ve y su carril no se liberaria nunca. Despues se disponen los carriles de
//! nuevo con lo que queda.

use std::collections::{HashMap, HashSet};

use crate::commit::CommitRecord;
use crate::refs::RefName;
use crate::sha::Sha;

/// Desde que refs se recorre la historia.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum RefSelection {
    /// Ramas, ramas remotas, tags y HEAD: el grafo normal.
    #[default]
    All,
    /// Solo estas refs (aislar).
    Only(Vec<RefName>),
    /// Todas menos estas (ocultar).
    Excluding(Vec<RefName>),
}

/// Lo que el usuario filtro.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GraphFilter {
    /// Refs ocultas: sus commits solo salen si los alcanza otra ref.
    pub hidden: Vec<RefName>,
    /// Refs aisladas: si hay alguna, solo se ve su historia.
    pub only: Vec<RefName>,
    /// Texto que tiene que aparecer en el nombre o el correo del autor (sin
    /// distinguir mayusculas).
    pub author: Option<String>,
    /// Fecha de autor minima, en segundos unix.
    pub since: Option<i64>,
    /// Fecha de autor maxima, en segundos unix.
    pub until: Option<i64>,
}

impl GraphFilter {
    /// No filtra nada.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }

    /// Refs desde las que hay que recorrer la historia.
    #[must_use]
    pub fn selection(&self) -> RefSelection {
        if !self.only.is_empty() {
            RefSelection::Only(self.only.clone())
        } else if !self.hidden.is_empty() {
            RefSelection::Excluding(self.hidden.clone())
        } else {
            RefSelection::All
        }
    }

    /// Hay que quitar filas despues de leer: autor o fechas. Entonces se lee la
    /// historia entera, porque no se sabe cuantas filas quedaran.
    #[must_use]
    pub const fn filters_rows(&self) -> bool {
        self.author.is_some() || self.since.is_some() || self.until.is_some()
    }

    /// El commit pasa los filtros de autor y fecha.
    #[must_use]
    pub fn keeps(&self, commit: &CommitRecord) -> bool {
        let seconds = commit.authored_at.seconds();
        let author_matches = self.author.as_deref().is_none_or(|wanted| {
            let wanted = wanted.to_lowercase();
            commit.author.name.to_lowercase().contains(&wanted)
                || commit.author.email.to_lowercase().contains(&wanted)
        });
        author_matches
            && self.since.is_none_or(|since| seconds >= since)
            && self.until.is_none_or(|until| seconds <= until)
    }
}

/// Se queda con los commits que pasan el filtro y reescribe sus padres al
/// ancestro visible mas cercano por cada camino, sin repetir. Un padre que no
/// esta en la lista (historia cortada) se deja tal cual.
///
/// La entrada viene en orden topologico, asi que recorrerla al reves visita cada
/// padre antes que sus hijos.
#[must_use]
pub fn filter_history(commits: Vec<CommitRecord>, filter: &GraphFilter) -> Vec<CommitRecord> {
    let known: HashSet<Sha> = commits.iter().map(|commit| commit.node.sha).collect();
    let mut visible_ancestors: HashMap<Sha, Vec<Sha>> = HashMap::with_capacity(commits.len());
    let mut kept = Vec::new();
    for mut commit in commits.into_iter().rev() {
        let parents = rewrite_parents(&commit.node.parents, &known, &visible_ancestors);
        let sha = commit.node.sha;
        if filter.keeps(&commit) {
            visible_ancestors.insert(sha, vec![sha]);
            commit.node.parents = parents;
            kept.push(commit);
        } else {
            visible_ancestors.insert(sha, parents);
        }
    }
    kept.reverse();
    kept
}

fn rewrite_parents(
    parents: &[Sha],
    known: &HashSet<Sha>,
    visible_ancestors: &HashMap<Sha, Vec<Sha>>,
) -> Vec<Sha> {
    let mut rewritten: Vec<Sha> = Vec::with_capacity(parents.len());
    for parent in parents {
        let replacement: &[Sha] = if known.contains(parent) {
            visible_ancestors.get(parent).map_or(&[], Vec::as_slice)
        } else {
            std::slice::from_ref(parent)
        };
        for sha in replacement {
            if !rewritten.contains(sha) {
                rewritten.push(*sha);
            }
        }
    }
    rewritten
}
