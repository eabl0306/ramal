//! Nucleo de dominio del cliente de Git: commits, refs, tiempo de git y la
//! disposicion del grafo. Codigo puro: sin IO, sin reloj, sin runtime asincrono y
//! sin serde. La forma del JSON la decide `ramal-protocol` y el mapeo vive en
//! `ramal-engine`.

pub mod commit;
pub mod graph;
pub mod refs;
pub mod repo;
pub mod sha;
pub mod time;

pub use commit::{CommitNode, CommitRecord, Signature};
pub use refs::{GitRef, Head, RefError, RefKind, RefLabel, RefName, RefSnapshot};
pub use repo::{RepoId, RepoRoot};
pub use sha::{Sha, ShaError, ShortSha};
pub use time::{GitTime, TimeError};
