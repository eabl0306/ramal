//! Nucleo de dominio del cliente de Git: commits, refs, tiempo de git, la
//! disposicion del grafo, el estado del arbol de trabajo, diffs, conflictos y la
//! confianza en un repositorio. Codigo puro: sin IO, sin reloj, sin runtime asincrono y
//! sin serde. La forma del JSON la decide `git-rudder-protocol` y el mapeo vive en
//! `git-rudder-engine`.

pub mod blame;
pub mod commit;
pub mod conflict;
pub mod diff;
pub mod graph;
pub mod history;
pub mod operation;
pub mod path;
pub mod refs;
pub mod remote;
pub mod repo;
pub mod sha;
pub mod stash;
pub mod status;
pub mod submodule;
pub mod time;
pub mod trust;

pub use commit::{ChangedFile, CommitDetail, CommitNode, CommitRecord, LineCounts, Signature};
pub use path::{PathError, RepoPath};
pub use refs::{GitRef, Head, RefError, RefKind, RefLabel, RefName, RefSnapshot};
pub use repo::{RepoId, RepoRoot};
pub use sha::{Sha, ShaError, ShortSha};
pub use time::{GitTime, TimeError};
pub use trust::TrustLevel;
