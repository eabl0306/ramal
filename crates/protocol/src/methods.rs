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

use crate::engine::{EngineInfo, EngineInfoParams};
use crate::graph::{GraphWindow, GraphWindowParams};
use crate::repo::{OpenRepoParams, RepoHandle};

/// Version del protocolo. Sube con cualquier cambio incompatible. El extension
/// host la compara al arrancar el sidecar con la que trae generada.
///
/// 2: `graph/window` lleva `snapshot` y puede responder `stale_window`. Un motor
/// de la version 1 rechazaria el parametro nuevo por `deny_unknown_fields`.
pub const PROTOCOL_VERSION: u32 = 2;

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
    /// Abre un repositorio por ruta. Solo los hosts: un webview nunca manda rutas.
    "repo/open" => RepoOpenMethod(OpenRepoParams) -> RepoHandle, host_only = true;
    /// Ventana del grafo de un repositorio abierto.
    "graph/window" => GraphWindowMethod(GraphWindowParams) -> GraphWindow, host_only = false;
}
