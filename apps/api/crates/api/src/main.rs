//! Binario de la API: composition root. Cablea los adaptadores con los casos de uso
//! y arranca el servidor.

// Clippy (nursery) propone `const fn` para un main vacio al compilar el arnes de
// tests, pero Rust no deja que main sea const. El allow se va con el stub.
#[allow(
    clippy::missing_const_for_fn,
    reason = "main cannot be const; this stub is empty until the binary is implemented"
)]
fn main() {}
