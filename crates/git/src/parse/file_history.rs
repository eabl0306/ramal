//! Historia de un fichero: `git log --follow --name-status -z` con el formato del
//! grafo.
//!
//! Con `-z`, cada commit es un campo con sus siete lineas y detras van sus
//! ficheros: la letra de estado y una ruta (dos si es un renombrado), cada cosa en
//! su campo. Un campo de commit empieza por 40 cifras hexadecimales y una letra
//! de estado nunca, asi que se distinguen sin ambiguedad. Un merge puede no traer
//! ficheros.

use git_rudder_core_domain::diff::FileStatus;
use git_rudder_core_domain::history::FileHistoryEntry;
use git_rudder_core_domain::sha::SHA_HEX_LEN;

use super::ParseError;
use super::changes::status_of;
use super::log::parse_record;

/// Lee la salida completa. Un commit sin fichero (un merge) lleva la ruta del
/// commit anterior en la lista, que es la que tenia el fichero entonces.
///
/// # Errors
///
/// [`ParseError`] si un commit o un estado no tienen la forma esperada.
pub fn parse_file_history(stdout: &[u8], path: &str) -> Result<Vec<FileHistoryEntry>, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut fields = text
        .split('\0')
        .map(|field| field.trim_start_matches('\n'))
        .filter(|field| !field.is_empty())
        .peekable();
    let mut entries = Vec::new();
    let mut current_path = path.to_owned();
    while let Some(field) = fields.next() {
        if !is_commit(field) {
            return Err(ParseError(format!("expected a commit, got {field:?}")));
        }
        let commit = parse_record(field)?;
        let mut entry = FileHistoryEntry {
            commit,
            path: current_path.clone(),
            old_path: None,
            status: FileStatus::Modified,
        };
        if let Some(code) = fields.next_if(|next| !is_commit(next)) {
            entry.status = status_of(code)?;
            let mut path = || {
                fields
                    .next()
                    .map(str::to_owned)
                    .ok_or_else(|| ParseError(format!("{code} without a path")))
            };
            if matches!(entry.status, FileStatus::Renamed | FileStatus::Copied) {
                entry.old_path = Some(path()?);
            }
            entry.path = path()?;
        }
        // Mas antiguo que este commit, el fichero tenia el nombre viejo.
        current_path = entry.old_path.clone().unwrap_or_else(|| entry.path.clone());
        entries.push(entry);
    }
    Ok(entries)
}

/// Empieza por un sha completo seguido de un salto de linea.
fn is_commit(field: &str) -> bool {
    field
        .get(..SHA_HEX_LEN)
        .is_some_and(|sha| sha.bytes().all(|byte| byte.is_ascii_hexdigit()))
        && field.as_bytes().get(SHA_HEX_LEN) == Some(&b'\n')
}

#[cfg(test)]
mod tests;
