//! Lecturas de un repositorio abierto que no necesitan mas que su puerto: estado,
//! diffs, detalle de commit, refs, historia de un fichero, blame, comparaciones,
//! stashes, submodulos y conflictos.
//!
//! Cada caso de uso busca el repositorio en el registro (un `repoId` inventado da
//! `repo_not_open`), le pasa al puerto la raiz y la confianza, y valida lo que el
//! puerto no puede: paginas, revisiones que existen y stashes que lo son.

mod history;
mod inventory;
mod working_tree;

pub use history::{BlameFile, CompareRefs, FileHistory, FileHistoryPage, ReadCommitDetail};
pub use inventory::{ListRefs, ListStashes, ListSubmodules, ReadConflict, ShowStash};
pub use working_tree::{ReadFileDiff, ReadStatus};

/// `Debug` de un caso de uso: solo el nombre, sus puertos no lo tienen.
macro_rules! debug_use_case {
    ($($name:ident),+ $(,)?) => {
        $(
            impl std::fmt::Debug for $name {
                fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    formatter.debug_struct(stringify!($name)).finish_non_exhaustive()
                }
            }
        )+
    };
}

pub(crate) use debug_use_case;

#[cfg(test)]
mod tests;
