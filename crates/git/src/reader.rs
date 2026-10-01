//! `GitCliReader`: implementa `GitReader` lanzando git.
//!
//! La ruta del repositorio nunca va en argv: es el directorio de trabajo del
//! proceso. Asi una ruta que empiece por `-` no puede convertirse en una opcion.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use async_trait::async_trait;
use ramal_core_application::{GitReadError, GitReader, GitVersionInfo, LogQuery};
use ramal_core_domain::{CommitRecord, Head, RefName, RefSnapshot, RepoRoot, Sha};

use crate::cli::version::{capabilities_for, parse_version};
use crate::cli::{GitBinary, GitCommand, GitRunner, RunError, RunLimits, RunOutput};
use crate::parse::log::{LOG_DATE, LOG_FORMAT, parse_log};
use crate::parse::refs::{REFS_FORMAT, parse_refs};

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
    runner: Option<GitRunner>,
    version: OnceLock<GitVersionInfo>,
}

impl GitCliReader {
    /// Resuelve el ejecutable una vez. Si no hay git, el lector se construye igual
    /// y cada llamada devuelve `NotFound`: el motor arranca y `engine/info` puede
    /// contarlo.
    #[must_use]
    pub fn new(config: GitCliConfig) -> Self {
        let runner =
            GitBinary::resolve(config.binary).map(|binary| GitRunner::new(binary, config.limits));
        if runner.is_none() {
            tracing::warn!("no git executable found in PATH");
        }
        Self {
            runner,
            version: OnceLock::new(),
        }
    }

    fn runner(&self) -> Result<&GitRunner, GitReadError> {
        self.runner.as_ref().ok_or(GitReadError::NotFound)
    }

    async fn run(&self, cwd: &Path, command: &GitCommand) -> Result<RunOutput, GitReadError> {
        self.runner()?
            .run(Some(cwd), command)
            .await
            .map_err(run_error)
    }
}

#[async_trait]
impl GitReader for GitCliReader {
    async fn version(&self) -> Result<GitVersionInfo, GitReadError> {
        if let Some(info) = self.version.get() {
            return Ok(info.clone());
        }
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
        let info = GitVersionInfo {
            path: runner.binary().to_path_buf(),
            version,
            capabilities: capabilities_for(version),
        };
        Ok(self.version.get_or_init(|| info).clone())
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
        // Solo las refs que el grafo sabe etiquetar (las mismas que read_refs)
        // mas HEAD, para que un HEAD separado siga saliendo. Con --all salian
        // como ramas sin nombre refs/original (filter-branch), refs/prefetch
        // (git maintenance), refs/jj, refs/pull o refs/bisect, y tras un
        // filter-branch el historial viejo entero aparecia duplicado. Stash y
        // notas tampoco son historia del proyecto. --ignore-missing evita que un
        // HEAD sin nacer (rama huerfana en un repo con commits) haga fallar el
        // log. Los HEAD separados de otros worktrees quedan fuera a proposito
        // hasta que los worktrees tengan sus propias refs.
        let command = GitCommand::new("log")
            .arg("--topo-order")
            .arg("--branches")
            .arg("--remotes")
            .arg("--tags")
            .arg("--ignore-missing")
            .arg("-z")
            .arg("--no-color")
            .arg("--no-show-signature")
            .arg("--encoding=UTF-8")
            .arg(LOG_DATE)
            .arg(LOG_FORMAT)
            .max_count(query.max_count)
            // Tiene que ir lo ultimo: revisions() escribe --end-of-options y lo
            // que venga detras ya no se lee como opcion.
            .revisions(["HEAD"]);
        let output = self.run(repo.as_path(), &command).await?;
        if !output.success() {
            return Err(failed("log", &output));
        }
        parse_log(&output.stdout).map_err(|error| parse_error(&error))
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

fn first_line(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// La raiz que imprime git, sin el salto de linea. En Unix una ruta puede no ser
/// UTF-8, asi que se conserva tal cual.
fn path_from_stdout(mut stdout: Vec<u8>) -> PathBuf {
    while stdout
        .last()
        .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
    {
        stdout.pop();
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(std::ffi::OsString::from_vec(stdout))
    }
    #[cfg(not(unix))]
    {
        PathBuf::from(String::from_utf8_lossy(&stdout).into_owned())
    }
}

/// Con `LC_ALL=C` los mensajes de git son estables.
fn is_not_a_repository(stderr: &str) -> bool {
    stderr.contains("not a git repository") || stderr.contains("must be run in a work tree")
}

fn run_error(error: RunError) -> GitReadError {
    match error {
        RunError::NotFound(_) => GitReadError::NotFound,
        RunError::MissingDirectory(path) => GitReadError::NotARepository { path },
        other => GitReadError::Failed {
            detail: other.to_string(),
        },
    }
}

fn parse_error(error: &dyn std::fmt::Display) -> GitReadError {
    GitReadError::Failed {
        detail: error.to_string(),
    }
}

fn failed(subcommand: &str, output: &RunOutput) -> GitReadError {
    GitReadError::Failed {
        detail: format!(
            "git {subcommand} exited with {:?}: {}",
            output.code,
            output.stderr.trim()
        ),
    }
}
