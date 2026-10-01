//! Capa de aplicacion de la API de Ramal.
//!
//! Declara los puertos que necesita el nucleo y los casos de uso que los
//! orquestan. No conoce Salvo, SQLx ni ningun runtime: solo `async-trait`, para
//! poder guardar los puertos como `Arc<dyn Puerto>`, y `chrono` para hablar de
//! instantes. Los tests corren con [`testing::block_on`], sin tokio.
//!
//! Los casos de uso que pueden fallar devuelven [`AppError`], y el adaptador HTTP
//! lo traduce una sola vez a `application/problem+json`.

pub mod error;
pub mod port;
pub mod use_case;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use error::{AppError, FieldError, ValidationErrors};
pub use port::{Clock, HealthProbe, ProbeError};
pub use use_case::health::{CheckLiveness, CheckReadiness, Probes};
