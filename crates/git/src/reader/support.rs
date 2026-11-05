//! Ayudas comunes de los lectores: errores, salida de git y rutas.

use std::path::{Path, PathBuf};

use git_rudder_core_application::GitReadError;
use git_rudder_core_domain::{RepoPath, RepoRoot, TrustLevel};

use crate::cli::{GitCommand, RunError, RunOutput};

/// Primera linea de la salida, sin el salto.
pub(super) fn first_line(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// La raiz que imprime git, sin el salto de linea. En Unix una ruta puede no ser
/// UTF-8, asi que se conserva tal cual.
pub(super) fn path_from_stdout(mut stdout: Vec<u8>) -> PathBuf {
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
pub(super) fn is_not_a_repository(stderr: &str) -> bool {
    stderr.contains("not a git repository") || stderr.contains("must be run in a work tree")
}

/// Mensajes de git cuando una revision o una ruta no existen.
pub(super) fn is_missing(stderr: &str) -> bool {
    [
        "unknown revision",
        "bad revision",
        "bad object",
        "invalid object name",
        "does not exist",
        "no such path",
        "Needed a single revision",
        "not a valid object name",
        "ambiguous argument",
    ]
    .iter()
    .any(|needle| stderr.contains(needle))
}

pub(super) fn run_error(error: RunError) -> GitReadError {
    match error {
        RunError::NotFound(_) => GitReadError::NotFound,
        RunError::MissingDirectory(path) => GitReadError::NotARepository { path },
        other => GitReadError::Failed {
            detail: other.to_string(),
        },
    }
}

pub(super) fn parse_error(error: &dyn std::fmt::Display) -> GitReadError {
    GitReadError::Failed {
        detail: error.to_string(),
    }
}

pub(super) fn failed(subcommand: &str, output: &RunOutput) -> GitReadError {
    GitReadError::Failed {
        detail: format!(
            "git {subcommand} exited with {:?}: {}",
            output.code,
            output.stderr.trim()
        ),
    }
}

/// Un fallo de git que significa que lo pedido no existe se cuenta como tal.
pub(super) fn failed_or_missing(
    command: &GitCommand,
    output: &RunOutput,
    what: &str,
) -> GitReadError {
    if is_missing(&output.stderr) {
        return GitReadError::Missing {
            what: what.to_owned(),
        };
    }
    failed(&command.subcommand(), output)
}

/// Opciones de un subcomando que calcula diffs: sin confianza, ni diff externo
/// ni `textconv` (ADR 0020). Con ella, git usa lo que diga la configuracion.
pub(super) fn diff_safety(command: GitCommand, trust: TrustLevel) -> GitCommand {
    let command = command.arg("--no-ext-diff");
    if trust.is_trusted() {
        command
    } else {
        command.arg("--no-textconv")
    }
}

/// Comprueba que un fichero que se va a leer del disco no sale del arbol de
/// trabajo despues de seguir enlaces simbolicos (un enlace a una carpeta de
/// fuera haria leer lo que no es del repositorio). Un fichero que no existe no
/// sale de ningun sitio.
pub(super) async fn ensure_inside(repo: &RepoRoot, path: &RepoPath) -> Result<(), GitReadError> {
    let root = tokio::fs::canonicalize(repo.as_path())
        .await
        .map_err(|error| GitReadError::Failed {
            detail: format!("cannot resolve the work tree: {error}"),
        })?;
    let full = repo.as_path().join(path.as_str());
    let Ok(resolved) = tokio::fs::canonicalize(&full).await else {
        return Ok(());
    };
    if resolved.starts_with(&root) {
        return Ok(());
    }
    Err(GitReadError::Missing {
        what: format!("{path} inside the work tree"),
    })
}

/// Una ruta del repositorio como texto, para pasarla a git tras `--`.
pub(super) fn path_arg(path: &RepoPath) -> &Path {
    Path::new(path.as_str())
}
