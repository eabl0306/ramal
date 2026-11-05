//! `InventoryReader`: ramas, tags y remotos; stashes; submodulos.

use async_trait::async_trait;
use git_rudder_core_application::{GitReadError, InventoryReader};
use git_rudder_core_domain::diff::FileDiff;
use git_rudder_core_domain::refs::RefListing;
use git_rudder_core_domain::stash::{StashContents, StashEntry};
use git_rudder_core_domain::submodule::Submodule;
use git_rudder_core_domain::{RepoPath, RepoRoot, Sha, TrustLevel};

use super::GitCliReader;
use super::diff_file::{MAX_DIFF_BYTES, file_diff};
use super::support::{diff_safety, failed, first_line, parse_error};
use crate::cli::{GitCommand, Invocation};
use crate::parse::changes::parse_name_status;
use crate::parse::diff::{FileSection, parse_diff};
use crate::parse::inventory::{
    LISTING_FORMAT, STASH_FORMAT, parse_listing, parse_remotes, parse_stashes, parse_submodules,
};

#[async_trait]
impl InventoryReader for GitCliReader {
    async fn list_refs(&self, repo: &RepoRoot) -> Result<RefListing, GitReadError> {
        let refs = GitCommand::new("for-each-ref")
            .arg(LISTING_FORMAT)
            .arg("refs/heads")
            .arg("refs/remotes")
            .arg("refs/tags");
        let head = GitCommand::new("symbolic-ref").arg("-q").arg("HEAD");
        let remotes = GitCommand::new("config")
            .arg("-z")
            .arg("--get-regexp")
            .arg("^remote\\..*\\.(url|pushurl)$");
        let dir = repo.as_path();
        let (refs, head, remotes) = tokio::try_join!(
            self.run(dir, &refs),
            self.run(dir, &head),
            self.run(dir, &remotes),
        )?;
        if !refs.success() {
            return Err(failed("for-each-ref", &refs));
        }
        let head = head.success().then(|| first_line(&head.stdout));
        let mut listing =
            parse_listing(&refs.stdout, head.as_deref()).map_err(|error| parse_error(&error))?;
        // `config --get-regexp` sale con 1 si no hay ningun remoto.
        if remotes.success() {
            listing.remotes = parse_remotes(&remotes.stdout);
        }
        Ok(listing)
    }

    async fn list_stashes(&self, repo: &RepoRoot) -> Result<Vec<StashEntry>, GitReadError> {
        let command = GitCommand::new("stash")
            .arg("list")
            .arg("-z")
            .arg("--no-show-signature")
            .arg(STASH_FORMAT);
        let stdout = self.stdout(repo, TrustLevel::Trusted, &command).await?;
        parse_stashes(&stdout).map_err(|error| parse_error(&error))
    }

    async fn stash_contents(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        stash: Sha,
    ) -> Result<StashContents, GitReadError> {
        let (first_parent, stash_hex) = (format!("{stash}^1"), stash.to_hex());
        let tracked = self.stash_diff(repo, trust, &first_parent, &stash_hex);
        let untracked = async {
            let third = format!("{stash}^3");
            if self.revision_exists(repo, &third).await? {
                // El tercer padre es un commit raiz con solo los no rastreados:
                // su diff contra nada los da todos como anadidos.
                self.stash_diff(repo, trust, EMPTY_TREE, &third).await
            } else {
                Ok(Vec::new())
            }
        };
        let (tracked, untracked) = tokio::try_join!(tracked, untracked)?;
        Ok(StashContents { tracked, untracked })
    }

    async fn list_submodules(&self, repo: &RepoRoot) -> Result<Vec<Submodule>, GitReadError> {
        let command = GitCommand::new("submodule")
            .arg("status")
            .arg("--recursive");
        let stdout = self.stdout(repo, TrustLevel::Untrusted, &command).await?;
        parse_submodules(&stdout).map_err(|error| parse_error(&error))
    }
}

/// El arbol vacio de git: existe en cualquier repositorio sin estar guardado.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

impl GitCliReader {
    async fn revision_exists(&self, repo: &RepoRoot, revision: &str) -> Result<bool, GitReadError> {
        let command = GitCommand::new("rev-parse")
            .arg("-q")
            .arg("--verify")
            .revisions([revision]);
        let output = self.run(repo.as_path(), &command).await?;
        Ok(output.success())
    }

    /// El diff de todos los ficheros entre dos commits, uno por fichero. Las
    /// secciones del diff salen en el mismo orden que `--name-status`, y de ahi
    /// se saca la ruta de cada una sin leer la cabecera ambigua.
    async fn stash_diff(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        from: &str,
        to: &str,
    ) -> Result<Vec<FileDiff>, GitReadError> {
        let names = GitCommand::new("diff")
            .arg("--name-status")
            .arg("-z")
            .arg("-M")
            .revisions([from, to]);
        let patch = diff_safety(GitCommand::new("diff"), trust)
            .arg("--no-color")
            .arg("-M")
            .revisions([from, to]);
        let call = Invocation {
            untrusted: !trust.is_trusted(),
            truncate_at: Some(MAX_DIFF_BYTES),
            ..Invocation::new(Some(repo.as_path()), &patch)
        };
        let (names, patch) = tokio::try_join!(self.stdout(repo, trust, &names), self.invoke(call))?;
        if !patch.success() {
            return Err(failed("diff", &patch));
        }
        let names = parse_name_status(&names).map_err(|error| parse_error(&error))?;
        let text = String::from_utf8_lossy(&patch.stdout);
        let sections = parse_diff(&text, false);
        let count = sections.len();
        Ok(names
            .into_iter()
            .zip(sections)
            .enumerate()
            .filter_map(|(position, ((_, old, path), section))| {
                let truncated = patch.truncated && position + 1 == count;
                section_diff(&path, old.as_deref(), section, truncated)
            })
            .collect())
    }
}

fn section_diff(
    path: &str,
    old: Option<&str>,
    section: FileSection,
    truncated: bool,
) -> Option<FileDiff> {
    let request = git_rudder_core_application::DiffRequest {
        target: git_rudder_core_domain::diff::DiffTarget::WorkingTree,
        path: RepoPath::parse(path).ok()?,
        old_path: old.and_then(|old| RepoPath::parse(old).ok()),
        options: git_rudder_core_domain::diff::DiffOptions::default(),
    };
    Some(file_diff(&request, vec![section], truncated))
}
