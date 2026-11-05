//! Entrada y salida de git: stdin, y stdout con su limite.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use super::RunError;

/// Escribe stdin y lo cierra, para que git sepa que no hay mas.
pub(super) async fn write_input(
    input: Option<(tokio::process::ChildStdin, &[u8])>,
) -> Result<(), RunError> {
    let Some((mut pipe, bytes)) = input else {
        return Ok(());
    };
    pipe.write_all(bytes).await?;
    pipe.shutdown().await?;
    Ok(())
}

/// stdout entero, o hasta `limit`: si `truncate`, se corta y se sigue leyendo
/// para que git no se bloquee; si no, pasarse es un error.
pub(super) async fn read_stdout<R: AsyncRead + Unpin>(
    reader: Option<R>,
    limit: usize,
    truncate: bool,
) -> Result<(Vec<u8>, bool), RunError> {
    if !truncate {
        return read_capped(reader, limit).await.map(|bytes| (bytes, false));
    }
    let kept = read_truncated(reader, limit.saturating_add(1)).await?;
    let truncated = kept.len() > limit;
    let mut kept = kept;
    kept.truncate(limit);
    Ok((kept, truncated))
}

/// Lee todo `reader` y falla si pasa de `limit` bytes.
async fn read_capped<R: AsyncRead + Unpin>(
    reader: Option<R>,
    limit: usize,
) -> Result<Vec<u8>, RunError> {
    let Some(reader) = reader else {
        return Ok(Vec::new());
    };
    let mut buffer = Vec::new();
    let cap = u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1);
    reader.take(cap).read_to_end(&mut buffer).await?;
    if buffer.len() > limit {
        return Err(RunError::OutputTooLarge(limit));
    }
    Ok(buffer)
}

/// Lee todo `reader` pero solo guarda los primeros `limit` bytes. Se sigue leyendo
/// para que git no se bloquee con la tuberia llena.
pub(super) async fn read_truncated<R: AsyncRead + Unpin>(
    reader: Option<R>,
    limit: usize,
) -> Result<Vec<u8>, RunError> {
    let Some(mut reader) = reader else {
        return Ok(Vec::new());
    };
    let mut kept = Vec::new();
    // En el monton: un trozo de 8 KiB en la pila haria enorme cada futuro que
    // espera a git.
    let mut chunk = vec![0_u8; 8 * 1024];
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            return Ok(kept);
        }
        let room = limit.saturating_sub(kept.len()).min(read);
        kept.extend_from_slice(chunk.get(..room).unwrap_or_default());
    }
}
