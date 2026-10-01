//! Huella de la foto de refs: igual mientras el historial que ensena el grafo es
//! el mismo y distinta en cuanto cambia cualquier cosa que lo decide.

#![allow(
    unused_crate_dependencies,
    reason = "integration tests only use the crate under test"
)]
#![allow(
    clippy::unwrap_used,
    reason = "tests fail loudly on purpose; a panic here is a failed assertion"
)]

mod support;

use ramal_core_domain::{GitRef, Head, RefName, RefSnapshot};
use support::{git_ref, sha, snapshot_on_branch};

fn main_branch() -> RefName {
    RefName::parse("refs/heads/main").unwrap()
}

fn base_refs() -> Vec<GitRef> {
    vec![
        git_ref("refs/heads/main", 1),
        git_ref("refs/heads/feature", 2),
        git_ref("refs/tags/v1.0", 1),
    ]
}

fn base() -> RefSnapshot {
    snapshot_on_branch(base_refs(), "refs/heads/main", 1)
}

#[test]
fn the_fingerprint_is_stable_hex_that_does_not_depend_on_ref_order() {
    let mut reversed = base_refs();
    reversed.reverse();
    let shuffled = snapshot_on_branch(reversed, "refs/heads/main", 1);

    assert_eq!(base().fingerprint(), shuffled.fingerprint());
    assert_eq!(base().fingerprint().len(), 32);
    assert!(
        base()
            .fingerprint()
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    );
}

#[test]
fn the_fingerprint_does_not_change_between_runs_or_rust_versions() {
    // Valor calculado aparte (FNV-1a de 128 bits sobre "head refs/heads/main -\n
    // default -\n"). Si cambia, la huella ya no es determinista.
    let unborn = RefSnapshot::new(
        vec![],
        Head::Branch {
            name: main_branch(),
            target: None,
        },
        None,
    );

    assert_eq!(unborn.fingerprint(), "0e8017666b8ccf7e67c473e1a67778dc");
}

#[test]
fn moving_adding_or_removing_a_ref_changes_the_fingerprint() {
    let original = base().fingerprint();

    let moved = vec![
        git_ref("refs/heads/main", 1),
        git_ref("refs/heads/feature", 3),
        git_ref("refs/tags/v1.0", 1),
    ];
    let mut added = base_refs();
    added.push(git_ref("refs/remotes/origin/main", 1));
    let mut removed = base_refs();
    removed.pop();

    for refs in [moved, added, removed] {
        let changed = snapshot_on_branch(refs, "refs/heads/main", 1);
        assert_ne!(changed.fingerprint(), original);
    }
}

#[test]
fn moving_head_changes_the_fingerprint_even_on_the_same_commit() {
    let original = base().fingerprint();
    let other_branch = snapshot_on_branch(base_refs(), "refs/heads/feature", 1);
    let detached = RefSnapshot::new(base_refs(), Head::Detached { target: sha(1) }, None);
    let unborn = RefSnapshot::new(
        base_refs(),
        Head::Branch {
            name: RefName::parse("refs/heads/orphan").unwrap(),
            target: None,
        },
        None,
    );

    for changed in [other_branch, detached, unborn] {
        assert_ne!(changed.fingerprint(), original);
    }
}

#[test]
fn the_default_branch_is_part_of_the_fingerprint() {
    let head = Head::Branch {
        name: main_branch(),
        target: Some(sha(1)),
    };
    let remote = RefName::parse("refs/remotes/origin/main").unwrap();
    let with_default = RefSnapshot::new(base_refs(), head, Some(remote));

    assert_ne!(with_default.fingerprint(), base().fingerprint());
}
