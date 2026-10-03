//! `git version`: version detectada y capacidades.
//!
//! Lo que llego despues de la minima se calcula a partir de la version real, no se
//! supone. El rechazo de un git anterior a 2.34 lo decide el caso de uso
//! `OpenRepository` con [`GitVersionInfo::is_supported`], para que `engine/info`
//! pueda describir igualmente un git viejo.
//!
//! [`GitVersionInfo::is_supported`]: git_rudder_core_application::GitVersionInfo::is_supported

use git_rudder_core_application::{GitCapabilities, GitVersion};

/// Primera version con `merge-tree --write-tree` y `rebase --update-refs`.
const MERGE_TREE_AND_UPDATE_REFS: GitVersion = GitVersion::new(2, 38, 0);

/// Lee la salida de `git version`: `git version 2.53.0`, `git version 2.39.5
/// (Apple Git-154)` o `git version 2.45.1.windows.1`. Un componente que falta
/// cuenta como 0 (`git version 2.40`).
#[must_use]
pub fn parse_version(output: &str) -> Option<GitVersion> {
    let text = output.trim().strip_prefix("git version ")?;
    let number = text.split_whitespace().next()?;
    let mut parts = number.split('.').map(leading_number);
    let major = parts.next()??;
    let minor = parts.next().flatten().unwrap_or(0);
    let patch = parts.next().flatten().unwrap_or(0);
    Some(GitVersion::new(major, minor, patch))
}

/// Las cifras del principio: `1-rc2` da 1 y `windows` no da nada.
fn leading_number(part: &str) -> Option<u32> {
    let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Capacidades de una version de git.
#[must_use]
pub fn capabilities_for(version: GitVersion) -> GitCapabilities {
    let modern = version >= MERGE_TREE_AND_UPDATE_REFS;
    GitCapabilities {
        merge_tree_write_tree: modern,
        rebase_update_refs: modern,
    }
}

#[cfg(test)]
mod tests;
