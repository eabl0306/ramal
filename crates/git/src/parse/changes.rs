//! Ficheros cambiados: `--name-status -z` y `--numstat -z` de `diff-tree` o
//! `diff`, que salen en el mismo orden y se juntan aqui.
//!
//! Con `-z` las rutas van tal cual. Un renombrado o una copia llevan dos rutas:
//! en `--name-status`, `R100\0viejo\0nuevo\0`; en `--numstat`,
//! `anadidas\tquitadas\t\0viejo\0nuevo\0`. Un binario cuenta `-\t-`.

use git_rudder_core_domain::diff::FileStatus;
use git_rudder_core_domain::{ChangedFile, LineCounts};

use super::ParseError;

/// Lee las dos salidas y las junta fichero a fichero.
///
/// # Errors
///
/// [`ParseError`] si a una entrada le faltan rutas o las dos salidas no tienen
/// los mismos ficheros.
pub fn parse_changes(name_status: &[u8], numstat: &[u8]) -> Result<Vec<ChangedFile>, ParseError> {
    let statuses = parse_name_status(name_status)?;
    let counts = parse_numstat(numstat)?;
    if statuses.len() != counts.len() {
        return Err(ParseError(format!(
            "{} files in --name-status and {} in --numstat",
            statuses.len(),
            counts.len()
        )));
    }
    Ok(statuses
        .into_iter()
        .zip(counts)
        .map(|((status, old_path, path), lines)| ChangedFile {
            path,
            old_path,
            status,
            lines,
        })
        .collect())
}

/// `(estado, ruta vieja, ruta)` de cada fichero.
///
/// # Errors
///
/// [`ParseError`] si a una entrada le faltan rutas.
pub fn parse_name_status(
    stdout: &[u8],
) -> Result<Vec<(FileStatus, Option<String>, String)>, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut fields = text.split('\0').map(|field| field.trim_start_matches('\n'));
    let mut out = Vec::new();
    while let Some(code) = fields.next().filter(|code| !code.is_empty()) {
        let status = status_of(code)?;
        let mut path = || {
            fields
                .next()
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| ParseError(format!("{code} without a path")))
        };
        let entry = if matches!(status, FileStatus::Renamed | FileStatus::Copied) {
            let old = path()?;
            (status, Some(old), path()?)
        } else {
            (status, None, path()?)
        };
        out.push(entry);
    }
    Ok(out)
}

/// La letra de git: `A`, `M`, `D`, `T`, `R<similitud>`, `C<similitud>` y, en un
/// merge, `U`.
///
/// # Errors
///
/// [`ParseError`] si no es una letra de estado.
pub fn status_of(code: &str) -> Result<FileStatus, ParseError> {
    let letter = code.chars().next();
    let digits = code.get(1..).unwrap_or_default();
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ParseError(format!("bad status code: {code}")));
    }
    Ok(match letter {
        Some('A') => FileStatus::Added,
        Some('M' | 'U') => FileStatus::Modified,
        Some('D') => FileStatus::Deleted,
        Some('T') => FileStatus::TypeChanged,
        Some('R') => FileStatus::Renamed,
        Some('C') => FileStatus::Copied,
        _ => return Err(ParseError(format!("bad status code: {code}"))),
    })
}

/// Lineas de cada fichero, `None` si es binario.
fn parse_numstat(stdout: &[u8]) -> Result<Vec<Option<LineCounts>>, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut fields = text.split('\0').map(|field| field.trim_start_matches('\n'));
    let mut out = Vec::new();
    while let Some(field) = fields.next().filter(|field| !field.is_empty()) {
        let parts: Vec<&str> = field.splitn(3, '\t').collect();
        let [added, removed, path] = parts.as_slice() else {
            return Err(ParseError(format!("bad numstat entry: {field}")));
        };
        if path.is_empty() {
            // Renombrado: las dos rutas vienen en los campos siguientes.
            fields.next();
            fields.next();
        }
        out.push(
            added
                .parse()
                .ok()
                .zip(removed.parse().ok())
                .map(|(additions, deletions)| LineCounts {
                    additions,
                    deletions,
                }),
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
