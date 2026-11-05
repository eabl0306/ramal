//! `ConfigureGit`: cambia el ejecutable de git sin reiniciar el motor.
//!
//! Con el git del sistema, el usuario necesita ver cual se usa y poder cambiarlo
//! (D-43), y "Volver a comprobar" tras instalar git tiene que funcionar sin
//! reiniciar (D-02). Cambiar la ruta recalcula version y capacidades; una ruta que
//! no lleva a un git da `git_not_found`. Solo lo llaman los hosts: es una ruta
//! absoluta que decide que programa se ejecuta.

use std::path::PathBuf;
use std::sync::Arc;

use crate::error::EngineError;
use crate::port::git_reader::GitVersionInfo;
use crate::port::host::GitSetup;

/// Cambia y vuelve a comprobar el ejecutable de git.
pub struct ConfigureGit {
    setup: Arc<dyn GitSetup>,
}

impl std::fmt::Debug for ConfigureGit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ConfigureGit")
            .finish_non_exhaustive()
    }
}

impl ConfigureGit {
    /// Construye el caso de uso.
    #[must_use]
    pub fn new(setup: Arc<dyn GitSetup>) -> Self {
        Self { setup }
    }

    /// Usa `binary` (o el primer git del `PATH` si es `None`) y devuelve lo que
    /// encontro. Un git anterior al minimo no es un error aqui, como en
    /// `engine/info`: se informa para que la interfaz lo explique.
    ///
    /// # Errors
    ///
    /// [`EngineError::GitUnavailable`] si la ruta no lleva a un git que se pueda
    /// ejecutar y [`EngineError::GitFailed`] si su version no se entiende.
    pub async fn execute(&self, binary: Option<PathBuf>) -> Result<GitVersionInfo, EngineError> {
        Ok(self.setup.reconfigure(binary).await?)
    }
}

#[cfg(test)]
mod tests;
