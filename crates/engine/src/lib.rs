//! Motor del cliente: adaptador driving (dispatch JSON-RPC) y composition root
//! compartido. Es lo unico que ven los dos hosts, el de Tauri y el sidecar de VS
//! Code, y por eso el protocolo es identico en ambos productos.
//!
//! Cada handler hace tres cosas y nada mas: deserializa los parametros
//! (`deny_unknown_fields`), llama al caso de uso y serializa el resultado. Una
//! regla de negocio dentro de un handler esta en el sitio equivocado.

mod builder;
mod dispatch;
mod engine;
pub mod mapping;

pub use builder::{ENGINE_VERSION, EngineBuilder, EngineConfig};
pub use engine::{CallContext, CallOrigin, Engine};

// `unused_crate_dependencies` se evalua por target, y el de los tests unitarios
// recibe tambien las dev-dependencies que solo usan los tests de `tests/`.
#[cfg(test)]
use {pretty_assertions as _, tempfile as _, tokio as _};
