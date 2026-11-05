//! Motor del cliente: adaptador driving (dispatch JSON-RPC) y composition root
//! compartido. Es lo unico que ven los dos hosts, el de Tauri y el sidecar de VS
//! Code, y por eso el protocolo es identico en ambos productos.
//!
//! Cada handler hace tres cosas y nada mas: deserializa los parametros
//! (`deny_unknown_fields`), llama al caso de uso y serializa el resultado. Una
//! regla de negocio dentro de un handler esta en el sitio equivocado.

pub mod adapters;
mod builder;
mod cancel;
mod dispatch;
mod engine;
pub mod host_ports;
pub mod mapping;

pub use builder::{ENGINE_VERSION, EngineBuilder, EngineConfig, RepoReaders};
pub use engine::{CANCEL_NOTIFICATION, CallContext, CallOrigin, Engine};
pub use host_ports::NotificationSink;

// `unused_crate_dependencies` se evalua por target, y el de los tests unitarios
// recibe tambien las dev-dependencies que solo usan los tests de `tests/`.
#[cfg(test)]
use pretty_assertions as _;
