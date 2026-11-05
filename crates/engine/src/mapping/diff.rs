//! Diff de un fichero y detalle de commit -> DTO.

use git_rudder_core_domain::diff::{
    DiffContent, DiffLine, FileDiff, FileStatus, Hunk, LfsPointer, LineKind, WordSegment,
};
use git_rudder_core_domain::{ChangedFile, CommitDetail, CommitRecord};
use git_rudder_protocol::commit as commit_dto;
use git_rudder_protocol::diff as dto;

/// Diff listo para serializar.
#[must_use]
pub fn diff_to_dto(diff: &FileDiff) -> dto::FileDiff {
    dto::FileDiff {
        path: diff.path.clone(),
        old_path: diff.old_path.clone(),
        status: status_to_dto(diff.status),
        content: content_to_dto(&diff.content),
        additions: diff.additions,
        deletions: diff.deletions,
        truncated: diff.truncated,
    }
}

/// Que le paso a un fichero.
#[must_use]
pub const fn status_to_dto(status: FileStatus) -> dto::FileStatus {
    match status {
        FileStatus::Added => dto::FileStatus::Added,
        FileStatus::Modified => dto::FileStatus::Modified,
        FileStatus::Deleted => dto::FileStatus::Deleted,
        FileStatus::Renamed => dto::FileStatus::Renamed,
        FileStatus::Copied => dto::FileStatus::Copied,
        FileStatus::TypeChanged => dto::FileStatus::TypeChanged,
    }
}

fn content_to_dto(content: &DiffContent) -> dto::DiffContent {
    match content {
        DiffContent::Text { hunks } => dto::DiffContent::Text {
            hunks: hunks.iter().map(hunk_to_dto).collect(),
        },
        DiffContent::Binary { image } => dto::DiffContent::Binary { image: *image },
        DiffContent::Lfs { old, new } => dto::DiffContent::Lfs {
            old: old.as_ref().map(lfs_to_dto),
            new: new.as_ref().map(lfs_to_dto),
        },
    }
}

fn lfs_to_dto(pointer: &LfsPointer) -> dto::LfsPointer {
    dto::LfsPointer {
        oid: pointer.oid.clone(),
        size: pointer.size.to_string(),
    }
}

fn hunk_to_dto(hunk: &Hunk) -> dto::Hunk {
    dto::Hunk {
        old_start: hunk.old_start,
        old_lines: hunk.old_lines,
        new_start: hunk.new_start,
        new_lines: hunk.new_lines,
        section: hunk.section.clone(),
        lines: hunk.lines.iter().map(line_to_dto).collect(),
    }
}

const fn kind_to_dto(kind: LineKind) -> dto::LineKind {
    match kind {
        LineKind::Context => dto::LineKind::Context,
        LineKind::Added => dto::LineKind::Added,
        LineKind::Removed => dto::LineKind::Removed,
    }
}

fn line_to_dto(line: &DiffLine) -> dto::DiffLine {
    dto::DiffLine {
        kind: kind_to_dto(line.kind),
        old_line: line.old_line,
        new_line: line.new_line,
        text: line.text.clone(),
        words: line.words.iter().map(word_to_dto).collect(),
        no_newline_at_end: line.no_newline_at_end,
    }
}

fn word_to_dto(word: &WordSegment) -> dto::WordSegment {
    dto::WordSegment {
        kind: kind_to_dto(word.kind),
        text: word.text.clone(),
    }
}

/// Resumen de un commit para una lista.
#[must_use]
pub fn commit_summary(commit: &CommitRecord) -> commit_dto::CommitSummary {
    commit_dto::CommitSummary {
        sha: commit.node.sha.to_hex(),
        short_sha: commit.short_sha.as_str().to_owned(),
        parents: commit.node.parents.iter().map(|sha| sha.to_hex()).collect(),
        summary: commit.summary.clone(),
        author_name: commit.author.name.clone(),
        author_email: commit.author.email.clone(),
        authored_at: commit.authored_at.to_iso8601(),
    }
}

/// Un fichero cambiado.
#[must_use]
pub fn changed_file(file: &ChangedFile) -> commit_dto::ChangedFile {
    commit_dto::ChangedFile {
        path: file.path.clone(),
        old_path: file.old_path.clone(),
        status: status_to_dto(file.status),
        additions: file.lines.map(|lines| lines.additions),
        deletions: file.lines.map(|lines| lines.deletions),
    }
}

/// Detalle de un commit.
#[must_use]
pub fn detail_to_dto(detail: &CommitDetail) -> commit_dto::CommitDetail {
    commit_dto::CommitDetail {
        sha: detail.sha.to_hex(),
        short_sha: detail.short_sha.as_str().to_owned(),
        parents: detail.parents.iter().map(|sha| sha.to_hex()).collect(),
        author_name: detail.author.name.clone(),
        author_email: detail.author.email.clone(),
        authored_at: detail.authored_at.to_iso8601(),
        committer_name: detail.committer.name.clone(),
        committer_email: detail.committer.email.clone(),
        committed_at: detail.committed_at.to_iso8601(),
        summary: detail.summary().to_owned(),
        message: detail.message.clone(),
        files: detail.files.iter().map(changed_file).collect(),
    }
}
