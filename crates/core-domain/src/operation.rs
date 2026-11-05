//! Operaciones a medias: merge, rebase, cherry-pick o revert.
//!
//! La interfaz tiene que poder retomar un conflicto o un rebase parado, tambien
//! despues de reiniciar, asi que el motor lo detecta cada vez que lee el estado
//! a partir de los ficheros que git deja en su carpeta (`MERGE_HEAD`,
//! `rebase-merge/`, `CHERRY_PICK_HEAD`...).

use crate::sha::Sha;

/// Progreso de un rebase: paso actual y total ("3 de 7").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RebaseProgress {
    /// Paso en curso, empezando en 1.
    pub step: u32,
    /// Pasos en total.
    pub total: u32,
}

/// Un rebase en curso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseState {
    /// Rama que se esta rebasando (`main`); `None` si se rebaso un HEAD separado.
    pub branch: Option<String>,
    /// Commit sobre el que se rebasa.
    pub onto: Option<Sha>,
    /// Paso actual y total, si git los dejo escritos.
    pub progress: Option<RebaseProgress>,
    /// Commit en el que se paro (por un conflicto o un `edit`).
    pub stopped_at: Option<Sha>,
    /// Es un rebase interactivo (con lista de pasos editable).
    pub interactive: bool,
}

/// Lo que esta a medias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Un merge con conflictos o con `--no-commit`.
    Merge {
        /// Commit que entra (`MERGE_HEAD`).
        incoming: Option<Sha>,
        /// Rama que entra, sacada del mensaje del merge (`Merge branch 'x'`).
        incoming_name: Option<String>,
    },
    /// Un rebase.
    Rebase(RebaseState),
    /// Un cherry-pick.
    CherryPick {
        /// Commit que se esta aplicando.
        commit: Option<Sha>,
    },
    /// Un revert.
    Revert {
        /// Commit que se esta deshaciendo.
        commit: Option<Sha>,
    },
}

/// Rama que entra en un merge a partir de la primera linea de `MERGE_MSG`.
///
/// Reconoce las formas que escribe git: `Merge branch 'x'`, `Merge branch 'x'
/// into y`, `Merge branch 'x' of <url>`, `Merge remote-tracking branch 'x'`,
/// `Merge tag 'x'` y `Merge commit 'x'`. Lo demas (un mensaje escrito a mano)
/// no da nombre.
#[must_use]
pub fn merge_source_name(message: &str) -> Option<String> {
    let first = message.lines().next()?.trim();
    let rest = [
        "Merge branch '",
        "Merge remote-tracking branch '",
        "Merge tag '",
        "Merge commit '",
    ]
    .into_iter()
    .find_map(|prefix| first.strip_prefix(prefix))?;
    let (name, _) = rest.split_once('\'')?;
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
mod tests;
