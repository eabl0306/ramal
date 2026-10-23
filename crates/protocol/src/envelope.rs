//! Sobre JSON-RPC 2.0: peticion, respuesta, error, notificacion y codigos.
//!
//! Es el mismo en los dos productos; solo cambia el transporte (`invoke` en
//! escritorio, marcos `Content-Length` por stdio en VS Code).

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

/// Codigos de error: los estandar de JSON-RPC, el de cancelacion de LSP y uno
/// propio para cualquier error del motor. La interfaz no decide por el codigo
/// sino por `error.data.kind`.
pub mod error_code {
    /// El texto no es JSON.
    pub const PARSE_ERROR: i32 = -32_700;
    /// El JSON no es una peticion valida.
    pub const INVALID_REQUEST: i32 = -32_600;
    /// El metodo no existe (o no se puede llamar desde ahi).
    pub const METHOD_NOT_FOUND: i32 = -32_601;
    /// Los parametros no encajan con el metodo.
    pub const INVALID_PARAMS: i32 = -32_602;
    /// Fallo interno del motor.
    pub const INTERNAL_ERROR: i32 = -32_603;
    /// Peticion cancelada (planificado; es el codigo de LSP).
    pub const REQUEST_CANCELLED: i32 = -32_800;
    /// Cualquier error del motor con su `kind`.
    pub const ENGINE_ERROR: i32 = -32_000;
}

/// Version del protocolo JSON-RPC. Solo existe `"2.0"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum JsonRpcVersion {
    /// JSON-RPC 2.0.
    #[default]
    #[serde(rename = "2.0")]
    V2,
}

/// Id de una peticion: entero no negativo menor que 2^31, para que sea un
/// `number` seguro en TypeScript. No se admiten ids de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, TS)]
#[ts(export)]
pub struct RequestId(u32);

impl RequestId {
    /// Mayor id admitido: 2^31 - 1.
    pub const MAX: u32 = 0x7FFF_FFFF;

    /// El id, si esta en rango.
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        if value <= Self::MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Valor numerico.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Serialize for RequestId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u32::deserialize(deserializer)?;
        Self::new(value).ok_or_else(|| D::Error::custom("request id must be below 2^31"))
    }
}

/// Peticion. Sin `id` es una notificacion y no recibe respuesta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcRequest {
    /// Siempre `"2.0"`.
    pub jsonrpc: JsonRpcVersion,
    /// Id con el que se correlaciona la respuesta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub id: Option<RequestId>,
    /// `<area>/<verbo>`, por ejemplo `graph/window`.
    pub method: String,
    /// Siempre un objeto; los metodos sin parametros reciben `{}`. Si falta, el
    /// motor lo toma como `{}`.
    #[serde(default = "empty_params")]
    pub params: serde_json::Value,
}

fn empty_params() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

/// Lo que la interfaz usa para decidir ante un error. Es una lista cerrada: un
/// `kind` nuevo sin traduccion en `@git-rudder/repo-view` no compila.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorKind {
    /// Parametros que no encajan con el metodo (o una peticion mal formada).
    InvalidParams,
    /// El `repoId` no esta abierto en el motor.
    RepoNotOpen,
    /// La ruta no esta dentro de un repositorio.
    NotARepository,
    /// No hay git instalado.
    GitNotFound,
    /// El git instalado es anterior al minimo.
    GitVersionUnsupported,
    /// git fallo; el detalle solo esta en el log del motor.
    GitFailed,
    /// La pagina pedida de `graph/window` llevaba el `snapshot` de unas refs que
    /// ya no son las del repositorio. La interfaz vuelve a pedir desde la fila 0.
    StaleWindow,
    /// Fallo inesperado del motor, o una peticion que no deberia existir.
    Internal,
}

/// Datos estables de un error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcErrorData {
    /// Tipo de error.
    pub kind: ErrorKind,
}

/// Error de una respuesta. `message` esta en ingles y es para el log; la interfaz
/// traduce `data.kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcError {
    /// Codigo JSON-RPC (ver [`error_code`]).
    pub code: i32,
    /// Mensaje para el log, en ingles.
    pub message: String,
    /// Tipo estable del error.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub data: Option<RpcErrorData>,
}

impl RpcError {
    /// Error con su tipo.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>, kind: ErrorKind) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(RpcErrorData { kind }),
        }
    }
}

/// Resultado o error de una respuesta: exactamente uno de los dos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum RpcOutcome {
    /// Respuesta correcta.
    Result(serde_json::Value),
    /// Respuesta con error.
    Error(RpcError),
}

/// Respuesta a una peticion con id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcResponse {
    /// Siempre `"2.0"`.
    pub jsonrpc: JsonRpcVersion,
    /// Id de la peticion, o `null` si no se pudo leer (JSON roto, id invalido).
    pub id: Option<RequestId>,
    /// `result` o `error`.
    #[serde(flatten)]
    pub outcome: RpcOutcome,
}

impl RpcResponse {
    /// Respuesta correcta.
    #[must_use]
    pub const fn success(id: Option<RequestId>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            outcome: RpcOutcome::Result(result),
        }
    }

    /// Respuesta con error.
    #[must_use]
    pub const fn failure(id: Option<RequestId>, error: RpcError) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            outcome: RpcOutcome::Error(error),
        }
    }
}

/// Notificacion: un mensaje sin id y sin respuesta (planificadas: `$/cancel`,
/// `$/progress`, `repo/changed`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcNotification {
    /// Siempre `"2.0"`.
    pub jsonrpc: JsonRpcVersion,
    /// Metodo de la notificacion.
    pub method: String,
    /// Siempre un objeto.
    #[serde(default = "empty_params")]
    pub params: serde_json::Value,
}

#[cfg(test)]
mod tests;
