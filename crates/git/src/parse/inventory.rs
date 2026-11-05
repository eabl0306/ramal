//! Ramas con upstream, tags, remotos, stashes y submodulos.

use git_rudder_core_domain::refs::{BranchInfo, RefListing, TagInfo, Tracking};
use git_rudder_core_domain::remote::Remote;
use git_rudder_core_domain::stash::{StashEntry, parse_stash_subject};
use git_rudder_core_domain::status::AheadBehind;
use git_rudder_core_domain::submodule::{Submodule, SubmoduleState};
use git_rudder_core_domain::{GitTime, RefKind, RefName, Sha};

use super::ParseError;

/// `--format` de `for-each-ref` para la barra lateral.
///
/// Nombre, objeto, objeto pelado, tipo, upstream, distancia, asunto (de un tag
/// anotado) y destino simbolico. NUL entre campos: un nombre de ref no puede
/// llevarlo.
pub const LISTING_FORMAT: &str = "--format=%(refname)%00%(objectname)%00%(*objectname)%00\
                                  %(objecttype)%00%(upstream)%00%(upstream:track,nobracket)%00\
                                  %(contents:subject)%00%(symref)";

/// `--format` de `git stash list -z`: sha, selector (`stash@{0}`), asunto del
/// reflog y fecha del commit en segundos. Sin `--date`, que cambiaria el
/// selector por una fecha.
pub const STASH_FORMAT: &str = "--format=%H%n%gd%n%gs%n%ct";

/// Ramas y tags de `for-each-ref` con [`LISTING_FORMAT`]. `head` es la rama de
/// HEAD (nombre completo), si la hay.
///
/// # Errors
///
/// [`ParseError`] si una linea no tiene sus ocho campos o un sha no es valido.
pub fn parse_listing(stdout: &[u8], head: Option<&str>) -> Result<RefListing, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut listing = RefListing::default();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let fields: Vec<&str> = line.split('\0').collect();
        let [name, object, peeled, kind, upstream, track, subject, symref] = fields.as_slice()
        else {
            return Err(ParseError(format!("ref line with {} fields", fields.len())));
        };
        if !symref.is_empty() {
            continue;
        }
        let Ok(name) = RefName::parse(name) else {
            continue;
        };
        let peeled_or_object = if peeled.is_empty() { object } else { peeled };
        let target = Sha::parse(peeled_or_object).map_err(|error| ParseError(error.to_string()))?;
        match name.kind() {
            RefKind::Tag => listing.tags.push(TagInfo {
                annotation: (*kind == "tag").then(|| (*subject).to_owned()),
                name,
                target,
            }),
            RefKind::LocalBranch => listing.local.push(BranchInfo {
                is_head: head == Some(name.as_str()),
                tracking: tracking(upstream, track)?,
                name,
                target,
            }),
            RefKind::RemoteBranch => listing.remote.push(BranchInfo {
                name,
                target,
                is_head: false,
                tracking: None,
            }),
        }
    }
    Ok(listing)
}

/// `ahead 1, behind 2`, `ahead 1`, `behind 2`, vacio (al dia) o `gone`.
fn tracking(upstream: &str, track: &str) -> Result<Option<Tracking>, ParseError> {
    if upstream.is_empty() {
        return Ok(None);
    }
    if track == "gone" {
        return Ok(Some(Tracking {
            upstream: upstream.to_owned(),
            distance: None,
        }));
    }
    let mut distance = AheadBehind::default();
    for part in track.split(", ").filter(|part| !part.is_empty()) {
        let (key, count) = part
            .split_once(' ')
            .ok_or_else(|| ParseError(format!("bad upstream track: {track}")))?;
        let count = count
            .parse()
            .map_err(|_| ParseError(format!("bad upstream track: {track}")))?;
        match key {
            "ahead" => distance.ahead = count,
            "behind" => distance.behind = count,
            _ => return Err(ParseError(format!("bad upstream track: {track}"))),
        }
    }
    Ok(Some(Tracking {
        upstream: upstream.to_owned(),
        distance: Some(distance),
    }))
}

/// Remotos de `git config -z --get-regexp ^remote\..*\.(url|pushurl)$`: cada
/// entrada es `remote.<nombre>.url\n<url>`. Un nombre de remoto puede llevar
/// puntos, asi que se corta por el ultimo.
#[must_use]
pub fn parse_remotes(stdout: &[u8]) -> Vec<Remote> {
    let text = String::from_utf8_lossy(stdout);
    let mut remotes: Vec<Remote> = Vec::new();
    for entry in text.split('\0').filter(|entry| !entry.is_empty()) {
        let (key, value) = entry.split_once('\n').unwrap_or((entry, ""));
        let Some((name, setting)) = key
            .strip_prefix("remote.")
            .and_then(|rest| rest.rsplit_once('.'))
        else {
            continue;
        };
        let position = remotes.iter().position(|remote| remote.name == name);
        let remote = if let Some(position) = position {
            remotes.get_mut(position)
        } else {
            remotes.push(Remote {
                name: name.to_owned(),
                fetch_url: None,
                push_url: None,
            });
            remotes.last_mut()
        };
        let Some(remote) = remote else {
            continue;
        };
        match setting {
            "url" if remote.fetch_url.is_none() => remote.fetch_url = Some(value.to_owned()),
            "pushurl" if remote.push_url.is_none() => remote.push_url = Some(value.to_owned()),
            _ => {}
        }
    }
    remotes
}

/// Stashes de `git stash list -z` con [`STASH_FORMAT`].
///
/// # Errors
///
/// [`ParseError`] si un registro no tiene sus campos.
pub fn parse_stashes(stdout: &[u8]) -> Result<Vec<StashEntry>, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    text.split('\0')
        .map(|record| record.trim_start_matches('\n'))
        .filter(|record| !record.is_empty())
        .map(|record| {
            let fields: Vec<&str> = record.splitn(4, '\n').collect();
            let [sha, selector, subject, seconds] = fields.as_slice() else {
                return Err(ParseError(format!("stash with {} fields", fields.len())));
            };
            let index = selector
                .strip_prefix("stash@{")
                .and_then(|rest| rest.strip_suffix('}'))
                .and_then(|number| number.parse().ok())
                .ok_or_else(|| ParseError(format!("bad stash selector: {selector}")))?;
            let (branch, message) = parse_stash_subject(subject);
            Ok(StashEntry {
                index,
                sha: Sha::parse(sha).map_err(|error| ParseError(error.to_string()))?,
                message,
                branch,
                created_at: GitTime::new(seconds.trim().parse().unwrap_or(0), 0),
            })
        })
        .collect()
}

/// Submodulos de `git submodule status --recursive`: `<marca><sha> <ruta>` y,
/// si git sabe describirlo, ` (<descripcion>)`. La marca es `-` sin
/// inicializar, `+` en otro commit, `U` en conflicto y un espacio al dia.
///
/// # Errors
///
/// [`ParseError`] si una linea no tiene marca, sha y ruta.
pub fn parse_submodules(stdout: &[u8]) -> Result<Vec<Submodule>, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let bad = || ParseError(format!("bad submodule line: {line}"));
            let (flag, rest) = line.split_at_checked(1).ok_or_else(bad)?;
            let state = match flag {
                "-" => SubmoduleState::Uninitialized,
                "+" => SubmoduleState::Modified,
                "U" => SubmoduleState::Conflicted,
                " " => SubmoduleState::UpToDate,
                _ => return Err(bad()),
            };
            let (sha, rest) = rest.split_once(' ').ok_or_else(bad)?;
            let (path, describe) = match rest.rsplit_once(" (") {
                Some((path, describe)) if describe.ends_with(')') => {
                    (path, describe.strip_suffix(')').map(str::to_owned))
                }
                _ => (rest, None),
            };
            Ok(Submodule {
                path: path.to_owned(),
                sha: Sha::parse(sha).map_err(|_| bad())?,
                state,
                describe,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
