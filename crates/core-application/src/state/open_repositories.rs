//! Registro `RepoId -> RepoRoot` de los repositorios abiertos en esta sesion.
//!
//! Varias pestanas pueden tener varios repositorios abiertos a la vez. La interfaz
//! solo conoce el `RepoId`: un id inventado no esta aqui y da `repo_not_open`.
//!
//! El id sale de la raiz y no de un contador. Existe por un fallo real: en VS Code
//! el sidecar se reinicia tras una caida con el registro vacio, y con un contador
//! el `repo-1` del proceso nuevo podia ser otro repositorio que el del viejo, asi
//! que un panel abierto pedia el grafo de un repo y recibia el de otro. Con el id
//! derivado de la raiz, el host vuelve a abrir la misma carpeta y obtiene el mismo
//! id, y un id viejo nunca apunta a otro repositorio.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use git_rudder_core_domain::{RepoId, RepoRoot};

/// Base de FNV-1a de 64 bits.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// Primo de FNV-1a de 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Registro compartido por los casos de uso. Seguro entre hilos.
#[derive(Debug, Default)]
pub struct OpenRepositories {
    inner: Mutex<HashMap<RepoId, RepoRoot>>,
}

impl OpenRepositories {
    /// Registro vacio.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registra la raiz y devuelve su id. Abrir dos veces el mismo repositorio
    /// devuelve el mismo id, asi dos pestanas sobre un repo comparten identidad,
    /// y lo mismo pasa en otro registro (un motor reiniciado).
    pub fn register(&self, root: RepoRoot) -> RepoId {
        let id = id_for(&root);
        self.lock().entry(id.clone()).or_insert(root);
        id
    }

    /// Raiz del repositorio `id`, si esta abierto.
    #[must_use]
    pub fn get(&self, id: &RepoId) -> Option<RepoRoot> {
        self.lock().get(id).cloned()
    }

    /// Cuantos repositorios hay abiertos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// No hay ningun repositorio abierto.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Un hilo que entra en panico con el candado tomado no deja el registro a
    /// medias (cada operacion es una insercion o una lectura), asi que se recupera
    /// el candado envenenado en lugar de propagar el panico.
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<RepoId, RepoRoot>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// `repo-` y el hash FNV-1a de 64 bits de la raiz en 16 cifras hexadecimales.
///
/// Un hash explicito y no `DefaultHasher`, cuyo algoritmo puede cambiar entre
/// versiones de Rust. No revela la ruta, que el webview nunca debe ver. Dos raices
/// con el mismo hash tienen una probabilidad del orden de n^2 / 2^65: con los
/// repositorios que caben en una sesion es despreciable.
fn id_for(root: &RepoRoot) -> RepoId {
    let hash = root
        .as_path()
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
        });
    RepoId::new(format!("repo-{hash:016x}"))
}

#[cfg(test)]
mod tests;
