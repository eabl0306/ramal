//! `StatusReader`: estado del arbol de trabajo, operacion a medias y diffs.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use git_rudder_core_application::{DiffRequest, GitReadError, StatusReader};
use git_rudder_core_domain::diff::{DiffTarget, FileDiff, Whitespace};
use git_rudder_core_domain::status::WorkingTreeStatus;
use git_rudder_core_domain::{RepoRoot, TrustLevel};

use super::GitCliReader;
use super::diff_file::{MAX_DIFF_BYTES, file_diff};
use super::support::{
    diff_safety, ensure_inside, failed_or_missing, parse_error, path_arg, path_from_stdout,
};
use crate::cli::{GitCommand, Invocation};
use crate::parse::diff::parse_diff;
use crate::parse::operation::{OperationFiles, RebaseFiles, detect_operation};
use crate::parse::status::parse_status;

#[async_trait]
impl StatusReader for GitCliReader {
    async fn read_status(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
    ) -> Result<WorkingTreeStatus, GitReadError> {
        let status = GitCommand::new("status")
            .arg("--porcelain=v2")
            .arg("--branch")
            .arg("-z")
            .arg("--untracked-files=all")
            .arg("--ahead-behind")
            .arg("--find-renames");
        let git_dir = GitCommand::new("rev-parse").arg("--absolute-git-dir");
        let (status, git_dir) = tokio::try_join!(
            self.filtered_stdout(repo, trust, &status),
            self.stdout(repo, trust, &git_dir),
        )?;
        let mut parsed = parse_status(&status).map_err(|error| parse_error(&error))?;
        let files = read_operation_files(&path_from_stdout(git_dir)).await;
        parsed.operation = detect_operation(&files);
        Ok(parsed)
    }

    async fn read_diff(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        request: &DiffRequest,
    ) -> Result<FileDiff, GitReadError> {
        let command = diff_command(request, trust);
        let (text, truncated) = self.diff_text(repo, trust, &command).await?;
        let mut sections = parse_diff(&text, request.options.word_diff);
        let untracked = sections.is_empty()
            && request.target == DiffTarget::WorkingTree
            && self.is_untracked(repo, trust, request).await?;
        if !untracked {
            return Ok(file_diff(request, sections, truncated));
        }
        // Un fichero sin seguimiento no sale en `git diff`: se compara con
        // /dev/null leyendolo del disco, asi que antes se comprueba que la ruta no
        // salga del arbol de trabajo por un enlace simbolico.
        ensure_inside(repo, &request.path).await?;
        let no_index = options(
            diff_safety(GitCommand::new("diff").arg("--no-index"), trust),
            request,
        )
        .paths([Path::new("/dev/null"), path_arg(&request.path)]);
        let (text, truncated) = self.diff_text(repo, trust, &no_index).await?;
        sections = parse_diff(&text, request.options.word_diff);
        Ok(file_diff(request, sections, truncated))
    }
}

impl GitCliReader {
    /// Texto de un diff, recortado si es enorme. `--no-index` sale con 1 si hay
    /// diferencias, y eso no es un fallo.
    async fn diff_text(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        command: &GitCommand,
    ) -> Result<(String, bool), GitReadError> {
        let overrides = self.filter_overrides(repo, trust).await?;
        let call = Invocation {
            untrusted: !trust.is_trusted(),
            config: &overrides,
            truncate_at: Some(MAX_DIFF_BYTES),
            ..Invocation::new(Some(repo.as_path()), command)
        };
        let output = self.invoke(call).await?;
        if !matches!(output.code, Some(0 | 1)) {
            return Err(failed_or_missing(command, &output, "file"));
        }
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.truncated {
            // La ultima linea puede haber quedado a medias.
            let keep = text.rfind('\n').map_or(0, |end| end + 1);
            text.truncate(keep);
        }
        Ok((text, output.truncated))
    }

    async fn is_untracked(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        request: &DiffRequest,
    ) -> Result<bool, GitReadError> {
        let command = GitCommand::new("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .arg("-z")
            .paths([path_arg(&request.path)]);
        let output = self.stdout(repo, trust, &command).await?;
        Ok(!output.is_empty())
    }
}

fn diff_command(request: &DiffRequest, trust: TrustLevel) -> GitCommand {
    let base = match request.target {
        DiffTarget::WorkingTree | DiffTarget::Range { .. } => GitCommand::new("diff"),
        DiffTarget::Index => GitCommand::new("diff").arg("--cached"),
        DiffTarget::Commit(_) => GitCommand::new("show")
            .arg("--format=")
            .arg("--no-show-signature")
            .arg("--diff-merges=first-parent"),
    };
    let command = options(diff_safety(base, trust), request).arg("-M");
    let command = match request.target {
        DiffTarget::Commit(sha) => command.revisions([sha.to_hex()]),
        DiffTarget::Range { from, to } => command.revisions([from.to_hex(), to.to_hex()]),
        DiffTarget::WorkingTree | DiffTarget::Index => command,
    };
    let old = request.old_path.iter().map(path_arg);
    command.paths(std::iter::once(path_arg(&request.path)).chain(old))
}

/// Color, contexto, espacios y palabras.
fn options(command: GitCommand, request: &DiffRequest) -> GitCommand {
    let options = request.options;
    let command = command.arg("--no-color").number(
        "-U",
        usize::try_from(options.context_lines).unwrap_or(usize::MAX),
    );
    let command = match options.whitespace {
        Whitespace::Exact => command,
        Whitespace::IgnoreAll => command.arg("--ignore-all-space"),
        Whitespace::IgnoreChange => command.arg("--ignore-space-change"),
        Whitespace::IgnoreAtEol => command.arg("--ignore-space-at-eol"),
    };
    if options.word_diff {
        command.arg("--word-diff=porcelain")
    } else {
        command
    }
}

/// Lo que git deja en su carpeta durante una operacion. Un fichero que no se
/// puede leer cuenta como que no esta.
async fn read_operation_files(git_dir: &Path) -> OperationFiles {
    let read = |name: &str| read_text(git_dir.join(name));
    let (merge_head, merge_msg, cherry_pick_head, revert_head) = tokio::join!(
        read("MERGE_HEAD"),
        read("MERGE_MSG"),
        read("CHERRY_PICK_HEAD"),
        read("REVERT_HEAD"),
    );
    let rebase_merge = read_rebase(git_dir.join("rebase-merge"), "msgnum", "end").await;
    let rebase_apply = if exists(&git_dir.join("rebase-apply").join("applying")).await {
        None
    } else {
        read_rebase(git_dir.join("rebase-apply"), "next", "last").await
    };
    OperationFiles {
        rebase_merge,
        rebase_apply,
        merge_head,
        merge_msg,
        cherry_pick_head,
        revert_head,
    }
}

async fn read_rebase(dir: PathBuf, step: &str, total: &str) -> Option<RebaseFiles> {
    if !exists(&dir).await {
        return None;
    }
    let (head_name, onto, step, total, stopped) = tokio::join!(
        read_text(dir.join("head-name")),
        read_text(dir.join("onto")),
        read_text(dir.join(step)),
        read_text(dir.join(total)),
        read_text(dir.join("stopped-sha")),
    );
    Some(RebaseFiles {
        head_name,
        onto,
        step,
        total,
        stopped,
        interactive: exists(&dir.join("interactive")).await,
    })
}

async fn read_text(path: PathBuf) -> Option<String> {
    tokio::fs::read_to_string(path).await.ok()
}

async fn exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}
