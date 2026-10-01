//! Errores del motor hacia el protocolo.
//!
//! `message` es para el log y va en ingles; la interfaz decide y traduce por
//! `data.kind`. El detalle de un fallo de git no sale del motor: se registra aqui y
//! al cliente llega un mensaje fijo.

use ramal_core_application::EngineError;
use ramal_protocol::{ErrorKind, RpcError, error_code};

/// Error de un caso de uso.
#[must_use]
pub fn engine_error(error: &EngineError) -> RpcError {
    let (code, kind) = match error {
        EngineError::RepositoryNotOpen { .. } => (error_code::ENGINE_ERROR, ErrorKind::RepoNotOpen),
        EngineError::NotARepository { .. } => (error_code::ENGINE_ERROR, ErrorKind::NotARepository),
        EngineError::GitUnavailable => (error_code::ENGINE_ERROR, ErrorKind::GitNotFound),
        EngineError::GitVersionUnsupported { .. } => {
            (error_code::ENGINE_ERROR, ErrorKind::GitVersionUnsupported)
        }
        EngineError::InvalidWindow(_) => (error_code::INVALID_PARAMS, ErrorKind::InvalidParams),
        EngineError::GitFailed { detail } => {
            tracing::warn!(%detail, "git command failed");
            return RpcError::new(
                error_code::ENGINE_ERROR,
                "git command failed",
                ErrorKind::GitFailed,
            );
        }
    };
    RpcError::new(code, error.to_string(), kind)
}

/// Parametros que no encajan con el metodo.
#[must_use]
pub fn invalid_params(detail: &str) -> RpcError {
    RpcError::new(
        error_code::INVALID_PARAMS,
        format!("invalid params: {detail}"),
        ErrorKind::InvalidParams,
    )
}

/// Metodo que no existe. Para la interfaz es un fallo interno: solo llama a los
/// metodos del mapa generado, asi que un metodo desconocido es un error de
/// empaquetado, no algo que el usuario pueda arreglar.
#[must_use]
pub fn method_not_found(method: &str) -> RpcError {
    RpcError::new(
        error_code::METHOD_NOT_FOUND,
        format!("method {method} does not exist"),
        ErrorKind::Internal,
    )
}

/// Metodo del host pedido desde un webview. Se responde como si no existiera.
#[must_use]
pub fn forbidden_method(method: &str) -> RpcError {
    RpcError::new(
        error_code::METHOD_NOT_FOUND,
        format!("method {method} is not available to webviews"),
        ErrorKind::Internal,
    )
}

/// Fallo inesperado del motor.
#[must_use]
pub fn internal_error(detail: &str) -> RpcError {
    tracing::error!(%detail, "internal engine error");
    RpcError::new(
        error_code::INTERNAL_ERROR,
        "internal engine error",
        ErrorKind::Internal,
    )
}
