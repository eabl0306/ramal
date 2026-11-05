//! Confianza recordada en el dispositivo (ADR 0020).
//!
//! Un fichero de texto con una raiz por linea, en la carpeta de datos del host
//! (la de la app en escritorio, el `globalStorage` de la extension en VS Code). No
//! se sincroniza: confiar es una decision sobre una carpeta de un equipo. Se
//! escribe entero en un fichero temporal y se renombra, para que una caida a
//! mitad no deje el fichero a medias. Sin ruta, la confianza vive en memoria y se
//! olvida al cerrar.
//!
//! Una raiz con un salto de linea en el nombre no se puede guardar en este
//! formato: se recuerda solo en memoria.

use std::collections::BTreeSet;
use std::path::PathBuf;

use async_trait::async_trait;
use git_rudder_core_application::{TrustStore, TrustStoreError};
use git_rudder_core_domain::RepoRoot;
use tokio::sync::Mutex;

/// Almacen de confianza en un fichero (o en memoria).
#[derive(Debug)]
pub struct FileTrustStore {
    path: Option<PathBuf>,
    cache: Mutex<Option<BTreeSet<String>>>,
}

impl FileTrustStore {
    /// Almacen en `path`, o solo en memoria si es `None`.
    #[must_use]
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            cache: Mutex::new(None),
        }
    }

    async fn load(&self) -> Result<BTreeSet<String>, TrustStoreError> {
        let Some(path) = &self.path else {
            return Ok(BTreeSet::new());
        };
        match tokio::fs::read_to_string(path).await {
            Ok(text) => Ok(text
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeSet::new()),
            Err(error) => Err(TrustStoreError(error.to_string())),
        }
    }

    async fn save(&self, roots: &BTreeSet<String>) -> Result<(), TrustStoreError> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let io = |error: std::io::Error| TrustStoreError(error.to_string());
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(io)?;
        }
        let text = roots.iter().filter(|root| !root.contains('\n')).fold(
            String::new(),
            |mut text, root| {
                text.push_str(root);
                text.push('\n');
                text
            },
        );
        let temporary = path.with_extension("tmp");
        tokio::fs::write(&temporary, text).await.map_err(io)?;
        tokio::fs::rename(&temporary, path).await.map_err(io)
    }
}

fn key(root: &RepoRoot) -> String {
    root.as_path().to_string_lossy().into_owned()
}

#[async_trait]
impl TrustStore for FileTrustStore {
    async fn is_trusted(&self, root: &RepoRoot) -> Result<bool, TrustStoreError> {
        let mut cache = self.cache.lock().await;
        if cache.is_none() {
            *cache = Some(self.load().await?);
        }
        Ok(cache
            .as_ref()
            .is_some_and(|roots| roots.contains(&key(root))))
    }

    async fn trust(&self, root: &RepoRoot) -> Result<(), TrustStoreError> {
        let mut cache = self.cache.lock().await;
        let mut roots = match cache.take() {
            Some(roots) => roots,
            None => self.load().await?,
        };
        roots.insert(key(root));
        let saved = self.save(&roots).await;
        *cache = Some(roots);
        saved
    }
}

#[cfg(test)]
mod tests;
