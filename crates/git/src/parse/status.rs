//! `git status --porcelain=v2 --branch -z`: cabeceras `# branch.*` y una entrada
//! por fichero, separadas por NUL.
//!
//! Con `-z` las rutas van tal cual (sin comillas ni escapes) y un renombrado lleva
//! la ruta de origen en el campo siguiente. Los campos de una entrada se separan
//! por espacios y la ruta va la ultima, asi que una ruta con espacios se toma
//! entera con `splitn`.

use git_rudder_core_domain::Sha;
use git_rudder_core_domain::status::{
    AheadBehind, BranchStatus, ChangeKind, ConflictCode, EntryState, StatusEntry, SubmoduleChange,
    WorkingTreeStatus,
};

use super::ParseError;

/// Lee la salida completa. La operacion a medias no sale de aqui: la anade quien
/// llama a partir de los ficheros de la carpeta de git.
///
/// # Errors
///
/// [`ParseError`] si una entrada no tiene sus campos o un codigo no es de git.
pub fn parse_status(stdout: &[u8]) -> Result<WorkingTreeStatus, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut fields = text.split('\0').filter(|field| !field.is_empty());
    let mut status = WorkingTreeStatus::default();
    while let Some(field) = fields.next() {
        if let Some(header) = field.strip_prefix("# ") {
            read_header(&mut status.branch, header)?;
            continue;
        }
        let entry = match field.split_at_checked(2) {
            Some(("1 ", rest)) => ordinary(rest)?,
            Some(("2 ", rest)) => {
                let original = fields
                    .next()
                    .ok_or_else(|| ParseError(format!("rename without origin: {field}")))?;
                renamed(rest, original)?
            }
            Some(("u ", rest)) => unmerged(rest)?,
            Some(("? ", path)) => plain(path, EntryState::Untracked),
            Some(("! ", path)) => plain(path, EntryState::Ignored),
            _ => return Err(ParseError(format!("unknown status entry: {field}"))),
        };
        status.entries.push(entry);
    }
    Ok(status)
}

fn read_header(branch: &mut BranchStatus, header: &str) -> Result<(), ParseError> {
    let (key, value) = header.split_once(' ').unwrap_or((header, ""));
    match key {
        "branch.oid" if value != "(initial)" => {
            branch.head_sha = Some(sha(value)?);
        }
        "branch.head" if value != "(detached)" => branch.branch = Some(value.to_owned()),
        "branch.upstream" => branch.upstream = Some(value.to_owned()),
        "branch.ab" => branch.ahead_behind = Some(ahead_behind(value)?),
        _ => {}
    }
    Ok(())
}

/// `+<ahead> -<behind>`.
fn ahead_behind(value: &str) -> Result<AheadBehind, ParseError> {
    let bad = || ParseError(format!("bad branch.ab: {value}"));
    let (ahead, behind) = value.split_once(' ').ok_or_else(bad)?;
    let ahead = ahead.strip_prefix('+').ok_or_else(bad)?;
    let behind = behind.strip_prefix('-').ok_or_else(bad)?;
    Ok(AheadBehind {
        ahead: ahead.parse().map_err(|_| bad())?,
        behind: behind.parse().map_err(|_| bad())?,
    })
}

/// `<XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>`.
fn ordinary(rest: &str) -> Result<StatusEntry, ParseError> {
    let parts: Vec<&str> = rest.splitn(8, ' ').collect();
    let [xy, sub, _, _, _, _, _, path] = parts.as_slice() else {
        return Err(ParseError(format!("short status entry: {rest}")));
    };
    changed(xy, sub, path, None)
}

/// `<XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path>` y el origen aparte.
fn renamed(rest: &str, original: &str) -> Result<StatusEntry, ParseError> {
    let parts: Vec<&str> = rest.splitn(9, ' ').collect();
    let [xy, sub, _, _, _, _, _, _, path] = parts.as_slice() else {
        return Err(ParseError(format!("short rename entry: {rest}")));
    };
    changed(xy, sub, path, Some(original))
}

fn changed(
    xy: &str,
    sub: &str,
    path: &str,
    original: Option<&str>,
) -> Result<StatusEntry, ParseError> {
    let mut codes = xy.chars();
    let (Some(index), Some(worktree), None) = (codes.next(), codes.next(), codes.next()) else {
        return Err(ParseError(format!("bad status code: {xy}")));
    };
    Ok(StatusEntry {
        path: path.to_owned(),
        original_path: original.map(str::to_owned),
        state: EntryState::Changed {
            index: change_kind(index)?,
            worktree: change_kind(worktree)?,
            submodule: submodule(sub)?,
        },
    })
}

/// `<XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>`.
fn unmerged(rest: &str) -> Result<StatusEntry, ParseError> {
    let parts: Vec<&str> = rest.splitn(10, ' ').collect();
    let [xy, _, _, _, _, _, _, _, _, path] = parts.as_slice() else {
        return Err(ParseError(format!("short unmerged entry: {rest}")));
    };
    let conflict = match *xy {
        "DD" => ConflictCode::BothDeleted,
        "AU" => ConflictCode::AddedByUs,
        "UD" => ConflictCode::DeletedByThem,
        "UA" => ConflictCode::AddedByThem,
        "DU" => ConflictCode::DeletedByUs,
        "AA" => ConflictCode::BothAdded,
        "UU" => ConflictCode::BothModified,
        other => return Err(ParseError(format!("bad conflict code: {other}"))),
    };
    Ok(plain(path, EntryState::Unmerged { conflict }))
}

fn plain(path: &str, state: EntryState) -> StatusEntry {
    StatusEntry {
        path: path.to_owned(),
        original_path: None,
        state,
    }
}

fn change_kind(code: char) -> Result<ChangeKind, ParseError> {
    Ok(match code {
        '.' => ChangeKind::Unmodified,
        'M' => ChangeKind::Modified,
        'T' => ChangeKind::TypeChanged,
        'A' => ChangeKind::Added,
        'D' => ChangeKind::Deleted,
        'R' => ChangeKind::Renamed,
        'C' => ChangeKind::Copied,
        other => return Err(ParseError(format!("bad change code: {other}"))),
    })
}

/// `N...` si no es un submodulo; `S<c><m><u>` si lo es.
fn submodule(sub: &str) -> Result<Option<SubmoduleChange>, ParseError> {
    let bytes = sub.as_bytes();
    match bytes {
        [b'N', ..] => Ok(None),
        [b'S', commit, tracked, untracked] => Ok(Some(SubmoduleChange {
            commit_changed: *commit == b'C',
            has_tracked_changes: *tracked == b'M',
            has_untracked: *untracked == b'U',
        })),
        _ => Err(ParseError(format!("bad submodule field: {sub}"))),
    }
}

fn sha(text: &str) -> Result<Sha, ParseError> {
    Sha::parse(text).map_err(|error| ParseError(error.to_string()))
}

#[cfg(test)]
mod tests;
