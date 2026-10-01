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
    let (sender, receiver) = mpsc::unbounded_channel::<Vec<u8>>();
    let writer_task = tokio::spawn(write_loop(writer, receiver));
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
    drop(sender);
    writer_task.await.map_err(|_| ServeError::WriterStopped)??;
    outcome
}

/// Atiende un mensaje y pasa la respuesta a la tarea escritora.
async fn answer<H: MessageHandler>(
    handler: Arc<H>,
    message: Vec<u8>,
    sender: mpsc::UnboundedSender<Vec<u8>>,
) {
    if let Some(response) = handler.handle(message).await {
        // Si la escritora ya no esta, la salida se cerro: no hay a quien responder
        // y el error lo devuelve ella.
        let _ = sender.send(response);
    }
}

/// La unica tarea que escribe en la salida.
async fn write_loop<W: AsyncWrite + Unpin>(
    mut writer: W,
    mut receiver: mpsc::UnboundedReceiver<Vec<u8>>,
) -> io::Result<()> {
    while let Some(body) = receiver.recv().await {
        write_frame(&mut writer, &body).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
