//! `diff/file`: el diff de un fichero, como lo da git.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Que lados comparar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum DiffTarget {
    /// Del indice al arbol de trabajo (lo no preparado; un fichero sin
    /// seguimiento sale entero).
    WorkingTree,
    /// De HEAD al indice (lo preparado).
    Index,
    /// De un commit contra su primer padre.
    Commit {
        /// Sha completo.
        sha: String,
    },
    /// Entre dos commits.
    Range {
        /// Sha de partida.
        from: String,
        /// Sha de llegada.
        to: String,
    },
}

/// Como tratar los espacios.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Whitespace {
    /// Todo cuenta.
    #[default]
    Exact,
    /// Ignorar todos los espacios.
    IgnoreAll,
    /// Ignorar cambios en la cantidad de espacios.
    IgnoreChange,
    /// Ignorar espacios al final de linea (el `\r` de CRLF incluido).
    IgnoreAtEol,
}

/// Opciones del diff; todas tienen valor por defecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct DiffOptions {
    /// Espacios; por defecto `exact`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub whitespace: Option<Whitespace>,
    /// Cambios por palabra; por defecto no.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub word_diff: Option<bool>,
    /// Lineas de contexto; por defecto 3, como mucho 1000.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub context_lines: Option<u32>,
}

/// Parametros de `diff/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export)]
pub struct DiffFileParams {
    /// Repositorio abierto.
    pub repo_id: String,
    /// Ruta relativa a la raiz, con `/`. Sin `..` ni rutas absolutas.
    pub path: String,
    /// Ruta en el lado viejo si se renombro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub old_path: Option<String>,
    /// Que lados comparar.
    pub target: DiffTarget,
    /// Espacios, palabras y contexto.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub options: Option<DiffOptions>,
}

/// Que le paso al fichero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum FileStatus {
    /// Nuevo.
    Added,
    /// Cambiado.
    Modified,
    /// Borrado.
    Deleted,
    /// Renombrado.
    Renamed,
    /// Copiado.
    Copied,
    /// Cambio de tipo.
    TypeChanged,
}

/// Clase de una linea o de un trozo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum LineKind {
    /// Igual en los dos lados.
    Context,
    /// Solo en el nuevo.
    Added,
    /// Solo en el viejo.
    Removed,
}

/// Un trozo de linea del diff por palabras.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WordSegment {
    /// Comun, anadido o quitado.
    pub kind: LineKind,
    /// Texto.
    pub text: String,
}

/// Una linea de un hunk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct DiffLine {
    /// Comun, anadida o quitada.
    pub kind: LineKind,
    /// Numero en el lado viejo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub old_line: Option<u32>,
    /// Numero en el lado nuevo.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub new_line: Option<u32>,
    /// Texto sin el salto de linea.
    pub text: String,
    /// Trozos por palabra; vacio sin `wordDiff`.
    pub words: Vec<WordSegment>,
    /// Ultima linea del fichero sin salto final.
    pub no_newline_at_end: bool,
}

/// Un bloque de cambios.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Hunk {
    /// Primera linea en el lado viejo.
    pub old_start: u32,
    /// Lineas del lado viejo.
    pub old_lines: u32,
    /// Primera linea en el lado nuevo.
    pub new_start: u32,
    /// Lineas del lado nuevo.
    pub new_lines: u32,
    /// Funcion o seccion del cambio, tras `@@ ... @@`.
    pub section: String,
    /// Lineas en orden: lo quitado antes que lo anadido.
    pub lines: Vec<DiffLine>,
}

/// Puntero de Git LFS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LfsPointer {
    /// `sha256:<hex>`.
    pub oid: String,
    /// Tamano en bytes, en decimal (puede pasar de 2^53).
    pub size: String,
}

/// Lo que se puede ensenar del contenido.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum DiffContent {
    /// Texto con hunks (vacio si solo cambio el nombre o el modo).
    Text {
        /// Hunks.
        hunks: Vec<Hunk>,
    },
    /// Binario, sin bytes.
    Binary {
        /// Es una imagen que se puede pintar.
        image: bool,
    },
    /// Punteros de LFS.
    Lfs {
        /// Lado viejo.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        old: Option<LfsPointer>,
        /// Lado nuevo.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        new: Option<LfsPointer>,
    },
}

/// Resultado de `diff/file`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FileDiff {
    /// Ruta en el lado nuevo (en el viejo si se borro).
    pub path: String,
    /// Ruta en el lado viejo si se renombro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub old_path: Option<String>,
    /// Que le paso.
    pub status: FileStatus,
    /// Contenido.
    pub content: DiffContent,
    /// Lineas anadidas.
    pub additions: u32,
    /// Lineas quitadas.
    pub deletions: u32,
    /// Se recorto por grande: faltan hunks del final.
    pub truncated: bool,
}
