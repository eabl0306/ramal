//! Estado del arbol de trabajo y operacion a medias -> DTO.

use git_rudder_core_application::RepoSummary;
use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::Sha;
use git_rudder_core_domain::operation::Operation;
use git_rudder_core_domain::status::{
    BranchStatus, ChangeKind, ConflictCode, EntryState, StatusCounts, StatusEntry,
    WorkingTreeStatus,
};
use git_rudder_protocol::repos as repos_dto;
use git_rudder_protocol::status as dto;

use super::to_u32;

/// Estado listo para serializar.
#[must_use]
pub fn status_to_dto(repo_id: &RepoId, status: &WorkingTreeStatus) -> dto::WorkingTreeStatus {
    dto::WorkingTreeStatus {
        repo_id: repo_id.as_str().to_owned(),
        branch: branch_to_dto(&status.branch),
        entries: status.entries.iter().map(entry_to_dto).collect(),
        counts: counts_to_dto(status.counts()),
        operation: status.operation.as_ref().map(operation_to_dto),
    }
}

/// Resumen de una tarjeta del inicio.
#[must_use]
pub fn summary_to_dto(summary: &RepoSummary) -> repos_dto::RepoSummary {
    repos_dto::RepoSummary {
        root: summary.root.as_path().to_string_lossy().into_owned(),
        branch: branch_to_dto(&summary.branch),
        counts: counts_to_dto(summary.counts),
        operation: summary.operation.as_ref().map(operation_to_dto),
    }
}

fn branch_to_dto(branch: &BranchStatus) -> dto::BranchState {
    dto::BranchState {
        head_sha: branch.head_sha.map(Sha::to_hex),
        branch: branch.branch.clone(),
        upstream: branch.upstream.clone(),
        ahead: branch.ahead_behind.map(|distance| distance.ahead),
        behind: branch.ahead_behind.map(|distance| distance.behind),
        is_detached: branch.is_detached(),
        is_unborn: branch.is_unborn(),
    }
}

fn counts_to_dto(counts: StatusCounts) -> dto::StatusCounts {
    dto::StatusCounts {
        staged: to_u32(counts.staged),
        unstaged: to_u32(counts.unstaged),
        untracked: to_u32(counts.untracked),
        conflicted: to_u32(counts.conflicted),
    }
}

fn entry_to_dto(entry: &StatusEntry) -> dto::StatusEntry {
    let state = match entry.state {
        EntryState::Changed {
            index,
            worktree,
            submodule,
        } => dto::EntryState::Changed {
            index: change_to_dto(index),
            worktree: change_to_dto(worktree),
            submodule: submodule.map(|change| dto::SubmoduleChange {
                commit_changed: change.commit_changed,
                has_tracked_changes: change.has_tracked_changes,
                has_untracked: change.has_untracked,
            }),
        },
        EntryState::Unmerged { conflict } => dto::EntryState::Unmerged {
            conflict: conflict_to_dto(conflict),
        },
        EntryState::Untracked => dto::EntryState::Untracked,
        EntryState::Ignored => dto::EntryState::Ignored,
    };
    dto::StatusEntry {
        path: entry.path.clone(),
        original_path: entry.original_path.clone(),
        state,
        staged: entry.is_staged(),
        unstaged: entry.is_unstaged(),
    }
}

const fn change_to_dto(kind: ChangeKind) -> dto::ChangeKind {
    match kind {
        ChangeKind::Unmodified => dto::ChangeKind::Unmodified,
        ChangeKind::Modified => dto::ChangeKind::Modified,
        ChangeKind::TypeChanged => dto::ChangeKind::TypeChanged,
        ChangeKind::Added => dto::ChangeKind::Added,
        ChangeKind::Deleted => dto::ChangeKind::Deleted,
        ChangeKind::Renamed => dto::ChangeKind::Renamed,
        ChangeKind::Copied => dto::ChangeKind::Copied,
    }
}

/// Forma de un conflicto.
#[must_use]
pub const fn conflict_to_dto(code: ConflictCode) -> dto::ConflictCode {
    match code {
        ConflictCode::BothDeleted => dto::ConflictCode::BothDeleted,
        ConflictCode::AddedByUs => dto::ConflictCode::AddedByUs,
        ConflictCode::DeletedByThem => dto::ConflictCode::DeletedByThem,
        ConflictCode::AddedByThem => dto::ConflictCode::AddedByThem,
        ConflictCode::DeletedByUs => dto::ConflictCode::DeletedByUs,
        ConflictCode::BothAdded => dto::ConflictCode::BothAdded,
        ConflictCode::BothModified => dto::ConflictCode::BothModified,
    }
}

fn operation_to_dto(operation: &Operation) -> dto::Operation {
    match operation {
        Operation::Merge {
            incoming,
            incoming_name,
        } => dto::Operation::Merge {
            incoming: incoming.map(Sha::to_hex),
            incoming_name: incoming_name.clone(),
        },
        Operation::Rebase(rebase) => dto::Operation::Rebase {
            branch: rebase.branch.clone(),
            onto: rebase.onto.map(Sha::to_hex),
            step: rebase.progress.map(|progress| progress.step),
            total: rebase.progress.map(|progress| progress.total),
            stopped_at: rebase.stopped_at.map(Sha::to_hex),
            interactive: rebase.interactive,
        },
        Operation::CherryPick { commit } => dto::Operation::CherryPick {
            commit: commit.map(Sha::to_hex),
        },
        Operation::Revert { commit } => dto::Operation::Revert {
            commit: commit.map(Sha::to_hex),
        },
    }
}
