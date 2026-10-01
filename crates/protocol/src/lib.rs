//! Contrato entre las interfaces y el motor: nombres de metodo, DTOs de parametros
//! y resultados, y el sobre JSON-RPC 2.0.
//!
//! No depende de ningun otro crate de Ramal a proposito: si importara el dominio,
//! cambiar un tipo interno cambiaria el contrato sin que nadie lo viera. El mapeo
//! entre dominio y DTO vive en `ramal-engine`. Los tipos no llevan sufijo `Dto`:
//! son exactamente los nombres que ve TypeScript, que los recibe generados con
//! ts-rs en `packages/bridge/src/generated` (`just protocol-gen`).
//!
//! Reglas de forma: campos en camelCase, parametros con `deny_unknown_fields`,
//! opcionales con `skip_serializing_if` mas `#[ts(optional)]`, y ningun entero de
//! 64 bits (ts-rs los traduce a `bigint` y romperian el JSON normal).

pub mod engine;
pub mod envelope;
pub mod graph;
pub mod methods;
pub mod repo;

pub use envelope::{
    ErrorKind, JsonRpcVersion, RequestId, RpcError, RpcErrorData, RpcNotification, RpcOutcome,
    RpcRequest, RpcResponse, error_code,
};
pub use methods::{METHODS, MethodInfo, MethodVisitor, PROTOCOL_VERSION, RpcMethod};
