//! La operacion a medias a partir de los ficheros que git deja en su carpeta.
//!
//! - Rebase interactivo o con merge: `rebase-merge/` con `head-name`, `onto`,
//!   `msgnum`, `end`, `stopped-sha` y el fichero `interactive`.
//! - Rebase con apply: `rebase-apply/` con `head-name`, `onto`, `next` y `last`.
//!   Si tiene `applying` es un `git am`, que no es un rebase.
//! - Merge: `MERGE_HEAD` y la primera linea de `MERGE_MSG`.
//! - `CHERRY_PICK_HEAD` y `REVERT_HEAD`.
//!
//! Durante un rebase parado puede haber tambien un `CHERRY_PICK_HEAD` o un
//! `MERGE_HEAD` de su paso, asi que el rebase manda.

use git_rudder_core_domain::Sha;
use git_rudder_core_domain::operation::{
    Operation, RebaseProgress, RebaseState, merge_source_name,
};

/// Lo leido de una carpeta de rebase.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RebaseFiles {
    /// `head-name`.
    pub head_name: Option<String>,
    /// `onto`.
    pub onto: Option<String>,
    /// Paso actual (`msgnum` o `next`).
    pub step: Option<String>,
    /// Total (`end` o `last`).
    pub total: Option<String>,
    /// `stopped-sha`.
    pub stopped: Option<String>,
    /// Existe `interactive`.
    pub interactive: bool,
}

/// Lo leido de la carpeta de git.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperationFiles {
    /// `rebase-merge/`, si existe.
    pub rebase_merge: Option<RebaseFiles>,
    /// `rebase-apply/`, si existe y no es un `git am`.
    pub rebase_apply: Option<RebaseFiles>,
    /// `MERGE_HEAD`.
    pub merge_head: Option<String>,
    /// `MERGE_MSG`.
    pub merge_msg: Option<String>,
    /// `CHERRY_PICK_HEAD`.
    pub cherry_pick_head: Option<String>,
    /// `REVERT_HEAD`.
    pub revert_head: Option<String>,
}

/// La operacion en curso, si la hay.
#[must_use]
pub fn detect_operation(files: &OperationFiles) -> Option<Operation> {
    if let Some(rebase) = files.rebase_merge.as_ref().or(files.rebase_apply.as_ref()) {
        return Some(Operation::Rebase(rebase_state(rebase)));
    }
    if let Some(head) = &files.merge_head {
        return Some(Operation::Merge {
            incoming: sha(head),
            incoming_name: files.merge_msg.as_deref().and_then(merge_source_name),
        });
    }
    if let Some(head) = &files.cherry_pick_head {
        return Some(Operation::CherryPick { commit: sha(head) });
    }
    files
        .revert_head
        .as_ref()
        .map(|head| Operation::Revert { commit: sha(head) })
}

fn rebase_state(files: &RebaseFiles) -> RebaseState {
    let number = |text: &Option<String>| text.as_deref().and_then(|t| t.trim().parse::<u32>().ok());
    let progress = number(&files.step)
        .zip(number(&files.total))
        .map(|(step, total)| RebaseProgress { step, total });
    let branch = files
        .head_name
        .as_deref()
        .map(str::trim)
        .and_then(|name| name.strip_prefix("refs/heads/"))
        .map(str::to_owned);
    RebaseState {
        branch,
        onto: files.onto.as_deref().and_then(sha),
        progress,
        stopped_at: files.stopped.as_deref().and_then(sha),
        interactive: files.interactive,
    }
}

/// La primera linea si es un sha completo.
fn sha(text: &str) -> Option<Sha> {
    Sha::parse(text.lines().next()?.trim()).ok()
}

#[cfg(test)]
mod tests;
