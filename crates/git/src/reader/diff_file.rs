//! Del diff de git al `FileDiff` del dominio: que seccion es la del fichero
//! pedido, si es binario o imagen, si es un puntero de LFS y cuantas lineas
//! cambian.

use git_rudder_core_application::DiffRequest;
use git_rudder_core_domain::diff::{
    DiffContent, DiffLine, FileDiff, FileStatus, Hunk, LfsPointer, LineKind, is_image_extension,
};

use crate::parse::diff::FileSection;

/// Bytes de diff como mucho. Un fichero generado de cien mil lineas no se lee en
/// la interfaz; se ensena el principio y se avisa de que esta recortado.
pub(super) const MAX_DIFF_BYTES: usize = 4 * 1024 * 1024;

/// El diff del fichero pedido. Sin seccion, el fichero no cambio.
pub(super) fn file_diff(
    request: &DiffRequest,
    sections: Vec<FileSection>,
    truncated: bool,
) -> FileDiff {
    let path = request.path.as_str();
    let mut sections = sections;
    let index = sections
        .iter()
        .position(|section| section.is_about(path))
        .unwrap_or(0);
    if index >= sections.len() {
        return FileDiff {
            path: path.to_owned(),
            old_path: request.old_path.as_ref().map(ToString::to_string),
            status: FileStatus::Modified,
            content: DiffContent::Text { hunks: Vec::new() },
            additions: 0,
            deletions: 0,
            truncated,
        };
    }
    let section = sections.swap_remove(index);
    let (additions, deletions) = counts(&section.hunks);
    FileDiff {
        path: section.new_path.clone().unwrap_or_else(|| path.to_owned()),
        old_path: section
            .old_path
            .clone()
            .or_else(|| request.old_path.as_ref().map(ToString::to_string)),
        status: section.status,
        content: content(request.path.extension().as_deref(), section),
        additions,
        deletions,
        truncated,
    }
}

fn content(extension: Option<&str>, section: FileSection) -> DiffContent {
    if section.binary {
        return DiffContent::Binary {
            image: extension.is_some_and(is_image_extension),
        };
    }
    let old = side_text(&section.hunks, LineKind::Added);
    let new = side_text(&section.hunks, LineKind::Removed);
    let old = (section.status != FileStatus::Added)
        .then(|| LfsPointer::parse(&old))
        .flatten();
    let new = (section.status != FileStatus::Deleted)
        .then(|| LfsPointer::parse(&new))
        .flatten();
    if old.is_some() || new.is_some() {
        return DiffContent::Lfs { old, new };
    }
    DiffContent::Text {
        hunks: section.hunks,
    }
}

/// Texto de un lado: todas las lineas menos las de `skip`.
fn side_text(hunks: &[Hunk], skip: LineKind) -> String {
    hunks
        .iter()
        .flat_map(|hunk| &hunk.lines)
        .filter(|line| line.kind != skip)
        .map(|line: &DiffLine| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn counts(hunks: &[Hunk]) -> (u32, u32) {
    let lines = hunks.iter().flat_map(|hunk| &hunk.lines);
    lines.fold((0_u32, 0_u32), |(added, removed), line| match line.kind {
        LineKind::Added => (added.saturating_add(1), removed),
        LineKind::Removed => (added, removed.saturating_add(1)),
        LineKind::Context => (added, removed),
    })
}
