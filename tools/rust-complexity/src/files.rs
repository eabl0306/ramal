//! Los ficheros de Rust a medir: todos los `.rs` bajo las raices, menos los
//! directorios de artefactos y de codigo generado.

use std::path::{Path, PathBuf};

use crate::CheckError;

/// Directorios donde nunca hay codigo que medir: builds, dependencias de npm y
/// lo que escriben los generadores (los esquemas de Tauri en src-tauri/gen).
const IGNORED_DIRS: &[&str] = &[
    ".git",
    "build",
    "coverage",
    "dist",
    "gen",
    "generated",
    "node_modules",
    "out",
    "target",
];

/// Todos los `.rs` bajo `roots`, ordenados.
///
/// # Errors
///
/// [`CheckError::MissingRoot`] si una raiz no existe y [`CheckError::Read`] si un
/// directorio no se puede listar.
pub fn rust_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>, CheckError> {
    let mut found = Vec::new();
    for root in roots {
        if !root.exists() {
            return Err(CheckError::MissingRoot(root.clone()));
        }
        walk(root, &mut found)?;
    }
    found.sort();
    Ok(found)
}

fn walk(path: &Path, found: &mut Vec<PathBuf>) -> Result<(), CheckError> {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path.to_path_buf());
        }
        return Ok(());
    }
    let read_error = |source| CheckError::Read {
        path: path.to_path_buf(),
        source,
    };
    for entry in std::fs::read_dir(path).map_err(read_error)? {
        let child = entry.map_err(read_error)?.path();
        let ignored = child
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| IGNORED_DIRS.contains(&name));
        if !(ignored && child.is_dir()) {
            walk(&child, found)?;
        }
    }
    Ok(())
}
