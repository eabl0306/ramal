//! Consultas de commits que comparten las lecturas de historial: registros del
//! log, un commit por prefijo, merge-base y ficheros cambiados.

use git_rudder_core_application::{GitReadError, HistoryReader as _};
use git_rudder_core_domain::{ChangedFile, CommitRecord, RepoRoot, Sha, TrustLevel};

use super::GitCliReader;
use super::support::{failed, first_line, parse_error};
use crate::cli::GitCommand;
use crate::parse::changes::parse_changes;
use crate::parse::log::{LOG_DATE, LOG_FORMAT, parse_log};

impl GitCliReader {
    pub(super) async fn log_records(
        &self,
        repo: &RepoRoot,
        command: &GitCommand,
    ) -> Result<Vec<CommitRecord>, GitReadError> {
        let stdout = self.stdout(repo, TrustLevel::Trusted, command).await?;
        parse_log(&stdout).map_err(|error| parse_error(&error))
    }

    pub(super) async fn commit_by_prefix(
        &self,
        repo: &RepoRoot,
        prefix: &str,
    ) -> Result<Vec<CommitRecord>, GitReadError> {
        let Ok(sha) = self.resolve_commit(repo, prefix).await else {
            return Ok(Vec::new());
        };
        let command = GitCommand::new("log")
            .arg("-1")
            .arg("-z")
            .arg("--no-show-signature")
            .arg("--encoding=UTF-8")
            .arg(LOG_DATE)
            .arg(LOG_FORMAT)
            .revisions([sha.to_hex()]);
        self.log_records(repo, &command).await
    }

    pub(super) async fn merge_base(
        &self,
        repo: &RepoRoot,
        base: Sha,
        head: Sha,
    ) -> Result<Option<Sha>, GitReadError> {
        let command = GitCommand::new("merge-base").revisions([base.to_hex(), head.to_hex()]);
        let output = self.run(repo.as_path(), &command).await?;
        match output.code {
            Some(0) => Sha::parse(&first_line(&output.stdout))
                .map(Some)
                .map_err(|error| parse_error(&error)),
            Some(1) => Ok(None),
            _ => Err(failed("merge-base", &output)),
        }
    }

    /// Ficheros de `base` a `head`; sin `base`, todo lo de `head` (commit raiz).
    pub(super) async fn changed_files(
        &self,
        repo: &RepoRoot,
        base: Option<Sha>,
        head: Sha,
    ) -> Result<Vec<ChangedFile>, GitReadError> {
        let command = |format: &'static str| {
            let command = GitCommand::new("diff-tree")
                .arg("-r")
                .arg("-z")
                .arg("-M")
                .arg("--no-commit-id")
                .arg(format);
            match base {
                Some(base) => command.revisions([base.to_hex(), head.to_hex()]),
                None => command.arg("--root").revisions([head.to_hex()]),
            }
        };
        let (status, numstat) = (command("--name-status"), command("--numstat"));
        let (status, numstat) = tokio::try_join!(
            self.stdout(repo, TrustLevel::Trusted, &status),
            self.stdout(repo, TrustLevel::Trusted, &numstat),
        )?;
        parse_changes(&status, &numstat).map_err(|error| parse_error(&error))
    }
}
