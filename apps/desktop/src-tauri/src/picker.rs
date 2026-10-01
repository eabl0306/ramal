//! Elegir la carpeta de un repositorio.
//!
//! El dialogo nativo lo ensena Rust, nunca el webview: el webview no manda rutas
//! (architecture.md 4.3). Por eso el plugin de dialogo no tiene ningun permiso en
//! las capacidades: solo lo usa el comando `repo_pick_and_open`.
//!
//! El selector va detras de un trait para que los tests y el E2E nativo puedan
//! poner una carpeta fija: WebDriver no puede manejar un dialogo del sistema.

use std::path::PathBuf;

use tauri::{AppHandle, Manager as _, Runtime};
use tauri_plugin_dialog::{DialogExt as _, FilePath};
use tokio::sync::oneshot;

use crate::surfaces::MAIN_WINDOW;

/// Respuesta de un selector: la carpeta elegida, o `None` si se cancelo. Si el
/// selector se suelta sin contestar, el receptor da error y cuenta como cancelado.
pub type PickedFolder = oneshot::Receiver<Option<PathBuf>>;

/// Algo que sabe pedir una carpeta al usuario.
pub trait FolderPicker: Send + Sync {
    /// Empieza a pedir la carpeta; la respuesta llega por el receptor.
    fn pick_folder(&self) -> PickedFolder;
}

/// El dialogo nativo de carpetas de `tauri-plugin-dialog`, sobre la ventana
/// principal para que sea modal a ella.
pub struct DialogFolderPicker<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> DialogFolderPicker<R> {
    /// Selector sobre la app en marcha.
    #[must_use]
    pub const fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> std::fmt::Debug for DialogFolderPicker<R> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("DialogFolderPicker").finish()
    }
}

impl<R: Runtime> FolderPicker for DialogFolderPicker<R> {
    fn pick_folder(&self) -> PickedFolder {
        let (sender, receiver) = oneshot::channel();
        let mut dialog = self.app.dialog().file();
        if let Some(window) = self.app.get_webview_window(MAIN_WINDOW) {
            dialog = dialog.set_parent(&window);
        }
        dialog.pick_folder(move |folder| {
            // Si nadie espera ya la respuesta, no hay nada que hacer con ella.
            let _ = sender.send(chosen_folder(folder));
        });
        receiver
    }
}

/// Lo que contesta el dialogo, como carpeta. En escritorio siempre da una ruta;
/// una URL que no es de fichero no es una carpeta y cuenta como cancelar.
fn chosen_folder(picked: Option<FilePath>) -> Option<PathBuf> {
    picked.and_then(|folder| folder.into_path().ok())
}

/// Una respuesta fija: la carpeta que se le dio, o cancelar si no se le dio
/// ninguna. La usan los tests y el E2E nativo (feature `e2e`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FixedFolderPicker {
    folder: Option<PathBuf>,
}

impl FixedFolderPicker {
    /// Selector que siempre elige `folder`.
    #[must_use]
    pub const fn choosing(folder: PathBuf) -> Self {
        Self {
            folder: Some(folder),
        }
    }

    /// Selector que siempre cancela.
    #[must_use]
    pub const fn cancelling() -> Self {
        Self { folder: None }
    }
}

impl FolderPicker for FixedFolderPicker {
    fn pick_folder(&self) -> PickedFolder {
        let (sender, receiver) = oneshot::channel();
        // El receptor esta vivo: se devuelve justo despues.
        let _ = sender.send(self.folder.clone());
        receiver
    }
}

#[cfg(test)]
mod tests;
