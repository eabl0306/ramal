//! `EngineBuilder` y `EngineConfig`: construyen los adaptadores que son iguales en
//! los dos productos.
//!
//! La composicion tiene dos niveles. Este es el de dentro: git CLI, vigilancia
//! del disco, busqueda de repositorios, confianza en un fichero y, mas adelante,
//! plataformas, IA y Git Rudder API. Cada host es la raiz de fuera: lee su
//! configuracion (rutas, binario de git, carpeta de datos) y aporta lo que solo
//! el sabe construir (el `NotificationSink`, el `SecretStore`). Nadie mas
//! construye adaptadores.

use std::path::PathBuf;
use std::sync::Arc;

use git_rudder_core_application::{
    BACKGROUND_STATUS_CONCURRENCY, BlameFile, CompareRefs, ConfigureGit, ConflictReader,
    DescribeEngine, FileHistory, GitReader, GitSetup, GraphCache, HistoryReader, InventoryReader,
    ListRefs, ListStashes, ListSubmodules, LoadGraphWindow, OpenDependencies, OpenRepositories,
    OpenRepository, OpenSubmodule, ReadCommitDetail, ReadConflict, ReadFileDiff, ReadStatus,
    RepoInspector, RepoWatcher, ScanRepositories, SearchHistory, ShowStash, StatusReader,
    SummarizeRepository, TrustRepository, TrustStore,
};
use git_rudder_git::{FsScanner, GitCliConfig, GitCliReader, NotifyWatcher};

use crate::adapters::trust_store::FileTrustStore;
use crate::engine::{Engine, UseCases};
use crate::host_ports::{NotificationSink, ProtocolEvents};

/// Version del motor, la del crate.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lo que el host sabe y el motor necesita.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineConfig {
    /// Ejecutable de git elegido por el usuario. En VS Code es el `git.path` del
    /// editor; sin el, el primer `git` del `PATH`.
    pub git_path: Option<PathBuf>,
    /// Fichero donde se recuerda en que repositorios confio el usuario, en la
    /// carpeta de datos del host. Sin el, la confianza se olvida al cerrar.
    pub trust_file: Option<PathBuf>,
}

/// Todos los puertos de lectura de un repositorio en un solo objeto: el CLI de
/// git en produccion y un doble en los tests.
pub trait RepoReaders:
    StatusReader + HistoryReader + InventoryReader + ConflictReader + RepoInspector
{
}

impl<T> RepoReaders for T where
    T: StatusReader + HistoryReader + InventoryReader + ConflictReader + RepoInspector
{
}

/// Construye un [`Engine`].
pub struct EngineBuilder {
    config: EngineConfig,
    git_reader: Option<Arc<dyn GitReader>>,
    repo_readers: Option<Arc<dyn RepoReaders>>,
    sink: Option<Arc<dyn NotificationSink>>,
}

impl std::fmt::Debug for EngineBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EngineBuilder")
            .field("config", &self.config)
            .field("custom_git_reader", &self.git_reader.is_some())
            .field("custom_repo_readers", &self.repo_readers.is_some())
            .field("notifications", &self.sink.is_some())
            .finish()
    }
}

impl EngineBuilder {
    /// Builder con la configuracion del host.
    #[must_use]
    pub const fn new(config: EngineConfig) -> Self {
        Self {
            config,
            git_reader: None,
            repo_readers: None,
            sink: None,
        }
    }

    /// Sustituye el lector de git del CLI (version, refs y log) por otro: un doble
    /// en los tests o, si un benchmark lo pide algun dia, `gix`.
    #[must_use]
    pub fn with_git_reader(mut self, reader: Arc<dyn GitReader>) -> Self {
        self.git_reader = Some(reader);
        self
    }

    /// Sustituye el resto de lecturas del CLI por otras (un doble en los tests).
    #[must_use]
    pub fn with_repo_readers(mut self, readers: Arc<dyn RepoReaders>) -> Self {
        self.repo_readers = Some(readers);
        self
    }

    /// Donde entrega el motor sus notificaciones. Sin el, no vigila el disco:
    /// no tendria a quien avisar.
    #[must_use]
    pub fn with_notification_sink(mut self, sink: Arc<dyn NotificationSink>) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Cablea adaptadores y casos de uso.
    #[must_use]
    pub fn build(self) -> Engine {
        let cli = Arc::new(GitCliReader::new(GitCliConfig {
            binary: self.config.git_path.clone(),
            ..GitCliConfig::default()
        }));
        let git: Arc<dyn GitReader> = self.git_reader.unwrap_or_else(|| cli.clone());
        let readers: Arc<dyn RepoReaders> = self.repo_readers.unwrap_or_else(|| cli.clone());
        let watcher = self.sink.map(|sink| {
            let events = Arc::new(ProtocolEvents::new(sink));
            Arc::new(NotifyWatcher::new(cli.clone(), events)) as Arc<dyn RepoWatcher>
        });
        let trust_store: Arc<dyn TrustStore> =
            Arc::new(FileTrustStore::new(self.config.trust_file));
        Engine::new(use_cases(&Ports {
            git,
            readers,
            setup: cli,
            trust_store,
            watcher,
        }))
    }
}

/// Los adaptadores ya construidos.
struct Ports {
    git: Arc<dyn GitReader>,
    readers: Arc<dyn RepoReaders>,
    setup: Arc<dyn GitSetup>,
    trust_store: Arc<dyn TrustStore>,
    watcher: Option<Arc<dyn RepoWatcher>>,
}

fn use_cases(ports: &Ports) -> UseCases {
    let registry = Arc::new(OpenRepositories::new());
    let cache = Arc::new(GraphCache::new());
    let git = &ports.git;
    let readers = &ports.readers;
    let open = Arc::new(OpenRepository::new(
        Arc::clone(git),
        Arc::clone(&registry),
        OpenDependencies {
            inspector: readers.clone(),
            trust_store: Arc::clone(&ports.trust_store),
            watcher: ports.watcher.clone(),
        },
    ));
    let history: Arc<dyn HistoryReader> = readers.clone();
    let inventory: Arc<dyn InventoryReader> = readers.clone();
    let status: Arc<dyn StatusReader> = readers.clone();
    let reg = || Arc::clone(&registry);
    UseCases {
        describe: DescribeEngine::new(Arc::clone(git), ENGINE_VERSION),
        configure_git: ConfigureGit::new(Arc::clone(&ports.setup)),
        trust: TrustRepository::new(Arc::clone(&ports.trust_store), reg()),
        open_submodule: OpenSubmodule::new(inventory.clone(), reg(), Arc::clone(&open)),
        open,
        summarize: SummarizeRepository::new(
            Arc::clone(git),
            status.clone(),
            Arc::clone(&ports.trust_store),
            BACKGROUND_STATUS_CONCURRENCY,
        ),
        scan: ScanRepositories::new(Arc::new(FsScanner)),
        graph: LoadGraphWindow::new(Arc::clone(git), reg(), Arc::clone(&cache)),
        search: SearchHistory::new(Arc::clone(git), history.clone(), reg(), cache),
        status: ReadStatus::new(status.clone(), reg()),
        diff: ReadFileDiff::new(status, reg()),
        commit: ReadCommitDetail::new(history.clone(), reg()),
        refs: ListRefs::new(inventory.clone(), reg()),
        stashes: ListStashes::new(inventory.clone(), reg()),
        stash: ShowStash::new(inventory.clone(), reg()),
        submodules: ListSubmodules::new(inventory, reg()),
        history: FileHistory::new(history.clone(), reg()),
        blame: BlameFile::new(history.clone(), reg()),
        compare: CompareRefs::new(history, reg()),
        conflict: ReadConflict::new(readers.clone(), reg()),
    }
}
