//! Lado nativo de la app de escritorio (Tauri 2): expone el motor de Git al
//! frontend por los comandos de Tauri.

// Clippy (nursery) propone `const fn` para un main vacio al compilar el arnes de
// tests, pero Rust no deja que main sea const. El allow se va con el stub.
#[allow(
    clippy::missing_const_for_fn,
    reason = "main cannot be const; this stub is empty until the binary is implemented"
)]
fn main() {}
