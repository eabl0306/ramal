//! Refs leidas con `git for-each-ref`, nunca con la decoracion `%D` del log: un
//! nombre de ref puede llevar comas y `%D` las usa de separador.
//!
//! Un nombre de ref no puede llevar caracteres de control (`check-ref-format`), asi
//! que NUL separa los campos y el salto de linea separa las refs.

use ramal_core_domain::{GitRef, RefName, Sha};

use super::ParseError;

/// `--format` de `for-each-ref`: nombre, tipo y objeto, tipo y objeto pelados (un
/// tag anotado apunta a un objeto tag; lo que interesa es su commit) y destino
/// simbolico (`origin/HEAD`).
pub const REFS_FORMAT: &str = "--format=%(refname)%00%(objecttype)%00%(objectname)%00\
                               %(*objecttype)%00%(*objectname)%00%(symref)";

/// La ref simbolica que dice cual es la rama por defecto del remoto.
const ORIGIN_HEAD: &str = "refs/remotes/origin/HEAD";

/// Refs leidas y la rama a la que apunta `origin/HEAD`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedRefs {
    /// Refs que apuntan a un commit.
    pub refs: Vec<GitRef>,
    /// Destino de `refs/remotes/origin/HEAD`, si existe.
    pub remote_default: Option<RefName>,
}

/// Lee la salida de `for-each-ref` con [`REFS_FORMAT`].
///
/// Se saltan las refs simbolicas (los `HEAD` de los remotos no son ramas) y los
/// tags que no acaban en un commit (un tag de un arbol, como `v2.6.11-tree` en
/// Linux).
///
/// # Errors
///
/// [`ParseError`] si una linea no tiene los seis campos o un nombre o un sha no son
/// validos.
pub fn parse_refs(stdout: &[u8]) -> Result<ParsedRefs, ParseError> {
    let text = String::from_utf8_lossy(stdout);
    let mut parsed = ParsedRefs::default();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let fields: Vec<&str> = line.split('\0').collect();
        let [name, kind, object, peeled_kind, peeled_object, symref] = fields.as_slice() else {
            return Err(ParseError(format!("ref line with {} fields", fields.len())));
        };
        if !symref.is_empty() {
            if *name == ORIGIN_HEAD {
                parsed.remote_default = RefName::parse(symref).ok();
            }
            continue;
        }
        let target = match (*kind, *peeled_kind) {
            ("commit", _) => *object,
            ("tag", "commit") => *peeled_object,
            _ => continue,
        };
        parsed.refs.push(GitRef {
            name: RefName::parse(name).map_err(|error| ParseError(error.to_string()))?,
            target: Sha::parse(target).map_err(|error| ParseError(error.to_string()))?,
        });
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests;
