//! Respuestas de error que no nacen de un caso de uso.
//!
//! Los errores de negocio los escribe [`crate::problem::render_problem`], pero la
//! ruta que no existe y el metodo que no se admite nunca llegan a un handler.
//! Salvo los resuelve con su pagina por defecto, que en la practica sale en HTML.
//! Aqui se cambia esa pagina por el mismo `application/problem+json` que usa el
//! resto del contrato: es lo que mira la prueba de humo del gateway para saber
//! que `/v1` llega a la API y no a la web.
//!
//! Salvo solo llama al catcher cuando la respuesta tiene estado de error y no
//! tiene cuerpo, asi que no puede pisar lo que haya escrito un handler.

use salvo::catcher::Catcher;
use salvo::prelude::{Depot, FlowCtrl, Handler, Request, Response, StatusCode};

use crate::problem::{problem_for_status, write_problem};

/// Convierte en problema los errores que genera el propio transporte.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProblemCatcher;

impl ProblemCatcher {
    /// Crea el catcher.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Catcher listo para `Service::catcher`.
    #[must_use]
    pub fn catcher() -> Catcher {
        Catcher::default().hoop(Self::new())
    }
}

#[salvo::async_trait]
impl Handler for ProblemCatcher {
    async fn handle(
        &self,
        _req: &mut Request,
        _depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let status = res.status_code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        write_problem(res, &problem_for_status(status));
        // Nadie mas toca esta respuesta: el catcher por defecto de Salvo
        // escribiria su pagina HTML encima.
        ctrl.skip_rest();
    }
}
