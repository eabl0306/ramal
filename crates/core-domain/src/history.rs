//! Historia mas alla del grafo: buscar en todo el historial, la historia de un
//! fichero y comparar dos ramas.

use crate::commit::{ChangedFile, CommitRecord};
use crate::diff::FileStatus;
use crate::sha::{SHA_HEX_LEN, SHORT_SHA_MIN_LEN, Sha};

/// Lo que se busca. Un texto que parece un sha corto se busca como sha y como
/// texto a la vez: `cafe` puede ser las dos cosas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    text: String,
    sha_prefix: Option<String>,
}

impl SearchQuery {
    /// Lee lo que escribio el usuario, sin espacios alrededor. `None` si queda
    /// vacio.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let looks_like_sha = (SHORT_SHA_MIN_LEN..=SHA_HEX_LEN).contains(&text.len())
            && text.bytes().all(|byte| byte.is_ascii_hexdigit());
        Some(Self {
            text: text.to_owned(),
            sha_prefix: looks_like_sha.then(|| text.to_ascii_lowercase()),
        })
    }

    /// Texto a buscar en mensajes y autores.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Prefijo de sha en minusculas, si el texto lo parece.
    #[must_use]
    pub fn sha_prefix(&self) -> Option<&str> {
        self.sha_prefix.as_deref()
    }
}

/// Por que un commit coincide con la busqueda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchKind {
    /// Su sha empieza por el texto.
    Sha,
    /// Su mensaje contiene el texto.
    Message,
    /// Su autor (nombre o correo) contiene el texto.
    Author,
}

/// Un commit encontrado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    /// El commit.
    pub commit: CommitRecord,
    /// Por que coincide.
    pub matched: MatchKind,
    /// Fila en el grafo, si ya se sabe.
    pub row: Option<usize>,
}

/// Una entrada de la historia de un fichero.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHistoryEntry {
    /// El commit.
    pub commit: CommitRecord,
    /// Ruta del fichero en ese commit: cambia al cruzar un renombrado.
    pub path: String,
    /// Ruta anterior si ese commit lo renombro.
    pub old_path: Option<String>,
    /// Que le hizo el commit al fichero.
    pub status: FileStatus,
}

/// Dos refs comparadas desde su ancestro comun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchComparison {
    /// Commit de la base.
    pub base: Sha,
    /// Commit de la cabeza.
    pub head: Sha,
    /// Ancestro comun mas reciente; `None` si no comparten historia.
    pub merge_base: Option<Sha>,
    /// Commits de la cabeza que no tiene la base, del mas nuevo al mas viejo.
    pub ahead: Vec<CommitRecord>,
    /// Commits de la base que no tiene la cabeza.
    pub behind: Vec<CommitRecord>,
    /// Ficheros que cambian del ancestro comun a la cabeza.
    pub files: Vec<ChangedFile>,
}

impl BranchComparison {
    /// La cabeza ya esta entera en la base: no hay nada que fusionar.
    #[must_use]
    pub const fn head_is_merged(&self) -> bool {
        self.ahead.is_empty()
    }
}

#[cfg(test)]
mod tests;
