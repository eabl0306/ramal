//! Casos de uso y puertos del motor de Git. Solo conoce el dominio: el acceso real
//! al repositorio llega por los puertos que implementa `git-rudder-git`, y quien ejecuta
//! los futuros es el runtime del host (aqui no hay tokio).

pub mod error;
pub mod port;
pub mod state;
pub mod use_case;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use error::EngineError;
pub use port::git_reader::{
    GitCapabilities, GitReadError, GitReader, GitVersion, GitVersionInfo, LogQuery, LogRevisions,
    MINIMUM_GIT_VERSION,
};
pub use port::host::{
    EngineEvent, EventSink, GitSetup, RepoScanner, RepoWatcher, TrustStore, TrustStoreError,
    WatchError,
};
pub use port::repo_reader::{
    ConflictReader, DiffRequest, FoundCommit, HistoryReader, InventoryReader, RepoInspector,
    StatusReader,
};
pub use state::graph_cache::GraphCache;
pub use state::open_repositories::{OpenRepo, OpenRepositories};
pub use use_case::engine::{ConfigureGit, DescribeEngine, EngineDescription};
pub use use_case::graph::{LoadGraphWindow, MAX_SEARCH_RESULTS, SearchHistory, WindowRequest};
pub use use_case::reads::{
    BlameFile, CompareRefs, FileHistory, FileHistoryPage, ListRefs, ListStashes, ListSubmodules,
    ReadCommitDetail, ReadConflict, ReadFileDiff, ReadStatus, ShowStash,
};
pub use use_case::repo::{
    OpenDependencies, OpenRepository, OpenSubmodule, OpenedRepository, TrustRepository,
};
pub use use_case::repos::{
    BACKGROUND_STATUS_CONCURRENCY, MAX_SCAN_DEPTH, RepoSummary, ScanRepositories,
    SummarizeRepository,
};
