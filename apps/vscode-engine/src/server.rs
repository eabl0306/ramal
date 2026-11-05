//! Bucle del servidor: una tarea por peticion y una unica tarea escritora en
//! stdout, para que dos respuestas nunca se mezclen.
//!
//! Termina cuando se cierra la entrada (lo normal: el extension host cerro stdin)
//! o con el primer marco invalido, que cierra la conexion. En los dos casos deja
//! terminar las peticiones en curso y escribe sus respuestas antes de salir.

use std::future::Future;
use std::io;
use std::sync::Arc;

use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::framing::{FrameError, FrameReader, write_frame};

/// Lo que hace el servidor con cada mensaje. Devuelve la respuesta serializada, o
/// `None` si no hay que responder (una notificacion).
pub trait MessageHandler: Send + Sync + 'static {
    /// Atiende un mensaje.
    fn handle(&self, message: Vec<u8>) -> impl Future<Output = Option<Vec<u8>>> + Send;
}

/// Por que se cerro la conexion.
#[derive(Debug, Error)]
pub enum ServeError {
    /// Llego un marco invalido.
    #[error("invalid frame: {0}")]
    Frame(#[from] FrameError),
    /// No se pudo escribir en la salida.
    #[error("could not write a response: {0}")]
    Write(#[from] io::Error),
    /// La tarea escritora termino de forma inesperada.
    #[error("the writer task stopped unexpectedly")]
    WriterStopped,
}

/// Lo que llega a la tarea escritora.
#[derive(Debug)]
enum Outgoing {
    /// Un mensaje ya serializado.
    Frame(Vec<u8>),
    /// No hay mas: la conexion se cierra. Hace falta porque las notificaciones
    /// del motor guardan un remitente, y el canal no se cerraria nunca solo.
    Close,
}

/// La cola de lo que se escribe en la salida: respuestas y notificaciones. Se
/// crea antes que el motor para que sus notificaciones salgan por la misma tarea
/// escritora que las respuestas, sin mezclarse.
#[derive(Debug)]
pub struct Outbox {
    sender: mpsc::UnboundedSender<Outgoing>,
    receiver: mpsc::UnboundedReceiver<Outgoing>,
}

impl Outbox {
    /// Cola vacia.
    #[must_use]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// Un remitente para meter mensajes ya serializados.
    #[must_use]
    pub fn sender(&self) -> OutboxSender {
        OutboxSender(self.sender.clone())
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Mete mensajes en la cola de salida.
#[derive(Debug, Clone)]
pub struct OutboxSender(mpsc::UnboundedSender<Outgoing>);

impl OutboxSender {
    /// Encola un mensaje. `false` si la conexion ya se cerro.
    #[must_use]
    pub fn send(&self, frame: Vec<u8>) -> bool {
        self.0.send(Outgoing::Frame(frame)).is_ok()
    }
}

/// Sirve el protocolo hasta que se cierra `reader`.
///
/// # Errors
///
/// [`ServeError`] si llega un marco invalido o la salida se cierra.
pub async fn serve<H, R, W>(handler: Arc<H>, reader: R, writer: W) -> Result<(), ServeError>
where
    H: MessageHandler,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin + Send + 'static,
{
    serve_with(handler, reader, writer, Outbox::new()).await
}

/// Como [`serve`], con una cola de salida que otros (las notificaciones del
/// motor) tambien usan.
///
/// # Errors
///
/// [`ServeError`] si llega un marco invalido o la salida se cierra.
pub async fn serve_with<H, R, W>(
    handler: Arc<H>,
    reader: R,
    writer: W,
    outbox: Outbox,
) -> Result<(), ServeError>
where
    H: MessageHandler,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin + Send + 'static,
{
    let sender = outbox.sender();
    let closer = outbox.sender;
    let writer_task = tokio::spawn(write_loop(writer, outbox.receiver));
    let mut frames = FrameReader::new(reader);
    let mut tasks = JoinSet::new();
    let outcome = loop {
        match frames.read_frame().await {
            Ok(Some(message)) => {
                tasks.spawn(answer(Arc::clone(&handler), message, sender.clone()));
                while tasks.try_join_next().is_some() {}
            }
            Ok(None) => break Ok(()),
            Err(error) => break Err(ServeError::Frame(error)),
        }
    };
    while let Some(joined) = tasks.join_next().await {
        if let Err(error) = joined {
            tracing::error!(%error, "a request task failed");
        }
    }
    // Lo que ya estaba en la cola se escribe antes del cierre.
    let _ = closer.send(Outgoing::Close);
    writer_task.await.map_err(|_| ServeError::WriterStopped)??;
    outcome
}

/// Atiende un mensaje y pasa la respuesta a la tarea escritora.
async fn answer<H: MessageHandler>(handler: Arc<H>, message: Vec<u8>, sender: OutboxSender) {
    if let Some(response) = handler.handle(message).await {
        // Si la escritora ya no esta, la salida se cerro: no hay a quien responder
        // y el error lo devuelve ella.
        let _ = sender.send(response);
    }
}

/// La unica tarea que escribe en la salida.
async fn write_loop<W: AsyncWrite + Unpin>(
    mut writer: W,
    mut receiver: mpsc::UnboundedReceiver<Outgoing>,
) -> io::Result<()> {
    while let Some(Outgoing::Frame(body)) = receiver.recv().await {
        write_frame(&mut writer, &body).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
