//! Registro `RepoId -> RepoRoot` de los repositorios abiertos en esta sesion.
//!
//! Varias pestanas pueden tener varios repositorios abiertos a la vez. La interfaz
//! solo conoce el `RepoId`: un id inventado no esta aqui y da `repo_not_open`.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use ramal_core_domain::{RepoId, RepoRoot};

/// Registro compartido por los casos de uso. Seguro entre hilos.
#[derive(Debug, Default)]
pub struct OpenRepositories {
    inner: Mutex<Registry>,
}

#[derive(Debug, Default)]
struct Registry {
    by_id: HashMap<RepoId, RepoRoot>,
    opened: u64,
}

impl OpenRepositories {
    /// Registro vacio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra la raiz y devuelve su id. Abrir dos veces el mismo repositorio
    /// devuelve el mismo id, asi dos pestanas sobre un repo comparten identidad.
    pub fn register(&self, root: RepoRoot) -> RepoId {
        let mut registry = self.lock();
        if let Some(existing) = registry
            .by_id
            .iter()
            .find_map(|(id, open)| (*open == root).then(|| id.clone()))
        {
            return existing;
        }
        registry.opened += 1;
        let id = RepoId::new(format!("repo-{}", registry.opened));
        registry.by_id.insert(id.clone(), root);
        id
    }

    /// Raiz del repositorio `id`, si esta abierto.
    #[must_use]
    pub fn get(&self, id: &RepoId) -> Option<RepoRoot> {
        self.lock().by_id.get(id).cloned()
    }

    /// Cuantos repositorios hay abiertos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().by_id.len()
    }

    /// No hay ningun repositorio abierto.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Un hilo que entra en panico con el candado tomado no deja el registro a
    /// medias (cada operacion es una insercion o una lectura), asi que se recupera
    /// el candado envenenado en lugar de propagar el panico.
    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
