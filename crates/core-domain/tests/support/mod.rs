//! Ayudas compartidas por los tests del dominio: shas legibles y commits en una
//! linea.

#![allow(
    dead_code,
    reason = "each test binary uses a different subset of these helpers"
)]
#![allow(
    clippy::unwrap_used,
    reason = "test helpers fail loudly on purpose; a panic here is a broken fixture"
)]

use ramal_core_domain::graph::GraphLayout;
use ramal_core_domain::{
    CommitNode, CommitRecord, GitRef, GitTime, Head, RefName, RefSnapshot, Sha, ShortSha, Signature,
};

/// Sha legible: el numero en hexadecimal relleno con ceros.
pub fn sha(id: u32) -> Sha {
    Sha::parse(&format!("{id:040x}")).unwrap()
}

/// Nodo con su sha y sus padres por numero.
pub fn node(id: u32, parents: &[u32]) -> CommitNode {
    CommitNode::new(sha(id), parents.iter().copied().map(sha).collect())
}

/// Lista de nodos a partir de pares (id, padres).
pub fn nodes(spec: &[(u32, &[u32])]) -> Vec<CommitNode> {
    spec.iter()
        .map(|(id, parents)| node(*id, parents))
        .collect()
}

/// Registro completo con datos fijos.
pub fn record(id: u32, parents: &[u32]) -> CommitRecord {
    CommitRecord {
        node: node(id, parents),
        short_sha: ShortSha::parse(&format!("{id:07x}")).unwrap(),
        author: Signature {
            name: "Ana".to_owned(),
            email: "ana@example.com".to_owned(),
        },
        authored_at: GitTime::new(1_790_000_000, 120),
        summary: format!("commit {id}"),
    }
}

/// Ref a un commit por numero.
pub fn git_ref(name: &str, target: u32) -> GitRef {
    GitRef {
        name: RefName::parse(name).unwrap(),
        target: sha(target),
    }
}

/// Foto de refs con HEAD en una rama.
pub fn snapshot_on_branch(refs: Vec<GitRef>, branch: &str, target: u32) -> RefSnapshot {
    let head = Head::Branch {
        name: RefName::parse(branch).unwrap(),
        target: Some(sha(target)),
    };
    RefSnapshot::new(refs, head, None)
}

/// Carriles de cada fila.
pub fn lanes(layout: &GraphLayout) -> Vec<usize> {
    layout.rows.iter().map(|row| row.lane).collect()
}
