//! `OpenRepository`: descubre la raiz del repositorio que contiene una ruta,
//! decide si se confia en el, lo registra, empieza a vigilarlo y emite su
//! `RepoId`.
//!
//! Aqui se aplica el git minimo: antes de tocar un repositorio se comprueba la
//! version, y un git anterior a 2.34 da `GitVersionUnsupported` en vez de fallos
//! raros mas adelante. Solo lo llaman los hosts: la ruta nunca viene de un webview.
//!
//! La confianza (ADR 0020): si el usuario ya confio en la raiz en este
//! dispositivo, o el host responde por ella (una carpeta de confianza de VS
//! Code), se confia. Si no, se mira la configuracion local y los hooks sin
//! ejecutar nada; sin nada que pueda lanzar un comando no hay nada que preguntar,
//! y con algo, el repositorio se abre sin confianza y la interfaz pregunta
//! ensenando lo que se encontro. Descubrir la raiz y leer las refs no ejecuta
//! nada de la configuracion, asi que se puede hacer antes de decidir.

use std::path::Path;
use std::sync::Arc;

use git_rudder_core_domain::trust::TrustFinding;
use git_rudder_core_domain::{RefSnapshot, RepoId, RepoRoot, TrustLevel};

use crate::error::EngineError;
use crate::port::git_reader::{GitReader, MINIMUM_GIT_VERSION};
use crate::port::host::{RepoWatcher, TrustStore};
use crate::port::repo_reader::RepoInspector;
use crate::state::open_repositories::OpenRepositories;

/// Repositorio recien abierto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenedRepository {
    /// Id con el que la interfaz lo pide a partir de ahora.
    pub id: RepoId,
    /// Raiz del arbol de trabajo.
    pub root: RepoRoot,
    /// Refs y HEAD en el momento de abrirlo.
    pub refs: RefSnapshot,
    /// Si se confia en el.
    pub trust: TrustLevel,
    /// Lo que obliga a preguntar, si no se confia.
    pub findings: Vec<TrustFinding>,
}

/// Lo que necesita `OpenRepository` ademas de git.
#[derive(Clone)]
pub struct OpenDependencies {
    /// Mira la configuracion sin ejecutarla.
    pub inspector: Arc<dyn RepoInspector>,
    /// Confianza recordada en este dispositivo.
    pub trust_store: Arc<dyn TrustStore>,
    /// Vigila el repositorio, si el host quiere notificaciones.
    pub watcher: Option<Arc<dyn RepoWatcher>>,
}

impl std::fmt::Debug for OpenDependencies {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenDependencies")
            .field("watching", &self.watcher.is_some())
            .finish_non_exhaustive()
    }
}

/// Abre un repositorio.
pub struct OpenRepository {
    git: Arc<dyn GitReader>,
    registry: Arc<OpenRepositories>,
    deps: OpenDependencies,
}

impl std::fmt::Debug for OpenRepository {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpenRepository")
            .field("registry", &self.registry)
            .field("deps", &self.deps)
            .finish_non_exhaustive()
    }
}

impl OpenRepository {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(
        git: Arc<dyn GitReader>,
        registry: Arc<OpenRepositories>,
        deps: OpenDependencies,
    ) -> Self {
        Self {
            git,
            registry,
            deps,
        }
    }

    /// Abre el repositorio que contiene `path`. `vouched` dice que el host ya
    /// responde por su confianza.
    ///
    /// # Errors
    ///
    /// [`EngineError::GitUnavailable`] sin git, [`EngineError::GitVersionUnsupported`]
    /// con un git anterior al minimo, [`EngineError::NotARepository`] si la ruta no
    /// esta en un repositorio y [`EngineError::GitFailed`] si git falla.
    pub async fn execute(
        &self,
        path: &Path,
        vouched: bool,
    ) -> Result<OpenedRepository, EngineError> {
        let git = self.git.version().await?;
        if !git.is_supported() {
            return Err(EngineError::GitVersionUnsupported {
                found: git.version,
                minimum: MINIMUM_GIT_VERSION,
            });
        }
        let root = self.git.discover(path).await?;
        let refs = self.git.read_refs(&root).await?;
        let (trust, findings) = self.decide_trust(&root, vouched).await?;
        let id = self.registry.register_with(root.clone(), trust);
        if let Some(watcher) = &self.deps.watcher {
            // Sin vigilancia la interfaz sigue funcionando: refresca al volver
            // el foco. El adaptador deja el motivo en el log.
            let _ = watcher.watch(&id, &root);
        }
        Ok(OpenedRepository {
            id,
            root,
            refs,
            trust,
            findings,
        })
    }

    async fn decide_trust(
        &self,
        root: &RepoRoot,
        vouched: bool,
    ) -> Result<(TrustLevel, Vec<TrustFinding>), EngineError> {
        // Un almacen que no se puede leer cuenta como "no confio todavia": lo
        // seguro es mirar la configuracion.
        let remembered = self
            .deps
            .trust_store
            .is_trusted(root)
            .await
            .unwrap_or(false);
        if vouched || remembered {
            return Ok((TrustLevel::Trusted, Vec::new()));
        }
        let findings = self.deps.inspector.inspect(root).await?.findings();
        let trust = if findings.is_empty() {
            TrustLevel::Trusted
        } else {
            TrustLevel::Untrusted
        };
        Ok((trust, findings))
    }
}

#[cfg(test)]
mod tests;
