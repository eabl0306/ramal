//! `git blame --porcelain`: una cabecera por grupo de lineas, los datos del
//! commit la primera vez que aparece y la linea detras de un tabulador.
//!
//! ```text
//! <sha> <linea original> <linea final> [<lineas del grupo>]
//! author Ana Ruiz
//! author-mail <ana@example.com>
//! author-time 1790761320
//! author-tz +0200
//! summary Fix login
//! filename src/a.rs
//! \t<texto de la linea>
//! ```

use std::collections::HashMap;

use git_rudder_core_domain::blame::{Blame, BlameCommit, BlameLine};
use git_rudder_core_domain::{GitTime, Sha, Signature};

use super::ParseError;

/// Lee la salida completa.
///
/// # Errors
///
/// [`ParseError`] si una cabecera no tiene sha y numeros de linea.
pub fn parse_blame(stdout: &[u8]) -> Result<Blame, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut blame = Blame::default();
    let mut index: HashMap<Sha, usize> = HashMap::new();
    let mut current: Option<(Sha, u32, u32)> = None;
    for line in text.split('\n') {
        if let Some(content) = line.strip_prefix('\t') {
            let (sha, original_line, final_line) = current
                .take()
                .ok_or_else(|| ParseError("blame line without a header".to_owned()))?;
            blame.lines.push(BlameLine {
                line: final_line,
                original_line,
                sha,
                text: content.to_owned(),
            });
            continue;
        }
        if current.is_none() {
            if line.is_empty() {
                continue;
            }
            let header = header(line)?;
            if let std::collections::hash_map::Entry::Vacant(slot) = index.entry(header.0) {
                slot.insert(blame.commits.len());
                blame.commits.push(empty_commit(header.0));
            }
            current = Some(header);
            continue;
        }
        let Some(commit) = current
            .and_then(|(sha, _, _)| index.get(&sha))
            .and_then(|position| blame.commits.get_mut(*position))
        else {
            continue;
        };
        read_key(commit, line);
    }
    Ok(blame)
}

fn header(line: &str) -> Result<(Sha, u32, u32), ParseError> {
    let bad = || ParseError(format!("bad blame header: {line}"));
    let mut parts = line.split(' ');
    let sha = Sha::parse(parts.next().ok_or_else(bad)?).map_err(|_| bad())?;
    let original = parts.next().and_then(|n| n.parse().ok()).ok_or_else(bad)?;
    let final_line = parts.next().and_then(|n| n.parse().ok()).ok_or_else(bad)?;
    Ok((sha, original, final_line))
}

const fn empty_commit(sha: Sha) -> BlameCommit {
    BlameCommit {
        sha,
        author: Signature {
            name: String::new(),
            email: String::new(),
        },
        authored_at: GitTime::new(0, 0),
        summary: String::new(),
        path: String::new(),
    }
}

/// Una linea `clave valor` de los datos del commit.
fn read_key(commit: &mut BlameCommit, line: &str) {
    let (key, value) = line.split_once(' ').unwrap_or((line, ""));
    match key {
        "author" => value.clone_into(&mut commit.author.name),
        "author-mail" => {
            let email = value.trim_start_matches('<').trim_end_matches('>');
            email.clone_into(&mut commit.author.email);
        }
        "author-time" => {
            let offset = commit.authored_at.offset_minutes();
            commit.authored_at = GitTime::new(value.parse().unwrap_or(0), offset);
        }
        "author-tz" => {
            let seconds = commit.authored_at.seconds();
            let zone = GitTime::parse_raw(&format!("0 {value}")).map_or(0, |t| t.offset_minutes());
            commit.authored_at = GitTime::new(seconds, zone);
        }
        "summary" => value.clone_into(&mut commit.summary),
        "filename" => value.clone_into(&mut commit.path),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
