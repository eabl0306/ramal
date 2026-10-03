//! Log del grafo: un commit por registro separado por NUL (`-z`) y un campo por
//! linea.
//!
//! El salto de linea separa los campos sin ambiguedad: el nombre y el correo
//! viven en una linea de la cabecera del commit, asi que no pueden llevarlo, y
//! `%s` une el primer parrafo con espacios. Antes se separaba con 0x1f, y git si
//! deja meter un 0x1f en el nombre o el correo: los campos se desplazaban y el
//! grafo entero fallaba. El asunto va el ultimo y se toma entero.
//!
//! Un commit con una fecha que no se puede leer no tumba el log: se le pone la
//! fecha cero, como hace el propio git log. Un sha o un padre invalido si es un
//! error, porque sin ellos no hay grafo que dibujar.

use git_rudder_core_domain::{CommitNode, CommitRecord, GitTime, Sha, ShortSha, Signature};

use super::ParseError;

/// `--format` del log: sha, sha corto, padres, fecha de autor, autor, correo y
/// asunto, uno por linea.
pub const LOG_FORMAT: &str = "--format=%H%n%h%n%P%n%ad%n%an%n%ae%n%s";

/// Formato de `%ad` que entiende [`GitTime::parse_raw`]: segundos y zona tal como
/// estan guardados, sin pasar por el calendario de git.
pub const LOG_DATE: &str = "--date=raw";

const FIELDS: usize = 7;

/// Fecha que se pone a un commit cuya fecha no se puede leer: la misma que
/// ensena git log en ese caso.
const UNREADABLE_DATE: GitTime = GitTime::new(0, 0);

/// Lee la salida completa de `git log -z` con [`LOG_FORMAT`] y [`LOG_DATE`].
///
/// # Errors
///
/// [`ParseError`] si un registro no tiene los siete campos o un sha, un sha
/// corto o un padre no son validos.
pub fn parse_log(stdout: &[u8]) -> Result<Vec<CommitRecord>, ParseError> {
    String::from_utf8_lossy(stdout)
        .split('\0')
        // Solo puede sobrar un salto antes del sha, que nunca esta vacio. Los
        // campos vacios de dentro (los padres de un commit raiz) se conservan.
        .map(|record| record.trim_start_matches('\n'))
        .filter(|record| !record.is_empty())
        .map(parse_record)
        .collect()
}

fn parse_record(record: &str) -> Result<CommitRecord, ParseError> {
    let fields: Vec<&str> = record.splitn(FIELDS, '\n').collect();
    let [sha, short, parents, date, name, email, summary] = fields.as_slice() else {
        return Err(ParseError(format!(
            "log record with {} fields",
            fields.len()
        )));
    };
    let sha = parse_sha(sha)?;
    Ok(CommitRecord {
        short_sha: ShortSha::parse(short).map_err(|error| ParseError(error.to_string()))?,
        author: Signature {
            name: (*name).to_owned(),
            email: (*email).to_owned(),
        },
        authored_at: author_date(sha, date),
        summary: (*summary).to_owned(),
        node: CommitNode::new(sha, parse_parents(parents)?),
    })
}

/// git guarda en la cabecera lo que le den, y `%ad` sale vacio si no la entiende
/// (`author T <t@t> notadate`, que puede venir de un importador).
fn author_date(sha: Sha, date: &str) -> GitTime {
    GitTime::parse_raw(date).unwrap_or_else(|error| {
        tracing::debug!(%sha, %error, "unreadable author date; using the epoch");
        UNREADABLE_DATE
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
