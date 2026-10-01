//! `DescribeEngine`: version del motor y del git que va a usar.
//!
//! Los hosts lo llaman al arrancar. Un git anterior al minimo no es un error aqui:
//! se informa con `is_supported() == false` para que la interfaz explique que
//! hacer. Sin git si es un error, porque no hay version que describir.

use std::sync::Arc;

use crate::error::EngineError;
use crate::port::git_reader::{GitReader, GitVersionInfo};

/// Lo que devuelve [`DescribeEngine`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineDescription {
    /// Version del motor.
    pub engine_version: String,
    /// El git que va a usar.
    pub git: GitVersionInfo,
}

/// Describe el motor y su git.
pub struct DescribeEngine {
    git: Arc<dyn GitReader>,
    engine_version: String,
}

impl std::fmt::Debug for DescribeEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DescribeEngine")
            .field("engine_version", &self.engine_version)
            .finish_non_exhaustive()
    }
}

impl DescribeEngine {
    /// Construye el caso de uso con la version del motor que lo aloja.
    #[must_use]
    pub fn new(git: Arc<dyn GitReader>, engine_version: impl Into<String>) -> Self {
        Self {
            git,
            engine_version: engine_version.into(),
        }
    }

    /// Lee la version de git.
    ///
    /// # Errors
    ///
    /// [`EngineError::GitUnavailable`] si no hay git, o
    /// [`EngineError::GitFailed`] si `git --version` falla.
    pub async fn execute(&self) -> Result<EngineDescription, EngineError> {
        let git = self.git.version().await?;
        Ok(EngineDescription {
            engine_version: self.engine_version.clone(),
            git,
        })
    }
}

#[cfg(test)]
mod tests;
