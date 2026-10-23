//! Datos de ejemplo para los tests: shas legibles, commits y versiones de git.

use std::path::PathBuf;

use git_rudder_core_domain::{CommitNode, CommitRecord, GitTime, Sha, ShortSha, Signature};

use crate::port::git_reader::{GitCapabilities, GitVersion, GitVersionInfo};

/// Sha legible: el numero en hexadecimal relleno con ceros.
#[must_use]
pub fn sha(id: u32) -> Sha {
    Sha::parse(&format!("{id:040x}")).unwrap_or_else(|_| unreachable!())
}

/// Commit con datos fijos y los padres por numero.
#[must_use]
pub fn commit(id: u32, parents: &[u32]) -> CommitRecord {
    CommitRecord {
        node: CommitNode::new(sha(id), parents.iter().copied().map(sha).collect()),
        short_sha: ShortSha::parse(&format!("{id:07x}")).unwrap_or_else(|_| unreachable!()),
        author: Signature {
            name: "Ana Ruiz".to_owned(),
            email: "ana@example.com".to_owned(),
        },
        authored_at: GitTime::new(1_790_761_320, 120),
        summary: format!("commit {id}"),
    }
}

/// Git en `/usr/bin/git` con la version dada y sin capacidades extra.
#[must_use]
pub fn version_info(version: GitVersion) -> GitVersionInfo {
    GitVersionInfo {
        path: PathBuf::from("/usr/bin/git"),
        version,
        capabilities: GitCapabilities::default(),
    }
}
