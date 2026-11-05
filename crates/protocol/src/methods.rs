//! Tabla de metodos del motor y version del protocolo.
//!
//! ts-rs genera tipos pero no valores ni el mapa de metodos. Por eso cada metodo
//! se declara una sola vez en `define_methods!`, y de esa declaracion salen el
//! tipo marcador para el dispatch de Rust, la lista [`METHODS`] y, por el test
//! `tests/export_bindings.rs`, el `methods.ts` de TypeScript (`EngineMethods`,
//! `ENGINE_METHODS`, `HOST_METHODS` y `PROTOCOL_VERSION`).

use serde::Serialize;
use serde::de::DeserializeOwned;
use ts_rs::TS;

use crate::commit::{CommitDetail, CommitDetailParams};
use crate::diff::{DiffFileParams, FileDiff};
use crate::engine::{EngineInfo, EngineInfoParams, GitInfo};
use crate::graph::{GraphWindow, GraphWindowParams};
use crate::history::{
    Blame, BlameParams, CompareParams, Comparison, FileHistoryPage, FileHistoryParams,
    GraphSearchParams, GraphSearchResult,
};
use crate::inventory::{
    ConflictFile, RepoPathParams, StashContents, StashList, StashShowParams, SubmoduleList,
};
use crate::refs::RefListing;
use crate::repo::{OpenRepoParams, RepoHandle};
use crate::repos::{
    ConfigureGitParams, RepoSummary, RepoSummaryParams, RepoTrusted, ScanReposParams,
    ScanReposResult,
};
use crate::status::{RepoParams, WorkingTreeStatus};

/// Version del protocolo. Sube con cualquier cambio incompatible. El extension
/// host la compara al arrancar el sidecar con la que trae generada.
///
/// 2: `graph/window` lleva `snapshot` y puede responder `stale_window`. Un motor
/// de la version 1 rechazaria el parametro nuevo por `deny_unknown_fields`.
///
/// 3: las lecturas del repositorio (estado, diffs, refs, historia, stashes,
/// submodulos y conflictos), el filtro de `graph/window`, la confianza en
/// `repo/open`, la notificacion `repo/changed` y `$/cancel`.
pub const PROTOCOL_VERSION: u32 = 3;

/// Un metodo del motor con sus tipos.
pub trait RpcMethod {
    /// Nombre `<area>/<verbo>`.
    const NAME: &'static str;
    /// Solo lo puede llamar un host (lleva rutas absolutas o secretos). Los hosts
    /// lo rechazan si la peticion viene de un webview.
    const HOST_ONLY: bool;
    /// Parametros.
    type Params: Serialize + DeserializeOwned + TS + 'static;
    /// Resultado.
    type Result: Serialize + DeserializeOwned + TS + 'static;
}

/// Recorre los metodos con sus tipos (para generar TypeScript o listas).
pub trait MethodVisitor {
    /// Se llama una vez por metodo, en orden de declaracion.
    fn visit<M: RpcMethod>(&mut self);
}

/// Nombre y alcance de un metodo, sin sus tipos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodInfo {
    /// Nombre `<area>/<verbo>`.
    pub name: &'static str,
    /// Solo lo puede llamar un host.
    pub host_only: bool,
}

impl MethodInfo {
    /// Informacion del metodo `M`.
    #[must_use]
    pub const fn of<M: RpcMethod>() -> Self {
        Self {
            name: M::NAME,
            host_only: M::HOST_ONLY,
        }
    }

    /// Busca un metodo por nombre.
    #[must_use]
    pub fn find(name: &str) -> Option<Self> {
        METHODS.iter().copied().find(|method| method.name == name)
    }
}

/// Declara cada metodo una vez: nombre, marcador, parametros, resultado y alcance.
macro_rules! define_methods {
    ($(
        $(#[$doc:meta])*
        $name:literal => $marker:ident($params:ty) -> $result:ty, host_only = $host:literal;
    )+) => {
        $(
            $(#[$doc])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct $marker;

            impl RpcMethod for $marker {
                const NAME: &'static str = $name;
                const HOST_ONLY: bool = $host;
                type Params = $params;
                type Result = $result;
            }
        )+

        /// Todos los metodos, en orden de declaracion.
        pub const METHODS: &[MethodInfo] = &[$(MethodInfo::of::<$marker>()),+];

        /// Visita cada metodo con sus tipos.
        pub fn visit_methods(visitor: &mut impl MethodVisitor) {
            $(visitor.visit::<$marker>();)+
        }
    };
}

define_methods! {
    /// Version del motor y de git. Los hosts la piden al arrancar.
    "engine/info" => EngineInfoMethod(EngineInfoParams) -> EngineInfo, host_only = false;
    /// Cambia el ejecutable de git y vuelve a comprobarlo. Decide que se ejecuta.
    "git/configure" => GitConfigureMethod(ConfigureGitParams) -> GitInfo, host_only = true;
    /// Abre un repositorio por ruta. Solo los hosts: un webview nunca manda rutas.
    "repo/open" => RepoOpenMethod(OpenRepoParams) -> RepoHandle, host_only = true;
    /// El usuario confia en un repositorio abierto. Solo los hosts, tras su propia
    /// confirmacion: si un webview pudiera, la pregunta no protegeria de nada.
    "repo/trust" => RepoTrustMethod(RepoParams) -> RepoTrusted, host_only = true;
    /// Estado de fondo de un repositorio de la lista, sin abrirlo.
    "repos/summary" => ReposSummaryMethod(RepoSummaryParams) -> RepoSummary, host_only = true;
    /// Repositorios dentro de una carpeta.
    "repos/scan" => ReposScanMethod(ScanReposParams) -> ScanReposResult, host_only = true;
    /// Abre como repositorio un submodulo inicializado de uno abierto. Lo abre el
    /// host, que es quien ensena el repositorio nuevo.
    "submodule/open" => SubmoduleOpenMethod(RepoPathParams) -> RepoHandle, host_only = true;
    /// Ventana del grafo de un repositorio abierto.
    "graph/window" => GraphWindowMethod(GraphWindowParams) -> GraphWindow, host_only = false;
    /// Busca en todo el historial por mensaje, autor o sha.
    "graph/search" => GraphSearchMethod(GraphSearchParams) -> GraphSearchResult, host_only = false;
    /// Estado del arbol de trabajo.
    "status/read" => StatusReadMethod(RepoParams) -> WorkingTreeStatus, host_only = false;
    /// Diff de un fichero.
    "diff/file" => DiffFileMethod(DiffFileParams) -> FileDiff, host_only = false;
    /// Detalle de un commit.
    "commit/detail" => CommitDetailMethod(CommitDetailParams) -> CommitDetail, host_only = false;
    /// Ramas, tags y remotos.
    "refs/list" => RefsListMethod(RepoParams) -> RefListing, host_only = false;
    /// Stashes.
    "stash/list" => StashListMethod(RepoParams) -> StashList, host_only = false;
    /// Ficheros y diff de un stash.
    "stash/show" => StashShowMethod(StashShowParams) -> StashContents, host_only = false;
    /// Submodulos.
    "submodule/list" => SubmoduleListMethod(RepoParams) -> SubmoduleList, host_only = false;
    /// Commits que tocan un fichero.
    "history/file" => HistoryFileMethod(FileHistoryParams) -> FileHistoryPage, host_only = false;
    /// Autor y commit de cada linea.
    "blame/file" => BlameFileMethod(BlameParams) -> Blame, host_only = false;
    /// Compara dos refs.
    "compare/refs" => CompareRefsMethod(CompareParams) -> Comparison, host_only = false;
    /// Lados y bloques de un fichero en conflicto.
    "conflict/read" => ConflictReadMethod(RepoPathParams) -> ConflictFile, host_only = false;
}
