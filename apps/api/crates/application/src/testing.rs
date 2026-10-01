//! Dobles de test de los puertos de la aplicacion.
//!
//! Solo se compila con la feature `testing` o dentro de los tests de este crate.
//! Permite ejercitar los casos de uso y el adaptador HTTP sin base de datos, sin
//! red y sin esperar a que pase el tiempo. Cada puerto nuevo trae aqui su doble.

mod block_on;
mod fake_health_probe;
mod fixed_clock;

pub use block_on::block_on;
pub use fake_health_probe::FakeHealthProbe;
pub use fixed_clock::FixedClock;
