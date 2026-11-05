//! `NotifyWatcher`: vigila el arbol de trabajo y `.git` y avisa con
//! `repo/changed` cuando el usuario trabaja desde una terminal.
//!
//! - Las rafagas se agrupan: tras el primer evento se espera a que haya
//!   [`QUIET`] sin eventos, y nunca mas de [`MAX_WAIT`]. Un commit toca media
//!   docena de ficheros de `.git` y sale un solo aviso, en menos de un segundo.
//! - Dentro de `.git` solo cuentan HEAD, el indice, las refs y los ficheros de
//!   una operacion a medias; los objetos, los logs y los `.lock` no.
//! - Fuera de `.git`, lo que ignora `.gitignore` no cuenta: se pregunta a `git
//!   check-ignore`, que no toma bloqueos.
//!
//! Vigilar no necesita runtime al construirse: la tarea que agrupa se lanza en el
//! runtime de quien abre el repositorio (el del host).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use git_rudder_core_application::{EngineEvent, EventSink, RepoWatcher, WatchError};
use git_rudder_core_domain::{RepoId, RepoRoot};
use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
use tokio::sync::mpsc;

use crate::cli::{GitCommand, Invocation};
use crate::reader::GitCliReader;

/// Silencio que cierra una rafaga.
pub const QUIET: Duration = Duration::from_millis(120);

/// Espera maxima desde el primer evento de una rafaga.
pub const MAX_WAIT: Duration = Duration::from_millis(500);

/// Ficheros de `.git` que cambian lo que ensena la interfaz.
const GIT_FILES: &[&str] = &[
    "HEAD",
    "index",
    "packed-refs",
    "ORIG_HEAD",
    "MERGE_HEAD",
    "MERGE_MSG",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "REBASE_HEAD",
];

/// Carpetas de `.git` cuyo contenido cambia lo que ensena la interfaz.
const GIT_DIRS: &[&str] = &["refs", "rebase-merge", "rebase-apply", "sequencer"];

/// Vigilante sobre `notify` (inotify, FSEvents o ReadDirectoryChangesW).
pub struct NotifyWatcher {
    git: Arc<GitCliReader>,
    sink: Arc<dyn EventSink>,
    watching: Mutex<HashMap<RepoId, RecommendedWatcher>>,
}

impl std::fmt::Debug for NotifyWatcher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self
            .watching
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len();
        formatter
            .debug_struct("NotifyWatcher")
            .field("watching", &count)
            .finish_non_exhaustive()
    }
}

impl NotifyWatcher {
    /// Vigilante que avisa en `sink` y pregunta a `git` que se ignora.
    #[must_use]
    pub fn new(git: Arc<GitCliReader>, sink: Arc<dyn EventSink>) -> Self {
        Self {
            git,
            sink,
            watching: Mutex::new(HashMap::new()),
        }
    }
}

impl RepoWatcher for NotifyWatcher {
    fn watch(&self, repo_id: &RepoId, root: &RepoRoot) -> Result<(), WatchError> {
        let mut watching = self.watching.lock().unwrap_or_else(PoisonError::into_inner);
        if watching.contains_key(repo_id) {
            return Ok(());
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|error| WatchError(format!("no async runtime to watch with: {error}")))?;
        let (sender, receiver) = mpsc::unbounded_channel::<PathBuf>();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let paths = event
                    .ok()
                    .filter(|event| is_change(event.kind))
                    .map(|event| event.paths)
                    .unwrap_or_default();
                for path in paths {
                    let _ = sender.send(path);
                }
            })
            .map_err(|error| watch_failed(root, &error))?;
        watcher
            .watch(root.as_path(), RecursiveMode::Recursive)
            .map_err(|error| watch_failed(root, &error))?;
        let task = Debouncer {
            repo_id: repo_id.clone(),
            root: root.clone(),
            git: Arc::clone(&self.git),
            sink: Arc::clone(&self.sink),
        };
        runtime.spawn(task.run(receiver));
        watching.insert(repo_id.clone(), watcher);
        drop(watching);
        Ok(())
    }
}

fn watch_failed(root: &RepoRoot, error: &notify::Error) -> WatchError {
    tracing::warn!(root = %root.as_path().display(), %error, "could not watch the repository");
    WatchError(error.to_string())
}

/// La tarea que agrupa los eventos de un repositorio.
struct Debouncer {
    repo_id: RepoId,
    root: RepoRoot,
    git: Arc<GitCliReader>,
    sink: Arc<dyn EventSink>,
}

impl Debouncer {
    async fn run(self, mut receiver: mpsc::UnboundedReceiver<PathBuf>) {
        while let Some(first) = receiver.recv().await {
            let burst = collect_burst(first, &mut receiver).await;
            if self.matters(burst).await {
                self.sink.publish(EngineEvent::RepoChanged {
                    repo_id: self.repo_id.clone(),
                });
            }
        }
    }

    /// Algo de la rafaga cambia lo que ensena la interfaz.
    async fn matters(&self, burst: HashSet<PathBuf>) -> bool {
        let git_dir = self.root.as_path().join(".git");
        let (inside, outside): (Vec<PathBuf>, Vec<PathBuf>) = burst
            .into_iter()
            .partition(|path| path.starts_with(&git_dir));
        if inside
            .iter()
            .any(|path| is_relevant_git_path(&git_dir, path))
        {
            return true;
        }
        !outside.is_empty() && self.any_not_ignored(&outside).await
    }

    /// `git check-ignore --stdin -z` devuelve los ignorados; si alguno no lo esta,
    /// el cambio cuenta. Si git falla, se avisa: refrescar de mas no rompe nada.
    async fn any_not_ignored(&self, paths: &[PathBuf]) -> bool {
        let mut input = Vec::new();
        for path in paths {
            input.extend_from_slice(path.as_os_str().as_encoded_bytes());
            input.push(0);
        }
        let command = GitCommand::new("check-ignore").arg("--stdin").arg("-z");
        let call = Invocation {
            stdin: Some(&input),
            untrusted: true,
            // check-ignore no admite magia de pathspec, ni siquiera `literal`.
            literal_pathspecs: false,
            ..Invocation::new(Some(self.root.as_path()), &command)
        };
        let Ok(output) = self.git.invoke(call).await else {
            return true;
        };
        let ignored = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|path| !path.is_empty())
            .count();
        !matches!(output.code, Some(0 | 1)) || ignored < paths.len()
    }
}

/// Junta los eventos hasta que haya un silencio o se cumpla la espera maxima.
async fn collect_burst(
    first: PathBuf,
    receiver: &mut mpsc::UnboundedReceiver<PathBuf>,
) -> HashSet<PathBuf> {
    let deadline = tokio::time::Instant::now() + MAX_WAIT;
    let mut burst = HashSet::from([first]);
    loop {
        let wait = QUIET.min(deadline.saturating_duration_since(tokio::time::Instant::now()));
        match tokio::time::timeout(wait, receiver.recv()).await {
            Ok(Some(path)) => {
                burst.insert(path);
            }
            Ok(None) | Err(_) => return burst,
        }
    }
}

/// Crear, cambiar o borrar. Leer un fichero o una carpeta tambien genera eventos
/// (git lee `.git` todo el rato, el propio motor incluido), y cambiar solo la
/// fecha de acceso no cambia nada que se ensene.
const fn is_change(kind: EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Remove(_) | EventKind::Any
    ) || matches!(kind, EventKind::Modify(modify) if !matches!(modify, ModifyKind::Metadata(_)))
}

/// HEAD, el indice, las refs o una operacion a medias; nunca un `.lock`.
fn is_relevant_git_path(git_dir: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(git_dir) else {
        return false;
    };
    if path
        .extension()
        .is_some_and(|extension| extension == "lock")
    {
        return false;
    }
    let mut parts = relative.components();
    let Some(first) = parts.next().and_then(|part| part.as_os_str().to_str()) else {
        return false;
    };
    GIT_DIRS.contains(&first) || (GIT_FILES.contains(&first) && parts.next().is_none())
}

#[cfg(test)]
mod tests;
