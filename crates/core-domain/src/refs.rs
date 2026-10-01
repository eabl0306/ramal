//! Refs de un repositorio: ramas locales, ramas remotas, tags y el estado de HEAD.
//!
//! Se guardan con el nombre completo (`refs/heads/main`) porque es el unico que no
//! es ambiguo: una rama local y un tag pueden llamarse igual.

mod fingerprint;

use std::collections::HashMap;

use thiserror::Error;

use crate::sha::Sha;

const LOCAL_PREFIX: &str = "refs/heads/";
const REMOTE_PREFIX: &str = "refs/remotes/";
const TAG_PREFIX: &str = "refs/tags/";

/// Fallo al leer el nombre de una ref.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RefError {
    /// No es una rama local, una rama remota ni un tag con nombre.
    #[error("unsupported ref name: {0}")]
    Unsupported(String),
}

/// Que clase de ref es. El orden de las variantes es el orden en que se ensenan
/// las etiquetas de una fila.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RefKind {
    /// `refs/heads/*`.
    LocalBranch,
    /// `refs/remotes/*`.
    RemoteBranch,
    /// `refs/tags/*`.
    Tag,
}

/// Nombre completo y validado de una ref.
///
/// El orden derivado compara primero la clase y despues el nombre: por eso
/// `kind` va antes que `full`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RefName {
    kind: RefKind,
    full: String,
}

impl RefName {
    /// Lee un nombre completo de rama local, rama remota o tag.
    ///
    /// # Errors
    ///
    /// [`RefError::Unsupported`] si el nombre no empieza por `refs/heads/`,
    /// `refs/remotes/` o `refs/tags/`, o si no hay nada despues del prefijo.
    pub fn parse(full: &str) -> Result<Self, RefError> {
        let kind = [
            (LOCAL_PREFIX, RefKind::LocalBranch),
            (REMOTE_PREFIX, RefKind::RemoteBranch),
            (TAG_PREFIX, RefKind::Tag),
        ]
        .into_iter()
        .find_map(|(prefix, kind)| {
            full.strip_prefix(prefix)
                .filter(|rest| !rest.is_empty())
                .map(|_| kind)
        })
        .ok_or_else(|| RefError::Unsupported(full.to_owned()))?;
        Ok(Self {
            full: full.to_owned(),
            kind,
        })
    }

    /// Nombre completo, por ejemplo `refs/remotes/origin/main`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.full
    }

    /// Clase de ref.
    #[must_use]
    pub const fn kind(&self) -> RefKind {
        self.kind
    }

    /// Nombre sin el prefijo de su clase: `main`, `origin/main` o `v1.0`.
    #[must_use]
    pub fn short_name(&self) -> &str {
        let prefix = match self.kind {
            RefKind::LocalBranch => LOCAL_PREFIX,
            RefKind::RemoteBranch => REMOTE_PREFIX,
            RefKind::Tag => TAG_PREFIX,
        };
        self.full.strip_prefix(prefix).unwrap_or(&self.full)
    }
}

/// Una ref y el commit al que apunta (con los tags anotados ya pelados).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRef {
    /// Nombre completo.
    pub name: RefName,
    /// Commit al que apunta.
    pub target: Sha,
}

/// Donde esta HEAD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Head {
    /// HEAD apunta a una rama. `target` es `None` si la rama todavia no tiene
    /// commits (repositorio recien creado).
    Branch {
        /// Rama a la que apunta HEAD.
        name: RefName,
        /// Commit de la rama, si ya existe.
        target: Option<Sha>,
    },
    /// HEAD separado, apuntando directamente a un commit.
    Detached {
        /// Commit al que apunta HEAD.
        target: Sha,
    },
}

/// Etiqueta de una ref en una fila del grafo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefLabel {
    /// Nombre de la ref.
    pub name: RefName,
    /// La ref es la rama a la que apunta HEAD.
    pub is_head: bool,
}

/// Foto de las refs de un repositorio en un momento dado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefSnapshot {
    refs: Vec<GitRef>,
    head: Head,
    remote_default: Option<RefName>,
}

impl RefSnapshot {
    /// Construye la foto. `remote_default` es la rama a la que apunta
    /// `refs/remotes/origin/HEAD`, si existe. Las refs se ordenan por clase y
    /// nombre para que el resultado no dependa del orden de git.
    #[must_use]
    pub fn new(mut refs: Vec<GitRef>, head: Head, remote_default: Option<RefName>) -> Self {
        refs.sort_by(|left, right| left.name.cmp(&right.name));
        Self {
            refs,
            head,
            remote_default,
        }
    }

    /// Refs ordenadas por clase y nombre.
    #[must_use]
    pub fn refs(&self) -> &[GitRef] {
        &self.refs
    }

    /// Estado de HEAD.
    #[must_use]
    pub const fn head(&self) -> &Head {
        &self.head
    }

    /// Commit de HEAD, o `None` si la rama todavia no tiene commits.
    #[must_use]
    pub const fn head_target(&self) -> Option<Sha> {
        match &self.head {
            Head::Branch { target, .. } => *target,
            Head::Detached { target } => Some(*target),
        }
    }

    /// Rama a la que apunta HEAD, o `None` si HEAD esta separado.
    #[must_use]
    pub const fn head_branch(&self) -> Option<&RefName> {
        match &self.head {
            Head::Branch { name, .. } => Some(name),
            Head::Detached { .. } => None,
        }
    }

    /// HEAD apunta a un commit y no a una rama.
    #[must_use]
    pub const fn is_detached(&self) -> bool {
        matches!(self.head, Head::Detached { .. })
    }

    /// El repositorio no tiene ningun commit: HEAD no ha nacido y no hay refs.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.head_target().is_none() && self.refs.is_empty()
    }

    /// Punta de la rama por defecto, que es la que se fija a la izquierda del
    /// grafo. Por orden: la rama local con el nombre al que apunta
    /// `origin/HEAD`, la propia rama remota, `main` y `master`. Se prefiere la
    /// local porque es la que el usuario mueve; si va por detras del remoto, el
    /// remoto aparece como una rama mas que desemboca en ella.
    #[must_use]
    pub fn default_branch_tip(&self) -> Option<Sha> {
        let from_remote = self.remote_default.as_ref().and_then(|remote| {
            let local = remote
                .short_name()
                .split_once('/')
                .map(|(_, branch)| format!("{LOCAL_PREFIX}{branch}"));
            local
                .and_then(|name| self.tip_of(&name))
                .or_else(|| self.tip_of(remote.as_str()))
        });
        from_remote
            .or_else(|| self.tip_of("refs/heads/main"))
            .or_else(|| self.tip_of("refs/heads/master"))
    }

    /// Etiquetas de cada commit en el orden en que se ensenan: la rama de HEAD
    /// primero y despues ramas locales, remotas y tags, cada grupo por nombre.
    #[must_use]
    pub fn labels_by_commit(&self) -> HashMap<Sha, Vec<RefLabel>> {
        let head_branch = self.head_branch();
        let mut labels: HashMap<Sha, Vec<RefLabel>> = HashMap::new();
        for git_ref in &self.refs {
            let is_head = head_branch == Some(&git_ref.name);
            let label = RefLabel {
                name: git_ref.name.clone(),
                is_head,
            };
            let row = labels.entry(git_ref.target).or_default();
            if is_head {
                row.insert(0, label);
            } else {
                row.push(label);
            }
        }
        labels
    }

    fn tip_of(&self, full_name: &str) -> Option<Sha> {
        self.refs
            .iter()
            .find(|git_ref| git_ref.name.as_str() == full_name)
            .map(|git_ref| git_ref.target)
    }
}
