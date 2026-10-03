//! Arranque del servidor HTTP y apagado ordenado.
//!
//! Vive aqui y no en el binario para que Salvo no salga de este crate: el
//! composition root pasa las dependencias, la direccion y la senal de apagado, y
//! no necesita conocer ni un tipo del framework.

use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;

use salvo::conn::Listener;
use salvo::conn::tcp::TcpAcceptor;
use salvo::prelude::{Server, TcpListener};
use thiserror::Error;

use crate::router::{RouterDependencies, build_service};

/// Fallo al arrancar el servidor.
#[derive(Debug, Error)]
pub enum ServeError {
    /// No se pudo abrir el puerto de escucha.
    #[error("could not bind {address}: {source}")]
    Bind {
        /// Direccion que se intento abrir.
        address: SocketAddr,
        /// Error original del listener, en `Box` para no inflar el `Result`.
        #[source]
        source: Box<salvo::Error>,
    },
}

/// Escucha en `address` hasta que `shutdown` termine.
///
/// Cuando `shutdown` termina, el servidor deja de aceptar conexiones y da hasta
/// `grace` a las peticiones en vuelo antes de cortar: un despliegue no deja a
/// medias una peticion que ya estaba dentro.
///
/// # Errors
///
/// Devuelve [`ServeError::Bind`] si el puerto no se puede abrir.
pub async fn serve<F>(
    dependencies: RouterDependencies,
    address: SocketAddr,
    shutdown: F,
    grace: Duration,
) -> Result<(), ServeError>
where
    F: Future<Output = ()> + Send + 'static,
{
    let service = build_service(&dependencies);
    let server = Server::new(bind(address).await?);
    let handle = server.handle();
    tokio::spawn(async move {
        shutdown.await;
        handle.stop_graceful(grace);
    });
    log::listening(address);
    server.serve(service).await;
    log::stopped();
    Ok(())
}

async fn bind(address: SocketAddr) -> Result<TcpAcceptor, ServeError> {
    TcpListener::new(address)
        .try_bind()
        .await
        .map_err(|source| ServeError::Bind {
            address,
            source: Box::new(source),
        })
}

/// Trazas del servidor. Cada macro de `tracing` expande a bastante codigo y,
/// mezclada con la logica, dispara el limite de complejidad cognitiva.
mod log {
    use std::net::SocketAddr;

    pub fn listening(address: SocketAddr) {
        tracing::info!(%address, "git-rudder api is listening");
    }

    pub fn stopped() {
        tracing::info!("git-rudder api stopped");
    }
}
