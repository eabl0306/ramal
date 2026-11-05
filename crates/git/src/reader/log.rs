//! Log del grafo: los commits en orden topologico desde las refs que se piden.

use git_rudder_core_application::{GitReadError, LogQuery, LogRevisions};
use git_rudder_core_domain::{CommitRecord, RefName, RepoRoot};

use super::GitCliReader;
use super::support::{failed, parse_error};
use crate::cli::{GitCommand, Invocation};
use crate::parse::log::{LOG_DATE, LOG_FORMAT, parse_log};

/// Lee el log que pide `query`.
///
/// Sin filtro, solo las refs que el grafo sabe etiquetar (las mismas que
/// `read_refs`) mas HEAD, para que un HEAD separado siga saliendo. Con `--all`
/// salian como ramas sin nombre `refs/original` (filter-branch), `refs/prefetch`
/// (git maintenance), `refs/jj`, `refs/pull` o `refs/bisect`, y tras un
/// filter-branch el historial viejo entero aparecia duplicado. Stash y notas
/// tampoco son historia del proyecto. `--ignore-missing` evita que un HEAD sin
/// nacer (rama huerfana en un repo con commits) haga fallar el log. Los HEAD
/// separados de otros worktrees quedan fuera a proposito hasta que los worktrees
/// tengan sus propias refs.
///
/// Con ramas ocultas o aisladas, las refs van por stdin (`--stdin`): un
/// repositorio con miles de ramas no cabe en la linea de comandos de Windows.
pub(super) async fn read_log(
    cli: &GitCliReader,
    repo: &RepoRoot,
    query: &LogQuery,
) -> Result<Vec<CommitRecord>, GitReadError> {
    let mut command = GitCommand::new("log").arg("--topo-order");
    let stdin = match &query.revisions {
        LogRevisions::All => {
            command = command.arg("--branches").arg("--remotes").arg("--tags");
            None
        }
        LogRevisions::Refs {
            names,
            include_head,
        } => {
            let mut lines: Vec<&str> = names.iter().map(RefName::as_str).collect();
            if *include_head {
                lines.push("HEAD");
            }
            // Sin revisiones, git log recorre HEAD: aislar algo que no existe
            // tiene que dar un grafo vacio, no el de HEAD.
            if lines.is_empty() {
                return Ok(Vec::new());
            }
            command = command.arg("--stdin");
            Some(lines.join("\n") + "\n")
        }
    };
    command = command
        .arg("--ignore-missing")
        .arg("-z")
        .arg("--no-color")
        .arg("--no-show-signature")
        .arg("--encoding=UTF-8")
        .arg(LOG_DATE)
        .arg(LOG_FORMAT);
    if query.skip > 0 {
        command = command.number("--skip=", query.skip);
    }
    if let Some(max) = query.max_count {
        command = command.max_count(max);
    }
    if matches!(query.revisions, LogRevisions::All) {
        // Tiene que ir lo ultimo: revisions() escribe --end-of-options y lo que
        // venga detras ya no se lee como opcion.
        command = command.revisions(["HEAD"]);
    }
    let call = Invocation {
        stdin: stdin.as_deref().map(str::as_bytes),
        ..Invocation::new(Some(repo.as_path()), &command)
    };
    let output = cli.invoke(call).await?;
    if !output.success() {
        return Err(failed("log", &output));
    }
    parse_log(&output.stdout).map_err(|error| parse_error(&error))
}
