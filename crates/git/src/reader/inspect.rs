//! `RepoInspector`: lo que podria ejecutar codigo, mirado sin ejecutar nada.
//!
//! `git config --local --list` no sigue `include` ni `includeIf`, asi que leer la
//! configuracion no carga ningun fichero de otro sitio (ADR 0020). La carpeta de
//! hooks sale de `rev-parse --git-path hooks`, que tiene en cuenta los worktrees
//! y `core.hooksPath`: es la carpeta cuyos hooks git ejecutaria.

use async_trait::async_trait;
use git_rudder_core_application::{GitReadError, RepoInspector};
use git_rudder_core_domain::trust::TrustInspection;
use git_rudder_core_domain::{RepoRoot, TrustLevel};

use super::GitCliReader;
use super::support::{failed, path_from_stdout};
use crate::cli::GitCommand;
use crate::parse::config::parse_config_list;

#[async_trait]
impl RepoInspector for GitCliReader {
    async fn inspect(&self, repo: &RepoRoot) -> Result<TrustInspection, GitReadError> {
        let config = GitCommand::new("config")
            .arg("--local")
            .arg("--list")
            .arg("--show-origin")
            .arg("-z");
        let hooks = GitCommand::new("rev-parse").arg("--git-path").arg("hooks");
        let dir = repo.as_path();
        let (config, hooks) = tokio::try_join!(
            self.run(dir, &config),
            self.stdout(repo, TrustLevel::Untrusted, &hooks)
        )?;
        if !config.success() {
            return Err(failed("config", &config));
        }
        let hooks_dir = dir.join(path_from_stdout(hooks));
        Ok(TrustInspection {
            config: parse_config_list(&config.stdout),
            hooks: list_files(&hooks_dir).await,
        })
    }
}

/// Nombres de los ficheros de una carpeta; ninguno si no existe.
async fn list_files(dir: &std::path::Path) -> Vec<String> {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return Vec::new();
    };
    let mut names = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let is_file = entry
            .file_type()
            .await
            .is_ok_and(|kind| kind.is_file() || kind.is_symlink());
        if is_file {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}
