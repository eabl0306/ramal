//! Cabecera y mensaje de un commit para el panel de detalle.
//!
//! Un campo por linea y el mensaje completo (`%B`) al final, que se toma entero:
//! puede tener todas las lineas que quiera, y los campos de delante no pueden
//! llevar saltos de linea.

use git_rudder_core_domain::{CommitDetail, GitTime, Sha, ShortSha, Signature};

use super::ParseError;

/// `--format` del detalle: sha, sha corto, padres, autor, correo, fecha de autor,
/// quien hizo el commit, su correo, su fecha y el mensaje completo.
pub const DETAIL_FORMAT: &str = "--format=%H%n%h%n%P%n%an%n%ae%n%ad%n%cn%n%ce%n%cd%n%B";

const FIELDS: usize = 10;

/// Lee la salida de `git log -1` con [`DETAIL_FORMAT`] y `--date=raw`. Los
/// ficheros los pone quien llama.
///
/// # Errors
///
/// [`ParseError`] si faltan campos o un sha no es valido.
pub fn parse_commit_detail(stdout: &[u8]) -> Result<CommitDetail, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let fields: Vec<&str> = text.splitn(FIELDS, '\n').collect();
    let [
        sha,
        short,
        parents,
        author,
        email,
        author_date,
        committer,
        committer_email,
        commit_date,
        message,
    ] = fields.as_slice()
    else {
        return Err(ParseError(format!(
            "commit detail with {} fields",
            fields.len()
        )));
    };
    let parse_sha = |text: &str| Sha::parse(text).map_err(|error| ParseError(error.to_string()));
    Ok(CommitDetail {
        sha: parse_sha(sha)?,
        short_sha: ShortSha::parse(short).map_err(|error| ParseError(error.to_string()))?,
        parents: parents
            .split_whitespace()
            .map(parse_sha)
            .collect::<Result<_, _>>()?,
        author: Signature {
            name: (*author).to_owned(),
            email: (*email).to_owned(),
        },
        authored_at: GitTime::parse_raw(author_date).unwrap_or_else(|_| GitTime::new(0, 0)),
        committer: Signature {
            name: (*committer).to_owned(),
            email: (*committer_email).to_owned(),
        },
        committed_at: GitTime::parse_raw(commit_date).unwrap_or_else(|_| GitTime::new(0, 0)),
        message: message.trim_end_matches('\n').to_owned(),
        files: Vec::new(),
    })
}

#[cfg(test)]
mod tests;
