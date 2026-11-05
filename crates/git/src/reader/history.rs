//! `HistoryReader`: revisiones, detalle de commit, busqueda, historia de un
//! fichero, blame y comparar ramas.

use async_trait::async_trait;
use git_rudder_core_application::{FoundCommit, GitReadError, HistoryReader};
use git_rudder_core_domain::blame::Blame;
use git_rudder_core_domain::history::{BranchComparison, FileHistoryEntry, MatchKind, SearchQuery};
use git_rudder_core_domain::{CommitDetail, RepoPath, RepoRoot, Sha, TrustLevel};

use super::GitCliReader;
use super::support::{failed, failed_or_missing, first_line, parse_error, path_arg};
use crate::cli::GitCommand;
use crate::parse::blame::parse_blame;
use crate::parse::commit::{DETAIL_FORMAT, parse_commit_detail};
use crate::parse::file_history::parse_file_history;
use crate::parse::log::{LOG_DATE, LOG_FORMAT};

/// Commits por lado como mucho al comparar dos ramas: mas no se leen en una
/// lista, y una rama de diez mil commits tardaria en llegar.
const MAX_COMPARE_COMMITS: usize = 1000;

/// Fichero con las revisiones que el blame ignora (formateos masivos), si esta
/// en la raiz.
const IGNORE_REVS_FILE: &str = ".git-blame-ignore-revs";

#[async_trait]
impl HistoryReader for GitCliReader {
    async fn resolve_commit(&self, repo: &RepoRoot, revision: &str) -> Result<Sha, GitReadError> {
        let missing = || GitReadError::Missing {
            what: format!("revision {revision}"),
        };
        if revision.is_empty() || revision.contains(|c: char| c.is_whitespace() || c.is_control()) {
            return Err(missing());
        }
        let command = GitCommand::new("rev-parse")
            .arg("-q")
            .arg("--verify")
            .revisions([format!("{revision}^{{commit}}")]);
        let output = self.run(repo.as_path(), &command).await?;
        match output.code {
            Some(0) => Sha::parse(&first_line(&output.stdout)).map_err(|error| parse_error(&error)),
            Some(1) => Err(missing()),
            _ => Err(failed("rev-parse", &output)),
        }
    }

    async fn commit_detail(&self, repo: &RepoRoot, sha: Sha) -> Result<CommitDetail, GitReadError> {
        let header = GitCommand::new("log")
            .arg("-1")
            .arg("--no-show-signature")
            .arg("--encoding=UTF-8")
            .arg(LOG_DATE)
            .arg(DETAIL_FORMAT)
            .revisions([sha.to_hex()]);
        let output = self.run(repo.as_path(), &header).await?;
        if !output.success() {
            return Err(failed_or_missing(
                &header,
                &output,
                &format!("commit {sha}"),
            ));
        }
        let mut detail =
            parse_commit_detail(&output.stdout).map_err(|error| parse_error(&error))?;
        let base = detail.parents.first().copied();
        detail.files = self.changed_files(repo, base, sha).await?;
        Ok(detail)
    }

    async fn search(
        &self,
        repo: &RepoRoot,
        query: &SearchQuery,
        limit: usize,
    ) -> Result<Vec<FoundCommit>, GitReadError> {
        let by_sha = match query.sha_prefix() {
            Some(prefix) => self.commit_by_prefix(repo, prefix).await?,
            None => Vec::new(),
        };
        let message = search_log(limit)
            .value("--grep=", query.text())
            .revisions(["HEAD"]);
        let author = search_log(limit)
            .value("--author=", query.text())
            .revisions(["HEAD"]);
        let (message, author) = tokio::try_join!(
            self.log_records(repo, &message),
            self.log_records(repo, &author),
        )?;
        let tagged = by_sha
            .into_iter()
            .map(|commit| (commit, MatchKind::Sha))
            .chain(
                message
                    .into_iter()
                    .map(|commit| (commit, MatchKind::Message)),
            )
            .chain(author.into_iter().map(|commit| (commit, MatchKind::Author)));
        let mut seen = std::collections::HashSet::new();
        Ok(tagged
            .filter(|(commit, _)| seen.insert(commit.node.sha))
            .take(limit)
            .map(|(commit, matched)| FoundCommit { commit, matched })
            .collect())
    }

    async fn file_history(
        &self,
        repo: &RepoRoot,
        path: &RepoPath,
        skip: usize,
        limit: usize,
    ) -> Result<Vec<FileHistoryEntry>, GitReadError> {
        let command = GitCommand::new("log")
            .arg("--follow")
            .arg("-M")
            .arg("--name-status")
            .arg("-z")
            .arg("--no-color")
            .arg("--no-show-signature")
            .arg("--encoding=UTF-8")
            .arg(LOG_DATE)
            .arg(LOG_FORMAT)
            // `--skip` no funciona con `--follow` (git se salta el seguimiento
            // del renombrado y no devuelve nada), asi que se lee hasta el final de
            // la pagina y se descarta el principio.
            .max_count(skip.saturating_add(limit))
            .revisions(["HEAD"])
            .paths([path_arg(path)]);
        let output = self.run(repo.as_path(), &command).await?;
        if !output.success() {
            // Una rama sin commits todavia no tiene historia.
            if output.stderr.contains("does not have any commits")
                || output.stderr.contains("bad revision 'HEAD'")
            {
                return Ok(Vec::new());
            }
            return Err(failed("log", &output));
        }
        let mut entries = parse_file_history(&output.stdout, path.as_str())
            .map_err(|error| parse_error(&error))?;
        Ok(entries.split_off(skip.min(entries.len())))
    }

    async fn blame(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        path: &RepoPath,
        revision: Option<Sha>,
    ) -> Result<Blame, GitReadError> {
        let mut command = GitCommand::new("blame").arg("--porcelain");
        if !trust.is_trusted() {
            command = command.arg("--no-textconv");
        }
        if tokio::fs::metadata(repo.as_path().join(IGNORE_REVS_FILE))
            .await
            .is_ok()
        {
            command = command.arg("--ignore-revs-file=.git-blame-ignore-revs");
        }
        if let Some(sha) = revision {
            command = command.object(sha);
        }
        let command = command.paths([path_arg(path)]);
        let stdout = self
            .filtered_stdout(repo, trust, &command)
            .await
            .map_err(|error| missing_path(error, path))?;
        parse_blame(&stdout).map_err(|error| parse_error(&error))
    }

    async fn compare(
        &self,
        repo: &RepoRoot,
        base: Sha,
        head: Sha,
    ) -> Result<BranchComparison, GitReadError> {
        let merge_base = self.merge_base(repo, base, head).await?;
        let ahead = record_log(MAX_COMPARE_COMMITS).revisions([format!("{base}..{head}")]);
        let behind = record_log(MAX_COMPARE_COMMITS).revisions([format!("{head}..{base}")]);
        let (ahead, behind, files) = tokio::try_join!(
            self.log_records(repo, &ahead),
            self.log_records(repo, &behind),
            self.changed_files(repo, Some(merge_base.unwrap_or(base)), head),
        )?;
        Ok(BranchComparison {
            base,
            head,
            merge_base,
            ahead,
            behind,
            files,
        })
    }
}

/// `git log` con el formato del grafo, sin revisiones todavia.
fn record_log(limit: usize) -> GitCommand {
    GitCommand::new("log")
        .arg("--topo-order")
        .arg("-z")
        .arg("--no-color")
        .arg("--no-show-signature")
        .arg("--encoding=UTF-8")
        .arg(LOG_DATE)
        .arg(LOG_FORMAT)
        .max_count(limit)
}

/// Lo que recorre la busqueda: las mismas refs que el grafo, sin distinguir
/// mayusculas y con el texto tal cual (no como expresion regular).
fn search_log(limit: usize) -> GitCommand {
    record_log(limit)
        .arg("-i")
        .arg("--fixed-strings")
        .arg("--branches")
        .arg("--remotes")
        .arg("--tags")
        .arg("--ignore-missing")
}

fn missing_path(error: GitReadError, path: &RepoPath) -> GitReadError {
    match error {
        GitReadError::Failed { detail } if detail.contains("no such path") => {
            GitReadError::Missing {
                what: format!("file {path}"),
            }
        }
        other => other,
    }
}
