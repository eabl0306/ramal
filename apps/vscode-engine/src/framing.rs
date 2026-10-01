//! Marcos `Content-Length` sobre stdin y stdout, como LSP.
//!
//! Un marco es `Content-Length: <bytes>\r\n\r\n` seguido de exactamente esos bytes
//! de JSON en UTF-8. Las demas cabeceras se ignoran. Un marco de mas de 32 MiB se
//! rechaza antes de reservar memoria, y quien llama cierra la conexion. Las
//! cabeceras tambien tienen tope: una linea sin fin no puede llenar la memoria.

use std::io;

use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

/// Tamano maximo del cuerpo de un marco.
pub const MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;

/// Longitud maxima de una linea de cabecera.
const MAX_HEADER_LINE: usize = 8 * 1024;

/// Numero maximo de lineas de cabecera en un marco.
const MAX_HEADERS: usize = 32;

/// Marco que no se puede leer. Cualquiera de estos cierra la conexion.
#[derive(Debug, Error)]
pub enum FrameError {
    /// El cuerpo anunciado pasa del limite.
    #[error("frame of {0} bytes exceeds the {MAX_FRAME_BYTES} byte limit")]
    TooLarge(usize),
    /// Las cabeceras no traen `Content-Length`.
    #[error("frame headers have no Content-Length")]
    MissingLength,
    /// `Content-Length` no es un numero.
    #[error("invalid Content-Length: {0}")]
    InvalidLength(String),
    /// Una linea de cabecera pasa del limite.
    #[error("frame header line is longer than {MAX_HEADER_LINE} bytes")]
    HeaderTooLong,
    /// Demasiadas lineas de cabecera.
    #[error("frame has more than {MAX_HEADERS} header lines")]
    TooManyHeaders,
    /// La entrada se cerro a mitad de un marco.
    #[error("input closed in the middle of a frame")]
    UnexpectedEof,
    /// Fallo de E/S.
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),
}

/// Lee marcos de un flujo.
#[derive(Debug)]
pub struct FrameReader<R> {
    reader: BufReader<R>,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    /// Lector sobre `reader`.
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
        }
    }

    /// Siguiente cuerpo, o `None` si la entrada se cerro limpiamente entre marcos.
    ///
    /// # Errors
    ///
    /// [`FrameError`] si las cabeceras no son validas, el marco es demasiado grande
    /// o la entrada se corta a mitad.
    pub async fn read_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        let Some(length) = self.read_headers().await? else {
            return Ok(None);
        };
        let mut body = vec![0; length];
        self.reader
            .read_exact(&mut body)
            .await
            .map_err(|error| match error.kind() {
                io::ErrorKind::UnexpectedEof => FrameError::UnexpectedEof,
                _ => FrameError::Io(error),
            })?;
        Ok(Some(body))
    }

    async fn read_headers(&mut self) -> Result<Option<usize>, FrameError> {
        let mut length = None;
        for index in 0..=MAX_HEADERS {
            match self.read_line().await? {
                None if index == 0 => return Ok(None),
                None => return Err(FrameError::UnexpectedEof),
                Some(line) if line.is_empty() => {
                    return length.map(Some).ok_or(FrameError::MissingLength);
                }
                Some(line) => {
                    length = content_length(&line)
                        .map(parse_length)
                        .transpose()?
                        .or(length);
                }
            }
        }
        Err(FrameError::TooManyHeaders)
    }

    /// Una linea sin `\r\n`, o `None` al final de la entrada.
    async fn read_line(&mut self) -> Result<Option<String>, FrameError> {
        let mut line = Vec::new();
        let cap = u64::try_from(MAX_HEADER_LINE).unwrap_or(u64::MAX) + 1;
        let read = (&mut self.reader)
            .take(cap)
            .read_until(b'\n', &mut line)
            .await?;
        if read == 0 {
            return Ok(None);
        }
        if line.last() != Some(&b'\n') {
            return Err(if line.len() > MAX_HEADER_LINE {
                FrameError::HeaderTooLong
            } else {
                FrameError::UnexpectedEof
            });
        }
        let text = String::from_utf8_lossy(&line);
        Ok(Some(text.trim_end_matches(['\r', '\n']).to_owned()))
    }
}

/// Valor de la cabecera `Content-Length` (el nombre no distingue mayusculas).
fn content_length(line: &str) -> Option<&str> {
    let (name, value) = line.split_once(':')?;
    name.trim()
        .eq_ignore_ascii_case("content-length")
        .then(|| value.trim())
}

fn parse_length(value: &str) -> Result<usize, FrameError> {
    let length: usize = value
        .parse()
        .map_err(|_| FrameError::InvalidLength(value.to_owned()))?;
    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge(length));
    }
    Ok(length)
}

/// Escribe un marco y vacia el buffer.
///
/// # Errors
///
/// El error de E/S si la salida se cierra.
pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, body: &[u8]) -> io::Result<()> {
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    writer.write_all(header.as_bytes()).await?;
    writer.write_all(body).await?;
    writer.flush().await
}

#[cfg(test)]
mod tests;
