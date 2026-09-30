//! Sidecar de la extension de VS Code: sirve el motor de Git por JSON-RPC sobre
//! stdio. La extension lo arranca como proceso hijo.

// Clippy (nursery) propone `const fn` para un main vacio al compilar el arnes de
// tests, pero Rust no deja que main sea const. El allow se va con el stub.
#[allow(
    clippy::missing_const_for_fn,
    reason = "main cannot be const; this stub is empty until the binary is implemented"
)]
fn main() {}
