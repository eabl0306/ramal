//! `GitCliReader`: implementa los puertos de lectura lanzando git.
//!
//! La ruta del repositorio nunca va en argv: es el directorio de trabajo del
//! proceso. Asi una ruta que empiece por `-` no puede convertirse en una opcion.
//!
//! El ejecutable se puede cambiar en caliente ([`GitSetup`]): el lanzador y la
//! version detectada viven tras un candado y cada llamada se lleva una copia del
//! lanzador, asi ningun candado cruza un `await`.

mod commits;
mod conflict;
mod diff_file;
mod history;
mod inspect;
mod inventory;
mod log;
mod safety;
mod status;
mod support;

use std::path::{Path, PathBuf};
use std::sync::{PoisonError, RwLock};

use async_trait::async_trait;
use git_rudder_core_application::{GitReadError, GitReader, GitSetup, GitVersionInfo, LogQuery};
use git_rudder_core_domain::{CommitRecord, Head, RefName, RefSnapshot, RepoRoot, Sha, TrustLevel};

use crate::cli::version::{capabilities_for, parse_version};
use crate::cli::{GitBinary, GitCommand, GitRunner, Invocation, RunLimits, RunOutput};
use crate::parse::refs::{REFS_FORMAT, parse_refs};
use support::{failed, first_line, is_not_a_repository, parse_error, path_from_stdout, run_error};

/// Configuracion del lector.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitCliConfig {
    /// Ejecutable explicito; sin el, el primer `git` del `PATH`.
    pub binary: Option<PathBuf>,
    /// Limites de cada llamada.
    pub limits: RunLimits,
}

/// Lector de git sobre el CLI.
#[derive(Debug)]
pub struct GitCliReader {
    runner: RwLock<Option<GitRunner>>,
    version: RwLock<Option<GitVersionInfo>>,
    limits: RunLimits,
}

impl GitCliReader {
    /// Resuelve el ejecutable una vez. Si no hay git, el lector se construye igual
    /// y cada llamada devuelve `NotFound`: el motor arranca y `engine/info` puede
    /// contarlo.
    #[must_use]
    pub fn new(config: GitCliConfig) -> Self {
        let reader = Self {
            runner: RwLock::new(None),
            version: RwLock::new(None),
            limits: config.limits,
        };
        reader.use_binary(config.binary);
        reader
    }

    fn use_binary(&self, explicit: Option<PathBuf>) {
        let runner = GitBinary::resolve(explicit).map(|binary| GitRunner::new(binary, self.limits));
        if runner.is_none() {
            tracing::warn!("no git executable found in PATH");
        }
        *self.runner.write().unwrap_or_else(PoisonError::into_inner) = runner;
        *self.version.write().unwrap_or_else(PoisonError::into_inner) = None;
    }

    fn runner(&self) -> Result<GitRunner, GitReadError> {
        self.runner
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .ok_or(GitReadError::NotFound)
    }

    /// Lanza una llamada.
    pub(crate) async fn invoke(&self, call: Invocation<'_>) -> Result<RunOutput, GitReadError> {
        self.runner()?.invoke(call).await.map_err(run_error)
    }

    /// Lanza `command` en el repositorio.
    pub(crate) async fn run(
        &self,
        cwd: &Path,
        command: &GitCommand,
    ) -> Result<RunOutput, GitReadError> {
        self.invoke(Invocation::new(Some(cwd), command)).await
    }

    /// Lanza `command` en el repositorio con su confianza y exige que salga bien.
    pub(crate) async fn stdout(
        &self,
        repo: &RepoRoot,
        trust: TrustLevel,
        command: &GitCommand,
    ) -> Result<Vec<u8>, GitReadError> {
        let call = Invocation {
            untrusted: !trust.is_trusted(),
            ..Invocation::new(Some(repo.as_path()), command)
        };
        let output = self.invoke(call).await?;
        if !output.success() {
            return Err(failed(&command.subcommand(), &output));
        }
        Ok(output.stdout)
    }

    async fn detect_version(&self) -> Result<GitVersionInfo, GitReadError> {
        let runner = self.runner()?;
        let output = runner
            .run(None, &GitCommand::new("version"))
            .await
            .map_err(run_error)?;
        if !output.success() {
            return Err(failed("version", &output));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        let version = parse_version(&text).ok_or_else(|| GitReadError::Failed {
            detail: format!("unrecognized git version output: {}", text.trim()),
        })?;
        Ok(GitVersionInfo {
            path: runner.binary().to_path_buf(),
            version,
            capabilities: capabilities_for(version),
        })
    }
}

#[async_trait]
impl GitReader for GitCliReader {
    async fn version(&self) -> Result<GitVersionInfo, GitReadError> {
        let cached = self
            .version
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(info) = cached {
            return Ok(info);
        }
        let info = self.detect_version().await?;
        *self.version.write().unwrap_or_else(PoisonError::into_inner) = Some(info.clone());
        Ok(info)
    }

    async fn discover(&self, path: &Path) -> Result<RepoRoot, GitReadError> {
        let command = GitCommand::new("rev-parse").arg("--show-toplevel");
        let output = self.run(path, &command).await?;
        if output.success() {
            return Ok(RepoRoot::new(path_from_stdout(output.stdout)));
        }
        if is_not_a_repository(&output.stderr) {
            return Err(GitReadError::NotARepository {
                path: path.to_path_buf(),
            });
        }
        Err(failed("rev-parse", &output))
    }

    async fn read_refs(&self, repo: &RepoRoot) -> Result<RefSnapshot, GitReadError> {
        let dir = repo.as_path();
        let refs = GitCommand::new("for-each-ref")
            .arg(REFS_FORMAT)
            .arg("refs/heads")
            .arg("refs/remotes")
            .arg("refs/tags");
        let symbolic = GitCommand::new("symbolic-ref").arg("-q").arg("HEAD");
        let target = GitCommand::new("rev-parse")
            .arg("-q")
            .arg("--verify")
            .revisions(["HEAD^{commit}"]);
        let (refs, symbolic, target) = tokio::try_join!(
            self.run(dir, &refs),
            self.run(dir, &symbolic),
            self.run(dir, &target),
        )?;
        if !refs.success() {
            return Err(failed("for-each-ref", &refs));
        }
        let parsed = parse_refs(&refs.stdout).map_err(|error| parse_error(&error))?;
        let head = head_from(&symbolic, &target)?;
        Ok(RefSnapshot::new(parsed.refs, head, parsed.remote_default))
    }

    async fn read_log(
        &self,
        repo: &RepoRoot,
        query: &LogQuery,
    ) -> Result<Vec<CommitRecord>, GitReadError> {
        log::read_log(self, repo, query).await
    }
}

#[async_trait]
impl GitSetup for GitCliReader {
    async fn reconfigure(&self, binary: Option<PathBuf>) -> Result<GitVersionInfo, GitReadError> {
        self.use_binary(binary);
        self.version().await
    }
}

/// HEAD a partir de `symbolic-ref -q HEAD` (1 si esta separado) y de
/// `rev-parse -q --verify HEAD^{commit}` (1 si la rama no ha nacido).
fn head_from(symbolic: &RunOutput, target: &RunOutput) -> Result<Head, GitReadError> {
    let target = match target.code {
        Some(0) => Some(
            Sha::parse(first_line(&target.stdout).as_str()).map_err(|error| parse_error(&error))?,
        ),
        Some(1) => None,
        _ => return Err(failed("rev-parse", target)),
    };
    match (symbolic.code, target) {
        (Some(0), target) => {
            let name = RefName::parse(first_line(&symbolic.stdout).as_str())
                .map_err(|error| parse_error(&error))?;
            Ok(Head::Branch { name, target })
        }
        (Some(1), Some(target)) => Ok(Head::Detached { target }),
        _ => Err(failed("symbolic-ref", symbolic)),
    }
}
