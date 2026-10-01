//! Adaptador driving HTTP de la API de Ramal, sobre Salvo.
//!
//! Es el unico crate que conoce Salvo (ADR 0003). Traduce peticiones a casos de
//! uso y respuestas de dominio a DTOs; el dominio nunca se serializa directo,
//! para que el contrato publico no cambie sin querer.
//!
//! Piezas que conviene conocer antes de anadir un endpoint:
//!
//! - [`build_service`] monta el router, el documento OpenAPI, el middleware de
//!   request id y el catcher. Es lo que arranca el binario y lo que prueban los
//!   tests, para que los dos vean exactamente las mismas respuestas.
//! - [`problem::render_problem`] escribe cualquier `AppError` en
//!   `application/problem+json`. Un handler nunca escribe un error a mano.
//! - Cada handler es un struct con sus casos de uso detras de `Arc` y lleva
//!   `#[endpoint]`: la ruta es su documentacion, y `apps/api/openapi.json` sale
//!   de recorrer el router ([`openapi::document_json`]).

pub mod catcher;
pub mod dto;
pub mod handler;
pub mod middleware;
pub mod openapi;
pub mod problem;
pub mod router;
pub mod server;

pub use catcher::ProblemCatcher;
pub use openapi::{API_TITLE, document_json};
pub use problem::{PROBLEM_CONTENT_TYPE, Problem, problem_for, render_problem};
pub use router::{HealthDependencies, RouterDependencies, build_service};
pub use server::{ServeError, serve};
