//! Log del grafo con separadores NUL (entre commits) y 0x1f (entre campos).
//!
//! Ni un nombre de autor ni un asunto pueden llevar NUL, asi que `-z` separa los
//! commits sin ambiguedad. El asunto va el ultimo y se toma entero, aunque traiga
//! un 0x1f: solo los campos de antes tienen que estar limpios.

use ramal_core_domain::{CommitNode, CommitRecord, GitTime, Sha, ShortSha, Signature};

use super::ParseError;

/// `--format` del log: sha, sha corto, padres, autor, correo, fecha ISO estricta y
/// asunto.
pub const LOG_FORMAT: &str = "--format=%H%x1f%h%x1f%P%x1f%an%x1f%ae%x1f%aI%x1f%s";

const FIELDS: usize = 7;

/// Lee la salida completa de `git log -z` con [`LOG_FORMAT`].
///
/// # Errors
///
/// [`ParseError`] si un registro no tiene los siete campos o un sha o una fecha no
/// son validos.
pub fn parse_log(stdout: &[u8]) -> Result<Vec<CommitRecord>, ParseError> {
    String::from_utf8_lossy(stdout)
        .split('\0')
        .map(|record| record.trim_start_matches('\n'))
        .filter(|record| !record.is_empty())
        .map(parse_record)
        .collect()
}

fn parse_record(record: &str) -> Result<CommitRecord, ParseError> {
    let fields: Vec<&str> = record.splitn(FIELDS, '\u{1f}').collect();
    let [sha, short, parents, name, email, date, summary] = fields.as_slice() else {
        return Err(ParseError(format!(
            "log record with {} fields",
            fields.len()
        )));
    };
    Ok(CommitRecord {
        node: CommitNode::new(parse_sha(sha)?, parse_parents(parents)?),
        short_sha: ShortSha::parse(short).map_err(|error| ParseError(error.to_string()))?,
        author: Signature {
            name: (*name).to_owned(),
            email: (*email).to_owned(),
        },
        authored_at: GitTime::parse_iso8601(date).map_err(|error| ParseError(error.to_string()))?,
        summary: (*summary).to_owned(),
    })
}

fn parse_parents(parents: &str) -> Result<Vec<Sha>, ParseError> {
    parents.split_whitespace().map(parse_sha).collect()
}

fn parse_sha(text: &str) -> Result<Sha, ParseError> {
    Sha::parse(text).map_err(|error| ParseError(error.to_string()))
}

#[cfg(test)]
mod tests;
