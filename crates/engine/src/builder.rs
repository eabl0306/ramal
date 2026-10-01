//! `EngineBuilder` y `EngineConfig`: construyen los adaptadores que son iguales en
//! los dos productos.
//!
//! La composicion tiene dos niveles. Este es el de dentro: git CLI y, mas
//! adelante, plataformas, IA y Ramal API. Cada host es la raiz de fuera: lee su
//! configuracion (rutas, binario de git) y aporta lo que solo el sabe construir
//! (el `SecretStore`, el `EventSink`). Nadie mas construye adaptadores.

use std::path::PathBuf;
use std::sync::Arc;

use ramal_core_application::{
    DescribeEngine, GitReader, LoadGraphWindow, OpenRepositories, OpenRepository,
};
use ramal_git::{GitCliConfig, GitCliReader};

use crate::engine::{Engine, UseCases};

/// Version del motor, la del crate.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lo que el host sabe y el motor necesita.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EngineConfig {
    /// Ejecutable de git elegido por el usuario. En VS Code es el `git.path` del
    /// editor; sin el, el primer `git` del `PATH`.
    pub git_path: Option<PathBuf>,
}

/// Construye un [`Engine`].
pub struct EngineBuilder {
    config: EngineConfig,
    git_reader: Option<Arc<dyn GitReader>>,
}

impl std::fmt::Debug for EngineBuilder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EngineBuilder")
            .field("config", &self.config)
            .field("custom_git_reader", &self.git_reader.is_some())
            .finish()
    }
}

impl EngineBuilder {
    /// Builder con la configuracion del host.
    #[must_use]
    pub const fn new(config: EngineConfig) -> Self {
        Self {
            config,
            git_reader: None,
        }
    }

    /// Sustituye el lector de git del CLI por otro: un doble en los tests o, si un
    /// benchmark lo pide algun dia, una lectura sobre `gix` con el CLI de respaldo.
    #[must_use]
    pub fn with_git_reader(mut self, reader: Arc<dyn GitReader>) -> Self {
        self.git_reader = Some(reader);
        self
    }

    /// Cablea adaptadores y casos de uso.
    #[must_use]
    pub fn build(self) -> Engine {
        let git = self.git_reader.unwrap_or_else(|| {
            Arc::new(GitCliReader::new(GitCliConfig {
                binary: self.config.git_path,
                ..GitCliConfig::default()
            }))
        });
        let registry = Arc::new(OpenRepositories::new());
        Engine::new(UseCases {
            describe: DescribeEngine::new(Arc::clone(&git), ENGINE_VERSION),
            open: OpenRepository::new(Arc::clone(&git), Arc::clone(&registry)),
            graph: LoadGraphWindow::new(git, registry),
        })
    }
}
