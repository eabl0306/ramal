//! Diff de un fichero como lo da git: hunks con el bloque de lineas quitadas y
//! despues el de anadidas, cambios por palabra y lo que no se puede ensenar como
//! texto (binarios, imagenes, punteros LFS).
//!
//! El prototipo usaba el mismo diff para todos los ficheros y lo intercalaba
//! linea a linea (B-17). Aqui el orden es el de git, que nunca mezcla: un cambio
//! es primero lo que se quita y despues lo que se pone.

mod lfs;

pub use lfs::LfsPointer;

use crate::sha::Sha;

/// Que diff se pide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffTarget {
    /// Del indice al arbol de trabajo: lo que no esta preparado. Un fichero sin
    /// seguimiento sale entero como anadido.
    WorkingTree,
    /// De HEAD al indice: lo preparado.
    Index,
    /// De un commit contra su primer padre (contra el arbol vacio si es raiz).
    Commit(Sha),
    /// Entre dos commits.
    Range {
        /// Commit de partida.
        from: Sha,
        /// Commit de llegada.
        to: Sha,
    },
}

/// Como tratar los espacios en blanco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Whitespace {
    /// Cualquier cambio cuenta.
    #[default]
    Exact,
    /// Ignorar todos los espacios (`-w`).
    IgnoreAll,
    /// Ignorar cambios en la cantidad de espacios (`-b`).
    IgnoreChange,
    /// Ignorar espacios al final de linea, el `\r` de CRLF incluido.
    IgnoreAtEol,
}

/// Opciones del diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiffOptions {
    /// Espacios.
    pub whitespace: Whitespace,
    /// Marcar los cambios por palabra dentro de cada hunk.
    pub word_diff: bool,
    /// Lineas de contexto alrededor de cada cambio.
    pub context_lines: u32,
}

impl DiffOptions {
    /// Contexto maximo: mas no ayuda a leer y alarga la respuesta.
    pub const MAX_CONTEXT_LINES: u32 = 1000;
}

impl Default for DiffOptions {
    /// Las mismas que `git diff`: tres lineas de contexto y sin ignorar nada.
    fn default() -> Self {
        Self {
            whitespace: Whitespace::Exact,
            word_diff: false,
            context_lines: 3,
        }
    }
}

/// Que le paso al fichero entre los dos lados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStatus {
    /// Nuevo.
    Added,
    /// Cambiado.
    Modified,
    /// Borrado.
    Deleted,
    /// Renombrado (con o sin cambios).
    Renamed,
    /// Copiado.
    Copied,
    /// Cambio de tipo.
    TypeChanged,
}

/// Clase de una linea o de un trozo de palabra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineKind {
    /// Igual en los dos lados.
    Context,
    /// Solo en el lado nuevo.
    Added,
    /// Solo en el lado viejo.
    Removed,
}

/// Un trozo de una linea en el diff por palabras.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSegment {
    /// Comun, anadido o quitado.
    pub kind: LineKind,
    /// Texto del trozo.
    pub text: String,
}

/// Una linea de un hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// Comun, anadida o quitada. En el diff por palabras, una linea con trozos de
    /// los dos lados es `Context`: tiene numero en los dos lados.
    pub kind: LineKind,
    /// Numero de linea en el lado viejo, si existe ahi.
    pub old_line: Option<u32>,
    /// Numero de linea en el lado nuevo, si existe ahi.
    pub new_line: Option<u32>,
    /// Texto sin el salto de linea.
    pub text: String,
    /// Trozos por palabra; vacio si no se pidio el diff por palabras.
    pub words: Vec<WordSegment>,
    /// La linea es la ultima del fichero y no termina en salto de linea.
    pub no_newline_at_end: bool,
}

/// Un bloque de cambios con su contexto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// Primera linea en el lado viejo.
    pub old_start: u32,
    /// Lineas del lado viejo.
    pub old_lines: u32,
    /// Primera linea en el lado nuevo.
    pub new_start: u32,
    /// Lineas del lado nuevo.
    pub new_lines: u32,
    /// Lo que git escribe tras `@@ ... @@`: la funcion o seccion del cambio.
    pub section: String,
    /// Lineas en orden.
    pub lines: Vec<DiffLine>,
}

/// Lo que se puede ensenar del contenido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffContent {
    /// Texto con sus hunks (vacio si solo cambio el nombre o el modo).
    Text {
        /// Hunks en orden.
        hunks: Vec<Hunk>,
    },
    /// Binario: no se vuelcan bytes.
    Binary {
        /// Por la extension, es una imagen que la interfaz puede pintar.
        image: bool,
    },
    /// Uno o los dos lados son punteros de Git LFS.
    Lfs {
        /// Puntero del lado viejo.
        old: Option<LfsPointer>,
        /// Puntero del lado nuevo.
        new: Option<LfsPointer>,
    },
}

/// El diff de un fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiff {
    /// Ruta en el lado nuevo (en el viejo si se borro).
    pub path: String,
    /// Ruta en el lado viejo si se renombro o se copio.
    pub old_path: Option<String>,
    /// Que le paso.
    pub status: FileStatus,
    /// Contenido.
    pub content: DiffContent,
    /// Lineas anadidas.
    pub additions: u32,
    /// Lineas quitadas.
    pub deletions: u32,
    /// El diff era demasiado grande y se recorto: faltan hunks del final.
    pub truncated: bool,
}

/// Extensiones de imagen que la interfaz sabe pintar.
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "avif"];

/// La extension (en minusculas) es de una imagen.
#[must_use]
pub fn is_image_extension(extension: &str) -> bool {
    IMAGE_EXTENSIONS.contains(&extension)
}

/// Un fichero que no cambio nada visible: sin hunks ni contenido nuevo.
#[must_use]
pub const fn empty_text() -> DiffContent {
    DiffContent::Text { hunks: Vec::new() }
}

#[cfg(test)]
mod tests;
