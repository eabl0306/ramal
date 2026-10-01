//! Middlewares comunes a todas las rutas.
//!
//! Los planificados (limite de ritmo, idempotencia) viven aqui tambien: repetir
//! una respuesta o rechazar un exceso es transporte, aunque su almacen sea un
//! puerto de la capa de aplicacion.

mod request_id;

pub use request_id::{REQUEST_ID_HEADER, REQUEST_ID_KEY, RequestId};
