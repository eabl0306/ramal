//! Host de Git Rudder Desktop sobre Tauri 2: adaptador driving del motor de Git y raiz
//! de composicion exterior del cliente de escritorio.
//!
//! El motor vive en este mismo proceso y el webview solo llega a el por el comando
//! `rpc`, que filtra cada peticion con la lista de metodos de su ventana. Abrir un
//! repositorio es otro comando, `repo_pick_and_open`, que ensena el dialogo nativo
//! desde Rust: el webview pinta contenido que no es de fiar y nunca manda rutas.

pub mod bootstrap;
pub mod commands;
pub mod picker;
pub mod surfaces;

pub use bootstrap::{context, run};

// `unused_crate_dependencies` se evalua por target, y el de los tests unitarios
// recibe tambien las dev-dependencies que solo usan los tests de `tests/`.
#[cfg(test)]
use tempfile as _;
