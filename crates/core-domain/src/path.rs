//! Rutas relativas al repositorio que llegan de la interfaz.
//!
//! El webview no es de fiar (ADR 0020): ningun parametro suyo lleva una ruta
//! absoluta, y una relativa solo se acepta si no puede salir del arbol de
//! trabajo por su forma: sin `..`, sin NUL, sin componentes vacios y sin raiz.
//! Que tampoco salga despues de seguir enlaces simbolicos lo comprueba el
//! adaptador, que es quien puede mirar el disco. Ademas git nunca sigue un enlace
//! al leer: ensena su destino como texto.
//!
//! El separador es `/`, el que usa git en todas las plataformas. Una barra
//! invertida tambien parte componentes al validar: en Windows lo es, y en Unix un
//! nombre con barra invertida y `..` no tiene uso legitimo que compense el riesgo.

use std::fmt;

use thiserror::Error;

/// Ruta que no se acepta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PathError {
    /// La ruta esta vacia.
    #[error("the path is empty")]
    Empty,
    /// Empieza por una raiz (`/`, `\` o una unidad como `C:`).
    #[error("the path is absolute")]
    Absolute,
    /// Tiene un componente `..`.
    #[error("the path climbs out with ..")]
    ParentComponent,
    /// Tiene un componente vacio o `.` (`a//b`, `a/./b` o una barra final).
    #[error("the path has an empty component")]
    EmptyComponent,
    /// Tiene un caracter NUL.
    #[error("the path contains a NUL character")]
    Nul,
}

/// Ruta relativa a la raiz del repositorio, validada por su forma.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RepoPath(String);

impl RepoPath {
    /// Valida una ruta que llega de fuera.
    ///
    /// # Errors
    ///
    /// [`PathError`] si esta vacia, es absoluta, tiene `..`, un componente vacio o
    /// un NUL.
    pub fn parse(text: &str) -> Result<Self, PathError> {
        if text.is_empty() {
            return Err(PathError::Empty);
        }
        if text.contains('\0') {
            return Err(PathError::Nul);
        }
        if is_absolute(text) {
            return Err(PathError::Absolute);
        }
        for component in text.split(['/', '\\']) {
            match component {
                ".." => return Err(PathError::ParentComponent),
                "" | "." => return Err(PathError::EmptyComponent),
                _ => {}
            }
        }
        Ok(Self(text.to_owned()))
    }

    /// Ruta tal como la escribe git, con `/`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Ultimo componente: el nombre del fichero.
    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// Extension del nombre en minusculas, sin el punto, si la tiene.
    #[must_use]
    pub fn extension(&self) -> Option<String> {
        let name = self.file_name();
        name.rsplit_once('.')
            .filter(|(stem, extension)| !stem.is_empty() && !extension.is_empty())
            .map(|(_, extension)| extension.to_ascii_lowercase())
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// `/x`, `\x` y `C:...` son absolutas en alguna plataforma.
fn is_absolute(text: &str) -> bool {
    let bytes = text.as_bytes();
    let drive = matches!(bytes, [letter, b':', ..] if letter.is_ascii_alphabetic());
    text.starts_with(['/', '\\']) || drive
}

#[cfg(test)]
mod tests;
