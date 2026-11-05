//! Refs, stashes, submodulos, conflictos, historia y confianza -> DTO.

use git_rudder_core_application::FileHistoryPage;
use git_rudder_core_domain::RepoId;
use git_rudder_core_domain::Sha;
use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::conflict::{ConflictFile, ConflictSegment};
use git_rudder_core_domain::history::{BranchComparison, MatchKind, SearchHit};
use git_rudder_core_domain::refs::{BranchInfo, RefListing};
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::submodule::{Submodule, SubmoduleState};
use git_rudder_core_domain::trust::TrustFinding;
use git_rudder_protocol::history as history_dto;
use git_rudder_protocol::inventory as dto;
use git_rudder_protocol::refs as refs_dto;
use git_rudder_protocol::repos as repos_dto;

use super::diff::{changed_file, commit_summary, diff_to_dto, status_to_dto};
use super::status::conflict_to_dto;
use super::to_u32;

/// Lo que lista la barra lateral.
#[must_use]
pub fn listing_to_dto(repo_id: &RepoId, listing: &RefListing) -> refs_dto::RefListing {
    refs_dto::RefListing {
        repo_id: repo_id.as_str().to_owned(),
        local: listing.local.iter().map(branch_to_dto).collect(),
        remote: listing.remote.iter().map(branch_to_dto).collect(),
        tags: listing
            .tags
            .iter()
            .map(|tag| refs_dto::TagInfo {
                name: tag.name.short_name().to_owned(),
                full_name: tag.name.as_str().to_owned(),
                sha: tag.target.to_hex(),
                annotation: tag.annotation.clone(),
            })
            .collect(),
        remotes: listing
            .remotes
            .iter()
            .map(|remote| refs_dto::RemoteInfo {
                name: remote.name.clone(),
                fetch_url: remote.fetch_url.clone(),
                push_url: remote.push_url.clone(),
                web_url: remote.web_url(),
            })
            .collect(),
    }
}

fn branch_to_dto(branch: &BranchInfo) -> refs_dto::BranchInfo {
    let tracking = branch.tracking.as_ref();
    let distance = tracking.and_then(|tracking| tracking.distance);
    refs_dto::BranchInfo {
        name: branch.name.short_name().to_owned(),
        full_name: branch.name.as_str().to_owned(),
        sha: branch.target.to_hex(),
        is_head: branch.is_head,
        upstream: tracking.map(|tracking| tracking.upstream.clone()),
        ahead: distance.map(|distance| distance.ahead),
        behind: distance.map(|distance| distance.behind),
        upstream_gone: tracking.is_some_and(git_rudder_core_domain::refs::Tracking::is_gone),
    }
}

/// Lista de stashes.
#[must_use]
pub fn stashes_to_dto(stashes: &[StashEntry]) -> dto::StashList {
    dto::StashList {
        stashes: stashes
            .iter()
            .map(|stash| dto::StashInfo {
                index: stash.index,
                sha: stash.sha.to_hex(),
                message: stash.message.clone(),
                branch: stash.branch.clone(),
                created_at: stash.created_at.to_iso8601(),
            })
            .collect(),
    }
}

/// Contenido de un stash.
#[must_use]
pub fn stash_contents_to_dto(contents: &StashContents) -> dto::StashContents {
    dto::StashContents {
        tracked: contents.tracked.iter().map(diff_to_dto).collect(),
        untracked: contents.untracked.iter().map(diff_to_dto).collect(),
    }
}

/// Lista de submodulos.
#[must_use]
pub fn submodules_to_dto(submodules: &[Submodule]) -> dto::SubmoduleList {
    dto::SubmoduleList {
        submodules: submodules
            .iter()
            .map(|submodule| dto::SubmoduleInfo {
                path: submodule.path.clone(),
                sha: submodule.sha.to_hex(),
                state: match submodule.state {
                    SubmoduleState::Uninitialized => dto::SubmoduleState::Uninitialized,
                    SubmoduleState::UpToDate => dto::SubmoduleState::UpToDate,
                    SubmoduleState::Modified => dto::SubmoduleState::Modified,
                    SubmoduleState::Conflicted => dto::SubmoduleState::Conflicted,
                },
                describe: submodule.describe.clone(),
                can_open: submodule.can_open(),
            })
            .collect(),
    }
}

/// Un fichero en conflicto.
#[must_use]
pub fn conflict_to_file_dto(file: &ConflictFile) -> dto::ConflictFile {
    let content = file.content.as_ref();
    dto::ConflictFile {
        path: file.path.clone(),
        code: conflict_to_dto(file.code),
        binary: file.binary,
        segments: content.map(|parsed| parsed.segments.iter().map(segment_to_dto).collect()),
        labels: content
            .and_then(|parsed| parsed.labels.as_ref())
            .map(|labels| dto::ConflictLabels {
                ours: labels.ours.clone(),
                theirs: labels.theirs.clone(),
                base: labels.base.clone(),
            }),
    }
}

fn segment_to_dto(segment: &ConflictSegment) -> dto::ConflictSegment {
    match segment {
        ConflictSegment::Context { lines } => dto::ConflictSegment::Context {
            lines: lines.clone(),
        },
        ConflictSegment::Hunk { ours, base, theirs } => dto::ConflictSegment::Hunk {
            ours: ours.clone(),
            base: base.clone(),
            theirs: theirs.clone(),
        },
    }
}

/// Resultados de una busqueda.
#[must_use]
pub fn search_to_dto(repo_id: &RepoId, hits: &[SearchHit]) -> history_dto::GraphSearchResult {
    history_dto::GraphSearchResult {
        repo_id: repo_id.as_str().to_owned(),
        hits: hits
            .iter()
            .map(|hit| history_dto::SearchHit {
                commit: commit_summary(&hit.commit),
                matched: match hit.matched {
                    MatchKind::Sha => history_dto::MatchKind::Sha,
                    MatchKind::Message => history_dto::MatchKind::Message,
                    MatchKind::Author => history_dto::MatchKind::Author,
                },
                row: hit.row.map(to_u32),
            })
            .collect(),
    }
}

/// Una pagina de la historia de un fichero.
#[must_use]
pub fn history_to_dto(page: &FileHistoryPage) -> history_dto::FileHistoryPage {
    history_dto::FileHistoryPage {
        entries: page
            .entries
            .iter()
            .map(|entry| history_dto::FileHistoryEntry {
                commit: commit_summary(&entry.commit),
                path: entry.path.clone(),
                old_path: entry.old_path.clone(),
                status: status_to_dto(entry.status),
            })
            .collect(),
        has_more: page.has_more,
    }
}

/// Blame de un fichero.
#[must_use]
pub fn blame_to_dto(blame: &Blame) -> history_dto::Blame {
    history_dto::Blame {
        lines: blame
            .lines
            .iter()
            .map(|line| history_dto::BlameLine {
                line: line.line,
                original_line: line.original_line,
                sha: line.sha.to_hex(),
                text: line.text.clone(),
            })
            .collect(),
        commits: blame
            .commits
            .iter()
            .map(|commit| history_dto::BlameCommit {
                sha: commit.sha.to_hex(),
                author_name: commit.author.name.clone(),
                author_email: commit.author.email.clone(),
                authored_at: commit.authored_at.to_iso8601(),
                summary: commit.summary.clone(),
                path: commit.path.clone(),
                uncommitted: commit.is_uncommitted(),
            })
            .collect(),
    }
}

/// Dos refs comparadas.
#[must_use]
pub fn comparison_to_dto(comparison: &BranchComparison) -> history_dto::Comparison {
    history_dto::Comparison {
        base: comparison.base.to_hex(),
        head: comparison.head.to_hex(),
        merge_base: comparison.merge_base.map(Sha::to_hex),
        ahead: comparison.ahead.iter().map(commit_summary).collect(),
        behind: comparison.behind.iter().map(commit_summary).collect(),
        files: comparison.files.iter().map(changed_file).collect(),
    }
}

/// Lo que obliga a preguntar antes de confiar.
#[must_use]
pub fn finding_to_dto(finding: &TrustFinding) -> repos_dto::TrustFinding {
    match finding {
        TrustFinding::ConfigKey { key, origin } => repos_dto::TrustFinding::ConfigKey {
            key: key.clone(),
            origin: origin.clone(),
        },
        TrustFinding::Hook { name } => repos_dto::TrustFinding::Hook { name: name.clone() },
    }
}
